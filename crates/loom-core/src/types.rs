use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Locale {
    #[default]
    Es,
    En,
    PtBr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyLevel {
    Local,
    Paid,
    Free,
    Anonymous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Sensitivity {
    Public,
    Internal,
    Confidential,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    Cli,
    HttpOpenAi,
    Mcp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderTier {
    Builtin,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DoctorStatus {
    Ok,
    Warning,
    Unavailable,
}

pub struct PrivacyGate;

impl PrivacyGate {
    pub fn is_allowed(sensitivity: Sensitivity, privacy: PrivacyLevel) -> bool {
        match sensitivity {
            Sensitivity::Confidential => {
                matches!(privacy, PrivacyLevel::Local | PrivacyLevel::Paid)
            }
            Sensitivity::Internal => matches!(
                privacy,
                PrivacyLevel::Local | PrivacyLevel::Paid | PrivacyLevel::Free
            ),
            Sensitivity::Public => true,
        }
    }
}

pub fn is_sensitive_path(path: &str) -> bool {
    let normalized = path.replace('\\', "/").to_ascii_lowercase();
    let file_name = Path::new(&normalized)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    if matches!(
        file_name,
        ".env" | "auth.json" | "mcp-auth.json" | ".credentials.json"
    ) {
        return true;
    }

    if file_name.starts_with(".env.") {
        return true;
    }

    file_name.ends_with(".pem") || file_name.ends_with(".key")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn privacy_gate_matrix() {
        assert!(PrivacyGate::is_allowed(
            Sensitivity::Confidential,
            PrivacyLevel::Local
        ));
        assert!(PrivacyGate::is_allowed(
            Sensitivity::Confidential,
            PrivacyLevel::Paid
        ));
        assert!(!PrivacyGate::is_allowed(
            Sensitivity::Confidential,
            PrivacyLevel::Free
        ));
        assert!(!PrivacyGate::is_allowed(
            Sensitivity::Confidential,
            PrivacyLevel::Anonymous
        ));

        assert!(PrivacyGate::is_allowed(
            Sensitivity::Internal,
            PrivacyLevel::Local
        ));
        assert!(PrivacyGate::is_allowed(
            Sensitivity::Internal,
            PrivacyLevel::Paid
        ));
        assert!(PrivacyGate::is_allowed(
            Sensitivity::Internal,
            PrivacyLevel::Free
        ));
        assert!(!PrivacyGate::is_allowed(
            Sensitivity::Internal,
            PrivacyLevel::Anonymous
        ));

        assert!(PrivacyGate::is_allowed(
            Sensitivity::Public,
            PrivacyLevel::Local
        ));
        assert!(PrivacyGate::is_allowed(
            Sensitivity::Public,
            PrivacyLevel::Paid
        ));
        assert!(PrivacyGate::is_allowed(
            Sensitivity::Public,
            PrivacyLevel::Free
        ));
        assert!(PrivacyGate::is_allowed(
            Sensitivity::Public,
            PrivacyLevel::Anonymous
        ));
    }

    #[test]
    fn detects_sensitive_paths_across_separators_and_case() {
        assert!(is_sensitive_path(".env"));
        assert!(is_sensitive_path("apps/web/.ENV.local"));
        assert!(is_sensitive_path(r"C:\Proyectos\Loom\.env.production"));
        assert!(is_sensitive_path("certs\\server.PEM"));
        assert!(is_sensitive_path("/tmp/private.key"));
        assert!(is_sensitive_path("C:\\Users\\test\\auth.json"));
        assert!(is_sensitive_path(".codex/mcp-auth.json"));
        assert!(is_sensitive_path("home/.credentials.json"));

        assert!(!is_sensitive_path("src/main.rs"));
        assert!(!is_sensitive_path("env.ts"));
        assert!(!is_sensitive_path("docs/auth.md"));
    }
}
