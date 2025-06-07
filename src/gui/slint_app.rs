//! Slint GUI application for DependencyWalker RS
//!
//! This module provides the Slint-based graphical user interface.

use crate::core::dependency::DependencyAnalyzer;
use crate::core::pe_parser::{PEFile, PEFileMap, PEInfo};
use crate::error::Error;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

// Include the generated Slint code
slint::include_modules!();

/// Slint application wrapper
#[allow(dead_code)]
pub struct SlintApp {
    /// The main Slint window
    window: MainWindow,
    /// Current dependency analyzer
    analyzer: DependencyAnalyzer,
    /// Current analysis state
    current_file: Option<PathBuf>,
    /// Analysis results cache
    analysis_cache: HashMap<PathBuf, AnalysisData>,
}

/// Internal analysis data structure
#[derive(Debug, Clone)]
struct AnalysisData {
    pub pe_info: PEInfo,
    pub dependency_tree: Vec<DependencyItemData>,
    pub selected_module: Option<ModuleInfoData>,
    pub import_functions: Vec<FunctionInfoData>,
    pub export_functions: Vec<FunctionInfoData>,
}

/// Dependency item data for Slint
#[derive(Debug, Clone)]
struct DependencyItemData {
    pub name: String,
    pub path: String,
    pub status: DependencyStatusData,
    pub children: Vec<DependencyItemData>,
    pub is_expanded: bool,
}

/// Dependency status for Slint
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
enum DependencyStatusData {
    Found,
    Missing,
    SystemDll,
    Delayed,
}

/// Module information for Slint
#[derive(Debug, Clone)]
struct ModuleInfoData {
    pub name: String,
    pub path: String,
    pub file_size: u64,
    pub architecture: String,
    pub pe_type: String,
    pub entry_point: String,
    pub image_base: String,
    pub subsystem: String,
    pub version_info: HashMap<String, String>,
}

/// Function information for Slint
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct FunctionInfoData {
    pub name: String,
    pub ordinal: String,
    pub address: String,
    pub hint: String,
    pub is_forwarded: bool,
    pub forward_name: String,
}

impl SlintApp {
    /// Create a new Slint application
    pub fn new() -> Result<Self, Error> {
        println!("Creating Slint application...");
        log::info!("Creating Slint window...");

        println!("About to call MainWindow::new()...");
        let window = match MainWindow::new() {
            Ok(w) => {
                println!("MainWindow::new() succeeded!");
                w
            }
            Err(e) => {
                println!("MainWindow::new() failed: {}", e);
                return Err(Error::generic(format!(
                    "Failed to create Slint window: {}",
                    e
                )));
            }
        };

        println!("Slint window created successfully");
        log::info!("Slint window created successfully");

        let app = Self {
            window,
            analyzer: DependencyAnalyzer::new(),
            current_file: None,
            analysis_cache: HashMap::new(),
        };

        // Set up callbacks
        println!("Setting up callbacks...");
        log::info!("Setting up callbacks...");
        app.setup_callbacks();

        // Initialize UI state
        println!("Initializing UI state...");
        log::info!("Initializing UI state...");
        app.initialize_ui();

        println!("Slint application created successfully");
        log::info!("Slint application created successfully");
        Ok(app)
    }

    /// Run the Slint application
    pub fn run(self) -> Result<(), Error> {
        println!("Starting Slint application event loop...");
        log::info!("Starting Slint application event loop...");

        println!("About to call window.run()...");
        match self.window.run() {
            Ok(()) => {
                println!("Window.run() completed successfully");
                Ok(())
            }
            Err(e) => {
                println!("Window.run() failed: {}", e);
                Err(Error::generic(format!(
                    "Failed to run Slint application: {}",
                    e
                )))
            }
        }
    }

