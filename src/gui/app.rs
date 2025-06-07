//! Main application logic for the GUI
//!
//! This module contains the main application state and logic for the ICED-based GUI.

use crate::core::dependency::DependencyAnalyzer;
use crate::core::dependency::DependencyNode;
use crate::gui::style::{AppTheme, Colors, FileTypes, FontSize, Spacing};
use crate::gui::{
    message::{AnalysisResult, DependencyInfo, DependencyStatus},
    Message,
};
use iced::{
    widget::{button, column, container, row, scrollable, text},
    Element, Event, Length, Subscription, Task, Theme,
};
use std::path::PathBuf;

/// Main application state
#[derive(Debug, Default)]
pub struct DependencyWalkerApp {
    /// Currently loaded file path
    current_file: Option<PathBuf>,
    /// Analysis result
    analysis_result: Option<AnalysisResult>,
    /// Application theme
    theme: AppTheme,
    /// Analysis settings
    analysis_settings: AnalysisSettings,
    /// UI state
    ui_state: UiState,
    /// Loading state
    is_loading: bool,
    /// Error message
    error_message: Option<String>,
}

/// Analysis settings
#[derive(Debug, Clone)]
pub struct AnalysisSettings {
    pub max_depth: u32,
    pub include_system_dlls: bool,
    pub search_paths: Vec<PathBuf>,
}

/// UI state
#[derive(Debug)]
pub struct UiState {
    /// Sidebar visibility
    pub sidebar_visible: bool,
    /// Selected dependency
    pub selected_dependency: Option<String>,
    /// File dialog open state
    pub file_dialog_open: bool,
    /// Drag and drop state
    pub is_dragging: bool,
}

impl Default for AnalysisSettings {
    fn default() -> Self {
        Self {
            max_depth: 10,
            include_system_dlls: false,
            search_paths: Vec::new(),
        }
    }
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            sidebar_visible: true,
            selected_dependency: None,
            file_dialog_open: false,
            is_dragging: false,
        }
    }
}

impl DependencyWalkerApp {
    /// Create a new application instance
    pub fn new() -> Self {
        Self {
            current_file: None,
            analysis_result: None,
            theme: AppTheme::default(),
            analysis_settings: AnalysisSettings::default(),
            ui_state: UiState::default(),
            is_loading: false,
            error_message: None,
        }
    }

    /// Run the application
    pub fn run() -> iced::Result {
        iced::application(
            "DependencyWalker RS",
            Self::update_static,
            Self::view_static,
        )
        .subscription(Self::subscription_static)
        .theme(Self::theme_static)
        .run()
    }

    /// Static update function for iced
    fn update_static(state: &mut Self, message: Message) -> Task<Message> {
        state.update(message)
    }

    /// Static view function for iced
    fn view_static(state: &Self) -> Element<Message> {
        state.view()
    }

    /// Static theme function for iced
    fn theme_static(state: &Self) -> Theme {
        state.theme()
    }

    /// Static subscription function for iced
    fn subscription_static(_state: &Self) -> Subscription<Message> {
        iced::event::listen_with(|event, _status, _id| match event {
            Event::Window(iced::window::Event::FileDropped(path)) => {
                Some(Message::FileDropped(path))
            }
            Event::Window(iced::window::Event::FileHovered(path)) => {
                Some(Message::FilesHovered(vec![path]))
            }
            Event::Window(iced::window::Event::FilesHoveredLeft) => {
                Some(Message::FileHoverCancelled)
            }
            Event::Window(iced::window::Event::Resized(size)) => Some(Message::WindowResized(size)),
            _ => None,
        })
    }

