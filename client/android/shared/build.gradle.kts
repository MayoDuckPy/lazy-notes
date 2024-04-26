plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.jetbrains.kotlin.android)
    id("org.mozilla.rust-android-gradle.rust-android")
}

android {
    namespace = "com.mayoduckpie.lazy_notes.shared"
    compileSdk = 34

    ndkVersion = "26.3.11579264"

    defaultConfig {
        minSdk = 24

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        consumerProguardFiles("consumer-rules.pro")
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_1_8
        targetCompatibility = JavaVersion.VERSION_1_8
    }
    kotlinOptions {
        jvmTarget = "1.8"
    }

    sourceSets.getByName("main") {
        java.srcDir("${projectDir}/../../shared_types/generated/java")
        // jniLibs.srcDir("$projectDir/build/rustJniLibs/android")
    }
}

dependencies {
    implementation("net.java.dev.jna:jna:5.14.0@aar")

    implementation(libs.androidx.core.ktx)
    implementation(libs.androidx.appcompat)
    implementation(libs.material)
    testImplementation(libs.junit)
    androidTestImplementation(libs.androidx.junit)
    androidTestImplementation(libs.androidx.espresso.core)
}

apply(plugin = "org.mozilla.rust-android-gradle.rust-android")

cargo {
    module = "../.."
    libname = "shared"
    targets = listOf("arm", "arm64", "x86", "x86_64")
    extraCargoBuildArguments = listOf("--package", "shared")
}

afterEvaluate {
    // The `cargoBuild` task isn't available until after evaluation.
    android.libraryVariants.forEach { variant ->
        var productFlavor = ""
        variant.productFlavors.forEach {
            productFlavor += it.name.capitalize()
        }

        val buildType = variant.buildType.name.capitalize()

        tasks.named("compileDebugKotlin") {
            dependsOn("typesGen", "bindGen")
        }

        tasks.named("generate${productFlavor}${buildType}Assets") {
            dependsOn("cargoBuild")
        }
    }

    // Must manually configure that Android build to depend on the JNI artifacts
    // tasks.withType<com.android.build.gradle.tasks.MergeSourceSetFolders>().configureEach {
    //     if (this.name.contains("Jni")) {
    //         this.dependsOn(tasks.named("cargoBuild"))
    //     }
    // }

    // tasks.withType(com.nishtahir.CargoBuildTask::class)
    //     .forEach { buildTask ->
    //         tasks.withType(com.android.build.gradle.tasks.MergeSourceSetFolders::class)
    //             .configureEach {
    //                 this.inputs.dir(
    //                     layout.buildDirectory.dir("rustJniLibs" + File.separatorChar + buildTask.toolchain!!.folder)
    //                 )
    //                 this.dependsOn(buildTask)
    //             }
    //     }
}

tasks {
    register<Exec>("bindGen") {
        workingDir = File("../..")
        val outDir = "${projectDir}/../../shared_types/generated/java"
        if (System.getProperty("os.name").lowercase().contains("windows")) {
            commandLine("cmd", "/c",
                "cargo build -p shared && " + "target\\debug\\uniffi-bindgen generate shared\\src\\shared.udl " + "--language kotlin " + "--out-dir " + outDir.replace("/", "\\"))
        } else {
            commandLine("sh", "-c",
                """\
                cargo build -p shared && \
                target/debug/uniffi-bindgen generate shared/src/shared.udl \
                --language kotlin \
                --out-dir $outDir
                """)
        }
    }
    register<Exec>("typesGen") {
        workingDir = File("../..")
        if (System.getProperty("os.name").lowercase().contains("windows")) {
            commandLine("cmd", "/c", "cargo build -p shared_types")
        } else {
            commandLine("sh", "-c", "cargo build -p shared_types")
        }
    }
}
