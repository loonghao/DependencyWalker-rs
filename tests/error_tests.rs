//! Unit tests for error handling

use dependencywalker_rs::error::{Error, Result};
use std::path::PathBuf;

#[test]
fn test_file_not_found_error() {
    let path = PathBuf::from("nonexistent.exe");
    let error = Error::FileNotFound { path: path.clone() };

    match &error {
        Error::FileNotFound { path: error_path } => {
            assert_eq!(error_path, &path);
        }
        _ => panic!("Expected FileNotFound error"),
    }

    let error_string = format!("{}", error);
    assert!(error_string.contains("File not found"));
    assert!(error_string.contains("nonexistent.exe"));
}

#[test]
fn test_invalid_format_error() {
    let path = PathBuf::from("invalid.txt");
    let reason = "Not a valid PE file".to_string();
    let error = Error::InvalidFormat {
        path: path.clone(),
        reason: reason.clone(),
    };

    match &error {
        Error::InvalidFormat {
            path: error_path,
            reason: error_reason,
        } => {
            assert_eq!(error_path, &path);
            assert_eq!(error_reason, &reason);
        }
        _ => panic!("Expected InvalidFormat error"),
    }

    let error_string = format!("{}", error);
    assert!(error_string.contains("Invalid file format"));
    assert!(error_string.contains("invalid.txt"));
}

#[test]
fn test_pe_error() {
    let message = "Failed to parse import table";
    let error = Error::PeError(message.to_string());

    match &error {
        Error::PeError(error_message) => {
            assert_eq!(error_message, message);
        }
        _ => panic!("Expected PeError"),
    }

    let error_string = format!("{}", error);
    assert!(error_string.contains("PE parsing error"));
    assert!(error_string.contains(message));
}

#[test]
fn test_io_error() {
    let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied");
    let error = Error::Io(io_error);

    let error_string = format!("{}", error);
    assert!(error_string.contains("IO error"));
    assert!(error_string.contains("Access denied"));
}

#[test]
fn test_dependency_resolution_error() {
    let message = "missing.dll: DLL not found in search paths";
    let error = Error::DependencyResolution(message.to_string());

    match &error {
        Error::DependencyResolution(error_message) => {
            assert_eq!(error_message, message);
        }
        _ => panic!("Expected DependencyResolution error"),
    }

    let error_string = format!("{}", error);
    assert!(error_string.contains("Dependency resolution error"));
    assert!(error_string.contains("missing.dll"));
}

#[test]
fn test_circular_dependency_error() {
    let chain = "app.exe -> lib1.dll -> lib2.dll -> lib1.dll";
    let error = Error::CircularDependency {
        chain: chain.to_string(),
    };

    match &error {
        Error::CircularDependency { chain: error_chain } => {
            assert_eq!(error_chain, chain);
        }
        _ => panic!("Expected CircularDependency error"),
    }

    let error_string = format!("{}", error);
    assert!(error_string.contains("Circular dependency detected"));
    assert!(error_string.contains(chain));
}

#[test]
fn test_configuration_error() {
    let message = "Invalid configuration value";
    let error = Error::Configuration(message.to_string());

    match &error {
        Error::Configuration(error_message) => {
            assert_eq!(error_message, message);
        }
        _ => panic!("Expected Configuration error"),
    }

    let error_string = format!("{}", error);
    assert!(error_string.contains("Configuration error"));
    assert!(error_string.contains(message));
}

#[test]
fn test_error_debug_format() {
    let error = Error::FileNotFound {
        path: PathBuf::from("test.exe"),
    };

    let debug_string = format!("{:?}", error);
    assert!(debug_string.contains("FileNotFound"));
    assert!(debug_string.contains("test.exe"));
}

#[test]
fn test_error_from_io_error() {
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let error: Error = io_error.into();

    match error {
        Error::Io(_) => {
            // Expected
        }
        _ => panic!("Expected Io error"),
    }
}

#[test]
fn test_result_type_alias() {
    // Test that our Result type alias works correctly
    fn test_function() -> Result<String> {
        Ok("success".to_string())
    }

    let result = test_function();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "success");
}

#[test]
fn test_result_error_propagation() {
    fn failing_function() -> Result<()> {
        Err(Error::PeError("Test error".to_string()))
    }

    fn calling_function() -> Result<String> {
        failing_function()?;
        Ok("should not reach here".to_string())
    }

    let result = calling_function();
    assert!(result.is_err());

    match result.unwrap_err() {
        Error::PeError(message) => {
            assert_eq!(message, "Test error");
        }
        _ => panic!("Expected PeError"),
    }
}

#[test]
fn test_error_chain() {
    let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied");
    let error = Error::Io(io_error);

    // Test that we can access the source error
    let error_string = format!("{}", error);
    assert!(error_string.contains("IO error"));
}

