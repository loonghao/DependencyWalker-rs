//! Modern style definitions for the ICED GUI application
//!
//! This module defines modern themes, styling, and visual components for the application.

use iced::{Color, Theme};

/// Modern application theme with enhanced visual design
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppTheme {
    Light,
    Dark,
    Modern,
    HighContrast,
}

impl Default for AppTheme {
    fn default() -> Self {
        Self::Modern
    }
}

impl AppTheme {
    pub fn to_iced_theme(self) -> Theme {
        match self {
            AppTheme::Light => Theme::Light,
            AppTheme::Dark => Theme::Dark,
            AppTheme::Modern => Theme::Dark, // Use dark as base for modern theme
            AppTheme::HighContrast => Theme::Dark,
        }
    }

    pub fn toggle(self) -> Self {
        match self {
            AppTheme::Light => AppTheme::Dark,
            AppTheme::Dark => AppTheme::Modern,
            AppTheme::Modern => AppTheme::HighContrast,
            AppTheme::HighContrast => AppTheme::Light,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            AppTheme::Light => "Light",
            AppTheme::Dark => "Dark",
            AppTheme::Modern => "Modern",
            AppTheme::HighContrast => "High Contrast",
        }
    }
}

/// Modern color palette for the application
pub struct Colors;

impl Colors {
    // Modern primary colors with better contrast
    pub const PRIMARY: Color = Color::from_rgb(0.0, 0.48, 1.0); // Modern blue
    pub const PRIMARY_HOVER: Color = Color::from_rgb(0.0, 0.42, 0.87); // Darker blue
    pub const SECONDARY: Color = Color::from_rgb(0.38, 0.38, 0.38); // Modern gray
    pub const ACCENT: Color = Color::from_rgb(0.67, 0.33, 1.0); // Purple accent

    // Background colors for modern design
    pub const BACKGROUND_PRIMARY: Color = Color::from_rgb(0.08, 0.08, 0.08); // Dark background
    pub const BACKGROUND_SECONDARY: Color = Color::from_rgb(0.12, 0.12, 0.12); // Card background
    pub const BACKGROUND_TERTIARY: Color = Color::from_rgb(0.16, 0.16, 0.16); // Elevated background

    // Text colors
    pub const TEXT_PRIMARY: Color = Color::from_rgb(0.95, 0.95, 0.95); // Primary text
    pub const TEXT_SECONDARY: Color = Color::from_rgb(0.7, 0.7, 0.7); // Secondary text
    pub const TEXT_MUTED: Color = Color::from_rgb(0.5, 0.5, 0.5); // Muted text

    // Status colors with modern palette
    pub const SUCCESS: Color = Color::from_rgb(0.0, 0.8, 0.4); // Modern green
    pub const WARNING: Color = Color::from_rgb(1.0, 0.6, 0.0); // Modern orange
    pub const ERROR: Color = Color::from_rgb(1.0, 0.23, 0.19); // Modern red
    pub const INFO: Color = Color::from_rgb(0.0, 0.48, 1.0); // Modern blue

    // Dependency status colors with better visibility
    pub const FOUND: Color = Color::from_rgb(0.0, 0.8, 0.4); // Green for found
    pub const MISSING: Color = Color::from_rgb(1.0, 0.23, 0.19); // Red for missing
    pub const SYSTEM_DLL: Color = Color::from_rgb(0.5, 0.7, 1.0); // Blue for system
    pub const DELAYED: Color = Color::from_rgb(1.0, 0.6, 0.0); // Orange for delayed
    pub const MLL_PLUGIN: Color = Color::from_rgb(0.8, 0.4, 1.0); // Purple for Maya plugins

    // Border and separator colors
    pub const BORDER: Color = Color::from_rgb(0.25, 0.25, 0.25);
    pub const BORDER_FOCUS: Color = Color::from_rgb(0.0, 0.48, 1.0);
    pub const SEPARATOR: Color = Color::from_rgb(0.2, 0.2, 0.2);
}

/// Modern spacing system based on 8px grid
pub struct Spacing;

impl Spacing {
    pub const TINY: u16 = 2; // 2px
    pub const SMALL: u16 = 4; // 4px
    pub const MEDIUM: u16 = 8; // 8px (base unit)
    pub const LARGE: u16 = 16; // 16px
    pub const EXTRA_LARGE: u16 = 24; // 24px
    pub const HUGE: u16 = 32; // 32px
    pub const MASSIVE: u16 = 48; // 48px
}

/// Modern typography scale
pub struct FontSize;

impl FontSize {
    pub const CAPTION: u16 = 11; // Small captions
    pub const SMALL: u16 = 12; // Small text
    pub const BODY: u16 = 14; // Body text (default)
    pub const MEDIUM: u16 = 16; // Medium text
    pub const LARGE: u16 = 18; // Large text
    pub const HEADING: u16 = 20; // Headings
    pub const TITLE: u16 = 24; // Titles
    pub const DISPLAY: u16 = 32; // Display text
}

/// Border radius for modern design
pub struct BorderRadius;

impl BorderRadius {
    pub const SMALL: f32 = 4.0; // Small radius
    pub const MEDIUM: f32 = 8.0; // Medium radius
    pub const LARGE: f32 = 12.0; // Large radius
    pub const ROUND: f32 = 50.0; // Fully rounded
}

/// Shadow definitions for depth
pub struct Shadow;

impl Shadow {
    pub const SMALL: f32 = 2.0; // Small shadow
    pub const MEDIUM: f32 = 4.0; // Medium shadow
    pub const LARGE: f32 = 8.0; // Large shadow
}

/// File type icons and colors
pub struct FileTypes;

impl FileTypes {
    /// Get icon for file extension - using Unicode symbols that are more widely supported
    pub fn get_icon(extension: &str) -> &'static str {
        match extension.to_lowercase().as_str() {
            "exe" => "▶",  // Play symbol for executable
            "dll" => "⚙",  // Gear symbol for DLL
            "sys" => "⚡", // Lightning for system files
            "ocx" => "◆",  // Diamond for OCX controls
            "mll" => "★",  // Star for Maya plugins
            _ => "◯",      // Circle for unknown files
        }
    }

    /// Get emoji icon for file extension (fallback for systems that support emoji)
    pub fn get_emoji_icon(extension: &str) -> &'static str {
        match extension.to_lowercase().as_str() {
            "exe" => "🚀",
            "dll" => "🔧",
            "sys" => "⚙️",
            "ocx" => "🎛️",
            "mll" => "🎭", // Maya plugin icon
            _ => "📄",
        }
    }

    pub fn get_color(extension: &str) -> Color {
        match extension.to_lowercase().as_str() {
            "exe" => Colors::PRIMARY,
            "dll" => Colors::SUCCESS,
            "sys" => Colors::WARNING,
            "ocx" => Colors::INFO,
            "mll" => Colors::MLL_PLUGIN, // Special color for Maya plugins
            _ => Colors::TEXT_MUTED,
        }
    }

    pub fn get_description(extension: &str) -> &'static str {
        match extension.to_lowercase().as_str() {
            "exe" => "Executable File",
            "dll" => "Dynamic Link Library",
            "sys" => "System Driver",
            "ocx" => "ActiveX Control",
            "mll" => "Maya Plugin Library", // Maya plugin description
            _ => "Unknown File Type",
        }
    }
}
