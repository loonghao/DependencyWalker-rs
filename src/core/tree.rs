//! Dependency tree data structures and utilities
//!
//! This module provides additional utilities for working with dependency trees,
//! including traversal, filtering, and analysis functions.

use crate::core::dependency::{DependencyNode, DependencyTree};
use std::path::Path;

/// Tree traversal order
#[derive(Debug, Clone, Copy)]
pub enum TraversalOrder {
    /// Depth-first pre-order (parent before children)
    PreOrder,
    /// Depth-first post-order (children before parent)
    PostOrder,
    /// Breadth-first (level by level)
    BreadthFirst,
}

/// Filter criteria for tree traversal
#[derive(Debug, Clone)]
pub struct NodeFilter {
    /// Only include nodes with errors
    pub errors_only: bool,
    /// Only include missing dependencies
    pub missing_only: bool,
    /// Only include circular dependencies
    pub circular_only: bool,
    /// Maximum depth to traverse
    pub max_depth: Option<usize>,
    /// Include system DLLs
    pub include_system: bool,
}

impl Default for NodeFilter {
    fn default() -> Self {
        Self {
            errors_only: false,
            missing_only: false,
            circular_only: false,
            max_depth: None,
            include_system: true,
        }
    }
}

impl NodeFilter {
    /// Create a filter for only problematic nodes
    pub fn problems_only() -> Self {
        Self {
            errors_only: true,
            missing_only: false,
            circular_only: false,
            max_depth: None,
            include_system: true,
        }
    }

    /// Create a filter for missing dependencies only
    pub fn missing_only() -> Self {
        Self {
            errors_only: false,
            missing_only: true,
            circular_only: false,
            max_depth: None,
            include_system: true,
        }
    }

    /// Check if a node matches the filter criteria
    pub fn matches(&self, node: &DependencyNode) -> bool {
        if self.errors_only && !node.has_errors() {
            return false;
        }

        if self.missing_only && node.found {
            return false;
        }

        if self.circular_only && !node.is_circular {
            return false;
        }

        if let Some(max_depth) = self.max_depth {
            if node.depth > max_depth {
                return false;
            }
        }

        if !self.include_system && is_system_dll(&node.name) {
            return false;
        }

        true
    }
}

/// Tree traversal iterator
pub struct TreeIterator<'a> {
    nodes: Vec<&'a DependencyNode>,
    current: usize,
}

impl<'a> TreeIterator<'a> {
    /// Create a new tree iterator with specified traversal order
    pub fn new(
        tree: &'a DependencyTree,
        order: TraversalOrder,
        filter: Option<NodeFilter>,
    ) -> Self {
        let mut nodes = Vec::new();

        if let Some(root) = &tree.root {
            match order {
                TraversalOrder::PreOrder => {
                    Self::collect_pre_order(root, &mut nodes, &filter);
                }
                TraversalOrder::PostOrder => {
                    Self::collect_post_order(root, &mut nodes, &filter);
                }
                TraversalOrder::BreadthFirst => {
                    Self::collect_breadth_first(root, &mut nodes, &filter);
                }
            }
        }

        Self { nodes, current: 0 }
    }

    fn collect_pre_order(
        node: &'a DependencyNode,
        nodes: &mut Vec<&'a DependencyNode>,
        filter: &Option<NodeFilter>,
    ) {
        if filter.as_ref().map_or(true, |f| f.matches(node)) {
            nodes.push(node);
        }

        for child in &node.children {
            Self::collect_pre_order(child, nodes, filter);
        }
    }

    fn collect_post_order(
        node: &'a DependencyNode,
        nodes: &mut Vec<&'a DependencyNode>,
        filter: &Option<NodeFilter>,
    ) {
        for child in &node.children {
            Self::collect_post_order(child, nodes, filter);
        }

        if filter.as_ref().map_or(true, |f| f.matches(node)) {
            nodes.push(node);
        }
    }

    fn collect_breadth_first(
        root: &'a DependencyNode,
        nodes: &mut Vec<&'a DependencyNode>,
        filter: &Option<NodeFilter>,
    ) {
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(root);

        while let Some(node) = queue.pop_front() {
            if filter.as_ref().map_or(true, |f| f.matches(node)) {
                nodes.push(node);
            }

            for child in &node.children {
                queue.push_back(child);
            }
        }
    }
}

impl<'a> Iterator for TreeIterator<'a> {
    type Item = &'a DependencyNode;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.nodes.len() {
            let node = self.nodes[self.current];
            self.current += 1;
            Some(node)
        } else {
            None
        }
    }
}

/// Tree analysis utilities
impl DependencyTree {
    /// Get an iterator over all nodes in the tree
    pub fn iter(&self, order: TraversalOrder) -> TreeIterator {
        TreeIterator::new(self, order, None)
    }

    /// Get an iterator with filtering
    pub fn iter_filtered(&self, order: TraversalOrder, filter: NodeFilter) -> TreeIterator {
        TreeIterator::new(self, order, Some(filter))
    }

    /// Find all nodes matching a predicate
    pub fn find_nodes<F>(&self, predicate: F) -> Vec<&DependencyNode>
    where
        F: Fn(&DependencyNode) -> bool,
    {
        self.iter(TraversalOrder::PreOrder)
            .filter(|node| predicate(node))
            .collect()
    }

    /// Get all nodes with errors
    pub fn get_error_nodes(&self) -> Vec<&DependencyNode> {
        self.find_nodes(|node| node.has_errors())
    }

    /// Get all missing dependencies as nodes
    pub fn get_missing_nodes(&self) -> Vec<&DependencyNode> {
        self.find_nodes(|node| !node.found)
    }

    /// Get dependency path from root to a specific node
    pub fn get_path_to_node(&self, target_path: &Path) -> Option<Vec<&DependencyNode>> {
        if let Some(root) = &self.root {
            let mut path = Vec::new();
            if self.find_path_recursive(root, target_path, &mut path) {
                return Some(path);
            }
        }
        None
    }

    fn find_path_recursive<'a>(
        &self,
        node: &'a DependencyNode,
        target: &Path,
        path: &mut Vec<&'a DependencyNode>,
    ) -> bool {
        path.push(node);

        if node.path == target {
            return true;
        }

        for child in &node.children {
            if self.find_path_recursive(child, target, path) {
                return true;
            }
        }

        path.pop();
        false
    }
}

/// Check if a DLL name is a system DLL
fn is_system_dll(dll_name: &str) -> bool {
    let system_dlls = [
        "kernel32.dll",
        "user32.dll",
        "gdi32.dll",
        "advapi32.dll",
        "shell32.dll",
        "ole32.dll",
        "oleaut32.dll",
        "comctl32.dll",
        "comdlg32.dll",
        "winmm.dll",
        "version.dll",
        "ws2_32.dll",
        "ntdll.dll",
        "msvcrt.dll",
        "vcruntime140.dll",
        "msvcp140.dll",
    ];

    let dll_lower = dll_name.to_lowercase();
    system_dlls.iter().any(|&sys_dll| dll_lower == sys_dll)
        || dll_lower.starts_with("api-ms-")
        || dll_lower.starts_with("ext-ms-")
}
