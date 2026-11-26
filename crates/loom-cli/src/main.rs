use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use loom_core::capsule::{CapsuleRequest, compile_capsule};
use loom_core::config::{default_providers_dir, load_config};
use loom_core::providers::ProviderRegistry;
use loom_core::router::{TaskRole, read_claudehub_quota, route_task};
use loom_core::skills::SkillIndex;
use loom_core::types::{DoctorStatus, Sensitivity};
use loom_drivers::doctor::run_doctor;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "loom",
    version,
    about = "Loom (Telar) - Orquestador local multiagente"
)]
struct Cli {
    #[arg(long, global = true, help = "Ruta alternativa a config.toml")]
    config: Option<PathBuf>,

    #[arg(long, global = true, help = "Emite salida JSON")]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(about = "Verifica herramientas, conectores y servicios locales")]
    Doctor,
    #[command(about = "Lista los proveedores registrados (nativos y personalizados)")]
    Providers {
        #[command(subcommand)]
        action: ProvidersAction,
    },
    #[command(about = "Valida la configuracion activa")]
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    #[command(about = "Lista o busca skills indexadas")]
    Skills {
        #[command(subcommand)]
        action: SkillsAction,
    },
    #[command(about = "Elige el proveedor optimo segun rol, sensibilidad y cuotas")]
    Route {
        #[arg(long)]
        role: String,
        #[arg(long, default_value = "internal")]
        sensitivity: String,
    },
    #[command(about = "Compila una capsula corta de contexto para un repositorio")]
    Capsule {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        objective: String,
        #[arg(long)]
        slug: Option<String>,
        #[arg(long)]
        allowed: Vec<String>,
        #[arg(long)]
        verify: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
enum ProvidersAction {
    List,
}

#[derive(Debug, Subcommand)]
enum ConfigAction {
    Check,
}

#[derive(Debug, Subcommand)]
enum SkillsAction {
    List {
        #[arg(long, default_value = "")]
        query: String,
        #[arg(long)]
        repo: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();
    let cfg = load_config(cli.config.as_deref())?;

    match cli.command {
        Commands::Doctor => {
            let registry = ProviderRegistry::load(&default_providers_dir(), &cfg.omniroute_url)?;
            let report = run_doctor(&cfg, &registry).await;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!(
                    "Loom Doctor ({}/{}) - ok: {}, aviso: {}, no disp: {}",
                    report.os,
                    report.arch,
                    report.summary.ok,
                    report.summary.warning,
                    report.summary.unavailable
                );
                for item in &report.checks {
                    let tag = match item.status {
                        DoctorStatus::Ok => "[OK]",
                        DoctorStatus::Warning => "[AVISO]",
                        DoctorStatus::Unavailable => "[NO DISP]",
                    };
                    let ver = item.version.as_deref().unwrap_or("-");
                    println!(
                        "{tag:<9} {:<13} {:<16} {} ({} ms)",
                        item.name, item.id, ver, item.latency_ms
                    );
                    if let Some(loc) = &item.location {
                        println!("          ruta: {loc}");
                    }
                    for w in &item.warnings {
                        println!("          aviso: {w}");
                    }
                }
            }
        }
        Commands::Providers {
            action: ProvidersAction::List,
        } => {
            let registry = ProviderRegistry::load(&default_providers_dir(), &cfg.omniroute_url)?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(registry.all())?);
            } else {
                println!("Proveedores registrados ({}):", registry.all().len());
                for p in registry.all() {
                    let state = if p.enabled { "activo" } else { "inactivo" };
                    println!(
                        "- {:<12} {:<14} {:?} | {:?} | {:?} | {}",
                        p.id, p.name, p.tier, p.kind, p.privacy, state
                    );
                }
            }
        }
        Commands::Config {
            action: ConfigAction::Check,
        } => {
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&cfg)?);
            } else {
                println!("Configuracion valida:");
                println!(
                    "  bind: {}:{} (demo: {})",
                    cfg.bind, cfg.port, cfg.demo_port
                );
                println!("  idioma: {:?}", cfg.locale);
                println!("  remoto: {}", cfg.remote_enabled);
                println!("  claudehub: {}", cfg.claudehub_url);
                println!("  omniroute: {}", cfg.omniroute_url);
            }
        }
        Commands::Skills {
            action: SkillsAction::List { query, repo },
        } => {
            let index = SkillIndex::discover(repo.as_deref());
            let matches = index.search(&query);
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&matches)?);
            } else {
                println!("Skills encontradas ({}):", matches.len());
                for s in matches.iter().take(40) {
                    println!("- [{}] {}: {}", s.source, s.name, s.description);
                }
                if matches.len() > 40 {
                    println!("... y {} mas (usa --query o --json)", matches.len() - 40);
                }
            }
        }
        Commands::Route { role, sensitivity } => {
            let parsed_role: TaskRole = serde_json::from_str(&format!("\"{role}\""))
                .with_context(|| format!("rol invalido: {role}"))?;
            let parsed_sens: Sensitivity = serde_json::from_str(&format!("\"{sensitivity}\""))
                .with_context(|| format!("sensibilidad invalida: {sensitivity}"))?;
            let registry = ProviderRegistry::load(&default_providers_dir(), &cfg.omniroute_url)?;
            let quota = read_claudehub_quota(None);
            let decision = route_task(parsed_role, parsed_sens, &registry, &quota)?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&decision)?);
            } else {
                println!("Proveedor elegido: {}", decision.provider_id);
                println!("Motivo: {}", decision.reason);
                if !decision.fallback_chain.is_empty() {
                    println!("Respaldos: {}", decision.fallback_chain.join(" -> "));
                }
            }
        }
        Commands::Capsule {
            repo,
            objective,
            slug,
            allowed,
            verify,
        } => {
            let cap = compile_capsule(&CapsuleRequest {
                repo_root: repo,
                vault_path: cfg.vault_path.clone(),
                project_slug: slug,
                objective,
                allowed_files: allowed,
                verification_cmd: verify,
                skill_names: Vec::new(),
            });
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&cap)?);
            } else {
                println!("Modo memoria: {}", cap.memory_mode);
                println!("{}", cap.rendered_prompt);
            }
        }
    }

    Ok(())
}
