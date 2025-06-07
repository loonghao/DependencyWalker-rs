//! Simple test for Slint GUI functionality
//!
//! This example demonstrates the basic Slint GUI functionality.

use dependencywalker_rs::gui::SlintApp;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    println!("Starting Slint GUI test...");
    
    // Create and run the Slint application
    let app = SlintApp::new()?;
    
    println!("Slint application created successfully!");
    println!("Running GUI... (Close the window to exit)");
    
    app.run()?;
    
    println!("Slint GUI test completed successfully!");
    
    Ok(())
}
