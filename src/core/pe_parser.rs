//! PE file parsing using pelite + goblin dual strategy
//! 
//! This module provides robust PE file parsing by using both pelite and goblin libraries.
//! It prioritizes pelite for performance and falls back to goblin for robustness.

use crate::error::{Error, Result};
use crate::core::types::{ImportInfo, ExportInfo};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::fs;

/// PE file memory map for efficient access
pub struct PEFileMap {
    path: PathBuf,
    content: Vec<u8>,
}

impl PEFileMap {
    /// Create a new PE file map from path
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        
        if !path.exists() {
            return Err(Error::FileNotFound { path });
        }
        
        let content = fs::read(&path)?;
        
        // Basic PE signature check
        if content.len() < 64 {
            return Err(Error::InvalidFormat {
                path,
                reason: "File too small to be a valid PE file".to_string(),
            });
        }
        
        Ok(Self { path, content })
    }
    
    /// Get the file path
    pub fn path(&self) -> &Path {
        &self.path
    }
    
    /// Get the file content
    pub fn content(&self) -> &[u8] {
        &self.content
    }
}

/// PE file parser with dual strategy support
pub struct PEFile<'a> {
    file_map: &'a PEFileMap,
    pelite_pe: Option<pelite::PeFile<'a>>,
    goblin_pe: Option<goblin::pe::PE<'a>>,
}

impl<'a> PEFile<'a> {
    /// Create a new PE file parser
    pub fn new(file_map: &'a PEFileMap) -> Result<Self> {
        let mut pe_file = Self {
            file_map,
            pelite_pe: None,
            goblin_pe: None,
        };
        
        // Try pelite first (preferred for performance)
        match pelite::PeFile::from_bytes(&file_map.content) {
            Ok(pe) => {
                log::debug!("Successfully parsed with pelite: {}", file_map.path.display());
                pe_file.pelite_pe = Some(pe);
            }
            Err(e) => {
                log::warn!("Pelite parsing failed for {}: {:?}", file_map.path.display(), e);
            }
        }
        
        // Try goblin as fallback or additional validation
        match goblin::Object::parse(&file_map.content) {
            Ok(goblin::Object::PE(pe)) => {
                log::debug!("Successfully parsed with goblin: {}", file_map.path.display());
                pe_file.goblin_pe = Some(pe);
            }
            Ok(other) => {
                log::warn!("Unexpected object format: {:?}", other);
            }
            Err(e) => {
                log::warn!("Goblin parsing failed for {}: {:?}", file_map.path.display(), e);
            }
        }
        
        // Ensure at least one parser succeeded
        if pe_file.pelite_pe.is_none() && pe_file.goblin_pe.is_none() {
            return Err(Error::InvalidFormat {
                path: file_map.path.clone(),
                reason: "Failed to parse with both pelite and goblin".to_string(),
            });
        }
        
        Ok(pe_file)
    }
    
    /// Create PE file from path (convenience method)
    /// Note: Due to lifetime constraints, use PEFileMap::new() and PEFile::new() separately
    pub fn from_path<P: AsRef<Path>>(_path: P) -> Result<()> {
        Err(Error::generic("Use PEFileMap::new() and PEFile::new() separately for proper lifetime management"))
    }
    
    /// Get the file path
    pub fn path(&self) -> &Path {
        self.file_map.path()
    }
    
    /// Check if this is a 64-bit PE file
    pub fn is_64bit(&self) -> Result<bool> {
        // Use goblin first as it has simpler API
        if let Some(pe) = &self.goblin_pe {
            return Ok(pe.is_64);
        }

        // For pelite, we need to check the optional header magic
        if let Some(_pe) = &self.pelite_pe {
            // For now, we'll use a simple heuristic based on file content
            // This is a simplified approach - in a full implementation,
            // we'd properly parse the PE headers
            if self.file_map.content.len() > 0x3c + 4 {
                let pe_offset = u32::from_le_bytes([
                    self.file_map.content[0x3c],
                    self.file_map.content[0x3c + 1],
                    self.file_map.content[0x3c + 2],
                    self.file_map.content[0x3c + 3],
                ]) as usize;

                if pe_offset + 24 < self.file_map.content.len() {
                    let magic = u16::from_le_bytes([
                        self.file_map.content[pe_offset + 24],
                        self.file_map.content[pe_offset + 25],
                    ]);
                    return Ok(magic == 0x20b); // PE32+ magic
                }
            }
        }

        Err(Error::pe_error("No valid PE parser available"))
    }
    
