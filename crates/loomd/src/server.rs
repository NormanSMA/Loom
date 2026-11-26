use crate::auth::{
    AuthManager, DeviceView, ExchangeResult, PairingTicket, TotpSetup, is_allowed_host,
};
use crate::demo::{
    AgentActivityState, ArtifactPreview, ClaudeHubBridge, HeatPoint, LiveEvent, MissionView,
    ProviderMetric, TelemetryLog, TelemetrySnapshot, build_demo_png_data_uri, demo_diff,
    demo_doctor, demo_missions, demo_providers, demo_skills, demo_telemetry, demo_workspaces,
};
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, Query, Request, State};
use axum::http::{StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use chrono::Utc;
use loom_core::capsule::{CapsuleRequest, compile_capsule};
use loom_core::config::{LoomConfig, default_providers_dir};
use loom_core::exchange::{LoomRunRecord, append_loom_run, default_exchange_dir};
use loom_core::providers::ProviderRegistry;
use loom_core::router::{TaskRole, read_claudehub_quota, route_task};
use loom_core::skills::SkillIndex;
use loom_core::types::Sensitivity;
use loom_core::workspaces::{
    ActiveChatView, WorkspaceInfo, dedup_active_chats, discover_local_workspaces,
    normalize_workspace_key,
};
use loom_drivers::doctor::run_doctor;
use loom_drivers::image_gen::{ImageBackend, ImageGenRequest, generate_image};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock, broadcast, oneshot};
use tower_http::services::{ServeDir, ServeFile};
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub config: LoomConfig,
    pub demo: bool,
    pub started_at: Instant,
    pub auth: Arc<AuthManager>,
    pub missions: Arc<RwLock<Vec<MissionView>>>,
    pub logs: Arc<RwLock<Vec<TelemetryLog>>>,
    pub artifacts: Arc<RwLock<HashMap<String, Vec<ArtifactPreview>>>>,
    pub active_cancels: Arc<Mutex<Vec<oneshot::Sender<()>>>>,
    pub events_tx: broadcast::Sender<LiveEvent>,
    pub http: reqwest::Client,
    pub exchange_dir: PathBuf,
    pub public_url: Option<String>,
}

impl AppState {
    pub fn new(config: LoomConfig, demo: bool) -> Self {
        let (events_tx, _) = broadcast::channel(128);
        let initial_missions = if demo { demo_missions() } else { Vec::new() };
        let mut initial_artifacts: HashMap<String, Vec<ArtifactPreview>> = HashMap::new();
        if demo {
            initial_artifacts.insert(
                "msn-ui-tokens".into(),
                vec![ArtifactPreview {
                    id: "art-demo-1".into(),
                    kind: "image".into(),
                    name: "ui-token-preview.png".into(),
                    mime_type: "image/png".into(),
                    data_uri: build_demo_png_data_uri(),
                    width: Some(64),
                    height: Some(64),
                    created_at: Utc::now().to_rfc3339(),
                }],
            );
        }
        let http = reqwest::Client::builder()
            .timeout(Duration::from_millis(1500))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            config,
            demo,
            started_at: Instant::now(),
            auth: Arc::new(AuthManager::new()),
            missions: Arc::new(RwLock::new(initial_missions)),
            logs: Arc::new(RwLock::new(Vec::new())),
            artifacts: Arc::new(RwLock::new(initial_artifacts)),
            active_cancels: Arc::new(Mutex::new(Vec::new())),
            events_tx,
            http,
            exchange_dir: default_exchange_dir(),
            public_url: None,
        }
    }
}

pub fn build_router(state: AppState, static_dir: Option<PathBuf>) -> Router {
    let api = Router::new()
        .route("/api/health", get(health))
        .route("/api/network", get(network_info))
        .route("/api/doctor", get(doctor))
        .route("/api/providers", get(providers))
        .route("/api/workspaces", get(workspaces))
        .route("/api/skills", get(skills))
        .route("/api/missions", get(list_missions).post(create_mission))
        .route("/api/missions/{id}/diff", get(mission_diff))
        .route("/api/missions/{id}/decision", post(mission_decision))
        .route("/api/missions/{id}/artifacts", get(mission_artifacts))
        .route("/api/images/generate", post(generate_image_route))
        .route("/api/telemetry", get(telemetry))
        .route("/api/claudehub", get(claudehub))
        .route("/api/auth/pair", post(auth_pair))
        .route("/api/auth/exchange", post(auth_exchange))
        .route("/api/auth/totp/status", get(totp_status))
        .route("/api/auth/totp/setup", post(totp_setup))
        .route("/api/auth/totp/verify", post(totp_verify))
        .route("/api/devices", get(list_devices))
        .route("/api/devices/{id}", delete(revoke_device))
        .route("/api/emergency-stop", post(emergency_stop))
        .route("/api/ws", get(ws_handler))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            host_guard_middleware,
        ));

    let app = Router::new().merge(api).with_state(state);

    if let Some(dir) = static_dir
        && dir.is_dir()
    {
        let fallback = ServeFile::new(dir.join("index.html"));
        return app.fallback_service(ServeDir::new(dir).not_found_service(fallback));
    }

    app
}

