use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SkillError {
    #[error("no se encontro la skill '{0}'")]
    NotFound(String),
    #[error("no se pudo leer la skill en {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillEntry {
    pub name: String,
    pub description: String,
    pub path: PathBuf,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct SkillIndex {
    entries: Vec<SkillEntry>,
}

impl SkillIndex {
    pub fn discover(project_root: Option<&Path>) -> Self {
        let mut roots = Vec::new();
        if let Some(root) = project_root {
            roots.push(("project".to_string(), root.join(".loom").join("skills")));
        }
        if let Some(home) = dirs::home_dir() {
            roots.push(("claude".to_string(), home.join(".claude").join("skills")));
            roots.push(("agents".to_string(), home.join(".agents").join("skills")));
            roots.push(("gemini".to_string(), home.join(".gemini").join("skills")));
        }
        Self::discover_from_dirs(&roots)
    }

    pub fn discover_from_dirs(roots: &[(String, PathBuf)]) -> Self {
        let mut by_name: BTreeMap<String, SkillEntry> = BTreeMap::new();

        for (source, dir) in roots {
            if !dir.is_dir() {
                continue;
            }
            let Ok(read_dir) = std::fs::read_dir(dir) else {
                continue;
            };
            let mut subdirs: Vec<PathBuf> = read_dir
                .filter_map(Result::ok)
                .map(|e| e.path())
                .filter(|p| p.is_dir())
                .collect();
            subdirs.sort();

            for subdir in subdirs {
                let skill_file = subdir.join("SKILL.md");
                if !skill_file.is_file() {
                    continue;
                }
                let Ok(raw) = std::fs::read_to_string(&skill_file) else {
                    continue;
                };
                let fallback_name = subdir
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("skill")
                    .to_string();
                let (name, description) = parse_frontmatter(&raw, &fallback_name);
                by_name.entry(name.clone()).or_insert(SkillEntry {
                    name,
                    description,
                    path: skill_file,
                    source: source.clone(),
                });
            }
        }

        Self {
            entries: by_name.into_values().collect(),
        }
    }

    pub fn all(&self) -> &[SkillEntry] {
        &self.entries
    }

    pub fn search(&self, query: &str) -> Vec<&SkillEntry> {
        let q = query.trim().to_ascii_lowercase();
        if q.is_empty() {
            return self.entries.iter().collect();
        }
        self.entries
            .iter()
            .filter(|e| {
                e.name.to_ascii_lowercase().contains(&q)
                    || e.description.to_ascii_lowercase().contains(&q)
            })
            .collect()
    }

    pub fn load_content(&self, name: &str) -> Result<String, SkillError> {
        let entry = self
            .entries
            .iter()
            .find(|e| e.name == name)
            .ok_or_else(|| SkillError::NotFound(name.to_string()))?;

        std::fs::read_to_string(&entry.path).map_err(|source| SkillError::Io {
            path: entry.path.clone(),
            source,
        })
    }
}

fn parse_frontmatter(raw: &str, fallback_name: &str) -> (String, String) {
    let normalized = raw.replace("\r\n", "\n");
    let mut lines = normalized.lines();
    if lines.next().map(str::trim) != Some("---") {
        return (fallback_name.to_string(), String::new());
    }

    let mut name: Option<String> = None;
    let mut description: Option<String> = None;
    let mut reading_desc_block = false;

    for line in lines {
        let trimmed = line.trim();
        if trimmed == "---" {
            break;
        }
        if reading_desc_block {
            if line.starts_with(' ') || line.starts_with('\t') {
                if !trimmed.is_empty() {
                    let existing = description.get_or_insert_with(String::new);
                    if !existing.is_empty() {
                        existing.push(' ');
                    }
                    existing.push_str(&strip_quotes(trimmed));
                }
                continue;
            }
            reading_desc_block = false;
        }
        if let Some(val) = trimmed.strip_prefix("name:") {
            let clean = strip_quotes(val.trim());
            if !clean.is_empty() {
                name = Some(clean);
            }
        } else if let Some(val) = trimmed.strip_prefix("description:") {
            let clean = strip_quotes(val.trim());
            if matches!(clean.as_str(), ">" | ">-" | "|" | "|-") {
                reading_desc_block = true;
            } else if !clean.is_empty() {
                description = Some(clean);
            }
        }
    }

    (
        name.unwrap_or_else(|| fallback_name.to_string()),
        description.unwrap_or_default(),
    )
}

fn strip_quotes(s: &str) -> String {
    s.trim_matches('"').trim_matches('\'').trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(label: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("loom-skills-{}-{}", std::process::id(), label));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn discovers_with_priority_dedup_and_search() {
        let root = temp_root("prio");
        let proj = root.join("proj");
        let global = root.join("global");
        std::fs::create_dir_all(proj.join("ui-design")).unwrap();
        std::fs::create_dir_all(global.join("ui-design")).unwrap();
        std::fs::create_dir_all(global.join("rust-audit")).unwrap();

        std::fs::write(
            proj.join("ui-design/SKILL.md"),
            "---\nname: ui-design\ndescription: \"Proyecto local\"\n---\nCuerpo proyecto",
        )
        .unwrap();
        std::fs::write(
            global.join("ui-design/SKILL.md"),
            "---\nname: ui-design\ndescription: \"Global\"\n---\nCuerpo global",
        )
        .unwrap();
        std::fs::write(
            global.join("rust-audit/SKILL.md"),
            "---\nname: rust-audit\ndescription: Revisar crates Rust\n---\nContenido audit",
        )
        .unwrap();

        let index =
            SkillIndex::discover_from_dirs(&[("project".into(), proj), ("agents".into(), global)]);
        assert_eq!(index.all().len(), 2);

        let ui = index.search("proyecto");
        assert_eq!(ui.len(), 1);
        assert_eq!(ui[0].source, "project");
        assert!(
            index
                .load_content("ui-design")
                .unwrap()
                .contains("Cuerpo proyecto")
        );
        assert!(index.load_content("inexistente").is_err());

        let _ = std::fs::remove_dir_all(&root);
    }
}
