// Phase 3 Android library sample consuming signer-core via JNA.
//
// Converted from kotlin("jvm") to com.android.library in Sprint A2 to support
// emulator-based instrumented tests that exercise the per-ABI .so files
// produced by build-android.sh. The JNA dependency (5.14.0) is retained —
// UniFFI's generated Kotlin calls Native.load("signer_core", ...) which JNA
// resolves to libsigner_core.so packaged in the APK's jni/<abi>/ directory.

plugins {
    id("com.android.library")
    kotlin("android")
}

android {
    namespace = "tech.wideas.clad.signer.sample"
    compileSdk = 35

    defaultConfig {
        minSdk = 26
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }

    // targetSdk is deprecated in library defaultConfig (AGP 8.7+); set here instead.
    testOptions {
        targetSdk = 35
    }

    kotlin {
        jvmToolchain(17)
    }

    sourceSets {
        // UniFFI-generated Kotlin binding emitted to <crate-root>/build/generated-kotlin
        // by build-android.sh. The two-dot path resolves from android/sample/ up
        // to the crate root (sample → android → signer-core).
        getByName("main") {
            java.srcDirs("../../build/generated-kotlin")
            // Per-ABI .so files staged by build-android.sh into build/aar-stage/jni/
            // are picked up here so they are packaged into the APK for emulator runs.
            jniLibs.srcDirs("../../build/aar-stage/jni")
        }
    }
}

dependencies {
    // JNA powers UniFFI's generated Native.load() call. The @aar classifier
    // includes libjnidispatch.so for Android ABIs.
    implementation("net.java.dev.jna:jna:5.14.0@aar")
    implementation("org.jetbrains.kotlin:kotlin-stdlib")

    androidTestImplementation("androidx.test.ext:junit:1.2.1")
    androidTestImplementation("androidx.test:runner:1.6.2")
}