async fn host_guard_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let host = req
        .headers()
        .get(header::HOST)
        .and_then(|v| v.to_str().ok());
    if !is_allowed_host(host, state.config.remote_enabled) {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(next.run(req).await)
}

async fn health(State(state): State<AppState>) -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "loomd",
        "version": env!("CARGO_PKG_VERSION"),
        "demo": state.demo,
        "uptime_secs": state.started_at.elapsed().as_secs(),
        "port": if state.demo { state.config.demo_port } else { state.config.port },
    }))
}

async fn doctor(State(state): State<AppState>) -> impl IntoResponse {
    if state.demo {
        return Json(demo_doctor()).into_response();
    }
    let Ok(registry) =
        ProviderRegistry::load(&default_providers_dir(), &state.config.omniroute_url)
    else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "no se pudo cargar registro de proveedores" })),
        )
            .into_response();
    };
    Json(run_doctor(&state.config, &registry).await).into_response()
}

async fn providers(State(state): State<AppState>) -> impl IntoResponse {
    if state.demo {
        return Json(demo_providers(&state.config.omniroute_url)).into_response();
    }
    match ProviderRegistry::load(&default_providers_dir(), &state.config.omniroute_url) {
        Ok(reg) => Json(reg.all().to_vec()).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
struct SkillsQuery {
    #[serde(default)]
    query: String,
}

async fn skills(State(state): State<AppState>, Query(q): Query<SkillsQuery>) -> impl IntoResponse {
    if state.demo {
        let needle = q.query.trim().to_ascii_lowercase();
        let list: Vec<_> = demo_skills()
            .into_iter()
            .filter(|s| {
                needle.is_empty()
                    || s.name.to_ascii_lowercase().contains(&needle)
                    || s.description.to_ascii_lowercase().contains(&needle)
            })
            .collect();
        return Json(list);
    }
    let idx = SkillIndex::discover(None);
    let matches: Vec<_> = idx.search(&q.query).into_iter().cloned().collect();
    Json(matches)
}

async fn list_missions(State(state): State<AppState>) -> Json<Vec<MissionView>> {
    Json(state.missions.read().await.clone())
}

#[derive(Debug, Deserialize)]
struct CreateMissionReq {
    title: String,
    #[serde(default = "default_mode")]
    mode: String,
    #[serde(default = "default_effort")]
    effort: String,
    #[serde(default = "default_sensitivity")]
    sensitivity: String,
    #[serde(default)]
    repo_root: Option<String>,
}

fn default_mode() -> String {
    "auto_dag".into()
}
fn default_effort() -> String {
    "normal".into()
}
fn default_sensitivity() -> String {
    "internal".into()
}

async fn create_mission(
    State(state): State<AppState>,
    Json(req): Json<CreateMissionReq>,
) -> impl IntoResponse {
    let title = req.title.trim();
    if title.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "title obligatorio" })),
        )
            .into_response();
    }

    let sensitivity: Sensitivity = serde_json::from_value(Value::String(req.sensitivity.clone()))
        .unwrap_or(Sensitivity::Internal);
    let role = match req.mode.as_str() {
        "image" => TaskRole::ImageGen,
        "direct" => TaskRole::Implementer,
        _ => TaskRole::Architect,
    };

    let registry = ProviderRegistry::load(&default_providers_dir(), &state.config.omniroute_url)
        .unwrap_or_else(|_| {
            ProviderRegistry::load(
                PathBuf::from("__none__").as_path(),
                &state.config.omniroute_url,
            )
            .expect("builtin providers always load")
        });
    let quota = read_claudehub_quota(None);
    let routed = route_task(role, sensitivity, &registry, &quota)
        .map(|d| d.provider_id)
        .unwrap_or_else(|_| "claude".into());

    if let Some(repo) = &req.repo_root {
        let _capsule = compile_capsule(&CapsuleRequest {
            repo_root: PathBuf::from(repo),
            vault_path: state.config.vault_path.clone(),
            project_slug: None,
            objective: title.to_string(),
            allowed_files: Vec::new(),
            verification_cmd: None,
            skill_names: Vec::new(),
        });
    }

    let id = format!("msn-{}", &Uuid::new_v4().simple().to_string()[..8]);
    let now_dt = Utc::now();
    let now = now_dt.to_rfc3339();
    if !state.demo {
        let project = req
            .repo_root
            .as_deref()
            .and_then(|p| std::path::Path::new(p).file_name())
            .and_then(|n| n.to_str())
            .filter(|s| !s.is_empty())
            .unwrap_or("loom")
            .to_string();
        let run_record = LoomRunRecord {
            id: format!("loom:{id}"),
            ts: now_dt.timestamp_millis(),
            session: id.clone(),
            project,
            role: "orchestrator".into(),
            agent: None,
            model: routed.clone(),
            provider: routed.clone(),
            input: 0,
            output: 0,
            cache_read: 0,
            cache_write: 0,
            reasoning: None,
        };
        let _ = append_loom_run(&state.exchange_dir, &run_record);
    }
    let mission = MissionView {
        id: id.clone(),
        title: title.to_string(),
        mode: req.mode,
        effort: req.effort,
        sensitivity: req.sensitivity,
        provider_id: routed.clone(),
        status: "queued".into(),
        state: AgentActivityState::Thinking,
        worktree: format!(".loom/worktrees/{id}"),
        repo_root: req.repo_root,
        gate_passed: None,
        updated_at: now.clone(),
    };

    state.missions.write().await.insert(0, mission.clone());
    push_log(
        &state.logs,
        TelemetryLog {
            ts: now.clone(),
            provider_id: routed.clone(),
            level: "info".into(),
            message: format!("Misión despachada: {title}"),
        },
    )
    .await;
    let _ = state.events_tx.send(LiveEvent {
        kind: "mission_created".into(),
        state: AgentActivityState::Thinking,
        provider_id: routed,
        target: id,
        detail: title.to_string(),
        ts: now,
    });

    (StatusCode::CREATED, Json(mission)).into_response()
}

