//! Simple GUI test without complex dependencies
//! 
//! This example tests basic GUI functionality with minimal dependencies.

use std::path::PathBuf;

#[cfg(feature = "gui")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 DependencyWalker RS - Simple GUI Test");
    println!("=========================================");
    
    // Test basic dependency analysis without GUI
    test_dependency_analysis()?;
    
    println!("\n✅ Basic functionality test completed successfully!");
    println!("📝 Note: Full GUI test requires MSVC toolchain and proper Windows environment");
    
    Ok(())
}

#[cfg(not(feature = "gui"))]
fn main() {
    eprintln!("GUI feature not enabled. Please compile with --features gui");
    std::process::exit(1);
}

fn test_dependency_analysis() -> Result<(), Box<dyn std::error::Error>> {
    use dependencywalker_rs::core::dependency::DependencyAnalyzer;
    
    println!("🔍 Testing dependency analysis engine...");
    
    // Create analyzer
    let mut analyzer = DependencyAnalyzer::new();
    analyzer.set_max_depth(3);
    analyzer.set_include_system_dlls(true);
    
    // Test with a common Windows DLL
    let test_dll = "C:\\Windows\\System32\\kernel32.dll";
    
    if std::path::Path::new(test_dll).exists() {
        println!("📁 Analyzing: {}", test_dll);
        
        match analyzer.build_tree(test_dll) {
            Ok(tree) => {
                println!("✅ Analysis successful!");
                println!("📊 Tree summary: {}", tree.summary());
                
                if let Some(root) = &tree.root {
                    println!("🌳 Root module: {}", root.name);
                    println!("📦 Direct dependencies: {}", root.children.len());
                    
                    // Show first few dependencies
                    for (i, child) in root.children.iter().take(3).enumerate() {
                        let status = if child.found { "✓" } else { "✗" };
                        println!("  {} {} {}", status, i + 1, child.name);
                    }
                    
                    if root.children.len() > 3 {
                        println!("  ... and {} more", root.children.len() - 3);
                    }
                }
            }
            Err(e) => {
                println!("❌ Analysis failed: {}", e);
                return Err(e.into());
            }
        }
    } else {
        println!("⚠️  Test DLL not found, skipping analysis test");
    }
    
    Ok(())
}
