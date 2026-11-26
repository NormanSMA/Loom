use anyhow::Result;
use clap::Parser;
use loom_core::config::load_config;
use loomd::demo::demo_live_event;
use loomd::server::{AppState, build_router};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Parser)]
#[command(name = "loomd", version, about = "Daemon local de Loom (Telar)")]
struct Args {
    #[arg(long, help = "Inicia con datos ficticios en el puerto demo")]
    demo: bool,

    #[arg(long, help = "Puerto de escucha")]
    port: Option<u16>,

    #[arg(long, help = "Carpeta de archivos estaticos web/PWA")]
    static_dir: Option<PathBuf>,

    #[arg(long, help = "Ruta alternativa a config.toml")]
    config: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();
    let cfg = load_config(args.config.as_deref())?;
    let port = args
        .port
        .unwrap_or(if args.demo { cfg.demo_port } else { cfg.port });
    let addr = format!("{}:{port}", cfg.bind);

    let state = AppState::new(cfg, args.demo);
    if args.demo {
        let tx = state.events_tx.clone();
        tokio::spawn(async move {
            let mut step = 0_usize;
            let mut ticker = tokio::time::interval(Duration::from_secs(3));
            loop {
                ticker.tick().await;
                let _ = tx.send(demo_live_event(step));
                step = step.wrapping_add(1);
            }
        });
    }

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("loomd escuchando en http://{addr} (demo={})", args.demo);

    let app = build_router(state, args.static_dir);
    axum::serve(listener, app).await?;
    Ok(())
}
