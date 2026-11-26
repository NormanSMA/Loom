package dev.nsma.loom

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class SoundSynthesizerTest {

    @Test
    fun generatesNonEmptySamplesForEveryCue() {
        val synth = SoundSynthesizer()
        SoundSynthesizer.Cue.entries.forEach { cue ->
            val samples = synth.generateWaveform(cue)
            assertTrue("Expected non-empty samples for $cue", samples.isNotEmpty())
            assertTrue(
                "Expected non-zero audio energy for $cue",
                samples.any { it != 0.toShort() }
            )
        }
    }

    @Test
    fun validatesDurationAndPcmRange() {
        val synth = SoundSynthesizer()
        SoundSynthesizer.Cue.entries.forEach { cue ->
            val samples = synth.generateWaveform(cue)
            val expectedCount = (SoundSynthesizer.SAMPLE_RATE * cue.durationMs) / 1000
            assertEquals(expectedCount, samples.size)
            assertTrue(
                samples.all { it >= Short.MIN_VALUE && it <= Short.MAX_VALUE }
            )
        }
    }

    @Test
    fun mutedProducesSilence() {
        val synth = SoundSynthesizer().apply { muted = true }
        SoundSynthesizer.Cue.entries.forEach { cue ->
            val samples = synth.generateWaveform(cue)
            assertTrue(samples.isNotEmpty())
            assertTrue("Expected silence when muted for $cue", samples.all { it == 0.toShort() })
        }
    }

    @Test
    fun resolvesCueNamesSafely() {
        assertNotNull(SoundSynthesizer.Cue.fromName("click"))
        assertNotNull(SoundSynthesizer.Cue.fromName("stateChange"))
        assertNull(SoundSynthesizer.Cue.fromName("unknown_cue"))
    }
}
