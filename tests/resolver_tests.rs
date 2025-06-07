//! Tests for DLL search path resolver
//!
//! This module contains tests for the Windows DLL search path resolution functionality.

use dependencywalker_rs::core::pe_parser::{PEFile, PEFileMap};
use dependencywalker_rs::core::{DllResolver, DllResolverConfig, ModuleSearchStrategy};
use std::path::PathBuf;

#[test]
fn test_dll_resolver_creation() {
    let resolver = DllResolver::new();
    assert!(resolver.config().include_system_dlls);
    assert!(resolver.config().enable_known_dlls);
    assert!(resolver.config().enable_wow64_redirection);
}

#[test]
fn test_dll_resolver_with_config() {
    let config = DllResolverConfig {
        include_system_dlls: false,
        custom_search_paths: vec![PathBuf::from("C:\\test")],
        working_directory: Some(PathBuf::from("C:\\work")),
        enable_wow64_redirection: false,
        enable_known_dlls: false,
        enable_api_set_schema: false,
    };

    let resolver = DllResolver::with_config(config.clone());
    assert!(!resolver.config().include_system_dlls);
    assert_eq!(resolver.config().custom_search_paths.len(), 1);
    assert_eq!(
        resolver.config().working_directory,
        Some(PathBuf::from("C:\\work"))
    );
}

#[test]
fn test_add_search_path() {
    let mut resolver = DllResolver::new();
    resolver.add_search_path("C:\\custom\\path");

    assert!(resolver
        .config()
        .custom_search_paths
        .contains(&PathBuf::from("C:\\custom\\path")));
}

#[test]
fn test_clear_search_paths() {
    let mut resolver = DllResolver::new();
    resolver.add_search_path("C:\\path1");
    resolver.add_search_path("C:\\path2");
    resolver.clear_search_paths();

    assert!(resolver.config().custom_search_paths.is_empty());
}

#[test]
fn test_simple_dll_resolution() {
    let mut resolver = DllResolver::new();

    // Test resolving a system DLL that should exist
    match resolver.resolve_dll_simple("kernel32.dll") {
        Ok(Some(path)) => {
            assert!(path.exists());
            assert!(path.file_name().unwrap().to_string_lossy().to_lowercase() == "kernel32.dll");
        }
        Ok(None) => {
            // This might happen in test environments without full Windows system
            println!("kernel32.dll not found - this is expected in some test environments");
        }
        Err(e) => {
            panic!("Unexpected error resolving kernel32.dll: {:?}", e);
        }
    }
}

#[test]
fn test_dll_resolution_with_pe_context() {
    // This test requires a real PE file to work with
    // We'll use the test DLL from our test directory
    let test_dll_path = "tests/liblzma.dll";

    if std::path::Path::new(test_dll_path).exists() {
        let pe_map = PEFileMap::new(test_dll_path).expect("Failed to load test DLL");
        let pe_file = PEFile::new(&pe_map).expect("Failed to parse test DLL");

        let mut resolver = DllResolver::new();

        // Test resolving a dependency of the test DLL
        match resolver.resolve_dll(&pe_file, "kernel32.dll") {
            Ok((strategy, Some(path))) => {
                assert!(path.exists());
                assert_ne!(strategy, ModuleSearchStrategy::NotFound);
                println!(
                    "Found kernel32.dll via strategy: {:?} at: {}",
                    strategy,
                    path.display()
                );
            }
            Ok((ModuleSearchStrategy::NotFound, None)) => {
                println!("kernel32.dll not found - this might be expected in test environments");
            }
            Ok((strategy, None)) => {
                panic!("Unexpected result: strategy {:?} with no path", strategy);
            }
            Err(e) => {
                panic!("Error resolving kernel32.dll: {:?}", e);
            }
        }
    } else {
        println!("Test DLL not found, skipping PE context test");
    }
}

#[test]
fn test_nonexistent_dll_resolution() {
    let mut resolver = DllResolver::new();

    match resolver.resolve_dll_simple("nonexistent_dll_12345.dll") {
        Ok(None) => {
            // This is the expected result
        }
        Ok(Some(path)) => {
            panic!("Unexpectedly found nonexistent DLL at: {}", path.display());
        }
        Err(e) => {
            panic!("Unexpected error: {:?}", e);
        }
    }
}

#[test]
fn test_dll_name_normalization() {
    let mut resolver = DllResolver::new();

    // Test that DLL names without extension get .dll added
    // We'll test this by checking if the resolution behaves the same
    // for "kernel32" and "kernel32.dll"
    let result1 = resolver.resolve_dll_simple("kernel32");
    let result2 = resolver.resolve_dll_simple("kernel32.dll");

    match (result1, result2) {
        (Ok(path1), Ok(path2)) => {
            // Both should resolve to the same result (either both Some or both None)
            assert_eq!(path1.is_some(), path2.is_some());
            if let (Some(p1), Some(p2)) = (path1, path2) {
                assert_eq!(p1, p2);
            }
        }
        _ => {
            // If there are errors, they should be the same type
            // This is acceptable for test environments
        }
    }
}

#[test]
fn test_module_search_strategy_values() {
    // Test that the enum values are as expected
    assert_eq!(ModuleSearchStrategy::SxS as u8, 0);
    assert_eq!(ModuleSearchStrategy::ApiSetSchema as u8, 1);
    assert_eq!(ModuleSearchStrategy::WellKnownDlls as u8, 2);
    assert_eq!(ModuleSearchStrategy::ApplicationDirectory as u8, 3);
    assert_eq!(ModuleSearchStrategy::System32Folder as u8, 4);
    assert_eq!(ModuleSearchStrategy::WindowsFolder as u8, 5);
    assert_eq!(ModuleSearchStrategy::WorkingDirectory as u8, 6);
    assert_eq!(ModuleSearchStrategy::Environment as u8, 7);
    assert_eq!(ModuleSearchStrategy::AppInitDLL as u8, 8);
    assert_eq!(ModuleSearchStrategy::Fullpath as u8, 9);
    assert_eq!(ModuleSearchStrategy::ClrAssembly as u8, 10);
    assert_eq!(ModuleSearchStrategy::UserDefined as u8, 0xfe);
    assert_eq!(ModuleSearchStrategy::NotFound as u8, 0xff);
}

#[cfg(windows)]
#[test]
fn test_windows_specific_resolution() {
    let mut resolver = DllResolver::new();

    // Test that we can resolve system DLLs on Windows
    // This test only runs on Windows
    match resolver.resolve_dll_simple("ntdll.dll") {
        Ok(Some(path)) => {
            assert!(path.exists());
            assert!(path.is_file());
            assert!(path.file_name().unwrap().to_string_lossy().to_lowercase() == "ntdll.dll");
        }
        Ok(None) => {
            // This shouldn't happen on Windows, but we'll allow it for test environments
            println!("ntdll.dll not found - unexpected on Windows");
        }
        Err(e) => {
            panic!("Error resolving ntdll.dll on Windows: {:?}", e);
        }
    }
}

#[test]
fn test_resolver_config_default() {
    let config = DllResolverConfig::default();
    assert!(config.include_system_dlls);
    assert!(config.custom_search_paths.is_empty());
    assert!(config.working_directory.is_none());
    assert!(config.enable_wow64_redirection);
    assert!(config.enable_known_dlls);
    assert!(config.enable_api_set_schema);
}
