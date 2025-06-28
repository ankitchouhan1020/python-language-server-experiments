//! Index update coordinator that handles file change events

use crate::watcher::{FileEvent, FileEventHandler};
use crate::{PythonParser, Result, Symbol, SymbolIndex};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Instant;
use tracing::{error, info, warn};

/// State of the index updater
#[derive(Debug, Clone, Copy, PartialEq)]
enum UpdaterState {
    /// Idle, ready to process events
    Idle,
    /// Currently processing a full re-index
    ReIndexing,
    /// Processing incremental updates
    Updating,
}

/// Manages index updates from file system events
pub struct IndexUpdater {
    index: Arc<SymbolIndex>,
    state: Arc<RwLock<UpdaterState>>,
    pending_updates: Arc<Mutex<Vec<FileEvent>>>,
    workspace_root: PathBuf,
}

impl IndexUpdater {
    pub fn new(index: Arc<SymbolIndex>, workspace_root: PathBuf) -> Self {
        Self {
            index,
            state: Arc::new(RwLock::new(UpdaterState::Idle)),
            pending_updates: Arc::new(Mutex::new(Vec::new())),
            workspace_root,
        }
    }

    /// Process a single file update
    fn process_file_update(&self, path: &Path) -> Result<()> {
        let start = Instant::now();

        // Create a parser for this file
        let mut parser = PythonParser::new()?;

        // Read and parse the file
        match std::fs::read_to_string(path) {
            Ok(content) => match parser.parse_file(path, &content) {
                Ok(symbols) => {
                    self.index.add_file(path.to_path_buf(), symbols)?;
                    info!(
                        "Updated file {} in {:.2}ms",
                        path.display(),
                        start.elapsed().as_secs_f64() * 1000.0
                    );
                    Ok(())
                }
                Err(e) => {
                    warn!("Failed to parse {}: {}", path.display(), e);
                    Ok(())
                }
            },
            Err(e) => {
                warn!("Failed to read {}: {}", path.display(), e);
                Ok(())
            }
        }
    }

    /// Process multiple file updates
    fn process_file_updates(&self, paths: &[PathBuf]) -> Result<()> {
        let start = Instant::now();

        // Process files in parallel
        let updates: Vec<(PathBuf, Vec<Symbol>)> = paths
            .iter()
            .filter_map(|path| {
                let mut parser = match PythonParser::new() {
                    Ok(p) => p,
                    Err(e) => {
                        warn!("Failed to create parser: {}", e);
                        return None;
                    }
                };

                match std::fs::read_to_string(path) {
                    Ok(content) => match parser.parse_file(path, &content) {
                        Ok(symbols) => Some((path.clone(), symbols)),
                        Err(e) => {
                            warn!("Failed to parse {}: {}", path.display(), e);
                            None
                        }
                    },
                    Err(e) => {
                        warn!("Failed to read {}: {}", path.display(), e);
                        None
                    }
                }
            })
            .collect();

        let (updated_count, symbol_count) = self.index.update_files_batch(updates)?;

        info!(
            "Updated {} files with {} symbols in {:.2}ms",
            updated_count,
            symbol_count,
            start.elapsed().as_secs_f64() * 1000.0
        );

        Ok(())
    }

    /// Perform a full re-index of the workspace
    fn perform_full_reindex(&self) -> Result<()> {
        info!("Starting full workspace re-index");
        let start = Instant::now();

        // Create a new temporary index
        let new_index = Arc::new(SymbolIndex::new());

        // Index the workspace into the new index
        new_index.clone().index_workspace(&self.workspace_root)?;

        // Atomically swap the indices
        self.index.swap_index(&new_index);

        info!(
            "Full re-index completed in {:.2}s",
            start.elapsed().as_secs_f64()
        );

        Ok(())
    }

