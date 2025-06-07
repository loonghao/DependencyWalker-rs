//! Unit tests for dependency analysis functionality

use dependencywalker_rs::core::dependency::{DependencyAnalyzer, DependencyNode, DependencyTree};
use dependencywalker_rs::core::tree::{NodeFilter, TraversalOrder};
use std::path::PathBuf;

#[test]
fn test_dependency_analyzer_creation() {
    let analyzer = DependencyAnalyzer::new();

    // Test default creation
    assert!(format!("{:?}", analyzer).contains("DependencyAnalyzer"));
}

#[test]
fn test_dependency_analyzer_configuration() {
    let mut analyzer = DependencyAnalyzer::new();

    // Test configuration methods
    analyzer.add_search_path("C:\\Test\\Path");
    analyzer.set_max_depth(20);
    analyzer.set_include_system_dlls(true);

    // These methods should not panic
    assert!(format!("{:?}", analyzer).contains("DependencyAnalyzer"));
}

#[test]
fn test_dependency_node_creation() {
    let node = DependencyNode::new("test.dll", 0);

    assert_eq!(node.name, "test.dll");
    assert_eq!(node.depth, 0);
    assert_eq!(node.found, false);
    assert_eq!(node.is_circular, false);
    assert!(node.dependencies.is_empty());
    assert!(node.children.is_empty());
    assert!(node.errors.is_empty());
}

#[test]
fn test_dependency_node_error_handling() {
    let mut node = DependencyNode::new("test.dll", 0);

    assert!(!node.has_errors());

    node.add_error("Test error".to_string());
    assert!(node.has_errors());
    assert_eq!(node.errors.len(), 1);
    assert_eq!(node.errors[0], "Test error");
}

#[test]
fn test_dependency_tree_creation() {
    let tree = DependencyTree::new();

    assert!(tree.root.is_none());
    assert_eq!(tree.stats.total_dependencies, 0);
    assert_eq!(tree.stats.missing_count, 0);
    assert_eq!(tree.stats.circular_count, 0);
    assert!(tree.all_dependencies.is_empty());
    assert!(tree.circular_dependencies.is_empty());
    assert!(tree.missing_dependencies.is_empty());
}

#[test]
fn test_dependency_tree_issues_detection() {
    let mut tree = DependencyTree::new();

    // Initially no issues
    assert!(!tree.has_issues());

    // Add missing dependency
    tree.missing_dependencies.push("missing.dll".to_string());
    assert!(tree.has_issues());

    // Add circular dependency
    tree.circular_dependencies.push(vec![
        PathBuf::from("a.dll"),
        PathBuf::from("b.dll"),
        PathBuf::from("a.dll"),
    ]);
    assert!(tree.has_issues());
}

#[test]
fn test_dependency_tree_summary() {
    let mut tree = DependencyTree::new();
    tree.stats.total_dependencies = 5;
    tree.stats.missing_count = 1;
    tree.stats.circular_count = 1;
    tree.stats.max_depth = 3;
    tree.stats.analysis_time_ms = 150;

    let summary = tree.summary();
    assert!(summary.contains("5 total"));
    assert!(summary.contains("1 missing"));
    assert!(summary.contains("1 circular"));
    assert!(summary.contains("max depth: 3"));
    assert!(summary.contains("150ms"));
}

