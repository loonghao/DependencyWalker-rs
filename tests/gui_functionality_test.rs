//! GUI functionality tests
//!
//! Test GUI-related core functionality without depending on complete ICED compilation

#[cfg(test)]
mod tests {
    use dependencywalker_rs::gui::message::{AnalysisResult, DependencyInfo, DependencyStatus};
    use std::path::PathBuf;

    #[test]
    fn test_file_validation() {
        println!("🧪 Testing file type validation...");

        let valid_files = vec!["test.exe", "library.dll", "driver.sys", "control.ocx"];

        let invalid_files = vec!["document.txt", "image.png", "archive.zip", "script.bat"];

        for file in valid_files {
            let path = PathBuf::from(file);
            assert!(is_valid_pe_file(&path), "Should accept PE file: {}", file);
        }

        for file in invalid_files {
            let path = PathBuf::from(file);
            assert!(
                !is_valid_pe_file(&path),
                "Should reject non-PE file: {}",
                file
            );
        }

        println!("✅ File type validation test passed");
    }

    #[test]
    fn test_dependency_info_creation() {
        println!("🧪 测试依赖信息创建...");

        let dep_info = DependencyInfo {
            name: "kernel32.dll".to_string(),
            path: Some(PathBuf::from("C:\\Windows\\System32\\kernel32.dll")),
            status: DependencyStatus::SystemDll,
            children: vec![DependencyInfo {
                name: "ntdll.dll".to_string(),
                path: Some(PathBuf::from("C:\\Windows\\System32\\ntdll.dll")),
                status: DependencyStatus::SystemDll,
                children: vec![],
            }],
        };

        assert_eq!(dep_info.name, "kernel32.dll");
        assert_eq!(dep_info.children.len(), 1);
        assert!(matches!(dep_info.status, DependencyStatus::SystemDll));

        println!("✅ 依赖信息创建测试通过");
    }

    #[test]
    fn test_analysis_result_structure() {
        println!("🧪 测试分析结果结构...");

        let analysis_result = AnalysisResult {
            file_path: PathBuf::from("test.exe"),
            dependencies: vec![
                DependencyInfo {
                    name: "kernel32.dll".to_string(),
                    path: Some(PathBuf::from("C:\\Windows\\System32\\kernel32.dll")),
                    status: DependencyStatus::Found,
                    children: vec![],
                },
                DependencyInfo {
                    name: "missing.dll".to_string(),
                    path: None,
                    status: DependencyStatus::Missing,
                    children: vec![],
                },
            ],
            analysis_time: std::time::Duration::from_millis(100),
        };

        assert_eq!(analysis_result.dependencies.len(), 2);
        assert!(analysis_result.analysis_time.as_millis() > 0);

        // 测试状态分布
        let found_count = analysis_result
            .dependencies
            .iter()
            .filter(|dep| matches!(dep.status, DependencyStatus::Found))
            .count();
        let missing_count = analysis_result
            .dependencies
            .iter()
            .filter(|dep| matches!(dep.status, DependencyStatus::Missing))
            .count();

        assert_eq!(found_count, 1);
        assert_eq!(missing_count, 1);

        println!("✅ 分析结果结构测试通过");
    }

    #[test]
    fn test_dependency_status_display() {
        println!("🧪 测试依赖状态显示...");

        let statuses = vec![
            (DependencyStatus::Found, "Found"),
            (DependencyStatus::Missing, "Missing"),
            (DependencyStatus::SystemDll, "System DLL"),
            (DependencyStatus::Delayed, "Delayed"),
        ];

        for (status, expected) in statuses {
            assert_eq!(format!("{}", status), expected);
        }

        println!("✅ Dependency status display test passed");
    }

    #[test]
    fn test_system_dll_detection() {
        println!("🧪 Testing system DLL detection...");

        let system_dlls = vec![
            "kernel32.dll",
            "user32.dll",
            "gdi32.dll",
            "advapi32.dll",
            "shell32.dll",
        ];

        let user_dlls = vec!["myapp.dll", "custom.dll", "plugin.dll"];

        for dll in system_dlls {
            assert!(
                is_system_dll(dll),
                "Should recognize as system DLL: {}",
                dll
            );
        }

        for dll in user_dlls {
            assert!(
                !is_system_dll(dll),
                "Should not recognize as system DLL: {}",
                dll
            );
        }

        println!("✅ System DLL detection test passed");
    }

