use crate::types::Locale;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("no se pudo leer la configuracion en {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("error de sintaxis TOML en {path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: Box<toml::de::Error>,
    },
    #[error("puerto invalido en configuracion: {0}")]
    InvalidPort(String),
    #[error("bind inseguro '{0}' con remote_enabled = false; usa loopback")]
    UnsafeBind(String),
    #[error("timeouts invalidos: idle={idle}s, total={total}s")]
    InvalidTimeouts { idle: u64, total: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct LoomConfig {
    pub bind: String,
    pub port: u16,
    pub demo_port: u16,
    pub remote_enabled: bool,
    pub locale: Locale,
    pub vault_path: Option<PathBuf>,
    pub claudehub_url: String,
    pub omniroute_url: String,
    pub idle_timeout_secs: u64,
    pub total_timeout_secs: u64,
}

impl Default for LoomConfig {
    fn default() -> Self {
        Self {
            bind: "127.0.0.1".to_string(),
            port: 4320,
            demo_port: 4321,
            remote_enabled: false,
            locale: Locale::Es,
            vault_path: None,
            claudehub_url: "http://127.0.0.1:4317".to_string(),
            omniroute_url: "http://127.0.0.1:20128".to_string(),
            idle_timeout_secs: 120,
            total_timeout_secs: 900,
        }
    }
}

pub fn default_data_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".loom")
}

pub fn default_config_path() -> PathBuf {
    default_data_dir().join("config.toml")
}

pub fn default_providers_dir() -> PathBuf {
    default_data_dir().join("providers")
}

pub fn default_db_path() -> PathBuf {
    default_data_dir().join("loom.db")
}

pub fn load_config(path: Option<&Path>) -> Result<LoomConfig, ConfigError> {
    let target = path.map(PathBuf::from).unwrap_or_else(default_config_path);
    let config = if target.exists() {
        let raw = std::fs::read_to_string(&target).map_err(|source| ConfigError::Io {
            path: target.clone(),
            source,
        })?;
        toml::from_str::<LoomConfig>(&raw).map_err(|source| ConfigError::Parse {
            path: target,
            source: Box::new(source),
        })?
    } else {
        LoomConfig::default()
    };

    validate_config(&config)?;
    Ok(config)
}

pub fn validate_config(config: &LoomConfig) -> Result<(), ConfigError> {
    if config.port == 0 || config.demo_port == 0 {
        return Err(ConfigError::InvalidPort(
            "los puertos no pueden ser 0".to_string(),
        ));
    }
    if config.port == config.demo_port {
        return Err(ConfigError::InvalidPort(
            "port y demo_port deben ser distintos".to_string(),
        ));
    }
    if !config.remote_enabled && !is_loopback(&config.bind) {
        return Err(ConfigError::UnsafeBind(config.bind.clone()));
    }
    if config.idle_timeout_secs == 0 || config.total_timeout_secs < config.idle_timeout_secs {
        return Err(ConfigError::InvalidTimeouts {
            idle: config.idle_timeout_secs,
            total: config.total_timeout_secs,
        });
    }
    Ok(())
}

fn is_loopback(bind: &str) -> bool {
    matches!(bind.trim(), "127.0.0.1" | "::1" | "localhost")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_file(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("loom-cfg-{}-{}.toml", std::process::id(), name));
        p
    }

    #[test]
    fn defaults_are_valid_and_safe() {
        let cfg = LoomConfig::default();
        assert_eq!(cfg.bind, "127.0.0.1");
        assert_eq!(cfg.port, 4320);
        assert_eq!(cfg.demo_port, 4321);
        assert!(!cfg.remote_enabled);
        assert_eq!(cfg.locale, Locale::Es);
        assert!(validate_config(&cfg).is_ok());
    }

    #[test]
    fn loads_partial_toml_and_applies_defaults() {
        let path = temp_file("partial");
        std::fs::write(&path, "port = 4420\nlocale = \"en\"\n").unwrap();
        let cfg = load_config(Some(&path)).unwrap();
        let _ = std::fs::remove_file(&path);

        assert_eq!(cfg.port, 4420);
        assert_eq!(cfg.demo_port, 4321);
        assert_eq!(cfg.locale, Locale::En);
        assert_eq!(cfg.bind, "127.0.0.1");
    }

    #[test]
    fn rejects_public_bind_when_remote_disabled() {
        let path = temp_file("unsafe-bind");
        std::fs::write(&path, "bind = \"0.0.0.0\"\nremote_enabled = false\n").unwrap();
        let err = load_config(Some(&path)).unwrap_err();
        let _ = std::fs::remove_file(&path);

        assert!(matches!(err, ConfigError::UnsafeBind(_)));
    }
}
