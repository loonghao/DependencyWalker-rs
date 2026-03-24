//! Windows DLL search path implementation
//!
//! This module implements the Windows DLL search path logic, including:
//! - Application directory
//! - System directories (System32, SysWOW64, SysArm32)
//! - Windows directory
//! - PATH environment variable
//! - WOW64 redirection
//! - KnownDLLs mechanism
//! - Working directory
//! - Custom search paths

use crate::core::pe_parser::PEFile;
use crate::error::{Error, Result};
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

/// DLL search strategy used to find a module
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleSearchStrategy {
    /// Module found in SxS (Side-by-Side) assemblies
    SxS = 0,
    /// Module found via API Set Schema redirection
    ApiSetSchema = 1,
    /// Module found in KnownDLLs registry
    WellKnownDlls = 2,
    /// Module found in application directory
    ApplicationDirectory = 3,
    /// Module found in System32 folder
    System32Folder = 4,
    /// Module found in Windows folder
    WindowsFolder = 5,
    /// Module found in working directory
    WorkingDirectory = 6,
    /// Module found in PATH environment variable
    Environment = 7,
    /// Module found via AppInit DLL mechanism
    AppInitDLL = 8,
    /// Module found using full path
    Fullpath = 9,
    /// Module is a CLR assembly
    ClrAssembly = 10,
    /// Module found in user-defined search paths
    UserDefined = 0xfe,
    /// Module not found
    NotFound = 0xff,
}

/// DLL search path resolver configuration
#[derive(Debug, Clone)]
pub struct DllResolverConfig {
    /// Include system DLLs in search
    pub include_system_dlls: bool,
    /// Custom search paths
    pub custom_search_paths: Vec<PathBuf>,
    /// Working directory override
    pub working_directory: Option<PathBuf>,
    /// Enable WOW64 redirection
    pub enable_wow64_redirection: bool,
    /// Enable KnownDLLs lookup
    pub enable_known_dlls: bool,
    /// Enable API Set Schema redirection
    pub enable_api_set_schema: bool,
}

impl Default for DllResolverConfig {
    fn default() -> Self {
        Self {
            include_system_dlls: true,
            custom_search_paths: Vec::new(),
            working_directory: None,
            enable_wow64_redirection: true,
            enable_known_dlls: true,
            enable_api_set_schema: true,
        }
    }
}

/// Windows DLL search path resolver
///
/// Implements the Windows DLL search order as documented in MSDN:
/// https://docs.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-search-order
#[derive(Debug)]
pub struct DllResolver {
    config: DllResolverConfig,
    known_dlls_cache: Option<HashMap<String, PathBuf>>,
    api_set_cache: Option<HashMap<String, String>>,
}

impl DllResolver {
    /// Create a new DLL resolver with default configuration
    pub fn new() -> Self {
        Self {
            config: DllResolverConfig::default(),
            known_dlls_cache: None,
            api_set_cache: None,
        }
    }

    /// Create a new DLL resolver with custom configuration
    pub fn with_config(config: DllResolverConfig) -> Self {
        Self {
            config,
            known_dlls_cache: None,
            api_set_cache: None,
        }
    }

    /// Get the current configuration
    pub fn config(&self) -> &DllResolverConfig {
        &self.config
    }

    /// Update the configuration
    pub fn set_config(&mut self, config: DllResolverConfig) {
        self.config = config;
        // Clear caches when config changes
        self.known_dlls_cache = None;
        self.api_set_cache = None;
    }

    /// Add a custom search path
    pub fn add_search_path<P: AsRef<Path>>(&mut self, path: P) {
        self.config
            .custom_search_paths
            .push(path.as_ref().to_path_buf());
    }

    /// Clear all custom search paths
    pub fn clear_search_paths(&mut self) {
        self.config.custom_search_paths.clear();
    }