    /// Set up all callback functions
    fn setup_callbacks(&self) {
        let window_weak = self.window.as_weak();

        // File selection callback
        self.window.on_file_selected({
            let window_weak = window_weak.clone();
            move |file_path| {
                if let Some(window) = window_weak.upgrade() {
                    Self::handle_file_selected(window, file_path.to_string());
                }
            }
        });

        // Browse files callback
        self.window.on_browse_files({
            let window_weak = window_weak.clone();
            move || {
                if let Some(window) = window_weak.upgrade() {
                    Self::handle_browse_files(window);
                }
            }
        });

        // Dependency selection callback
        self.window.on_dependency_selected({
            let window_weak = window_weak.clone();
            move |dependency_name| {
                if let Some(window) = window_weak.upgrade() {
                    Self::handle_dependency_selected(window, dependency_name.to_string());
                }
            }
        });

        // Dependency expansion callback
        self.window.on_dependency_expanded({
            let window_weak = window_weak.clone();
            move |dependency_name| {
                if let Some(window) = window_weak.upgrade() {
                    Self::handle_dependency_expanded(window, dependency_name.to_string());
                }
            }
        });

        // Function selection callback
        self.window.on_function_selected({
            let window_weak = window_weak.clone();
            move |function_name| {
                if let Some(window) = window_weak.upgrade() {
                    Self::handle_function_selected(window, function_name.to_string());
                }
            }
        });

        // Refresh analysis callback
        self.window.on_refresh_analysis({
            let window_weak = window_weak.clone();
            move || {
                if let Some(window) = window_weak.upgrade() {
                    Self::handle_refresh_analysis(window);
                }
            }
        });

        // Expand all callback
        self.window.on_expand_all({
            let window_weak = window_weak.clone();
            move || {
                if let Some(window) = window_weak.upgrade() {
                    Self::handle_expand_all(window);
                }
            }
        });

        // Collapse all callback
        self.window.on_collapse_all({
            let window_weak = window_weak.clone();
            move || {
                if let Some(window) = window_weak.upgrade() {
                    Self::handle_collapse_all(window);
                }
            }
        });

        // Toggle theme callback
        self.window.on_toggle_theme({
            let window_weak = window_weak.clone();
            move || {
                if let Some(window) = window_weak.upgrade() {
                    Self::handle_toggle_theme(window);
                }
            }
        });

        // Set up drag and drop support using winit events
        self.setup_drag_drop_support();
    }

    /// Set up drag and drop support using winit window events
    fn setup_drag_drop_support(&self) {
        #[cfg(feature = "gui")]
        {
            use i_slint_backend_winit::WinitWindowAccessor;

            // Get the underlying Slint window
            let slint_window = self.window.window();

            // Check if we have winit backend support
            if !slint_window.has_winit_window() {
                log::warn!("Winit backend not available, drag and drop will not work");
                log::info!("Please use the 'Browse Files' button to select files");
                return;
            }

            let window_weak = self.window.as_weak();

            // Set up winit window event handler for drag and drop
            slint_window.on_winit_window_event({
                move |_slint_window, winit_event| {
                    match winit_event {
                        winit::event::WindowEvent::DroppedFile(path) => {
                            log::info!("File dropped: {:?}", path);

                            if let Some(window) = window_weak.upgrade() {
                                // Reset drag state
                                window.set_is_dragging(false);

                                // Handle the dropped file
                                let file_path = path.to_string_lossy().to_string();
                                Self::handle_file_selected(window, file_path);
                            }

                            i_slint_backend_winit::WinitWindowEventResult::PreventDefault
                        }
                        winit::event::WindowEvent::HoveredFile(_path) => {
                            log::debug!("File hovered over window");

                            if let Some(window) = window_weak.upgrade() {
                                window.set_is_dragging(true);
                            }

                            i_slint_backend_winit::WinitWindowEventResult::PreventDefault
                        }
                        winit::event::WindowEvent::HoveredFileCancelled => {
                            log::debug!("File hover cancelled");

                            if let Some(window) = window_weak.upgrade() {
                                window.set_is_dragging(false);
                            }

                            i_slint_backend_winit::WinitWindowEventResult::PreventDefault
                        }
                        _ => i_slint_backend_winit::WinitWindowEventResult::Propagate,
                    }
                }
            });

            log::info!("Drag and drop support enabled via winit backend");
        }

        #[cfg(not(feature = "gui"))]
        {
            log::warn!("GUI feature not enabled, drag and drop not available");
        }
    }

    /// Initialize the UI state
    fn initialize_ui(&self) {
        // Set initial state to Welcome
        self.window.set_current_state(AppState::Welcome);
        self.window.set_current_file("".into());
        self.window.set_error_message("".into());
        self.window.set_is_loading(false);
        self.window.set_file_dialog_open(false);
        self.window.set_is_dragging(false);

        // Initialize empty data
        self.window
            .set_dependencies(Rc::new(slint::VecModel::default()).into());
        self.window.set_selected_dependency("".into());
        self.window
            .set_import_functions(Rc::new(slint::VecModel::default()).into());
        self.window
            .set_export_functions(Rc::new(slint::VecModel::default()).into());
        self.window.set_selected_function("".into());

        // Initialize UI state
        self.window.set_show_imports(true);
        self.window.set_show_exports(true);
    }

