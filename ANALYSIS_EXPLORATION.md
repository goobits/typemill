# TypeMill Analysis Functionality - Comprehensive Exploration

## Overview

The TypeMill project has a sophisticated analysis subsystem consisting of:
1. **Analysis Crates** (`/workspace/analysis/`) - Pure analysis libraries
2. **Analysis Handlers** (`/workspace/crates/mill-handlers/src/handlers/tools/analysis/`) - MCP tool handlers
3. **Language Plugins** - Language-specific implementations (Markdown, TypeScript, Rust, Python)
4. **Shared Infrastructure** - Common types, traits, and orchestration

---

## Part 1: Analysis Crates/Modules Structure

### Directory Layout

```
analysis/
├── mill-analysis-common/           # Shared traits and types
│   ├── src/traits.rs              # LspProvider, AnalysisEngine traits
│   ├── src/types.rs               # AnalysisMetadata struct
│   ├── src/graph.rs               # DependencyGraph, SymbolNode
│   ├── src/error.rs               # AnalysisError enum
│   └── src/lib.rs
│
├── mill-analysis-graph/            # Graph structures for analysis
│   ├── src/query.rs               # GraphQuery trait (SCC, paths, transitive deps)
│   ├── src/dependency.rs          # DependencyGraph, ModuleNode
│   ├── src/call.rs                # CallGraph structures
│   ├── src/cache.rs               # Graph caching
│   └── src/lib.rs
│
├── mill-analysis-dead-code/        # Dead code detector
│   ├── src/detector.rs            # Main analysis engine (LSP-based)
│   ├── src/config.rs              # DeadCodeConfig
│   ├── src/types.rs               # DeadCodeReport, DeadSymbol, AnalysisStats
│   ├── src/utils.rs               # Helper functions
│   └── src/lib.rs                 # AnalysisEngine impl
│
├── mill-analysis-deep-dead-code/   # Advanced dead code analysis
│   ├── src/ast_parser/            # TypeScript AST parsing
│   ├── src/dead_code_finder.rs    # Deep dead code detection
│   ├── src/graph_builder.rs       # Call graph building
│   └── src/lib.rs
│
└── mill-analysis-circular-deps/    # Circular dependency detection
    ├── src/builder.rs             # DependencyGraphBuilder
    ├── src/lib.rs                 # find_circular_dependencies
    └── Cargo.toml                 # Feature-gated
```

### Key Traits and Types

#### **AnalysisEngine Trait** (`mill-analysis-common`)
```rust
#[async_trait]
pub trait AnalysisEngine: Send + Sync {
    type Config;
    type Result;

    async fn analyze(
        &self,
        lsp: Arc<dyn LspProvider>,
        workspace_path: &Path,
        config: Self::Config,
    ) -> Result<Self::Result, AnalysisError>;

    fn metadata(&self) -> AnalysisMetadata;
}
```

#### **LspProvider Trait** (`mill-analysis-common`)
Abstraction for LSP communication:
- `workspace_symbols()` - Query all workspace symbols
- `find_references()` - Find symbol references
- `document_symbols()` - Get symbols in a file

#### **DependencyGraph** (`mill-analysis-graph`)
Graph structure using petgraph:
- Nodes: `SymbolNode` (id, name, kind, file_path, is_public)
- Edges: `UsageContext` (TypeAnnotation, FunctionCall, Import, etc.)
- Methods: find_unreferenced_nodes(), add_symbol(), add_dependency()

#### **DeadCodeAnalyzer** (`mill-analysis-dead-code`)
Implements `AnalysisEngine`:
- Uses LSP workspace/symbol and textDocument/documentSymbol
- Fallback per-file analysis if workspace/symbol unavailable
- Produces `DeadCodeReport` with `DeadSymbol` findings

---

## Part 2: Analysis Capabilities Currently Implemented

### 8 Analysis Tools (MCP endpoints)

#### **1. analyze.quality**
Handler: `QualityHandler` → 4 kinds

**Kinds:**
- `complexity` - Cyclomatic/cognitive complexity analysis
  - Thresholds: cyclomatic (15), cognitive (10), nesting (4), params (5), function_length (50)
  - Outputs: ComplexityRating (Simple, Moderate, Complex, VeryComplex)
- `smells` - Code smell detection
  - Long methods (>50 SLOC), God classes (>20 methods)
  - Magic numbers (repeated constants), TODOs
- `maintainability` - Workspace-wide or file-level maintainability
  - Complexity distribution, attention ratio, avg metrics