async fn push_log(logs: &RwLock<Vec<TelemetryLog>>, entry: TelemetryLog) {
    let mut guard = logs.write().await;
    guard.insert(0, entry);
    guard.truncate(30);
}

async fn workspaces(State(state): State<AppState>) -> Json<Vec<WorkspaceInfo>> {
    if state.demo {
        return Json(demo_workspaces());
    }

    let roots: Vec<PathBuf> = std::env::current_dir().into_iter().collect();
    let mut list = tokio::task::spawn_blocking(move || discover_local_workspaces(&roots))
        .await
        .unwrap_or_default();

    let base = state.config.claudehub_url.trim_end_matches('/');
    let act_url = format!("{base}/api/active");
    let ses_url = format!("{base}/api/sessions?range=30d");
    let (act_res, ses_res) = tokio::join!(
        state.http.get(&act_url).send(),
        state.http.get(&ses_url).send()
    );

    let mut index: HashMap<String, usize> = list
        .iter()
        .enumerate()
        .map(|(i, w)| (normalize_workspace_key(std::path::Path::new(&w.path)), i))
        .collect();

    if let Ok(resp) = act_res
        && let Ok(val) = resp.json::<Value>().await
        && let Some(chats) = val.get("chats").and_then(Value::as_array)
    {
        for c in chats {
            let Some(cwd) = c.get("cwd").and_then(Value::as_str) else {
                continue;
            };
            let cwd_path = std::path::Path::new(cwd);
            if !cwd_path.is_dir() {
                continue;
            }
            let source = c
                .get("source")
                .and_then(Value::as_str)
                .unwrap_or("claude")
                .to_string();
            let last_ts = c.get("lastTs").and_then(Value::as_i64).unwrap_or(0);
            let idx = ensure_workspace(&mut list, &mut index, cwd_path, &source, last_ts);
            list[idx].active_chats.push(ActiveChatView {
                source,
                session: c
                    .get("session")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                title: c
                    .get("title")
                    .and_then(Value::as_str)
                    .unwrap_or("Sesión activa")
                    .to_string(),
                model: c
                    .get("model")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown")
                    .to_string(),
                working: c.get("working").and_then(Value::as_bool).unwrap_or(false),
                last_ts,
            });
        }
    }

    if let Ok(resp) = ses_res
        && let Ok(sessions) = resp.json::<Vec<Value>>().await
    {
        for s in sessions.iter().take(25) {
            let Some(cwd) = s.get("cwd").and_then(Value::as_str) else {
                continue;
            };
            let cwd_path = std::path::Path::new(cwd);
            if !cwd_path.is_dir() {
                continue;
            }
            let source = s.get("source").and_then(Value::as_str).unwrap_or("claude");
            let last_ts = s.get("last").and_then(Value::as_i64).unwrap_or(0);
            ensure_workspace(&mut list, &mut index, cwd_path, source, last_ts);
        }
    }

    for ws in &mut list {
        dedup_active_chats(&mut ws.active_chats);
    }

    list.sort_by(|a, b| {
        b.active_chats
            .iter()
            .any(|c| c.working)
            .cmp(&a.active_chats.iter().any(|c| c.working))
            .then_with(|| b.active_chats.len().cmp(&a.active_chats.len()))
            .then_with(|| b.last_ts.cmp(&a.last_ts))
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    list.truncate(40);
    Json(list)
}

fn ensure_workspace(
    list: &mut Vec<WorkspaceInfo>,
    index: &mut HashMap<String, usize>,
    path: &std::path::Path,
    source: &str,
    ts: i64,
) -> usize {
    let key = normalize_workspace_key(path);
    if let Some(&idx) = index.get(&key) {
        if !list[idx].sources.iter().any(|s| s == source) {
            list[idx].sources.push(source.to_string());
        }
        if ts > list[idx].last_ts {
            list[idx].last_ts = ts;
        }
        return idx;
    }
    let display = path.to_string_lossy().replace('\\', "/");
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(&display)
        .to_string();
    let idx = list.len();
    list.push(WorkspaceInfo {
        name,
        path: display,
        is_git_repo: path.join(".git").exists(),
        sources: vec![source.to_string()],
        active_chats: Vec::new(),
        last_ts: ts,
    });
    index.insert(key, idx);
    idx
}

#[derive(Debug, Serialize)]
struct MissionDiffResp {
    mission_id: String,
    summary: String,
    diff: String,
}

async fn mission_diff(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    if state.demo {
        let (summary, diff) = demo_diff(&id);
        return Json(MissionDiffResp {
            mission_id: id,
            summary,
            diff,
        })
        .into_response();
    }

    let repo_root = state
        .missions
        .read()
        .await
        .iter()
        .find(|m| m.id == id)
        .and_then(|m| m.repo_root.clone())
        .filter(|p| std::path::Path::new(p).is_dir())
        .unwrap_or_else(|| ".".to_string());

    let out = tokio::task::spawn_blocking(move || read_git_diff(&repo_root))
        .await
        .unwrap_or_else(|_| {
            (
                "Sin datos".into(),
                "Sin cambios pendientes en worktree.".into(),
            )
        });

    Json(MissionDiffResp {
        mission_id: id,
        summary: out.0,
        diff: out.1,
    })
    .into_response()
}

fn read_git_diff(repo_root: &str) -> (String, String) {
    let stat_out = std::process::Command::new("git")
        .args(["-C", repo_root, "diff", "--stat"])
        .output();
    let diff_out = std::process::Command::new("git")
        .args(["-C", repo_root, "diff"])
        .output();

    let summary = stat_out
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Worktree limpio".into());

    let mut diff = diff_out
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    if diff.trim().is_empty() {
        diff = "Sin cambios pendientes en worktree.".into();
    } else if diff.len() > 16_384 {
        diff.truncate(16_384);
        diff.push_str("\n... (diff truncado a 16 KiB)");
    }

    (summary, diff)
}

#[derive(Debug, Deserialize)]
struct DecisionReq {
    decision: String,
}

async fn mission_decision(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<DecisionReq>,
) -> impl IntoResponse {
    let approved = match req.decision.trim().to_ascii_lowercase().as_str() {
        "approve" => true,
        "reject" => false,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "decision debe ser approve o reject" })),
            )
                .into_response();
        }
    };

    let now = Utc::now().to_rfc3339();
    let mut updated: Option<MissionView> = None;
    {
        let mut guard = state.missions.write().await;
        if let Some(m) = guard.iter_mut().find(|m| m.id == id) {
            m.status = if approved {
                "approved".into()
            } else {
                "rejected".into()
            };
            m.gate_passed = Some(approved);
            m.state = AgentActivityState::Idle;
            m.updated_at = now.clone();
            updated = Some(m.clone());
        }
    }

    let Some(mission) = updated else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "mision no encontrada" })),
        )
            .into_response();
    };

    push_log(
        &state.logs,
        TelemetryLog {
            ts: now.clone(),
            provider_id: mission.provider_id.clone(),
            level: if approved { "info" } else { "warn" }.into(),
            message: format!(
                "{}: {}",
                if approved {
                    "Misión aprobada"
                } else {
                    "Misión rechazada"
                },
                mission.title
            ),
        },
    )
    .await;

    let _ = state.events_tx.send(LiveEvent {
        kind: "agent_activity".into(),
        state: AgentActivityState::Idle,
        provider_id: mission.provider_id.clone(),
        target: mission.id.clone(),
        detail: if approved {
            "Cambio aprobado por operador".into()
        } else {
            "Cambio rechazado por operador".into()
        },
        ts: now,
    });

    (StatusCode::OK, Json(mission)).into_response()
}

