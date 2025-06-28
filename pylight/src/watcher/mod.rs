//! File system watcher with debouncing support

use crate::Result;
use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebounceEventResult, DebouncedEventKind, Debouncer};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};

/// Configuration for the file watcher
#[derive(Debug, Clone)]
pub struct WatcherConfig {
    /// Debounce period in milliseconds
    pub debounce_ms: u64,
    /// Maximum time to wait for debounce timeout in milliseconds
    pub timeout_ms: u64,
    /// Number of files that triggers a full re-index instead of incremental updates
    pub batch_threshold: usize,
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            debounce_ms: 100,
            timeout_ms: 60000,
            batch_threshold: 10,
        }
    }
}

/// File system event type after processing
#[derive(Debug, Clone)]
pub enum FileEvent {
    /// Single file was modified or created
    FileChanged(PathBuf),
    /// Multiple files changed, triggering a full re-index
    BulkChange(Vec<PathBuf>),
    /// File was removed
    FileRemoved(PathBuf),
}

/// Manages file system watching with debouncing
pub struct FileWatcher {
    _config: WatcherConfig,
    watcher: Debouncer<notify::RecommendedWatcher>,
    _event_handler: Arc<dyn FileEventHandler + Send + Sync>,
}

/// Trait for handling file system events
pub trait FileEventHandler: Send + Sync {
    /// Handle a file system event
    fn handle_event(&self, event: FileEvent);

    /// Check if a path should be watched (e.g., is it a Python file?)
    fn should_watch(&self, path: &Path) -> bool;
}

impl FileWatcher {
    /// Create a new file watcher with the given configuration
    pub fn new(
        config: WatcherConfig,
        event_handler: Arc<dyn FileEventHandler + Send + Sync>,
    ) -> Result<Self> {
        let event_handler_clone = event_handler.clone();
        let batch_threshold = config.batch_threshold;

        // Create debouncer with our configuration
        let debouncer = new_debouncer(
            Duration::from_millis(config.debounce_ms),
            move |res: DebounceEventResult| {
                match res {
                    Ok(events) => {
                        // Process debounced events
                        let mut changed_paths = HashSet::new();
                        let mut removed_paths = HashSet::new();

                        for event in events {
                            if event.kind == DebouncedEventKind::Any {
                                // Check if file exists to determine if it's a create/modify or remove
                                if event.path.exists() {
                                    if event_handler_clone.should_watch(&event.path) {
                                        changed_paths.insert(event.path);
                                    }
                                } else if event_handler_clone.should_watch(&event.path) {
                                    removed_paths.insert(event.path);
                                }
                            }
                        }

                        // Handle removed files
                        for path in removed_paths {
                            event_handler_clone.handle_event(FileEvent::FileRemoved(path));
                        }

                        // Handle changed files
                        let changed_files: Vec<PathBuf> = changed_paths.into_iter().collect();
                        if !changed_files.is_empty() {
                            if changed_files.len() >= batch_threshold {
                                // Many files changed - trigger bulk update
                                info!(
                                    "Detected {} file changes, triggering bulk re-index",
                                    changed_files.len()
                                );
                                event_handler_clone
                                    .handle_event(FileEvent::BulkChange(changed_files));
                            } else {
                                // Few files changed - update individually
                                for path in changed_files {
                                    info!("File changed: {}", path.display());
                                    event_handler_clone.handle_event(FileEvent::FileChanged(path));
                                }
                            }
                        }
                    }
                    Err(error) => {
                        error!("File watcher error: {:?}", error);
                    }
                }
            },
        )?;

        Ok(Self {
            _config: config,
            watcher: debouncer,
            _event_handler: event_handler,
        })
    }

    /// Start watching a directory recursively
    pub fn watch(&mut self, path: &Path) -> Result<()> {
        info!("Starting file watcher for: {}", path.display());
        self.watcher
            .watcher()
            .watch(path, RecursiveMode::Recursive)?;
        Ok(())
    }

    /// Stop watching a directory
    pub fn unwatch(&mut self, path: &Path) -> Result<()> {
        info!("Stopping file watcher for: {}", path.display());
        self.watcher.watcher().unwatch(path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;

    #[allow(dead_code)]
    struct TestEventHandler {
        events: Arc<Mutex<Vec<FileEvent>>>,
        counter: Arc<AtomicUsize>,
    }

    impl FileEventHandler for TestEventHandler {
        fn handle_event(&self, event: FileEvent) {
            self.events.lock().unwrap().push(event);
            self.counter.fetch_add(1, Ordering::SeqCst);
        }

        fn should_watch(&self, path: &Path) -> bool {
            path.extension()
                .and_then(|s| s.to_str())
                .map(|s| s == "py")
                .unwrap_or(false)
        }
    }

    #[test]
    fn test_watcher_config_default() {
        let config = WatcherConfig::default();
        assert_eq!(config.debounce_ms, 100);
        assert_eq!(config.timeout_ms, 60000);
        assert_eq!(config.batch_threshold, 10);
    }
}
