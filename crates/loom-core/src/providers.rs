use crate::types::{PrivacyLevel, ProviderKind, ProviderTier};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("no se pudo leer el directorio de proveedores {path}: {source}")]
    ReadDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("no se pudo leer el manifiesto {path}: {source}")]
    ReadFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("manifiesto TOML invalido en {path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: Box<toml::de::Error>,
    },
    #[error("manifiesto invalido en {path}: {reason}")]
    Validation { path: PathBuf, reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderManifest {
    pub id: String,
    pub name: String,
    #[serde(default = "default_custom_tier")]
    pub tier: ProviderTier,
    pub kind: ProviderKind,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default = "default_version_args")]
    pub version_args: Vec<String>,
    #[serde(default)]
    pub endpoint: Option<String>,
    #[serde(default)]
    pub env_key: Option<String>,
    pub privacy: PrivacyLevel,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub windows_sandbox_fix: bool,
}

fn default_custom_tier() -> ProviderTier {
    ProviderTier::Custom
}

fn default_version_args() -> Vec<String> {
    vec!["--version".to_string()]
}

fn default_true() -> bool {
    true
}

pub fn builtin_providers(omniroute_url: &str) -> Vec<ProviderManifest> {
    vec![
        ProviderManifest {
            id: "claude".to_string(),
            name: "Claude Code".to_string(),
            tier: ProviderTier::Builtin,
            kind: ProviderKind::Cli,
            command: Some("claude".to_string()),
            version_args: default_version_args(),
            endpoint: None,
            env_key: None,
            privacy: PrivacyLevel::Paid,
            capabilities: vec_strings(&["code", "plan", "review", "mcp"]),
            enabled: true,
            windows_sandbox_fix: false,
        },
        ProviderManifest {
            id: "codex".to_string(),
            name: "Codex CLI".to_string(),
            tier: ProviderTier::Builtin,
            kind: ProviderKind::Cli,
            command: Some("codex".to_string()),
            version_args: default_version_args(),
            endpoint: None,
            env_key: None,
            privacy: PrivacyLevel::Paid,
            capabilities: vec_strings(&["code", "review", "image"]),
            enabled: true,
            windows_sandbox_fix: true,
        },
        ProviderManifest {
            id: "gemini".to_string(),
            name: "Gemini CLI".to_string(),
            tier: ProviderTier::Builtin,
            kind: ProviderKind::Cli,
            command: Some("gemini".to_string()),
            version_args: default_version_args(),
            endpoint: None,
            env_key: Some("GEMINI_API_KEY".to_string()),
            privacy: PrivacyLevel::Free,
            capabilities: vec_strings(&["long-context", "code", "research"]),
            enabled: true,
            windows_sandbox_fix: false,
        },
        ProviderManifest {
            id: "antigravity".to_string(),
            name: "Antigravity".to_string(),
            tier: ProviderTier::Builtin,
            kind: ProviderKind::Mcp,
            command: Some("agy".to_string()),
            version_args: default_version_args(),
            endpoint: None,
            env_key: None,
            privacy: PrivacyLevel::Paid,
            capabilities: vec_strings(&["orchestrate", "mcp", "browser", "artifacts"]),
            enabled: true,
            windows_sandbox_fix: false,
        },
        ProviderManifest {
            id: "opencode".to_string(),
            name: "OpenCode".to_string(),
            tier: ProviderTier::Builtin,
            kind: ProviderKind::Cli,
            command: Some("opencode".to_string()),
            version_args: default_version_args(),
            endpoint: None,
            env_key: None,
            privacy: PrivacyLevel::Free,
            capabilities: vec_strings(&["code", "boilerplate", "fast"]),
            enabled: true,
            windows_sandbox_fix: false,
        },
        ProviderManifest {
            id: "omniroute".to_string(),
            name: "OmniRoute".to_string(),
            tier: ProviderTier::Builtin,
            kind: ProviderKind::HttpOpenAi,
            command: None,
            version_args: Vec::new(),
            endpoint: Some(omniroute_url.to_string()),
            env_key: Some("OMNIROUTE_API_KEY".to_string()),
            privacy: PrivacyLevel::Free,
            capabilities: vec_strings(&["routing", "fast", "fallback"]),
            enabled: true,
            windows_sandbox_fix: false,
        },
        ProviderManifest {
            id: "ollama".to_string(),
            name: "Ollama Local".to_string(),
            tier: ProviderTier::Builtin,
            kind: ProviderKind::HttpOpenAi,
            command: None,
            version_args: Vec::new(),
            endpoint: Some("http://127.0.0.1:11434".to_string()),
            env_key: None,
            privacy: PrivacyLevel::Local,
            capabilities: vec_strings(&["code", "offline"]),
            enabled: false,
            windows_sandbox_fix: false,
        },
    ]
}

