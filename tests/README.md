# Codebuddy Test Suite

This directory contains the comprehensive test suite for Codebuddy's MCP (Model Context Protocol) server implementation.

## 📁 Test Structure

```
tests/
├── core/                 # Core functionality tests (3 files)
│   ├── comprehensive.test.ts   # All 28 tools test (~60s)
│   ├── intelligence.test.ts    # Intelligence features focus
│   └── playground.test.ts      # Playground validation
├── unit/                 # Unit tests with logical organization (11 files)
│   ├── handlers/         # MCP handler tests
│   │   └── handlers.test.ts    # Direct handler testing
│   ├── file-operations/  # File system operation tests
│   │   ├── file-editor.test.ts       # File editing operations
│   │   ├── file-editor-rollback.test.ts  # Edit rollback scenarios
│   │   └── file-editor-symlink.test.ts   # Symlink handling
│   ├── get-diagnostics.test.ts     # Diagnostic formatting & edge cases
│   ├── mcp-tools.test.ts           # MCP tool index conversion (0-based/1-based)
│   ├── multi-position.test.ts      # Position fallback logic testing
│   ├── progress-tracking.test.ts   # Progress notification system
│   ├── restart-server.test.ts      # Server restart timing
│   ├── server-lifecycle.test.ts    # LSP server lifecycle management
│   └── server-management.test.ts   # Server management operations
├── integration/          # Integration tests (6 files)
│   ├── call-hierarchy-adaptive.test.ts # Adaptive call hierarchy testing
│   ├── delete-file-enhanced.test.ts    # Enhanced file deletion with impact analysis
│   ├── edge-cases.test.ts              # Unicode, large files, boundary conditions
│   ├── error-cases.test.ts             # Error handling scenarios
│   ├── error-recovery.test.ts          # Server crash recovery testing
│   └── lsp-client.test.ts              # LSP client integration
└── helpers/
    └── mcp-test-client.ts  # Shared MCP testing utilities
```

## 🚀 Running Tests

### Comprehensive Testing
```bash
npm run test:comprehensive  # Test all 28 MCP tools
npm run test               # Run default test suite
npm run test:all           # Run all tests
```

### Unit Testing
```bash
npm run test:mcp:unit       # Run handler and client tests
npm run test:mcp:restart    # Test server restart functionality
```

## 📊 Test Coverage

### Core Tests (`/core`)

#### `comprehensive.test.ts` 🔬
- **Purpose**: Complete validation of all MCP tools
- **Coverage**: All 28 MCP tools
- **Duration**: ~60 seconds
- **Categories tested**:
  - Core Tools (4/4)
  - Document Tools (7/7)
  - Intelligence Tools (5/5)
  - Hierarchy Tools (3/3)
  - File Operations (3/3)
  - Server Management (1/1)
  - Advanced Workflow Tools (5/5)

#### `intelligence.test.ts` 🧠
- **Purpose**: Validate TypeScript intelligence features
- **Coverage**: 5 intelligence tools
- **Duration**: ~20 seconds
- **Tools tested**:
  - `get_hover` - Type information
  - `get_completions` - Code suggestions
  - `get_signature_help` - Function signatures
  - `get_inlay_hints` - Parameter hints
  - `get_semantic_tokens` - Syntax highlighting

#### `playground.test.ts` 🎮
- **Purpose**: Validate playground test environment
- **Coverage**: Key playground features
- **Duration**: ~15 seconds
- **Validates**:
  - Diagnostics on test files
  - Hover information
  - Symbol references
  - Document outline

### Unit Tests (`/unit` - 11 files)

#### Handler Testing (`handlers/`)
- **`handlers.test.ts`** 🔧 - Direct MCP handler testing with improved assertions

#### File Operations (`file-operations/`)  
- **`file-editor.test.ts`** 📝 - Core file editing operations
- **`file-editor-rollback.test.ts`** ↩️ - Edit rollback and error recovery
- **`file-editor-symlink.test.ts`** 🔗 - Symlink handling edge cases

#### Core Unit Tests
- **`get-diagnostics.test.ts`** 🩺 - Diagnostic severity mapping & formatting edge cases
- **`mcp-tools.test.ts`** 🔢 - Critical 0-based/1-based index conversion testing
- **`multi-position.test.ts`** 🎯 - Position fallback logic when definitions aren't found
- **`progress-tracking.test.ts`** 📊 - Progress notification system validation
- **`restart-server.test.ts`** 🔄 - Server restart timing (~700ms typical)
- **`server-lifecycle.test.ts`** ♻️ - LSP server process lifecycle management
- **`server-management.test.ts`** ⚙️ - Server configuration and management

