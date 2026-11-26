# Contribuir a Loom

Gracias por aportar a Loom. Sigue el flujo de Issues y Pull Requests para mantener trazabilidad y calidad.

## Flujo de trabajo

1. Abre o toma un Issue describiendo el problema o mejora.
2. Crea una rama desde `main` (`feat/...` o `fix/...`).
3. Ejecuta las verificaciones locales antes de abrir el PR:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm install
pnpm web:typecheck
pnpm web:test:i18n
pnpm web:test
pnpm web:build
```

## Reglas de codigo

- Cero `unsafe` en Rust salvo wrappers encapsulados en `process_group.rs`.
- TypeScript estricto: prohibido `any` y `@ts-ignore`.
- Usa solo variables CSS de `apps/web/src/styles/tokens.css`. Sin hex ni rgb sueltos en componentes `.tsx`.
- Cero emojis en codigo fuente o comentarios.
- Mantener paridad de claves en `apps/web/src/i18n/locales/*.json` (`es`, `en`, `pt-BR`).
