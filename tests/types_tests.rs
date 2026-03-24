//! Unit tests for PE types and data structures

use dependencywalker_rs::core::types::{
    Architecture, DetailedPEInfo, ExportInfo, ImportInfo, PEType, ParseStats, VersionInfo,
};
use std::collections::HashMap;
use std::path::PathBuf;

#[test]
fn test_architecture_string_representation() {
    assert_eq!(Architecture::X86.as_str(), "x86");
    assert_eq!(Architecture::X64.as_str(), "x64");
    assert_eq!(Architecture::ARM.as_str(), "ARM");
    assert_eq!(Architecture::ARM64.as_str(), "ARM64");
    assert_eq!(Architecture::Unknown.as_str(), "Unknown");
}

#[test]
fn test_pe_type_string_representation() {
    assert_eq!(PEType::Executable.as_str(), "EXE");
    assert_eq!(PEType::DynamicLibrary.as_str(), "DLL");
    assert_eq!(PEType::Driver.as_str(), "SYS");
    assert_eq!(PEType::Unknown.as_str(), "Unknown");
}

#[test]
fn test_import_info_creation() {
    let import = ImportInfo {
        name: Some("CreateFileW".to_string()),
        ordinal: Some(42),
        hint: Some(100),
        address: Some(0x1000),
    };

    assert_eq!(import.name, Some("CreateFileW".to_string()));
    assert_eq!(import.ordinal, Some(42));
    assert_eq!(import.hint, Some(100));
    assert_eq!(import.address, Some(0x1000));
}

#[test]
fn test_export_info_creation() {
    let export = ExportInfo {
        name: Some("MyFunction".to_string()),
        ordinal: 1,
        rva: 0x2000,
        is_forwarded: false,
        forward_name: None,
    };

    assert_eq!(export.name, Some("MyFunction".to_string()));
    assert_eq!(export.ordinal, 1);
    assert_eq!(export.rva, 0x2000);
    assert!(!export.is_forwarded);
    assert!(export.forward_name.is_none());
}

#[test]
fn test_version_info_creation() {
    let version = VersionInfo {
        file_version: Some("1.0.0.0".to_string()),
        product_version: Some("1.0.0".to_string()),
        company_name: Some("Test Company".to_string()),
        file_description: Some("Test Application".to_string()),
        product_name: Some("Test Product".to_string()),
        copyright: Some("Copyright 2025".to_string()),
    };

    assert_eq!(version.file_version, Some("1.0.0.0".to_string()));
    assert_eq!(version.product_version, Some("1.0.0".to_string()));
    assert_eq!(version.company_name, Some("Test Company".to_string()));
}

#[test]
fn test_detailed_pe_info_creation() {
    let mut imports = HashMap::new();
    imports.insert(
        "kernel32.dll".to_string(),
        vec![ImportInfo {
            name: Some("CreateFileW".to_string()),
            ordinal: None,
            hint: Some(100),
            address: Some(0x1000),
        }],
    );

    let exports = vec![ExportInfo {
        name: Some("MyExport".to_string()),
        ordinal: 1,
        rva: 0x2000,
        is_forwarded: false,
        forward_name: None,
    }];

    let version_info = VersionInfo {
        file_version: Some("1.0.0.0".to_string()),
        product_version: Some("1.0.0".to_string()),
        company_name: Some("Test Company".to_string()),
        file_description: Some("Test DLL".to_string()),
        product_name: Some("Test Product".to_string()),
        copyright: Some("Copyright 2025".to_string()),
    };

    let pe_info = DetailedPEInfo {
        path: PathBuf::from("test.dll"),
        file_size: 1024,
        architecture: Architecture::X64,
        pe_type: PEType::DynamicLibrary,
        dll_name: Some("test.dll".to_string()),
        entry_point: Some(0x1000),
        image_base: Some(0x10000000),
        subsystem: Some("Windows GUI".to_string()),
        dependencies: vec!["kernel32.dll".to_string(), "user32.dll".to_string()],
        imports,
        exports,
        version_info: Some(version_info),
    };

    assert_eq!(pe_info.path, PathBuf::from("test.dll"));
    assert_eq!(pe_info.file_size, 1024);
    assert_eq!(pe_info.architecture, Architecture::X64);
    assert_eq!(pe_info.pe_type, PEType::DynamicLibrary);
    assert_eq!(pe_info.dll_name, Some("test.dll".to_string()));
    assert_eq!(pe_info.dependencies.len(), 2);
    assert_eq!(pe_info.imports.len(), 1);
    assert_eq!(pe_info.exports.len(), 1);
    assert!(pe_info.version_info.is_some());
}

