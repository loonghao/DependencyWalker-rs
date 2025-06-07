//! DependencyWalker RS - Unified CLI and GUI Application
//!
//! A modern Windows Dependency Walker with both command-line and graphical interfaces.
//! Automatically detects whether to run in CLI or GUI mode based on arguments and environment.

// Note: We use console subsystem to support CLI mode
// GUI mode will handle window creation appropriately
#![cfg_attr(all(not(debug_assertions), feature = "gui", not(feature = "cli")), windows_subsystem = "windows")]

use dependencywalker_rs::{init, Result};
use std::path::PathBuf;

#[cfg(feature = "cli")]
use clap::{Parser, Subcommand};
#[cfg(feature = "cli")]
use dependencywalker_rs::cli::{analyze_command, tree_command, list_command};



#[cfg(feature = "cli")]
#[derive(Parser)]
#[command(
    name = "depwalker",
    version,
    about = "A modern Windows Dependency Walker implemented in Rust",
    long_about = "DependencyWalker RS is a modern, fast, and reliable tool for analyzing Windows PE file dependencies. It provides comprehensive dependency analysis with support for API Set redirection, SxS assemblies, and more."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Output format
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,

    /// Force GUI mode even with command line arguments
    #[arg(long)]
    gui: bool,
}

#[cfg(feature = "cli")]
#[derive(Subcommand)]
enum Commands {
    /// Analyze dependencies of a PE file
    Analyze {
        /// Path to the PE file to analyze
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Maximum recursion depth
        #[arg(short, long, default_value_t = 10)]
        depth: u32,

        /// Include system DLLs in analysis
        #[arg(short, long)]
        system: bool,

        /// Additional search paths
        #[arg(short, long)]
        path: Vec<PathBuf>,
    },

    /// Display dependency tree
    Tree {
        /// Path to the PE file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Show only missing dependencies
        #[arg(short, long)]
        missing: bool,
    },

    /// List all dependencies
    List {
        /// Path to the PE file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },
}

#[cfg(feature = "cli")]
#[derive(clap::ValueEnum, Clone)]
enum OutputFormat {
    Text,
    Json,
    Xml,
}

fn main() -> Result<()> {
    // Determine run mode based on arguments and environment first
    let args: Vec<String> = std::env::args().collect();
    println!("Args: {:?}", args);

    let should_run_gui = should_use_gui_mode(&args);
    println!("Should run GUI: {}", should_run_gui);

    // Set up logging before initialization
    if !should_run_gui && args.iter().any(|arg| arg == "-v" || arg == "--verbose") {
        std::env::set_var("RUST_LOG", "debug");
    }

    // Initialize the library
    println!("Initializing library...");
    init()?;
    println!("Library initialized");

    if should_run_gui {
        println!("Running in GUI mode...");
        #[cfg(feature = "gui")]
        {
            run_gui_mode()?;
        }
        #[cfg(not(feature = "gui"))]
        {
            // If GUI is not available, show help instead
            println!("DependencyWalker RS v{}", env!("CARGO_PKG_VERSION"));
            println!("GUI mode not available. This binary was compiled without GUI support.");
            println!("Use --help for CLI usage information.");
        }
    } else {
        println!("Running in CLI mode...");
        #[cfg(feature = "cli")]
        {
            run_cli_mode()?;
        }
        #[cfg(not(feature = "cli"))]
        {
            eprintln!("CLI mode not available. This binary was compiled without CLI support.");
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Determine whether to run in GUI mode based on command line arguments
fn should_use_gui_mode(args: &[String]) -> bool {
    // If no arguments provided, default to GUI mode
    if args.len() <= 1 {
        return true;
    }

    // Check for explicit GUI flag
    if args.iter().any(|arg| arg == "--gui") {
        return true;
    }

    // Check for help or version flags (should use CLI)
    if args.iter().any(|arg| arg == "--help" || arg == "-h" || arg == "--version" || arg == "-V") {
        return false;
    }

    // Check for subcommands (should use CLI)
    if args.len() > 1 && ["analyze", "tree", "list"].contains(&args[1].as_str()) {
        return false;
    }

    // If arguments are provided but no recognized subcommand, try GUI mode
    true
}

#[cfg(feature = "cli")]
fn run_cli_mode() -> Result<()> {
    let cli = Cli::parse();

    // Set log level based on verbosity
    if cli.verbose {
        std::env::set_var("RUST_LOG", "debug");
    }

    // If GUI flag is set, switch to GUI mode
    if cli.gui {
        #[cfg(feature = "gui")]
        {
            return run_gui_mode();
        }
        #[cfg(not(feature = "gui"))]
        {
            eprintln!("GUI mode not available. This binary was compiled without GUI support.");
            std::process::exit(1);
        }
    }

    // Convert output format
    let output_format = match cli.format {
        OutputFormat::Text => dependencywalker_rs::cli::output::Format::Text,
        OutputFormat::Json => dependencywalker_rs::cli::output::Format::Json,
        OutputFormat::Xml => dependencywalker_rs::cli::output::Format::Xml,
    };

    match &cli.command {
        Some(Commands::Analyze { file, depth, system, path }) => {
            if let Err(e) = analyze_command(file, *depth, *system, path, output_format) {
                eprintln!("Error during analysis: {}", e);
                std::process::exit(1);
            }
        }

        Some(Commands::Tree { file, missing }) => {
            if let Err(e) = tree_command(file, *missing, output_format) {
                eprintln!("Error displaying tree: {}", e);
                std::process::exit(1);
            }
        }

        Some(Commands::List { file, detailed }) => {
            if let Err(e) = list_command(file, *detailed, output_format) {
                eprintln!("Error listing dependencies: {}", e);
                std::process::exit(1);
            }
        }

        None => {
            println!("DependencyWalker RS v{}", env!("CARGO_PKG_VERSION"));
            println!("Use --help for usage information or run without arguments for GUI mode.");
        }
    }

    Ok(())
}

#[cfg(feature = "gui")]
fn run_gui_mode() -> Result<()> {
    // Check if we should use Slint or ICED
    let use_slint = std::env::var("DEPWALKER_UI")
        .map(|ui| ui.to_lowercase() == "slint")
        .unwrap_or(true); // Default to Slint

    if use_slint {
        use dependencywalker_rs::gui::SlintApp;

        println!("Starting Slint GUI application");
        log::info!("Starting Slint GUI application");

        // Create and run the Slint application
        println!("Creating SlintApp...");
        let app = SlintApp::new()
            .map_err(|e| anyhow::anyhow!("Failed to create Slint application: {}", e))?;

        println!("Running SlintApp...");
        app.run()
            .map_err(|e| anyhow::anyhow!("Failed to run Slint GUI: {}", e))?;

        println!("SlintApp finished running");
    } else {
        use dependencywalker_rs::gui::DependencyWalkerApp;

        log::info!("Starting ICED GUI application");

        // Run the ICED application
        DependencyWalkerApp::run()
            .map_err(|e| dependencywalker_rs::Error::from(anyhow::anyhow!("Failed to run ICED GUI: {}", e)))?;
    }

    Ok(())
}

// Slint GUI integration is now handled through the ui/main.slint file
// and the setup_gui_callbacks function above


#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert()
    }
}




