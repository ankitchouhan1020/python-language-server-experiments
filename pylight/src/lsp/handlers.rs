//! LSP request handlers

use crate::{Result, SymbolIndex, SearchEngine};
use lsp_types::{WorkspaceSymbolParams, SymbolInformation, Location, Range, Position, SymbolKind as LspSymbolKind};
use std::sync::Arc;

pub fn handle_workspace_symbol(
    params: WorkspaceSymbolParams,
    index: Arc<SymbolIndex>,
    search_engine: Arc<SearchEngine>,
) -> Result<Vec<SymbolInformation>> {
    if params.query.is_empty() {
        return Ok(vec![]);
    }

    let all_symbols = index.get_all_symbols();
    let search_results = search_engine.search(&params.query, &all_symbols);

    let lsp_symbols: Vec<SymbolInformation> = search_results
        .into_iter()
        .take(100) // Limit results
        .filter_map(|result| {
            let symbol = &result.symbol;
            let uri = url::Url::from_file_path(&symbol.file_path).ok()?;

            #[allow(deprecated)]
            Some(SymbolInformation {
                name: symbol.name.clone(),
                kind: match symbol.kind {
                    crate::SymbolKind::Function | crate::SymbolKind::NestedFunction => LspSymbolKind::FUNCTION,
                    crate::SymbolKind::Method => LspSymbolKind::METHOD,
                    crate::SymbolKind::Class | crate::SymbolKind::NestedClass => LspSymbolKind::CLASS,
                },
                tags: None,
                location: Location {
                    uri: uri.as_str().parse().unwrap(),
                    range: Range {
                        start: Position {
                            line: (symbol.line as u32).saturating_sub(1),
                            character: symbol.column as u32,
                        },
                        end: Position {
                            line: (symbol.line as u32).saturating_sub(1),
                            character: symbol.column as u32,
                        },
                    },
                },
                container_name: symbol.container_name.clone(),
                deprecated: None,
            })
        })
        .collect();

    Ok(lsp_symbols)
}