// Top-level build file where you can add configuration options common to all sub-projects/modules.
plugins {
    alias(libs.plugins.android.application) apply false
    alias(libs.plugins.android.library) apply false
    alias(libs.plugins.jetbrains.kotlin.android) version "1.9.10" apply false
    id("org.mozilla.rust-android-gradle.rust-android") version "0.9.3" apply false
}
