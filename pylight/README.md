# pylight

A high-performance Python symbol search language server written in Rust.

## Features

- **Fast Symbol Extraction**: Uses tree-sitter for robust Python parsing
- **Fuzzy Search**: Powered by skim matcher for intelligent symbol search
- **LSP Support**: Implements workspace/symbol for editor integration
- **Parallel Processing**: Leverages multiple cores for workspace indexing
- **Clean Architecture**: Modular design with clear separation of concerns

## Architecture

```
pylight/
├── src/
│   ├── lib.rs          # Public API
│   ├── parser/         # Tree-sitter Python parsing
│   ├── symbols/        # Symbol types and definitions
│   ├── index/          # Symbol storage and indexing
│   ├── search/         # Fuzzy search implementation
│   ├── lsp/            # Language Server Protocol
│   └── bin/
│       └── pylight.rs  # CLI binary
├── tests/              # Integration tests
└── benches/            # Performance benchmarks
```

## Usage

## Build & Test

```bash
# Build in release mode
cargo build --release

# Run tests
cargo test

# Run benchmarks
cargo bench
```

### As an LSP Server

```bash
# Start the LSP server (communicates via stdio)
pylight
```

### Standalone Mode

```bash
# Index a directory and search for symbols
pylight --standalone --directory /path/to/project --query "test"
```

## Local testing tool

There is tool to help with development that opens a web page where you can try the symbol search outside of VSCode

Run this and open the webpage, enter the path to python code:

```
cargo run --release --bin pylight_devtools
```

## Integration with VSCode

The `pylight` LSP server is designed to work with the `pydance` VSCode extension.
The extension will automatically start the language server when opening Python files.

## Development

1. Write integration tests (`tests/integration/`)
2. Write unit tests for components (`src/*/tests.rs`)
3. Implement functionality to pass tests
4. Benchmark critical paths (`benches/`)

## License

MIT
