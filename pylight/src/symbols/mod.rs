//! Symbol definitions and types

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SymbolKind {
    Function,
    Class,
    Method,
    NestedFunction,
    NestedClass,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub file_path: PathBuf,
    pub line: usize,
    pub column: usize,
    pub container_name: Option<String>,
    pub module_path: String,
}

impl Symbol {
    pub fn new(name: String, kind: SymbolKind, file_path: PathBuf, line: usize, column: usize) -> Self {
        Self {
            name,
            kind,
            file_path,
            line,
            column,
            container_name: None,
            module_path: String::new(),
        }
    }

    pub fn with_container(mut self, container: String) -> Self {
        self.container_name = Some(container);
        self
    }

    pub fn with_module(mut self, module: String) -> Self {
        self.module_path = module;
        self
    }
}