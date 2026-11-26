package dev.nsma.loom

import android.Manifest
import android.annotation.SuppressLint
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.view.View
import android.webkit.WebResourceError
import android.webkit.WebResourceRequest
import android.webkit.WebSettings
import android.webkit.WebView
import android.webkit.WebViewClient
import androidx.activity.OnBackPressedCallback
import androidx.activity.result.contract.ActivityResultContracts
import androidx.appcompat.app.AlertDialog
import androidx.appcompat.app.AppCompatActivity
import androidx.core.content.ContextCompat
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat
import com.google.android.material.button.MaterialButton
import com.google.android.material.dialog.MaterialAlertDialogBuilder
import com.google.android.material.textfield.TextInputEditText
import com.google.android.material.textfield.TextInputLayout
import dev.nsma.loom.databinding.ActivityMainBinding

class MainActivity : AppCompatActivity() {

    private lateinit var binding: ActivityMainBinding
    private lateinit var soundSynthesizer: SoundSynthesizer
    private lateinit var notificationHelper: NotificationHelper
    private lateinit var bridge: LoomAndroidBridge

    private var currentEndpoint: String = EndpointConfig.DEFAULT_ENDPOINT
    private var lastLoadFailed: Boolean = false

    private val notificationPermissionLauncher = registerForActivityResult(
        ActivityResultContracts.RequestPermission()
    ) { _ ->
        notificationHelper.ensureChannel()
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        binding = ActivityMainBinding.inflate(layoutInflater)
        setContentView(binding.root)
        ViewCompat.setOnApplyWindowInsetsListener(binding.root) { view, insets ->
            val bars = insets.getInsets(
                WindowInsetsCompat.Type.systemBars() or WindowInsetsCompat.Type.displayCutout()
            )
            view.setPadding(bars.left, bars.top, bars.right, bars.bottom)
            insets
        }

        val prefs = getSharedPreferences(PREFS_NAME, MODE_PRIVATE)
        currentEndpoint = EndpointConfig.normalize(
            prefs.getString(KEY_ENDPOINT, EndpointConfig.DEFAULT_ENDPOINT).orEmpty()
        ).getOrDefault(EndpointConfig.DEFAULT_ENDPOINT)

        soundSynthesizer = SoundSynthesizer().apply {
            muted = prefs.getBoolean(KEY_MUTED, false)
        }
        notificationHelper = NotificationHelper(this).apply {
            ensureChannel()
        }
        bridge = LoomAndroidBridge(applicationContext, soundSynthesizer, notificationHelper)

        configureTopBar()
        configureOfflinePanel()
        configureWebView()
        configureBackNavigation()
        requestNotificationPermissionIfNeeded()

        val initialUrl = resolveUrlFromIntent(intent) ?: currentEndpoint
        loadUrl(initialUrl)
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
        val pairUrl = resolveUrlFromIntent(intent)
        if (pairUrl != null) {
            loadUrl(pairUrl)
        }
    }

    override fun onDestroy() {
        binding.webView.destroy()
        soundSynthesizer.release()
        super.onDestroy()
    }

    private fun configureTopBar() {
        updateSoundButtonUi()
        binding.soundButton.setOnClickListener {
            val nextMuted = !soundSynthesizer.muted
            soundSynthesizer.muted = nextMuted
            getSharedPreferences(PREFS_NAME, MODE_PRIVATE)
                .edit()
                .putBoolean(KEY_MUTED, nextMuted)
                .apply()
            updateSoundButtonUi()
            if (!nextMuted) {
                soundSynthesizer.play("click")
            }
        }
        binding.endpointButton.setOnClickListener {
            soundSynthesizer.play("click")
            showEndpointDialog()
        }
    }

    private fun configureOfflinePanel() {
        binding.retryButton.setOnClickListener {
            soundSynthesizer.play("click")
            loadUrl(currentEndpoint)
        }
        binding.configureNodeButton.setOnClickListener {
            soundSynthesizer.play("click")
            showEndpointDialog()
        }
        binding.testSoundButton.setOnClickListener {
            soundSynthesizer.play("test")
            notificationHelper.vibrate("tick")
        }
    }

