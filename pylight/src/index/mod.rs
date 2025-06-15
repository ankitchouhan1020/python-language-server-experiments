//! Symbol indexing and storage

use crate::{Result, Symbol};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

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
        if let Some(old_symbols) = file_symbols.get(&path) {
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