- `readability` - Readability issues
  - Deep nesting (>4), too many parameters (>5), long functions (>50 SLOC)
  - Low comment ratio (<10% for >20 SLOC)

**Uses:** mill_ast::complexity module, language plugins for parsing

#### **2. analyze.documentation**
Handler: `DocumentationHandler` → 5 kinds

**Kinds:**
- `coverage` - Documentation coverage percentage
  - Public vs private symbol breakdown
  - Lists undocumented public symbols
- `quality` - Documentation quality assessment
  - Meaningful descriptions, parameter docs, return docs
  - Examples in complex functions (complexity > 10)
- `style` - Documentation style consistency
  - Mixed comment styles, capitalization, punctuation
- `examples` - Code example presence and coverage
  - Functions with examples, complex without examples
- `todos` - TODO/FIXME/HACK tracking
  - Categories, severity, location, count trends

**Algorithm:** Regex-based doc comment extraction, pattern matching

#### **3. analyze.dead_code**
Handler: `DeadCodeHandler` → 1 kind (configured)

**Features:**
- Uses LSP workspace/symbol query
- Reference counting via find_references
- Identifies unused functions, classes, variables
- Categorizes by symbol kind
- Reports with locations and usage stats

**Uses:** mill_analysis_dead_code crate

#### **4. analyze.dependencies**
Handler: `DependenciesHandler` → multiple kinds

**Kinds:**
- `imports` - Import statement analysis
  - Categorizes: external, internal, relative
  - Uses language plugin AST parsing
- `graph` - Full dependency graph
- `circular` - Circular dependency detection
  - SCC-based (Tarjan algorithm)
  - Reports cycle paths and import chains
- `coupling` - Module coupling strength
- `cohesion` - Module cohesion metrics

**Uses:** mill_analysis_circular_deps, language plugins

#### **5. analyze.structure**
Handler: `StructureHandler` → multiple kinds

**Kinds:**
- `symbols` - All symbols categorized by kind
- `hierarchy` - Class/module hierarchy structure
- `interfaces` - Interface/trait definitions
- `inheritance` - Inheritance chains and depth
- `modules` - Module organization patterns

**Algorithm:** Plugin symbol parsing, categorization

#### **6. analyze.tests**
Handler: `TestsHandler` → multiple kinds

**Kinds:**
- `coverage` - Test coverage ratio
  - Production functions vs test functions
- `quality` - Test quality metrics
- `assertions` - Assertion pattern analysis
- `organization` - Test organization patterns