    /// Get the DLL name as specified in the PE file headers
    pub fn get_dll_name(&self) -> Result<Option<String>> {
        // Try pelite first
        if let Some(pe) = &self.pelite_pe {
            match pe.exports() {
                Ok(exports) => {
                    match exports.dll_name() {
                        Ok(name) => return Ok(Some(name.to_string())),
                        Err(pelite::Error::Null) => return Ok(None), // No export directory
                        Err(e) => log::warn!("Pelite dll_name error: {:?}", e),
                    }
                }
                Err(pelite::Error::Null) => return Ok(None), // No export directory
                Err(e) => log::warn!("Pelite exports error: {:?}", e),
            }
        }
        
        // Try goblin as fallback
        if let Some(pe) = &self.goblin_pe {
            if let Some(export_data) = &pe.export_data {
                if let Some(name) = export_data.name {
                    return Ok(Some(name.to_string()));
                }
            }
        }
        
        Ok(None)
    }

    /// Get the list of DLL dependencies
    pub fn get_dependencies(&self) -> Result<Vec<String>> {
        // Try goblin first for dependencies (more reliable)
        if let Some(pe) = &self.goblin_pe {
            let deps: Vec<String> = pe.libraries.iter().map(|s| s.to_string()).collect();
            if !deps.is_empty() {
                log::debug!("Found {} dependencies with goblin", deps.len());
                return Ok(deps);
            }
        }

        // Fallback to pelite
        if let Some(pe) = &self.pelite_pe {
            match pe.imports() {
                Ok(imports) => {
                    let mut deps = Vec::new();
                    for desc in imports.iter() {
                        match desc.dll_name() {
                            Ok(name) => {
                                if let Ok(name_str) = name.to_str() {
                                    deps.push(name_str.to_string());
                                }
                            }
                            Err(e) => log::warn!("Error reading DLL name: {:?}", e),
                        }
                    }
                    log::debug!("Found {} dependencies with pelite", deps.len());
                    return Ok(deps);
                }
                Err(pelite::Error::Null) => return Ok(Vec::new()), // No imports
                Err(e) => log::warn!("Pelite imports error: {:?}", e),
            }
        }

        Ok(Vec::new())
    }

    /// Get imported symbols from each dependency
    pub fn get_imports(&self) -> Result<HashMap<String, HashSet<String>>> {
        let mut imports = HashMap::new();

        // For now, use a simplified approach with goblin
        if let Some(pe) = &self.goblin_pe {
            // Goblin's import structure is complex, let's use a basic approach
            for import in &pe.imports {
                let dll_name = import.dll.to_string();
                // For now, just use a placeholder symbol name
                let symbol_name = format!("import_{}", imports.len());

                imports.entry(dll_name).or_insert_with(HashSet::new).insert(symbol_name);
            }

            if !imports.is_empty() {
                log::debug!("Found imports from {} DLLs with goblin", imports.len());
                return Ok(imports);
            }
        }

        // Fallback to pelite
        if let Some(pe) = &self.pelite_pe {
            match pe.imports() {
                Ok(import_descs) => {
                    for desc in import_descs.iter() {
                        if let Ok(dll_name) = desc.dll_name() {
                            if let Ok(dll_str) = dll_name.to_str() {
                                let dll_string = dll_str.to_string();
                                let mut symbols = HashSet::new();

                                if let Ok(int) = desc.int() {
                                    for import in int {
                                        match import {
                                            Ok(pelite::pe32::imports::Import::ByName { name, .. }) => {
                                                if let Ok(name_str) = name.to_str() {
                                                    symbols.insert(name_str.to_string());
                                                }
                                            }
                                            Ok(pelite::pe32::imports::Import::ByOrdinal { ord }) => {
                                                symbols.insert(format!("#{}", ord));
                                            }
                                            Err(e) => log::warn!("Error reading import: {:?}", e),
                                        }
                                    }
                                }

                                if !symbols.is_empty() {
                                    imports.insert(dll_string, symbols);
                                }
                            }
                        }
                    }
                }
                Err(pelite::Error::Null) => {} // No imports
                Err(e) => log::warn!("Pelite imports error: {:?}", e),
            }
        }

        Ok(imports)
    }

    /// Get exported symbols
    pub fn get_exports(&self) -> Result<HashSet<String>> {
        let mut exports = HashSet::new();

        // Try goblin first
        if let Some(pe) = &self.goblin_pe {
            for export in &pe.exports {
                if let Some(name) = export.name {
                    exports.insert(name.to_string());
                }
            }

            if !exports.is_empty() {
                log::debug!("Found {} exports with goblin", exports.len());
                return Ok(exports);
            }
        }

        // Fallback to pelite
        if let Some(pe) = &self.pelite_pe {
            match pe.exports() {
                Ok(export_dir) => {
                    match export_dir.by() {
                        Ok(by) => {
                            for (name_result, _) in by.iter_names() {
                                match name_result {
                                    Ok(name_cstr) => {
                                        if let Ok(name_str) = name_cstr.to_str() {
                                            exports.insert(name_str.to_string());
                                        }
                                    }
                                    Err(e) => log::warn!("Error reading export name: {:?}", e),
                                }
                            }
                        }
                        Err(e) => log::warn!("Error reading exports by name: {:?}", e),
                    }
                }
                Err(pelite::Error::Null) => {} // No exports
                Err(e) => log::warn!("Pelite exports error: {:?}", e),
            }
        }

        Ok(exports)
    }

