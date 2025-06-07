# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial MVP implementation of DependencyWalker RS
- Complete Slint-based GUI with drag-and-drop support
- PE parser for Windows executables and DLLs using dual strategy (pelite + goblin)
- Dependency tree visualization with detailed module information
- Support for both CLI and GUI modes in single executable
- Comprehensive testing suite and documentation
- GitHub Actions workflow for automated releases
- GoReleaser configuration for cross-platform distribution
- Zero-dependency executable deployment
- Modern Windows native interface
- API Set Schema redirection support
- Delay load dependency detection
- Security feature detection
- Intelligent search and filtering capabilities

### Features
- 🔍 **Deep Analysis**: Comprehensive dependency analysis with configurable depth
- 🌳 **Tree Visualization**: Interactive dependency tree with missing dependency highlighting
- 📊 **Multiple Formats**: Export results in Text, JSON, or XML formats
- ⚡ **Zero Dependencies**: Single executable with no external dependencies
- 🖥️ **Modern GUI**: Native Windows GUI built with Slint
- 💻 **Powerful CLI**: Full-featured command-line interface
- 🎯 **Smart Mode Detection**: Automatically switches between CLI and GUI modes
- 🔧 **DLL Resolution**: Full Windows DLL search path implementation
- 📦 **PE File Support**: Robust parsing of PE files (.exe, .dll, .sys, .ocx, .mll)
- 🌐 **Maya Plugin Support**: Special support for Maya plugin libraries (.mll)

### Technical Implementation
- **Language**: 100% Rust with MSVC toolchain
- **UI Framework**: Slint for modern, native GUI
- **Architecture**: Modular design with clear separation of concerns
- **Testing**: Comprehensive test suite with unit and integration tests
- **Documentation**: Complete API documentation and user guides
- **CI/CD**: GitHub Actions workflow for automated testing and releases
- **Distribution**: GoReleaser configuration for cross-platform builds
- **Packaging**: Single executable deployment

### Documentation
- Complete README with usage instructions
- API documentation for developers
- Manual testing guides
- Feature implementation summaries
- Contributing guidelines
- License information

## [0.1.0] - TBD

### Added
- Initial release
- MVP functionality complete
- Ready for production use

---

## Release Notes Template

### [Version] - Date

#### Added
- New features

#### Changed
- Changes in existing functionality

#### Deprecated
- Soon-to-be removed features

#### Removed
- Now removed features

#### Fixed
- Bug fixes

#### Security
- Vulnerability fixes
