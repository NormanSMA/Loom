package dev.nsma.loom

import java.net.URI
import java.net.URLEncoder
import java.nio.charset.StandardCharsets

object EndpointConfig {
    const val DEFAULT_ENDPOINT = "http://127.0.0.1:4320"

    private val ALLOWED_HTTP_HOSTS = setOf("127.0.0.1", "localhost")

    fun normalize(raw: String): Result<String> {
        val trimmed = raw.trim()
        if (trimmed.isEmpty()) {
            return Result.failure(IllegalArgumentException("La URL del nodo no puede estar vacía."))
        }

        val uri = try {
            URI(trimmed)
        } catch (error: Exception) {
            return Result.failure(IllegalArgumentException("Formato de URL inválido.", error))
        }

        val scheme = uri.scheme?.lowercase()
            ?: return Result.failure(IllegalArgumentException("La URL debe incluir http:// o https://."))

        if (scheme != "http" && scheme != "https") {
            return Result.failure(IllegalArgumentException("Esquema no permitido: $scheme."))
        }

        if (!uri.userInfo.isNullOrEmpty()) {
            return Result.failure(IllegalArgumentException("No se permiten credenciales en la URL."))
        }

        val host = uri.host?.lowercase()
        if (host.isNullOrBlank()) {
            return Result.failure(IllegalArgumentException("La URL debe incluir un host válido."))
        }

        if (scheme == "http" && host !in ALLOWED_HTTP_HOSTS) {
            return Result.failure(
                IllegalArgumentException("HTTP sin cifrar solo se permite en 127.0.0.1 o localhost.")
            )
        }

        val rawPath = uri.rawPath.orEmpty().trimEnd('/')
        val portSegment = if (uri.port != -1) ":${uri.port}" else ""
        return Result.success("$scheme://$host$portSegment$rawPath")
    }

    fun buildPairUrl(endpoint: String, pairCode: String): String {
        val base = normalize(endpoint).getOrDefault(DEFAULT_ENDPOINT)
        val encoded = URLEncoder.encode(pairCode.trim(), StandardCharsets.UTF_8.name())
        val separator = if (base.contains("?")) "&" else "?"
        return "$base${separator}pair=$encoded"
    }
}
