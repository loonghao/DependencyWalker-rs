//! Unit tests for PE parser functionality

use dependencywalker_rs::core::pe_parser::{PEFileMap, PEFile};
use dependencywalker_rs::error::Error;
use std::path::Path;

#[test]
fn test_pe_file_map_nonexistent_file() {
    let result = PEFileMap::new("nonexistent_file.exe");
    assert!(result.is_err());

    if let Err(Error::FileNotFound { path }) = result {
        assert_eq!(path, Path::new("nonexistent_file.exe"));
    } else {
        panic!("Expected FileNotFound error");
    }
}

#[test]
fn test_pe_info_structure() {
    // Test the PEInfo structure and its methods
    use dependencywalker_rs::core::pe_parser::PEInfo;
    use std::path::PathBuf;
    
    let info = PEInfo {
        path: PathBuf::from("test.dll"),
        is_64bit: false,
        dll_name: Some("test.dll".to_string()),
        dependencies: vec!["kernel32.dll".to_string(), "user32.dll".to_string()],
        import_count: 10,
        export_count: 5,
    };
    
    assert!(info.is_dll());
    assert!(!info.is_64bit);
    assert_eq!(info.dependencies.len(), 2);
    
    let description = info.description();
    assert!(description.contains("x86"));
    assert!(description.contains("DLL"));
    assert!(description.contains("2 dependencies"));
    assert!(description.contains("10 imports"));
    assert!(description.contains("5 exports"));
}

#[test]
fn test_pe_info_exe() {
    use dependencywalker_rs::core::pe_parser::PEInfo;
    use std::path::PathBuf;
    
    let info = PEInfo {
        path: PathBuf::from("test.exe"),
        is_64bit: true,
        dll_name: None,
        dependencies: vec!["kernel32.dll".to_string()],
        import_count: 20,
        export_count: 0,
    };
    
    assert!(!info.is_dll());
    assert!(info.is_64bit);
    
    let description = info.description();
    assert!(description.contains("x64"));
    assert!(description.contains("EXE"));
    assert!(description.contains("1 dependencies"));
    assert!(description.contains("20 imports"));
    assert!(description.contains("0 exports"));
}

#[test]
fn test_real_dll_parsing() {
    // Test with the provided liblzma.dll file
    let dll_path = "tests/liblzma.dll";

    // Check if the test file exists
    if !std::path::Path::new(dll_path).exists() {
        println!("Skipping real DLL test - liblzma.dll not found");
        return;
    }

    println!("Testing with real DLL: {}", dll_path);

    // Create PE file map
    let pe_map = PEFileMap::new(dll_path).expect("Failed to create PEFileMap for liblzma.dll");

    // Parse the PE file
    let pe_file = PEFile::new(&pe_map).expect("Failed to parse liblzma.dll");

    // Test basic properties
    println!("PE file path: {}", pe_file.path().display());

    // Test architecture detection
    match pe_file.is_64bit() {
        Ok(is_64) => println!("Architecture: {}", if is_64 { "x64" } else { "x86" }),
        Err(e) => println!("Failed to detect architecture: {:?}", e),
    }

    // Test DLL name extraction
    match pe_file.get_dll_name() {
        Ok(Some(name)) => {
            println!("DLL name: {}", name);
            assert!(!name.is_empty());
        }
        Ok(None) => println!("No DLL name found (might be an EXE)"),
        Err(e) => println!("Failed to get DLL name: {:?}", e),
    }

    // Test dependency extraction
    match pe_file.get_dependencies() {
        Ok(deps) => {
            println!("Found {} dependencies:", deps.len());
            for dep in &deps {
                println!("  - {}", dep);
            }
            // liblzma.dll should have some dependencies
            assert!(!deps.is_empty(), "Expected liblzma.dll to have dependencies");
        }
        Err(e) => panic!("Failed to get dependencies: {:?}", e),
    }

    // Test import extraction
    match pe_file.get_imports() {
        Ok(imports) => {
            println!("Found imports from {} DLLs:", imports.len());
            for (dll, symbols) in &imports {
                println!("  {} ({} symbols)", dll, symbols.len());
                // Print first few symbols as examples
                for (i, symbol) in symbols.iter().enumerate() {
                    if i < 3 {
                        println!("    - {}", symbol);
                    } else if i == 3 {
                        println!("    ... and {} more", symbols.len() - 3);
                        break;
                    }
                }
            }
        }
        Err(e) => println!("Failed to get imports: {:?}", e),
    }

    // Test export extraction
    match pe_file.get_exports() {
        Ok(exports) => {
            println!("Found {} exports:", exports.len());
            // Print first few exports as examples
            for (i, export) in exports.iter().enumerate() {
                if i < 5 {
                    println!("  - {}", export);
                } else if i == 5 {
                    println!("  ... and {} more", exports.len() - 5);
                    break;
                }
            }
            // liblzma.dll should have exports
            assert!(!exports.is_empty(), "Expected liblzma.dll to have exports");
        }
        Err(e) => println!("Failed to get exports: {:?}", e),
    }

    // Test getting complete info
    match pe_file.get_info() {
        Ok(info) => {
            println!("PE Info: {}", info.description());
            assert_eq!(info.path.file_name().unwrap().to_str().unwrap(), "liblzma.dll");
            assert!(info.dependencies.len() > 0);
        }
        Err(e) => println!("Failed to get PE info: {:?}", e),
    }
}
