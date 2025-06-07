//! 拖拽测试界面 - 模拟GUI拖拽功能

use std::env;
use std::path::Path;
use std::io::{self, Write};

fn main() {
    println!("🎯 DependencyWalker RS - 拖拽测试界面");
    println!("=====================================");
    println!("✨ 依赖更新完成！Rust 1.87 + 最新依赖版本");
    println!();
    
    let args: Vec<String> = env::args().collect();
    
    if args.len() >= 2 {
        // 如果有命令行参数，直接分析
        let file_path = &args[1];
        analyze_file(file_path);
    } else {
        // 交互式模式
        interactive_mode();
    }
}

fn interactive_mode() {
    println!("🖱️  拖拽测试模式");
    println!("请将DLL/EXE文件拖拽到此窗口，然后按Enter");
    println!("或者直接输入文件路径:");
    println!();
    
    loop {
        print!("📁 请输入文件路径 (或输入 'quit' 退出): ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let input = input.trim();
                
                if input.is_empty() {
                    continue;
                }
                
                if input.to_lowercase() == "quit" || input.to_lowercase() == "exit" {
                    println!("👋 再见！");
                    break;
                }
                
                // 处理拖拽的文件路径（可能包含引号）
                let file_path = input.trim_matches('"').trim_matches('\'');
                analyze_file(file_path);
                
                println!("\n{}", "=".repeat(50));
                println!("继续测试其他文件，或输入 'quit' 退出");
            }
            Err(error) => {
                println!("❌ 读取输入时出错: {}", error);
                break;
            }
        }
    }
}

fn analyze_file(file_path: &str) {
    println!("\n🔍 分析文件: {}", file_path);
    println!("{}", "-".repeat(50));
    
    let path = Path::new(file_path);
    
    if !path.exists() {
        println!("❌ 错误: 文件不存在");
        println!("💡 提示: 请检查路径是否正确");
        return;
    }
    
    if !path.is_file() {
        println!("❌ 错误: 这不是一个文件");
        return;
    }
    
    // 显示基本信息
    println!("✅ 文件存在且可访问");
    
    if let Some(file_name) = path.file_name() {
        println!("📄 文件名: {}", file_name.to_string_lossy());
    }
    
    if let Some(parent) = path.parent() {
        println!("📂 所在目录: {}", parent.display());
    }
    
    // 文件大小和时间
    if let Ok(metadata) = path.metadata() {
        let size = metadata.len();
        println!("📊 文件大小: {} bytes ({:.2} KB)", size, size as f64 / 1024.0);
        
        if let Ok(modified) = metadata.modified() {
            println!("📅 修改时间: {:?}", modified);
        }
    }
    
    // 文件类型检测
    if let Some(extension) = path.extension() {
        let ext_lower = extension.to_string_lossy().to_lowercase();
        println!("📋 文件扩展名: .{}", ext_lower);
        
        match ext_lower.as_str() {
            "dll" => {
                println!("🔧 文件类型: 动态链接库 (Dynamic Link Library)");
                println!("✅ 支持完整的依赖分析");
            }
            "exe" => {
                println!("⚙️  文件类型: 可执行文件 (Executable)");
                println!("✅ 支持完整的依赖分析");
            }
            "sys" => {
                println!("🖥️  文件类型: 系统驱动文件 (System Driver)");
                println!("✅ 支持完整的依赖分析");
            }
            "ocx" => {
                println!("🎛️  文件类型: ActiveX控件 (OCX Control)");
                println!("✅ 支持完整的依赖分析");
            }
            "mll" => {
                println!("🎭 文件类型: Maya插件库 (Maya Plugin Library)");
                println!("✅ 支持完整的依赖分析 (Maya专用格式)");
                println!("🎨 Maya插件特殊功能:");
                println!("   • Maya API依赖检测");
                println!("   • 插件兼容性分析");
                println!("   • Maya版本要求检查");
            }
            _ => {
                println!("❓ 文件类型: 未知或不支持");
                println!("⚠️  可能不是PE格式文件");
            }
        }
    }
    
    println!("\n🎯 模拟分析结果:");
    println!("  🔍 PE文件头: 已解析");
    println!("  📋 导入表: 已分析");
    println!("  📤 导出表: 已分析");
    println!("  🌳 依赖树: 已构建");
    println!("  ⚠️  缺失依赖: 已检测");
    
    println!("\n✨ 拖拽功能测试成功！");
    println!("💡 在真实的GUI版本中，您将看到:");
    println!("   • 可视化的依赖树");
    println!("   • 详细的PE文件信息");
    println!("   • 交互式的依赖浏览");
    println!("   • 导出功能");
}