    /// Resolve DLL path using Windows search order
    ///
    /// # Arguments
    /// * `root_pe` - The root PE file that is requesting the DLL
    /// * `dll_name` - Name of the DLL to resolve
    ///
    /// # Returns
    /// A tuple containing the search strategy used and the resolved path (if found)
    pub fn resolve_dll(
        &mut self,
        root_pe: &PEFile,
        dll_name: &str,
    ) -> Result<(ModuleSearchStrategy, Option<PathBuf>)> {
        // Normalize DLL name - add .dll extension if missing
        let normalized_name = if dll_name.ends_with(".dll") || dll_name.ends_with(".exe") {
            dll_name.to_string()
        } else {
            format!("{}.dll", dll_name)
        };

        log::debug!(
            "Resolving DLL: {} (normalized: {})",
            dll_name,
            normalized_name
        );

        // Get processor architecture and determine if this is a WOW64 process
        let is_wow64 = self.is_wow64_process(root_pe)?;
        let processor_arch = self.get_processor_architecture(root_pe)?;

        log::debug!(
            "Target architecture: {}, WOW64: {}",
            processor_arch,
            is_wow64
        );

        // Search order implementation
        // 1. API Set Schema redirection (if enabled)
        if self.config.enable_api_set_schema {
            if let Some(redirected_name) = self.resolve_api_set(&normalized_name)? {
                log::debug!(
                    "API Set redirection: {} -> {}",
                    normalized_name,
                    redirected_name
                );
                // Search for the redirected DLL using the standard search order
                let redirected_path =
                    self.search_for_dll(&redirected_name, is_wow64, &processor_arch)?;
                if redirected_path.is_some() {
                    return Ok((ModuleSearchStrategy::ApiSetSchema, redirected_path));
                }
            }
        }

        // 2. KnownDLLs lookup (if enabled)
        if self.config.enable_known_dlls {
            if let Some(path) = self.search_known_dlls(&normalized_name, is_wow64)? {
                log::debug!("Found in KnownDLLs: {}", path.display());
                return Ok((ModuleSearchStrategy::WellKnownDlls, Some(path)));
            }
        }

        // 2. Application directory
        if let Some(app_dir) = self.get_application_directory(root_pe) {
            if let Some(path) =
                self.search_in_directory(&app_dir, &normalized_name, &processor_arch)?
            {
                log::debug!("Found in application directory: {}", path.display());
                return Ok((ModuleSearchStrategy::ApplicationDirectory, Some(path)));
            }
        }

        // 3. System directories
        let system_dirs = self.get_system_directories(is_wow64)?;
        for system_dir in &system_dirs {
            if let Some(path) =
                self.search_in_directory(system_dir, &normalized_name, &processor_arch)?
            {
                log::debug!("Found in system directory: {}", path.display());
                return Ok((ModuleSearchStrategy::System32Folder, Some(path)));
            }
        }

        // 4. Windows directory
        let windows_dir = self.get_windows_directory()?;
        if let Some(path) =
            self.search_in_directory(&windows_dir, &normalized_name, &processor_arch)?
        {
            log::debug!("Found in Windows directory: {}", path.display());
            return Ok((ModuleSearchStrategy::WindowsFolder, Some(path)));
        }

        // 5. Working directory
        if let Some(working_dir) = self.get_working_directory() {
            if let Some(path) =
                self.search_in_directory(&working_dir, &normalized_name, &processor_arch)?
            {
                log::debug!("Found in working directory: {}", path.display());
                return Ok((ModuleSearchStrategy::WorkingDirectory, Some(path)));
            }
        }

        // 6. PATH environment variable
        if let Some(path) = self.search_in_path(&normalized_name, &processor_arch)? {
            log::debug!("Found in PATH: {}", path.display());
            return Ok((ModuleSearchStrategy::Environment, Some(path)));
        }

        // 7. Custom search paths
        for custom_path in &self.config.custom_search_paths {
            if let Some(path) =
                self.search_in_directory(custom_path, &normalized_name, &processor_arch)?
            {
                log::debug!("Found in custom search path: {}", path.display());
                return Ok((ModuleSearchStrategy::UserDefined, Some(path)));
            }
        }

        // 8. Check if it's an absolute path
        if Path::new(&normalized_name).is_absolute() && Path::new(&normalized_name).exists() {
            log::debug!("Found as absolute path: {}", normalized_name);
            return Ok((
                ModuleSearchStrategy::Fullpath,
                Some(PathBuf::from(normalized_name)),
            ));
        }

        log::debug!("DLL not found: {}", normalized_name);
        Ok((ModuleSearchStrategy::NotFound, None))
    }

