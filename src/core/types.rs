//! PE-related data structures and types

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Import information for a symbol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportInfo {
    /// Symbol name (if imported by name)
    pub name: Option<String>,
    /// Ordinal number (if imported by ordinal)
    pub ordinal: Option<u16>,
    /// Hint value
    pub hint: Option<u16>,
    /// Address where the symbol is loaded
    pub address: Option<u64>,
}

impl ImportInfo {
    /// Create a new import by name
    pub fn by_name(name: String, hint: Option<u16>) -> Self {
        Self {
            name: Some(name),
            ordinal: None,
            hint,
            address: None,
        }
    }
    
    /// Create a new import by ordinal
    pub fn by_ordinal(ordinal: u16) -> Self {
        Self {
            name: None,
            ordinal: Some(ordinal),
            hint: None,
            address: None,
        }
    }
    
    /// Get display name for this import
    pub fn display_name(&self) -> String {
        if let Some(name) = &self.name {
            name.clone()
        } else if let Some(ord) = self.ordinal {
            format!("#{}", ord)
        } else {
            "Unknown".to_string()
        }
    }
}

/// Export information for a symbol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportInfo {
    /// Symbol name
    pub name: Option<String>,
    /// Ordinal number
    pub ordinal: u16,
    /// RVA (Relative Virtual Address)
    pub rva: u32,
    /// Whether this is a forwarded export
    pub is_forwarded: bool,
    /// Forward string (if forwarded)
    pub forward_name: Option<String>,
}

impl ExportInfo {
    /// Get display name for this export
    pub fn display_name(&self) -> String {
        if let Some(name) = &self.name {
            name.clone()
        } else {
            format!("#{}", self.ordinal)
        }
    }
}

/// PE file architecture
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Architecture {
    X86,
    X64,
    ARM,
    ARM64,
    Unknown,
}

impl Architecture {
    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Architecture::X86 => "x86",
            Architecture::X64 => "x64",
            Architecture::ARM => "ARM",
            Architecture::ARM64 => "ARM64",
            Architecture::Unknown => "Unknown",
        }
    }
}

/// PE file type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PEType {
    Executable,
    DynamicLibrary,
    Driver,
    Unknown,
}

impl PEType {
    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            PEType::Executable => "EXE",
            PEType::DynamicLibrary => "DLL",
            PEType::Driver => "SYS",
            PEType::Unknown => "Unknown",
        }
    }
}

/// Detailed PE file information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedPEInfo {
    /// File path
    pub path: PathBuf,
    /// File size in bytes
    pub file_size: u64,
    /// PE architecture
    pub architecture: Architecture,
    /// PE file type
    pub pe_type: PEType,
    /// DLL name (for DLLs)
    pub dll_name: Option<String>,
    /// Entry point RVA
    pub entry_point: Option<u32>,
    /// Image base address
    pub image_base: Option<u64>,
    /// Subsystem
    pub subsystem: Option<String>,
    /// Dependencies
    pub dependencies: Vec<String>,
    /// Import table information
    pub imports: std::collections::HashMap<String, Vec<ImportInfo>>,
    /// Export table information
    pub exports: Vec<ExportInfo>,
    /// File version information
    pub version_info: Option<VersionInfo>,
}

/// Version information from PE resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub file_version: Option<String>,
    pub product_version: Option<String>,
    pub company_name: Option<String>,
    pub file_description: Option<String>,
    pub product_name: Option<String>,
    pub copyright: Option<String>,
}

/// PE parsing statistics
#[derive(Debug, Clone, Default)]
pub struct ParseStats {
    /// Number of dependencies found
    pub dependency_count: usize,
    /// Number of imported symbols
    pub import_count: usize,
    /// Number of exported symbols
    pub export_count: usize,
    /// Parsing time in milliseconds
    pub parse_time_ms: u64,
    /// Which parser was used successfully
    pub parser_used: Option<String>,
    /// Any warnings encountered
    pub warnings: Vec<String>,
}

impl ParseStats {
    /// Add a warning message
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
    
    /// Check if parsing was successful
    pub fn is_successful(&self) -> bool {
        self.parser_used.is_some()
    }
}
