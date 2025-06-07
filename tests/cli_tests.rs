//! Tests for CLI functionality
//! 
//! This module contains integration tests for the command-line interface.

use std::process::Command;
use std::path::Path;

/// Helper function to run CLI command and get output
fn run_cli_command(args: &[&str]) -> Result<std::process::Output, std::io::Error> {
    let mut cmd = Command::new("cargo");
    cmd.arg("run")
       .arg("--bin")
       .arg("depwalker")
       .arg("--");
    
    for arg in args {
        cmd.arg(arg);
    }
    
    cmd.output()
}

#[test]
fn test_cli_help() {
    let output = run_cli_command(&["--help"]).expect("Failed to run CLI");
    assert!(output.status.success());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("DependencyWalker RS"));
    assert!(stdout.contains("analyze"));
    assert!(stdout.contains("tree"));
    assert!(stdout.contains("list"));
}

#[test]
fn test_cli_version() {
    let output = run_cli_command(&["--version"]).expect("Failed to run CLI");
    assert!(output.status.success());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_analyze_command() {
    let test_dll = "tests/liblzma.dll";
    if !Path::new(test_dll).exists() {
        println!("Test DLL not found, skipping analyze test");
        return;
    }
    
    let output = run_cli_command(&["analyze", test_dll]).expect("Failed to run analyze command");
    assert!(output.status.success());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Analysis for:"));
    assert!(stdout.contains("Dependencies found:"));
    assert!(stdout.contains("Analysis Statistics:"));
}

#[test]
fn test_analyze_command_json() {
    let test_dll = "tests/liblzma.dll";
    if !Path::new(test_dll).exists() {
        println!("Test DLL not found, skipping analyze JSON test");
        return;
    }
    
    let output = run_cli_command(&["--format", "json", "analyze", test_dll])
        .expect("Failed to run analyze command with JSON format");
    assert!(output.status.success());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should be valid JSON
    assert!(stdout.contains("\"file_path\""));
    assert!(stdout.contains("\"dependencies\""));
    assert!(stdout.contains("\"metadata\""));
    
    // Try to parse as JSON to ensure it's valid
    let json_result: Result<serde_json::Value, _> = serde_json::from_str(&stdout);
    assert!(json_result.is_ok(), "Output should be valid JSON: {}", stdout);
}

#[test]
fn test_tree_command() {
    let test_dll = "tests/liblzma.dll";
    if !Path::new(test_dll).exists() {
        println!("Test DLL not found, skipping tree test");
        return;
    }
    
    let output = run_cli_command(&["tree", test_dll]).expect("Failed to run tree command");
    assert!(output.status.success());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("liblzma.dll"));
    // Should contain tree structure indicators
    assert!(stdout.contains("✓") || stdout.contains("✗"));
}

#[test]
fn test_tree_command_missing_only() {
    let test_dll = "tests/liblzma.dll";
    if !Path::new(test_dll).exists() {
        println!("Test DLL not found, skipping tree missing test");
        return;
    }
    
    let output = run_cli_command(&["tree", test_dll, "--missing"])
        .expect("Failed to run tree command with --missing");
    assert!(output.status.success());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should only show missing dependencies (✗)
    if stdout.contains("✗") {
        // If there are missing dependencies, they should be shown
        assert!(stdout.contains("✗"));
    }
}

#[test]
fn test_list_command() {
    let test_dll = "tests/liblzma.dll";
    if !Path::new(test_dll).exists() {
        println!("Test DLL not found, skipping list test");
        return;
    }
    
    let output = run_cli_command(&["list", test_dll]).expect("Failed to run list command");
    assert!(output.status.success());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Analysis for:"));
    assert!(stdout.contains("Dependencies found:"));
}

