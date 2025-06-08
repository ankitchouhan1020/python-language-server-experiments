import * as vscode from 'vscode';

// Mock symbol data based on test.py
export const mockSymbols: vscode.SymbolInformation[] = [
  new vscode.SymbolInformation(
    "TestClass",
    vscode.SymbolKind.Class,
    "",
    new vscode.Location(
      vscode.Uri.file("/test.py"),
      new vscode.Range(new vscode.Position(0, 0), new vscode.Position(2, 12))
    )
  ),
  new vscode.SymbolInformation(
    "test_method",
    vscode.SymbolKind.Method,
    "TestClass",
    new vscode.Location(
      vscode.Uri.file("/test.py"),
      new vscode.Range(new vscode.Position(1, 4), new vscode.Position(2, 12))
    )
  ),
  new vscode.SymbolInformation(
    "test_function",
    vscode.SymbolKind.Function,
    "",
    new vscode.Location(
      vscode.Uri.file("/test.py"),
      new vscode.Range(new vscode.Position(5, 0), new vscode.Position(6, 17))
    )
  ),
  new vscode.SymbolInformation(
    "another_test_function",
    vscode.SymbolKind.Function,
    "",
    new vscode.Location(
      vscode.Uri.file("/test.py"),
      new vscode.Range(new vscode.Position(10, 0), new vscode.Position(11, 22))
    )
  ),
  new vscode.SymbolInformation(
    "TEST_CONSTANT",
    vscode.SymbolKind.Constant,
    "",
    new vscode.Location(
      vscode.Uri.file("/test.py"),
      new vscode.Range(new vscode.Position(14, 0), new vscode.Position(14, 22))
    )
  ),
];

export class MockWorkspaceSymbolProvider implements vscode.WorkspaceSymbolProvider {
  provideWorkspaceSymbols(
    query: string,
    _token: vscode.CancellationToken
  ): vscode.ProviderResult<vscode.SymbolInformation[]> {
    // Filter symbols based on query
    if (!query) {
      return mockSymbols;
    }
    
    const lowerQuery = query.toLowerCase();
    return mockSymbols.filter(symbol => 
      symbol.name.toLowerCase().includes(lowerQuery)
    );
  }
}

export function registerMockProviders(): vscode.Disposable[] {
  const disposables: vscode.Disposable[] = [];
  
  // Register mock workspace symbol provider
  disposables.push(
    vscode.languages.registerWorkspaceSymbolProvider(
      new MockWorkspaceSymbolProvider()
    )
  );
  
  return disposables;
}