async fn mission_artifacts(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<Vec<ArtifactPreview>> {
    let guard = state.artifacts.read().await;
    Json(guard.get(&id).cloned().unwrap_or_default())
}

#[derive(Debug, Deserialize)]
struct GenerateImageRouteReq {
    prompt: String,
    #[serde(default)]
    style_context: Option<String>,
    #[serde(default)]
    mission_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct GenerateImageRouteResp {
    backend_used: String,
    mime_type: String,
    width: u32,
    height: u32,
    data_uri: String,
}

async fn generate_image_route(
    State(state): State<AppState>,
    Json(req): Json<GenerateImageRouteReq>,
) -> impl IntoResponse {
    let prompt = req.prompt.trim();
    if prompt.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "prompt obligatorio" })),
        )
            .into_response();
    }

    let now = Utc::now().to_rfc3339();
    let result = if state.demo {
        Ok(GenerateImageRouteResp {
            backend_used: "demo_raster".into(),
            mime_type: "image/png".into(),
            width: 64,
            height: 64,
            data_uri: build_demo_png_data_uri(),
        })
    } else {
        let out_dir = PathBuf::from(".loom").join("artifacts");
        let file_name = format!("gen-{}.png", &Uuid::new_v4().simple().to_string()[..8]);
        let out_path = out_dir.join(file_name);
        match generate_image(&ImageGenRequest {
            prompt: prompt.to_string(),
            style_context: req.style_context,
            output_path: out_path.clone(),
            backend: ImageBackend::Auto,
            custom_endpoint: None,
        })
        .await
        {
            Ok(generated) => {
                let bytes = std::fs::read(&generated.output_path).unwrap_or_default();
                let mime = if generated.validation.format == "png" {
                    "image/png"
                } else {
                    "image/webp"
                };
                Ok(GenerateImageRouteResp {
                    backend_used: generated.backend_used,
                    mime_type: mime.into(),
                    width: generated.validation.width.unwrap_or(64),
                    height: generated.validation.height.unwrap_or(64),
                    data_uri: format!("data:{mime};base64,{}", STANDARD.encode(bytes)),
                })
            }
            Err(err) => Err(err.to_string()),
        }
    };

    match result {
        Ok(resp) => {
            if let Some(mission_id) = req.mission_id.as_deref().filter(|s| !s.is_empty()) {
                let preview = ArtifactPreview {
                    id: format!("art-{}", &Uuid::new_v4().simple().to_string()[..8]),
                    kind: "image".into(),
                    name: prompt.chars().take(36).collect(),
                    mime_type: resp.mime_type.clone(),
                    data_uri: resp.data_uri.clone(),
                    width: Some(resp.width),
                    height: Some(resp.height),
                    created_at: now,
                };
                state
                    .artifacts
                    .write()
                    .await
                    .entry(mission_id.to_string())
                    .or_default()
                    .insert(0, preview);
            }
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(msg) => (StatusCode::BAD_REQUEST, Json(json!({ "error": msg }))).into_response(),
    }
}