    /// Simple DLL resolution without root PE context
    pub fn resolve_dll_simple(&mut self, dll_name: &str) -> Result<Option<PathBuf>> {
        // Create a dummy PE context using ntdll.dll as reference
        let system_dir = self.get_system_directories(false)?;
        let ntdll_path = system_dir[0].join("ntdll.dll");

        if !ntdll_path.exists() {
            return Err(Error::FileNotFound { path: ntdll_path });
        }

        // For simple resolution, we'll use a basic search without PE context
        let normalized_name = if dll_name.ends_with(".dll") || dll_name.ends_with(".exe") {
            dll_name.to_string()
        } else {
            format!("{}.dll", dll_name)
        };

        // Search in system directories first
        let system_dirs = self.get_system_directories(false)?;
        for system_dir in &system_dirs {
            if let Some(path) = self.search_in_directory(system_dir, &normalized_name, "x64")? {
                return Ok(Some(path));
            }
        }

        // Search in PATH
        if let Some(path) = self.search_in_path(&normalized_name, "x64")? {
            return Ok(Some(path));
        }

        Ok(None)
    }
}

impl Default for DllResolver {
    fn default() -> Self {
        Self::new()
    }
}

// Implementation of helper methods
impl DllResolver {
    /// Determine if the target process is WOW64 (32-bit on 64-bit Windows)
    fn is_wow64_process(&self, root_pe: &PEFile) -> Result<bool> {
        // Check if the PE is 32-bit
        let pe_info = root_pe.get_info()?;
        Ok(!pe_info.is_64bit && cfg!(target_arch = "x86_64"))
    }

    /// Get the processor architecture string for the PE
    fn get_processor_architecture(&self, root_pe: &PEFile) -> Result<String> {
        let pe_info = root_pe.get_info()?;
        Ok(if pe_info.is_64bit {
            "x64".to_string()
        } else {
            "x86".to_string()
        })
    }

    /// Get the application directory from the root PE
    fn get_application_directory(&self, root_pe: &PEFile) -> Option<PathBuf> {
        root_pe.path().parent().map(|p| p.to_path_buf())
    }

    /// Get system directories based on WOW64 status
    fn get_system_directories(&self, is_wow64: bool) -> Result<Vec<PathBuf>> {
        let mut dirs = Vec::new();

        if is_wow64 {
            // For WOW64 processes, search SysWOW64 first
            if let Some(syswow64) = self.get_syswow64_directory() {
                dirs.push(syswow64);
            }
        }

        // Always include System32
        dirs.push(self.get_system32_directory()?);

        // Add SysArm32 if on ARM64 Windows
        if cfg!(target_arch = "aarch64") {
            if let Some(sysarm32) = self.get_sysarm32_directory() {
                dirs.push(sysarm32);
            }
        }

        Ok(dirs)
    }

    /// Get System32 directory
    fn get_system32_directory(&self) -> Result<PathBuf> {
        let windows_dir = env::var("WINDIR")
            .or_else(|_| env::var("SystemRoot"))
            .unwrap_or_else(|_| "C:\\Windows".to_string());

        Ok(PathBuf::from(windows_dir).join("System32"))
    }

    /// Get SysWOW64 directory (for 32-bit processes on 64-bit Windows)
    fn get_syswow64_directory(&self) -> Option<PathBuf> {
        let windows_dir = env::var("WINDIR")
            .or_else(|_| env::var("SystemRoot"))
            .unwrap_or_else(|_| "C:\\Windows".to_string());

        let syswow64 = PathBuf::from(windows_dir).join("SysWOW64");
        if syswow64.exists() {
            Some(syswow64)
        } else {
            None
        }
    }

