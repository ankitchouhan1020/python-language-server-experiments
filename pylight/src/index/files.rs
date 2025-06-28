//! File operations for the Python Language Server

use std::path::PathBuf;
use walkdir::WalkDir;

/// Collect all Python files in a directory
pub fn collect_python_files(root: &PathBuf) -> Vec<PathBuf> {
    WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && e.path().extension().is_some_and(|ext| ext == "py"))
        .map(|e| {
            e.path()
                .canonicalize()
                .unwrap_or_else(|_| e.path().to_path_buf())
        })
        .collect()
}
