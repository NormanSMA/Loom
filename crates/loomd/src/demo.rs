use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use chrono::Utc;
use loom_core::providers::{ProviderManifest, ProviderRegistry};
use loom_core::skills::SkillEntry;
use loom_core::types::DoctorStatus;
use loom_core::workspaces::{ActiveChatView, WorkspaceInfo};
use loom_drivers::doctor::{CheckItem, DoctorReport};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentActivityState {
    Thinking,
    Reading,
    Analysing,
    Debugging,
    Creating,
    Idle,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiveEvent {
    pub kind: String,
    pub state: AgentActivityState,
    pub provider_id: String,
    pub target: String,
    pub detail: String,
    pub ts: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissionView {
    pub id: String,
    pub title: String,
    pub mode: String,
    pub effort: String,
    pub sensitivity: String,
    pub provider_id: String,
    pub status: String,
    pub state: AgentActivityState,
    pub worktree: String,
    #[serde(default)]
    pub repo_root: Option<String>,
    pub gate_passed: Option<bool>,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactPreview {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub mime_type: String,
    pub data_uri: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeatPoint {
    pub day: String,
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProviderMetric {
    pub provider_id: String,
    pub name: String,
    pub calls: u32,
    pub success_rate: f64,
    pub p95_ms: u32,
    pub errors_429: u32,
    pub errors_403: u32,
    pub tokens: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TelemetryLog {
    pub ts: String,
    pub provider_id: String,
    pub level: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TelemetrySnapshot {
    pub total_tokens: u64,
    pub active_agents: u32,
    pub gate_pass_rate: f64,
    pub avg_latency_ms: u32,
    pub heatmap: Vec<HeatPoint>,
    pub by_provider: Vec<ProviderMetric>,
    pub logs: Vec<TelemetryLog>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClaudeHubBridge {
    pub connected: bool,
    pub version: Option<String>,
    pub url: String,
    pub level: Option<u8>,
    pub level_label: Option<String>,
    pub today_fresh: Option<u64>,
}

/// Genera un PNG RGBA valido sin comprimir (zlib store) para vista previa en modo demo.
pub fn build_demo_png_bytes(width: u32, height: u32) -> Vec<u8> {
    let mut raw = Vec::with_capacity(((width * 4 + 1) * height) as usize);
    for y in 0..height {
        raw.push(0); // filter none
        for x in 0..width {
            let edge = x < 2 || y < 2 || x + 2 >= width || y + 2 >= height;
            let checker = ((x / 8) + (y / 8)) % 2 == 0;
            if edge {
                raw.extend_from_slice(&[59, 130, 246, 255]); // accent-blue
            } else if checker {
                raw.extend_from_slice(&[18, 18, 22, 255]);
            } else {
                raw.extend_from_slice(&[34, 197, 94, 220]);
            }
        }
    }

    let mut zlib = Vec::with_capacity(raw.len() + 16);
    zlib.push(0x78);
    zlib.push(0x01);
    let len = raw.len() as u16;
    zlib.push(0x01); // final stored block
    zlib.extend_from_slice(&len.to_le_bytes());
    zlib.extend_from_slice(&(!len).to_le_bytes());
    zlib.extend_from_slice(&raw);
    zlib.extend_from_slice(&adler32(&raw).to_be_bytes());

    let mut png = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.push(8); // bit depth
    ihdr.push(6); // RGBA
    ihdr.extend_from_slice(&[0, 0, 0]);
    write_png_chunk(&mut png, b"IHDR", &ihdr);
    write_png_chunk(&mut png, b"IDAT", &zlib);
    write_png_chunk(&mut png, b"IEND", &[]);
    png
}

pub fn build_demo_png_data_uri() -> String {
    let bytes = build_demo_png_bytes(64, 64);
    format!("data:image/png;base64,{}", STANDARD.encode(bytes))
}

fn write_png_chunk(out: &mut Vec<u8>, tag: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(tag);
    out.extend_from_slice(data);
    let mut crc_input = Vec::with_capacity(4 + data.len());
    crc_input.extend_from_slice(tag);
    crc_input.extend_from_slice(data);
    out.extend_from_slice(&crc32(&crc_input).to_be_bytes());
}

fn adler32(data: &[u8]) -> u32 {
    let mut s1 = 1_u32;
    let mut s2 = 0_u32;
    for &b in data {
        s1 = (s1 + u32::from(b)) % 65_521;
        s2 = (s2 + s1) % 65_521;
    }
    (s2 << 16) | s1
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFF_u32;
    for &b in data {
        crc ^= u32::from(b);
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

pub fn demo_doctor() -> DoctorReport {
    DoctorReport::from_checks(vec![
        CheckItem {
            id: "claudehub".into(),
            name: "ClaudeHub".into(),
            category: "companion".into(),
            status: DoctorStatus::Ok,
            version: Some("0.12.1".into()),
            location: Some("http://127.0.0.1:4317".into()),
            latency_ms: 4,
            warnings: vec![],
        },
        CheckItem {
            id: "git".into(),
            name: "Git".into(),
            category: "core".into(),
            status: DoctorStatus::Ok,
            version: Some("git version 2.55.0".into()),
            location: Some("/usr/bin/git".into()),
            latency_ms: 12,
            warnings: vec![],
        },
        CheckItem {
            id: "claude".into(),
            name: "Claude Code".into(),
            category: "provider".into(),
            status: DoctorStatus::Ok,
            version: Some("2.1.312".into()),
            location: Some("claude".into()),
            latency_ms: 118,
            warnings: vec![],
        },
        CheckItem {
            id: "codex".into(),
            name: "Codex CLI".into(),
            category: "provider".into(),
            status: DoctorStatus::Ok,
            version: Some("0.167.0".into()),
            location: Some("codex".into()),
            latency_ms: 340,
            warnings: vec![],
        },
        CheckItem {
            id: "gemini".into(),
            name: "Gemini CLI".into(),
            category: "provider".into(),
            status: DoctorStatus::Ok,
            version: Some("0.65.0".into()),
            location: Some("gemini".into()),
            latency_ms: 420,
            warnings: vec![],
        },
        CheckItem {
            id: "opencode".into(),
            name: "OpenCode".into(),
            category: "provider".into(),
            status: DoctorStatus::Ok,
            version: Some("v2.1.4".into()),
            location: Some("opencode".into()),
            latency_ms: 195,
            warnings: vec![],
        },
        CheckItem {
            id: "omniroute".into(),
            name: "OmniRoute".into(),
            category: "provider".into(),
            status: DoctorStatus::Ok,
            version: Some("HTTP 200".into()),
            location: Some("http://127.0.0.1:20128".into()),
            latency_ms: 9,
            warnings: vec![],
        },
    ])
}

pub fn demo_providers(omniroute_url: &str) -> Vec<ProviderManifest> {
    ProviderRegistry::load(Path::new("__demo__"), omniroute_url)
        .map(|r| r.all().to_vec())
        .unwrap_or_default()
}

pub fn demo_workspaces() -> Vec<WorkspaceInfo> {
    let now = Utc::now().timestamp_millis();
    vec![
        WorkspaceInfo {
            name: "Loom".into(),
            path: "C:/Proyectos/Loom".into(),
            is_git_repo: true,
            sources: vec!["claude".into(), "codex".into(), "git".into()],
            active_chats: vec![ActiveChatView {
                source: "claude".into(),
                session: "msn-auth-qr".into(),
                title: "Emparejamiento QR y controles 375px".into(),
                model: "Sonnet 5.5".into(),
                working: true,
                last_ts: now - 15_000,
            }],
            last_ts: now - 15_000,
        },
        WorkspaceInfo {
            name: "ClaudeHub".into(),
            path: "C:/Proyectos/ClaudeHub".into(),
            is_git_repo: true,
            sources: vec!["claude".into(), "opencode".into(), "git".into()],
            active_chats: vec![ActiveChatView {
                source: "opencode".into(),
                session: "ses-oc-1".into(),
                title: "Lectura SQLite OpenCode y roles".into(),
                model: "DeepSeek V4 Pro".into(),
                working: false,
                last_ts: now - 300_000,
            }],
            last_ts: now - 300_000,
        },
        WorkspaceInfo {
            name: "inventario-api".into(),
            path: "C:/Proyectos/inventario-api".into(),
            is_git_repo: true,
            sources: vec!["codex".into(), "git".into()],
            active_chats: vec![],
            last_ts: now - 3_600_000,
        },
    ]
}

pub fn demo_missions() -> Vec<MissionView> {
    let now = Utc::now().to_rfc3339();
    vec![
        MissionView {
            id: "msn-auth-qr".into(),
            title: "Emparejamiento QR efímero y guardia Host".into(),
            mode: "auto_dag".into(),
            effort: "deep_think".into(),
            sensitivity: "confidential".into(),
            provider_id: "claude".into(),
            status: "running".into(),
            state: AgentActivityState::Debugging,
            worktree: ".loom/worktrees/msn-auth-qr".into(),
            repo_root: Some("C:/Proyectos/Loom".into()),
            gate_passed: Some(true),
            updated_at: now.clone(),
        },
        MissionView {
            id: "msn-ui-tokens".into(),
            title: "Auditoría de contraste AA y responsive 375px".into(),
            mode: "direct".into(),
            effort: "normal".into(),
            sensitivity: "internal".into(),
            provider_id: "codex".into(),
            status: "running".into(),
            state: AgentActivityState::Analysing,
            worktree: ".loom/worktrees/msn-ui-tokens".into(),
            repo_root: Some("C:/Proyectos/ClaudeHub".into()),
            gate_passed: Some(true),
            updated_at: now.clone(),
        },
        MissionView {
            id: "msn-docs-scan".into(),
            title: "Resumen de logs largos y contrato de diseño".into(),
            mode: "direct".into(),
            effort: "fast".into(),
            sensitivity: "internal".into(),
            provider_id: "gemini".into(),
            status: "completed".into(),
            state: AgentActivityState::Reading,
            worktree: ".loom/worktrees/msn-docs-scan".into(),
            repo_root: Some("C:/Proyectos/Loom".into()),
            gate_passed: Some(true),
            updated_at: now,
        },
    ]
}

pub fn demo_diff(mission_id: &str) -> (String, String) {
    (
        "2 archivos modificados (+14, -3)".into(),
        format!(
            "--- a/crates/loomd/src/server.rs\n+++ b/crates/loomd/src/server.rs\n@@ -12,4 +12,9 @@ // mision {mission_id}\n-    return Err(StatusCode::FORBIDDEN);\n+    if !is_allowed_host(host, state.config.remote_enabled) {{\n+        return Err(StatusCode::FORBIDDEN);\n+    }}\n+    Ok(next.run(req).await)"
        ),
    )
}

pub fn demo_skills() -> Vec<SkillEntry> {
    vec![
        SkillEntry {
            name: "ui-ux-pro-max".into(),
            description: "Tokens de diseño, contraste WCAG AA y responsive 375px.".into(),
            path: ".loom/skills/ui-ux-pro-max/SKILL.md".into(),
            source: "project".into(),
        },
        SkillEntry {
            name: "rust-supervisor".into(),
            description: "Aislamiento por Job Objects y control de timeout JSONL.".into(),
            path: ".loom/skills/rust-supervisor/SKILL.md".into(),
            source: "project".into(),
        },
        SkillEntry {
            name: "code-review".into(),
            description: "Revisión estricta de regresiones, alcance y seguridad.".into(),
            path: ".agents/skills/code-review/SKILL.md".into(),
            source: "agents".into(),
        },
    ]
}

pub fn demo_telemetry() -> TelemetrySnapshot {
    let mut heatmap = Vec::with_capacity(98);
    let base = Utc::now().date_naive();
    for i in (0_i64..98).rev() {
        let day = base - chrono::Duration::days(i);
        let count = ((i * 7 + 3) % 11) as u32;
        heatmap.push(HeatPoint {
            day: day.format("%Y-%m-%d").to_string(),
            count,
        });
    }

    TelemetrySnapshot {
        total_tokens: 1_842_500,
        active_agents: 3,
        gate_pass_rate: 96.4,
        avg_latency_ms: 412,
        heatmap,
        by_provider: vec![
            ProviderMetric {
                provider_id: "claude".into(),
                name: "Claude Code".into(),
                calls: 84,
                success_rate: 98.8,
                p95_ms: 1120,
                errors_429: 0,
                errors_403: 0,
                tokens: 920_000,
            },
            ProviderMetric {
                provider_id: "codex".into(),
                name: "Codex CLI".into(),
                calls: 52,
                success_rate: 96.1,
                p95_ms: 1480,
                errors_429: 1,
                errors_403: 0,
                tokens: 512_000,
            },
            ProviderMetric {
                provider_id: "gemini".into(),
                name: "Gemini CLI".into(),
                calls: 37,
                success_rate: 94.5,
                p95_ms: 980,
                errors_429: 1,
                errors_403: 0,
                tokens: 280_500,
            },
            ProviderMetric {
                provider_id: "opencode".into(),
                name: "OpenCode".into(),
                calls: 41,
                success_rate: 97.5,
                p95_ms: 690,
                errors_429: 0,
                errors_403: 0,
                tokens: 130_000,
            },
        ],
        logs: vec![
            TelemetryLog {
                ts: Utc::now().to_rfc3339(),
                provider_id: "claude".into(),
                level: "info".into(),
                message: "Quality Gate superado en worktree msn-auth-qr".into(),
            },
            TelemetryLog {
                ts: Utc::now().to_rfc3339(),
                provider_id: "codex".into(),
                level: "info".into(),
                message: "Read tokens.css · 142 lines".into(),
            },
            TelemetryLog {
                ts: Utc::now().to_rfc3339(),
                provider_id: "gemini".into(),
                level: "warn".into(),
                message: "Reintento suave aplicado tras latencia alta".into(),
            },
        ],
    }
}

pub fn demo_live_event(step: usize) -> LiveEvent {
    let presets = [
        (
            AgentActivityState::Thinking,
            "claude",
            "supervisor.rs · 184 lines",
            "Evaluando grafo de tareas aisladas",
        ),
        (
            AgentActivityState::Reading,
            "gemini",
            "contrato-agentes.md · 72 lines",
            "Extrayendo reglas UI del proyecto",
        ),
        (
            AgentActivityState::Analysing,
            "codex",
            "tokens.css · 140 lines",
            "Verificando contraste AA y escala 375px",
        ),
        (
            AgentActivityState::Debugging,
            "claude",
            "auth.rs · 210 lines",
            "Validando tokens QR de un solo uso",
        ),
        (
            AgentActivityState::Creating,
            "opencode",
            "DotHeatmap.tsx · 96 lines",
            "Generando matriz de puntos interactiva",
        ),
    ];
    let (state, provider, target, detail) = presets[step % presets.len()].clone();
    LiveEvent {
        kind: "agent_activity".into(),
        state,
        provider_id: provider.into(),
        target: target.into(),
        detail: detail.into(),
        ts: Utc::now().to_rfc3339(),
    }
}