    @SuppressLint("SetJavaScriptEnabled")
    private fun configureWebView() {
        binding.swipeRefresh.setColorSchemeColors(
            ContextCompat.getColor(this, R.color.loom_accent_blue),
            ContextCompat.getColor(this, R.color.loom_accent_green)
        )
        binding.swipeRefresh.setProgressBackgroundColorSchemeColor(
            ContextCompat.getColor(this, R.color.loom_bg_elevated)
        )
        binding.swipeRefresh.setOnRefreshListener {
            loadUrl(currentEndpoint)
        }

        binding.webView.settings.apply {
            javaScriptEnabled = true
            domStorageEnabled = true
            mediaPlaybackRequiresUserGesture = false
            mixedContentMode = WebSettings.MIXED_CONTENT_NEVER_ALLOW
            allowFileAccess = false
            allowContentAccess = false
        }
        binding.webView.addJavascriptInterface(bridge, "LoomAndroidBridge")

        binding.webView.webViewClient = object : WebViewClient() {
            override fun shouldOverrideUrlLoading(
                view: WebView?,
                request: WebResourceRequest?
            ): Boolean {
                val targetUri = request?.url ?: return true
                val scheme = targetUri.scheme?.lowercase()
                if (scheme != "http" && scheme != "https") {
                    return true
                }
                val endpointUri = Uri.parse(currentEndpoint)
                val sameScheme = scheme == endpointUri.scheme?.lowercase()
                val sameHost = endpointUri.host?.lowercase() == targetUri.host?.lowercase()
                val samePort = effectivePort(endpointUri) == effectivePort(targetUri)
                if (sameScheme && sameHost && samePort) {
                    return false
                }
                if (scheme == "https") {
                    startActivity(Intent(Intent.ACTION_VIEW, targetUri))
                }
                return true
            }

            override fun onPageStarted(view: WebView?, url: String?, favicon: android.graphics.Bitmap?) {
                lastLoadFailed = false
                setStatus(StatusState.CONNECTING)
                binding.offlinePanel.visibility = View.GONE
            }

            override fun onPageFinished(view: WebView?, url: String?) {
                binding.swipeRefresh.isRefreshing = false
                if (!lastLoadFailed) {
                    binding.offlinePanel.visibility = View.GONE
                    setStatus(StatusState.ONLINE)
                }
            }

            override fun onReceivedHttpError(
                view: WebView?,
                request: WebResourceRequest?,
                errorResponse: android.webkit.WebResourceResponse?
            ) {
                if (request?.isForMainFrame == true && (errorResponse?.statusCode ?: 0) >= 400) {
                    lastLoadFailed = true
                    binding.swipeRefresh.isRefreshing = false
                    setStatus(StatusState.OFFLINE)
                    showOfflineState("HTTP ${errorResponse?.statusCode ?: 500} · $currentEndpoint")
                }
            }

            override fun onReceivedError(
                view: WebView?,
                request: WebResourceRequest?,
                error: WebResourceError?
            ) {
                if (request?.isForMainFrame == true) {
                    lastLoadFailed = true
                    binding.swipeRefresh.isRefreshing = false
                    setStatus(StatusState.OFFLINE)
                    showOfflineState(
                        detail = "${currentEndpoint}\n${error?.description ?: "Error de red"}"
                    )
                }
            }
        }
    }

    private fun configureBackNavigation() {
        onBackPressedDispatcher.addCallback(
            this,
            object : OnBackPressedCallback(true) {
                override fun handleOnBackPressed() {
                    if (binding.offlinePanel.visibility != View.VISIBLE && binding.webView.canGoBack()) {
                        binding.webView.goBack()
                    } else {
                        isEnabled = false
                        onBackPressedDispatcher.onBackPressed()
                    }
                }
            }
        )
    }

