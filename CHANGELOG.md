# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1](https://github.com/loonghao/DependencyWalker-rs/compare/dependencywalker-rs-v0.1.0...dependencywalker-rs-v0.1.1) (2026-03-24)


### Features

* add automatic version management with Release Please ([6bab00e](https://github.com/loonghao/DependencyWalker-rs/commit/6bab00ea1a1219435352093eddb7f0bea501c2f8))
* add comprehensive CI/CD pipeline with GoReleaser integration ([36c9981](https://github.com/loonghao/DependencyWalker-rs/commit/36c9981eb64a4a52b8163087a5f693a65a549146))
* implement MVP version of DependencyWalker RS ([f517ffc](https://github.com/loonghao/DependencyWalker-rs/commit/f517ffc3170fc950ec849aa271d8560a25e37209))
* Release MVP version of DependencyWalker RS ([76e81f8](https://github.com/loonghao/DependencyWalker-rs/commit/76e81f8c970167d77785f471314e0187e2b64241))


### Bug Fixes

* resolve all Clippy warnings and errors ([2738c43](https://github.com/loonghao/DependencyWalker-rs/commit/2738c434ed910312463eb75088629f2862b21229))
* resolve CI PowerShell path issues and missing examples ([13bf27a](https://github.com/loonghao/DependencyWalker-rs/commit/13bf27a1cf3ee8c7e208d02faffad3172c341581))
* resolve test failures and apply code formatting ([fe86c6f](https://github.com/loonghao/DependencyWalker-rs/commit/fe86c6ff8f16a01122eb56f2d070bbcc0dc826e4))


### Code Refactoring

* translate Chinese comments and descriptions to English ([e09c54b](https://github.com/loonghao/DependencyWalker-rs/commit/e09c54b882feb108c4324faaa7891c99556c75ab))

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
