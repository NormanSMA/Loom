use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WorktreeError {
    #[error("fallo de I/O al gestionar worktree: {0}")]
    Io(#[from] std::io::Error),
    #[error("identificador de worktree invalido: {0}")]
    InvalidId(String),
    #[error("comando git '{command}' fallo: {stderr}")]
    Git { command: String, stderr: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorktreeLease {
    pub mission_id: String,
    pub task_id: String,
    pub branch: String,
    pub path: PathBuf,
    pub allocated_port: u16,
    pub test_db_name: String,
}

pub struct WorktreeManager;

impl WorktreeManager {
    pub fn allocate_port(task_id: &str) -> u16 {
        let mut hash: u32 = 2_166_136_261;
        for b in task_id.as_bytes() {
            hash ^= u32::from(*b);
            hash = hash.wrapping_mul(16_777_619);
        }
        4400 + (hash % 500) as u16
    }

    pub fn create(
        repo_root: &Path,
        mission_id: &str,
        task_id: &str,
        base_ref: &str,
    ) -> Result<WorktreeLease, WorktreeError> {
        let safe_mission = sanitize_id(mission_id)?;
        let safe_task = sanitize_id(task_id)?;
        let branch = format!("loom/{safe_mission}-{safe_task}");
        let worktrees_root = repo_root.join(".loom").join("worktrees");
        std::fs::create_dir_all(&worktrees_root)?;

        let path = worktrees_root.join(format!("{safe_mission}-{safe_task}"));
        let path_str = path.to_string_lossy().to_string();

        run_git(
            repo_root,
            &["worktree", "add", "-b", &branch, &path_str, base_ref],
        )?;

        Ok(WorktreeLease {
            mission_id: mission_id.to_string(),
            task_id: task_id.to_string(),
            branch,
            path,
            allocated_port: Self::allocate_port(&safe_task),
            test_db_name: format!("loom_test_{safe_task}"),
        })
    }

    pub fn remove(repo_root: &Path, lease: &WorktreeLease) -> Result<(), WorktreeError> {
        let path_str = lease.path.to_string_lossy().to_string();
        run_git(repo_root, &["worktree", "remove", "--force", &path_str])?;
        run_git(repo_root, &["branch", "-D", &lease.branch])?;
        Ok(())
    }
}

fn sanitize_id(raw: &str) -> Result<String, WorktreeError> {
    let clean: String = raw
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    if clean.is_empty() {
        return Err(WorktreeError::InvalidId(raw.to_string()));
    }
    Ok(clean)
}

fn run_git(repo_root: &Path, args: &[&str]) -> Result<(), WorktreeError> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(args)
        .output()?;
    if !out.status.success() {
        return Err(WorktreeError::Git {
            command: format!("git {}", args.join(" ")),
            stderr: String::from_utf8_lossy(&out.stderr).trim().to_string(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_and_removes_isolated_git_worktree() {
        let mut root = std::env::temp_dir();
        root.push(format!("loom-wt-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        run_git(&root, &["init"]).unwrap();
        run_git(&root, &["config", "user.email", "test@loom.local"]).unwrap();
        run_git(&root, &["config", "user.name", "Loom Test"]).unwrap();
        std::fs::write(root.join("README.md"), "base\n").unwrap();
        run_git(&root, &["add", "README.md"]).unwrap();
        run_git(&root, &["commit", "-m", "init"]).unwrap();

        let lease = WorktreeManager::create(&root, "m1", "t1", "HEAD").unwrap();
        assert!(lease.path.join("README.md").is_file());
        assert!((4400..4900).contains(&lease.allocated_port));

        WorktreeManager::remove(&root, &lease).unwrap();
        assert!(!lease.path.exists());

        let _ = std::fs::remove_dir_all(&root);
    }
}
