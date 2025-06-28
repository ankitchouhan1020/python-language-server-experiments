//! File operations for the Python Language Server

use crate::ignore::IgnoreFilter;
use ignore::WalkBuilder;
use std::path::PathBuf;

/// Collect all Python files in a directory
pub fn collect_python_files(root: &PathBuf) -> Vec<PathBuf> {
    let ignore_filter = IgnoreFilter::new(root.clone());

    // Use the ignore crate's WalkBuilder which respects .gitignore
    let mut builder = WalkBuilder::new(root);
    builder
        .standard_filters(false) // We'll use our own filter
        .follow_links(false);

    builder
        .build()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            let path = entry.path();
            // Must be a Python file and not ignored
            entry.file_type().map(|ft| ft.is_file()).unwrap_or(false)
                && path.extension().and_then(|s| s.to_str()) == Some("py")
                && !ignore_filter.should_ignore(path)
        })
        .map(|entry| {
            entry
                .path()
                .canonicalize()
                .unwrap_or_else(|_| entry.path().to_path_buf())
        })
        .collect()
}
