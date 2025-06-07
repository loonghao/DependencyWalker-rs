//! Error types and handling for DependencyWalker RS

use std::path::PathBuf;
use thiserror::Error;

/// Main error type for DependencyWalker RS
#[derive(Error, Debug)]
pub enum Error {
    /// IO related errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// PE file parsing errors
    #[error("PE parsing error: {0}")]
    PeError(String),

    /// Pelite library errors
    #[error("Pelite error: {0}")]
    Pelite(#[from] pelite::Error),

    /// Goblin library errors
    #[error("Goblin error: {0}")]
    Goblin(#[from] goblin::error::Error),

    /// Windows API errors (暂时注释掉)
    // #[error("Windows API error: {0}")]
    // WindowsApi(#[from] windows::core::Error),

    /// File not found
    #[error("File not found: {path}")]
    FileNotFound { path: PathBuf },

    /// Invalid file format
    #[error("Invalid file format: {path} - {reason}")]
    InvalidFormat { path: PathBuf, reason: String },

    /// Dependency resolution errors
    #[error("Dependency resolution error: {0}")]
    DependencyResolution(String),

    /// Circular dependency detected
    #[error("Circular dependency detected: {chain}")]
    CircularDependency { chain: String },

    /// Symbol resolution errors
    #[error("Symbol resolution error: {symbol} in {dll}")]
    SymbolResolution { symbol: String, dll: String },

    /// API Set redirection errors
    #[error("API Set redirection error: {0}")]
    ApiSetRedirection(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Generic error with context
    #[error("Error: {message}")]
    Generic { message: String },
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Create a new PE parsing error
    pub fn pe_error<S: Into<String>>(message: S) -> Self {
        Error::PeError(message.into())
    }

    /// Create a new dependency resolution error
    pub fn dependency_error<S: Into<String>>(message: S) -> Self {
        Error::DependencyResolution(message.into())
    }

    /// Create a new symbol resolution error
    pub fn symbol_error<S: Into<String>>(symbol: S, dll: S) -> Self {
        Error::SymbolResolution {
            symbol: symbol.into(),
            dll: dll.into(),
        }
    }

    /// Create a new generic error
    pub fn generic<S: Into<String>>(message: S) -> Self {
        Error::Generic {
            message: message.into(),
        }
    }

    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            Error::FileNotFound { .. } => false,
            Error::InvalidFormat { .. } => false,
            Error::CircularDependency { .. } => false,
            Error::PeError(_) => true,
            Error::DependencyResolution(_) => true,
            Error::SymbolResolution { .. } => true,
            _ => false,
        }
    }
}

/// Convert anyhow::Error to our Error type
impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::Generic {
            message: err.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use std::path::Path; // Unused import

    #[test]
    fn test_error_creation() {
        let err = Error::pe_error("test error");
        assert!(matches!(err, Error::PeError(_)));
    }

    #[test]
    fn test_file_not_found() {
        let path = PathBuf::from("nonexistent.exe");
        let err = Error::FileNotFound { path: path.clone() };
        assert_eq!(
            format!("{}", err),
            format!("File not found: {}", path.display())
        );
    }

    #[test]
    fn test_recoverable() {
        let recoverable = Error::pe_error("test");
        let non_recoverable = Error::FileNotFound {
            path: PathBuf::from("test"),
        };

        assert!(recoverable.is_recoverable());
        assert!(!non_recoverable.is_recoverable());
    }
}
