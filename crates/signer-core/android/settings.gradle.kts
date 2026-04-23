// Phase 3 Android library sample — real com.android.library consuming the
// per-ABI .so files compiled by build-android.sh. Instrumented tests run on
// an emulator via reactivecircus/android-emulator-runner in CI.

pluginManagement {
    repositories {
        google()
        gradlePluginPortal()
        mavenCentral()
    }
    plugins {
        id("com.android.library") version "8.7.0"
        kotlin("android") version "2.0.21"
    }
}

dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
    }
}

rootProject.name = "signer-core-android-sample"
include(":sample")
