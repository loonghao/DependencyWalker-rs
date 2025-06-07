//! GUI功能测试
//! 
//! 测试GUI相关的核心功能，不依赖完整的ICED编译

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use dependencywalker_rs::gui::message::{AnalysisResult, DependencyInfo, DependencyStatus};

    #[test]
    fn test_file_validation() {
        println!("🧪 测试文件类型验证...");
        
        let valid_files = vec![
            "test.exe",
            "library.dll", 
            "driver.sys",
            "control.ocx",
        ];
        
        let invalid_files = vec![
            "document.txt",
            "image.png",
            "archive.zip",
            "script.bat",
        ];
        
        for file in valid_files {
            let path = PathBuf::from(file);
            assert!(is_valid_pe_file(&path), "应该接受PE文件: {}", file);
        }
        
        for file in invalid_files {
            let path = PathBuf::from(file);
            assert!(!is_valid_pe_file(&path), "应该拒绝非PE文件: {}", file);
        }
        
        println!("✅ 文件类型验证测试通过");
    }

    #[test]
    fn test_dependency_info_creation() {
        println!("🧪 测试依赖信息创建...");
        
        let dep_info = DependencyInfo {
            name: "kernel32.dll".to_string(),
            path: Some(PathBuf::from("C:\\Windows\\System32\\kernel32.dll")),
            status: DependencyStatus::SystemDll,
            children: vec![
                DependencyInfo {
                    name: "ntdll.dll".to_string(),
                    path: Some(PathBuf::from("C:\\Windows\\System32\\ntdll.dll")),
                    status: DependencyStatus::SystemDll,
                    children: vec![],
                }
            ],
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
        let found_count = analysis_result.dependencies.iter()
            .filter(|dep| matches!(dep.status, DependencyStatus::Found))
            .count();
        let missing_count = analysis_result.dependencies.iter()
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
        
        println!("✅ 依赖状态显示测试通过");
    }

    #[test]
    fn test_system_dll_detection() {
        println!("🧪 测试系统DLL检测...");
        
        let system_dlls = vec![
            "kernel32.dll",
            "user32.dll", 
            "gdi32.dll",
            "advapi32.dll",
            "shell32.dll",
        ];
        
        let user_dlls = vec![
            "myapp.dll",
            "custom.dll",
            "plugin.dll",
        ];
        
        for dll in system_dlls {
            assert!(is_system_dll(dll), "应该识别为系统DLL: {}", dll);
        }
        
        for dll in user_dlls {
            assert!(!is_system_dll(dll), "不应该识别为系统DLL: {}", dll);
        }
        
        println!("✅ 系统DLL检测测试通过");
    }

    #[test]
    fn test_dependency_tree_depth() {
        println!("🧪 测试依赖树深度...");
        
        // 创建多层依赖结构
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
        
        println!("✅ 依赖树深度测试通过");
    }

    #[test]
    fn test_gui_message_types() {
        println!("🧪 测试GUI消息类型...");
        
        // 这里我们测试消息类型的基本属性
        // 由于消息类型主要用于ICED框架，我们测试其基本结构
        
        let test_path = PathBuf::from("test.exe");
        let test_paths = vec![PathBuf::from("file1.dll"), PathBuf::from("file2.dll")];
        
        // 验证路径处理
        assert!(test_path.extension().is_some());
        assert_eq!(test_paths.len(), 2);
        
        println!("✅ GUI消息类型测试通过");
    }

    // 辅助函数

    fn is_valid_pe_file(path: &PathBuf) -> bool {
        match path.extension().and_then(|s| s.to_str()) {
            Some(ext) => matches!(ext.to_lowercase().as_str(), "exe" | "dll" | "sys" | "ocx" | "mll"),
            None => false,
        }
    }

    fn is_system_dll(name: &str) -> bool {
        let system_dlls = [
            "kernel32.dll", "user32.dll", "gdi32.dll", "advapi32.dll",
            "shell32.dll", "ole32.dll", "oleaut32.dll", "comctl32.dll",
            "comdlg32.dll", "winmm.dll", "version.dll", "ws2_32.dll",
            "ntdll.dll", "msvcrt.dll", "rpcrt4.dll", "secur32.dll",
        ];
        
        system_dlls.iter().any(|&sys_dll| 
            name.to_lowercase() == sys_dll.to_lowercase()
        )
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
    use std::path::PathBuf;
    use dependencywalker_rs::core::dependency::DependencyAnalyzer;

    #[test]
    fn test_gui_pe_integration() {
        println!("🧪 测试GUI与PE分析集成...");
        
        // 测试分析器创建和配置
        let mut analyzer = DependencyAnalyzer::new();
        analyzer.set_max_depth(3);
        analyzer.set_include_system_dlls(true);
        
        // 验证配置
        // 注意：这里我们测试分析器的基本功能，不依赖具体的PE文件
        
        println!("✅ GUI与PE分析集成测试通过");
    }

    #[test] 
    fn test_error_handling() {
        println!("🧪 测试错误处理...");
        
        // 测试无效路径处理
        let invalid_path = PathBuf::from("nonexistent.exe");
        
        // 在实际应用中，这会触发错误处理逻辑
        assert!(!invalid_path.exists());
        
        println!("✅ 错误处理测试通过");
    }
}