#[test]
fn test_multiple_error_types() {
    let errors = vec![
        Error::FileNotFound {
            path: PathBuf::from("file1.exe"),
        },
        Error::InvalidFormat {
            path: PathBuf::from("file2.txt"),
            reason: "Not a PE file".to_string(),
        },
        Error::PeError("Parse failed".to_string()),
        Error::DependencyResolution("missing.dll: Not found".to_string()),
        Error::CircularDependency {
            chain: "app.exe -> lib.dll -> app.exe".to_string(),
        },
        Error::Configuration("Config error".to_string()),
    ];

    // Test that all error types can be created and formatted
    for error in errors {
        let error_string = format!("{}", error);
        assert!(!error_string.is_empty());

        let debug_string = format!("{:?}", error);
        assert!(!debug_string.is_empty());
    }
}

#[test]
fn test_error_equality() {
    let error1 = Error::PeError("Test".to_string());
    let error2 = Error::PeError("Test".to_string());
    let error3 = Error::PeError("Different".to_string());

    // Note: Error doesn't implement PartialEq, so we test by matching
    match (&error1, &error2) {
        (Error::PeError(msg1), Error::PeError(msg2)) => {
            assert_eq!(msg1, msg2);
        }
        _ => panic!("Both should be PeError"),
    }

    match (&error1, &error3) {
        (Error::PeError(msg1), Error::PeError(msg2)) => {
            assert_ne!(msg1, msg2);
        }
        _ => panic!("Both should be PeError"),
    }
}

#[test]
fn test_error_context() {
    // Test that errors provide useful context information
    let path = PathBuf::from("C:\\test\\missing.exe");
    let error = Error::FileNotFound { path: path.clone() };

    let error_message = format!("{}", error);
    assert!(error_message.contains("C:\\test\\missing.exe"));

    // Test with dependency resolution error
    let error =
        Error::DependencyResolution("important.dll: Required for application startup".to_string());

    let error_message = format!("{}", error);
    assert!(error_message.contains("important.dll"));
    assert!(error_message.contains("Required for application startup"));
}

#[test]
fn test_symbol_resolution_error() {
    let symbol = "CreateFileW";
    let dll = "kernel32.dll";
    let error = Error::SymbolResolution {
        symbol: symbol.to_string(),
        dll: dll.to_string(),
    };

    match &error {
        Error::SymbolResolution {
            symbol: error_symbol,
            dll: error_dll,
        } => {
            assert_eq!(error_symbol, symbol);
            assert_eq!(error_dll, dll);
        }
        _ => panic!("Expected SymbolResolution error"),
    }

    let error_string = format!("{}", error);
    assert!(error_string.contains("Symbol resolution error"));
    assert!(error_string.contains(symbol));
    assert!(error_string.contains(dll));
}

#[test]
fn test_generic_error() {
    let message = "Something went wrong";
    let error = Error::Generic {
        message: message.to_string(),
    };

    match &error {
        Error::Generic {
            message: error_message,
        } => {
            assert_eq!(error_message, message);
        }
        _ => panic!("Expected Generic error"),
    }

    let error_string = format!("{}", error);
    assert!(error_string.contains("Error"));
    assert!(error_string.contains(message));
}

#[test]
fn test_error_helper_functions() {
    // Test helper functions
    let pe_error = Error::pe_error("PE parsing failed");
    assert!(matches!(pe_error, Error::PeError(_)));

    let dep_error = Error::dependency_error("Dependency not found");
    assert!(matches!(dep_error, Error::DependencyResolution(_)));

    let symbol_error = Error::symbol_error("CreateFile", "kernel32.dll");
    assert!(matches!(symbol_error, Error::SymbolResolution { .. }));

    let generic_error = Error::generic("Generic error");
    assert!(matches!(generic_error, Error::Generic { .. }));
}

#[test]
fn test_error_recoverability() {
    // Test recoverable errors
    let recoverable_errors = vec![
        Error::PeError("Parse error".to_string()),
        Error::DependencyResolution("Dep error".to_string()),
        Error::SymbolResolution {
            symbol: "test".to_string(),
            dll: "test.dll".to_string(),
        },
    ];

    for error in recoverable_errors {
        assert!(error.is_recoverable());
    }

    // Test non-recoverable errors
    let non_recoverable_errors = vec![
        Error::FileNotFound {
            path: PathBuf::from("test.exe"),
        },
        Error::InvalidFormat {
            path: PathBuf::from("test.txt"),
            reason: "Not PE".to_string(),
        },
        Error::CircularDependency {
            chain: "test".to_string(),
        },
    ];

    for error in non_recoverable_errors {
        assert!(!error.is_recoverable());
    }
}
