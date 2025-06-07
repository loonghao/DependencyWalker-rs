//! Dependency relationship analysis and tree building
//! 
//! This module provides functionality to analyze PE file dependencies and build
//! comprehensive dependency trees with cycle detection and missing dependency reporting.

use crate::error::Result;
use crate::core::pe_parser::{PEFileMap, PEFile};
use crate::core::resolver::{DllResolver, ModuleSearchStrategy};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::Instant;
use serde::{Deserialize, Serialize};

/// Maximum recursion depth to prevent stack overflow
const MAX_RECURSION_DEPTH: usize = 50;



/// Represents a single node in the dependency tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyNode {
    /// File path of this dependency
    pub path: PathBuf,
    /// Display name (usually filename)
    pub name: String,
    /// Whether this file was found and successfully parsed
    pub found: bool,
    /// Whether this is a 64-bit PE file
    pub is_64bit: Option<bool>,
    /// Direct dependencies of this node
    pub dependencies: Vec<String>,
    /// Child nodes in the dependency tree
    pub children: Vec<DependencyNode>,
    /// Depth in the dependency tree (0 = root)
    pub depth: usize,
    /// Any errors encountered while processing this node
    pub errors: Vec<String>,
    /// Whether this node is part of a circular dependency
    pub is_circular: bool,
}

impl DependencyNode {
    /// Create a new dependency node
    pub fn new<P: AsRef<Path>>(path: P, depth: usize) -> Self {
        let path = path.as_ref().to_path_buf();
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();
        
        Self {
            path,
            name,
            found: false,
            is_64bit: None,
            dependencies: Vec::new(),
            children: Vec::new(),
            depth,
            errors: Vec::new(),
            is_circular: false,
        }
    }
    
    /// Add an error message to this node
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }
    
    /// Check if this node has any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
    
    /// Get the total number of dependencies (recursive)
    pub fn total_dependencies(&self) -> usize {
        self.children.len() + self.children.iter().map(|c| c.total_dependencies()).sum::<usize>()
    }
    
    /// Find a node by path (recursive search)
    pub fn find_node(&self, path: &Path) -> Option<&DependencyNode> {
        if self.path == path {
            return Some(self);
        }
        
        for child in &self.children {
            if let Some(node) = child.find_node(path) {
                return Some(node);
            }
        }
        
        None
    }
}

/// Represents the complete dependency tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyTree {
    /// Root node of the tree
    pub root: Option<DependencyNode>,
    /// Analysis statistics
    pub stats: AnalysisStats,
    /// All unique dependencies found (path -> node info)
    pub all_dependencies: HashMap<PathBuf, DependencyInfo>,
    /// Circular dependency chains detected
    pub circular_dependencies: Vec<Vec<PathBuf>>,
    /// Missing dependencies
    pub missing_dependencies: Vec<String>,
}

/// Information about a dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyInfo {
    pub path: PathBuf,
    pub found: bool,
    pub is_64bit: Option<bool>,
    pub reference_count: usize,
    pub first_seen_depth: usize,
}

/// Analysis statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalysisStats {
    /// Total number of unique dependencies
    pub total_dependencies: usize,
    /// Number of missing dependencies
    pub missing_count: usize,
    /// Number of circular dependency chains
    pub circular_count: usize,
    /// Maximum depth reached
    pub max_depth: usize,
    /// Analysis duration in milliseconds
    pub analysis_time_ms: u64,
    /// Number of files successfully parsed
    pub parsed_files: usize,
    /// Number of parsing errors
    pub parse_errors: usize,
}

impl DependencyTree {
    /// Create a new empty dependency tree
    pub fn new() -> Self {
        Self {
            root: None,
            stats: AnalysisStats::default(),
            all_dependencies: HashMap::new(),
            circular_dependencies: Vec::new(),
            missing_dependencies: Vec::new(),
        }
    }
    
    /// Get all missing dependencies
    pub fn get_missing_dependencies(&self) -> &[String] {
        &self.missing_dependencies
    }
    
    /// Get all circular dependency chains
    pub fn get_circular_dependencies(&self) -> &[Vec<PathBuf>] {
        &self.circular_dependencies
    }
    
    /// Check if the tree has any issues (missing or circular dependencies)
    pub fn has_issues(&self) -> bool {
        !self.missing_dependencies.is_empty() || !self.circular_dependencies.is_empty()
    }
    
    /// Get a summary of the analysis
    pub fn summary(&self) -> String {
        format!(
            "Dependencies: {} total, {} missing, {} circular chains, max depth: {}, analysis time: {}ms",
            self.stats.total_dependencies,
            self.stats.missing_count,
            self.stats.circular_count,
            self.stats.max_depth,
            self.stats.analysis_time_ms
        )
    }
}

