//! Core LSP server implementation

use crate::{Error, Result, SymbolIndex, SearchEngine, PythonParser};
use lsp_server::{Connection, Message, Response};
use lsp_types::{ServerCapabilities, WorkspaceSymbolParams, InitializeParams};
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use walkdir::WalkDir;

pub struct LspServer {
    connection: Connection,
    index: Arc<SymbolIndex>,
    search_engine: Arc<SearchEngine>,
    workspace_root: Option<PathBuf>,
}

impl LspServer {
    pub fn new() -> Result<Self> {
        let (connection, _io_threads) = Connection::stdio();
        
        Ok(Self {
            connection,
            index: Arc::new(SymbolIndex::new()),
            search_engine: Arc::new(SearchEngine::new()),
            workspace_root: None,
        })
    }

    pub fn run(mut self) -> Result<()> {
        // Initialize server
        let server_capabilities = ServerCapabilities {
            workspace_symbol_provider: Some(lsp_types::OneOf::Left(true)),
            ..ServerCapabilities::default()
        };

        let initialization_params = self.connection
            .initialize(serde_json::to_value(server_capabilities).unwrap())
            .map_err(|e| Error::Lsp(format!("Failed to initialize: {}", e)))?;

        // Extract workspace root
        if let Ok(params) = serde_json::from_value::<InitializeParams>(initialization_params) {
            #[allow(deprecated)]
            if let Some(root_uri) = params.root_uri {
                if let Ok(url) = url::Url::parse(root_uri.as_str()) {
                    if let Ok(path) = url.to_file_path() {
                        self.workspace_root = Some(path.clone());
                    
                        // Start background indexing
                        let index = self.index.clone();
                        let root = path.clone();
                        thread::spawn(move || {
                            if let Err(e) = index_workspace(&root, index) {
                                tracing::error!("Failed to index workspace: {}", e);
                            }
                        });
                    }
                }
            }
        }

        // Main message loop
        for msg in &self.connection.receiver {
            match msg {
                Message::Request(req) => {
                    if self.connection.handle_shutdown(&req).unwrap() {
                        return Ok(());
                    }
                    
                    // Handle workspace/symbol requests
                    if req.method == "workspace/symbol" {
                        match serde_json::from_value::<WorkspaceSymbolParams>(req.params) {
                            Ok(params) => {
                                match super::handlers::handle_workspace_symbol(
                                    params,
                                    self.index.clone(),
                                    self.search_engine.clone(),
                                ) {
                                    Ok(symbols) => {
                                        let resp = Response {
                                            id: req.id,
                                            result: Some(serde_json::to_value(symbols).unwrap()),
                                            error: None,
                                        };
                                        self.connection.sender.send(Message::Response(resp)).unwrap();
                                    }
                                    Err(e) => {
                                        let resp = Response {
                                            id: req.id,
                                            result: None,
                                            error: Some(lsp_server::ResponseError {
                                                code: lsp_server::ErrorCode::InternalError as i32,
                                                message: format!("Error: {}", e),
                                                data: None,
                                            }),
                                        };
                                        self.connection.sender.send(Message::Response(resp)).unwrap();
                                    }
                                }
                            }
                            Err(e) => {
                                let resp = Response {
                                    id: req.id,
                                    result: None,
                                    error: Some(lsp_server::ResponseError {
                                        code: lsp_server::ErrorCode::InvalidParams as i32,
                                        message: format!("Invalid params: {}", e),
                                        data: None,
                                    }),
                                };
                                self.connection.sender.send(Message::Response(resp)).unwrap();
                            }
                        }
                    }
                }
                Message::Response(_) => {}
                Message::Notification(_) => {}
            }
        }

        Ok(())
    }
}

fn index_workspace(root: &PathBuf, index: Arc<SymbolIndex>) -> Result<()> {
    tracing::info!("Starting workspace indexing for: {}", root.display());
    
    let mut parser = PythonParser::new()?;
    let mut file_count = 0;
    let mut symbol_count = 0;
    
    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "py"))
    {
        let path = entry.path();
        match std::fs::read_to_string(path) {
            Ok(content) => {
                match parser.parse_file(path, &content) {
                    Ok(symbols) => {
                        symbol_count += symbols.len();
                        index.add_file(path.to_path_buf(), symbols)?;
                        file_count += 1;
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse {}: {}", path.display(), e);
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to read {}: {}", path.display(), e);
            }
        }
    }
    
    tracing::info!("Indexed {} files with {} symbols", file_count, symbol_count);
    Ok(())
}