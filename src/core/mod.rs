//! Core functionality modules for DependencyWalker RS
//!
//! This module contains the core components for PE file analysis and dependency resolution.

/// PE file parsing and analysis
pub mod pe_parser;

/// PE-related data structures and types
pub mod types;

/// Dependency analysis and tree construction
pub mod dependency;

/// Dependency tree utilities and traversal
pub mod tree;

/// DLL search path resolution
pub mod resolver;

/// Symbol analysis and resolution
pub mod symbols {
    //! Import/export symbol analysis

    use crate::error::Result;

    /// Symbol information
    #[derive(Debug, Clone)]
    pub struct Symbol {
        pub name: String,
        pub ordinal: Option<u16>,
        pub is_import: bool,
    }

    /// Symbol analyzer
    pub struct SymbolAnalyzer {
        // Will be implemented in subsequent tasks
    }

    impl SymbolAnalyzer {
        /// Create a new symbol analyzer
        pub fn new() -> Self {
            Self {}
        }

        /// Analyze symbols (placeholder)
        pub fn analyze_symbols(&self, _pe_path: &str) -> Result<Vec<Symbol>> {
            // TODO: Implement in symbol analysis task
            Ok(vec![])
        }
    }

    impl Default for SymbolAnalyzer {
        fn default() -> Self {
            Self::new()
        }
    }
}

// Re-export commonly used types
pub use dependency::{
    AnalysisStats, DependencyAnalyzer, DependencyInfo, DependencyNode, DependencyTree,
};
pub use pe_parser::{PEFile, PEFileMap, PEInfo};
pub use resolver::{DllResolver, DllResolverConfig, ModuleSearchStrategy};
pub use symbols::{Symbol, SymbolAnalyzer};
pub use tree::{NodeFilter, TraversalOrder, TreeIterator};
pub use types::{
    Architecture, DetailedPEInfo, ExportInfo, ImportInfo, PEType, ParseStats, VersionInfo,
};
