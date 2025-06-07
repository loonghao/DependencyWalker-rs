//! Test GUI application
//!
//! This example tests the ICED GUI implementation.

use dependencywalker_rs::{init, Result};

#[cfg(feature = "gui")]
fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();

    // Initialize the library
    init()?;

    println!("Starting GUI test...");

    // Run the GUI application
    use dependencywalker_rs::gui::DependencyWalkerApp;
    match DependencyWalkerApp::run() {
        Ok(()) => {
            println!("GUI application closed successfully");
            Ok(())
        }
        Err(e) => {
            eprintln!("GUI application error: {}", e);
            Err(anyhow::anyhow!("GUI error: {}", e).into())
        }
    }
}

#[cfg(not(feature = "gui"))]
fn main() {
    eprintln!("GUI feature not enabled. Please compile with --features gui");
    std::process::exit(1);
}
