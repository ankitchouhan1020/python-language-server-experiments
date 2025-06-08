import * as path from 'path';
import { runTests } from '@vscode/test-electron';

async function main() {
    try {
        // The folder containing the Extension Manifest package.json
        const extensionDevelopmentPath = path.resolve(__dirname, '../../');

        // The path to test runner for integration tests
        const extensionTestsPath = path.resolve(__dirname, './suite/index');

        // The path to the test workspace - this ensures we have a proper workspace
        const testWorkspace = path.resolve(__dirname, '../../src/testFixture');

        console.log('Running integration tests with workspace:', testWorkspace);

        // Download VS Code, unzip it and run the integration test
        await runTests({ 
            extensionDevelopmentPath, 
            extensionTestsPath,
            launchArgs: [testWorkspace],
            // Set environment variable to indicate integration test mode
            extensionTestsEnv: {
                INTEGRATION_TEST: 'true'
            }
        });
    } catch (err) {
        console.error('Failed to run integration tests');
        process.exit(1);
    }
}

main();