async fn telemetry(State(state): State<AppState>) -> Json<TelemetrySnapshot> {
    if state.demo {
        return Json(demo_telemetry());
    }

    let missions = state.missions.read().await.clone();
    let count = missions.len() as u32;
    let decided: Vec<&MissionView> = missions
        .iter()
        .filter(|m| m.gate_passed.is_some())
        .collect();
    let gate_pass_rate = if decided.is_empty() {
        100.0
    } else {
        let passed = decided
            .iter()
            .filter(|m| m.gate_passed == Some(true))
            .count() as f64;
        (passed / decided.len() as f64) * 100.0
    };

    let base = state.config.claudehub_url.trim_end_matches('/');
    let sum_url = format!("{base}/api/summary?range=all");
    let src_url = format!("{base}/api/sources");
    let (sum_res, src_res) = tokio::join!(
        state.http.get(&sum_url).send(),
        state.http.get(&src_url).send()
    );

    let mut total_tokens = 0_u64;
    let mut day_counts: HashMap<String, u64> = HashMap::new();
    if let Ok(resp) = sum_res
        && let Ok(val) = resp.json::<Value>().await
    {
        total_tokens = val
            .pointer("/tokens/total")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        if let Some(arr) = val.get("heatmap").and_then(Value::as_array) {
            for pt in arr {
                if let (Some(day), Some(tokens)) = (
                    pt.get("day").and_then(Value::as_str),
                    pt.get("tokens").and_then(Value::as_u64),
                ) {
                    day_counts.insert(day.to_string(), tokens);
                }
            }
        }
    }

    let max_day = day_counts.values().copied().max().unwrap_or(1).max(1);
    let today = Utc::now().date_naive();
    let mut heatmap = Vec::with_capacity(98);
    for i in (0_i64..98).rev() {
        let day = (today - chrono::Duration::days(i))
            .format("%Y-%m-%d")
            .to_string();
        let raw = day_counts.get(&day).copied().unwrap_or(0);
        let count = if raw == 0 {
            0
        } else {
            (((raw as f64 / max_day as f64) * 9.0).ceil() as u32).clamp(1, 10)
        };
        heatmap.push(HeatPoint { day, count });
    }

    let mut by_provider = Vec::new();
    if let Ok(resp) = src_res
        && let Ok(map) = resp.json::<Value>().await
        && let Some(obj) = map.as_object()
    {
        let labels = [
            ("claude", "Claude Code"),
            ("codex", "Codex CLI"),
            ("gemini", "Gemini CLI"),
            ("antigravity", "Antigravity"),
            ("opencode", "OpenCode"),
            ("omniroute", "OmniRoute"),
            ("loom", "Loom"),
        ];
        for (id, label) in labels {
            if let Some(src) = obj.get(id) {
                let records = src.get("records").and_then(Value::as_u64).unwrap_or(0) as u32;
                let ok = src.get("ok").and_then(Value::as_bool).unwrap_or(false);
                if records > 0 || ok {
                    by_provider.push(ProviderMetric {
                        provider_id: id.into(),
                        name: label.into(),
                        calls: records,
                        success_rate: if ok { 100.0 } else { 0.0 },
                        p95_ms: 0,
                        errors_429: 0,
                        errors_403: 0,
                        tokens: 0,
                    });
                }
            }
        }
    }

    let logs = state.logs.read().await.clone();
    Json(TelemetrySnapshot {
        total_tokens,
        active_agents: count,
        gate_pass_rate,
        avg_latency_ms: 0,
        heatmap,
        by_provider,
        logs,
    })
}