#[test]
fn test_parse_stats_creation() {
    let mut stats = ParseStats::default();

    assert_eq!(stats.dependency_count, 0);
    assert_eq!(stats.import_count, 0);
    assert_eq!(stats.export_count, 0);
    assert_eq!(stats.parse_time_ms, 0);
    assert!(stats.parser_used.is_none());
    assert!(stats.warnings.is_empty());
    assert!(!stats.is_successful());

    // Test adding warnings
    stats.add_warning("Test warning".to_string());
    assert_eq!(stats.warnings.len(), 1);
    assert_eq!(stats.warnings[0], "Test warning");

    // Test successful parsing
    stats.parser_used = Some("goblin".to_string());
    assert!(stats.is_successful());
}

#[test]
fn test_parse_stats_with_data() {
    let stats = ParseStats {
        dependency_count: 5,
        import_count: 100,
        export_count: 20,
        parse_time_ms: 150,
        parser_used: Some("pelite".to_string()),
        warnings: vec!["Warning 1".to_string(), "Warning 2".to_string()],
    };

    assert_eq!(stats.dependency_count, 5);
    assert_eq!(stats.import_count, 100);
    assert_eq!(stats.export_count, 20);
    assert_eq!(stats.parse_time_ms, 150);
    assert_eq!(stats.parser_used, Some("pelite".to_string()));
    assert_eq!(stats.warnings.len(), 2);
    assert!(stats.is_successful());
}

#[test]
fn test_architecture_equality() {
    assert_eq!(Architecture::X86, Architecture::X86);
    assert_ne!(Architecture::X86, Architecture::X64);
    assert_ne!(Architecture::ARM, Architecture::ARM64);
}

#[test]
fn test_pe_type_equality() {
    assert_eq!(PEType::Executable, PEType::Executable);
    assert_ne!(PEType::Executable, PEType::DynamicLibrary);
    assert_ne!(PEType::Driver, PEType::Unknown);
}

#[test]
fn test_import_info_optional_fields() {
    // Test with minimal fields
    let import1 = ImportInfo {
        name: Some("Function1".to_string()),
        ordinal: None,
        hint: None,
        address: None,
    };

    assert!(import1.name.is_some());
    assert!(import1.ordinal.is_none());
    assert!(import1.hint.is_none());
    assert!(import1.address.is_none());

    // Test with ordinal only
    let import2 = ImportInfo {
        name: None,
        ordinal: Some(42),
        hint: None,
        address: Some(0x1000),
    };

    assert!(import2.name.is_none());
    assert_eq!(import2.ordinal, Some(42));
    assert_eq!(import2.address, Some(0x1000));
}

#[test]
fn test_export_info_with_forwarder() {
    let export = ExportInfo {
        name: Some("ForwardedFunction".to_string()),
        ordinal: 10,
        rva: 0,
        is_forwarded: true,
        forward_name: Some("KERNEL32.CreateFileW".to_string()),
    };

    assert_eq!(export.name, Some("ForwardedFunction".to_string()));
    assert_eq!(export.ordinal, 10);
    assert_eq!(export.rva, 0);
    assert!(export.is_forwarded);
    assert_eq!(
        export.forward_name,
        Some("KERNEL32.CreateFileW".to_string())
    );
}
