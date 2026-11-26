use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeSet, HashMap};
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveChatView {
    pub source: String,
    pub session: String,
    pub title: String,
    pub model: String,
    pub working: bool,
    pub last_ts: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceInfo {
    pub name: String,
    pub path: String,
    pub is_git_repo: bool,
    pub sources: Vec<String>,
    pub active_chats: Vec<ActiveChatView>,
    pub last_ts: i64,
}

/// Descubre proyectos y rutas activas desde Claude Code, Codex CLI y repositorios Git locales.
/// Nunca extrae texto de prompts ni respuestas; solo lee metadatos `cwd` existentes en disco.
pub fn discover_local_workspaces(extra_roots: &[PathBuf]) -> Vec<WorkspaceInfo> {
    let claude_dir = dirs::home_dir().map(|h| h.join(".claude").join("projects"));
    let codex_dir = dirs::home_dir().map(|h| h.join(".codex").join("sessions"));
    discover_workspaces_from(extra_roots, claude_dir.as_deref(), codex_dir.as_deref())
}

pub fn discover_workspaces_from(
    extra_roots: &[PathBuf],
    claude_projects_dir: Option<&Path>,
    codex_sessions_dir: Option<&Path>,
) -> Vec<WorkspaceInfo> {
    let mut by_key: HashMap<String, WorkspaceBuilder> = HashMap::new();

    for root in extra_roots {
        if !root.is_dir() {
            continue;
        }
        if root.join(".git").exists() {
            register_path(&mut by_key, root, "git", 0);
        }
        if let Some(parent) = root.parent() {
            scan_git_siblings(&mut by_key, parent);
        }
    }

    if let Some(dir) = claude_projects_dir {
        scan_claude_cwds(&mut by_key, dir);
    }
    if let Some(dir) = codex_sessions_dir {
        scan_codex_cwds(&mut by_key, dir);
    }

    let mut items: Vec<WorkspaceInfo> = by_key.into_values().map(WorkspaceBuilder::build).collect();
    items.sort_by(|a, b| {
        b.active_chats
            .iter()
            .any(|c| c.working)
            .cmp(&a.active_chats.iter().any(|c| c.working))
            .then_with(|| b.last_ts.cmp(&a.last_ts))
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    items.truncate(40);
    items
}

struct WorkspaceBuilder {
    name: String,
    path: String,
    is_git_repo: bool,
    sources: BTreeSet<String>,
    last_ts: i64,
}

impl WorkspaceBuilder {
    fn build(self) -> WorkspaceInfo {
        WorkspaceInfo {
            name: self.name,
            path: self.path,
            is_git_repo: self.is_git_repo,
            sources: self.sources.into_iter().collect(),
            active_chats: Vec::new(),
            last_ts: self.last_ts,
        }
    }
}

pub fn normalize_workspace_key(path: &Path) -> String {
    let clean = path.to_string_lossy().replace('\\', "/");
    let trimmed = clean.trim_end_matches('/');
    #[cfg(windows)]
    {
        trimmed.to_ascii_lowercase()
    }
    #[cfg(not(windows))]
    {
        trimmed.to_string()
    }
}

pub fn register_workspace_entry(
    map: &mut HashMap<String, (PathBuf, BTreeSet<String>, i64)>,
    path: &Path,
    source: &str,
    ts: i64,
) {
    if !path.is_dir() {
        return;
    }
    let key = normalize_workspace_key(path);
    let entry = map
        .entry(key)
        .or_insert_with(|| (path.to_path_buf(), BTreeSet::new(), ts));
    entry.1.insert(source.to_string());
    if ts > entry.2 {
        entry.2 = ts;
    }
}

fn register_path(map: &mut HashMap<String, WorkspaceBuilder>, path: &Path, source: &str, ts: i64) {
    if !path.is_dir() {
        return;
    }
    let key = normalize_workspace_key(path);
    let display = path.to_string_lossy().replace('\\', "/");
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(&display)
        .to_string();
    let is_git = path.join(".git").exists();

    let entry = map.entry(key).or_insert_with(|| WorkspaceBuilder {
        name,
        path: display,
        is_git_repo: is_git,
        sources: BTreeSet::new(),
        last_ts: ts,
    });
    entry.sources.insert(source.to_string());
    if ts > entry.last_ts {
        entry.last_ts = ts;
    }
}

fn scan_git_siblings(map: &mut HashMap<String, WorkspaceBuilder>, parent: &Path) {
    let Ok(rd) = fs::read_dir(parent) else {
        return;
    };
    let mut count = 0usize;
    for entry in rd.flatten() {
        if count >= 40 {
            break;
        }
        let path = entry.path();
        if path.is_dir() && path.join(".git").exists() {
            let mtime = modified_ms(&path);
            register_path(map, &path, "git", mtime);
            count += 1;
        }
    }
}

fn scan_claude_cwds(map: &mut HashMap<String, WorkspaceBuilder>, dir: &Path) {
    let mut files = Vec::new();
    collect_jsonl(dir, &mut files, 3, &|_| true);
    files.sort_by_key(|a| std::cmp::Reverse(a.1));
    for (file, mtime) in files.into_iter().take(20) {
        if let Some(cwd) = extract_cwd_from_file(&file, 20, &["cwd"]) {
            register_path(map, Path::new(&cwd), "claude", mtime);
        }
    }
}

fn scan_codex_cwds(map: &mut HashMap<String, WorkspaceBuilder>, dir: &Path) {
    let mut files = Vec::new();
    collect_jsonl(dir, &mut files, 5, &|name| {
        name.starts_with("rollout-") && name.ends_with(".jsonl")
    });
    files.sort_by_key(|a| std::cmp::Reverse(a.1));
    for (file, mtime) in files.into_iter().take(15) {
        if let Some(cwd) = extract_codex_cwd(&file) {
            register_path(map, Path::new(&cwd), "codex", mtime);
        }
    }
}

fn collect_jsonl(
    dir: &Path,
    out: &mut Vec<(PathBuf, i64)>,
    depth_left: usize,
    match_name: &dyn Fn(&str) -> bool,
) {
    if depth_left == 0 {
        return;
    }
    let Ok(rd) = fs::read_dir(dir) else {
        return;
    };
    for entry in rd.flatten() {
        let p = entry.path();
        if p.is_dir() {
            collect_jsonl(&p, out, depth_left - 1, match_name);
        } else if let Some(name) = p.file_name().and_then(|n| n.to_str())
            && name.ends_with(".jsonl")
            && match_name(name)
        {
            out.push((p.clone(), modified_ms(&p)));
        }
    }
}

fn extract_cwd_from_file(path: &Path, max_lines: usize, keys: &[&str]) -> Option<String> {
    let f = File::open(path).ok()?;
    let reader = BufReader::new(f);
    for line in reader.lines().take(max_lines).flatten() {
        if !line.contains("\"cwd\"") {
            continue;
        }
        let Ok(val) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        for key in keys {
            if let Some(cwd) = val.get(*key).and_then(Value::as_str)
                && !cwd.trim().is_empty()
            {
                return Some(cwd.trim().to_string());
            }
        }
    }
    None
}

fn extract_codex_cwd(path: &Path) -> Option<String> {
    let f = File::open(path).ok()?;
    let reader = BufReader::new(f);
    for line in reader.lines().take(8).flatten() {
        if !line.contains("\"cwd\"") {
            continue;
        }
        let Ok(val) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        if let Some(cwd) = val.pointer("/payload/cwd").and_then(Value::as_str)
            && !cwd.trim().is_empty()
        {
            return Some(cwd.trim().to_string());
        }
        if let Some(cwd) = val.get("cwd").and_then(Value::as_str)
            && !cwd.trim().is_empty()
        {
            return Some(cwd.trim().to_string());
        }
    }
    None
}

fn modified_ms(path: &Path) -> i64 {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn discovers_git_claude_and_codex_workspaces_without_duplicates() {
        let root = std::env::temp_dir().join(format!("loom-ws-{}", Uuid::new_v4()));
        let repos = root.join("repos");
        let proj_a = repos.join("app-a");
        let proj_b = repos.join("app-b");
        fs::create_dir_all(proj_a.join(".git")).unwrap();
        fs::create_dir_all(&proj_b).unwrap();

        let claude_dir = root.join("claude").join("proj1");
        fs::create_dir_all(&claude_dir).unwrap();
        let line_a = serde_json::json!({ "cwd": proj_a.to_string_lossy(), "type": "user" });
        fs::write(claude_dir.join("s1.jsonl"), format!("{line_a}\n")).unwrap();

        let codex_dir = root.join("codex").join("2026");
        fs::create_dir_all(&codex_dir).unwrap();
        let line_b = serde_json::json!({ "type": "session_meta", "payload": { "cwd": proj_b.to_string_lossy() } });
        fs::write(codex_dir.join("rollout-1.jsonl"), format!("{line_b}\n")).unwrap();

        let found = discover_workspaces_from(
            std::slice::from_ref(&proj_a),
            Some(root.join("claude").as_path()),
            Some(root.join("codex").as_path()),
        );
        let a = found.iter().find(|w| w.name == "app-a").unwrap();
        assert!(a.is_git_repo);
        assert!(a.sources.contains(&"git".to_string()));
        assert!(a.sources.contains(&"claude".to_string()));

        let b = found.iter().find(|w| w.name == "app-b").unwrap();
        assert!(!b.is_git_repo);
        assert_eq!(b.sources, vec!["codex".to_string()]);

        let _ = fs::remove_dir_all(&root);
    }
}
