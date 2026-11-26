use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesignTokensSummary {
    pub package_manager: Option<String>,
    pub has_tailwind: bool,
    pub tailwind_config: Option<String>,
    pub globals_css: Option<String>,
    pub components_json: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CapsuleRequest {
    pub repo_root: PathBuf,
    pub vault_path: Option<PathBuf>,
    pub project_slug: Option<String>,
    pub objective: String,
    pub allowed_files: Vec<String>,
    pub verification_cmd: Option<String>,
    pub skill_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextCapsule {
    pub project_name: String,
    pub memory_mode: String,
    pub contract_path: Option<PathBuf>,
    pub status_path: Option<PathBuf>,
    pub decisions_path: Option<PathBuf>,
    pub design_tokens: DesignTokensSummary,
    pub allowed_files: Vec<String>,
    pub rendered_prompt: String,
}

pub fn compile_capsule(req: &CapsuleRequest) -> ContextCapsule {
    let project_name = req
        .repo_root
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("proyecto")
        .to_string();
    let slug = req
        .project_slug
        .clone()
        .unwrap_or_else(|| project_name.to_ascii_lowercase());

    let (memory_mode, contract_path, status_path, decisions_path) =
        resolve_memory_files(&req.repo_root, req.vault_path.as_deref(), &slug);
    let design_tokens = inspect_design_tokens(&req.repo_root);

    let mut lines = Vec::new();
    lines.push(format!("Objetivo: {}", req.objective.trim()));
    if !req.allowed_files.is_empty() {
        lines.push(format!("Archivos: {}", req.allowed_files.join(", ")));
    }
    if let Some(p) = &contract_path {
        lines.push(format!("Contrato: {}", p.display()));
    }
    if let Some(p) = &status_path {
        lines.push(format!("Estado: {}", p.display()));
    }
    if let Some(p) = &decisions_path {
        lines.push(format!("Decisiones: {}", p.display()));
    }
    if let Some(pm) = &design_tokens.package_manager {
        lines.push(format!("Gestor: {pm}"));
    }
    if design_tokens.has_tailwind {
        let refs: Vec<&str> = [
            design_tokens.tailwind_config.as_deref(),
            design_tokens.globals_css.as_deref(),
            design_tokens.components_json.as_deref(),
        ]
        .into_iter()
        .flatten()
        .collect();
        if !refs.is_empty() {
            lines.push(format!("Tokens UI: {}", refs.join(", ")));
        }
    }
    if !req.skill_names.is_empty() {
        lines.push(format!("Skills: {}", req.skill_names.join(", ")));
    }
    if let Some(cmd) = &req.verification_cmd {
        lines.push(format!("Hecho cuando: {cmd} pasa con codigo 0."));
    }
    lines.push(
        "Limites: no toques archivos fuera de Archivos; sin commit; informe <= 12 lineas.".into(),
    );

    ContextCapsule {
        project_name,
        memory_mode,
        contract_path,
        status_path,
        decisions_path,
        design_tokens,
        allowed_files: req.allowed_files.clone(),
        rendered_prompt: lines.join("\n"),
    }
}

fn resolve_memory_files(
    repo_root: &Path,
    vault_path: Option<&Path>,
    slug: &str,
) -> (String, Option<PathBuf>, Option<PathBuf>, Option<PathBuf>) {
    if let Some(vault) = vault_path
        && vault.is_dir()
    {
        let contract = existing_file(vault.join("wiki/ia/contrato-agentes.md"));
        let status = existing_file(vault.join(format!("wiki/projects/{slug}/status.md")));
        let decisions = existing_file(vault.join(format!("wiki/projects/{slug}/decisiones.md")));
        return ("obsidian".to_string(), contract, status, decisions);
    }

    let loom_dir = repo_root.join(".loom");
    let contract = existing_file(loom_dir.join("contract.md"));
    let status = existing_file(loom_dir.join("status.md"));
    let decisions = existing_file(loom_dir.join("decisions.md"));
    ("loom_local".to_string(), contract, status, decisions)
}

fn existing_file(path: PathBuf) -> Option<PathBuf> {
    path.is_file().then_some(path)
}

fn inspect_design_tokens(repo_root: &Path) -> DesignTokensSummary {
    let package_manager = if repo_root.join("pnpm-lock.yaml").is_file() {
        Some("pnpm".to_string())
    } else if repo_root.join("yarn.lock").is_file() {
        Some("yarn".to_string())
    } else if repo_root.join("package-lock.json").is_file() {
        Some("npm".to_string())
    } else if repo_root.join("Cargo.toml").is_file() {
        Some("cargo".to_string())
    } else {
        None
    };

    let tailwind_config = [
        "tailwind.config.ts",
        "tailwind.config.js",
        "tailwind.config.mjs",
        "tailwind.config.cjs",
    ]
    .into_iter()
    .find(|rel| repo_root.join(rel).is_file())
    .map(str::to_string);

    let globals_css = [
        "src/app/globals.css",
        "src/globals.css",
        "src/index.css",
        "app/globals.css",
    ]
    .into_iter()
    .find(|rel| repo_root.join(rel).is_file())
    .map(str::to_string);

    let components_json = repo_root
        .join("components.json")
        .is_file()
        .then(|| "components.json".to_string());

    let pkg_has_tailwind = std::fs::read_to_string(repo_root.join("package.json"))
        .map(|s| s.contains("tailwindcss"))
        .unwrap_or(false);

    let has_tailwind = tailwind_config.is_some() || pkg_has_tailwind;

    DesignTokensSummary {
        package_manager,
        has_tailwind,
        tailwind_config,
        globals_css,
        components_json,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("loom-cap-{}-{}", std::process::id(), name));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn compiles_local_loom_capsule_and_detects_ui_tokens() {
        let root = temp_dir("local");
        let repo = root.join("mi-app");
        std::fs::create_dir_all(repo.join(".loom")).unwrap();
        std::fs::create_dir_all(repo.join("src")).unwrap();
        std::fs::write(repo.join(".loom/status.md"), "Estado actual").unwrap();
        std::fs::write(repo.join("pnpm-lock.yaml"), "").unwrap();
        std::fs::write(repo.join("tailwind.config.ts"), "").unwrap();
        std::fs::write(repo.join("src/globals.css"), "").unwrap();

        let cap = compile_capsule(&CapsuleRequest {
            repo_root: repo,
            vault_path: None,
            project_slug: None,
            objective: "Crear boton accesible".into(),
            allowed_files: vec!["src/Button.tsx".into()],
            verification_cmd: Some("pnpm test".into()),
            skill_names: vec!["ui-ux".into()],
        });

        assert_eq!(cap.memory_mode, "loom_local");
        assert!(cap.status_path.is_some());
        assert_eq!(cap.design_tokens.package_manager.as_deref(), Some("pnpm"));
        assert!(cap.design_tokens.has_tailwind);
        assert!(
            cap.rendered_prompt
                .contains("Tokens UI: tailwind.config.ts, src/globals.css")
        );
        assert!(cap.rendered_prompt.lines().count() <= 15);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn compiles_obsidian_vault_capsule() {
        let root = temp_root("obsidian");
        let repo = root.join("Loom");
        let vault = root.join("vault");
        std::fs::create_dir_all(&repo).unwrap();
        std::fs::create_dir_all(vault.join("wiki/ia")).unwrap();
        std::fs::create_dir_all(vault.join("wiki/projects/loom")).unwrap();
        std::fs::write(vault.join("wiki/ia/contrato-agentes.md"), "reglas").unwrap();
        std::fs::write(vault.join("wiki/projects/loom/status.md"), "ok").unwrap();

        let cap = compile_capsule(&CapsuleRequest {
            repo_root: repo,
            vault_path: Some(vault),
            project_slug: Some("loom".into()),
            objective: "Implementar MCP".into(),
            allowed_files: vec![],
            verification_cmd: None,
            skill_names: vec![],
        });

        assert_eq!(cap.memory_mode, "obsidian");
        assert!(cap.contract_path.is_some());
        assert!(cap.status_path.is_some());

        let _ = std::fs::remove_dir_all(&root);
    }

    fn temp_root(name: &str) -> PathBuf {
        temp_dir(name)
    }
}