**Uses:** Regex-based test detection (test_, #[test], it(, etc.)

#### **7. analyze.batch**
Handler: `BatchAnalysisHandler` → 1 kind

**Features:**
- Multi-file, multi-query batch analysis
- Runs multiple analysis queries in one request
- AST caching for performance
- Aggregated metrics and timing

**Use Case:** Workspace-wide analysis efficiency

#### **8. analyze.module_dependencies** (Rust-specific)
Handler: `ModuleDependenciesHandler` → 1 kind

**Features:**
- Analyzes which Cargo dependencies a module needs
- Supports file or directory analysis
- Classifies: external (crates.io), workspace (internal)
- Detects standard library usage
- Identifies unresolved imports

**Use Case:** Crate extraction workflows

---

## Part 3: Language Plugin Integration

### Plugin System Architecture

**Flow:**
1. Handler receives tool call with file path
2. Gets language plugin by file extension
3. Calls `plugin.parse()` → `ParsedSource` with symbols
4. Calls complexity analysis on parsed data
5. Calls analysis function with parsed data
6. Builds `AnalysisResult` with findings

### Current Language Support

#### **Markdown Plugin** (`mill-lang-markdown`)
**Features:**
- Parses headers as symbols (# → Module, ## → Class, ### → Function)
- Provides `ImportParser` for markdown links
- Handles file renaming in markdown references
- Regex patterns for:
  - Inline links: `[text](path)`
  - Reference definitions: `[ref]: path`
  - Image references: `![alt](image.png)`
  - Autolinks: `<path>`
  - Prose identifiers (e.g., file path mentions)

**Capabilities:**
- `with_imports()` - File reference tracking
- `ImportRenameSupport` - Rewrites links on file rename
- `ImportMoveSupport` - Updates links on file move
- `ImportMutationSupport` - Handles file mutations

**Path Handling:**
- Converts project-relative paths to file-relative paths
- Supports cross-directory link updates
- Handles # anchors in links

#### **Other Plugins** (referenced but analysis-specific logic extracted)
- **TypeScript/JavaScript** - Uses LSP parser, imports analysis
- **Rust** - Uses LSP, module dependency analysis
- **Python** - Uses LSP, complexity analysis

### Plugin Registry Usage

Accessed via `LanguagePluginRegistry`:
- `get_plugin(extension)` → get by file extension
- `supported_extensions()` → list all supported
- Used for language-specific analysis dispatch

---

## Part 4: Handlers Call Analysis

### Handler Architecture Pattern

**Pattern used by all handlers:**

```rust
pub struct <Name>Handler;

#[async_trait]
impl ToolHandler for <Name>Handler {
    fn tool_names(&self) -> &[&str] {
        &["analyze.<domain>"]
    }

    fn is_internal(&self) -> bool {
        false  // PUBLIC tool
    }

    async fn handle_tool_call(
        &self,
        context: &ToolHandlerContext,
        tool_call: &ToolCall,
    ) -> ServerResult<Value> {
        // Parse args
        let kind = extract_kind(&args)?;
        
        // Dispatch based on kind
        match kind {
            "subkind1" => run_analysis(context, tool_call, "category", "subkind1", detect_fn).await,
            "subkind2" => /* different logic */,
            ...
        }
    }
}
```

### Analysis Engine (`engine.rs`)

**Core function:** `run_analysis()`

**Workflow:**
1. Parse `ScopeParam` (type, path, include/exclude)
2. Extract file path
3. Read file content
4. Get language plugin by extension
5. Parse file with plugin
6. Run complexity analysis
7. Execute custom analysis function
8. Build `AnalysisResult` from findings
9. Finalize with timing
10. Serialize to JSON

**Type signature:**
```rust
pub type AnalysisFn = fn(
    &mill_ast::complexity::ComplexityReport,
    &str,  // content
    &[mill_plugin_api::Symbol],
    &str,  // language
    &str,  // file_path
    &crate::LanguagePluginRegistry,
) -> Vec<Finding>;
```

### Analysis Function Implementations

Example pattern from `documentation.rs`:

```rust
pub fn detect_coverage(
    _complexity_report: &mill_ast::complexity::ComplexityReport,
    content: &str,
    symbols: &[Symbol],
    language: &str,
    file_path: &str,
    _registry: &crate::LanguagePluginRegistry,
) -> Vec<Finding> {
    // 1. Extract documentable symbols
    let documentable: Vec<&Symbol> = symbols.iter()
        .filter(|s| matches!(s.kind, SymbolKind::Function | SymbolKind::Class | ...))
        .collect();

    // 2. Check doc status for each symbol
    for symbol in &documentable {
        let is_public = is_symbol_public(symbol, &lines, language);
        let has_doc = has_doc_comment(symbol, &lines, language);
        // ...
    }

    // 3. Generate findings
    findings.push(Finding {
        id: format!("doc-coverage-{}", file_path),
        kind: "coverage".to_string(),
        severity: determine_severity(coverage_percentage),
        location: FindingLocation { /* ... */ },
        metrics: Some(metrics_map),
        message: format!("Documentation coverage: {:.1}%", coverage_percentage),
        suggestions: vec![/* suggestions */],
    });

    findings
}
```

---

## Part 5: Markdown Plugin Details

### Design

**Purpose:** Treat markdown file references as "imports" for rename/move tracking

**Not processed:**
- Code blocks (triple backticks)
- Inline code (single backticks)
- HTML `<a href="">` tags

### Capabilities Implementation

#### **ImportParser**
Detects file references in markdown:
- Inline links: `[API](docs/api.md)`
- Reference definitions: `[ref]: docs/api.md`
- Image references: `![alt](images/logo.png)`
- Autolinks: `<docs/api.md>`

#### **ImportRenameSupport**
Two-pass rewriting:
1. Project-relative path matching
2. File-relative path matching

With optional prose identifier update (when `update_markdown_prose=true`):
- Finds identifiers like `docs/development/contributing.md` in prose
- Updates to new path

#### **ImportMoveSupport**
Handles file moves with relative path recalculation

#### **ImportMutationSupport**
Tracks mutations from rename/move operations

### Key Functions

**Path Computation:**
```rust
// File-relative: from current file's directory
let old_relative = pathdiff::diff_paths(old_path, current_dir)?;
let new_relative = pathdiff::diff_paths(new_path, current_dir)?;

// Project-relative: from workspace root
let old_project_relative = old_path.strip_prefix(project_root)?;
```

**Prose Updating:**
- Identifies identifiers matching old path pattern
- Replaces with new path
- Context-aware (checks surrounding text)

---

## Part 6: Data Structures and Results

### Finding Structure (`mill_foundation`)

```rust
pub struct Finding {
    pub id: String,                    // Unique ID
    pub kind: String,                  // Type (e.g., "long_method")
    pub severity: Severity,            // High/Medium/Low
    pub location: FindingLocation,     // File, range, symbol
    pub metrics: Option<HashMap>,      // Numeric data
    pub message: String,               // Human-readable
    pub suggestions: Vec<Suggestion>,  // Actionable fixes
}

pub struct FindingLocation {
    pub file_path: String,
    pub range: Option<Range>,          // Start/end positions
    pub symbol: Option<String>,        // Symbol name
    pub symbol_kind: Option<String>,   // Function, class, etc.
}
```

### AnalysisResult Structure

```rust
pub struct AnalysisResult {
    pub category: String,              // "quality", "documentation"
    pub kind: String,                  // "complexity", "coverage"
    pub scope: AnalysisScope,
    pub metadata: AnalysisMetadata,
    pub findings: Vec<Finding>,
    pub summary: AnalysisSummary,
}

pub struct AnalysisSummary {
    pub total_findings: usize,
    pub files_analyzed: usize,
    pub symbols_analyzed: Option<usize>,
    pub findings_by_severity: HashMap<String, usize>,
    pub analysis_time_ms: u64,
    // ... more fields
}
```

### Suggestion Structure

```rust
pub struct Suggestion {
    pub action: String,                // Action identifier
    pub description: String,           // Human-readable description
    pub target: Option<String>,        // Refactor target (optional)
    pub estimated_impact: String,      // Expected improvement
    pub safety: SafetyLevel,          // Safe/RequiresReview/Dangerous
    pub confidence: f64,               // 0.0-1.0
    pub reversible: bool,              // Can be undone
    pub refactor_call: Option<Value>,  // MCP call to execute refactor
}
```

---

## Part 7: Quality and Scope Details

### Quality Analysis Deep Dive

**Complexity Metrics:**
- **Cyclomatic Complexity:** Decision points (if/while/for)
- **Cognitive Complexity:** Complexity adjusted for readability
- **Nesting Depth:** Max nesting level
- **Parameter Count:** Number of function parameters
- **SLOC:** Source Lines of Code

**Severity Levels by Ratio:**
- Functions with attention needed > 30% → High severity
- Functions with attention needed > 10% → Medium severity
- Otherwise → Low severity

**Code Smells Detected:**
1. Long methods (>50 SLOC → Medium, >100 → High)
2. God classes (>20 methods)
3. Magic numbers (repeated constants)
4. Deep nesting (>4 levels → Medium, >6 → High)
5. Too many parameters (>5 → Medium, >7 → High)
6. Low comment ratio (<10% for >20 SLOC)

### Workspace Scope Analysis

**Feature:** analyze.quality with `scope.type="workspace"`

**Process:**
1. Lists all files in directory
2. Filters to analyzable files by extension
3. Iterates each file:
   - Reads content
   - Gets plugin for extension
   - Parses file
   - Runs complexity analysis
   - Aggregates stats
4. Calculates:
   - Total functions, files, SLOC
   - Average cyclomatic/cognitive complexity
   - Max complexity
   - Ratio needing attention
5. Determines workspace-level severity
6. Returns aggregated findings

---

## Part 8: Batch Analysis Architecture

### BatchAnalysisHandler

**Purpose:** Execute multiple analysis queries in one request

**Request Structure:**
```json
{
  "queries": [
    {
      "command": "analyze.quality",
      "kind": "complexity",
      "scope": {"type": "file", "path": "src/main.rs"}
    },
    {
      "command": "analyze.documentation",
      "kind": "coverage",
      "scope": {"type": "file", "path": "src/main.rs"}
    }
  ]
}
```

**Response Structure:**
```rust
pub struct BatchAnalysisResult {
    pub results: Vec<SingleQueryResult>,
    pub summary: BatchSummary,
    pub metadata: BatchMetadata,
}
```

**Optimization:** AST caching
- Parses file once
- Reuses `ParsedSource` for multiple analyses
- Tracks cache hits/misses

### Module Dependency Analysis (Rust)

**Purpose:** Determine Cargo dependencies needed by a module

**Analysis Process:**
1. Parse use statements from file/directory
2. Cross-reference with Cargo.toml
3. Classify dependencies:
   - External (crates.io)
   - Workspace (internal crates)
   - Std library
4. Resolve versions from manifest
5. Report with usage counts

**Use Case:** Support crate extraction workflows

---

## Part 9: Integration Points

### How Analysis Integrates with System

**1. Handler Registration (plugin_dispatcher.rs)**
```rust
let mut registry = self.tool_registry.lock().await;
register_handlers_with_logging!(registry, {
    QualityHandler => "1 tool (analyze.quality)",
    DeadCodeHandler => "1 tool (analyze.dead_code)",
    DependenciesHandler => "1 tool (analyze.dependencies)",
    StructureHandler => "1 tool (analyze.structure)",
    DocumentationHandler => "1 tool (analyze.documentation)",
    TestsHandler => "1 tool (analyze.tests)",
    BatchAnalysisHandler => "1 tool (analyze.batch)",
    ModuleDependenciesHandler => "1 tool (analyze.module_dependencies)",
    // ... more handlers
});
```

**2. File Service Integration**
- Handlers use `context.app_state.file_service.read_file()`
- Respects file locking, caching, virtual workspaces

**3. Language Plugin Integration**
- `context.app_state.language_plugins.get_plugin(ext)`
- `plugin.parse(content)` → `ParsedSource` with symbols

**4. LSP Integration**
- Deep analysis uses LSP queries (workspace/symbol, find_references)
- Supports language servers for TypeScript, Rust, Python

**5. Complexity Analysis Integration**
- `mill_ast::complexity::analyze_file_complexity()`
- Provides metrics used by quality and documentation handlers

### Suggestion Generation Pipeline

**Pattern used across handlers:**

1. Detect issue in analysis function
2. Create `RefactoringCandidate` with:
   - Refactor type (ExtractMethod, etc.)
   - Location, message
   - Evidence strength
   - Side effects, recursion info
3. Generate suggestions via `SuggestionGenerator`
4. Add suggestions to Finding

**Refactor Types in Candidates:**
- ExtractMethod
- ExtractConstant
- ExtractVariable
- ConsolidateParameters
- SplitClass
- ReduceParameters
- (TODO: More types for comprehensive refactoring)

---

## Summary: Key Capabilities

### Analysis Coverage Matrix

| Domain | Tool | Kinds | Uses LSP | Uses Plugin | File/Workspace |
|--------|------|-------|----------|-------------|-----------------|
| Quality | analyze.quality | 4 | No | Yes | Both |
| Documentation | analyze.documentation | 5 | No | Yes | File |
| Dead Code | analyze.dead_code | 1 | Yes | Yes | Workspace |
| Dependencies | analyze.dependencies | 4+ | Yes/No | Yes | Both |
| Structure | analyze.structure | 4+ | No | Yes | File |
| Tests | analyze.tests | 4+ | No | Yes | File |
| Batch | analyze.batch | 1 | Yes/No | Yes | File |
| Module Deps | analyze.module_dependencies | 1 | No | Yes | File/Dir |

### Markdown Plugin Features

- **Link Detection:** Inline, reference, image, autolinks
- **Path Rewriting:** Project-relative → file-relative
- **Prose Updating:** Plain-text path identifier updates
- **Cross-directory Support:** Handles moves across directories
- **Header Extraction:** Headers as symbols for navigation

---

## Architecture Insight

**Key Design Pattern:**

The analysis system uses a **plugin-first, LSP-enhanced approach**:

1. **Plugin Parsing** is primary (language plugins parse all files)
2. **LSP Queries** supplement where deeper analysis needed (dead code, imports)
3. **Handlers Dispatch** based on kind parameter
4. **Analysis Functions** focus on detection logic, not orchestration
5. **Suggestions** generated from candidates for user-actionable recommendations

**This enables:**
- Consistent error handling across handlers
- Language-agnostic analysis framework
- Easy addition of new analysis types
- Reusable orchestration (run_analysis)
- Integration with refactoring system

---

## Proposed Enhancements (from code TODOs)

1. **Duplicate Code Detection** - Token-based similarity analysis
2. **AST-based Doc Comment Extraction** - More accurate doc parsing
3. **Workspace-wide Coverage** - Multiple files in one analysis
4. **Stale TODOs Detection** - Track TODO age
5. **Documentation Staleness** - Detect outdated docs
6. **Test Coverage via Instrumentation** - Real coverage metrics
7. **Symbol Complexity Scores** - More granular complexity data
8. **Visibility Analysis** - Full AST-based instead of heuristic
9. **Additional Refactor Types** - More refactoring suggestions
10. **Language Plugin Extensions** - More language support