async fn claudehub(State(state): State<AppState>) -> Json<ClaudeHubBridge> {
    if state.demo {
        return Json(ClaudeHubBridge {
            connected: true,
            version: Some("0.12.1".into()),
            url: state.config.claudehub_url.clone(),
            level: Some(2),
            level_label: Some("Flujo".into()),
            today_fresh: Some(1_420_000),
        });
    }

    let base = state.config.claudehub_url.trim_end_matches('/');
    let upd_url = format!("{base}/api/update");
    let act_url = format!("{base}/api/active");

    let Ok(upd_resp) = state.http.get(&upd_url).send().await else {
        return Json(ClaudeHubBridge {
            connected: false,
            version: None,
            url: state.config.claudehub_url.clone(),
            level: None,
            level_label: None,
            today_fresh: None,
        });
    };

    let version = upd_resp
        .json::<Value>()
        .await
        .ok()
        .and_then(|v| v.get("current").and_then(Value::as_str).map(str::to_string));

    let active_json = match state.http.get(&act_url).send().await {
        Ok(resp) => resp.json::<Value>().await.ok(),
        Err(_) => None,
    };

    let level = active_json
        .as_ref()
        .and_then(|v| v.pointer("/level/level").and_then(Value::as_u64))
        .map(|n| n as u8);
    let level_label = active_json
        .as_ref()
        .and_then(|v| v.pointer("/level/label").and_then(Value::as_str))
        .map(str::to_string);
    let today_fresh = active_json
        .as_ref()
        .and_then(|v| v.get("todayFresh").and_then(Value::as_u64));

    Json(ClaudeHubBridge {
        connected: true,
        version,
        url: state.config.claudehub_url.clone(),
        level,
        level_label,
        today_fresh,
    })
}

#[derive(Debug, Serialize)]
struct NetworkInfo {
    local_url: String,
    public_url: Option<String>,
    lan_ip: Option<String>,
    remote_enabled: bool,
}

fn detect_lan_ipv4() -> Option<String> {
    let sock = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    sock.connect("8.8.8.8:80").ok()?;
    match sock.local_addr().ok()?.ip() {
        std::net::IpAddr::V4(ip) if !ip.is_loopback() => Some(ip.to_string()),
        _ => None,
    }
}

async fn network_info(State(state): State<AppState>) -> Json<NetworkInfo> {
    let port = if state.demo {
        state.config.demo_port
    } else {
        state.config.port
    };
    let local_url = format!("http://{}:{port}", state.config.bind);
    let lan_ip = detect_lan_ipv4();
    let public_url = state.public_url.clone().or_else(|| {
        if state.config.remote_enabled {
            lan_ip.as_ref().map(|ip| format!("http://{ip}:{port}"))
        } else {
            None
        }
    });
    Json(NetworkInfo {
        local_url,
        public_url,
        lan_ip,
        remote_enabled: state.config.remote_enabled,
    })
}

#[derive(Debug, Default, Deserialize)]
struct PairReq {
    base_url: Option<String>,
    ttl_secs: Option<i64>,
}

