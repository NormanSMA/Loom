use crate::supervisor::{RunSpec, TerminationReason, resolve_program, run_supervised};
use chrono::Utc;
use loom_core::config::LoomConfig;
use loom_core::providers::{ProviderManifest, ProviderRegistry};
use loom_core::types::{DoctorStatus, ProviderKind};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::task::JoinSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckItem {
    pub id: String,
    pub name: String,
    pub category: String,
    pub status: DoctorStatus,
    pub version: Option<String>,
    pub location: Option<String>,
    pub latency_ms: u64,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoctorSummary {
    pub ok: usize,
    pub warning: usize,
    pub unavailable: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoctorReport {
    pub generated_at: String,
    pub os: String,
    pub arch: String,
    pub checks: Vec<CheckItem>,
    pub summary: DoctorSummary,
}

impl DoctorReport {
    pub fn from_checks(mut checks: Vec<CheckItem>) -> Self {
        checks.sort_by(|a, b| a.category.cmp(&b.category).then_with(|| a.id.cmp(&b.id)));
        let summary = DoctorSummary {
            ok: checks
                .iter()
                .filter(|c| c.status == DoctorStatus::Ok)
                .count(),
            warning: checks
                .iter()
                .filter(|c| c.status == DoctorStatus::Warning)
                .count(),
            unavailable: checks
                .iter()
                .filter(|c| c.status == DoctorStatus::Unavailable)
                .count(),
        };
        Self {
            generated_at: Utc::now().to_rfc3339(),
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            checks,
            summary,
        }
    }
}

pub async fn run_doctor(config: &LoomConfig, registry: &ProviderRegistry) -> DoctorReport {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    let mut set = JoinSet::new();

    set.spawn(check_git());

    let ch_url = config.claudehub_url.clone();
    let ch_client = client.clone();
    set.spawn(async move { check_claudehub(&ch_client, &ch_url).await });

    if let Some(vault) = config.vault_path.clone() {
        set.spawn(async move { check_vault(vault) });
    }

    for manifest in registry.enabled().into_iter().cloned() {
        let http = client.clone();
        set.spawn(async move { check_provider(http, manifest).await });
    }

    let mut checks = Vec::new();
    while let Some(joined) = set.join_next().await {
        if let Ok(item) = joined {
            checks.push(item);
        }
    }

    DoctorReport::from_checks(checks)
}

async fn check_git() -> CheckItem {
    let started = Instant::now();
    let spec = RunSpec {
        program: "git".to_string(),
        args: vec!["--version".to_string()],
        cwd: None,
        stdin_payload: None,
        env: Vec::new(),
        idle_timeout: Duration::from_secs(4),
        total_timeout: Duration::from_secs(5),
    };
    let location = resolve_program("git").ok().map(|p| p.display().to_string());
    match run_supervised(spec, None).await {
        Ok(res) if res.termination == TerminationReason::Completed && res.exit_code == Some(0) => {
            CheckItem {
                id: "git".to_string(),
                name: "Git".to_string(),
                category: "core".to_string(),
                status: DoctorStatus::Ok,
                version: res.stdout_lines.first().map(|s| s.trim().to_string()),
                location,
                latency_ms: res.duration_ms,
                warnings: Vec::new(),
            }
        }
        _ => CheckItem {
            id: "git".to_string(),
            name: "Git".to_string(),
            category: "core".to_string(),
            status: DoctorStatus::Unavailable,
            version: None,
            location,
            latency_ms: started.elapsed().as_millis() as u64,
            warnings: vec!["Git es obligatorio para aislar worktrees en Fase 2".to_string()],
        },
    }
}

fn check_vault(vault: PathBuf) -> CheckItem {
    let ok = vault.exists() && vault.is_dir();
    CheckItem {
        id: "vault".to_string(),
        name: "Boveda Markdown".to_string(),
        category: "memory".to_string(),
        status: if ok {
            DoctorStatus::Ok
        } else {
            DoctorStatus::Warning
        },
        version: None,
        location: Some(vault.display().to_string()),
        latency_ms: 0,
        warnings: if ok {
            Vec::new()
        } else {
            vec!["vault_path configurado no existe; se usara .loom/ local".to_string()]
        },
    }
}

async fn check_claudehub(client: &reqwest::Client, base_url: &str) -> CheckItem {
    let started = Instant::now();
    let url = format!("{}/api/update", base_url.trim_end_matches('/'));
    match client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => {
            let version = resp
                .json::<Value>()
                .await
                .ok()
                .and_then(|v| v.get("current").and_then(Value::as_str).map(str::to_string));
            CheckItem {
                id: "claudehub".to_string(),
                name: "ClaudeHub".to_string(),
                category: "companion".to_string(),
                status: DoctorStatus::Ok,
                version,
                location: Some(base_url.to_string()),
                latency_ms: started.elapsed().as_millis() as u64,
                warnings: Vec::new(),
            }
        }
        Ok(resp) => CheckItem {
            id: "claudehub".to_string(),
            name: "ClaudeHub".to_string(),
            category: "companion".to_string(),
            status: DoctorStatus::Warning,
            version: None,
            location: Some(base_url.to_string()),
            latency_ms: started.elapsed().as_millis() as u64,
            warnings: vec![format!("HTTP {} al consultar ClaudeHub", resp.status())],
        },
        Err(_) => CheckItem {
            id: "claudehub".to_string(),
            name: "ClaudeHub".to_string(),
            category: "companion".to_string(),
            status: DoctorStatus::Unavailable,
            version: None,
            location: Some(base_url.to_string()),
            latency_ms: started.elapsed().as_millis() as u64,
            warnings: vec!["Servicio hermano opcional apagado".to_string()],
        },
    }
}

async fn check_provider(client: reqwest::Client, m: ProviderManifest) -> CheckItem {
    match m.kind {
        ProviderKind::Cli => check_cli_provider(m).await,
        ProviderKind::Mcp => check_mcp_provider(m).await,
        ProviderKind::HttpOpenAi => check_http_provider(&client, m).await,
    }
}

async fn check_cli_provider(m: ProviderManifest) -> CheckItem {
    let started = Instant::now();
    let Some(cmd) = m.command.clone() else {
        return CheckItem {
            id: m.id,
            name: m.name,
            category: "provider".to_string(),
            status: DoctorStatus::Unavailable,
            version: None,
            location: None,
            latency_ms: 0,
            warnings: vec!["Manifiesto CLI sin comando".to_string()],
        };
    };

    let Ok(resolved) = resolve_program(&cmd) else {
        return CheckItem {
            id: m.id,
            name: m.name,
            category: "provider".to_string(),
            status: DoctorStatus::Unavailable,
            version: None,
            location: None,
            latency_ms: started.elapsed().as_millis() as u64,
            warnings: vec![format!("No se encontro '{cmd}' en PATH")],
        };
    };

    let mut warnings = Vec::new();
    if cfg!(windows) && m.windows_sandbox_fix {
        warnings.push("Windows: usar windows.sandbox=unelevated en perfiles con escritura".into());
    }
    if let Some(key) = &m.env_key
        && std::env::var_os(key).is_none()
    {
        warnings.push(format!("Variable {key} no definida en entorno"));
    }

    let spec = RunSpec {
        program: cmd,
        args: m.version_args.clone(),
        cwd: None,
        stdin_payload: None,
        env: Vec::new(),
        idle_timeout: Duration::from_secs(4),
        total_timeout: Duration::from_secs(5),
    };

    match run_supervised(spec, None).await {
        Ok(res) if res.termination == TerminationReason::Completed && res.exit_code == Some(0) => {
            let version = res
                .stdout_lines
                .first()
                .or_else(|| res.stderr_lines.first())
                .map(|s| s.trim().to_string());
            CheckItem {
                id: m.id,
                name: m.name,
                category: "provider".to_string(),
                status: DoctorStatus::Ok,
                version,
                location: Some(resolved.display().to_string()),
                latency_ms: res.duration_ms,
                warnings,
            }
        }
        Ok(res) => {
            warnings.push(format!(
                "Salida {:?} ({:?})",
                res.termination, res.exit_code
            ));
            CheckItem {
                id: m.id,
                name: m.name,
                category: "provider".to_string(),
                status: DoctorStatus::Warning,
                version: None,
                location: Some(resolved.display().to_string()),
                latency_ms: res.duration_ms,
                warnings,
            }
        }
        Err(e) => {
            warnings.push(e.to_string());
            CheckItem {
                id: m.id,
                name: m.name,
                category: "provider".to_string(),
                status: DoctorStatus::Warning,
                version: None,
                location: Some(resolved.display().to_string()),
                latency_ms: started.elapsed().as_millis() as u64,
                warnings,
            }
        }
    }
}

async fn check_mcp_provider(m: ProviderManifest) -> CheckItem {
    let started = Instant::now();
    let local_dir = dirs::home_dir().map(|h| h.join(".gemini").join("antigravity"));
    let has_local = local_dir.as_ref().is_some_and(|p| p.exists());

    if let Some(cmd) = &m.command
        && let Ok(resolved) = resolve_program(cmd)
    {
        let mut item = check_cli_provider(m).await;
        item.warnings
            .push("No usar proxies OAuth con cuentas Google/Antigravity".to_string());
        if item.location.is_none() {
            item.location = Some(resolved.display().to_string());
        }
        return item;
    }

    if has_local {
        return CheckItem {
            id: m.id,
            name: m.name,
            category: "provider".to_string(),
            status: DoctorStatus::Ok,
            version: Some("app-local".to_string()),
            location: local_dir.map(|p| p.display().to_string()),
            latency_ms: started.elapsed().as_millis() as u64,
            warnings: vec!["Modo MCP (App local detectada); no usar proxies OAuth".to_string()],
        };
    }

    CheckItem {
        id: m.id,
        name: m.name,
        category: "provider".to_string(),
        status: DoctorStatus::Unavailable,
        version: None,
        location: None,
        latency_ms: started.elapsed().as_millis() as u64,
        warnings: vec!["No se detecto CLI ni carpeta local de Antigravity".to_string()],
    }
}

async fn check_http_provider(client: &reqwest::Client, m: ProviderManifest) -> CheckItem {
    let started = Instant::now();
    let Some(endpoint) = m.endpoint.clone() else {
        return CheckItem {
            id: m.id,
            name: m.name,
            category: "provider".to_string(),
            status: DoctorStatus::Unavailable,
            version: None,
            location: None,
            latency_ms: 0,
            warnings: vec!["Manifiesto HTTP sin endpoint".to_string()],
        };
    };

    let base = endpoint.trim_end_matches('/');
    let probe_url = if m.id == "omniroute" {
        format!("{base}/api/health")
    } else if m.id == "ollama" {
        format!("{base}/api/tags")
    } else {
        format!("{base}/v1/models")
    };

    let mut warnings = Vec::new();
    let missing_env = m
        .env_key
        .as_ref()
        .is_some_and(|k| std::env::var_os(k).is_none());
    if missing_env && let Some(k) = &m.env_key {
        warnings.push(format!("Falta {k} en entorno"));
    }

    match client.get(&probe_url).send().await {
        Ok(resp) if resp.status().is_success() => {
            let status = if missing_env && m.id == "omniroute" {
                DoctorStatus::Warning
            } else {
                DoctorStatus::Ok
            };
            CheckItem {
                id: m.id,
                name: m.name,
                category: "provider".to_string(),
                status,
                version: Some(format!("HTTP {}", resp.status().as_u16())),
                location: Some(endpoint),
                latency_ms: started.elapsed().as_millis() as u64,
                warnings,
            }
        }
        Ok(resp) => {
            warnings.push(format!("Respuesta HTTP {}", resp.status().as_u16()));
            CheckItem {
                id: m.id,
                name: m.name,
                category: "provider".to_string(),
                status: DoctorStatus::Warning,
                version: None,
                location: Some(endpoint),
                latency_ms: started.elapsed().as_millis() as u64,
                warnings,
            }
        }
        Err(_) => {
            warnings.push("Endpoint local no responde".to_string());
            CheckItem {
                id: m.id,
                name: m.name,
                category: "provider".to_string(),
                status: DoctorStatus::Unavailable,
                version: None,
                location: Some(endpoint),
                latency_ms: started.elapsed().as_millis() as u64,
                warnings,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    #[test]
    fn summarizes_checks_deterministically() {
        let report = DoctorReport::from_checks(vec![
            CheckItem {
                id: "b".into(),
                name: "B".into(),
                category: "provider".into(),
                status: DoctorStatus::Warning,
                version: None,
                location: None,
                latency_ms: 1,
                warnings: vec![],
            },
            CheckItem {
                id: "a".into(),
                name: "A".into(),
                category: "core".into(),
                status: DoctorStatus::Ok,
                version: Some("1.0".into()),
                location: None,
                latency_ms: 1,
                warnings: vec![],
            },
            CheckItem {
                id: "c".into(),
                name: "C".into(),
                category: "companion".into(),
                status: DoctorStatus::Unavailable,
                version: None,
                location: None,
                latency_ms: 1,
                warnings: vec![],
            },
        ]);

        assert_eq!(report.summary.ok, 1);
        assert_eq!(report.summary.warning, 1);
        assert_eq!(report.summary.unavailable, 1);
        assert_eq!(report.checks[0].id, "c");
        assert_eq!(report.checks[1].id, "a");
        assert_eq!(report.checks[2].id, "b");
    }

    #[tokio::test]
    async fn probes_http_companion_endpoint() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            if let Ok((mut sock, _)) = listener.accept().await {
                let mut buf = [0_u8; 512];
                let _ = sock.read(&mut buf).await;
                let body = r#"{"current":"0.12.1"}"#;
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = sock.write_all(resp.as_bytes()).await;
            }
        });

        let client = reqwest::Client::new();
        let item = check_claudehub(&client, &format!("http://{addr}")).await;
        assert_eq!(item.status, DoctorStatus::Ok);
        assert_eq!(item.version.as_deref(), Some("0.12.1"));
    }
}