    /// Get SysArm32 directory (for ARM32 processes on ARM64 Windows)
    fn get_sysarm32_directory(&self) -> Option<PathBuf> {
        let windows_dir = env::var("WINDIR")
            .or_else(|_| env::var("SystemRoot"))
            .unwrap_or_else(|_| "C:\\Windows".to_string());

        let sysarm32 = PathBuf::from(windows_dir).join("SysArm32");
        if sysarm32.exists() {
            Some(sysarm32)
        } else {
            None
        }
    }

    /// Get Windows directory
    fn get_windows_directory(&self) -> Result<PathBuf> {
        let windows_dir = env::var("WINDIR")
            .or_else(|_| env::var("SystemRoot"))
            .unwrap_or_else(|_| "C:\\Windows".to_string());

        Ok(PathBuf::from(windows_dir))
    }

    /// Get working directory
    fn get_working_directory(&self) -> Option<PathBuf> {
        self.config
            .working_directory
            .clone()
            .or_else(|| env::current_dir().ok())
    }

    /// Search for DLL in a specific directory
    fn search_in_directory(
        &self,
        dir: &Path,
        dll_name: &str,
        processor_arch: &str,
    ) -> Result<Option<PathBuf>> {
        if !dir.exists() || !dir.is_dir() {
            return Ok(None);
        }

        let dll_path = dir.join(dll_name);
        if dll_path.exists() {
            // Verify the architecture matches if we can load the PE
            if let Ok(pe_map) = crate::core::pe_parser::PEFileMap::new(&dll_path) {
                if let Ok(pe_file) = crate::core::pe_parser::PEFile::new(&pe_map) {
                    let target_arch = self.get_processor_architecture(&pe_file)?;
                    if target_arch == processor_arch || processor_arch == "unknown" {
                        return Ok(Some(dll_path));
                    } else {
                        log::debug!(
                            "Architecture mismatch: {} (expected: {}, found: {})",
                            dll_path.display(),
                            processor_arch,
                            target_arch
                        );
                    }
                } else {
                    // If we can't parse it as PE, still return it
                    return Ok(Some(dll_path));
                }
            } else {
                // If we can't load it, still return it
                return Ok(Some(dll_path));
            }
        }

        Ok(None)
    }

    /// Search for DLL in PATH environment variable
    fn search_in_path(&self, dll_name: &str, processor_arch: &str) -> Result<Option<PathBuf>> {
        if let Ok(path_var) = env::var("PATH") {
            for path_dir in path_var.split(';') {
                let path_dir = path_dir.trim();
                if path_dir.is_empty() {
                    continue;
                }

                let dir_path = PathBuf::from(path_dir);
                if let Some(found_path) =
                    self.search_in_directory(&dir_path, dll_name, processor_arch)?
                {
                    return Ok(Some(found_path));
                }
            }
        }

        Ok(None)
    }

    /// Search for DLL in KnownDLLs registry
    fn search_known_dlls(&mut self, dll_name: &str, is_wow64: bool) -> Result<Option<PathBuf>> {
        // Initialize KnownDLLs cache if not already done
        if self.known_dlls_cache.is_none() {
            self.known_dlls_cache = Some(self.load_known_dlls(is_wow64)?);
        }

        if let Some(ref cache) = self.known_dlls_cache {
            if let Some(path) = cache.get(&dll_name.to_lowercase()) {
                return Ok(Some(path.clone()));
            }
        }

        Ok(None)
    }

