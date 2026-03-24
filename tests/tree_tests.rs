//! Unit tests for dependency tree utilities

use dependencywalker_rs::core::dependency::{DependencyNode, DependencyTree};
use dependencywalker_rs::core::tree::{NodeFilter, TraversalOrder};
use std::path::PathBuf;

fn create_test_node(name: &str, depth: usize, found: bool) -> DependencyNode {
    DependencyNode {
        name: name.to_string(),
        path: PathBuf::from(format!("C:\\test\\{}", name)),
        found,
        is_64bit: Some(true),
        depth,
        is_circular: false,
        dependencies: Vec::new(),
        children: Vec::new(),
        errors: Vec::new(),
    }
}

fn create_test_tree() -> DependencyTree {
    let mut root = create_test_node("root.exe", 0, true);

    // Add some children
    let mut child1 = create_test_node("child1.dll", 1, true);
    let child2 = create_test_node("child2.dll", 1, false); // Missing
    let child3 = create_test_node("kernel32.dll", 1, true); // System DLL

    // Add grandchildren
    let grandchild1 = create_test_node("grandchild1.dll", 2, true);
    let mut grandchild2 = create_test_node("grandchild2.dll", 2, true);
    grandchild2.errors.push("Test error".to_string());

    child1.children.push(grandchild1);
    child1.children.push(grandchild2);

    root.children.push(child1);
    root.children.push(child2);
    root.children.push(child3);

    DependencyTree {
        root: Some(root),
        all_dependencies: std::collections::HashMap::new(),
        circular_dependencies: Vec::new(),
        missing_dependencies: vec!["child2.dll".to_string()],
        stats: dependencywalker_rs::core::dependency::AnalysisStats {
            total_dependencies: 6,
            missing_count: 1,
            circular_count: 0,
            analysis_time_ms: 100,
            max_depth: 2,
            parsed_files: 5,
            parse_errors: 0,
        },
    }
}

#[test]
fn test_node_filter_default() {
    let filter = NodeFilter::default();

    assert!(!filter.errors_only);
    assert!(!filter.missing_only);
    assert!(!filter.circular_only);
    assert!(filter.max_depth.is_none());
    assert!(filter.include_system);
}

#[test]
fn test_node_filter_creation() {
    let filter = NodeFilter {
        errors_only: true,
        missing_only: false,
        circular_only: false,
        max_depth: Some(2),
        include_system: false,
    };

    assert!(filter.errors_only);
    assert!(!filter.missing_only);
    assert!(!filter.circular_only);
    assert_eq!(filter.max_depth, Some(2));
    assert!(!filter.include_system);
}

#[test]
fn test_node_filter_matches_found_node() {
    let filter = NodeFilter::default();
    let node = create_test_node("test.dll", 1, true);

    assert!(filter.matches(&node));
}

#[test]
fn test_node_filter_matches_missing_only() {
    let filter = NodeFilter {
        missing_only: true,
        ..NodeFilter::default()
    };

    let found_node = create_test_node("found.dll", 1, true);
    let missing_node = create_test_node("missing.dll", 1, false);

    assert!(!filter.matches(&found_node));
    assert!(filter.matches(&missing_node));
}

#[test]
fn test_node_filter_matches_errors_only() {
    let filter = NodeFilter {
        errors_only: true,
        ..NodeFilter::default()
    };

    let normal_node = create_test_node("normal.dll", 1, true);
    let mut error_node = create_test_node("error.dll", 1, true);
    error_node.errors.push("Test error".to_string());

    assert!(!filter.matches(&normal_node));
    assert!(filter.matches(&error_node));
}

#[test]
fn test_node_filter_matches_max_depth() {
    let filter = NodeFilter {
        max_depth: Some(1),
        ..NodeFilter::default()
    };

    let shallow_node = create_test_node("shallow.dll", 1, true);
    let deep_node = create_test_node("deep.dll", 2, true);

    assert!(filter.matches(&shallow_node));
    assert!(!filter.matches(&deep_node));
}

#[test]
fn test_node_filter_matches_system_dlls() {
    let filter = NodeFilter {
        include_system: false,
        ..NodeFilter::default()
    };

    let user_dll = create_test_node("user.dll", 1, true);
    let system_dll = create_test_node("kernel32.dll", 1, true);

    assert!(filter.matches(&user_dll));
    assert!(!filter.matches(&system_dll));
}