    /// Process pending updates
    fn process_pending_updates(&self) {
        let events = {
            let mut pending = self.pending_updates.lock().unwrap();
            std::mem::take(&mut *pending)
        };

        if !events.is_empty() {
            info!("Processing {} pending file events", events.len());
            for event in events {
                self.handle_event_internal(event);
            }
        }
    }

    /// Internal event handler
    fn handle_event_internal(&self, event: FileEvent) {
        // Check if we're already processing something
        let current_state = *self.state.read().unwrap();

        match current_state {
            UpdaterState::ReIndexing => {
                // Queue the event for later
                self.pending_updates.lock().unwrap().push(event);
                return;
            }
            UpdaterState::Updating => {
                // Allow concurrent updates for now
                // In the future, we might want to queue these as well
            }
            UpdaterState::Idle => {}
        }

        match event {
            FileEvent::FileChanged(path) => {
                *self.state.write().unwrap() = UpdaterState::Updating;

                if let Err(e) = self.process_file_update(&path) {
                    error!("Failed to update file {}: {}", path.display(), e);
                }

                *self.state.write().unwrap() = UpdaterState::Idle;
            }
            FileEvent::BulkChange(paths) => {
                *self.state.write().unwrap() = UpdaterState::ReIndexing;

                // Decide whether to do incremental updates or full re-index
                let total_indexed = self.index.get_file_count();
                let changed_ratio = paths.len() as f64 / total_indexed.max(1) as f64;

                if changed_ratio > 0.5 {
                    // More than 50% of files changed, do a full re-index
                    info!(
                        "Large change detected ({} files, {:.0}% of total), performing full re-index",
                        paths.len(),
                        changed_ratio * 100.0
                    );

                    if let Err(e) = self.perform_full_reindex() {
                        error!("Failed to perform full re-index: {}", e);
                    }
                } else {
                    // Do incremental updates
                    info!(
                        "Bulk change detected ({} files), performing incremental updates",
                        paths.len()
                    );

                    if let Err(e) = self.process_file_updates(&paths) {
                        error!("Failed to process bulk updates: {}", e);
                    }
                }

                *self.state.write().unwrap() = UpdaterState::Idle;

                // Process any pending updates that came in during re-indexing
                self.process_pending_updates();
            }
            FileEvent::FileRemoved(path) => {
                *self.state.write().unwrap() = UpdaterState::Updating;

                if let Err(e) = self.index.remove_file(&path) {
                    error!("Failed to remove file {}: {}", path.display(), e);
                }

                *self.state.write().unwrap() = UpdaterState::Idle;
            }
        }
    }
}

impl FileEventHandler for IndexUpdater {
    fn handle_event(&self, event: FileEvent) {
        // Clone what we need for the thread
        let updater = Arc::new(Self {
            index: self.index.clone(),
            state: self.state.clone(),
            pending_updates: self.pending_updates.clone(),
            workspace_root: self.workspace_root.clone(),
        });

        // Spawn a thread to handle the event
        thread::spawn(move || {
            updater.handle_event_internal(event);
        });
    }

    fn should_watch(&self, path: &Path) -> bool {
        // Only watch Python files
        path.extension()
            .and_then(|s| s.to_str())
            .map(|s| s == "py")
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_updater_creation() {
        let temp_dir = TempDir::new().unwrap();
        let index = Arc::new(SymbolIndex::new());
        let updater = IndexUpdater::new(index, temp_dir.path().to_path_buf());

        assert_eq!(*updater.state.read().unwrap(), UpdaterState::Idle);
    }

    #[test]
    fn test_should_watch_python_files() {
        let temp_dir = TempDir::new().unwrap();
        let index = Arc::new(SymbolIndex::new());
        let updater = IndexUpdater::new(index, temp_dir.path().to_path_buf());

        assert!(updater.should_watch(Path::new("test.py")));
        assert!(updater.should_watch(Path::new("path/to/file.py")));
        assert!(!updater.should_watch(Path::new("test.txt")));
        assert!(!updater.should_watch(Path::new("test.js")));
        assert!(!updater.should_watch(Path::new("test")));
    }
}
