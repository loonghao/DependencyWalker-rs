//! Example demonstrating DLL search path resolution
//! 
//! This example shows how to use the DLL resolver to find DLLs using Windows search paths.

use dependencywalker_rs::core::{DllResolver, DllResolverConfig, ModuleSearchStrategy};
use dependencywalker_rs::core::pe_parser::{PEFileMap, PEFile};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    println!("DependencyWalker RS - DLL Resolver Example");
    println!("==========================================");
    
    // Create a DLL resolver with default configuration
    let mut resolver = DllResolver::new();
    
    println!("\n1. Testing simple DLL resolution:");
    test_simple_resolution(&mut resolver)?;
    
    println!("\n2. Testing DLL resolution with PE context:");
    test_pe_context_resolution(&mut resolver)?;
    
    println!("\n3. Testing custom search paths:");
    test_custom_search_paths(&mut resolver)?;
    
    println!("\n4. Testing different configurations:");
    test_different_configurations()?;
    
    Ok(())
}

fn test_simple_resolution(resolver: &mut DllResolver) -> Result<(), Box<dyn std::error::Error>> {
    let test_dlls = [
        "kernel32.dll",
        "user32.dll", 
        "ntdll.dll",
        "nonexistent.dll",
    ];
    
    for dll_name in &test_dlls {
        match resolver.resolve_dll_simple(dll_name) {
            Ok(Some(path)) => {
                println!("  ✓ Found {}: {}", dll_name, path.display());
            }
            Ok(None) => {
                println!("  ✗ Not found: {}", dll_name);
            }
            Err(e) => {
                println!("  ⚠ Error resolving {}: {:?}", dll_name, e);
            }
        }
    }
    
    Ok(())
}

fn test_pe_context_resolution(resolver: &mut DllResolver) -> Result<(), Box<dyn std::error::Error>> {
    // Try to use the test DLL if it exists
    let test_dll_path = "tests/liblzma.dll";
    
    if std::path::Path::new(test_dll_path).exists() {
        println!("  Using test DLL: {}", test_dll_path);
        
        let pe_map = PEFileMap::new(test_dll_path)?;
        let pe_file = PEFile::new(&pe_map)?;
        
        // Get dependencies and try to resolve them
        let dependencies = pe_file.get_dependencies()?;
        println!("  Dependencies found: {:?}", dependencies);
        
        for dep in &dependencies {
            match resolver.resolve_dll(&pe_file, dep) {
                Ok((strategy, Some(path))) => {
                    println!("    ✓ {} -> {} (via {:?})", dep, path.display(), strategy);
                }
                Ok((strategy, None)) => {
                    println!("    ✗ {} not found (strategy: {:?})", dep, strategy);
                }
                Err(e) => {
                    println!("    ⚠ Error resolving {}: {:?}", dep, e);
                }
            }
        }
    } else {
        println!("  Test DLL not found, skipping PE context test");
    }
    
    Ok(())
}

fn test_custom_search_paths(resolver: &mut DllResolver) -> Result<(), Box<dyn std::error::Error>> {
    // Add some custom search paths
    resolver.add_search_path("C:\\Program Files\\Common Files");
    resolver.add_search_path("C:\\Windows\\WinSxS");
    
    println!("  Added custom search paths:");
    for path in &resolver.config().custom_search_paths {
        println!("    - {}", path.display());
    }
    
    // Test resolution with custom paths
    match resolver.resolve_dll_simple("kernel32.dll") {
        Ok(Some(path)) => {
            println!("  ✓ Found kernel32.dll with custom paths: {}", path.display());
        }
        Ok(None) => {
            println!("  ✗ kernel32.dll not found even with custom paths");
        }
        Err(e) => {
            println!("  ⚠ Error: {:?}", e);
        }
    }
    
    Ok(())
}

fn test_different_configurations() -> Result<(), Box<dyn std::error::Error>> {
    // Test with system DLLs disabled
    let config1 = DllResolverConfig {
        include_system_dlls: false,
        custom_search_paths: vec![PathBuf::from("C:\\test")],
        working_directory: None,
        enable_wow64_redirection: false,
        enable_known_dlls: false,
    };
    
    let mut resolver1 = DllResolver::with_config(config1);
    println!("  Testing with system DLLs disabled:");
    match resolver1.resolve_dll_simple("kernel32.dll") {
        Ok(Some(path)) => {
            println!("    ✓ Still found kernel32.dll: {}", path.display());
        }
        Ok(None) => {
            println!("    ✗ kernel32.dll not found (expected with system DLLs disabled)");
        }
        Err(e) => {
            println!("    ⚠ Error: {:?}", e);
        }
    }
    
    // Test with KnownDLLs disabled
    let config2 = DllResolverConfig {
        include_system_dlls: true,
        custom_search_paths: Vec::new(),
        working_directory: None,
        enable_wow64_redirection: true,
        enable_known_dlls: false,
    };
    
    let mut resolver2 = DllResolver::with_config(config2);
    println!("  Testing with KnownDLLs disabled:");
    match resolver2.resolve_dll_simple("kernel32.dll") {
        Ok(Some(path)) => {
            println!("    ✓ Found kernel32.dll without KnownDLLs: {}", path.display());
        }
        Ok(None) => {
            println!("    ✗ kernel32.dll not found without KnownDLLs");
        }
        Err(e) => {
            println!("    ⚠ Error: {:?}", e);
        }
    }
    
    // Test with custom working directory
    let config3 = DllResolverConfig {
        include_system_dlls: true,
        custom_search_paths: Vec::new(),
        working_directory: Some(PathBuf::from("C:\\Windows\\System32")),
        enable_wow64_redirection: true,
        enable_known_dlls: true,
    };
    
    let resolver3 = DllResolver::with_config(config3);
    println!("  Testing with custom working directory:");
    println!("    Working directory: {:?}", resolver3.config().working_directory);
    
    Ok(())
}
