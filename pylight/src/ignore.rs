//! Ignore management using gitignore files when available

use ignore::gitignore::GitignoreBuilder;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::debug;

/// VCS directories that should always be ignored
const VCS_DIRS: &[&str] = &[".git", ".hg", ".svn", ".bzr"];

/// Default Python-specific directories to ignore when no .gitignore exists
const DEFAULT_PYTHON_IGNORE: &[&str] = &[
    "__pycache__/",
    "*.pyc",
    "*.pyo",
    "*.pyd",
    ".Python",
    "*.so",
    ".venv/",
    "venv/",
    "env/",
    ".env/",
    "virtualenv/",
    ".pytest_cache/",
    ".mypy_cache/",
    ".tox/",
    ".ruff_cache/",
    "*.egg-info/",
    ".eggs/",
    "build/",
    "dist/",
    "htmlcov/",
    ".coverage",
    ".hypothesis/",
    "node_modules/",
    ".idea/",
    ".vscode/",
];

/// Manages ignore patterns for the workspace
#[derive(Clone)]
pub struct IgnoreFilter {
    matcher: Arc<ignore::gitignore::Gitignore>,
    workspace_root: PathBuf,
}

impl IgnoreFilter {
    /// Create a new IgnoreFilter for the given workspace root
    pub fn new(workspace_root: PathBuf) -> Self {
        let mut builder = GitignoreBuilder::new(&workspace_root);

        // Always ignore VCS directories
        for vcs in VCS_DIRS {
            let _ = builder.add_line(None, vcs);
        }

        // Try to load .gitignore from the workspace root
        let gitignore_path = workspace_root.join(".gitignore");
        let has_gitignore = if gitignore_path.exists() {
            if let Some(err) = builder.add(&gitignore_path) {
                debug!("Failed to parse .gitignore: {}", err);
                false
            } else {
                debug!("Loaded .gitignore from {:?}", gitignore_path);
                true
            }
        } else {
            false
        };

        if !has_gitignore {
            debug!("No .gitignore found, using Python defaults");
            // Add default Python ignore patterns
            for pattern in DEFAULT_PYTHON_IGNORE {
                let _ = builder.add_line(None, pattern);
                // Also add pattern without trailing slash for directories
                if let Some(stripped) = pattern.strip_suffix('/') {
                    let _ = builder.add_line(None, stripped);
                }
            }
        }

        let matcher = builder.build().unwrap_or_else(|_| {
            // Fallback to empty matcher if build fails
            GitignoreBuilder::new(&workspace_root).build().unwrap()
        });

        Self {
            matcher: Arc::new(matcher),
            workspace_root,
        }
    }

    /// Check if a path should be ignored
    pub fn should_ignore(&self, path: &Path) -> bool {
        // Check if any component in the path is a VCS directory
        for component in path.components() {
            if let std::path::Component::Normal(name) = component {
                if let Some(name_str) = name.to_str() {
                    if VCS_DIRS.contains(&name_str) {
                        return true;
                    }
                }
            }
        }

        // Try to make path relative for gitignore matching
        let check_path = if path.is_absolute() {
            match path.strip_prefix(&self.workspace_root) {
                Ok(rel) => rel,
                Err(_) => return false, // Path outside workspace
            }
        } else {
            path
        };

        // Check the path itself
        if self.matcher.matched(check_path, path.is_dir()).is_ignore() {
            return true;
        }

        // For files, also check if any parent directory should be ignored
        if !path.is_dir() {
            let mut current = check_path;
            while let Some(parent) = current.parent() {
                if !parent.as_os_str().is_empty() && self.matcher.matched(parent, true).is_ignore()
                {
                    return true;
                }
                current = parent;
            }
        }

        false
    }

    /// Check if a path should be watched (inverse of should_ignore)
    pub fn should_watch(&self, path: &Path) -> bool {
        !self.should_ignore(path)
    }
}

// Compatibility layer for existing code
use std::cell::RefCell;
thread_local! {
    static CURRENT_FILTER: RefCell<Option<Arc<IgnoreFilter>>> = const { RefCell::new(None) };
}

