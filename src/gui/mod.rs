//! GUI module for DependencyWalker RS
//!
//! This module provides the graphical user interface using both ICED and Slint frameworks.

pub mod app;
pub mod components;
pub mod message;
pub mod style;

// Slint GUI module
#[cfg(feature = "gui")]
pub mod slint_app;

// Re-export commonly used types
pub use app::DependencyWalkerApp;
pub use message::Message;

#[cfg(feature = "gui")]
pub use slint_app::SlintApp;
