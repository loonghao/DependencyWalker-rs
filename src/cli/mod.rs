//! Command Line Interface module for DependencyWalker RS

/// Command implementations
pub mod commands {
    //! CLI command implementations

    use crate::error::Result;
    use crate::core::{DependencyAnalyzer, PEFileMap, PEFile};
    use super::output::{OutputData, DependencyInfo, Format};
    use std::path::Path;
    use std::time::Instant;

    /// Analyze command implementation
    pub fn analyze_command(
        file: &Path,
        depth: u32,
        include_system: bool,
        additional_paths: &[std::path::PathBuf],
        output_format: Format,
    ) -> Result<()> {
        log::info!("Analyzing file: {}", file.display());
        log::debug!("Max depth: {}", depth);
        log::debug!("Include system DLLs: {}", include_system);
        log::debug!("Additional paths: {:?}", additional_paths);

        let start_time = Instant::now();

        // Create and configure dependency analyzer
        let mut analyzer = DependencyAnalyzer::new();
        analyzer.set_max_depth(depth as usize);
        analyzer.set_include_system_dlls(include_system);

        // Add additional search paths
        for path in additional_paths {
            analyzer.add_search_path(path);
        }

        // Check if file exists first
        if !file.exists() {
            return Err(crate::error::Error::FileNotFound {
                path: file.to_path_buf()
            });
        }

        // Build dependency tree
        let tree = analyzer.build_tree(file)?;

        // Create output data
        let mut output_data = OutputData::new(file.display().to_string());

        // Convert tree to output format
        if let Some(root) = &tree.root {
            convert_node_to_output(root, &mut output_data);
        }

        // Add analysis statistics
        output_data.add_metadata("analysis_time_ms", tree.stats.analysis_time_ms.to_string());
        output_data.add_metadata("total_dependencies", tree.stats.total_dependencies.to_string());
        output_data.add_metadata("missing_count", tree.stats.missing_count.to_string());
        output_data.add_metadata("circular_count", tree.stats.circular_count.to_string());
        output_data.add_metadata("max_depth", tree.stats.max_depth.to_string());

        // Add missing dependencies as warnings
        for missing in tree.get_missing_dependencies() {
            output_data.warnings.push(format!("Missing dependency: {}", missing));
        }

        // Add circular dependencies as warnings
        for circular in tree.get_circular_dependencies() {
            let cycle_str = circular.iter()
                .map(|p| p.file_name().unwrap_or_default().to_string_lossy())
                .collect::<Vec<_>>()
                .join(" -> ");
            output_data.warnings.push(format!("Circular dependency: {}", cycle_str));
        }

        // Output results
        let formatted_output = output_data.format(output_format)?;
        println!("{}", formatted_output);

        let elapsed = start_time.elapsed();
        log::info!("Analysis completed in {:?}", elapsed);

        Ok(())
    }

    /// Tree command implementation
    pub fn tree_command(file: &Path, show_missing_only: bool, output_format: Format) -> Result<()> {
        log::info!("Displaying dependency tree for: {}", file.display());
        log::debug!("Show missing only: {}", show_missing_only);

        // Check if file exists first
        if !file.exists() {
            return Err(crate::error::Error::FileNotFound {
                path: file.to_path_buf()
            });
        }

        let mut analyzer = DependencyAnalyzer::new();
        analyzer.set_include_system_dlls(true);

        let tree = analyzer.build_tree(file)?;

        if let Some(root) = &tree.root {
            if output_format == Format::Text {
                print_tree_node(root, 0, show_missing_only);
            } else {
                // For JSON/XML, use the same output structure as analyze
                let mut output_data = OutputData::new(file.display().to_string());
                convert_node_to_output(root, &mut output_data);

                if show_missing_only {
                    output_data.dependencies.retain(|dep| !dep.found);
                }

                let formatted_output = output_data.format(output_format)?;
                println!("{}", formatted_output);
            }
        } else {
            println!("No dependency tree could be built for: {}", file.display());
        }

        Ok(())
    }

