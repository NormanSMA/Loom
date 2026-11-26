use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub const LOOM_RUNS_FILENAME: &str = "loom-runs.jsonl";
pub const MAX_RECORD_BYTES: usize = 16 * 1024;
pub const MAX_LOG_BYTES: u64 = 5 * 1024 * 1024;

#[derive(Debug, Error)]
pub enum ExchangeError {
    #[error("el registro excede el limite de {0} bytes")]
    RecordTooLarge(usize),
    #[error("error de escritura en intercambio: {0}")]
    Io(#[from] std::io::Error),
    #[error("error al serializar registro: {0}")]
    Json(#[from] serde_json::Error),
}

/// Registro de ejecucion emitido por Loom hacia `~/.claudehub/loom-runs.jsonl`.
/// Solo contiene metricas numericas y metadatos de sesion/rol; nunca prompts ni rutas completas.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LoomRunRecord {
    pub id: String,
    pub ts: i64,
    pub session: String,
    pub project: String,
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    pub model: String,
    pub provider: String,
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    pub cache_write: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<u64>,
}

pub fn default_exchange_dir() -> PathBuf {
    if let Ok(custom) = std::env::var("CLAUDEHUB_EXCHANGE") {
        let trimmed = custom.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claudehub")
}

pub fn append_loom_run(
    exchange_dir: &Path,
    record: &LoomRunRecord,
) -> Result<PathBuf, ExchangeError> {
    append_loom_run_with_limit(exchange_dir, record, MAX_LOG_BYTES)
}

fn append_loom_run_with_limit(
    exchange_dir: &Path,
    record: &LoomRunRecord,
    max_log_bytes: u64,
) -> Result<PathBuf, ExchangeError> {
    let serialized = serde_json::to_string(record)?;
    if serialized.len() > MAX_RECORD_BYTES {
        return Err(ExchangeError::RecordTooLarge(MAX_RECORD_BYTES));
    }

    fs::create_dir_all(exchange_dir)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(exchange_dir, fs::Permissions::from_mode(0o700));
    }

    let path = exchange_dir.join(LOOM_RUNS_FILENAME);
    if let Ok(meta) = fs::metadata(&path)
        && meta.len() >= max_log_bytes
    {
        let backup = exchange_dir.join(format!("{LOOM_RUNS_FILENAME}.1"));
        let _ = fs::rename(&path, backup);
    }

    let mut file = OpenOptions::new().create(true).append(true).open(&path)?;
    file.write_all(serialized.as_bytes())?;
    file.write_all(b"\n")?;
    file.flush()?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn sample_record(id: &str) -> LoomRunRecord {
        LoomRunRecord {
            id: id.into(),
            ts: 1_785_803_236_000,
            session: "msn-123".into(),
            project: "Loom".into(),
            role: "orchestrator".into(),
            agent: None,
            model: "claude-sonnet-5-5".into(),
            provider: "claude".into(),
            input: 100,
            output: 50,
            cache_read: 25,
            cache_write: 10,
            reasoning: None,
        }
    }

    #[test]
    fn appends_camel_case_jsonl_to_exchange_dir() {
        let dir = std::env::temp_dir().join(format!("loom-ex-{}", Uuid::new_v4()));
        let path = append_loom_run(&dir, &sample_record("loom:run-1")).unwrap();
        let raw = fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = raw.lines().collect();
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("\"cacheRead\":25"));
        assert!(lines[0].contains("\"cacheWrite\":10"));
        assert!(!lines[0].contains("\"agent\""));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn rejects_oversized_records() {
        let dir = std::env::temp_dir().join(format!("loom-ex-{}", Uuid::new_v4()));
        let mut rec = sample_record("loom:huge");
        rec.project = "x".repeat(MAX_RECORD_BYTES + 128);
        let err = append_loom_run(&dir, &rec).unwrap_err();
        assert!(matches!(err, ExchangeError::RecordTooLarge(_)));
        assert!(!dir.join(LOOM_RUNS_FILENAME).exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn rotates_log_when_threshold_reached() {
        let dir = std::env::temp_dir().join(format!("loom-ex-{}", Uuid::new_v4()));
        append_loom_run_with_limit(&dir, &sample_record("loom:r1"), 40).unwrap();
        append_loom_run_with_limit(&dir, &sample_record("loom:r2"), 40).unwrap();

        let rotated = fs::read_to_string(dir.join(format!("{LOOM_RUNS_FILENAME}.1"))).unwrap();
        let current = fs::read_to_string(dir.join(LOOM_RUNS_FILENAME)).unwrap();
        assert!(rotated.contains("loom:r1"));
        assert!(current.contains("loom:r2"));
        assert!(!current.contains("loom:r1"));
        let _ = fs::remove_dir_all(&dir);
    }
}
