//! Drag & Drop Test Interface - Simulate GUI drag & drop functionality

use std::env;
use std::io::{self, Write};
use std::path::Path;

fn main() {
    println!("🎯 DependencyWalker RS - Drag & Drop Test Interface");
    println!("==================================================");
    println!("✨ Dependencies updated! Rust 1.87 + latest dependency versions");
    println!();

    let args: Vec<String> = env::args().collect();

    if args.len() >= 2 {
        // If command line arguments provided, analyze directly
        let file_path = &args[1];
        analyze_file(file_path);
    } else {
        // Interactive mode
        interactive_mode();
    }
}

fn interactive_mode() {
    println!("🖱️  Drag & Drop Test Mode");
    println!("Please drag DLL/EXE files to this window, then press Enter");
    println!("Or directly input file path:");
    println!();

    loop {
        print!("📁 Please enter file path (or type 'quit' to exit): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let input = input.trim();

                if input.is_empty() {
                    continue;
                }

                if input.to_lowercase() == "quit" || input.to_lowercase() == "exit" {
                    println!("👋 Goodbye!");
                    break;
                }

                // Handle dragged file path (may contain quotes)
                let file_path = input.trim_matches('"').trim_matches('\'');
                analyze_file(file_path);

                println!("\n{}", "=".repeat(50));
                println!("Continue testing other files, or type 'quit' to exit");
            }
            Err(error) => {
                println!("❌ Error reading input: {}", error);
                break;
            }
        }
    }
}

fn analyze_file(file_path: &str) {
    println!("\n🔍 Analyzing file: {}", file_path);
    println!("{}", "-".repeat(50));

    let path = Path::new(file_path);

    if !path.exists() {
        println!("❌ Error: File does not exist");
        println!("💡 Tip: Please check if the path is correct");
        return;
    }

    if !path.is_file() {
        println!("❌ Error: This is not a file");
        return;
    }

    // Display basic information
    println!("✅ File exists and is accessible");

    if let Some(file_name) = path.file_name() {
        println!("📄 File name: {}", file_name.to_string_lossy());
    }

    if let Some(parent) = path.parent() {
        println!("📂 Directory: {}", parent.display());
    }

    // File size and time
    if let Ok(metadata) = path.metadata() {
        let size = metadata.len();
        println!(
            "📊 File size: {} bytes ({:.2} KB)",
            size,
            size as f64 / 1024.0
        );

        if let Ok(modified) = metadata.modified() {
            println!("📅 Modified time: {:?}", modified);
        }
    }

    // File type detection
    if let Some(extension) = path.extension() {
        let ext_lower = extension.to_string_lossy().to_lowercase();
        println!("📋 File extension: .{}", ext_lower);

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

    println!("\n🎯 Simulated analysis results:");
    println!("  🔍 PE file header: Parsed");
    println!("  📋 Import table: Analyzed");
    println!("  📤 Export table: Analyzed");
    println!("  🌳 Dependency tree: Built");
    println!("  ⚠️  Missing dependencies: Detected");

    println!("\n✨ Drag & drop functionality test successful!");
    println!("💡 In the real GUI version, you will see:");
    println!("   • Visual dependency tree");
    println!("   • Detailed PE file information");
    println!("   • Interactive dependency browsing");
    println!("   • Export functionality");
}