fn vec_strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| (*s).to_string()).collect()
}

#[derive(Debug, Clone)]
pub struct ProviderRegistry {
    providers: Vec<ProviderManifest>,
}

impl ProviderRegistry {
    pub fn load(custom_dir: &Path, omniroute_url: &str) -> Result<Self, ProviderError> {
        let mut by_id: BTreeMap<String, ProviderManifest> = BTreeMap::new();
        for p in builtin_providers(omniroute_url) {
            by_id.insert(p.id.clone(), p);
        }

        if custom_dir.exists() {
            let entries =
                std::fs::read_dir(custom_dir).map_err(|source| ProviderError::ReadDir {
                    path: custom_dir.to_path_buf(),
                    source,
                })?;

            for entry in entries {
                let entry = entry.map_err(|source| ProviderError::ReadDir {
                    path: custom_dir.to_path_buf(),
                    source,
                })?;
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("toml") {
                    continue;
                }
                let raw =
                    std::fs::read_to_string(&path).map_err(|source| ProviderError::ReadFile {
                        path: path.clone(),
                        source,
                    })?;
                let mut manifest: ProviderManifest =
                    toml::from_str(&raw).map_err(|source| ProviderError::Parse {
                        path: path.clone(),
                        source: Box::new(source),
                    })?;
                if manifest.id.trim().is_empty() || manifest.name.trim().is_empty() {
                    return Err(ProviderError::Validation {
                        path,
                        reason: "id y name son obligatorios".to_string(),
                    });
                }
                manifest.tier = ProviderTier::Custom;
                by_id.insert(manifest.id.clone(), manifest);
            }
        }

        Ok(Self {
            providers: by_id.into_values().collect(),
        })
    }

    pub fn all(&self) -> &[ProviderManifest] {
        &self.providers
    }

    pub fn enabled(&self) -> Vec<&ProviderManifest> {
        self.providers.iter().filter(|p| p.enabled).collect()
    }

    pub fn get(&self, id: &str) -> Option<&ProviderManifest> {
        self.providers.iter().find(|p| p.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("loom-prov-{}-{}", std::process::id(), name));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn loads_custom_provider_and_overrides_builtin() {
        let dir = temp_dir("custom");
        std::fs::write(
            dir.join("lmstudio.toml"),
            r#"
id = "lmstudio"
name = "LM Studio"
kind = "http_open_ai"
endpoint = "http://127.0.0.1:1234"
privacy = "local"
capabilities = ["code", "offline"]
"#,
        )
        .unwrap();

        std::fs::write(
            dir.join("ollama.toml"),
            r#"
id = "ollama"
name = "Ollama Work"
kind = "http_open_ai"
endpoint = "http://127.0.0.1:11434"
privacy = "local"
enabled = true
"#,
        )
        .unwrap();

        let reg = ProviderRegistry::load(&dir, "http://127.0.0.1:20128").unwrap();
        let _ = std::fs::remove_dir_all(&dir);

        let lm = reg.get("lmstudio").expect("lmstudio loaded");
        assert_eq!(lm.tier, ProviderTier::Custom);
        assert_eq!(lm.privacy, PrivacyLevel::Local);
        assert!(lm.enabled);

        let ollama = reg.get("ollama").expect("ollama overridden");
        assert_eq!(ollama.tier, ProviderTier::Custom);
        assert_eq!(ollama.name, "Ollama Work");
        assert!(ollama.enabled);
    }
}
