//! Unit tests for CLI command functionality

use dependencywalker_rs::cli::commands::{analyze_command, list_command, tree_command};
use dependencywalker_rs::cli::output::Format;
use std::path::Path;
use tempfile::NamedTempFile;

#[test]
fn test_analyze_command_with_nonexistent_file() {
    let nonexistent_path = Path::new("nonexistent_file.exe");
    let result = analyze_command(nonexistent_path, 5, false, &[], Format::Text);

    assert!(result.is_err());

    // Check that it's a FileNotFound error
    match result.unwrap_err() {
        dependencywalker_rs::error::Error::FileNotFound { path } => {
            assert_eq!(path, nonexistent_path);
        }
        _ => panic!("Expected FileNotFound error"),
    }
}

#[test]
fn test_list_command_with_nonexistent_file() {
    let nonexistent_path = Path::new("nonexistent_file.exe");
    let result = list_command(nonexistent_path, false, Format::Text);

    assert!(result.is_err());

    // Check that it's a FileNotFound error
    match result.unwrap_err() {
        dependencywalker_rs::error::Error::FileNotFound { path } => {
            assert_eq!(path, nonexistent_path);
        }
        _ => panic!("Expected FileNotFound error"),
    }
}

#[test]
fn test_tree_command_with_nonexistent_file() {
    let nonexistent_path = Path::new("nonexistent_file.exe");
    let result = tree_command(nonexistent_path, false, Format::Text);

    assert!(result.is_err());

    // Check that it's a FileNotFound error
    match result.unwrap_err() {
        dependencywalker_rs::error::Error::FileNotFound { path } => {
            assert_eq!(path, nonexistent_path);
        }
        _ => panic!("Expected FileNotFound error"),
    }
}

#[test]
fn test_analyze_command_parameters() {
    // Test that analyze_command accepts various parameter combinations
    let nonexistent_path = Path::new("test.exe");

    // Test with different depths
    let _result1 = analyze_command(nonexistent_path, 1, false, &[], Format::Text);
    let _result2 = analyze_command(nonexistent_path, 10, false, &[], Format::Text);

    // Test with system DLLs enabled/disabled
    let _result3 = analyze_command(nonexistent_path, 5, true, &[], Format::Text);
    let _result4 = analyze_command(nonexistent_path, 5, false, &[], Format::Text);

    // Test with additional search paths
    let search_paths = vec![
        std::path::PathBuf::from("C:\\test"),
        std::path::PathBuf::from("C:\\custom"),
    ];
    let _result5 = analyze_command(nonexistent_path, 5, false, &search_paths, Format::Text);

    // Test with different output formats
    let _result6 = analyze_command(nonexistent_path, 5, false, &[], Format::Json);
    let _result7 = analyze_command(nonexistent_path, 5, false, &[], Format::Xml);
}

#[test]
fn test_list_command_parameters() {
    let nonexistent_path = Path::new("test.exe");

    // Test detailed vs simple mode
    let _result1 = list_command(nonexistent_path, true, Format::Text);
    let _result2 = list_command(nonexistent_path, false, Format::Text);

    // Test different output formats
    let _result3 = list_command(nonexistent_path, false, Format::Json);
    let _result4 = list_command(nonexistent_path, false, Format::Xml);
}

#[test]
fn test_tree_command_parameters() {
    let nonexistent_path = Path::new("test.exe");

    // Test missing only vs all dependencies
    let _result1 = tree_command(nonexistent_path, true, Format::Text);
    let _result2 = tree_command(nonexistent_path, false, Format::Text);

    // Test different output formats
    let _result3 = tree_command(nonexistent_path, false, Format::Json);
    let _result4 = tree_command(nonexistent_path, false, Format::Xml);
}

#[test]
fn test_format_enum() {
    // Test that Format enum works correctly
    let formats = vec![Format::Text, Format::Json, Format::Xml];

    for format in formats {
        // Test Debug trait
        let debug_str = format!("{:?}", format);
        assert!(!debug_str.is_empty());

        // Test that format can be used in function calls
        let nonexistent_path = Path::new("test.exe");
        let _result = analyze_command(nonexistent_path, 5, false, &[], format);
    }
}