    /// Update the application state
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenFileDialog => {
                // Prevent opening multiple file dialogs
                if self.ui_state.file_dialog_open {
                    return Task::none();
                }

                // Set dialog open state
                self.ui_state.file_dialog_open = true;

                // Open file dialog
                Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .add_filter("PE Files", &["exe", "dll", "sys", "ocx", "mll"])
                            .add_filter("Executable Files", &["exe"])
                            .add_filter("Library Files", &["dll", "mll"])
                            .add_filter("System Files", &["sys"])
                            .add_filter("Control Files", &["ocx"])
                            .add_filter("Maya Plugins", &["mll"])
                            .add_filter("All Files", &["*"])
                            .set_title("Select PE file or Maya plugin to analyze")
                            .pick_file()
                            .await
                            .map(|handle| handle.path().to_path_buf())
                    },
                    Message::FileSelected,
                )
            }
            Message::FileSelected(path) => {
                // Reset dialog open state
                self.ui_state.file_dialog_open = false;

                if let Some(path) = path {
                    // Validate file extension
                    if let Some(extension) = path.extension() {
                        let ext = extension.to_string_lossy().to_lowercase();
                        if !["exe", "dll", "sys", "ocx", "mll"].contains(&ext.as_str()) {
                            self.error_message = Some(format!("Unsupported file type: .{}", ext));
                            return Task::none();
                        }
                    } else {
                        self.error_message = Some("File has no extension".to_string());
                        return Task::none();
                    }

                    self.current_file = Some(path.clone());
                    self.error_message = None;
                    Task::perform(async move { path }, Message::AnalyzeFile)
                } else {
                    Task::none()
                }
            }
            Message::FileDropped(path) => {
                // Reset drag state
                self.ui_state.is_dragging = false;

                // Validate file extension
                if let Some(extension) = path.extension() {
                    let ext = extension.to_string_lossy().to_lowercase();
                    if !["exe", "dll", "sys", "ocx", "mll"].contains(&ext.as_str()) {
                        self.error_message = Some(format!("Unsupported file type: .{}", ext));
                        return Task::none();
                    }
                } else {
                    self.error_message = Some("File has no extension".to_string());
                    return Task::none();
                }

                self.current_file = Some(path.clone());
                self.error_message = None;
                Task::perform(async move { path }, Message::AnalyzeFile)
            }
            Message::AnalyzeFile(path) => {
                self.is_loading = true;
                self.error_message = None;

                // Clone settings for the async task
                let max_depth = self.analysis_settings.max_depth;
                let include_system_dlls = self.analysis_settings.include_system_dlls;
                let search_paths = self.analysis_settings.search_paths.clone();

                Task::perform(
                    async move {
                        let start_time = std::time::Instant::now();

                        // Create and configure dependency analyzer
                        let mut analyzer = DependencyAnalyzer::new();
                        analyzer.set_max_depth(max_depth as usize);
                        analyzer.set_include_system_dlls(include_system_dlls);

                        // Add search paths
                        for search_path in search_paths {
                            analyzer.add_search_path(&search_path);
                        }

                        // Build dependency tree
                        match analyzer.build_tree(&path) {
                            Ok(tree) => {
                                let analysis_time = start_time.elapsed();

                                // Convert DependencyTree to AnalysisResult
                                let dependencies = if let Some(root) = &tree.root {
                                    convert_dependency_node_to_info(root)
                                } else {
                                    vec![]
                                };

                                Ok(AnalysisResult {
                                    file_path: path,
                                    dependencies,
                                    analysis_time,
                                })
                            }
                            Err(e) => Err(format!("Analysis failed: {}", e)),
                        }
                    },
                    Message::AnalysisCompleted,
                )
            }
            Message::AnalysisCompleted(result) => {
                self.is_loading = false;
                match result {
                    Ok(analysis_result) => {
                        self.analysis_result = Some(analysis_result);
                        self.error_message = None;
                    }
                    Err(error) => {
                        self.error_message = Some(error);
                    }
                }
                Task::none()
            }
            Message::ToggleTheme => {
                self.theme = self.theme.toggle();
                Task::none()
            }
            Message::ToggleSidebar => {
                self.ui_state.sidebar_visible = !self.ui_state.sidebar_visible;
                Task::none()
            }
            Message::SelectDependency(name) => {
                self.ui_state.selected_dependency = Some(name);
                Task::none()
            }
            Message::WindowResized(_size) => Task::none(),
            Message::UpdateMaxDepth(depth) => {
                self.analysis_settings.max_depth = depth;
                Task::none()
            }
            Message::ToggleSystemDlls => {
                self.analysis_settings.include_system_dlls =
                    !self.analysis_settings.include_system_dlls;
                Task::none()
            }
            Message::AddSearchPath(path) => {
                self.analysis_settings.search_paths.push(path);
                Task::none()
            }
            Message::RemoveSearchPath(index) => {
                if index < self.analysis_settings.search_paths.len() {
                    self.analysis_settings.search_paths.remove(index);
                }
                Task::none()
            }
            Message::FilesHovered(paths) => {
                // Handle file hover for drag & drop feedback
                self.ui_state.is_dragging = true;
                log::debug!("Files hovered: {:?}", paths);
                Task::none()
            }
            Message::FileHoverCancelled => {
                // Handle hover cancellation
                self.ui_state.is_dragging = false;
                log::debug!("File hover cancelled");
                Task::none()
            }
        }
    }

    /// Get the application title
    pub fn title(&self) -> String {
        match &self.current_file {
            Some(path) => format!("DependencyWalker RS - {}", path.display()),
            None => "DependencyWalker RS".to_string(),
        }
    }

    /// Get the application theme
    pub fn theme(&self) -> Theme {
        self.theme.to_iced_theme()
    }

    /// Create the main view
    pub fn view(&self) -> Element<Message> {
        let content = column![self.create_menu_bar(), self.create_main_layout(),].spacing(0);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// Create menu bar
    fn create_menu_bar(&self) -> Element<Message> {
        let menu_bar = row![
            button("File").padding(8),
            button("View").padding(8),
            button("Options").padding(8),
            button("Help").padding(8),
        ]
        .spacing(0);

        let toolbar = row![
            button("Open").on_press(Message::OpenFileDialog).padding(4),
            button("Refresh").padding(4),
            text("|").size(16),
            button("Expand All").padding(4),
            button("Collapse All").padding(4),
        ]
        .spacing(8);

        column![
            container(menu_bar)
                .width(Length::Fill)
                .padding(4)
                .style(|theme: &Theme| {
                    iced::widget::container::Style {
                        background: Some(iced::Background::Color(
                            theme.extended_palette().background.weak.color,
                        )),
                        border: iced::Border {
                            width: 1.0,
                            color: theme.extended_palette().background.strong.color,
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                }),
            container(toolbar)
                .width(Length::Fill)
                .padding(8)
                .style(|theme: &Theme| {
                    iced::widget::container::Style {
                        background: Some(iced::Background::Color(
                            theme.extended_palette().background.base.color,
                        )),
                        border: iced::Border {
                            width: 1.0,
                            color: theme.extended_palette().background.strong.color,
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                }),
        ]
        .into()
    }

    /// Create main layout with traditional DependencyWalker style
    fn create_main_layout(&self) -> Element<Message> {
        if self.current_file.is_none() {
            // Show welcome screen with drag & drop area
            self.create_welcome_screen()
        } else {
            // Show analysis layout
            self.create_analysis_layout()
        }
    }

    /// Create modern welcome screen with drag & drop support
    fn create_welcome_screen(&self) -> Element<Message> {
        // Determine drag state styling
        let drag_border_color = if self.ui_state.is_dragging {
            Colors::PRIMARY
        } else {
            Colors::BORDER
        };

        let drag_border_width = if self.ui_state.is_dragging { 3.0 } else { 2.0 };

        let drag_background = if self.ui_state.is_dragging {
            Colors::PRIMARY.scale_alpha(0.1)
        } else {
            Colors::BACKGROUND_SECONDARY
        };

        let main_content = container(
            column![
                // Modern title with gradient-like styling
                text("DependencyWalker RS")
                    .size(FontSize::DISPLAY)
                    .style(|_theme: &Theme| {
                        iced::widget::text::Style {
                            color: Some(Colors::PRIMARY),
                        }
                    }),
                text("Modern Windows & Maya Plugin Dependency Analyzer")
                    .size(FontSize::HEADING)
                    .style(|_theme: &Theme| {
                        iced::widget::text::Style {
                            color: Some(Colors::TEXT_SECONDARY),
                        }
                    }),
                // Spacer
                text("").size(Spacing::EXTRA_LARGE),
                // Modern drop zone with better styling and drag feedback
                container(
                    column![
                        // File type icons row - using Unicode symbols for better compatibility
                        row![
                            text("▶").size(FontSize::DISPLAY).style(|_theme: &Theme| {
                                iced::widget::text::Style {
                                    color: Some(Colors::PRIMARY),
                                }
                            }),
                            text("⚙").size(FontSize::DISPLAY).style(|_theme: &Theme| {
                                iced::widget::text::Style {
                                    color: Some(Colors::SUCCESS),
                                }
                            }),
                            text("⚡").size(FontSize::DISPLAY).style(|_theme: &Theme| {
                                iced::widget::text::Style {
                                    color: Some(Colors::WARNING),
                                }
                            }),
                            text("◆").size(FontSize::DISPLAY).style(|_theme: &Theme| {
                                iced::widget::text::Style {
                                    color: Some(Colors::INFO),
                                }
                            }),
                            text("★").size(FontSize::DISPLAY).style(|_theme: &Theme| {
                                iced::widget::text::Style {
                                    color: Some(Colors::MLL_PLUGIN),
                                }
                            }),
                        ]
                        .spacing(Spacing::LARGE),
                        text(if self.ui_state.is_dragging {
                            "Drop files here!"
                        } else {
                            "Drag & Drop files here"
                        })
                        .size(FontSize::HEADING)
                        .style(move |_theme: &Theme| {
                            iced::widget::text::Style {
                                color: Some(if self.ui_state.is_dragging {
                                    Colors::PRIMARY
                                } else {
                                    Colors::TEXT_PRIMARY
                                }),
                            }
                        }),
                        text("or").size(FontSize::MEDIUM).style(|_theme: &Theme| {
                            iced::widget::text::Style {
                                color: Some(Colors::TEXT_MUTED),
                            }
                        }),
                        button(if self.ui_state.file_dialog_open {
                            "Opening..."
                        } else {
                            "📂 Browse Files"
                        })
                        .on_press_maybe(if self.ui_state.file_dialog_open {
                            None
                        } else {
                            Some(Message::OpenFileDialog)
                        })
                        .padding([Spacing::MEDIUM, Spacing::LARGE])
                        .style(move |_theme: &Theme, status| {
                            let mut style = iced::widget::button::Style::default();
                            let base_color = if self.ui_state.file_dialog_open {
                                Colors::TEXT_MUTED
                            } else {
                                Colors::PRIMARY
                            };
                            style.background = Some(iced::Background::Color(base_color));
                            style.text_color = Colors::TEXT_PRIMARY;
                            style.border = iced::Border {
                                width: 0.0,
                                radius: 8.0.into(),
                                color: iced::Color::TRANSPARENT,
                            };

                            match status {
                                iced::widget::button::Status::Hovered
                                    if !self.ui_state.file_dialog_open =>
                                {
                                    style.background =
                                        Some(iced::Background::Color(Colors::PRIMARY_HOVER));
                                }
                                _ => {}
                            }

                            style
                        }),
                    ]
                    .spacing(Spacing::LARGE)
                    .align_x(iced::Alignment::Center)
                )
                .padding(Spacing::MASSIVE)
                .style(move |_theme: &Theme| {
                    iced::widget::container::Style {
                        background: Some(iced::Background::Color(drag_background)),
                        border: iced::Border {
                            width: drag_border_width,
                            color: drag_border_color,
                            radius: 12.0.into(),
                        },
                        ..Default::default()
                    }
                }),
                // Spacer
                text("").size(Spacing::LARGE),
                // Supported formats with modern styling
                column![
                    text("Supported Formats:")
                        .size(FontSize::MEDIUM)
                        .style(|_theme: &Theme| {
                            iced::widget::text::Style {
                                color: Some(Colors::TEXT_SECONDARY),
                            }
                        }),
                    row![
                        text("▶ .exe")
                            .size(FontSize::SMALL)
                            .style(|_theme: &Theme| {
                                iced::widget::text::Style {
                                    color: Some(Colors::PRIMARY),
                                }
                            }),
                        text("⚙ .dll")
                            .size(FontSize::SMALL)
                            .style(|_theme: &Theme| {
                                iced::widget::text::Style {
                                    color: Some(Colors::SUCCESS),
                                }
                            }),
                        text("⚡ .sys")
                            .size(FontSize::SMALL)
                            .style(|_theme: &Theme| {
                                iced::widget::text::Style {
                                    color: Some(Colors::WARNING),
                                }
                            }),
                        text("◆ .ocx")
                            .size(FontSize::SMALL)
                            .style(|_theme: &Theme| {
                                iced::widget::text::Style {
                                    color: Some(Colors::INFO),
                                }
                            }),
                        text("★ .mll")
                            .size(FontSize::SMALL)
                            .style(|_theme: &Theme| {
                                iced::widget::text::Style {
                                    color: Some(Colors::MLL_PLUGIN),
                                }
                            }),
                    ]
                    .spacing(Spacing::LARGE),
                    text("Maya Plugin Libraries (.mll) fully supported!")
                        .size(FontSize::CAPTION)
                        .style(|_theme: &Theme| {
                            iced::widget::text::Style {
                                color: Some(Colors::MLL_PLUGIN),
                            }
                        }),
                ]
                .spacing(Spacing::SMALL)
                .align_x(iced::Alignment::Center),
            ]
            .spacing(Spacing::MEDIUM)
            .align_x(iced::Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill);

        if let Some(error) = &self.error_message {
            column![
                main_content,
                container(text(format!("Error: {}", error)).style(|theme: &Theme| {
                    iced::widget::text::Style {
                        color: Some(theme.extended_palette().danger.base.color),
                    }
                }))
                .padding(16)
                .width(Length::Fill)
            ]
            .spacing(Spacing::MEDIUM)
            .align_x(iced::Alignment::Center)
            .into()
        } else {
            main_content.into()
        }
    }

    /// Create analysis layout (traditional DependencyWalker style)
    fn create_analysis_layout(&self) -> Element<Message> {
        let file_info = if let Some(file_path) = &self.current_file {
            text(format!("File: {}", file_path.display()))
        } else {
            text("No file loaded")
        };

        // Top section: Dependency tree (left) + Module details (right)
        let top_section = row![
            // Left: Dependency tree
            container(
                column![
                    text("Module Dependency Tree").size(14),
                    if let Some(analysis) = &self.analysis_result {
                        self.create_dependency_tree(analysis)
                    } else if self.is_loading {
                        container(text("Analyzing..."))
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .center_x(Length::Fill)
                            .center_y(Length::Fill)
                            .into()
                    } else {
                        container(text("No analysis available"))
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .center_x(Length::Fill)
                            .center_y(Length::Fill)
                            .into()
                    }
                ]
                .spacing(8)
            )
            .width(Length::FillPortion(1))
            .height(Length::Fill)
            .padding(8)
            .style(|theme: &Theme| {
                iced::widget::container::Style {
                    background: Some(iced::Background::Color(
                        theme.extended_palette().background.base.color,
                    )),
                    border: iced::Border {
                        width: 1.0,
                        color: theme.extended_palette().background.strong.color,
                        ..Default::default()
                    },
                    ..Default::default()
                }
            }),
            // Right: Module details
            container(
                column![
                    text("Module Details").size(14),
                    if let Some(selected) = &self.ui_state.selected_dependency {
                        self.create_module_details(selected)
                    } else {
                        container(text("Select a module to view details"))
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .center_x(Length::Fill)
                            .center_y(Length::Fill)
                            .into()
                    }
                ]
                .spacing(8)
            )
            .width(Length::FillPortion(1))
            .height(Length::Fill)
            .padding(8)
            .style(|theme: &Theme| {
                iced::widget::container::Style {
                    background: Some(iced::Background::Color(
                        theme.extended_palette().background.base.color,
                    )),
                    border: iced::Border {
                        width: 1.0,
                        color: theme.extended_palette().background.strong.color,
                        ..Default::default()
                    },
                    ..Default::default()
                }
            }),
        ]
        .spacing(4);

        // Bottom section: Import/Export functions
        let bottom_section = container(
            column![
                text("Import/Export Functions").size(14),
                if let Some(analysis) = &self.analysis_result {
                    self.create_functions_list(analysis)
                } else {
                    container(text("No functions to display"))
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .center_x(Length::Fill)
                        .center_y(Length::Fill)
                        .into()
                }
            ]
            .spacing(8),
        )
        .width(Length::Fill)
        .height(Length::FillPortion(1))
        .padding(8)
        .style(|theme: &Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(
                theme.extended_palette().background.base.color,
            )),
            border: iced::Border {
                width: 1.0,
                color: theme.extended_palette().background.strong.color,
                ..Default::default()
            },
            ..Default::default()
        });

        column![
            container(file_info).padding(8),
            container(top_section)
                .width(Length::Fill)
                .height(Length::FillPortion(2)),
            bottom_section,
        ]
        .spacing(4)
        .into()
    }

    /// Create dependency tree view
    fn create_dependency_tree<'a>(&self, analysis: &'a AnalysisResult) -> Element<'a, Message> {
        let tree_items = analysis
            .dependencies
            .iter()
            .fold(column![].spacing(1), |col, dep| {
                col.push(self.create_dependency_item(dep, 0))
            });

        scrollable(tree_items)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// Create a single dependency item with modern styling and file type icons
    fn create_dependency_item<'a>(
        &self,
        dep: &'a DependencyInfo,
        depth: usize,
    ) -> Element<'a, Message> {
        // Get file extension for icon and color
        let extension = std::path::Path::new(&dep.name)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        let file_icon = FileTypes::get_icon(extension);
        let file_color = FileTypes::get_color(extension);

        let status_icon = match dep.status {
            DependencyStatus::Found => "✓",
            DependencyStatus::Missing => "✗",
            DependencyStatus::SystemDll => "🔧",
            DependencyStatus::Delayed => "⏳",
        };

        let status_color = match dep.status {
            DependencyStatus::Found => Colors::SUCCESS,
            DependencyStatus::Missing => Colors::ERROR,
            DependencyStatus::SystemDll => Colors::SYSTEM_DLL,
            DependencyStatus::Delayed => Colors::DELAYED,
        };

        let is_selected = self
            .ui_state
            .selected_dependency
            .as_ref()
            .map(|s| s == &dep.name)
            .unwrap_or(false);

        // Create indentation based on depth
        let indent = "  ".repeat(depth);
        let tree_prefix = if depth > 0 { "├─ " } else { "" };

        let item_button = button(
            row![
                text(format!("{}{}", indent, tree_prefix)).size(FontSize::SMALL),
                text(file_icon)
                    .size(FontSize::MEDIUM)
                    .style(move |_theme: &Theme| {
                        iced::widget::text::Style {
                            color: Some(file_color),
                        }
                    }),
                text(&dep.name).size(FontSize::BODY),
                text(status_icon)
                    .size(FontSize::SMALL)
                    .style(move |_theme: &Theme| {
                        iced::widget::text::Style {
                            color: Some(status_color),
                        }
                    }),
            ]
            .spacing(Spacing::SMALL),
        )
        .on_press(Message::SelectDependency(dep.name.clone()))
        .width(Length::Fill)
        .style(move |theme: &Theme, status| {
            let mut style = iced::widget::button::Style::default();

            if is_selected {
                style.background = Some(iced::Background::Color(
                    theme.extended_palette().primary.weak.color,
                ));
            }

            if status == iced::widget::button::Status::Hovered {
                style.background = Some(iced::Background::Color(
                    theme.extended_palette().background.weak.color,
                ));
            }

            style
        });

        // Create a column for this item and its children
        let mut item_column = column![item_button].spacing(1);

        // Add children with increased depth
        for child in &dep.children {
            item_column = item_column.push(self.create_dependency_item(child, depth + 1));
        }

        item_column.into()
    }

    /// Create module details view
    fn create_module_details<'a>(&self, selected_name: &str) -> Element<'a, Message> {
        if let Some(analysis) = &self.analysis_result {
            if let Some(dep) = analysis
                .dependencies
                .iter()
                .find(|d| d.name == selected_name)
            {
                let details = column![
                    text(format!("Module: {}", dep.name)).size(16),
                    text("").size(8), // Spacer
                    text(format!("Status: {}", dep.status)).size(12),
                    if let Some(path) = &dep.path {
                        text(format!("Path: {}", path.display())).size(10)
                    } else {
                        text("Path: Not found").size(10)
                    },
                    text("").size(8), // Spacer
                    text(format!("Dependencies: {}", dep.children.len())).size(12),
                ]
                .spacing(4);

                scrollable(details)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            } else {
                text("Module not found").into()
            }
        } else {
            text("No analysis data").into()
        }
    }

    /// Create functions list view
    fn create_functions_list<'a>(&self, analysis: &'a AnalysisResult) -> Element<'a, Message> {
        let functions_info = column![
            text("Import Functions:").size(14),
            text("• LoadLibraryA").size(12),
            text("• GetProcAddress").size(12),
            text("• FreeLibrary").size(12),
            text("").size(8), // Spacer
            text("Export Functions:").size(14),
            text("• DllMain").size(12),
            text(format!(
                "Total dependencies: {}",
                analysis.dependencies.len()
            ))
            .size(12),
        ]
        .spacing(4);

        scrollable(functions_info)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

/// Convert DependencyNode to DependencyInfo for GUI display
fn convert_dependency_node_to_info(node: &DependencyNode) -> Vec<DependencyInfo> {
    let mut result = vec![];

    // Convert current node
    let status = if !node.found {
        DependencyStatus::Missing
    } else if is_system_dll(&node.name) {
        DependencyStatus::SystemDll
    } else {
        DependencyStatus::Found
    };

    let children = node
        .children
        .iter()
        .flat_map(convert_dependency_node_to_info)
        .collect();

    result.push(DependencyInfo {
        name: node.name.clone(),
        path: if node.found {
            Some(node.path.clone())
        } else {
            None
        },
        status,
        children,
    });

    result
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