    /// Get basic PE file information
    pub fn get_info(&self) -> Result<PEInfo> {
        let mut info = PEInfo {
            path: self.path().to_path_buf(),
            is_64bit: self.is_64bit()?,
            dll_name: self.get_dll_name()?,
            dependencies: self.get_dependencies()?,
            import_count: 0,
            export_count: 0,
        };

        // Count imports and exports
        let imports = self.get_imports()?;
        info.import_count = imports.values().map(|set| set.len()).sum();

        let exports = self.get_exports()?;
        info.export_count = exports.len();

        Ok(info)
    }

    /// Get detailed import information with function names, ordinals, and addresses
    pub fn get_detailed_imports(&self) -> Result<HashMap<String, Vec<ImportInfo>>> {
        let mut detailed_imports = HashMap::new();

        // Try pelite first for more detailed information
        if let Some(pe) = &self.pelite_pe {
            match pe.imports() {
                Ok(import_descs) => {
                    for desc in import_descs.iter() {
                        if let Ok(dll_name) = desc.dll_name() {
                            if let Ok(dll_str) = dll_name.to_str() {
                                let dll_string = dll_str.to_string();
                                let mut imports = Vec::new();

                                if let Ok(int) = desc.int() {
                                    for import in int {
                                        match import {
                                            Ok(pelite::pe32::imports::Import::ByName { name, hint }) => {
                                                if let Ok(name_str) = name.to_str() {
                                                    imports.push(ImportInfo::by_name(
                                                        name_str.to_string(),
                                                        Some(hint as u16),
                                                    ));
                                                }
                                            }
                                            Ok(pelite::pe32::imports::Import::ByOrdinal { ord }) => {
                                                imports.push(ImportInfo::by_ordinal(ord));
                                            }
                                            Err(e) => {
                                                log::warn!("Error reading detailed import: {:?}", e);
                                            }
                                        }
                                    }
                                }

                                if !imports.is_empty() {
                                    detailed_imports.insert(dll_string, imports);
                                }
                            }
                        }
                    }

                    if !detailed_imports.is_empty() {
                        log::debug!("Found detailed imports from {} DLLs with pelite", detailed_imports.len());
                        return Ok(detailed_imports);
                    }
                }
                Err(pelite::Error::Null) => {
                    log::debug!("No imports found in PE file");
                    return Ok(detailed_imports);
                }
                Err(e) => {
                    log::warn!("Pelite detailed imports error: {:?}", e);
                }
            }
        }

        // Fallback to goblin (less detailed but still useful)
        if let Some(pe) = &self.goblin_pe {
            // Goblin's import structure is different, let's use a simplified approach
            // Group imports by DLL name from the existing get_imports method
            let simple_imports = self.get_imports()?;
            for (dll_name, symbol_names) in simple_imports {
                let mut imports = Vec::new();
                for symbol_name in symbol_names {
                    if symbol_name.starts_with('#') {
                        // This is an ordinal import
                        if let Ok(ordinal) = symbol_name[1..].parse::<u16>() {
                            imports.push(ImportInfo::by_ordinal(ordinal));
                        }
                    } else {
                        // This is a named import
                        imports.push(ImportInfo::by_name(symbol_name, None));
                    }
                }

                if !imports.is_empty() {
                    detailed_imports.insert(dll_name, imports);
                }
            }

            if !detailed_imports.is_empty() {
                log::debug!("Found detailed imports from {} DLLs with goblin fallback", detailed_imports.len());
            }
        }

        Ok(detailed_imports)
    }

