//! Build script for DependencyWalker RS
//!
//! This script handles build-time configuration for the application.
//! Compiles Slint UI files when the GUI feature is enabled.

fn main() {
    // Tell Cargo to rerun this build script if this file changes
    println!("cargo:rerun-if-changed=build.rs");

    // Compile Slint UI files when GUI feature is enabled
    #[cfg(feature = "gui")]
    {
        println!("cargo:rerun-if-changed=ui/");

        // Check if ui directory exists, if not create it
        if !std::path::Path::new("ui").exists() {
            std::fs::create_dir_all("ui").expect("Failed to create ui directory");
        }

        // Only compile if main.slint exists
        if std::path::Path::new("ui/main.slint").exists() {
            slint_build::compile("ui/main.slint").expect("Failed to compile Slint UI");
            println!("cargo:warning=Slint UI compiled successfully");
        } else {
            println!("cargo:warning=ui/main.slint not found, will be created in next task");
        }
    }

    // Additional configuration for Windows-specific features
    #[cfg(target_os = "windows")]
    {
        // Enable Windows-specific features if needed
        println!("cargo:rustc-link-lib=user32");
        println!("cargo:rustc-link-lib=gdi32");
        println!("cargo:rustc-link-lib=shell32");
    }

    #[cfg(not(feature = "gui"))]
    {
        println!("cargo:warning=GUI feature not enabled, skipping Slint compilation");
    }
}