    /// Handle file selection
    fn handle_file_selected(window: MainWindow, file_path: String) {
        log::info!("File selected: {}", file_path);

        let path = PathBuf::from(file_path.clone());
        if !path.exists() {
            window.set_error_message(format!("File not found: {}", file_path).into());
            return;
        }

        // Validate file extension
        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            if !["exe", "dll", "sys", "ocx", "mll"].contains(&ext.as_str()) {
                window.set_error_message(format!("Unsupported file type: .{}", ext).into());
                return;
            }
        } else {
            window.set_error_message("File has no extension".into());
            return;
        }

        // Update UI state
        window.set_current_file(file_path.into());
        window.set_current_state(AppState::Analysis);
        window.set_is_loading(true);
        window.set_error_message("".into());

        // Start analysis in a separate thread
        let window_weak = window.as_weak();
        let path_clone = path.clone();

        std::thread::spawn(move || {
            let result = Self::analyze_file(&path_clone);

            // Update UI on main thread
            slint::invoke_from_event_loop(move || {
                if let Some(window) = window_weak.upgrade() {
                    window.set_is_loading(false);

                    match result {
                        Ok(analysis_data) => {
                            Self::update_ui_with_analysis(window, analysis_data);
                        }
                        Err(e) => {
                            window.set_error_message(format!("Analysis failed: {}", e).into());
                            window.set_current_state(AppState::Welcome);
                        }
                    }
                }
            })
            .unwrap_or_else(|e| {
                log::error!("Failed to invoke from event loop: {:?}", e);
            });
        });
    }

    /// Handle browse files action
    fn handle_browse_files(window: MainWindow) {
        println!("Browse files requested - button clicked!");
        log::info!("Browse files requested");

        // Use rfd for file dialog
        let window_weak = window.as_weak();

        std::thread::spawn(move || {
            let file_dialog = rfd::FileDialog::new()
                .add_filter("PE Files", &["exe", "dll", "sys", "ocx", "mll"])
                .add_filter("Executable Files", &["exe"])
                .add_filter("Library Files", &["dll", "mll"])
                .add_filter("System Files", &["sys"])
                .add_filter("Control Files", &["ocx"])
                .add_filter("Maya Plugins", &["mll"])
                .add_filter("All Files", &["*"])
                .set_title("Select PE file or Maya plugin to analyze");

            let selected_file = file_dialog.pick_file();

            slint::invoke_from_event_loop(move || {
                if let Some(window) = window_weak.upgrade() {
                    // Reset dialog state first
                    window.set_file_dialog_open(false);

                    if let Some(path) = selected_file {
                        let file_path = path.to_string_lossy().to_string();
                        Self::handle_file_selected(window, file_path);
                    }
                }
            })
            .unwrap_or_else(|e| {
                log::error!("Failed to invoke from event loop: {:?}", e);
            });
        });
    }

    /// Analyze a file and return analysis data
    fn analyze_file(path: &Path) -> Result<AnalysisData, Error> {
        log::info!("Starting analysis of: {}", path.display());

        let mut analyzer = DependencyAnalyzer::new();
        analyzer.set_max_depth(10);
        analyzer.set_include_system_dlls(false);

        // Build dependency tree
        let tree = analyzer.build_tree(path)?;

        // Parse PE file for detailed information
        let file_map = PEFileMap::new(path)?;
        let pe_file = PEFile::new(&file_map)?;
        let pe_info = pe_file.get_info()?;

        // Convert data for Slint
        let dependency_tree = if let Some(root) = &tree.root {
            vec![Self::convert_dependency_node(root)]
        } else {
            vec![]
        };

        // Get detailed import and export information
        let detailed_imports = pe_file.get_detailed_imports()?;
        let detailed_exports = pe_file.get_detailed_exports()?;

        // Convert detailed imports to function data
        let mut import_functions = Vec::new();
        for (dll_name, imports) in &detailed_imports {
            for import in imports {
                let name = if let Some(import_name) = &import.name {
                    import_name.clone()
                } else if let Some(ordinal) = import.ordinal {
                    format!("Ordinal #{}", ordinal)
                } else {
                    "Unknown".to_string()
                };

                let ordinal_str = import.ordinal.map(|o| o.to_string()).unwrap_or_default();
                let hint_str = import.hint.map(|h| h.to_string()).unwrap_or_default();

                import_functions.push(FunctionInfoData {
                    name: format!("{} -> {}", dll_name, name),
                    ordinal: ordinal_str,
                    address: String::new(), // Import addresses are resolved at runtime
                    hint: hint_str,
                    is_forwarded: false,
                    forward_name: String::new(),
                });
            }
        }

        // Convert detailed exports to function data
        let mut export_functions = Vec::new();
        for export in &detailed_exports {
            let name = export
                .name
                .clone()
                .unwrap_or_else(|| format!("Ordinal #{}", export.ordinal));

            export_functions.push(FunctionInfoData {
                name,
                ordinal: export.ordinal.to_string(),
                address: format!("0x{:08X}", export.rva),
                hint: String::new(),
                is_forwarded: export.is_forwarded,
                forward_name: export.forward_name.clone().unwrap_or_default(),
            });
        }

        Ok(AnalysisData {
            pe_info,
            dependency_tree,
            selected_module: None,
            import_functions,
            export_functions,
        })
    }

    /// Convert dependency node to Slint data structure
    fn convert_dependency_node(
        node: &crate::core::dependency::DependencyNode,
    ) -> DependencyItemData {
        let status = if !node.found {
            DependencyStatusData::Missing
        } else if Self::is_system_dll(&node.name) {
            DependencyStatusData::SystemDll
        } else {
            DependencyStatusData::Found
        };

        let children = node
            .children
            .iter()
            .map(Self::convert_dependency_node)
            .collect();

        DependencyItemData {
            name: node.name.clone(),
            path: node.path.to_string_lossy().to_string(),
            status,
            children,
            is_expanded: false,
        }
    }

    /// Check if a DLL is a system DLL
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

    /// Update UI with analysis results
    fn update_ui_with_analysis(window: MainWindow, analysis_data: AnalysisData) {
        log::info!("Updating UI with analysis results");

        // Convert dependency tree to Slint model (flatten for display)
        let dependencies_model = Rc::new(slint::VecModel::default());
        for dep in &analysis_data.dependency_tree {
            Self::add_dependency_items_to_model(&dependencies_model, dep, 0);
        }
        window.set_dependencies(dependencies_model.into());

        // Convert import functions to Slint model
        let import_functions_model = Rc::new(slint::VecModel::default());
        for func in &analysis_data.import_functions {
            let mut slint_func = Self::convert_to_slint_function_info(func);
            slint_func.is_export = false; // Import functions
            import_functions_model.push(slint_func);
        }
        window.set_import_functions(import_functions_model.into());

        // Convert export functions to Slint model
        let export_functions_model = Rc::new(slint::VecModel::default());
        for func in &analysis_data.export_functions {
            let mut slint_func = Self::convert_to_slint_function_info(func);
            slint_func.is_export = true; // Export functions
            export_functions_model.push(slint_func);
        }
        window.set_export_functions(export_functions_model.into());

        // Set module info if available
        if let Some(module_info) = &analysis_data.selected_module {
            window.set_module_info(Self::convert_to_slint_module_info(module_info));
        } else {
            // Create default module info from PE info
            let module_info = Self::create_module_info_from_pe(&analysis_data.pe_info);
            window.set_module_info(Self::convert_to_slint_module_info(&module_info));
        }

        log::info!("UI updated successfully");
    }

    /// Convert internal dependency item to Slint DependencyItem
    #[allow(dead_code)]
    fn convert_to_slint_dependency_item(dep: &DependencyItemData) -> DependencyItem {
        let status = match dep.status {
            DependencyStatusData::Found => DependencyStatus::Found,
            DependencyStatusData::Missing => DependencyStatus::Missing,
            DependencyStatusData::SystemDll => DependencyStatus::SystemDll,
            DependencyStatusData::Delayed => DependencyStatus::Delayed,
        };

        // Extract file extension for icon display
        let extension = std::path::Path::new(&dep.name)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_string();

        DependencyItem {
            name: dep.name.clone().into(),
            status,
            path: dep.path.clone().into(),
            extension: extension.into(),
            depth: 0, // Will be calculated during tree traversal
            has_children: !dep.children.is_empty(),
            is_expanded: dep.is_expanded,
        }
    }

    /// Add dependency items to model recursively with proper depth
    fn add_dependency_items_to_model(
        model: &Rc<slint::VecModel<DependencyItem>>,
        dep: &DependencyItemData,
        depth: i32,
    ) {
        let status = match dep.status {
            DependencyStatusData::Found => DependencyStatus::Found,
            DependencyStatusData::Missing => DependencyStatus::Missing,
            DependencyStatusData::SystemDll => DependencyStatus::SystemDll,
            DependencyStatusData::Delayed => DependencyStatus::Delayed,
        };

        // Extract file extension for icon display
        let extension = std::path::Path::new(&dep.name)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_string();

        let item = DependencyItem {
            name: dep.name.clone().into(),
            status,
            path: dep.path.clone().into(),
            extension: extension.into(),
            depth,
            has_children: !dep.children.is_empty(),
            is_expanded: dep.is_expanded,
        };

        model.push(item);

        // Add children if expanded
        if dep.is_expanded {
            for child in &dep.children {
                Self::add_dependency_items_to_model(model, child, depth + 1);
            }
        }
    }

    /// Convert internal function info to Slint FunctionInfo
    fn convert_to_slint_function_info(func: &FunctionInfoData) -> FunctionInfo {
        FunctionInfo {
            name: func.name.clone().into(),
            ordinal: func.ordinal.clone().into(),
            address: func.address.clone().into(),
            is_export: false, // Will be set correctly by the caller
        }
    }

    /// Convert internal module info to Slint ModuleInfo
    fn convert_to_slint_module_info(module: &ModuleInfoData) -> ModuleInfo {
        // Extract file extension for icon display
        let extension = std::path::Path::new(&module.name)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_string();

        // Format file size
        let file_size_str = Self::format_file_size(module.file_size);

        // Get version info
        let version = module
            .version_info
            .get("FileVersion")
            .cloned()
            .unwrap_or_default();

        let description = module
            .version_info
            .get("FileDescription")
            .cloned()
            .unwrap_or_default();

        ModuleInfo {
            name: module.name.clone().into(),
            status: DependencyStatus::Found, // Default status, should be updated based on context
            path: module.path.clone().into(),
            extension: extension.into(),
            file_size: file_size_str.into(),
            version: version.into(),
            description: description.into(),
            dependencies_count: 0, // Will be updated when we have dependency info
            exports_count: 0,      // Will be updated when we have export info
            imports_count: 0,      // Will be updated when we have import info
            // Additional PE information
            architecture: module.architecture.clone().into(),
            pe_type: module.pe_type.clone().into(),
            entry_point: module.entry_point.clone().into(),
            image_base: module.image_base.clone().into(),
            subsystem: module.subsystem.clone().into(),
            checksum: "".into(),  // TODO: Add checksum to ModuleInfoData
            timestamp: "".into(), // TODO: Add timestamp to ModuleInfoData
        }
    }

    /// Format file size in human-readable format
    fn format_file_size(size: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
        let mut size_f = size as f64;
        let mut unit_index = 0;

        while size_f >= 1024.0 && unit_index < UNITS.len() - 1 {
            size_f /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", size, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size_f, UNITS[unit_index])
        }
    }

    /// Create module info from PE information
    fn create_module_info_from_pe(pe_info: &PEInfo) -> ModuleInfoData {
        let version_info = HashMap::new(); // PEInfo doesn't have version info yet

        // Get file size if possible
        let file_size = std::fs::metadata(&pe_info.path)
            .map(|metadata| metadata.len())
            .unwrap_or(0);

        // For now, use placeholder values since PEInfo doesn't have these fields
        // TODO: Extend PEInfo to include more detailed PE information
        let entry_point = String::new(); // Not available in current PEInfo
        let image_base = String::new(); // Not available in current PEInfo
        let subsystem = String::new(); // Not available in current PEInfo

        ModuleInfoData {
            name: pe_info
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
                .to_string(),
            path: pe_info.path.to_string_lossy().to_string(),
            file_size,
            architecture: if pe_info.is_64bit { "x64" } else { "x86" }.to_string(),
            pe_type: if pe_info.is_dll() { "DLL" } else { "EXE" }.to_string(),
            entry_point,
            image_base,
            subsystem,
            version_info,
        }
    }

    /// Handle dependency selection
    fn handle_dependency_selected(window: MainWindow, dependency_name: String) {
        log::info!("Dependency selected: {}", dependency_name);
        window.set_selected_dependency(dependency_name.clone().into());

        // Try to analyze the selected dependency and update module details
        let window_weak = window.as_weak();
        let dep_name = dependency_name.clone();

        std::thread::spawn(move || {
            // Try to find the selected dependency file
            let result = Self::find_and_analyze_dependency(&dep_name);

            slint::invoke_from_event_loop(move || {
                if let Some(window) = window_weak.upgrade() {
                    match result {
                        Ok(Some(analysis_data)) => {
                            // Update module info with the selected dependency's details
                            let module_info =
                                Self::create_module_info_from_pe(&analysis_data.pe_info);
                            window
                                .set_module_info(Self::convert_to_slint_module_info(&module_info));

                            // Update function lists with the selected dependency's functions
                            let import_functions_model = Rc::new(slint::VecModel::default());
                            for func in &analysis_data.import_functions {
                                let mut slint_func = Self::convert_to_slint_function_info(func);
                                slint_func.is_export = false;
                                import_functions_model.push(slint_func);
                            }
                            window.set_import_functions(import_functions_model.into());

                            let export_functions_model = Rc::new(slint::VecModel::default());
                            for func in &analysis_data.export_functions {
                                let mut slint_func = Self::convert_to_slint_function_info(func);
                                slint_func.is_export = true;
                                export_functions_model.push(slint_func);
                            }
                            window.set_export_functions(export_functions_model.into());
                        }
                        Ok(None) => {
                            log::warn!("Could not find dependency file: {}", dep_name);
                        }
                        Err(e) => {
                            log::error!("Failed to analyze dependency {}: {}", dep_name, e);
                        }
                    }
                }
            })
            .unwrap_or_else(|e| {
                log::error!("Failed to invoke from event loop: {:?}", e);
            });
        });
    }

    /// Find and analyze a dependency by name
    fn find_and_analyze_dependency(dependency_name: &str) -> Result<Option<AnalysisData>, Error> {
        // Common Windows system directories to search for DLLs
        let search_paths = vec![
            std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string()) + "\\System32",
            std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string()) + "\\SysWOW64",
            std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string()),
            std::env::current_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        ];

        // Try to find the dependency file
        for search_path in &search_paths {
            let potential_path = PathBuf::from(search_path).join(dependency_name);

            if potential_path.exists() {
                log::info!("Found dependency at: {}", potential_path.display());

                // Analyze the found dependency
                match Self::analyze_file(&potential_path) {
                    Ok(analysis_data) => return Ok(Some(analysis_data)),
                    Err(e) => {
                        log::warn!("Failed to analyze {}: {}", potential_path.display(), e);
                        continue;
                    }
                }
            }
        }

        // Also try searching in PATH environment variable
        if let Ok(path_env) = std::env::var("PATH") {
            for path_dir in path_env.split(';') {
                let potential_path = PathBuf::from(path_dir).join(dependency_name);

                if potential_path.exists() {
                    log::info!("Found dependency in PATH at: {}", potential_path.display());

                    match Self::analyze_file(&potential_path) {
                        Ok(analysis_data) => return Ok(Some(analysis_data)),
                        Err(e) => {
                            log::warn!("Failed to analyze {}: {}", potential_path.display(), e);
                            continue;
                        }
                    }
                }
            }
        }

        log::warn!("Could not find dependency: {}", dependency_name);
        Ok(None)
    }

    /// Handle dependency expansion
    fn handle_dependency_expanded(_window: MainWindow, dependency_name: String) {
        log::info!("Dependency expanded: {}", dependency_name);

        // TODO: Implement dependency expansion logic
        // This would involve updating the tree model to show/hide children
    }

    /// Handle function selection
    fn handle_function_selected(window: MainWindow, function_name: String) {
        log::info!("Function selected: {}", function_name);
        window.set_selected_function(function_name.into());
    }

    /// Handle refresh analysis
    fn handle_refresh_analysis(window: MainWindow) {
        log::info!("Refresh analysis requested");

        let current_file = window.get_current_file().to_string();
        if !current_file.is_empty() {
            Self::handle_file_selected(window, current_file);
        }
    }

    /// Handle expand all
    fn handle_expand_all(_window: MainWindow) {
        log::info!("Expand all requested");

        // TODO: Implement expand all logic
        // This would involve updating all dependency items to be expanded
    }

    /// Handle collapse all
    fn handle_collapse_all(_window: MainWindow) {
        log::info!("Collapse all requested");

        // TODO: Implement collapse all logic
        // This would involve updating all dependency items to be collapsed
    }

    /// Handle theme toggle
    fn handle_toggle_theme(_window: MainWindow) {
        log::info!("Theme toggle requested");

        // TODO: Implement theme switching logic
        // This could involve updating CSS properties or switching theme files
    }
}
