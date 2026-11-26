package dev.nsma.loom

import android.content.Context
import android.content.pm.PackageManager
import android.os.Build
import android.webkit.JavascriptInterface
import org.json.JSONObject

class LoomAndroidBridge(
    private val context: Context,
    private val soundSynthesizer: SoundSynthesizer,
    private val notificationHelper: NotificationHelper
) {

    @JavascriptInterface
    fun notify(title: String?, body: String?, kind: String?) {
        notificationHelper.notify(
            title = title.orEmpty(),
            body = body.orEmpty(),
            kind = kind.orEmpty().ifBlank { "info" }
        )
    }

    @JavascriptInterface
    fun playCue(cue: String?) {
        soundSynthesizer.play(cue.orEmpty())
    }

    @JavascriptInterface
    fun haptic(kind: String?) {
        notificationHelper.vibrate(kind.orEmpty().ifBlank { "tick" })
    }

    @JavascriptInterface
    fun getDeviceInfo(): String {
        val appVersion = try {
            val info = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                context.packageManager.getPackageInfo(
                    context.packageName,
                    PackageManager.PackageInfoFlags.of(0)
                )
            } else {
                @Suppress("DEPRECATION")
                context.packageManager.getPackageInfo(context.packageName, 0)
            }
            info.versionName ?: "0.1.0"
        } catch (_: Throwable) {
            "0.1.0"
        }

        return JSONObject().apply {
            put("platform", "android")
            put("appVersion", appVersion)
            put("sdkInt", Build.VERSION.SDK_INT)
            put("manufacturer", Build.MANUFACTURER)
            put("model", Build.MODEL)
            put("muted", soundSynthesizer.muted)
        }.toString()
    }
}
