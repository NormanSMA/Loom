package dev.nsma.loom

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class EndpointConfigTest {

    @Test
    fun defaultEndpointIsLoopback() {
        assertEquals("http://127.0.0.1:4320", EndpointConfig.DEFAULT_ENDPOINT)
    }

    @Test
    fun normalizesTrailingSlashes() {
        val local = EndpointConfig.normalize("  http://127.0.0.1:4320///  ")
        assertTrue(local.isSuccess)
        assertEquals("http://127.0.0.1:4320", local.getOrThrow())

        val remote = EndpointConfig.normalize("HTTPS://Loom.Example.com/app///")
        assertTrue(remote.isSuccess)
        assertEquals("https://loom.example.com/app", remote.getOrThrow())
    }

    @Test
    fun allowsHttpForLoopback() {
        val ip = EndpointConfig.normalize("http://127.0.0.1:4320")
        val localhost = EndpointConfig.normalize("http://localhost:4320")

        assertTrue(ip.isSuccess)
        assertTrue(localhost.isSuccess)
        assertEquals("http://localhost:4320", localhost.getOrThrow())
    }

    @Test
    fun rejectsHttpForRemoteHosts() {
        val remoteHttp = EndpointConfig.normalize("http://example.com:4320")
        val lanHttp = EndpointConfig.normalize("http://192.168.1.50:4320")

        assertTrue(remoteHttp.isFailure)
        assertTrue(lanHttp.isFailure)
    }

    @Test
    fun rejectsJavascriptAndNonHttpSchemes() {
        assertTrue(EndpointConfig.normalize("javascript:alert(1)").isFailure)
        assertTrue(EndpointConfig.normalize("file:///etc/passwd").isFailure)
        assertTrue(EndpointConfig.normalize("ftp://127.0.0.1:4320").isFailure)
        assertTrue(EndpointConfig.normalize("").isFailure)
    }
}