    /// List command implementation
    pub fn list_command(file: &Path, detailed: bool, output_format: Format) -> Result<()> {
        log::info!("Listing dependencies for: {}", file.display());
        log::debug!("Detailed mode: {}", detailed);

        // Check if file exists first
        if !file.exists() {
            return Err(crate::error::Error::FileNotFound {
                path: file.to_path_buf()
            });
        }

        // Parse the PE file directly to get immediate dependencies
        let pe_map = PEFileMap::new(file)?;
        let pe_file = PEFile::new(&pe_map)?;
        let dependencies = pe_file.get_dependencies()?;

        let mut output_data = OutputData::new(file.display().to_string());

        if detailed {
            // Use dependency analyzer for detailed analysis
            let mut analyzer = DependencyAnalyzer::new();
            analyzer.set_max_depth(1); // Only immediate dependencies
            analyzer.set_include_system_dlls(true);

            let tree = analyzer.build_tree(file)?;

            if let Some(root) = &tree.root {
                convert_node_to_output(root, &mut output_data);
            }
        } else {
            // Simple list of immediate dependencies
            for dep_name in dependencies {
                let dep_info = DependencyInfo {
                    name: dep_name,
                    path: None,
                    found: false, // We don't resolve paths in simple mode
                    symbols: Vec::new(),
                    architecture: None,
                    file_size: None,
                    version: None,
                };
                output_data.dependencies.push(dep_info);
            }
        }

        let formatted_output = output_data.format(output_format)?;
        println!("{}", formatted_output);

        Ok(())
    }

    /// Convert dependency tree node to output format
    fn convert_node_to_output(node: &crate::core::DependencyNode, output_data: &mut OutputData) {
        let dep_info = DependencyInfo {
            name: node.name.clone(),
            path: if node.found { Some(node.path.display().to_string()) } else { None },
            found: node.found,
            symbols: Vec::new(), // Will be populated in symbol analysis task
            architecture: node.is_64bit.map(|is_64| if is_64 { "x64".to_string() } else { "x86".to_string() }),
            file_size: None,
            version: None,
        };

        output_data.dependencies.push(dep_info);

        // Add errors from this node
        for error in &node.errors {
            output_data.errors.push(format!("{}: {}", node.name, error));
        }

        // Recursively process children
        for child in &node.children {
            convert_node_to_output(child, output_data);
        }
    }

    /// Print dependency tree in text format
    fn print_tree_node(node: &crate::core::DependencyNode, depth: usize, show_missing_only: bool) {
        if show_missing_only && node.found {
            return;
        }

        let indent = "  ".repeat(depth);
        let status = if node.found { "✓" } else { "✗" };
        let arch = node.is_64bit.map(|is_64| if is_64 { " (x64)" } else { " (x86)" }).unwrap_or("");

        println!("{}{} {}{}", indent, status, node.name, arch);

        if !node.errors.is_empty() {
            for error in &node.errors {
                println!("{}  ⚠️  {}", indent, error);
            }
        }

        for child in &node.children {
            print_tree_node(child, depth + 1, show_missing_only);
        }
    }
}

/// Output formatting utilities
pub mod output {
    //! Output formatting for different formats
    
    use serde::{Deserialize, Serialize};
    
    /// Output format enumeration
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Format {
        Text,
        Json,
        Xml,
    }
    
    /// Generic output data structure
    #[derive(Debug, Serialize, Deserialize)]
    pub struct OutputData {
        pub file_path: String,
        pub dependencies: Vec<DependencyInfo>,
        pub errors: Vec<String>,
        pub warnings: Vec<String>,
        pub metadata: std::collections::HashMap<String, String>,
    }
    
    /// Dependency information for output
    #[derive(Debug, Serialize, Deserialize)]
    pub struct DependencyInfo {
        pub name: String,
        pub path: Option<String>,
        pub found: bool,
        pub symbols: Vec<String>,
        pub architecture: Option<String>,
        pub file_size: Option<u64>,
        pub version: Option<String>,
    }
    
    impl OutputData {
        /// Create new output data
        pub fn new(file_path: String) -> Self {
            Self {
                file_path,
                dependencies: Vec::new(),
                errors: Vec::new(),
                warnings: Vec::new(),
                metadata: std::collections::HashMap::new(),
            }
        }

        /// Add metadata entry
        pub fn add_metadata(&mut self, key: &str, value: String) {
            self.metadata.insert(key.to_string(), value);
        }
        
        /// Format output according to specified format
        pub fn format(&self, format: Format) -> crate::error::Result<String> {
            match format {
                Format::Text => Ok(self.format_text()),
                Format::Json => {
                    serde_json::to_string_pretty(self)
                        .map_err(|e| crate::error::Error::generic(e.to_string()))
                }
                Format::Xml => self.format_xml(),
            }
        }
        