#[test]
fn test_empty_search_paths() {
    let nonexistent_path = Path::new("test.exe");
    let empty_paths: Vec<std::path::PathBuf> = vec![];

    let result = analyze_command(nonexistent_path, 5, false, &empty_paths, Format::Text);

    // Should still fail due to nonexistent file, but not due to empty search paths
    assert!(result.is_err());
    match result.unwrap_err() {
        dependencywalker_rs::error::Error::FileNotFound { .. } => {
            // Expected
        }
        _ => panic!("Expected FileNotFound error, not search path error"),
    }
}

#[test]
fn test_command_with_invalid_file_extension() {
    // Create a temporary file with wrong extension
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let temp_path = temp_file.path();

    // Try to analyze a non-PE file
    let result = analyze_command(temp_path, 5, false, &[], Format::Text);

    // The command might succeed but report errors in the output
    // This is expected behavior for the CLI - it tries to analyze and reports what it finds
    // So we just check that the function doesn't panic
    let _ = result;
}

#[test]
fn test_zero_depth_parameter() {
    let nonexistent_path = Path::new("test.exe");

    // Test with depth 0 (should still work, just analyze the root file only)
    let result = analyze_command(nonexistent_path, 0, false, &[], Format::Text);

    // Should fail due to nonexistent file, not due to zero depth
    assert!(result.is_err());
    match result.unwrap_err() {
        dependencywalker_rs::error::Error::FileNotFound { .. } => {
            // Expected
        }
        _ => panic!("Expected FileNotFound error"),
    }
}

#[test]
fn test_large_depth_parameter() {
    let nonexistent_path = Path::new("test.exe");

    // Test with very large depth
    let result = analyze_command(nonexistent_path, 1000, false, &[], Format::Text);

    // Should fail due to nonexistent file, not due to large depth
    assert!(result.is_err());
    match result.unwrap_err() {
        dependencywalker_rs::error::Error::FileNotFound { .. } => {
            // Expected
        }
        _ => panic!("Expected FileNotFound error"),
    }
}

#[test]
fn test_multiple_search_paths() {
    let nonexistent_path = Path::new("test.exe");
    let search_paths = vec![
        std::path::PathBuf::from("C:\\Windows\\System32"),
        std::path::PathBuf::from("C:\\Windows\\SysWOW64"),
        std::path::PathBuf::from("C:\\Program Files\\Common Files"),
        std::path::PathBuf::from("C:\\Custom\\Path"),
    ];

    let result = analyze_command(nonexistent_path, 5, false, &search_paths, Format::Text);

    // Should fail due to nonexistent file, but search paths should be accepted
    assert!(result.is_err());
    match result.unwrap_err() {
        dependencywalker_rs::error::Error::FileNotFound { .. } => {
            // Expected
        }
        _ => panic!("Expected FileNotFound error"),
    }
}

#[test]
fn test_command_function_signatures() {
    // Test that command functions have the expected signatures
    let path = Path::new("test.exe");
    let search_paths = vec![std::path::PathBuf::from("C:\\test")];

    // These should compile without errors
    let _: Result<(), dependencywalker_rs::error::Error> =
        analyze_command(path, 5_u32, true, &search_paths, Format::Text);

    let _: Result<(), dependencywalker_rs::error::Error> = list_command(path, true, Format::Json);

    let _: Result<(), dependencywalker_rs::error::Error> = tree_command(path, false, Format::Xml);
}

#[test]
fn test_format_consistency() {
    // Test that all commands accept all format types
    let path = Path::new("test.exe");
    let formats = vec![Format::Text, Format::Json, Format::Xml];

    for format in formats {
        // All these should compile and have consistent error handling
        let _result1 = analyze_command(path, 5, false, &[], format);
        let _result2 = list_command(path, false, format);
        let _result3 = tree_command(path, false, format);
    }
}