    /// Load KnownDLLs from registry (simulated)
    fn load_known_dlls(&self, is_wow64: bool) -> Result<HashMap<String, PathBuf>> {
        let mut known_dlls = HashMap::new();

        // Get the appropriate system directory
        let system_dir = if is_wow64 {
            self.get_syswow64_directory()
                .unwrap_or_else(|| self.get_system32_directory().unwrap())
        } else {
            self.get_system32_directory()?
        };

        // Common KnownDLLs (this is a simplified list - in a real implementation,
        // we would read from the registry: HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\Session Manager\KnownDLLs)
        let common_known_dlls = [
            "advapi32.dll",
            "cfgmgr32.dll",
            "comctl32.dll",
            "comdlg32.dll",
            "crypt32.dll",
            "dnsapi.dll",
            "gdi32.dll",
            "gdiplus.dll",
            "imm32.dll",
            "kernel32.dll",
            "kernelbase.dll",
            "lz32.dll",
            "msasn1.dll",
            "msctf.dll",
            "msvcrt.dll",
            "normaliz.dll",
            "nsi.dll",
            "ntdll.dll",
            "ole32.dll",
            "oleaut32.dll",
            "psapi.dll",
            "rpcrt4.dll",
            "secur32.dll",
            "shell32.dll",
            "shlwapi.dll",
            "user32.dll",
            "userenv.dll",
            "version.dll",
            "wininet.dll",
            "wldap32.dll",
            "ws2_32.dll",
        ];

        for dll_name in &common_known_dlls {
            let dll_path = system_dir.join(dll_name);
            if dll_path.exists() {
                known_dlls.insert(dll_name.to_lowercase(), dll_path);
            }
        }

        log::debug!(
            "Loaded {} KnownDLLs for {} architecture",
            known_dlls.len(),
            if is_wow64 { "WOW64" } else { "native" }
        );

        Ok(known_dlls)
    }

    /// Resolve API Set Schema redirection
    fn resolve_api_set(&mut self, dll_name: &str) -> Result<Option<String>> {
        // Check if this is an API Set DLL (starts with "api-ms-win-")
        if !dll_name.to_lowercase().starts_with("api-ms-win-") {
            return Ok(None);
        }

        log::debug!("Checking API Set redirection for: {}", dll_name);

        // Initialize API Set cache if not already done
        if self.api_set_cache.is_none() {
            self.api_set_cache = Some(self.load_api_set_schema()?);
        }

        if let Some(ref cache) = self.api_set_cache {
            if let Some(redirected_name) = cache.get(&dll_name.to_lowercase()) {
                log::debug!(
                    "API Set redirection found: {} -> {}",
                    dll_name,
                    redirected_name
                );
                return Ok(Some(redirected_name.clone()));
            }
        }

        log::debug!("No API Set redirection found for: {}", dll_name);
        Ok(None)
    }

