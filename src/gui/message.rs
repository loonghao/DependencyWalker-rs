//! Message types for the ICED GUI application
//!
//! This module defines all the messages that can be sent within the ICED application.

use std::path::PathBuf;

/// Analysis result for GUI display
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub file_path: PathBuf,
    pub dependencies: Vec<DependencyInfo>,
    pub analysis_time: std::time::Duration,
}

/// Dependency information for GUI display
#[derive(Debug, Clone)]
pub struct DependencyInfo {
    pub name: String,
    pub path: Option<PathBuf>,
    pub status: DependencyStatus,
    pub children: Vec<DependencyInfo>,
}

/// Dependency status
#[derive(Debug, Clone)]
pub enum DependencyStatus {
    Found,
    Missing,
    SystemDll,
    Delayed,
}

/// Main application messages
#[derive(Debug, Clone)]
pub enum Message {
    /// File operations
    OpenFileDialog,
    FileSelected(Option<PathBuf>),
    FileDropped(PathBuf),
    FilesHovered(Vec<PathBuf>),
    FileHoverCancelled,

    /// Analysis operations
    AnalyzeFile(PathBuf),
    AnalysisCompleted(Result<AnalysisResult, String>),

    /// UI operations
    ToggleTheme,
    ToggleSidebar,
    SelectDependency(String),

    /// Window operations
    WindowResized(iced::Size),

    /// Settings
    UpdateMaxDepth(u32),
    ToggleSystemDlls,
    AddSearchPath(PathBuf),
    RemoveSearchPath(usize),
}

impl std::fmt::Display for DependencyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DependencyStatus::Found => write!(f, "Found"),
            DependencyStatus::Missing => write!(f, "Missing"),
            DependencyStatus::SystemDll => write!(f, "System DLL"),
            DependencyStatus::Delayed => write!(f, "Delayed"),
        }
    }
}
