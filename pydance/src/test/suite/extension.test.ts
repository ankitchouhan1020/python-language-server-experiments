import * as vscode from "vscode";
import * as assert from "assert";
import { getDocUri, activate } from "./helper";
import { registerMockProviders } from "./mockLanguageServer";

suite("Extension Test Suite", () => {
  const docUri = getDocUri("test.py");

  test("Should activate extension", async () => {
    const ext = vscode.extensions.getExtension("ToughType.pydance");
    await ext?.activate();
    assert.ok(ext?.isActive);
  });

  suite("Mock Tests", () => {
    let disposables: vscode.Disposable[] = [];

    setup(() => {
      // Skip mock tests in integration test mode since the real language server will interfere
      if (process.env.INTEGRATION_TEST === "true") {
        return;
      }
      // Register mock providers before each test
      disposables = registerMockProviders();
    });

    teardown(() => {
      // Clean up mock providers after each test
      disposables.forEach((d) => d.dispose());
    });

    test("Should provide workspace symbols with mock provider", async function () {
      // Skip if running in integration test mode
      if (process.env.INTEGRATION_TEST === "true") {
        console.log("Skipping mock test in integration test mode");
        this.skip();
      }
      // Test searching for "test" - should return all symbols containing "test"
      const testSymbols = await vscode.commands.executeCommand<
        vscode.SymbolInformation[]
      >("vscode.executeWorkspaceSymbolProvider", "test");

      assert.ok(Array.isArray(testSymbols));
      assert.strictEqual(testSymbols.length, 5); // All our mock symbols contain "test"

      // Verify specific symbols
      const classSymbol = testSymbols.find((s) => s.name === "TestClass");
      assert.ok(classSymbol);
      assert.strictEqual(classSymbol.kind, vscode.SymbolKind.Class);

      const methodSymbol = testSymbols.find((s) => s.name === "test_method");
      assert.ok(methodSymbol);
      assert.strictEqual(methodSymbol.kind, vscode.SymbolKind.Method);
      assert.strictEqual(methodSymbol.containerName, "TestClass");
    });

    test("Should filter symbols based on query", async function () {
      // Skip if running in integration test mode
      if (process.env.INTEGRATION_TEST === "true") {
        console.log("Skipping mock test in integration test mode");
        this.skip();
      }
      // Test searching for "function"
      const functionSymbols = await vscode.commands.executeCommand<
        vscode.SymbolInformation[]
      >("vscode.executeWorkspaceSymbolProvider", "function");

      assert.strictEqual(functionSymbols.length, 2); // test_function and another_test_function

      const testFunction = functionSymbols.find(
        (s) => s.name === "test_function"
      );
      assert.ok(testFunction);
      assert.strictEqual(testFunction.kind, vscode.SymbolKind.Function);

      const anotherTestFunction = functionSymbols.find(
        (s) => s.name === "another_test_function"
      );
      assert.ok(anotherTestFunction);
      assert.strictEqual(anotherTestFunction.kind, vscode.SymbolKind.Function);
    });
  });

  suite("Integration Tests", () => {
    test("Should provide workspace symbols with real language server", async function () {
      // Skip if not running in integration test mode
      if (process.env.INTEGRATION_TEST !== "true") {
        console.log("Not in integration test mode, skipping");
        this.skip();
      }

      // First check if we have a workspace folder
      const workspaceFolders = vscode.workspace.workspaceFolders;
      if (!workspaceFolders || workspaceFolders.length === 0) {
        console.log("No workspace folder found, skipping integration test");
        this.skip();
      }

      // Check if pylight binary exists
      // eslint-disable-next-line @typescript-eslint/no-var-requires
      const fs = require("fs");
      // eslint-disable-next-line @typescript-eslint/no-var-requires
      const path = require("path");
      const extensionPath =
        vscode.extensions.getExtension("ToughType.pydance")!.extensionPath;
      const pylightPath = path.join(extensionPath, "pylight");

      if (!fs.existsSync(pylightPath)) {
        console.log("pylight binary not found, skipping integration test");
        this.skip();
      }

      await testWorkspaceSymbols(docUri, [
        new vscode.SymbolInformation(
          "TestClass",
          vscode.SymbolKind.Class,
          "",
          new vscode.Location(
            docUri,
            new vscode.Range(
              new vscode.Position(0, 0), // Line 1 (0-indexed)
              new vscode.Position(0, 0)
            )
          )
        ),
        new vscode.SymbolInformation(
          "test_method",
          vscode.SymbolKind.Function, // The server returns Function for methods
          "TestClass",
          new vscode.Location(
            docUri,
            new vscode.Range(
              new vscode.Position(1, 0), // Line 2, column 0 (0-indexed)
              new vscode.Position(1, 0)
            )
          )
        ),
        new vscode.SymbolInformation(
          "test_function",
          vscode.SymbolKind.Function,
          "",
          new vscode.Location(
            docUri,
            new vscode.Range(
              new vscode.Position(5, 0), // Line 6 (0-indexed)
              new vscode.Position(5, 0)
            )
          )
        ),
        new vscode.SymbolInformation(
          "another_test_function",
          vscode.SymbolKind.Function,
          "",
          new vscode.Location(
            docUri,
            new vscode.Range(
              new vscode.Position(9, 0), // Line 10 (0-indexed)
              new vscode.Position(9, 0)
            )
          )
        ),
        // Note: TEST_CONSTANT is not returned by the server when searching for "test"
      ]);
    });
  });
});

async function testWorkspaceSymbols(
  docUri: vscode.Uri,
  expectedSymbols: vscode.SymbolInformation[]
) {
  // Activate extension and language server
  await activate(docUri);

  // Wait a bit more for the language server to index the file
  await new Promise((resolve) => setTimeout(resolve, 1000));

  // Execute workspace symbol search
  const actualSymbols = await vscode.commands.executeCommand<
    vscode.SymbolInformation[]
  >("vscode.executeWorkspaceSymbolProvider", "test");

  // Verify we get an array back
  assert.ok(Array.isArray(actualSymbols));

  // Log what we actually found for debugging
  console.log(
    `Found ${actualSymbols.length} symbols:`,
    actualSymbols.map((s) => ({
      name: s.name,
      kind: s.kind,
      containerName: s.containerName,
      uri: s.location.uri.toString(),
      line: s.location.range.start.line,
      character: s.location.range.start.character,
    }))
  );

  // First check that we got the expected number of symbols
  assert.strictEqual(
    actualSymbols.length,
    expectedSymbols.length,
    `Expected ${expectedSymbols.length} symbols but found ${actualSymbols.length}`
  );

  // Check that we found the expected symbols with correct positions
  for (const expected of expectedSymbols) {
    const found = actualSymbols.find(
      (symbol) => symbol.name === expected.name && symbol.kind === expected.kind
    );
    assert.ok(
      found,
      `Expected to find symbol ${expected.name} of kind ${expected.kind}`
    );

    // Verify the position matches
    const expectedStart = expected.location.range.start;
    const foundStart = found.location.range.start;
    assert.strictEqual(
      foundStart.line,
      expectedStart.line,
      `Symbol ${expected.name} expected at line ${expectedStart.line} but found at line ${foundStart.line}`
    );
  }
}