        /// Format as human-readable text
        fn format_text(&self) -> String {
            let mut output = String::new();
            output.push_str(&format!("Analysis for: {}\n", self.file_path));
            output.push_str(&format!("Dependencies found: {}\n", self.dependencies.len()));

            // Add metadata if available
            if !self.metadata.is_empty() {
                output.push_str("\nAnalysis Statistics:\n");
                if let Some(time) = self.metadata.get("analysis_time_ms") {
                    output.push_str(&format!("  Analysis time: {}ms\n", time));
                }
                if let Some(total) = self.metadata.get("total_dependencies") {
                    output.push_str(&format!("  Total dependencies: {}\n", total));
                }
                if let Some(missing) = self.metadata.get("missing_count") {
                    output.push_str(&format!("  Missing dependencies: {}\n", missing));
                }
                if let Some(circular) = self.metadata.get("circular_count") {
                    output.push_str(&format!("  Circular dependencies: {}\n", circular));
                }
                if let Some(depth) = self.metadata.get("max_depth") {
                    output.push_str(&format!("  Maximum depth: {}\n", depth));
                }
            }

            if !self.dependencies.is_empty() {
                output.push_str("\nDependencies:\n");
                for dep in &self.dependencies {
                    let status = if dep.found { "✓" } else { "✗" };
                    output.push_str(&format!("  {} {} ", status, dep.name));

                    if let Some(arch) = &dep.architecture {
                        output.push_str(&format!("({}) ", arch));
                    }

                    if let Some(path) = &dep.path {
                        output.push_str(&format!("-> {}", path));
                    }

                    if let Some(version) = &dep.version {
                        output.push_str(&format!(" [v{}]", version));
                    }

                    output.push('\n');
                }
            }

            if !self.errors.is_empty() {
                output.push_str("\nErrors:\n");
                for error in &self.errors {
                    output.push_str(&format!("  ❌ {}\n", error));
                }
            }

            if !self.warnings.is_empty() {
                output.push_str("\nWarnings:\n");
                for warning in &self.warnings {
                    output.push_str(&format!("  ⚠️  {}\n", warning));
                }
            }

            output
        }

        /// Format as XML
        fn format_xml(&self) -> crate::error::Result<String> {
            let mut xml = String::new();
            xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
            xml.push_str("<OutputData>\n");

            // File path
            xml.push_str(&format!("  <FilePath>{}</FilePath>\n", self.escape_xml(&self.file_path)));

            // Dependencies
            xml.push_str("  <Dependencies>\n");
            for dep in &self.dependencies {
                xml.push_str("    <Dependency>\n");
                xml.push_str(&format!("      <Name>{}</Name>\n", self.escape_xml(&dep.name)));
                xml.push_str(&format!("      <Found>{}</Found>\n", dep.found));

                if let Some(path) = &dep.path {
                    xml.push_str(&format!("      <Path>{}</Path>\n", self.escape_xml(path)));
                }

                if let Some(arch) = &dep.architecture {
                    xml.push_str(&format!("      <Architecture>{}</Architecture>\n", self.escape_xml(arch)));
                }

                if let Some(version) = &dep.version {
                    xml.push_str(&format!("      <Version>{}</Version>\n", self.escape_xml(version)));
                }

                if let Some(size) = dep.file_size {
                    xml.push_str(&format!("      <FileSize>{}</FileSize>\n", size));
                }

                if !dep.symbols.is_empty() {
                    xml.push_str("      <Symbols>\n");
                    for symbol in &dep.symbols {
                        xml.push_str(&format!("        <Symbol>{}</Symbol>\n", self.escape_xml(symbol)));
                    }
                    xml.push_str("      </Symbols>\n");
                }

                xml.push_str("    </Dependency>\n");
            }
            xml.push_str("  </Dependencies>\n");

            // Metadata
            if !self.metadata.is_empty() {
                xml.push_str("  <Metadata>\n");
                for (key, value) in &self.metadata {
                    xml.push_str(&format!("    <{}>{}</{}>\n",
                                        self.escape_xml(key),
                                        self.escape_xml(value),
                                        self.escape_xml(key)));
                }
                xml.push_str("  </Metadata>\n");
            }

            // Errors
            if !self.errors.is_empty() {
                xml.push_str("  <Errors>\n");
                for error in &self.errors {
                    xml.push_str(&format!("    <Error>{}</Error>\n", self.escape_xml(error)));
                }
                xml.push_str("  </Errors>\n");
            }

            // Warnings
            if !self.warnings.is_empty() {
                xml.push_str("  <Warnings>\n");
                for warning in &self.warnings {
                    xml.push_str(&format!("    <Warning>{}</Warning>\n", self.escape_xml(warning)));
                }
                xml.push_str("  </Warnings>\n");
            }

            xml.push_str("</OutputData>\n");
            Ok(xml)
        }