### Integration Tests (`/integration` - 6 files)

#### `call-hierarchy-adaptive.test.ts` 🔗
- **Purpose**: Call hierarchy testing with system adaptation
- **Features**: Cross-file function call analysis, adaptive timeouts

#### `delete-file-enhanced.test.ts` 🗑️
- **Purpose**: Enhanced file deletion with impact analysis
- **Features**: Import impact detection, force deletion warnings

#### `edge-cases.test.ts` 🎯
- **Purpose**: Edge case and boundary condition testing
- **Features**: Unicode handling, large files, position boundaries

#### `error-cases.test.ts` ⚠️
- **Purpose**: Comprehensive error handling validation
- **Features**: Invalid inputs, malformed requests, resource limits

#### `error-recovery.test.ts` 🔄
- **Purpose**: Server crash recovery and resilience testing
- **Features**: LSP server crash simulation, memory management

#### `lsp-client.test.ts` 🔗
- **Purpose**: LSP client integration testing
- **Features**: Direct LSP client functionality across file types

## 🎯 Test Philosophy

### Test Categories

1. **Core Tests** - End-to-end MCP protocol testing
   - Tests through full MCP server
   - Validates actual tool responses
   - Ensures protocol compliance

2. **Unit Tests** - Component-level testing
   - Direct function testing
   - Faster execution
   - Easier debugging

### Coverage Goals

- ✅ **100% Tool Coverage**: All 28 MCP tools tested
- ✅ **Real Data Validation**: Tests verify actual TypeScript LSP data
- ✅ **Performance Monitoring**: Timing tracked for all operations
- ✅ **Error Handling**: Comprehensive error scenario coverage
- ✅ **Edge Cases**: Unicode, boundaries, large files
- ✅ **Recovery Testing**: Server crash and memory management

## 📈 Expected Results

### Success Criteria

| Test | Expected Result |
|------|-----------------|
| `comprehensive.test.ts` | 27-28/28 passed* |
| `intelligence.test.ts` | 5/5 with real TS data |
| `playground.test.ts` | All features operational |
| `handlers.test.ts` | All handlers functional |
| `call-hierarchy-adaptive.test.ts` | Cross-file calls detected |
| `delete-file-enhanced.test.ts` | Impact analysis working |
| `edge-cases.test.ts` | Unicode & boundaries handled |
| `error-cases.test.ts` | All error scenarios covered |
| `error-recovery.test.ts` | Server recovery functional |
| `lsp-client.test.ts` | Client operations working |
| `restart-server.test.ts` | ~700ms restart time |

*Note: `restart_server` may timeout in comprehensive test due to sequencing but works individually.

## 🔧 Troubleshooting

### Common Issues

1. **"No LSP server available"**
   - Ensure TypeScript Language Server is installed: `npx typescript-language-server --version`
   - Check config file exists: `/workspace/playground/codebuddy.json`

2. **Test timeouts**
   - Individual tests have different timeout settings
   - Intelligence features may take 2-4s (normal)
   - Run individual tests for debugging

3. **Build issues**
   - Ensure project is built: `npx esbuild index.ts --bundle --platform=node --target=node16 --format=esm --outdir=dist`
   - Or use bun: `bun run build`

## 🏗️ Test Maintenance

### Adding New Tests
1. Determine if it's a core or unit test
2. Follow existing naming convention: `[feature].test.ts`
3. Update this README with test details
4. Add npm script to package.json if needed

### Test Data Location
- Playground files: `/workspace/playground/src/`
- Config: `/workspace/playground/codebuddy.json`
- Test fixtures: `/workspace/tests/fixtures/`

## 📝 Historical Note

This test suite has evolved through multiple reorganizations:

**December 2024**: Consolidated 33+ scattered test files into 7 well-organized tests
- Removed 15 debug artifacts from troubleshooting sessions  
- Removed 11 duplicate or superseded tests
- Eliminated experimental and one-off test scripts

**September 2025**: Optimized to 16 focused tests by removing redundancy
- **Redundancy removal**: Eliminated `quick.test.ts` (duplicate of comprehensive) and `call-hierarchy.test.ts` (superseded by adaptive version)
- **Quality focus**: Each remaining test serves a distinct, validated purpose
- **Enhanced coverage**: All 28 MCP tools tested with comprehensive error handling, edge cases, and recovery scenarios
- **Structural clarity**: Logical organization with no overlapping functionality

The current 16-file structure represents optimized, non-redundant test coverage with maximum efficiency and comprehensive validation of Codebuddy functionality.