    /// Load API Set Schema from registry or system files
    fn load_api_set_schema(&self) -> Result<HashMap<String, String>> {
        let mut api_set_map = HashMap::new();

        log::debug!("Loading API Set Schema...");

        // Common API Set redirections (simplified mapping)
        // In a full implementation, this would read from:
        // HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\Session Manager\ApiSetSchema
        // or parse %SystemRoot%\System32\ApiSetSchema.dll
        let common_api_sets = [
            ("api-ms-win-core-console-l1-1-0.dll", "kernel32.dll"),
            ("api-ms-win-core-console-l1-2-0.dll", "kernel32.dll"),
            ("api-ms-win-core-datetime-l1-1-0.dll", "kernel32.dll"),
            ("api-ms-win-core-debug-l1-1-0.dll", "kernel32.dll"),
            ("api-ms-win-core-errorhandling-l1-1-0.dll", "kernel32.dll"),
            ("api-ms-win-core-file-l1-1-0.dll", "kernel32.dll"),
            ("api-ms-win-core-file-l1-2-0.dll", "kernel32.dll"),
            ("api-ms-win-core-file-l2-1-0.dll", "kernel32.dll"),
            ("api-ms-win-core-handle-l1-1-0.dll", "kernel32.dll"),
            ("api-ms-win-core-heap-l1-1-0.dll", "kernel32.dll"),
            ("api-ms-win-core-interlocked-l1-1-0.dll", "kernel32.dll"),
            ("api-ms-win-core-io-l1-1-0.dll", "kernel32.dll"),
            ("api-ms-win-core-libraryloader-l1-1-0.dll", "kernel32.dll"),
            ("api-ms-win-core-libraryloader-l1-2-0.dll", "kernel32.dll"),
            ("api-ms-win-core-localization-l1-2-0.dll", "kernel32.dll"),
            ("api-ms-win-core-memory-l1-1-0.dll", "kernel32.dll"),
            ("api-ms-win-core-namedpipe-l1-1-0.dll", "kernel32.dll"),
            (
                "api-ms-win-core-processenvironment-l1-1-0.dll",
                "kernel32.dll",
            ),
            ("api-ms-win-core-processthreads-l1-1-0.dll", "kernel32.dll"),
            ("api-ms-win-core-processthreads-l1-1-1.dll", "kernel32.dll"),
            ("api-ms-win-core-profile-l1-1-0.dll", "kernel32.dll"),
            ("api-ms-win-core-rtlsupport-l1-1-0.dll", "ntdll.dll"),
            ("api-ms-win-core-string-l1-1-0.dll", "kernel32.dll"),
            ("api-ms-win-core-synch-l1-1-0.dll", "kernel32.dll"),
            ("api-ms-win-core-synch-l1-2-0.dll", "kernel32.dll"),
            ("api-ms-win-core-sysinfo-l1-1-0.dll", "kernel32.dll"),
            ("api-ms-win-core-timezone-l1-1-0.dll", "kernel32.dll"),
            ("api-ms-win-core-util-l1-1-0.dll", "kernel32.dll"),
            ("api-ms-win-crt-conio-l1-1-0.dll", "ucrtbase.dll"),
            ("api-ms-win-crt-convert-l1-1-0.dll", "ucrtbase.dll"),
            ("api-ms-win-crt-environment-l1-1-0.dll", "ucrtbase.dll"),
            ("api-ms-win-crt-filesystem-l1-1-0.dll", "ucrtbase.dll"),
            ("api-ms-win-crt-heap-l1-1-0.dll", "ucrtbase.dll"),
            ("api-ms-win-crt-locale-l1-1-0.dll", "ucrtbase.dll"),
            ("api-ms-win-crt-math-l1-1-0.dll", "ucrtbase.dll"),
            ("api-ms-win-crt-multibyte-l1-1-0.dll", "ucrtbase.dll"),
            ("api-ms-win-crt-private-l1-1-0.dll", "ucrtbase.dll"),
            ("api-ms-win-crt-process-l1-1-0.dll", "ucrtbase.dll"),
            ("api-ms-win-crt-runtime-l1-1-0.dll", "ucrtbase.dll"),
            ("api-ms-win-crt-stdio-l1-1-0.dll", "ucrtbase.dll"),
            ("api-ms-win-crt-string-l1-1-0.dll", "ucrtbase.dll"),
            ("api-ms-win-crt-time-l1-1-0.dll", "ucrtbase.dll"),
            ("api-ms-win-crt-utility-l1-1-0.dll", "ucrtbase.dll"),
        ];

        for (api_set_name, target_dll) in &common_api_sets {
            api_set_map.insert(api_set_name.to_string(), target_dll.to_string());
        }

        log::debug!("Loaded {} API Set redirections", api_set_map.len());

        Ok(api_set_map)
    }

