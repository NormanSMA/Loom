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

fn is_placeholder_title(title: &str) -> bool {
    let trimmed = title.trim();
    trimmed.is_empty()
        || trimmed.eq_ignore_ascii_case("sesión activa")
        || trimmed.eq_ignore_ascii_case("sesion activa")
}

fn is_unknown_model(model: &str) -> bool {
    let trimmed = model.trim();
    trimmed.is_empty() || trimmed.eq_ignore_ascii_case("unknown")
}

fn merge_chat(target: &mut ActiveChatView, incoming: ActiveChatView) {
    if target.source.trim().is_empty() && !incoming.source.trim().is_empty() {
        target.source = incoming.source.trim().to_string();
    }
    if target.session.trim().is_empty() && !incoming.session.trim().is_empty() {
        target.session = incoming.session.trim().to_string();
    }

    let target_title_placeholder = is_placeholder_title(&target.title);
    let incoming_title = incoming.title.trim();
    let incoming_title_placeholder = is_placeholder_title(incoming_title);
    if (target_title_placeholder && !incoming_title_placeholder)
        || (target.title.trim().is_empty() && !incoming_title.is_empty())
        || (!target_title_placeholder
            && !incoming_title_placeholder
            && incoming_title.len() > target.title.trim().len())
    {
        target.title = incoming_title.to_string();
    }

    let incoming_model = incoming.model.trim();
    if (is_unknown_model(&target.model) && !is_unknown_model(incoming_model))
        || (target.model.trim().is_empty() && !incoming_model.is_empty())
    {
        target.model = incoming_model.to_string();
    }

    target.working = target.working || incoming.working;
    if incoming.last_ts > target.last_ts {
        target.last_ts = incoming.last_ts;
    }
}

/// Colapsa entradas duplicadas por `(source, session)` o `(source, title)` normalizados y ordena por actividad.
pub fn dedup_active_chats(chats: &mut Vec<ActiveChatView>) {
    let input = std::mem::take(chats);
    let mut merged: Vec<ActiveChatView> = Vec::with_capacity(input.len());
    let mut by_session: HashMap<(String, String), usize> = HashMap::new();
    let mut by_title: HashMap<(String, String), usize> = HashMap::new();

    for mut chat in input {
        chat.source = chat.source.trim().to_string();
        chat.session = chat.session.trim().to_string();
        chat.title = chat.title.trim().to_string();
        chat.model = chat.model.trim().to_string();

        let source_norm = chat.source.to_ascii_lowercase();
        let session_norm = chat.session.clone();
        let title_norm = chat.title.to_ascii_lowercase();

        let existing_idx = if !session_norm.is_empty() {
            by_session
                .get(&(source_norm.clone(), session_norm.clone()))
                .copied()
        } else {
            None
        }
        .or_else(|| {
            if !title_norm.is_empty() {
                by_title
                    .get(&(source_norm.clone(), title_norm.clone()))
                    .copied()
            } else {
                None
            }
        });

        let idx = if let Some(idx) = existing_idx {
            merge_chat(&mut merged[idx], chat);
            idx
        } else {
            let idx = merged.len();
            merged.push(chat);
            idx
        };

        let entry = &merged[idx];
        let entry_source = entry.source.to_ascii_lowercase();
        if !entry.session.is_empty() {
            by_session.insert((entry_source.clone(), entry.session.clone()), idx);
        }
        if !entry.title.is_empty() {
            by_title.insert((entry_source, entry.title.to_ascii_lowercase()), idx);
        }
    }

    merged.sort_by(|a, b| {
        b.working
            .cmp(&a.working)
            .then_with(|| b.last_ts.cmp(&a.last_ts))
            .then_with(|| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
            .then_with(|| a.title.cmp(&b.title))
    });

    *chats = merged;
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

    #[test]
    fn dedups_active_chats_by_session_and_title() {
        let mut chats = vec![
            ActiveChatView {
                source: "claude".into(),
                session: "sess-1".into(),
                title: "Sesión activa".into(),
                model: "unknown".into(),
                working: false,
                last_ts: 100,
            },
            ActiveChatView {
                source: "Claude".into(),
                session: " sess-1 ".into(),
                title: "Implementar deduplicación".into(),
                model: "claude-sonnet-4-6".into(),
                working: true,
                last_ts: 250,
            },
            ActiveChatView {
                source: "claude".into(),
                session: String::new(),
                title: "  implementar deduplicación  ".into(),
                model: "unknown".into(),
                working: false,
                last_ts: 180,
            },
            ActiveChatView {
                source: "codex".into(),
                session: "codex-9".into(),
                title: "Implementar deduplicación".into(),
                model: "gpt-5.4".into(),
                working: false,
                last_ts: 300,
            },
            ActiveChatView {
                source: "gemini".into(),
                session: "gem-2".into(),
                title: "Revisar SVG".into(),
                model: "gemini-2.5-pro".into(),
                working: true,
                last_ts: 200,
            },
        ];

        dedup_active_chats(&mut chats);

        assert_eq!(chats.len(), 3);
        assert_eq!(chats[0].source, "claude");
        assert_eq!(chats[0].session, "sess-1");
        assert_eq!(chats[0].title, "Implementar deduplicación");
        assert_eq!(chats[0].model, "claude-sonnet-4-6");
        assert!(chats[0].working);
        assert_eq!(chats[0].last_ts, 250);

        assert_eq!(chats[1].source, "gemini");
        assert!(chats[1].working);
        assert_eq!(chats[1].last_ts, 200);

        assert_eq!(chats[2].source, "codex");
        assert!(!chats[2].working);
        assert_eq!(chats[2].last_ts, 300);
    }
}
