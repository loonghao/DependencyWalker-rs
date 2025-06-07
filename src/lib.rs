//! # DependencyWalker RS
//!
//! A modern Windows Dependency Walker implemented in Rust.
//!
//! This library provides functionality to analyze Windows PE files and their dependencies,
//! offering both programmatic API and command-line/GUI interfaces.
//!
//! ## Features
//!
//! - **PE File Parsing**: Robust PE file analysis using pelite + goblin dual strategy
//! - **Dependency Analysis**: Complete dependency tree construction and analysis
//! - **DLL Search Path**: Full Windows DLL search path implementation
//! - **Symbol Resolution**: Import/export symbol analysis and matching
//! - **API Set Support**: Windows API Set redirection mechanism
//! - **Zero Dependencies**: Static linking for single executable deployment
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use dependencywalker_rs::core::pe_parser::{PEFileMap, PEFile};
//! use dependencywalker_rs::core::dependency::DependencyAnalyzer;
//!
//! // Parse a PE file
//! let pe_map = PEFileMap::new("example.exe")?;
//! let pe_file = PEFile::new(&pe_map)?;
//! let dependencies = pe_file.get_dependencies()?;
//!
//! // Analyze dependency tree
//! let mut analyzer = DependencyAnalyzer::new();
//! let tree = analyzer.build_tree("example.exe")?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod core;
pub mod error;

#[cfg(feature = "cli")]
pub mod cli;

#[cfg(feature = "gui")]
pub mod gui;

// Re-export commonly used types
pub use error::{Error, Result};

/// Library version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the library with default settings
pub fn init() -> Result<()> {
    // Try to initialize env_logger, but don't fail if it's already initialized
    let _ = env_logger::try_init();
    log::info!("DependencyWalker RS v{} initialized", VERSION);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.trim().is_empty());
    }

    #[test]
    fn test_init() {
        assert!(init().is_ok());
    }
}