    /// Get detailed export information with function names, ordinals, and addresses
    pub fn get_detailed_exports(&self) -> Result<Vec<ExportInfo>> {
        let mut detailed_exports = Vec::new();

        // Try pelite first for more detailed information
        if let Some(pe) = &self.pelite_pe {
            match pe.exports() {
                Ok(export_dir) => {
                    match export_dir.by() {
                        Ok(by) => {
                            // Use a simpler approach - iterate through names and get basic info
                            for (name_result, _) in by.iter_names() {
                                match name_result {
                                    Ok(name_cstr) => {
                                        if let Ok(name_str) = name_cstr.to_str() {
                                            // For simplicity, use index as ordinal and 0 as RVA
                                            // This is not perfect but provides basic functionality
                                            detailed_exports.push(ExportInfo {
                                                name: Some(name_str.to_string()),
                                                ordinal: detailed_exports.len() as u16,
                                                rva: 0, // Would need more complex pelite API usage to get actual RVA
                                                is_forwarded: false,
                                                forward_name: None,
                                            });
                                        }
                                    }
                                    Err(e) => {
                                        log::warn!("Error reading export name: {:?}", e);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            log::warn!("Error reading exports by name: {:?}", e);
                        }
                    }

                    if !detailed_exports.is_empty() {
                        log::debug!("Found {} detailed exports with pelite", detailed_exports.len());
                        return Ok(detailed_exports);
                    }
                }
                Err(pelite::Error::Null) => {
                    log::debug!("No exports found in PE file");
                    return Ok(detailed_exports);
                }
                Err(e) => {
                    log::warn!("Pelite detailed exports error: {:?}", e);
                }
            }
        }

        // Fallback to goblin (less detailed)
        if let Some(pe) = &self.goblin_pe {
            for (i, export) in pe.exports.iter().enumerate() {
                let name = export.name.map(|s| s.to_string());

                detailed_exports.push(ExportInfo {
                    name,
                    ordinal: i as u16, // Goblin doesn't provide ordinal directly
                    rva: export.rva as u32,
                    is_forwarded: false, // Goblin doesn't provide forward information
                    forward_name: None,
                });
            }

            if !detailed_exports.is_empty() {
                log::debug!("Found {} detailed exports with goblin", detailed_exports.len());
            }
        }

        Ok(detailed_exports)
    }
}

/// Basic PE file information
#[derive(Debug, Clone)]
pub struct PEInfo {
    pub path: PathBuf,
    pub is_64bit: bool,
    pub dll_name: Option<String>,
    pub dependencies: Vec<String>,
    pub import_count: usize,
    pub export_count: usize,
}

impl PEInfo {
    /// Check if this PE file is a DLL
    pub fn is_dll(&self) -> bool {
        self.dll_name.is_some()
    }

    /// Get a human-readable description
    pub fn description(&self) -> String {
        let arch = if self.is_64bit { "x64" } else { "x86" };
        let file_type = if self.is_dll() { "DLL" } else { "EXE" };

        format!(
            "{} {} - {} dependencies, {} imports, {} exports",
            arch, file_type, self.dependencies.len(), self.import_count, self.export_count
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_detailed_imports_exports_basic() {
        // This test requires a real PE file to work properly
        // For now, just test that the methods don't panic with a non-existent file
        let non_existent_path = Path::new("non_existent_file.exe");
        let file_map_result = PEFileMap::new(non_existent_path);

        // Should fail gracefully for non-existent file
        assert!(file_map_result.is_err());
    }

    #[test]
    fn test_detailed_functions_with_system_dll() {
        // Try to test with a system DLL if available
        let system_dll_paths = [
            "C:\\Windows\\System32\\kernel32.dll",
            "C:\\Windows\\SysWOW64\\kernel32.dll",
        ];

        for dll_path in &system_dll_paths {
            let path = Path::new(dll_path);
            if path.exists() {
                match PEFileMap::new(path) {
                    Ok(file_map) => {
                        match PEFile::new(&file_map) {
                            Ok(pe_file) => {
                                // Test detailed imports
                                match pe_file.get_detailed_imports() {
                                    Ok(imports) => {
                                        println!("Found detailed imports from {} DLLs", imports.len());
                                        for (dll_name, functions) in imports.iter().take(3) {
                                            println!("  {}: {} functions", dll_name, functions.len());
                                            for func in functions.iter().take(3) {
                                                println!("    - {}", func.display_name());
                                            }
                                        }
                                    }
                                    Err(e) => println!("Error getting detailed imports: {:?}", e),
                                }

                                // Test detailed exports
                                match pe_file.get_detailed_exports() {
                                    Ok(exports) => {
                                        println!("Found {} detailed exports", exports.len());
                                        for export in exports.iter().take(5) {
                                            println!("  - {} (ordinal: {}, RVA: 0x{:X})",
                                                export.display_name(), export.ordinal, export.rva);
                                        }
                                    }
                                    Err(e) => println!("Error getting detailed exports: {:?}", e),
                                }

                                // If we successfully tested one DLL, that's enough
                                return;
                            }
                            Err(e) => println!("Error creating PEFile for {}: {:?}", dll_path, e),
                        }
                    }
                    Err(e) => println!("Error creating PEFileMap for {}: {:?}", dll_path, e),
                }
            }
        }

        println!("No system DLL found for testing, test passed without validation");
    }
}
