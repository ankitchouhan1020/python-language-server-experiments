//! Symbol indexing and storage

use crate::{PythonParser, Result, Symbol};
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use walkdir::WalkDir;

pub struct SymbolIndex {
    symbols: Arc<RwLock<HashMap<PathBuf, Vec<Symbol>>>>,
    all_symbols: Arc<RwLock<Vec<Symbol>>>,
}

impl SymbolIndex {
    pub fn new() -> Self {
        Self {
            symbols: Arc::new(RwLock::new(HashMap::new())),
            all_symbols: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn add_file(&self, path: PathBuf, symbols: Vec<Symbol>) -> Result<()> {
        let mut file_symbols = self.symbols.write().unwrap();
        let mut all = self.all_symbols.write().unwrap();

        // Remove old symbols for this file if any
        if let Some(_old_symbols) = file_symbols.get(&path) {
            all.retain(|s| s.file_path != path);
        }

        // Add new symbols
        all.extend(symbols.clone());
        file_symbols.insert(path, symbols);

        Ok(())
    }

    pub fn remove_file(&self, path: &Path) -> Result<()> {
        let mut file_symbols = self.symbols.write().unwrap();
        let mut all = self.all_symbols.write().unwrap();

        file_symbols.remove(path);
        all.retain(|s| s.file_path != path);

        Ok(())
    }

    pub fn get_all_symbols(&self) -> Vec<Symbol> {
        self.all_symbols.read().unwrap().clone()
    }

    pub fn get_file_symbols(&self, path: &Path) -> Option<Vec<Symbol>> {
        self.symbols.read().unwrap().get(path).cloned()
    }

    pub fn clear(&self) {
        self.symbols.write().unwrap().clear();
        self.all_symbols.write().unwrap().clear();
    }

    /// Add multiple files in a single batch operation to minimize lock contention
    pub fn add_files_batch(&self, files: Vec<(PathBuf, Vec<Symbol>)>) -> Result<()> {
        let mut file_symbols = self.symbols.write().unwrap();
        let mut all = self.all_symbols.write().unwrap();

        for (path, symbols) in files {
            // Remove old symbols for this file if any
            if file_symbols.contains_key(&path) {
                all.retain(|s| s.file_path != path);
            }

            // Add new symbols
            all.extend(symbols.clone());
            file_symbols.insert(path, symbols);
        }

        Ok(())
    }

    /// Collect all Python files in a directory
    pub fn collect_python_files(root: &PathBuf) -> Vec<PathBuf> {
        WalkDir::new(root)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_type().is_file() && e.path().extension().is_some_and(|ext| ext == "py")
            })
            .map(|e| e.path().to_path_buf())
            .collect()
    }

    /// Parse and index a list of Python files in parallel
    /// Returns (number of files parsed, total symbols, elapsed time)
    pub fn parse_and_index_files(
        self: Arc<Self>,
        python_files: Vec<PathBuf>,
    ) -> Result<(usize, usize, std::time::Duration)> {
        let start_time = std::time::Instant::now();

        // Process files in parallel and collect all results
        let all_file_symbols: Vec<(PathBuf, Vec<Symbol>)> = python_files
            .par_iter()
            .filter_map(|path| {
                let thread_id = std::thread::current().id();
                tracing::debug!(
                    "Processing file: {} on thread {:?}",
                    path.display(),
                    thread_id
                );

                // Each thread gets its own parser
                let mut parser = match PythonParser::new() {
                    Ok(p) => p,
                    Err(e) => {
                        tracing::warn!("Failed to create parser: {}", e);
                        return None;
                    }
                };

                // Read and parse the file
                match std::fs::read_to_string(path) {
                    Ok(content) => match parser.parse_file(path, &content) {
                        Ok(symbols) => {
                            tracing::debug!(
                                "Parsed {} symbols from {} on thread {:?}",
                                symbols.len(),
                                path.display(),
                                thread_id
                            );
                            Some((path.clone(), symbols))
                        }
                        Err(e) => {
                            tracing::warn!("Failed to parse {}: {}", path.display(), e);
                            None
                        }
                    },
                    Err(e) => {
                        tracing::warn!("Failed to read {}: {}", path.display(), e);
                        None
                    }
                }
            })
            .collect();

        let elapsed = start_time.elapsed();

        // Calculate totals
        let total_symbols: usize = all_file_symbols
            .iter()
            .map(|(_, symbols)| symbols.len())
            .sum();
        let file_count = all_file_symbols.len();

        // Batch update the index
        self.add_files_batch(all_file_symbols)?;

        Ok((file_count, total_symbols, elapsed))
    }

    /// Index all Python files in a workspace directory
    pub fn index_workspace(self: Arc<Self>, root: &PathBuf) -> Result<()> {
        tracing::info!("Starting workspace indexing for: {}", root.display());

        // Collect all Python files first
        let python_files = Self::collect_python_files(root);
        tracing::info!("Found {} Python files to index", python_files.len());

        // Log thread pool info
        tracing::info!(
            "Starting parallel processing with {} concurrent tasks",
            rayon::current_num_threads()
        );

        // Parse and index files
        let (file_count, total_symbols, elapsed) = self.parse_and_index_files(python_files)?;

        tracing::info!(
            "Parallel parsing completed in {:.2}s ({:.0} files/sec)",
            elapsed.as_secs_f64(),
            file_count as f64 / elapsed.as_secs_f64()
        );

        tracing::info!(
            "Indexed {} files with {} symbols",
            file_count,
            total_symbols
        );

        Ok(())
    }
}

impl Default for SymbolIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_creation() {
        let index = SymbolIndex::new();
        assert_eq!(index.get_all_symbols().len(), 0);
    }
}
