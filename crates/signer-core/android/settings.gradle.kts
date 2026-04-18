// Phase 0 Gradle JVM sample — proves the Rust → UniFFI → Kotlin → JNA →
// loaded shared library pipeline on the dev host (and later in Linux CI).
// Not a real Android project; see `build-android.sh` for context.

pluginManagement {
    repositories {
        gradlePluginPortal()
        mavenCentral()
    }
}

dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        mavenCentral()
    }
}

rootProject.name = "signer-core-android-sample"
include(":sample")
