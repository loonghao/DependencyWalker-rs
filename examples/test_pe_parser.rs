//! Test program for PE parser functionality

use dependencywalker_rs::core::pe_parser::{PEFileMap, PEFile};
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    // Get the DLL path from command line or use default
    let dll_path = env::args().nth(1).unwrap_or_else(|| "tests/liblzma.dll".to_string());
    
    println!("Testing PE parser with: {}", dll_path);
    println!("{}", "=".repeat(50));
    
    // Check if file exists
    if !std::path::Path::new(&dll_path).exists() {
        eprintln!("Error: File '{}' not found", dll_path);
        eprintln!("Usage: cargo run --example test_pe_parser [path_to_pe_file]");
        std::process::exit(1);
    }
    
    // Create PE file map
    println!("Creating PE file map...");
    let pe_map = PEFileMap::new(&dll_path)?;
    println!("✓ Successfully loaded file ({} bytes)", pe_map.content().len());
    
    // Parse the PE file
    println!("\nParsing PE file...");
    let pe_file = PEFile::new(&pe_map)?;
    println!("✓ Successfully parsed PE file");
    
    // Test architecture detection
    println!("\nArchitecture Detection:");
    match pe_file.is_64bit() {
        Ok(is_64) => {
            let arch = if is_64 { "x64 (64-bit)" } else { "x86 (32-bit)" };
            println!("✓ Architecture: {}", arch);
        }
        Err(e) => println!("✗ Failed to detect architecture: {:?}", e),
    }
    
    // Test DLL name extraction
    println!("\nDLL Name Extraction:");
    match pe_file.get_dll_name() {
        Ok(Some(name)) => println!("✓ DLL name: {}", name),
        Ok(None) => println!("ℹ No DLL name found (might be an EXE)"),
        Err(e) => println!("✗ Failed to get DLL name: {:?}", e),
    }
    
    // Test dependency extraction
    println!("\nDependency Analysis:");
    match pe_file.get_dependencies() {
        Ok(deps) => {
            println!("✓ Found {} dependencies:", deps.len());
            for (i, dep) in deps.iter().enumerate() {
                println!("  {}. {}", i + 1, dep);
            }
            if deps.is_empty() {
                println!("  (No dependencies found)");
            }
        }
        Err(e) => println!("✗ Failed to get dependencies: {:?}", e),
    }
    
    // Test import extraction
    println!("\nImport Analysis:");
    match pe_file.get_imports() {
        Ok(imports) => {
            println!("✓ Found imports from {} DLLs:", imports.len());
            let mut total_symbols = 0;
            for (dll, symbols) in &imports {
                total_symbols += symbols.len();
                println!("  {} ({} symbols)", dll, symbols.len());
                
                // Show first few symbols as examples
                let mut shown = 0;
                for symbol in symbols {
                    if shown < 3 {
                        println!("    - {}", symbol);
                        shown += 1;
                    } else {
                        println!("    ... and {} more", symbols.len() - shown);
                        break;
                    }
                }
            }
            println!("  Total imported symbols: {}", total_symbols);
        }
        Err(e) => println!("✗ Failed to get imports: {:?}", e),
    }
    
    // Test export extraction
    println!("\nExport Analysis:");
    match pe_file.get_exports() {
        Ok(exports) => {
            println!("✓ Found {} exports:", exports.len());
            
            // Show first few exports as examples
            for (i, export) in exports.iter().enumerate() {
                if i < 10 {
                    println!("  {}. {}", i + 1, export);
                } else {
                    println!("  ... and {} more", exports.len() - 10);
                    break;
                }
            }
            
            if exports.is_empty() {
                println!("  (No exports found)");
            }
        }
        Err(e) => println!("✗ Failed to get exports: {:?}", e),
    }
    
    // Test getting complete info
    println!("\nComplete PE Information:");
    match pe_file.get_info() {
        Ok(info) => {
            println!("✓ {}", info.description());
            println!("  File: {}", info.path.display());
            println!("  Type: {}", if info.is_dll() { "Dynamic Library (DLL)" } else { "Executable (EXE)" });
            println!("  Architecture: {}", if info.is_64bit { "64-bit" } else { "32-bit" });
            println!("  Dependencies: {}", info.dependencies.len());
            println!("  Imported symbols: {}", info.import_count);
            println!("  Exported symbols: {}", info.export_count);
        }
        Err(e) => println!("✗ Failed to get PE info: {:?}", e),
    }
    
    println!("\n{}", "=".repeat(50));
    println!("PE analysis completed successfully!");
    
    Ok(())
}
