use chrono::Utc;
use rusqlite::{Connection, params};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("no se pudo crear el directorio de datos {0}: {1}")]
    CreateDir(String, #[source] std::io::Error),
    #[error("error SQLite: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

const MIGRATION_V1: &str = r#"
CREATE TABLE IF NOT EXISTS schema_migrations (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    root_path TEXT NOT NULL UNIQUE,
    vault_slug TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS missions (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    status TEXT NOT NULL,
    sensitivity TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY,
    mission_id TEXT NOT NULL REFERENCES missions(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    provider_id TEXT,
    status TEXT NOT NULL,
    worktree_path TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS worker_runs (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    provider_id TEXT NOT NULL,
    model TEXT,
    termination TEXT NOT NULL,
    exit_code INTEGER,
    duration_ms INTEGER NOT NULL,
    input_tokens INTEGER NOT NULL DEFAULT 0,
    output_tokens INTEGER NOT NULL DEFAULT 0,
    cache_read_tokens INTEGER NOT NULL DEFAULT 0,
    started_at TEXT NOT NULL,
    finished_at TEXT
);

CREATE TABLE IF NOT EXISTS artifacts (
    id TEXT PRIMARY KEY,
    mission_id TEXT NOT NULL REFERENCES missions(id) ON DELETE CASCADE,
    kind TEXT NOT NULL,
    rel_path TEXT NOT NULL,
    mime_type TEXT,
    sha256 TEXT,
    created_by TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS telemetry_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    provider_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    http_status INTEGER,
    latency_ms INTEGER,
    detail TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS devices (
    id TEXT PRIMARY KEY,
    label TEXT NOT NULL,
    token_hash TEXT NOT NULL,
    created_at TEXT NOT NULL,
    last_seen_at TEXT,
    revoked_at TEXT
);

CREATE TABLE IF NOT EXISTS audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    actor TEXT NOT NULL,
    action TEXT NOT NULL,
    target TEXT,
    created_at TEXT NOT NULL
);
"#;

pub struct LoomDb {
    conn: Connection,
}

impl LoomDb {
    pub fn open(path: &Path) -> Result<Self, DbError> {
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)
                .map_err(|e| DbError::CreateDir(parent.display().to_string(), e))?;
        }
        let conn = Connection::open(path)?;
        Self::init(conn)
    }

    pub fn open_in_memory() -> Result<Self, DbError> {
        let conn = Connection::open_in_memory()?;
        Self::init(conn)
    }

    fn init(conn: Connection) -> Result<Self, DbError> {
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA foreign_keys = ON;
             PRAGMA busy_timeout = 5000;",
        )?;

        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<(), DbError> {
        self.conn.execute_batch(MIGRATION_V1)?;
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT OR IGNORE INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
            params![1_i64, now],
        )?;
        Ok(())
    }

    pub fn schema_version(&self) -> Result<i64, DbError> {
        let v = self.conn.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )?;
        Ok(v)
    }

    pub fn journal_mode(&self) -> Result<String, DbError> {
        let mode: String = self
            .conn
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))?;
        Ok(mode)
    }

    pub fn list_tables(&self) -> Result<Vec<String>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
        )?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        let mut tables = Vec::new();
        for name in rows {
            tables.push(name?);
        }
        Ok(tables)
    }

    pub fn upsert_project(
        &self,
        id: &str,
        name: &str,
        root_path: &str,
        vault_slug: Option<&str>,
    ) -> Result<(), DbError> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO projects (id, name, root_path, vault_slug, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(root_path) DO UPDATE SET
               name = excluded.name,
               vault_slug = excluded.vault_slug",
            params![id, name, root_path, vault_slug, now],
        )?;
        Ok(())
    }

    pub fn record_telemetry(
        &self,
        provider_id: &str,
        event_type: &str,
        http_status: Option<i64>,
        latency_ms: Option<i64>,
        detail: Option<&str>,
    ) -> Result<(), DbError> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO telemetry_events (provider_id, event_type, http_status, latency_ms, detail, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![provider_id, event_type, http_status, latency_ms, detail, now],
        )?;
        Ok(())
    }

    pub fn upsert_mission(
        &self,
        id: &str,
        project_id: &str,
        title: &str,
        status: &str,
        sensitivity: &str,
    ) -> Result<(), DbError> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO missions (id, project_id, title, status, sensitivity, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)
             ON CONFLICT(id) DO UPDATE SET
               title = excluded.title,
               status = excluded.status,
               updated_at = excluded.updated_at",
            params![id, project_id, title, status, sensitivity, now],
        )?;
        Ok(())
    }

    pub fn insert_artifact(
        &self,
        record: &crate::artifacts::ArtifactRecord,
    ) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT INTO artifacts (id, mission_id, kind, rel_path, mime_type, sha256, created_by, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                record.id,
                record.mission_id,
                record.kind,
                record.rel_path,
                record.mime_type,
                record.sha256,
                record.created_by,
                record.created_at
            ],
        )?;
        Ok(())
    }

    pub fn list_artifacts(
        &self,
        mission_id: &str,
    ) -> Result<Vec<crate::artifacts::ArtifactRecord>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, mission_id, kind, rel_path, mime_type, sha256, created_by, created_at
             FROM artifacts
             WHERE mission_id = ?1
             ORDER BY created_at ASC",
        )?;
        let rows = stmt.query_map(params![mission_id], |row| {
            Ok(crate::artifacts::ArtifactRecord {
                id: row.get(0)?,
                mission_id: row.get(1)?,
                kind: row.get(2)?,
                rel_path: row.get(3)?,
                mime_type: row.get(4)?,
                sha256: row.get(5)?,
                created_by: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_is_idempotent_and_uses_wal_on_disk() {
        let mut path = std::env::temp_dir();
        path.push(format!("loom-db-{}.sqlite", std::process::id()));
        let _ = std::fs::remove_file(&path);

        let db1 = LoomDb::open(&path).unwrap();
        assert_eq!(db1.schema_version().unwrap(), 1);
        assert_eq!(db1.journal_mode().unwrap().to_ascii_lowercase(), "wal");
        let tables = db1.list_tables().unwrap();
        for expected in [
            "artifacts",
            "audit_log",
            "devices",
            "missions",
            "projects",
            "schema_migrations",
            "tasks",
            "telemetry_events",
            "worker_runs",
        ] {
            assert!(tables.contains(&expected.to_string()), "missing {expected}");
        }
        drop(db1);

        let db2 = LoomDb::open(&path).unwrap();
        assert_eq!(db2.schema_version().unwrap(), 1);
        db2.upsert_project("p1", "Loom", "C:/Proyectos/Loom", Some("loom"))
            .unwrap();
        db2.record_telemetry("claude", "doctor_ok", Some(200), Some(15), None)
            .unwrap();
        drop(db2);

        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(path.with_extension("sqlite-wal"));
        let _ = std::fs::remove_file(path.with_extension("sqlite-shm"));
    }
}
