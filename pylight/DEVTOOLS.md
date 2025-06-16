# Pylight DevTools

A web-based development tool for testing pylight performance and capabilities against large codebases.

## Usage

1. Build and run the devtools server:
   ```bash
   cargo run --release --bin pylight_devtools
   ```

2. Open http://localhost:8095 in your browser

3. Enter the absolute path to a Python codebase you want to test

4. Click "Index Codebase" to start pylight on that directory

5. Start typing in the search box - results appear instantly without debouncing

## Features

- **Live Search**: Every keystroke triggers a symbol search
- **Performance Metrics**: Shows search duration and result count
- **Zero Dependencies**: Pure HTML/CSS/JS frontend
- **Single Binary**: Everything runs from one Rust executable

## Architecture

The devtools server:
- Spawns and manages pylight LSP instances
- Translates HTTP requests to LSP protocol
- Serves the static HTML interface
- Collects and returns performance metrics

## Testing Large Codebases

Recommended test repositories:
- CPython source: `/path/to/cpython`
- Django: `/path/to/django`
- Your own large Python projects

The tool helps identify:
- Search performance bottlenecks
- Symbol matching accuracy
- Memory usage patterns
- Response time variations