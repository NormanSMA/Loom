# Politica de seguridad

## Modelo de confianza

- `loomd` escucha exclusivamente en `127.0.0.1:4320` (`4321` en modo demo) por defecto.
- La capa `Host` rechaza con `403` solicitudes no loopback cuando `remote_enabled = false`.
- Los tickets QR de emparejamiento expiran en 120 segundos y son de un solo uso.
- Los tokens de sesion se almacenan en memoria hasheados con SHA-256 y pueden revocarse por dispositivo.
- `PrivacyGate` impide enrutar tareas confidenciales hacia modelos gratuitos o anonimos.

## Reporte de vulnerabilidades

Reporta cualquier hallazgo de seguridad abriendo un aviso privado en GitHub Security Advisories para `NormanSMA/Loom` sin publicar credenciales ni detalles reproducibles en issues publicos.