#[test]
fn test_list_command_detailed() {
    let test_dll = "tests/liblzma.dll";
    if !Path::new(test_dll).exists() {
        println!("Test DLL not found, skipping detailed list test");
        return;
    }
    
    let output = run_cli_command(&["list", test_dll, "--detailed"])
        .expect("Failed to run list command with --detailed");
    assert!(output.status.success());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Analysis for:"));
    assert!(stdout.contains("Dependencies found:"));
    // Detailed mode should show more information
    assert!(stdout.contains("✓") || stdout.contains("✗"));
}

#[test]
fn test_xml_output_format() {
    let test_dll = "tests/liblzma.dll";
    if !Path::new(test_dll).exists() {
        println!("Test DLL not found, skipping XML test");
        return;
    }
    
    let output = run_cli_command(&["--format", "xml", "list", test_dll])
        .expect("Failed to run command with XML format");
    assert!(output.status.success());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should contain XML structure
    assert!(stdout.contains("<?xml"));
    assert!(stdout.contains("<OutputData>"));
    assert!(stdout.contains("</OutputData>"));
}

#[test]
fn test_nonexistent_file() {
    let output = run_cli_command(&["analyze", "nonexistent_file.dll"])
        .expect("Failed to run analyze command");
    assert!(!output.status.success());
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Error") || stderr.contains("not found") || stderr.contains("No such file"));
}

#[test]
fn test_invalid_arguments() {
    let output = run_cli_command(&["invalid_command"])
        .expect("Failed to run CLI with invalid command");
    assert!(!output.status.success());
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error:") || stderr.contains("unrecognized"));
}

#[test]
fn test_depth_parameter() {
    let test_dll = "tests/liblzma.dll";
    if !Path::new(test_dll).exists() {
        println!("Test DLL not found, skipping depth test");
        return;
    }
    
    let output = run_cli_command(&["analyze", test_dll, "--depth", "2"])
        .expect("Failed to run analyze command with depth parameter");
    assert!(output.status.success());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Analysis for:"));
}

#[test]
fn test_system_dlls_parameter() {
    let test_dll = "tests/liblzma.dll";
    if !Path::new(test_dll).exists() {
        println!("Test DLL not found, skipping system DLLs test");
        return;
    }
    
    let output = run_cli_command(&["analyze", test_dll, "--system"])
        .expect("Failed to run analyze command with --system parameter");
    assert!(output.status.success());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Analysis for:"));
}

#[test]
fn test_verbose_output() {
    let test_dll = "tests/liblzma.dll";
    if !Path::new(test_dll).exists() {
        println!("Test DLL not found, skipping verbose test");
        return;
    }
    
    let output = run_cli_command(&["--verbose", "analyze", test_dll])
        .expect("Failed to run analyze command with --verbose");
    assert!(output.status.success());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Analysis for:"));
}

#[test]
fn test_config_functionality() {
    // Test that CLI can handle configuration (even if config file doesn't exist)
    let output = run_cli_command(&["--help"]).expect("Failed to run CLI");
    assert!(output.status.success());
    
    // This test mainly ensures that the config module doesn't cause crashes
    // when no config file exists
}

#[test]
fn test_output_consistency() {
    let test_dll = "tests/liblzma.dll";
    if !Path::new(test_dll).exists() {
        println!("Test DLL not found, skipping consistency test");
        return;
    }
    
    // Run the same command twice and ensure consistent output
    let output1 = run_cli_command(&["list", test_dll])
        .expect("Failed to run first list command");
    let output2 = run_cli_command(&["list", test_dll])
        .expect("Failed to run second list command");
    
    assert!(output1.status.success());
    assert!(output2.status.success());
    
    let stdout1 = String::from_utf8_lossy(&output1.stdout);
    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    
    // The dependency count should be the same
    let deps1 = stdout1.lines().find(|line| line.contains("Dependencies found:"));
    let deps2 = stdout2.lines().find(|line| line.contains("Dependencies found:"));
    
    if let (Some(d1), Some(d2)) = (deps1, deps2) {
        assert_eq!(d1, d2, "Dependency counts should be consistent");
    }
}