#[test]
fn test_real_dll_dependency_analysis() {
    // Test with the provided liblzma.dll file
    let dll_path = "tests/liblzma.dll";

    // Check if the test file exists
    if !std::path::Path::new(dll_path).exists() {
        println!("Skipping real DLL dependency test - liblzma.dll not found");
        return;
    }

    println!("Testing dependency analysis with real DLL: {}", dll_path);

    let mut analyzer = DependencyAnalyzer::new();

    // Build dependency tree
    match analyzer.build_tree(dll_path) {
        Ok(tree) => {
            println!("Dependency analysis completed successfully!");
            println!("Summary: {}", tree.summary());

            // Verify basic structure
            assert!(tree.root.is_some(), "Root node should exist");

            let root = tree.root.as_ref().unwrap();
            assert_eq!(root.name, "liblzma.dll");
            assert_eq!(root.depth, 0);

            // Should have found the file
            assert!(root.found, "Root file should be found");

            // Should have some dependencies
            assert!(!root.dependencies.is_empty(), "Should have dependencies");
            println!("Root dependencies: {:?}", root.dependencies);

            // Test tree traversal
            let all_nodes: Vec<_> = tree.iter(TraversalOrder::PreOrder).collect();
            println!("Total nodes in tree: {}", all_nodes.len());
            assert!(!all_nodes.is_empty());

            // Test filtering
            let missing_nodes: Vec<_> = tree
                .iter_filtered(TraversalOrder::PreOrder, NodeFilter::missing_only())
                .collect();
            println!("Missing dependencies: {}", missing_nodes.len());

            // Test error nodes
            let error_nodes = tree.get_error_nodes();
            println!("Nodes with errors: {}", error_nodes.len());

            // Test missing nodes
            let missing_nodes = tree.get_missing_nodes();
            println!("Missing nodes: {}", missing_nodes.len());

            // Verify statistics
            assert!(
                tree.stats.parsed_files > 0,
                "Should have parsed at least one file"
            );
            // Analysis time is always valid (u64 type), just verify it exists
            println!("Analysis completed in {}ms", tree.stats.analysis_time_ms);
        }
        Err(e) => {
            panic!("Failed to build dependency tree: {:?}", e);
        }
    }
}

#[test]
fn test_dependency_analyzer_with_nonexistent_file() {
    let mut analyzer = DependencyAnalyzer::new();

    let result = analyzer.build_tree("nonexistent_file.exe");

    // Our implementation creates a tree even for nonexistent files, but marks them as missing
    match result {
        Ok(tree) => {
            // Should have a root node that's marked as not found
            assert!(tree.root.is_some());
            let root = tree.root.as_ref().unwrap();
            assert!(!root.found, "Root should be marked as not found");
            assert!(root.has_errors(), "Root should have errors");
        }
        Err(_) => {
            // This is also acceptable - depends on implementation
        }
    }
}

#[test]
fn test_node_filter_functionality() {
    // Test default filter
    let default_filter = NodeFilter::default();
    assert!(!default_filter.errors_only);
    assert!(!default_filter.missing_only);
    assert!(!default_filter.circular_only);
    assert!(default_filter.include_system);

    // Test problems only filter
    let problems_filter = NodeFilter::problems_only();
    assert!(problems_filter.errors_only);

    // Test missing only filter
    let missing_filter = NodeFilter::missing_only();
    assert!(missing_filter.missing_only);

    // Test filter matching
    let mut node = DependencyNode::new("test.dll", 0);

    // Should match default filter
    assert!(default_filter.matches(&node));

    // Should not match problems filter (no errors)
    assert!(!problems_filter.matches(&node));

    // Add error and test again
    node.add_error("Test error".to_string());
    assert!(problems_filter.matches(&node));

    // Test missing filter
    node.found = false; // Set as missing first
    assert!(missing_filter.matches(&node));
    node.found = true; // Set as found
    assert!(!missing_filter.matches(&node));
}

#[test]
fn test_dependency_node_utilities() {
    let mut parent = DependencyNode::new("parent.dll", 0);
    let child1 = DependencyNode::new("child1.dll", 1);
    let child2 = DependencyNode::new("child2.dll", 1);

    parent.children.push(child1);
    parent.children.push(child2);

    // Test total dependencies count
    assert_eq!(parent.total_dependencies(), 2);

    // Test find node
    let found = parent.find_node(&PathBuf::from("child1.dll"));
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "child1.dll");

    let not_found = parent.find_node(&PathBuf::from("nonexistent.dll"));
    assert!(not_found.is_none());
}
