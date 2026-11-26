#![forbid(unsafe_code)]

use loom_core::config::load_config;
use loomd::server::{AppState, build_router};
use std::io::Write;
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::time::Duration;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent};

const LOOMD_ADDR: &str = "127.0.0.1:4320";
const MAIN_URL: &str = "http://127.0.0.1:4320";
const ISLAND_URL: &str = "http://127.0.0.1:4320/?surface=island";
const ISLAND_WIDTH: f64 = 360.0;
const ISLAND_HEIGHT: f64 = 96.0;
const ISLAND_TOP_MARGIN: f64 = 6.0;

fn is_loomd_listening(addr: SocketAddr) -> bool {
    TcpStream::connect_timeout(&addr, Duration::from_millis(150)).is_ok()
}

fn resolve_static_dir(app: &AppHandle) -> Option<PathBuf> {
    let mut candidates = vec![
        PathBuf::from("apps/web/dist"),
        PathBuf::from("../../web/dist"),
    ];
    if let Ok(resource_dir) = app.path().resource_dir() {
        candidates.push(resource_dir.join("dist"));
        candidates.push(resource_dir.join("_up_/web/dist"));
        candidates.push(resource_dir);
    }
    candidates
        .into_iter()
        .find(|dir| dir.join("index.html").is_file())
}

fn ensure_loomd_running(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = LOOMD_ADDR.parse()?;
    if is_loomd_listening(addr) {
        return Ok(());
    }

    let static_dir = resolve_static_dir(app);
    let cfg = load_config(None)?;

    std::thread::spawn(move || {
        let Ok(rt) = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
        else {
            return;
        };
        rt.block_on(async move {
            let Ok(listener) = tokio::net::TcpListener::bind(addr).await else {
                return;
            };
            let router = build_router(AppState::new(cfg, false), static_dir);
            let _ = axum::serve(listener, router).await;
        });
    });

    for _ in 0..40 {
        if is_loomd_listening(addr) {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    Ok(())
}

fn show_main_window(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.unminimize();
        let _ = win.set_focus();
    }
}

fn toggle_island_window(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("island") {
        let visible = win.is_visible().unwrap_or(false);
        if visible {
            let _ = win.hide();
        } else {
            let _ = win.show();
            let _ = win.set_focus();
        }
    }
}

fn send_emergency_stop() {
    std::thread::spawn(|| {
        let Ok(addr) = LOOMD_ADDR.parse::<SocketAddr>() else {
            return;
        };
        if let Ok(mut stream) = TcpStream::connect_timeout(&addr, Duration::from_millis(500)) {
            let req = b"POST /api/emergency-stop HTTP/1.1\r\nHost: 127.0.0.1:4320\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
            let _ = stream.write_all(req);
            let _ = stream.flush();
        }
    });
}

fn setup_windows(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let main_url: tauri::Url = MAIN_URL.parse()?;
    let main_win = WebviewWindowBuilder::new(app, "main", WebviewUrl::External(main_url))
        .title("Loom — Telar")
        .inner_size(1240.0, 820.0)
        .min_inner_size(375.0, 640.0)
        .resizable(true)
        .visible(true)
        .build()?;
    let _ = main_win.show();
    let _ = main_win.unminimize();
    let _ = main_win.set_focus();

    let main_win_handle = main_win.clone();
    main_win.on_window_event(move |event| {
        if let WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let _ = main_win_handle.hide();
        }
    });

    let island_url: tauri::Url = ISLAND_URL.parse()?;
    let mut island_builder =
        WebviewWindowBuilder::new(app, "island", WebviewUrl::External(island_url))
            .title("Loom Island")
            .inner_size(ISLAND_WIDTH, ISLAND_HEIGHT)
            .decorations(false)
            .transparent(true)
            .always_on_top(true)
            .skip_taskbar(true)
            .resizable(false)
            .shadow(false)
            .visible(true);

    if let Ok(Some(monitor)) = app.primary_monitor() {
        let scale = monitor.scale_factor();
        let size = monitor.size();
        let pos = monitor.position();
        let x = (f64::from(pos.x) / scale) + ((f64::from(size.width) / scale) - ISLAND_WIDTH) / 2.0;
        let y = (f64::from(pos.y) / scale) + ISLAND_TOP_MARGIN;
        island_builder = island_builder.position(x, y);
    }

    let island_win = island_builder.build()?;
    let _ = island_win.show();
    Ok(())
}

fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let show_item = MenuItem::with_id(app, "show", "Mostrar Loom", true, None::<&str>)?;
    let island_item = MenuItem::with_id(
        app,
        "toggle-island",
        "Mostrar / Ocultar isla",
        true,
        None::<&str>,
    )?;
    let stop_item = MenuItem::with_id(app, "stop", "Parada total", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Salir", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show_item, &island_item, &stop_item, &quit_item])?;

    let mut builder = TrayIconBuilder::with_id("loom-tray")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => show_main_window(app),
            "toggle-island" => toggle_island_window(app),
            "stop" => send_emergency_stop(),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        });

    if let Some(icon) = app.default_window_icon().cloned() {
        builder = builder.icon(icon);
    }

    builder.build(app)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle();
            ensure_loomd_running(handle)?;
            setup_windows(handle)?;
            setup_tray(handle)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error al iniciar Loom Desktop");
}
