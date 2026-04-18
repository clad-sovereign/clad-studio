// Phase 0 FFI liveness check — calls `ping()` through the UniFFI-generated
// Swift binding, which in turn calls the Rust staticlib shipped in
// SignerCore.xcframework. If this test passes, the Rust → Swift FFI
// pipeline is working end-to-end on this platform.
//
// Exit criterion from docs/migration/01-phases.md Phase 0:
//   "One iOS test and one Android test call into Rust and pass."
import XCTest
@testable import SignerCore

final class PingTests: XCTestCase {
    func testPingReturnsExpectedGreeting() {
        XCTAssertEqual(ping(), "pong from signer-core")
    }
}