    private fun requestNotificationPermissionIfNeeded() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU &&
            !notificationHelper.hasNotificationPermission()
        ) {
            notificationPermissionLauncher.launch(Manifest.permission.POST_NOTIFICATIONS)
        }
    }

    private fun effectivePort(uri: Uri): Int {
        if (uri.port > 0) return uri.port
        return if (uri.scheme?.lowercase() == "https") 443 else 80
    }

    private fun resolveUrlFromIntent(intent: Intent?): String? {
        val data = intent?.data ?: return null
        if (data.scheme?.lowercase() != "loom" || data.host?.lowercase() != "pair") {
            return null
        }
        val code = data.getQueryParameter("code")
            ?.takeIf { it.isNotBlank() }
            ?: data.lastPathSegment?.takeIf { it.isNotBlank() }
            ?: return null
        return EndpointConfig.buildPairUrl(currentEndpoint, code)
    }

    private fun loadUrl(url: String) {
        lastLoadFailed = false
        setStatus(StatusState.CONNECTING)
        binding.offlinePanel.visibility = View.GONE
        binding.webView.loadUrl(url)
    }

    private fun showOfflineState(detail: String) {
        binding.offlineDetail.text = detail
        binding.offlinePanel.visibility = View.VISIBLE
    }

    private fun updateSoundButtonUi() {
        if (soundSynthesizer.muted) {
            binding.soundButton.setImageResource(R.drawable.ic_sound_off)
            binding.soundButton.contentDescription = getString(R.string.sound_button_unmute_desc)
        } else {
            binding.soundButton.setImageResource(R.drawable.ic_sound_on)
            binding.soundButton.contentDescription = getString(R.string.sound_button_mute_desc)
        }
    }

    private fun setStatus(state: StatusState) {
        when (state) {
            StatusState.CONNECTING -> {
                binding.statusChip.setText(R.string.status_connecting)
                binding.statusChip.setTextColor(ContextCompat.getColor(this, R.color.loom_accent_blue))
            }
            StatusState.ONLINE -> {
                binding.statusChip.setText(R.string.status_online)
                binding.statusChip.setTextColor(ContextCompat.getColor(this, R.color.loom_accent_green))
            }
            StatusState.OFFLINE -> {
                binding.statusChip.setText(R.string.status_offline)
                binding.statusChip.setTextColor(ContextCompat.getColor(this, R.color.loom_status_error))
            }
        }
    }

    private fun showEndpointDialog() {
        val dialogView = layoutInflater.inflate(R.layout.dialog_endpoint, null)
        val inputLayout = dialogView.findViewById<TextInputLayout>(R.id.endpointInputLayout)
        val input = dialogView.findViewById<TextInputEditText>(R.id.endpointInput)
        val usbButton = dialogView.findViewById<MaterialButton>(R.id.presetUsbButton)

        input.setText(currentEndpoint)
        usbButton.setOnClickListener {
            input.setText(EndpointConfig.DEFAULT_ENDPOINT)
            inputLayout.error = null
        }

        val dialog = MaterialAlertDialogBuilder(this)
            .setTitle(R.string.dialog_endpoint_title)
            .setView(dialogView)
            .setNegativeButton(R.string.dialog_cancel, null)
            .setPositiveButton(R.string.dialog_save, null)
            .create()

        dialog.setOnShowListener {
            dialog.getButton(AlertDialog.BUTTON_POSITIVE).setOnClickListener {
                val raw = input.text?.toString().orEmpty()
                val normalized = EndpointConfig.normalize(raw)
                normalized.fold(
                    onSuccess = { validUrl ->
                        currentEndpoint = validUrl
                        getSharedPreferences(PREFS_NAME, MODE_PRIVATE)
                            .edit()
                            .putString(KEY_ENDPOINT, validUrl)
                            .apply()
                        dialog.dismiss()
                        loadUrl(validUrl)
                    },
                    onFailure = {
                        inputLayout.error = getString(R.string.error_invalid_endpoint)
                    }
                )
            }
        }

        dialog.show()
    }

    private enum class StatusState {
        CONNECTING,
        ONLINE,
        OFFLINE
    }

    companion object {
        private const val PREFS_NAME = "loom_prefs"
        private const val KEY_ENDPOINT = "endpoint_url"
        private const val KEY_MUTED = "sound_muted"
    }
}