async fn auth_pair(
    State(state): State<AppState>,
    body: Option<Json<PairReq>>,
) -> Json<PairingTicket> {
    let default_url = state.public_url.clone().unwrap_or_else(|| {
        format!(
            "http://{}:{}",
            state.config.bind,
            if state.demo {
                state.config.demo_port
            } else {
                state.config.port
            }
        )
    });
    let (req_base, ttl_secs) = match body {
        Some(Json(req)) => (req.base_url, req.ttl_secs),
        None => (None, None),
    };
    let base = req_base
        .map(|s| s.trim().trim_end_matches('/').to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or(default_url);
    let ttl = chrono::Duration::seconds(ttl_secs.unwrap_or(120).clamp(30, 600));
    Json(state.auth.create_pairing_with_ttl(&base, ttl).await)
}

#[derive(Debug, Deserialize)]
struct ExchangeReq {
    code: String,
    #[serde(default)]
    label: String,
}

async fn auth_exchange(
    State(state): State<AppState>,
    Json(req): Json<ExchangeReq>,
) -> Result<Json<ExchangeResult>, StatusCode> {
    state
        .auth
        .exchange_pairing(&req.code, &req.label)
        .await
        .map(Json)
        .ok_or(StatusCode::UNAUTHORIZED)
}

#[derive(Debug, Serialize)]
struct TotpStatusResp {
    enrolled: bool,
    remote_enabled: bool,
    bind: String,
}

async fn totp_status(State(state): State<AppState>) -> Json<TotpStatusResp> {
    Json(TotpStatusResp {
        enrolled: state.auth.totp_enrolled().await,
        remote_enabled: state.config.remote_enabled,
        bind: state.config.bind.clone(),
    })
}

async fn totp_setup(State(state): State<AppState>) -> Result<Json<TotpSetup>, StatusCode> {
    state
        .auth
        .setup_totp("owner@loom.local")
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[derive(Debug, Deserialize)]
struct TotpVerifyReq {
    code: String,
}

async fn totp_verify(
    State(state): State<AppState>,
    Json(req): Json<TotpVerifyReq>,
) -> Result<Json<Value>, StatusCode> {
    if state.auth.verify_and_enable_totp(&req.code).await {
        Ok(Json(json!({ "verified": true, "enrolled": true })))
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

async fn list_devices(State(state): State<AppState>) -> Json<Vec<DeviceView>> {
    Json(state.auth.list_devices().await)
}

async fn revoke_device(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    if state.auth.revoke_device(&id).await {
        Ok(Json(json!({ "revoked": true, "id": id })))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

#[derive(Debug, Serialize)]
struct StopResp {
    stopped: usize,
}

async fn emergency_stop(State(state): State<AppState>) -> Json<StopResp> {
    let senders: Vec<_> = {
        let mut guard = state.active_cancels.lock().await;
        std::mem::take(&mut *guard)
    };
    let mut stopped = 0;
    for tx in senders {
        if tx.send(()).is_ok() {
            stopped += 1;
        }
    }
    let _ = state.events_tx.send(LiveEvent {
        kind: "emergency_stop".into(),
        state: AgentActivityState::Idle,
        provider_id: "loomd".into(),
        target: "all".into(),
        detail: format!("Detenidos {stopped} procesos activos"),
        ts: Utc::now().to_rfc3339(),
    });
    Json(StopResp { stopped })
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, state))
}

async fn handle_ws(mut socket: WebSocket, state: AppState) {
    let mut rx = state.events_tx.subscribe();
    let mut heartbeat = tokio::time::interval(Duration::from_secs(15));

    loop {
        tokio::select! {
            _ = heartbeat.tick() => {
                let ping_msg = json!({ "kind": "heartbeat", "ts": Utc::now().to_rfc3339() }).to_string();
                if socket.send(Message::Text(ping_msg.into())).await.is_err() {
                    break;
                }
            }
            ev = rx.recv() => {
                match ev {
                    Ok(event) => {
                        if let Ok(text) = serde_json::to_string(&event)
                            && socket.send(Message::Text(text.into())).await.is_err()
                        {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            incoming = socket.recv() => {
                match incoming {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(_)) => {}
                    Some(Err(_)) => break,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use tower::ServiceExt;

    fn test_app() -> (Router, AppState) {
        let state = AppState::new(LoomConfig::default(), true);
        let router = build_router(state.clone(), None);
        (router, state)
    }

    #[tokio::test]
    async fn enforces_host_guard_and_creates_missions() {
        let (app, _) = test_app();

        let bad = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .header("host", "evil.example:4320")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(bad.status(), StatusCode::FORBIDDEN);

        let ok = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .header("host", "127.0.0.1:4321")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(ok.status(), StatusCode::OK);

        let created = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/missions")
                    .header("host", "127.0.0.1:4321")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"title":"Auditar ruta","mode":"direct","sensitivity":"confidential"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(created.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn handles_pairing_device_revocation_and_emergency_stop() {
        let (app, state) = test_app();
        let (tx, rx) = oneshot::channel();
        state.active_cancels.lock().await.push(tx);

        let pair_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/pair")
                    .header("host", "127.0.0.1:4321")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(pair_resp.status(), StatusCode::OK);
        let pair_bytes = axum::body::to_bytes(pair_resp.into_body(), 65536)
            .await
            .unwrap();
        let ticket: PairingTicket = serde_json::from_slice(&pair_bytes).unwrap();

        let ex_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/exchange")
                    .header("host", "127.0.0.1:4321")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({ "code": ticket.code, "label": "Pixel 9" }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(ex_resp.status(), StatusCode::OK);
        let ex_bytes = axum::body::to_bytes(ex_resp.into_body(), 4096)
            .await
            .unwrap();
        let exchanged: ExchangeResult = serde_json::from_slice(&ex_bytes).unwrap();

        let rev_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/api/devices/{}", exchanged.device.id))
                    .header("host", "127.0.0.1:4321")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(rev_resp.status(), StatusCode::OK);

        let stop_resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/emergency-stop")
                    .header("host", "127.0.0.1:4321")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(stop_resp.status(), StatusCode::OK);
        assert!(rx.await.is_ok());
    }

    #[tokio::test]
    async fn emits_claudehub_exchange_record_in_real_mode() {
        let dir = std::env::temp_dir().join(format!("loomd-ex-{}", Uuid::new_v4()));
        let mut state = AppState::new(LoomConfig::default(), false);
        state.exchange_dir = dir.clone();
        let app = build_router(state, None);

        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/missions")
                    .header("host", "127.0.0.1:4320")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"title":"Puente exchange","mode":"direct","sensitivity":"internal","repo_root":"C:/Proyectos/Loom"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        let raw = std::fs::read_to_string(dir.join("loom-runs.jsonl")).unwrap();
        let rec: LoomRunRecord = serde_json::from_str(raw.lines().next().unwrap()).unwrap();
        assert_eq!(rec.project, "Loom");
        assert_eq!(rec.role, "orchestrator");
        assert!(rec.id.starts_with("loom:msn-"));
        assert!(!raw.contains("Proyectos"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn enrolls_and_verifies_totp_via_http() {
        let (app, _) = test_app();

        let status_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/auth/totp/status")
                    .header("host", "127.0.0.1:4321")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(status_resp.status(), StatusCode::OK);

        let setup_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/totp/setup")
                    .header("host", "127.0.0.1:4321")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(setup_resp.status(), StatusCode::OK);
        let setup_bytes = axum::body::to_bytes(setup_resp.into_body(), 65536)
            .await
            .unwrap();
        let setup: TotpSetup = serde_json::from_slice(&setup_bytes).unwrap();
        let code = totp_rs::TOTP::from_url(&setup.otpauth_uri)
            .unwrap()
            .generate_current()
            .unwrap();

        let bad_verify = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/totp/verify")
                    .header("host", "127.0.0.1:4321")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"code":"000000"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(bad_verify.status(), StatusCode::UNAUTHORIZED);

        let ok_verify = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/totp/verify")
                    .header("host", "127.0.0.1:4321")
                    .header("content-type", "application/json")
                    .body(Body::from(json!({ "code": code }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(ok_verify.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn reports_network_info_and_generates_public_qr_pair() {
        let cfg = LoomConfig {
            remote_enabled: true,
            ..Default::default()
        };
        let mut state = AppState::new(cfg, false);
        state.public_url = Some("https://loom-demo.trycloudflare.com".into());
        let app = build_router(state, None);

        let net_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/network")
                    .header("host", "loom-demo.trycloudflare.com")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(net_resp.status(), StatusCode::OK);
        let net_bytes = axum::body::to_bytes(net_resp.into_body(), 4096)
            .await
            .unwrap();
        let net: Value = serde_json::from_slice(&net_bytes).unwrap();
        assert_eq!(
            net.get("public_url").and_then(Value::as_str),
            Some("https://loom-demo.trycloudflare.com")
        );

        let pair_resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/pair")
                    .header("host", "127.0.0.1:4320")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"ttl_secs":600}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(pair_resp.status(), StatusCode::OK);
        let pair_bytes = axum::body::to_bytes(pair_resp.into_body(), 65536)
            .await
            .unwrap();
        let ticket: PairingTicket = serde_json::from_slice(&pair_bytes).unwrap();
        assert!(
            ticket
                .qr_uri
                .starts_with("https://loom-demo.trycloudflare.com/?pair=lm_")
        );
        assert!(ticket.qr_svg.contains("<svg"));
    }

    #[tokio::test]
    async fn serves_workspaces_diffs_decisions_and_generated_images() {
        let (app, _) = test_app();

        let ws_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/workspaces")
                    .header("host", "127.0.0.1:4321")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(ws_resp.status(), StatusCode::OK);
        let ws_bytes = axum::body::to_bytes(ws_resp.into_body(), 32768)
            .await
            .unwrap();
        let ws: Vec<WorkspaceInfo> = serde_json::from_slice(&ws_bytes).unwrap();
        assert!(!ws.is_empty());

        let diff_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/missions/msn-auth-qr/diff")
                    .header("host", "127.0.0.1:4321")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(diff_resp.status(), StatusCode::OK);

        let dec_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/missions/msn-auth-qr/decision")
                    .header("host", "127.0.0.1:4321")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"decision":"approve"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(dec_resp.status(), StatusCode::OK);
        let dec_bytes = axum::body::to_bytes(dec_resp.into_body(), 8192)
            .await
            .unwrap();
        let dec_mission: MissionView = serde_json::from_slice(&dec_bytes).unwrap();
        assert_eq!(dec_mission.status, "approved");

        let img_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/images/generate")
                    .header("host", "127.0.0.1:4321")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"prompt":"Icono vector","mission_id":"msn-auth-qr"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(img_resp.status(), StatusCode::OK);
        let img_bytes = axum::body::to_bytes(img_resp.into_body(), 65536)
            .await
            .unwrap();
        let img_val: Value = serde_json::from_slice(&img_bytes).unwrap();
        let data_uri = img_val.get("data_uri").and_then(Value::as_str).unwrap();
        assert!(data_uri.starts_with("data:image/png;base64,"));
        let decoded = STANDARD
            .decode(data_uri.trim_start_matches("data:image/png;base64,"))
            .unwrap();
        assert!(decoded.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]));

        let art_resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/missions/msn-auth-qr/artifacts")
                    .header("host", "127.0.0.1:4321")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(art_resp.status(), StatusCode::OK);
        let art_bytes = axum::body::to_bytes(art_resp.into_body(), 65536)
            .await
            .unwrap();
        let arts: Vec<ArtifactPreview> = serde_json::from_slice(&art_bytes).unwrap();
        assert_eq!(arts.len(), 1);
    }
}
