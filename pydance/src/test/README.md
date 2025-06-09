# Pydance Extension Tests

This directory contains tests for the Pydance VS Code extension.

## Test Structure

- **Mock Tests**: Test the extension functionality using mock language server responses
- **Integration Tests**: Test the full extension with the actual pylight language server

## Running Tests

### Run all tests (mock tests only by default):
```bash
npm test
```

### Run integration tests (requires pylight binary):
```bash
npm run test:integration
```

## Test Files

- `extension.test.ts`: Main test suite with both mock and integration tests
- `mockLanguageServer.ts`: Mock implementation of language server responses
- `helper.ts`: Test utilities and helpers
- `runTest.ts`: Default test runner (runs without workspace)
- `runIntegrationTest.ts`: Integration test runner (runs with workspace)

## Prerequisites for Integration Tests

1. The `pylight` binary must be present in the extension root directory
2. The test will use the `testFixture/test.py` file as the test workspace

## Mock Tests

Mock tests verify:
- Extension activation
- Workspace symbol provider functionality
- Symbol filtering based on search queries

These tests don't require the actual language server and use a mock provider instead.

## Integration Tests

Integration tests verify the full flow:
- Extension activation with a real workspace
- Language server initialization
- Actual symbol search through the pylight language server

These tests require the pylight binary and will be skipped if it's not available.