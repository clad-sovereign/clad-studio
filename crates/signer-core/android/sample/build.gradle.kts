// Phase 0 JVM sample consuming signer-core via JNA.
//
// Design: deliberately a `java-library` Gradle module, NOT a
// `com.android.library`. An emulator-based instrumented test adds no
// correctness signal for a `string -> string` FFI hop; the JVM runs the
// same UniFFI-generated Kotlin + JNA code that an Android process would
// load, just against the host-platform shared library staged at
// `../build/android-host/`. Phase 3 swaps in a real `com.android.library`
// consuming the per-ABI .so files.

plugins {
    kotlin("jvm") version "2.0.21"
}

java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(17))
    }
}

dependencies {
    // JNA powers UniFFI's generated loadIndirect() function.
    implementation("net.java.dev.jna:jna:5.14.0")
    implementation("org.jetbrains.kotlin:kotlin-stdlib")

    testImplementation(kotlin("test"))
    testImplementation("org.junit.jupiter:junit-jupiter:5.10.3")
}

// The UniFFI-generated binding file is emitted into
// `<crate-root>/build/generated-kotlin/tech/wideas/clad/signer/signer_core.kt`
// by `build-android.sh`. Compile it as part of the `main` source set so the
// test classpath can import `tech.wideas.clad.signer.ping`.
//
// `../../build/...` is the path from `android/sample/` up to the crate root
// (two directory levels: `sample -> android -> signer-core`) then into
// `build/generated-kotlin`.
sourceSets {
    main {
        java.srcDirs("../../build/generated-kotlin")
    }
}

tasks.test {
    useJUnitPlatform()
    // Point JNA at the host-platform .dylib / .so staged by build-android.sh.
    // `jna.library.path` is the canonical JNA search override; we also set
    // `java.library.path` for belt-and-braces.
    //
    // Path resolves from `android/sample/` up to the crate root then into
    // `build/android-host` (same two-dot pattern as the `sourceSets` srcDirs
    // above).
    val hostLibDir = file("../../build/android-host").absolutePath
    systemProperty("jna.library.path", hostLibDir)
    systemProperty("java.library.path", hostLibDir)
    // The test JVM is forked; make sure each fork sees the overrides even
    // when running in parallel.
    jvmArgs("-Djna.library.path=$hostLibDir", "-Djava.library.path=$hostLibDir")
    testLogging {
        events("passed", "failed", "skipped")
        showStandardStreams = true
        exceptionFormat = org.gradle.api.tasks.testing.logging.TestExceptionFormat.FULL
    }
}
