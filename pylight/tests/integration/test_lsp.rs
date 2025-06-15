use pylight::{LspServer, PythonParser, SymbolIndex};
use std::path::PathBuf;

#[test]
fn test_lsp_handler() {
    use pylight::{SearchEngine, Symbol, SymbolKind};
    use std::sync::Arc;
    
    // Create test symbols
    let index = Arc::new(SymbolIndex::new());
    let search_engine = Arc::new(SearchEngine::new());
    
    let symbols = vec![
        Symbol::new("test_function".to_string(), SymbolKind::Function, PathBuf::from("/test/file.py"), 10, 0),
        Symbol::new("TestClass".to_string(), SymbolKind::Class, PathBuf::from("/test/file.py"), 20, 0),
    ];
    
    index.add_file(PathBuf::from("/test/file.py"), symbols).unwrap();
    
    // Test workspace symbol search
    let params = lsp_types::WorkspaceSymbolParams {
        query: "test".to_string(),
        ..Default::default()
    };
    
    let results = pylight::lsp::handlers::handle_workspace_symbol(params, index, search_engine).unwrap();
    
    assert_eq!(results.len(), 2);
    assert!(results.iter().any(|s| s.name == "test_function"));
    assert!(results.iter().any(|s| s.name == "TestClass"));
}

#[test]
fn test_empty_query_returns_empty() {
    use pylight::SearchEngine;
    use std::sync::Arc;
    
    let index = Arc::new(SymbolIndex::new());
    let search_engine = Arc::new(SearchEngine::new());
    
    let params = lsp_types::WorkspaceSymbolParams {
        query: "".to_string(),
        ..Default::default()
    };
    
    let results = pylight::lsp::handlers::handle_workspace_symbol(params, index, search_engine).unwrap();
    assert_eq!(results.len(), 0);
}