// Phase 0 FFI liveness check — calls `ping()` through the UniFFI-generated
// Kotlin binding, which loads the host-platform shared library via JNA and
// calls into the compiled Rust `cdylib`. If this test passes, the Rust →
// Kotlin FFI pipeline is working end-to-end.
//
// Exit criterion from docs/migration/01-phases.md Phase 0:
//   "One iOS test and one Android test call into Rust and pass."
//
// This exercises the same code path an Android app would use, against a
// host `.dylib`/`.so` instead of a per-ABI Android `.so`. Phase 3 adds the
// real-device test once mobile apps consume signer-core via feature flag.

package tech.wideas.clad.signer.sample

import tech.wideas.clad.signer.ping
import kotlin.test.Test
import kotlin.test.assertEquals

class PingTest {
    @Test
    fun `ping returns expected greeting`() {
        assertEquals("pong from signer-core", ping())
    }
}
