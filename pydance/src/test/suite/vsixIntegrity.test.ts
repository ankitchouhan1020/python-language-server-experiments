import * as assert from "assert";
import * as path from "path";
import * as fs from "fs";

suite("VSIX Integrity Test Suite", () => {
  test("node_modules contains required dependencies", () => {
    const extensionPath = path.resolve(__dirname, "../../../");
    const nodeModulesPath = path.join(extensionPath, "node_modules");
    
    // Check that node_modules exists
    assert.ok(
      fs.existsSync(nodeModulesPath),
      "node_modules directory should exist"
    );
    
    // Check for critical dependency
    const languageClientPath = path.join(
      nodeModulesPath,
      "vscode-languageclient"
    );
    assert.ok(
      fs.existsSync(languageClientPath),
      "vscode-languageclient should be installed"
    );
    
    // Check that the main entry point exists
    const mainPath = path.join(extensionPath, "out", "extension.js");
    assert.ok(
      fs.existsSync(mainPath),
      "Compiled extension.js should exist"
    );
    
    // Check that pylight binary exists
    const pylightPath = path.join(extensionPath, "pylight");
    assert.ok(
      fs.existsSync(pylightPath),
      "pylight binary should exist"
    );
  });
  
  test("Can load extension module", async () => {
    // This will throw if dependencies are missing
    try {
      // Use dynamic import to avoid linter error
      const extensionModule = await import("../../extension");
      assert.ok(extensionModule.activate, "Extension should export activate function");
      assert.ok(extensionModule.deactivate, "Extension should export deactivate function");
    } catch (error) {
      assert.fail(`Failed to load extension module: ${error}`);
    }
  });
});