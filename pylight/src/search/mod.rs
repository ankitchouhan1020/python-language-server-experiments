//! Symbol search functionality

use crate::symbols::Symbol;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::sync::Arc;

pub struct SearchEngine {
    matcher: SkimMatcherV2,
}

pub struct SearchResult {
    pub symbol: Arc<Symbol>,
    pub score: i64,
}

impl SearchEngine {
    pub fn new() -> Self {
        Self {
            matcher: SkimMatcherV2::default(),
        }
    }

    pub fn search(&self, query: &str, symbols: &[Arc<Symbol>]) -> Vec<SearchResult> {
        if query.is_empty() {
            // Return first 100 symbols when query is empty
            return symbols
                .iter()
                .take(100)
                .map(|symbol| SearchResult {
                    symbol: Arc::clone(symbol),
                    score: 0,
                })
                .collect();
        }

        let mut results: Vec<SearchResult> = symbols
            .iter()
            .filter_map(|symbol| {
                self.matcher
                    .fuzzy_match(&symbol.name, query)
                    .map(|score| SearchResult {
                        symbol: Arc::clone(symbol),
                        score,
                    })
            })
            .collect();

        // Sort by score descending
        results.sort_by(|a, b| b.score.cmp(&a.score));

        results
    }
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbols::SymbolKind;
    use std::path::PathBuf;

    #[test]
    fn test_search_engine_creation() {
        let engine = SearchEngine::new();
        let results = engine.search("test", &[]);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_search_basic() {
        let engine = SearchEngine::new();
        let symbols = vec![
            Arc::new(Symbol::new(
                "test_function".to_string(),
                SymbolKind::Function,
                PathBuf::from("test.py"),
                1,
                0,
            )),
            Arc::new(Symbol::new(
                "another_function".to_string(),
                SymbolKind::Function,
                PathBuf::from("test.py"),
                2,
                0,
            )),
        ];

        let results = engine.search("test", &symbols);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].symbol.name, "test_function");
    }
}