impl Default for DependencyTree {
    fn default() -> Self {
        Self::new()
    }
}

/// Dependency analyzer for building dependency trees
#[derive(Debug)]
pub struct DependencyAnalyzer {
    /// DLL resolver for path resolution
    dll_resolver: DllResolver,
    /// Maximum recursion depth
    max_depth: usize,
    /// Whether to include system DLLs in analysis
    include_system_dlls: bool,
}

impl DependencyAnalyzer {
    /// Create a new dependency analyzer with default settings
    pub fn new() -> Self {
        Self {
            dll_resolver: DllResolver::new(),
            max_depth: MAX_RECURSION_DEPTH,
            include_system_dlls: false,
        }
    }

    /// Add a search path for DLL resolution
    pub fn add_search_path<P: AsRef<Path>>(&mut self, path: P) {
        self.dll_resolver.add_search_path(path);
    }

    /// Set maximum recursion depth
    pub fn set_max_depth(&mut self, depth: usize) {
        self.max_depth = depth;
    }

    /// Set whether to include system DLLs
    pub fn set_include_system_dlls(&mut self, include: bool) {
        self.include_system_dlls = include;
        // Update resolver config
        let mut config = self.dll_resolver.config().clone();
        config.include_system_dlls = include;
        self.dll_resolver.set_config(config);
    }

    /// Build dependency tree for a PE file
    pub fn build_tree<P: AsRef<Path>>(&mut self, root_path: P) -> Result<DependencyTree> {
        let start_time = Instant::now();
        let root_path = root_path.as_ref().to_path_buf();

        log::info!("Building dependency tree for: {}", root_path.display());

        let mut tree = DependencyTree::new();
        let mut visited = HashSet::new();
        let mut processing_stack = Vec::new();

        // Build the tree recursively
        match self.build_node(&root_path, 0, &mut visited, &mut processing_stack, &mut tree) {
            Ok(root_node) => {
                tree.root = Some(root_node);
            }
            Err(e) => {
                log::error!("Failed to build dependency tree: {:?}", e);
                return Err(e);
            }
        }

        // Detect circular dependencies
        self.detect_circular_dependencies(&mut tree);

        // Finalize statistics
        tree.stats.analysis_time_ms = start_time.elapsed().as_millis() as u64;
        tree.stats.total_dependencies = tree.all_dependencies.len();
        tree.stats.missing_count = tree.missing_dependencies.len();
        tree.stats.circular_count = tree.circular_dependencies.len();

        log::info!("Dependency analysis completed: {}", tree.summary());

        Ok(tree)
    }