/// Initialize the ignore filter for compatibility functions
pub fn init_ignore_filter(workspace_root: PathBuf) -> Arc<IgnoreFilter> {
    let filter = Arc::new(IgnoreFilter::new(workspace_root));
    CURRENT_FILTER.with(|f| {
        *f.borrow_mut() = Some(filter.clone());
    });
    filter
}

/// Check if a directory should be ignored (compatibility function)
pub fn should_ignore_dir(path: &Path) -> bool {
    CURRENT_FILTER.with(|f| {
        if let Some(ref filter) = *f.borrow() {
            filter.should_ignore(path)
        } else {
            // Fallback: only check VCS directories
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| VCS_DIRS.contains(&name))
                .unwrap_or(false)
        }
    })
}

/// Check if a path should be watched (compatibility function)
pub fn should_watch_path(path: &Path) -> bool {
    !should_ignore_dir(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_vcs_dirs_always_ignored() {
        let temp_dir = TempDir::new().unwrap();
        let filter = IgnoreFilter::new(temp_dir.path().to_path_buf());

        assert!(filter.should_ignore(Path::new(".git")));
        assert!(filter.should_ignore(Path::new(".hg")));
        assert!(filter.should_ignore(Path::new(".svn")));
        assert!(filter.should_ignore(Path::new("src/.git")));
    }

    #[test]
    fn test_python_defaults_without_gitignore() {
        let temp_dir = TempDir::new().unwrap();
        let filter = IgnoreFilter::new(temp_dir.path().to_path_buf());

        assert!(filter.should_ignore(Path::new("__pycache__")));
        assert!(filter.should_ignore(Path::new("venv")));
        assert!(filter.should_ignore(Path::new(".venv")));
        assert!(filter.should_ignore(Path::new("file.pyc")));
        assert!(filter.should_ignore(Path::new("myproject.egg-info")));

        assert!(!filter.should_ignore(Path::new("src")));
        assert!(!filter.should_ignore(Path::new("main.py")));
    }

    #[test]
    fn test_gitignore_overrides_defaults() {
        let temp_dir = TempDir::new().unwrap();

        // Create a .gitignore that only ignores specific things
        fs::write(
            temp_dir.path().join(".gitignore"),
            "*.pyc\n.mypy_cache/\nmy_custom_dir/\n",
        )
        .unwrap();

        // Create the directories so is_dir() works correctly
        fs::create_dir(temp_dir.path().join(".mypy_cache")).unwrap();
        fs::create_dir(temp_dir.path().join("my_custom_dir")).unwrap();
        fs::create_dir(temp_dir.path().join("__pycache__")).unwrap();
        fs::create_dir(temp_dir.path().join("venv")).unwrap();
        fs::create_dir(temp_dir.path().join(".git")).unwrap();

        let filter = IgnoreFilter::new(temp_dir.path().to_path_buf());

        // Should ignore what's in gitignore
        assert!(filter.should_ignore(&temp_dir.path().join("test.pyc")));
        assert!(filter.should_ignore(&temp_dir.path().join(".mypy_cache")));
        assert!(filter.should_ignore(&temp_dir.path().join("my_custom_dir")));

        // Should NOT ignore Python defaults that aren't in gitignore
        assert!(!filter.should_ignore(&temp_dir.path().join("__pycache__")));
        assert!(!filter.should_ignore(&temp_dir.path().join("venv")));

        // VCS dirs are still always ignored
        assert!(filter.should_ignore(&temp_dir.path().join(".git")));
    }

    #[test]
    fn test_compatibility_functions() {
        let temp_dir = TempDir::new().unwrap();
        init_ignore_filter(temp_dir.path().to_path_buf());

        // Test compatibility functions work
        assert!(should_ignore_dir(Path::new(".git")));
        assert!(should_ignore_dir(Path::new("__pycache__")));
        assert!(!should_watch_path(Path::new(".git")));
        assert!(should_watch_path(Path::new("main.py")));
    }
}
