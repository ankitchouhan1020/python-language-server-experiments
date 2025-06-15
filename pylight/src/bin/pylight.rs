//! Pylight LSP server binary

use clap::Parser;
use pylight::{LspServer, Result};
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Run in standalone mode (index directory and exit)
    #[arg(long)]
    standalone: bool,

    /// Directory to index in standalone mode
    #[arg(short, long, requires = "standalone")]
    directory: Option<std::path::PathBuf>,

    /// Search query in standalone mode
    #[arg(short, long, requires = "standalone")]
    query: Option<String>,
}

fn main() -> Result<()> {
    // Initialize logging
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    let args = Args::parse();

    if args.standalone {
        // Standalone mode for testing
        run_standalone(args.directory, args.query)
    } else {
        // LSP server mode
        tracing::info!("Starting pylight LSP server");
        let server = LspServer::new()?;
        server.run()
    }
}

fn run_standalone(directory: Option<std::path::PathBuf>, query: Option<String>) -> Result<()> {
    use pylight::{PythonParser, SearchEngine, SymbolIndex};
    use walkdir::WalkDir;

    let dir = directory.unwrap_or_else(|| ".".into());
    tracing::info!("Running in standalone mode");
    tracing::info!("Indexing directory: {}", dir.display());

    let index = SymbolIndex::new();
    let mut parser = PythonParser::new()?;
    let mut file_count = 0;
    let mut symbol_count = 0;

    for entry in WalkDir::new(&dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "py"))
    {
        let path = entry.path();
        match std::fs::read_to_string(path) {
            Ok(content) => match parser.parse_file(path, &content) {
                Ok(symbols) => {
                    symbol_count += symbols.len();
                    index.add_file(path.to_path_buf(), symbols)?;
                    file_count += 1;
                }
                Err(e) => {
                    tracing::warn!("Failed to parse {}: {}", path.display(), e);
                }
            },
            Err(e) => {
                tracing::warn!("Failed to read {}: {}", path.display(), e);
            }
        }
    }

    println!("Indexed {} files with {} symbols", file_count, symbol_count);

    if let Some(query) = query {
        let search_engine = SearchEngine::new();
        let all_symbols = index.get_all_symbols();
        let results = search_engine.search(&query, &all_symbols);

        println!("\nSearch results for '{}':", query);
        for (i, result) in results.iter().take(20).enumerate() {
            println!(
                "{:2}. {} ({}:{})",
                i + 1,
                result.symbol.name,
                result.symbol.file_path.display(),
                result.symbol.line
            );
        }
    }

    Ok(())
}
