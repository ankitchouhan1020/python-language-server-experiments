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


    // Log build information
    tracing::info!(
        "Starting pylight v{} (built: {}, commit: {})",
        env!("CARGO_PKG_VERSION"),
        env!("BUILD_TIMESTAMP"),
        env!("GIT_COMMIT_HASH")
    );

    // Configure rayon to use all but one CPU (leave one for system tasks)
    let num_cpus = num_cpus::get();
    let num_threads = (num_cpus - 1).max(1); // Use at least 1 thread
    tracing::info!(
        "Detected {} CPUs, configuring rayon thread pool with {} threads",
        num_cpus,
        num_threads
    );
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
        .unwrap();

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
    use pylight::{SearchEngine, SymbolIndex};
    use std::sync::Arc;

    let dir = directory.unwrap_or_else(|| ".".into());
    tracing::info!("Running in standalone mode");
    tracing::info!("Indexing directory: {}", dir.display());

    // Create index and use the parallel index_workspace method
    let index = Arc::new(SymbolIndex::new());
    index.clone().index_workspace(&dir)?;

    if let Some(query) = query {
        let search_engine = SearchEngine::new();
        let all_symbols = index.get_all_symbols();
        let results = search_engine.search(&query, &all_symbols);

        tracing::info!("Search results for '{}':", query);
        for (i, result) in results.iter().take(20).enumerate() {
            tracing::info!(
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
