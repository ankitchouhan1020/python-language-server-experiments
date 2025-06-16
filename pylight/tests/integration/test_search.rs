use pylight::{SearchEngine, Symbol, SymbolKind};
use std::path::PathBuf;
use std::sync::Arc;

fn create_test_symbols() -> Vec<Arc<Symbol>> {
    vec![
        Arc::new(Symbol::new(
            "test_function".to_string(),
            SymbolKind::Function,
            PathBuf::from("test.py"),
            1,
            0,
        )),
        Arc::new(Symbol::new(
            "TestClass".to_string(),
            SymbolKind::Class,
            PathBuf::from("test.py"),
            10,
            0,
        )),
        Arc::new(Symbol::new(
            "another_test_func".to_string(),
            SymbolKind::Function,
            PathBuf::from("test.py"),
            20,
            0,
        )),
        Arc::new(Symbol::new(
            "helper_function".to_string(),
            SymbolKind::Function,
            PathBuf::from("helper.py"),
            5,
            0,
        )),
        Arc::new(Symbol::new(
            "HelperClass".to_string(),
            SymbolKind::Class,
            PathBuf::from("helper.py"),
            15,
            0,
        )),
        Arc::new(
            Symbol::new(
                "test_method".to_string(),
                SymbolKind::Method,
                PathBuf::from("test.py"),
                12,
                4,
            )
            .with_container("TestClass".to_string()),
        ),
    ]
}

#[test]
fn test_exact_match() {
    let engine = SearchEngine::new();
    let symbols = create_test_symbols();

    let results = engine.search("test_function", &symbols);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].symbol.name, "test_function");
}

#[test]
fn test_fuzzy_match() {
    let engine = SearchEngine::new();
    let symbols = create_test_symbols();

    let results = engine.search("test", &symbols);
    assert!(results.len() >= 3);

    // Should match: test_function, TestClass, another_test_func, test_method
    let matched_names: Vec<&str> = results.iter().map(|r| r.symbol.name.as_str()).collect();
    assert!(matched_names.contains(&"test_function"));
    assert!(matched_names.contains(&"TestClass"));
    assert!(matched_names.contains(&"test_method"));
}

#[test]
fn test_case_insensitive_search() {
    let engine = SearchEngine::new();
    let symbols = create_test_symbols();

    let results = engine.search("testclass", &symbols);
    assert!(results.iter().any(|r| r.symbol.name == "TestClass"));
}

#[test]
fn test_partial_match() {
    let engine = SearchEngine::new();
    let symbols = create_test_symbols();

    let results = engine.search("help", &symbols);
    assert!(results.iter().any(|r| r.symbol.name == "helper_function"));
    assert!(results.iter().any(|r| r.symbol.name == "HelperClass"));
}

#[test]
fn test_empty_query() {
    let engine = SearchEngine::new();
    let symbols = create_test_symbols();

    let results = engine.search("", &symbols);
    assert_eq!(results.len(), 0);
}

#[test]
fn test_no_matches() {
    let engine = SearchEngine::new();
    let symbols = create_test_symbols();

    let results = engine.search("xyz123", &symbols);
    assert_eq!(results.len(), 0);
}

#[test]
fn test_search_result_ordering() {
    let engine = SearchEngine::new();
    let symbols = create_test_symbols();

    let results = engine.search("test", &symbols);

    // Exact matches should score higher
    // Verify results are sorted by score (descending)
    for i in 1..results.len() {
        assert!(
            results[i - 1].score >= results[i].score,
            "Results not properly sorted by score"
        );
    }
}
