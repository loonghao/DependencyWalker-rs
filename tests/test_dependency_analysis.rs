//! Test program for dependency analysis functionality

use dependencywalker_rs::core::dependency::DependencyAnalyzer;
use dependencywalker_rs::core::tree::{NodeFilter, TraversalOrder};
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    // Get the DLL path from command line or use default
    let dll_path = env::args()
        .nth(1)
        .unwrap_or_else(|| "tests/liblzma.dll".to_string());

    println!("Testing dependency analysis with: {}", dll_path);
    println!("{}", "=".repeat(60));

    // Check if file exists
    if !std::path::Path::new(&dll_path).exists() {
        eprintln!("Error: File '{}' not found", dll_path);
        eprintln!("Usage: cargo run --example test_dependency_analysis [path_to_pe_file]");
        std::process::exit(1);
    }

    // Create dependency analyzer
    println!("Creating dependency analyzer...");
    let mut analyzer = DependencyAnalyzer::new();

    // Configure analyzer
    analyzer.set_max_depth(10);
    analyzer.set_include_system_dlls(true);
    analyzer.add_search_path("tests");

    println!("✓ Analyzer configured (max depth: 10, include system DLLs: true)");

    // Build dependency tree
    println!("\nBuilding dependency tree...");
    let tree = analyzer.build_tree(&dll_path)?;
    println!("✓ Dependency tree built successfully");

    // Display summary
    println!("\nAnalysis Summary:");
    println!("{}", tree.summary());

    // Display root information
    if let Some(root) = &tree.root {
        println!("\nRoot File Information:");
        println!("  Name: {}", root.name);
        println!("  Path: {}", root.path.display());
        println!("  Found: {}", root.found);
        println!(
            "  Architecture: {}",
            root.is_64bit
                .map_or("Unknown".to_string(), |is_64| if is_64 {
                    "x64".to_string()
                } else {
                    "x86".to_string()
                })
        );
        println!("  Direct dependencies: {}", root.dependencies.len());
        println!("  Total dependencies: {}", root.total_dependencies());

        if !root.dependencies.is_empty() {
            println!("\nDirect Dependencies:");
            for (i, dep) in root.dependencies.iter().enumerate() {
                println!("  {}. {}", i + 1, dep);
            }
        }

        if root.has_errors() {
            println!("\nRoot Errors:");
            for error in &root.errors {
                println!("  ❌ {}", error);
            }
        }
    }

    // Display all unique dependencies
    println!(
        "\nAll Unique Dependencies ({}):",
        tree.all_dependencies.len()
    );
    for (i, (path, info)) in tree.all_dependencies.iter().enumerate() {
        let status = if info.found { "✓" } else { "✗" };
        let arch = info.is_64bit.map_or("?".to_string(), |is_64| {
            if is_64 {
                "x64".to_string()
            } else {
                "x86".to_string()
            }
        });
        println!(
            "  {}. {} {} [{}] (refs: {}, depth: {})",
            i + 1,
            status,
            path.display(),
            arch,
            info.reference_count,
            info.first_seen_depth
        );
    }

    // Display missing dependencies
    if !tree.missing_dependencies.is_empty() {
        println!(
            "\nMissing Dependencies ({}):",
            tree.missing_dependencies.len()
        );
        for (i, missing) in tree.missing_dependencies.iter().enumerate() {
            println!("  {}. ❌ {}", i + 1, missing);
        }
    }

    // Display circular dependencies
    if !tree.circular_dependencies.is_empty() {
        println!(
            "\nCircular Dependencies ({}):",
            tree.circular_dependencies.len()
        );
        for (i, cycle) in tree.circular_dependencies.iter().enumerate() {
            println!("  {}. Cycle:", i + 1);
            for (j, path) in cycle.iter().enumerate() {
                println!(
                    "     {} {}",
                    if j == 0 { "→" } else { "  →" },
                    path.display()
                );
            }
        }
    }

    // Tree traversal examples
    println!("\nTree Traversal Examples:");

    // All nodes
    let all_nodes: Vec<_> = tree.iter(TraversalOrder::PreOrder).collect();
    println!("  Total nodes (pre-order): {}", all_nodes.len());

    // Missing nodes only
    let missing_nodes: Vec<_> = tree
        .iter_filtered(TraversalOrder::PreOrder, NodeFilter::missing_only())
        .collect();
    println!("  Missing nodes: {}", missing_nodes.len());

    // Error nodes
    let error_nodes = tree.get_error_nodes();
    println!("  Nodes with errors: {}", error_nodes.len());

    if !error_nodes.is_empty() {
        println!("\nError Details:");
        for (i, node) in error_nodes.iter().enumerate() {
            println!("  {}. {} (depth: {})", i + 1, node.name, node.depth);
            for error in &node.errors {
                println!("     ❌ {}", error);
            }
        }
    }

    // Display tree structure (first few levels)
    println!("\nDependency Tree Structure:");
    if let Some(root) = &tree.root {
        print_tree_node(root, 0, 3); // Max 3 levels for readability
    }

    // Performance statistics
    println!("\nPerformance Statistics:");
    println!("  Files parsed: {}", tree.stats.parsed_files);
    println!("  Parse errors: {}", tree.stats.parse_errors);
    println!("  Max depth reached: {}", tree.stats.max_depth);
    println!("  Analysis time: {}ms", tree.stats.analysis_time_ms);

    println!("\n{}", "=".repeat(60));
    println!("Dependency analysis completed successfully!");

    Ok(())
}

/// Helper function to print tree structure
fn print_tree_node(
    node: &dependencywalker_rs::core::dependency::DependencyNode,
    level: usize,
    max_level: usize,
) {
    if level > max_level {
        return;
    }

    let indent = "  ".repeat(level);
    let status = if node.found { "✓" } else { "✗" };
    let circular = if node.is_circular { " [CIRCULAR]" } else { "" };
    let errors = if node.has_errors() { " [ERRORS]" } else { "" };

    println!("{}{} {}{}{}", indent, status, node.name, circular, errors);

    // Print first few children
    for (i, child) in node.children.iter().enumerate() {
        if i < 5 {
            // Limit to first 5 children for readability
            print_tree_node(child, level + 1, max_level);
        } else if i == 5 {
            let child_indent = "  ".repeat(level + 1);
            println!(
                "{}... and {} more children",
                child_indent,
                node.children.len() - 5
            );
            break;
        }
    }
}
