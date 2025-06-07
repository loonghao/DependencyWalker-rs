# DependencyWalker RS

A modern Windows Dependency Walker implemented in Rust, providing fast and reliable PE file dependency analysis.

## Features

- 🔍 **PE File Analysis**: Robust parsing using pelite + goblin dual strategy
- 🌳 **Dependency Trees**: Complete dependency relationship visualization
- 🔧 **DLL Resolution**: Full Windows DLL search path implementation
- 🎯 **Symbol Analysis**: Import/export symbol matching and analysis
- ⚡ **API Set Support**: Windows API Set redirection mechanism
- 📦 **Zero Dependencies**: Single executable with static linking
- 🖥️ **Cross Platform**: Analyze Windows PE files from any OS
- 🎨 **Modern GUI**: Clean interface built with egui
- 💻 **CLI Interface**: Powerful command-line tools

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/your-org/dependencywalker_rs.git
cd dependencywalker_rs

# Build CLI version
cargo build --release --features cli

# Build GUI version
cargo build --release --features gui

# Build both
cargo build --release --features cli,gui
```

### Binary Releases

Download pre-built binaries from the [Releases](https://github.com/your-org/dependencywalker_rs/releases) page.

## Usage

### Command Line Interface

```bash
# Analyze a PE file
depwalker analyze example.exe

# Show dependency tree
depwalker tree example.exe

# List all dependencies
depwalker list example.exe --detailed

# Include system DLLs and custom paths
depwalker analyze example.exe --system --path "C:\MyLibs"

# Output as JSON
depwalker analyze example.exe --format json
```

### Graphical Interface

```bash
# Launch GUI application
depwalker-gui
```

### Programmatic API

```rust
use dependencywalker_rs::core::{PEFile, DependencyAnalyzer};

// Parse PE file
let pe_file = PEFile::from_path("example.exe")?;
let dependencies = pe_file.get_dependencies()?;

// Build dependency tree
let analyzer = DependencyAnalyzer::new();
let tree = analyzer.build_tree("example.exe")?;
```

## Development Status

This project is under active development. Current implementation status:

- ✅ Project structure and architecture
- ✅ Error handling framework
- ✅ CLI interface foundation
- ⏳ PE file parsing (in progress)
- ⏳ Dependency analysis engine
- ⏳ DLL search path resolver
- ⏳ Symbol analysis
- ⏳ API Set redirection
- ⏳ GUI implementation

## Architecture

```
dependencywalker_rs/
├── src/
│   ├── lib.rs              # Core library
│   ├── main.rs             # CLI application
│   ├── gui.rs              # GUI application
│   ├── error.rs            # Error handling
│   ├── core/               # Core functionality
│   │   ├── pe_parser.rs    # PE file parsing
│   │   ├── dependency.rs   # Dependency analysis
│   │   ├── resolver.rs     # DLL resolution
│   │   └── symbols.rs      # Symbol analysis
│   └── cli/                # CLI interface
│       ├── commands.rs     # Command implementations
│       └── output.rs       # Output formatting
```

## Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md) for details.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Dependencies](https://github.com/lucasg/Dependencies) - Inspiration and reference
- [dependency_runner](https://github.com/marcoesposito1988/dependency_runner) - Rust implementation reference
- [pelite](https://github.com/CasualX/pelite) - PE parsing library
- [goblin](https://github.com/m4b/goblin) - Binary parsing library
