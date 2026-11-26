use crate::db::{DbError, LoomDb};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Component, Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ArtifactError {
    #[error("ruta de artefacto invalida '{0}'")]
    InvalidPath(String),
    #[error("error de archivo en {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("error de base de datos: {0}")]
    Db(#[from] DbError),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactRecord {
    pub id: String,
    pub mission_id: String,
    pub kind: String,
    pub rel_path: String,
    pub mime_type: Option<String>,
    pub sha256: Option<String>,
    pub created_by: String,
    pub created_at: String,
}

pub struct NewArtifact<'a> {
    pub mission_id: &'a str,
    pub kind: &'a str,
    pub rel_path: &'a str,
    pub mime_type: Option<&'a str>,
    pub created_by: &'a str,
    pub bytes: &'a [u8],
}

pub struct ArtifactStore {
    root_dir: PathBuf,
}

impl ArtifactStore {
    pub fn new(root_dir: PathBuf) -> Self {
        Self { root_dir }
    }

    pub fn put(
        &self,
        db: &LoomDb,
        input: NewArtifact<'_>,
    ) -> Result<ArtifactRecord, ArtifactError> {
        let clean_mission = sanitize_segment(input.mission_id)?;
        let clean_rel = validate_rel_path(input.rel_path)?;
        let target = self.root_dir.join(&clean_mission).join(&clean_rel);

        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|source| ArtifactError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        std::fs::write(&target, input.bytes).map_err(|source| ArtifactError::Io {
            path: target,
            source,
        })?;

        let record = ArtifactRecord {
            id: Uuid::new_v4().to_string(),
            mission_id: input.mission_id.to_string(),
            kind: input.kind.to_string(),
            rel_path: clean_rel.to_string_lossy().replace('\\', "/"),
            mime_type: input.mime_type.map(str::to_string),
            sha256: Some(sha256_hex(input.bytes)),
            created_by: input.created_by.to_string(),
            created_at: Utc::now().to_rfc3339(),
        };

        db.insert_artifact(&record)?;
        Ok(record)
    }

    pub fn list_by_mission(
        &self,
        db: &LoomDb,
        mission_id: &str,
    ) -> Result<Vec<ArtifactRecord>, ArtifactError> {
        Ok(db.list_artifacts(mission_id)?)
    }
}

fn sanitize_segment(segment: &str) -> Result<String, ArtifactError> {
    if segment.trim().is_empty()
        || segment.contains('/')
        || segment.contains('\\')
        || segment.contains("..")
    {
        return Err(ArtifactError::InvalidPath(segment.to_string()));
    }
    Ok(segment.trim().to_string())
}

fn validate_rel_path(rel: &str) -> Result<PathBuf, ArtifactError> {
    let trimmed = rel.trim();
    if trimmed.is_empty() {
        return Err(ArtifactError::InvalidPath(rel.to_string()));
    }
    let candidate = Path::new(trimmed);
    let mut clean = PathBuf::new();
    for comp in candidate.components() {
        match comp {
            Component::Normal(part) => clean.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(ArtifactError::InvalidPath(rel.to_string()));
            }
        }
    }
    if clean.as_os_str().is_empty() {
        return Err(ArtifactError::InvalidPath(rel.to_string()));
    }
    Ok(clean)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write;
        let _ = write!(&mut hex, "{byte:02x}");
    }
    hex
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stores_artifact_computes_sha256_and_blocks_traversal() {
        let mut dir = std::env::temp_dir();
        dir.push(format!("loom-art-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);

        let db = LoomDb::open_in_memory().unwrap();
        db.upsert_project("p1", "Loom", "C:/Proyectos/Loom", None)
            .unwrap();
        db.upsert_mission("m1", "p1", "Test", "running", "internal")
            .unwrap();

        let store = ArtifactStore::new(dir.clone());
        let rec = store
            .put(
                &db,
                NewArtifact {
                    mission_id: "m1",
                    kind: "plan",
                    rel_path: "plans/step1.md",
                    mime_type: Some("text/markdown"),
                    created_by: "claude",
                    bytes: b"hola",
                },
            )
            .unwrap();

        assert_eq!(
            rec.sha256.as_deref(),
            Some("b221d9dbb083a7f33428d7c2a3c3198ae925614d70210e28716ccaa7cd4ddb79")
        );
        assert_eq!(store.list_by_mission(&db, "m1").unwrap().len(), 1);

        let bad = store.put(
            &db,
            NewArtifact {
                mission_id: "m1",
                kind: "secret",
                rel_path: "../secret.txt",
                mime_type: None,
                created_by: "codex",
                bytes: b"nope",
            },
        );
        assert!(matches!(bad, Err(ArtifactError::InvalidPath(_))));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