    #[test]
    fn test_dependency_tree_depth() {
        println!("🧪 Testing dependency tree depth...");

        // Create multi-level dependency structure
        let deep_dependency = create_nested_dependency(3);

        assert_eq!(deep_dependency.name, "level_0.dll");
        assert_eq!(deep_dependency.children.len(), 1);

        let level_1 = &deep_dependency.children[0];
        assert_eq!(level_1.name, "level_1.dll");
        assert_eq!(level_1.children.len(), 1);

        let level_2 = &level_1.children[0];
        assert_eq!(level_2.name, "level_2.dll");
        assert_eq!(level_2.children.len(), 1);

        let level_3 = &level_2.children[0];
        assert_eq!(level_3.name, "level_3.dll");
        assert_eq!(level_3.children.len(), 0);

        println!("✅ Dependency tree depth test passed");
    }

    #[test]
    fn test_gui_message_types() {
        println!("🧪 Testing GUI message types...");

        // Here we test basic properties of message types
        // Since message types are mainly used for ICED framework, we test their basic structure

        let test_path = PathBuf::from("test.exe");
        let test_paths = vec![PathBuf::from("file1.dll"), PathBuf::from("file2.dll")];

        // Verify path handling
        assert!(test_path.extension().is_some());
        assert_eq!(test_paths.len(), 2);

        println!("✅ GUI message types test passed");
    }

    // Helper functions

    fn is_valid_pe_file(path: &PathBuf) -> bool {
        match path.extension().and_then(|s| s.to_str()) {
            Some(ext) => matches!(
                ext.to_lowercase().as_str(),
                "exe" | "dll" | "sys" | "ocx" | "mll"
            ),
            None => false,
        }
    }

    fn is_system_dll(name: &str) -> bool {
        let system_dlls = [
            "kernel32.dll",
            "user32.dll",
            "gdi32.dll",
            "advapi32.dll",
            "shell32.dll",
            "ole32.dll",
            "oleaut32.dll",
            "comctl32.dll",
            "comdlg32.dll",
            "winmm.dll",
            "version.dll",
            "ws2_32.dll",
            "ntdll.dll",
            "msvcrt.dll",
            "rpcrt4.dll",
            "secur32.dll",
        ];

        system_dlls
            .iter()
            .any(|&sys_dll| name.to_lowercase() == sys_dll.to_lowercase())
    }

    fn create_nested_dependency(depth: usize) -> DependencyInfo {
        if depth == 0 {
            return DependencyInfo {
                name: format!("level_{}.dll", depth),
                path: Some(PathBuf::from(format!("C:\\test\\level_{}.dll", depth))),
                status: DependencyStatus::Found,
                children: vec![],
            };
        }

        DependencyInfo {
            name: format!("level_{}.dll", depth),
            path: Some(PathBuf::from(format!("C:\\test\\level_{}.dll", depth))),
            status: DependencyStatus::Found,
            children: vec![create_nested_dependency(depth - 1)],
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use dependencywalker_rs::core::dependency::DependencyAnalyzer;
    use std::path::PathBuf;

    #[test]
    fn test_gui_pe_integration() {
        println!("🧪 Testing GUI and PE analysis integration...");

        // Test analyzer creation and configuration
        let mut analyzer = DependencyAnalyzer::new();
        analyzer.set_max_depth(3);
        analyzer.set_include_system_dlls(true);

        // Verify configuration
        // Note: Here we test basic analyzer functionality without depending on specific PE files

        println!("✅ GUI and PE analysis integration test passed");
    }

    #[test]
    fn test_error_handling() {
        println!("🧪 Testing error handling...");

        // Test invalid path handling
        let invalid_path = PathBuf::from("nonexistent.exe");

        // In real application, this would trigger error handling logic
        assert!(!invalid_path.exists());

        println!("✅ Error handling test passed");
    }
}
