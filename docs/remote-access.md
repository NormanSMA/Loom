# Acceso remoto seguro para Loom (Telar)

Por defecto, `loomd` escucha solo en `127.0.0.1:4320` y bloquea con HTTP `403` cualquier cabecera `Host` ajena a loopback (`remote_enabled = false`).

Para operar Loom desde iPhone, Android u otro equipo sin exponer puertos al router, publica el socket loopback mediante un tunel privado.

---

## Opcion 1 — Tailscale Serve (recomendada para uso personal)

1. Instala Tailscale en tu equipo y en tu telefono con tu misma tailnet.
2. Activa `remote_enabled = true` en `~/.loom/config.toml` manteniendo `bind = "127.0.0.1"`:

```toml
bind = "127.0.0.1"
port = 4320
remote_enabled = true
```

3. Publica `loomd` dentro de tu red privada Tailscale con HTTPS automatico:

```bash
tailscale serve --bg --https=443 http://127.0.0.1:4320
```

4. Abre la URL `https://<tu-equipo>.<tu-tailnet>.ts.net` en el telefono e instala la PWA en pantalla de inicio.

---

## Opcion 2 — Cloudflare Tunnel + Cloudflare Access

1. Mantén `bind = "127.0.0.1"` y fija `remote_enabled = true` en `~/.loom/config.toml`.
2. Crea un tunel con `cloudflared` apuntando a `http://127.0.0.1:4320`:

```bash
cloudflared tunnel run --url http://127.0.0.1:4320 loom-node
```

3. Protege el subdominio en Cloudflare Zero Trust Access exigiendo tu correo o proveedor de identidad antes de reenviar trafico a `loomd`.

---

## Lista de verificacion de seguridad

1. Registra TOTP desde **Seguridad -> Configurar TOTP** (`POST /api/auth/totp/setup` y `/verify`) antes de activar acceso remoto.
2. Empareja dispositivos moviles con tickets QR de un solo uso (TTL 120 s).
3. Revoca sesiones perdidas o robadas al instante desde la lista de dispositivos (`DELETE /api/devices/{id}`).
4. Usa el boton **Parada total** (`POST /api/emergency-stop`) para cancelar subagentes activos ante cualquier anomalia.
5. Nunca configures `bind = "0.0.0.0"` directo hacia internet publica sin TLS ni control de acceso.
