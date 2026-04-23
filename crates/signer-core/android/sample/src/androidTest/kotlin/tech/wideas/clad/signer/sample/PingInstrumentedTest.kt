package tech.wideas.clad.signer.sample

import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Test
import org.junit.runner.RunWith
import tech.wideas.clad.signer.blake2b256
import tech.wideas.clad.signer.ping

/**
 * Instrumented liveness tests for signer-core on Android.
 *
 * Proves that:
 *  1. The per-ABI libsigner_core.so is loaded from the APK's jni/<abi>/ directory.
 *  2. The Phase-0 ping() greeting function round-trips through the FFI boundary.
 *  3. The Phase-2 blake2b256() crypto primitive is callable and produces the
 *     expected 32-byte digest, confirming the full Rust crypto stack is reachable
 *     at runtime — not just the greeting stub.
 */
@RunWith(AndroidJUnit4::class)
class PingInstrumentedTest {

    @Test
    fun pingReturnsExpectedGreeting() {
        assertEquals("pong from signer-core", ping())
    }

    @Test
    fun blake2b256ProducesThirtyTwoBytesForFixedInput() {
        // blake2b-256 always emits a 32-byte digest regardless of input length.
        val input = "hello signer-core".toByteArray(Charsets.UTF_8)
        val digest = blake2b256(input)
        assertEquals(32, digest.size)
    }

    @Test
    fun blake2b256IsDeterministic() {
        // Same input must always yield the same digest — fundamental hash property.
        val input = byteArrayOf(0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07)
        assertArrayEquals(blake2b256(input), blake2b256(input))
    }

    @Test
    fun blake2b256DifferentInputsProduceDifferentDigests() {
        // Smoke-checks avalanche effect: single-bit difference changes the digest.
        val a = blake2b256(byteArrayOf(0x00))
        val b = blake2b256(byteArrayOf(0x01))
        assertNotEquals(a.toList(), b.toList())
    }
}