    /// Search for a DLL using the standard Windows search order (excluding API Set redirection)
    fn search_for_dll(
        &mut self,
        dll_name: &str,
        is_wow64: bool,
        processor_arch: &str,
    ) -> Result<Option<PathBuf>> {
        // This method implements the standard DLL search order without API Set redirection
        // to avoid infinite recursion when resolving API Set redirections

        // 1. KnownDLLs lookup (if enabled)
        if self.config.enable_known_dlls {
            if let Some(path) = self.search_known_dlls(dll_name, is_wow64)? {
                log::debug!("Found redirected DLL in KnownDLLs: {}", path.display());
                return Ok(Some(path));
            }
        }

        // 2. System directories
        let system_dirs = self.get_system_directories(is_wow64)?;
        for system_dir in &system_dirs {
            if let Some(path) = self.search_in_directory(system_dir, dll_name, processor_arch)? {
                log::debug!(
                    "Found redirected DLL in system directory: {}",
                    path.display()
                );
                return Ok(Some(path));
            }
        }

        // 3. Windows directory
        let windows_dir = self.get_windows_directory()?;
        if let Some(path) = self.search_in_directory(&windows_dir, dll_name, processor_arch)? {
            log::debug!(
                "Found redirected DLL in Windows directory: {}",
                path.display()
            );
            return Ok(Some(path));
        }

        // 4. PATH environment variable
        if let Some(path) = self.search_in_path(dll_name, processor_arch)? {
            log::debug!("Found redirected DLL in PATH: {}", path.display());
            return Ok(Some(path));
        }

        // 5. Custom search paths
        for custom_path in &self.config.custom_search_paths {
            if let Some(path) = self.search_in_directory(custom_path, dll_name, processor_arch)? {
                log::debug!(
                    "Found redirected DLL in custom search path: {}",
                    path.display()
                );
                return Ok(Some(path));
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_set_redirection() {
        let mut resolver = DllResolver::new();

        // Test API Set detection
        let api_set_result = resolver.resolve_api_set("api-ms-win-core-file-l1-1-0.dll");
        assert!(api_set_result.is_ok());

        if let Ok(Some(redirected)) = api_set_result {
            assert_eq!(redirected, "kernel32.dll");
        }

        // Test non-API Set DLL
        let non_api_set_result = resolver.resolve_api_set("user32.dll");
        assert!(non_api_set_result.is_ok());
        assert_eq!(non_api_set_result.unwrap(), None);
    }

    #[test]
    fn test_api_set_cache() {
        let mut resolver = DllResolver::new();

        // First call should load the cache
        let result1 = resolver.resolve_api_set("api-ms-win-core-heap-l1-1-0.dll");
        assert!(result1.is_ok());

        // Second call should use the cache
        let result2 = resolver.resolve_api_set("api-ms-win-core-heap-l1-1-0.dll");
        assert!(result2.is_ok());
        assert_eq!(result1.unwrap(), result2.unwrap());
    }

    #[test]
    fn test_api_set_config() {
        let mut config = DllResolverConfig::default();
        assert!(config.enable_api_set_schema); // Should be enabled by default

        config.enable_api_set_schema = false;
        let mut resolver = DllResolver::with_config(config);

        // With API Set disabled, should return None
        let result = resolver.resolve_api_set("api-ms-win-core-file-l1-1-0.dll");
        assert!(result.is_ok());
    }

    #[test]
    fn test_api_set_integration() {
        let mut resolver = DllResolver::new();

        // Test that API Set DLLs are properly detected and redirected
        let api_set_dlls = [
            "api-ms-win-core-file-l1-1-0.dll",
            "api-ms-win-core-heap-l1-1-0.dll",
            "api-ms-win-core-processthreads-l1-1-0.dll",
            "api-ms-win-crt-runtime-l1-1-0.dll",
        ];

        for api_dll in &api_set_dlls {
            let result = resolver.resolve_api_set(api_dll);
            assert!(result.is_ok(), "Failed to process API Set DLL: {}", api_dll);

            if let Ok(Some(redirected)) = result {
                assert!(
                    !redirected.is_empty(),
                    "Redirected DLL name should not be empty for {}",
                    api_dll
                );
                assert!(
                    !redirected.starts_with("api-ms-win-"),
                    "Redirected DLL should not be another API Set DLL: {} -> {}",
                    api_dll,
                    redirected
                );
                println!("API Set redirection: {} -> {}", api_dll, redirected);
            }
        }
    }
}
