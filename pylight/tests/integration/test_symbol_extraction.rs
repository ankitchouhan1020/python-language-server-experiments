use pylight::{PythonParser, SymbolKind};
use std::path::Path;

#[test]
fn test_extract_simple_symbols() {
    let content = include_str!("../fixtures/simple.py");
    let path = Path::new("tests/fixtures/simple.py");

    let mut parser = PythonParser::new().expect("Failed to create parser");
    let symbols = parser
        .parse_file(path, content)
        .expect("Failed to parse file");

    // Verify we found the expected symbols
    let symbol_names: Vec<&str> = symbols.iter().map(|s| s.name.as_str()).collect();

    // Functions
    assert!(symbol_names.contains(&"simple_function"));
    assert!(symbol_names.contains(&"function_with_args"));
    assert!(symbol_names.contains(&"decorated_function"));
    assert!(symbol_names.contains(&"property_function"));

    // Classes
    assert!(symbol_names.contains(&"SimpleClass"));
    assert!(symbol_names.contains(&"AnotherClass"));

    // Methods
    assert!(symbol_names.contains(&"__init__"));
    assert!(symbol_names.contains(&"method"));
    assert!(symbol_names.contains(&"method_with_args"));

    // Verify symbol kinds
    let simple_func = symbols
        .iter()
        .find(|s| s.name == "simple_function")
        .unwrap();
    assert_eq!(simple_func.kind, SymbolKind::Function);

    let simple_class = symbols.iter().find(|s| s.name == "SimpleClass").unwrap();
    assert_eq!(simple_class.kind, SymbolKind::Class);

    let method = symbols
        .iter()
        .find(|s| s.name == "method" && s.container_name.is_some())
        .unwrap();
    assert_eq!(method.kind, SymbolKind::Method);
    assert_eq!(method.container_name.as_deref(), Some("SimpleClass"));
}

#[test]
fn test_extract_nested_symbols() {
    let content = include_str!("../fixtures/nested.py");
    let path = Path::new("tests/fixtures/nested.py");

    let mut parser = PythonParser::new().expect("Failed to create parser");
    let symbols = parser
        .parse_file(path, content)
        .expect("Failed to parse file");

    // Verify nested functions
    let inner_func = symbols.iter().find(|s| s.name == "inner_function").unwrap();
    assert_eq!(inner_func.kind, SymbolKind::NestedFunction);
    assert_eq!(inner_func.container_name.as_deref(), Some("outer_function"));

    let deeply_nested = symbols.iter().find(|s| s.name == "deeply_nested").unwrap();
    assert_eq!(deeply_nested.kind, SymbolKind::NestedFunction);
    assert_eq!(
        deeply_nested.container_name.as_deref(),
        Some("outer_function.inner_function")
    );

    // Verify nested classes
    let inner_class = symbols.iter().find(|s| s.name == "InnerClass").unwrap();
    assert_eq!(inner_class.kind, SymbolKind::NestedClass);
    assert_eq!(inner_class.container_name.as_deref(), Some("OuterClass"));

    // Verify method in nested class
    let inner_method = symbols.iter().find(|s| s.name == "inner_method").unwrap();
    assert_eq!(inner_method.kind, SymbolKind::Method);
    assert_eq!(
        inner_method.container_name.as_deref(),
        Some("OuterClass.InnerClass")
    );
}

#[test]
fn test_symbol_line_numbers() {
    let content = include_str!("../fixtures/simple.py");
    let path = Path::new("tests/fixtures/simple.py");

    let mut parser = PythonParser::new().expect("Failed to create parser");
    let symbols = parser
        .parse_file(path, content)
        .expect("Failed to parse file");

    // Check line numbers are correct
    let simple_func = symbols
        .iter()
        .find(|s| s.name == "simple_function")
        .unwrap();
    assert_eq!(simple_func.line, 3);

    let simple_class = symbols.iter().find(|s| s.name == "SimpleClass").unwrap();
    assert_eq!(simple_class.line, 11);
}

#[test]
fn test_decorated_symbols() {
    let content = include_str!("../fixtures/decorators.py");
    let path = Path::new("tests/fixtures/decorators.py");

    let mut parser = PythonParser::new().expect("Failed to create parser");
    let symbols = parser
        .parse_file(path, content)
        .expect("Failed to parse file");

    // All decorated functions should still be found
    let decorated_names = vec![
        "decorated_function",
        "property_method",
        "static_method",
        "class_method",
        "multi_decorated",
    ];

    for name in decorated_names {
        assert!(
            symbols.iter().any(|s| s.name == name),
            "Missing decorated function: {name}"
        );
    }
}