#[test]
fn test_node_filter_matches_circular() {
    let filter = NodeFilter {
        circular_only: true,
        ..NodeFilter::default()
    };

    let normal_node = create_test_node("normal.dll", 1, true);
    let mut circular_node = create_test_node("circular.dll", 1, true);
    circular_node.is_circular = true;

    assert!(!filter.matches(&normal_node));
    assert!(filter.matches(&circular_node));
}

#[test]
fn test_traversal_order_enum() {
    // Test that enum variants exist and can be used
    let _pre_order = TraversalOrder::PreOrder;
    let _post_order = TraversalOrder::PostOrder;
    let _breadth_first = TraversalOrder::BreadthFirst;

    // Test Debug trait
    assert!(format!("{:?}", TraversalOrder::PreOrder).contains("PreOrder"));
    assert!(format!("{:?}", TraversalOrder::PostOrder).contains("PostOrder"));
    assert!(format!("{:?}", TraversalOrder::BreadthFirst).contains("BreadthFirst"));
}

#[test]
fn test_traversal_order_debug() {
    // Test that TraversalOrder implements Debug
    let orders = vec![
        TraversalOrder::PreOrder,
        TraversalOrder::PostOrder,
        TraversalOrder::BreadthFirst,
    ];

    for order in orders {
        let debug_str = format!("{:?}", order);
        assert!(!debug_str.is_empty());
    }
}

#[test]
fn test_tree_summary() {
    let tree = create_test_tree();

    let summary = tree.summary();
    assert!(summary.contains("Dependencies: 6 total"));
    assert!(summary.contains("1 missing"));
    assert!(summary.contains("0 circular chains"));
    assert!(summary.contains("max depth: 2"));
}

#[test]
fn test_tree_get_error_nodes() {
    let tree = create_test_tree();

    let error_nodes = tree.get_error_nodes();
    assert_eq!(error_nodes.len(), 1);
    assert_eq!(error_nodes[0].name, "grandchild2.dll");
    assert!(!error_nodes[0].errors.is_empty());
}

#[test]
fn test_tree_get_missing_dependencies() {
    let tree = create_test_tree();

    let missing_deps = tree.get_missing_dependencies();
    assert_eq!(missing_deps.len(), 1);
    assert_eq!(missing_deps[0], "child2.dll");
}

#[test]
fn test_tree_get_circular_dependencies() {
    let mut tree = create_test_tree();

    // Initially no circular dependencies
    let circular_deps = tree.get_circular_dependencies();
    assert_eq!(circular_deps.len(), 0);

    // Add a circular dependency
    tree.circular_dependencies.push(vec![
        PathBuf::from("C:\\test\\child1.dll"),
        PathBuf::from("C:\\test\\grandchild1.dll"),
        PathBuf::from("C:\\test\\child1.dll"),
    ]);

    let circular_deps = tree.get_circular_dependencies();
    assert_eq!(circular_deps.len(), 1);
    assert_eq!(circular_deps[0].len(), 3);
}

#[test]
fn test_tree_stats() {
    let tree = create_test_tree();

    assert_eq!(tree.stats.total_dependencies, 6);
    assert_eq!(tree.stats.missing_count, 1);
    assert_eq!(tree.stats.circular_count, 0);
    assert_eq!(tree.stats.max_depth, 2);
}

#[test]
fn test_tree_has_issues() {
    let tree = create_test_tree();

    // Should have issues due to missing dependencies
    assert!(tree.has_issues());
}

#[test]
fn test_empty_tree() {
    let empty_tree = DependencyTree {
        root: None,
        all_dependencies: std::collections::HashMap::new(),
        circular_dependencies: Vec::new(),
        missing_dependencies: Vec::new(),
        stats: dependencywalker_rs::core::dependency::AnalysisStats {
            total_dependencies: 0,
            missing_count: 0,
            circular_count: 0,
            analysis_time_ms: 0,
            max_depth: 0,
            parsed_files: 0,
            parse_errors: 0,
        },
    };

    let error_nodes = empty_tree.get_error_nodes();
    assert_eq!(error_nodes.len(), 0);

    let missing_deps = empty_tree.get_missing_dependencies();
    assert_eq!(missing_deps.len(), 0);

    assert!(!empty_tree.has_issues());
}
