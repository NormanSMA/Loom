use crate::providers::ProviderRegistry;
use crate::types::{PrivacyGate, Sensitivity};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RouteError {
    #[error("ningun proveedor habilitado cumple sensibilidad {0:?}")]
    NoEligibleProvider(Sensitivity),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskRole {
    Architect,
    Explorer,
    Implementer,
    Reviewer,
    LongContext,
    FastFree,
    ImageGen,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct QuotaSnapshot {
    pub claude_5h_pct: Option<f64>,
    pub claude_7d_pct: Option<f64>,
    pub updated_at: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteDecision {
    pub provider_id: String,
    pub reason: String,
    pub fallback_chain: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ClaudeHubLimitsFile {
    #[serde(rename = "fiveHour")]
    five_hour: Option<ClaudeHubWindow>,
    #[serde(rename = "sevenDay")]
    seven_day: Option<ClaudeHubWindow>,
    #[serde(rename = "updatedAt")]
    updated_at: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct ClaudeHubWindow {
    pct: f64,
    #[serde(rename = "resetsAt")]
    resets_at: Option<i64>,
}

pub fn read_claudehub_quota(path: Option<&Path>) -> QuotaSnapshot {
    let default_path = dirs::home_dir().map(|h| h.join(".claudehub").join("rate-limits.json"));
    let Some(target) = path.map(Path::to_path_buf).or(default_path) else {
        return QuotaSnapshot::default();
    };
    let Ok(raw) = std::fs::read_to_string(target) else {
        return QuotaSnapshot::default();
    };
    parse_claudehub_quota_str(&raw, Utc::now().timestamp_millis())
}

pub fn parse_claudehub_quota_str(raw: &str, now_ms: i64) -> QuotaSnapshot {
    let Ok(parsed) = serde_json::from_str::<ClaudeHubLimitsFile>(raw) else {
        return QuotaSnapshot::default();
    };

    if let Some(ts) = parsed.updated_at
        && ts > now_ms + 300_000
    {
        return QuotaSnapshot::default();
    }

    let valid_pct = |w: Option<ClaudeHubWindow>| -> Option<f64> {
        let win = w?;
        if let Some(reset) = win.resets_at
            && reset <= now_ms
        {
            return None;
        }
        Some(win.pct.clamp(0.0, 100.0))
    };

    QuotaSnapshot {
        claude_5h_pct: valid_pct(parsed.five_hour),
        claude_7d_pct: valid_pct(parsed.seven_day),
        updated_at: parsed.updated_at,
    }
}

pub fn route_task(
    role: TaskRole,
    sensitivity: Sensitivity,
    registry: &ProviderRegistry,
    quota: &QuotaSnapshot,
) -> Result<RouteDecision, RouteError> {
    let claude_high =
        quota.claude_5h_pct.unwrap_or(0.0) >= 80.0 || quota.claude_7d_pct.unwrap_or(0.0) >= 80.0;
    let claude_critical = quota.claude_7d_pct.unwrap_or(0.0) >= 95.0;

    let preferred: Vec<&str> = match role {
        TaskRole::Architect => {
            if claude_critical {
                vec!["codex", "claude", "antigravity"]
            } else {
                vec!["claude", "codex", "antigravity"]
            }
        }
        TaskRole::Explorer => vec!["claude", "gemini", "opencode", "codex"],
        TaskRole::Implementer | TaskRole::Reviewer => {
            if claude_critical && sensitivity != Sensitivity::Confidential {
                vec!["codex", "opencode", "omniroute", "claude"]
            } else if claude_high {
                vec!["codex", "claude", "gemini", "opencode"]
            } else {
                vec!["claude", "codex", "gemini", "opencode"]
            }
        }
        TaskRole::LongContext => {
            if sensitivity == Sensitivity::Confidential {
                vec!["claude", "codex"]
            } else {
                vec!["gemini", "claude", "omniroute"]
            }
        }
        TaskRole::FastFree => vec!["opencode", "omniroute", "gemini", "ollama", "claude"],
        TaskRole::ImageGen => vec!["codex", "gemini", "antigravity"],
    };

    let mut eligible: Vec<String> = Vec::new();
    for id in preferred {
        if let Some(m) = registry.get(id)
            && m.enabled
            && PrivacyGate::is_allowed(sensitivity, m.privacy)
            && !eligible.contains(&m.id)
        {
            eligible.push(m.id.clone());
        }
    }

    for m in registry.enabled() {
        if PrivacyGate::is_allowed(sensitivity, m.privacy) && !eligible.contains(&m.id) {
            eligible.push(m.id.clone());
        }
    }

    let Some(first) = eligible.first().cloned() else {
        return Err(RouteError::NoEligibleProvider(sensitivity));
    };

    let reason = if claude_high && first != "claude" {
        "Desvio por cuota Claude >= 80% y politica de privacidad".to_string()
    } else if sensitivity == Sensitivity::Confidential {
        "Seleccion restringida a proveedores locales o de pago".to_string()
    } else {
        format!("Mejor candidato activo para rol {role:?}")
    };

    Ok(RouteDecision {
        provider_id: first,
        reason,
        fallback_chain: eligible.into_iter().skip(1).collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn registry() -> ProviderRegistry {
        ProviderRegistry::load(Path::new("__nonexistent__"), "http://127.0.0.1:20128").unwrap()
    }

    #[test]
    fn parses_real_claudehub_rate_limits_schema() {
        let json = r#"{"fiveHour":{"pct":41,"resetsAt":2000000},"sevenDay":{"pct":85,"resetsAt":2000000},"updatedAt":1000000}"#;
        let q = parse_claudehub_quota_str(json, 1_500_000);
        assert_eq!(q.claude_5h_pct, Some(41.0));
        assert_eq!(q.claude_7d_pct, Some(85.0));

        let expired = parse_claudehub_quota_str(json, 2_500_000);
        assert_eq!(expired.claude_5h_pct, None);
        assert_eq!(expired.claude_7d_pct, None);
    }

    #[test]
    fn blocks_free_providers_for_confidential_tasks() {
        let reg = registry();
        let decision = route_task(
            TaskRole::FastFree,
            Sensitivity::Confidential,
            &reg,
            &QuotaSnapshot::default(),
        )
        .unwrap();
        assert_ne!(decision.provider_id, "opencode");
        assert_ne!(decision.provider_id, "omniroute");
        assert_ne!(decision.provider_id, "gemini");
        assert_eq!(decision.provider_id, "claude");
    }

    #[test]
    fn shifts_implementer_to_codex_when_claude_quota_high() {
        let reg = registry();
        let quota = QuotaSnapshot {
            claude_5h_pct: Some(84.0),
            claude_7d_pct: Some(50.0),
            updated_at: Some(100),
        };
        let decision =
            route_task(TaskRole::Implementer, Sensitivity::Internal, &reg, &quota).unwrap();
        assert_eq!(decision.provider_id, "codex");
    }

    #[test]
    fn routes_long_context_to_gemini_when_non_confidential() {
        let reg = registry();
        let decision = route_task(
            TaskRole::LongContext,
            Sensitivity::Internal,
            &reg,
            &QuotaSnapshot::default(),
        )
        .unwrap();
        assert_eq!(decision.provider_id, "gemini");
        let _ = PathBuf::new();
    }
}