        /// Escape XML special characters
        fn escape_xml(&self, text: &str) -> String {
            text.replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
                .replace('"', "&quot;")
                .replace('\'', "&apos;")
        }
    }
}

/// Configuration management
pub mod config {
    //! Configuration file support for CLI

    use serde::{Deserialize, Serialize};
    use std::path::PathBuf;
    use crate::error::Result;

    /// CLI configuration structure
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CliConfig {
        /// Default output format
        pub default_format: String,
        /// Default maximum recursion depth
        pub default_depth: u32,
        /// Whether to include system DLLs by default
        pub include_system_dlls: bool,
        /// Default additional search paths
        pub search_paths: Vec<PathBuf>,
        /// Enable verbose output by default
        pub verbose: bool,
    }

    impl Default for CliConfig {
        fn default() -> Self {
            Self {
                default_format: "text".to_string(),
                default_depth: 10,
                include_system_dlls: false,
                search_paths: Vec::new(),
                verbose: false,
            }
        }
    }

    impl CliConfig {
        /// Load configuration from file
        pub fn load_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
            let content = std::fs::read_to_string(path)?;
            let config: CliConfig = toml::from_str(&content)
                .map_err(|e| crate::error::Error::generic(format!("Failed to parse config: {}", e)))?;
            Ok(config)
        }

        /// Save configuration to file
        pub fn save_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
            let content = toml::to_string_pretty(self)
                .map_err(|e| crate::error::Error::generic(format!("Failed to serialize config: {}", e)))?;
            std::fs::write(path, content)?;
            Ok(())
        }

        /// Get default config file path
        pub fn default_config_path() -> Result<PathBuf> {
            let mut path = dirs::config_dir()
                .ok_or_else(|| crate::error::Error::generic("Could not determine config directory"))?;
            path.push("dependencywalker_rs");
            std::fs::create_dir_all(&path)?;
            path.push("config.toml");
            Ok(path)
        }

        /// Load configuration with fallback to default
        pub fn load_or_default() -> Self {
            match Self::default_config_path() {
                Ok(path) if path.exists() => {
                    match Self::load_from_file(&path) {
                        Ok(config) => {
                            log::debug!("Loaded configuration from: {}", path.display());
                            config
                        }
                        Err(e) => {
                            log::warn!("Failed to load config from {}: {}", path.display(), e);
                            Self::default()
                        }
                    }
                }
                _ => Self::default(),
            }
        }
    }
}

/// Progress reporting utilities
pub mod progress {
    //! Progress reporting for long-running operations

    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    /// Simple progress reporter
    pub struct ProgressReporter {
        total: usize,
        current: Arc<AtomicUsize>,
        start_time: Instant,
        last_update: Instant,
    }

    impl ProgressReporter {
        /// Create a new progress reporter
        pub fn new(total: usize) -> Self {
            Self {
                total,
                current: Arc::new(AtomicUsize::new(0)),
                start_time: Instant::now(),
                last_update: Instant::now(),
            }
        }

        /// Update progress
        pub fn update(&mut self, current: usize) {
            self.current.store(current, Ordering::Relaxed);

            // Only update display every 100ms to avoid spam
            let now = Instant::now();
            if now.duration_since(self.last_update) > Duration::from_millis(100) {
                self.display();
                self.last_update = now;
            }
        }

        /// Increment progress by 1
        pub fn increment(&mut self) {
            let current = self.current.fetch_add(1, Ordering::Relaxed) + 1;
            if current % 10 == 0 || current == self.total {
                self.display();
            }
        }

        /// Display current progress
        fn display(&self) {
            let current = self.current.load(Ordering::Relaxed);
            let percentage = if self.total > 0 {
                (current as f64 / self.total as f64 * 100.0) as u32
            } else {
                0
            };

            let elapsed = self.start_time.elapsed();
            let rate = if elapsed.as_secs() > 0 {
                current as f64 / elapsed.as_secs_f64()
            } else {
                0.0
            };

            eprint!("\rProgress: {}/{} ({}%) - {:.1} items/sec",
                   current, self.total, percentage, rate);

            if current >= self.total {
                eprintln!(); // New line when complete
            }
        }

        /// Finish progress reporting
        pub fn finish(&self) {
            let current = self.current.load(Ordering::Relaxed);
            let elapsed = self.start_time.elapsed();
            eprintln!("\rCompleted: {}/{} in {:.2}s", current, self.total, elapsed.as_secs_f64());
        }
    }
}

// Re-export commonly used items
pub use commands::*;
pub use output::{Format as OutputFormat, OutputData, DependencyInfo};
pub use config::CliConfig;
pub use progress::ProgressReporter;
