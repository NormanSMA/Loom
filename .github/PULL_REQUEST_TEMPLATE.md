## Resumen

Describe que cambia este PR y enlaza el Issue (`Closes #...`).

## Lista de verificacion

- [ ] `cargo fmt --all -- --check` en verde
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` en verde
- [ ] `cargo test --workspace` en verde
- [ ] `pnpm web:typecheck && pnpm web:test:i18n && pnpm web:test && pnpm web:build` en verde
- [ ] Sin secretos, claves ni archivos `.db` en el diff
