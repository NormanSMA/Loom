package dev.nsma.loom

import android.media.AudioAttributes
import android.media.AudioFormat
import android.media.AudioTrack
import java.util.Locale
import java.util.concurrent.ExecutorService
import java.util.concurrent.Executors
import kotlin.math.PI
import kotlin.math.roundToInt
import kotlin.math.sin

class SoundSynthesizer {

    enum class Cue(val durationMs: Int) {
        CLICK(35),
        DISPATCH(140),
        STATE_CHANGE(120),
        COMPLETE(260),
        ALERT(240),
        TEST(220);

        companion object {
            fun fromName(raw: String?): Cue? {
                val normalized = raw?.trim()?.lowercase(Locale.ROOT) ?: return null
                return when (normalized) {
                    "click" -> CLICK
                    "dispatch" -> DISPATCH
                    "statechange", "state_change" -> STATE_CHANGE
                    "complete" -> COMPLETE
                    "alert" -> ALERT
                    "test" -> TEST
                    else -> null
                }
            }
        }
    }

    @Volatile
    var muted: Boolean = false

    private val executor: ExecutorService = Executors.newSingleThreadExecutor { runnable ->
        Thread(runnable, "loom-audio").apply { isDaemon = true }
    }

    fun generateWaveform(cue: Cue): ShortArray {
        val sampleCount = (SAMPLE_RATE * cue.durationMs) / 1000
        val samples = ShortArray(sampleCount)
        if (muted || sampleCount == 0) {
            return samples
        }

        val maxAmplitude = Short.MAX_VALUE * 0.45
        for (i in 0 until sampleCount) {
            val progress = i.toDouble() / sampleCount.toDouble()
            val timeSec = i.toDouble() / SAMPLE_RATE.toDouble()
            val envelope = envelopeFor(progress)
            val signal = sampleSignal(cue, progress, timeSec)
            val scaled = (signal * envelope * maxAmplitude).roundToInt()
                .coerceIn(Short.MIN_VALUE.toInt(), Short.MAX_VALUE.toInt())
            samples[i] = scaled.toShort()
        }
        return samples
    }

    fun play(cueName: String) {
        if (muted) {
            return
        }
        val cue = Cue.fromName(cueName) ?: return
        val samples = generateWaveform(cue)
        if (samples.isEmpty()) {
            return
        }

        executor.execute {
            if (muted) {
                return@execute
            }
            playPcm(samples, cue.durationMs)
        }
    }

    fun release() {
        executor.shutdownNow()
    }

    private fun playPcm(samples: ShortArray, durationMs: Int) {
        var track: AudioTrack? = null
        try {
            val bufferBytes = samples.size * 2
            val attributes = AudioAttributes.Builder()
                .setUsage(AudioAttributes.USAGE_ASSISTANCE_SONIFICATION)
                .setContentType(AudioAttributes.CONTENT_TYPE_SONIFICATION)
                .build()
            val format = AudioFormat.Builder()
                .setSampleRate(SAMPLE_RATE)
                .setEncoding(AudioFormat.ENCODING_PCM_16BIT)
                .setChannelMask(AudioFormat.CHANNEL_OUT_MONO)
                .build()

            track = AudioTrack.Builder()
                .setAudioAttributes(attributes)
                .setAudioFormat(format)
                .setBufferSizeInBytes(bufferBytes)
                .setTransferMode(AudioTrack.MODE_STATIC)
                .build()

            if (track.state != AudioTrack.STATE_INITIALIZED) {
                return
            }
            track.write(samples, 0, samples.size)
            track.play()
            Thread.sleep(durationMs.toLong() + 30L)
        } catch (_: Throwable) {
            // Audio output may be unavailable on headless or muted test environments.
        } finally {
            try {
                track?.stop()
            } catch (_: Throwable) {
            }
            try {
                track?.release()
            } catch (_: Throwable) {
            }
        }
    }

    private fun sampleSignal(cue: Cue, progress: Double, timeSec: Double): Double {
        return when (cue) {
            Cue.CLICK -> sin(2.0 * PI * 1120.0 * timeSec)
            Cue.DISPATCH -> {
                val freq = if (progress < 0.45) 523.25 else 783.99
                0.85 * sin(2.0 * PI * freq * timeSec) + 0.15 * sin(4.0 * PI * freq * timeSec)
            }
            Cue.STATE_CHANGE -> {
                val freq = if (progress < 0.5) 440.0 else 659.25
                sin(2.0 * PI * freq * timeSec)
            }
            Cue.COMPLETE -> {
                val freq = when {
                    progress < 0.33 -> 523.25
                    progress < 0.66 -> 659.25
                    else -> 783.99
                }
                0.8 * sin(2.0 * PI * freq * timeSec) + 0.2 * sin(2.0 * PI * (freq * 1.5) * timeSec)
            }
            Cue.ALERT -> {
                val freq = if (progress < 0.5) 880.0 else 659.25
                val gate = if (progress in 0.44..0.52) 0.15 else 1.0
                gate * sin(2.0 * PI * freq * timeSec)
            }
            Cue.TEST -> {
                0.6 * sin(2.0 * PI * 587.33 * timeSec) + 0.4 * sin(2.0 * PI * 880.0 * timeSec)
            }
        }
    }

    private fun envelopeFor(progress: Double): Double {
        val attack = (progress / 0.08).coerceIn(0.0, 1.0)
        val release = ((1.0 - progress) / 0.18).coerceIn(0.0, 1.0)
        return minOf(attack, release)
    }

    companion object {
        const val SAMPLE_RATE = 44100
    }
}
