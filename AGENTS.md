# Loom para agentes de IA

Loom (Telar) es un orquestador local multiagente en Rust, Tauri v2 y Web/PWA para coordinar Claude Code, Codex CLI, Gemini CLI, Antigravity, OpenCode, OmniRoute y proveedores personalizados.

## Comandos principales

| Objetivo | Comando | Direccion |
|---|---|---|
| Verificar configuracion | `cargo run -p loom-cli -- config check` | stdout |
| Diagnosticar herramientas | `cargo run -p loom-cli -- doctor` | stdout |
| Enrutar por cuota y privacidad | `cargo run -p loom-cli -- route --role implementer --sensitivity internal` | stdout |
| Compilar capsula de contexto | `cargo run -p loom-cli -- capsule --repo . --objective "..."` | stdout |
| Daemon en modo demo | `cargo run -p loomd -- --demo --static-dir apps/web/dist` | `http://127.0.0.1:4321` |
| Daemon real | `cargo run -p loomd -- --static-dir apps/web/dist` | `http://127.0.0.1:4320` |
| Servidor MCP stdio | `cargo run -p loom-mcp` | JSON-RPC 2.0 |

## Reglas de seguridad y arquitectura

- El servidor escucha solo en `127.0.0.1` salvo activacion explicita de `remote_enabled`.
- Rechaza con `403` cabeceras `Host` no loopback cuando `remote_enabled = false`.
- `PrivacyGate` bloquea el envio de tareas `confidential` o rutas `.env*`/`.pem`/`.key` hacia proveedores `free` o `anonymous`.
- Nunca interceptar ni reenviar credenciales OAuth de Antigravity/Google.
- Verifica todo cambio con `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `pnpm web:typecheck`, `pnpm web:test` y `pnpm web:build`.