    /// Build a single dependency node recursively
    fn build_node(
        &mut self,
        path: &Path,
        depth: usize,
        visited: &mut HashSet<PathBuf>,
        processing_stack: &mut Vec<PathBuf>,
        tree: &mut DependencyTree,
    ) -> Result<DependencyNode> {
        let mut node = DependencyNode::new(path, depth);

        // Check recursion depth limit
        if depth > self.max_depth {
            node.add_error(format!("Maximum recursion depth ({}) exceeded", self.max_depth));
            tree.stats.max_depth = tree.stats.max_depth.max(depth);
            return Ok(node);
        }

        // Check for circular dependency
        if processing_stack.contains(&path.to_path_buf()) {
            node.is_circular = true;
            node.add_error("Circular dependency detected".to_string());
            return Ok(node);
        }

        // For the root node, we use the path as-is
        let actual_path = path;

        // Check if file exists and can be parsed
        if !actual_path.exists() {
            node.found = false;
            node.add_error(format!("File not found: {}", actual_path.display()));
            tree.missing_dependencies.push(node.name.clone());
            return Ok(node);
        }

        // Try to parse the PE file
        let pe_map = match PEFileMap::new(actual_path) {
            Ok(map) => map,
            Err(e) => {
                node.add_error(format!("Failed to load file: {:?}", e));
                tree.stats.parse_errors += 1;
                return Ok(node);
            }
        };

        let pe_file = match PEFile::new(&pe_map) {
            Ok(pe) => pe,
            Err(e) => {
                node.add_error(format!("Failed to parse PE file: {:?}", e));
                tree.stats.parse_errors += 1;
                return Ok(node);
            }
        };

        // Update node information
        node.found = true;
        node.is_64bit = pe_file.is_64bit().ok();
        tree.stats.parsed_files += 1;

        // Get dependencies
        let dependencies = match pe_file.get_dependencies() {
            Ok(deps) => deps,
            Err(e) => {
                node.add_error(format!("Failed to get dependencies: {:?}", e));
                Vec::new()
            }
        };

        node.dependencies = dependencies.clone();

        // Add to processing stack to detect cycles
        processing_stack.push(path.to_path_buf());

        // Process each dependency recursively
        for dep_name in dependencies {
            // Skip system DLLs if not requested
            if !self.include_system_dlls && self.is_system_dll(&dep_name) {
                continue;
            }

            // Try to resolve dependency path using the new resolver
            let (_search_strategy, resolved_path) = match self.dll_resolver.resolve_dll(&pe_file, &dep_name) {
                Ok(result) => result,
                Err(e) => {
                    log::warn!("Failed to resolve DLL {}: {:?}", dep_name, e);
                    (ModuleSearchStrategy::NotFound, None)
                }
            };

            let dep_path = PathBuf::from(&dep_name);
            let actual_dep_path = resolved_path.as_ref().unwrap_or(&dep_path);

            // Record dependency info
            let dep_info = tree.all_dependencies.entry(actual_dep_path.clone()).or_insert_with(|| {
                DependencyInfo {
                    path: actual_dep_path.clone(),
                    found: actual_dep_path.exists(),
                    is_64bit: None,
                    reference_count: 0,
                    first_seen_depth: depth + 1,
                }
            });
            dep_info.reference_count += 1;

            // Recursively build child node if not already visited at this depth
            if !visited.contains(actual_dep_path) || depth + 1 <= 3 {
                visited.insert(actual_dep_path.clone());

                match self.build_node(actual_dep_path, depth + 1, visited, processing_stack, tree) {
                    Ok(child_node) => {
                        node.children.push(child_node);
                    }
                    Err(e) => {
                        log::warn!("Failed to build child node for {}: {:?}", dep_name, e);
                        let mut error_node = DependencyNode::new(actual_dep_path, depth + 1);
                        error_node.add_error(format!("Build failed: {:?}", e));
                        node.children.push(error_node);
                    }
                }
            }
        }

        // Remove from processing stack
        processing_stack.pop();

        tree.stats.max_depth = tree.stats.max_depth.max(depth);

        Ok(node)
    }



    /// Check if a DLL is a system DLL
    fn is_system_dll(&self, dll_name: &str) -> bool {
        let system_dlls = [
            "kernel32.dll", "user32.dll", "gdi32.dll", "advapi32.dll",
            "shell32.dll", "ole32.dll", "oleaut32.dll", "comctl32.dll",
            "comdlg32.dll", "winmm.dll", "version.dll", "ws2_32.dll",
            "ntdll.dll", "msvcrt.dll", "vcruntime140.dll", "msvcp140.dll",
        ];

        let dll_lower = dll_name.to_lowercase();
        system_dlls.iter().any(|&sys_dll| dll_lower == sys_dll) ||
        dll_lower.starts_with("api-ms-") ||
        dll_lower.starts_with("ext-ms-")
    }

    /// Detect circular dependencies using DFS
    fn detect_circular_dependencies(&self, tree: &mut DependencyTree) {
        let root_clone = tree.root.clone();
        if let Some(root) = root_clone {
            let mut visited = HashSet::new();
            let mut rec_stack = HashSet::new();
            let mut current_path = Vec::new();

            self.dfs_detect_cycles(&root, &mut visited, &mut rec_stack, &mut current_path, tree);
        }
    }

    /// DFS helper for cycle detection
    fn dfs_detect_cycles(
        &self,
        node: &DependencyNode,
        visited: &mut HashSet<PathBuf>,
        rec_stack: &mut HashSet<PathBuf>,
        current_path: &mut Vec<PathBuf>,
        tree: &mut DependencyTree,
    ) {
        visited.insert(node.path.clone());
        rec_stack.insert(node.path.clone());
        current_path.push(node.path.clone());

        for child in &node.children {
            if !visited.contains(&child.path) {
                self.dfs_detect_cycles(child, visited, rec_stack, current_path, tree);
            } else if rec_stack.contains(&child.path) {
                // Found a cycle - extract the cycle path
                if let Some(cycle_start) = current_path.iter().position(|p| p == &child.path) {
                    let cycle: Vec<PathBuf> = current_path[cycle_start..].to_vec();
                    let mut complete_cycle = cycle;
                    complete_cycle.push(child.path.clone()); // Complete the cycle
                    tree.circular_dependencies.push(complete_cycle);
                }
            }
        }

        current_path.pop();
        rec_stack.remove(&node.path);
    }
}

impl Default for DependencyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
