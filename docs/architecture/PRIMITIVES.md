# Code Primitives: The Foundation of Codebuddy

## Overview

Codebuddy's design philosophy is built on two foundational pillars that work together to provide comprehensive code intelligence and transformation capabilities. These primitives represent the "DNA" of developer tooling — minimal, composable building blocks that combine to solve complex software engineering challenges.

This document explains the conceptual framework underlying Codebuddy's tool design and how these primitives map to actual MCP tools in the system.

---

## The Two Pillars

### Pillar 1: Refactoring Primitives (Code Transformation)

Refactoring primitives are atomic operations for restructuring or improving code without changing its external behavior. Each primitive represents a single, focused transformation that can be composed with others to achieve complex refactoring goals.

### Pillar 2: Analysis Primitives (Code Understanding)

Analysis primitives power insight and precision before refactoring happens. These tools help understand code health, structure, and relationships, enabling intelligent decision-making about what transformations to apply.

Together, these pillars form a complete code intelligence ecosystem: **analysis informs refactoring, and refactoring builds on analysis**.

---

## Pillar 1: Refactoring Primitives

### Core Primitive Operations

#### 1. Rename
**Concept**: Change the name of a symbol (variable, function, class, module, or file) throughout the codebase.

**Implemented Tools**:
- `rename_symbol` - Rename variables, functions, classes, and other symbols
- `rename_file` - Rename files with automatic import updates
- `rename_directory` - Rename directories with automatic import updates

**Key Characteristics**:
- Scope-aware (local vs. global scope)
- Reference tracking (find all usages)
- Import/export updates
- Cross-file consistency

**Example Use Cases**:
- Improving code clarity with better naming
- Aligning with naming conventions
- Refactoring API surface areas

---

#### 2. Extract
**Concept**: Pull a block of code into its own function, file, or module for better organization and reusability.

**Implemented Tools**:
- `extract_function` - Extract code block into a new function
- `extract_variable` - Extract expression into a named variable
- `extract_module_to_package` - Extract module into a separate package

**Key Characteristics**:
- Scope preservation (captures necessary parameters)
- Return value detection
- Dependency analysis
- Import generation

**Example Use Cases**:
- Breaking down large functions
- Eliminating code duplication
- Creating reusable components
- Modularizing monolithic codebases

---

#### 3. Inject / Insert
**Concept**: Add code to an existing structure (imports, function parameters, class members).

**Implemented Tools**:
- `apply_edits` - Apply multiple edits atomically across files
- `write_file` - Write new content to files
- Code actions from `get_code_actions` - Add missing imports, implement interfaces

**Key Characteristics**:
- Position-aware insertion
- Syntax preservation
- Conflict detection
- Multi-file atomicity

**Example Use Cases**:
- Adding missing imports
- Implementing interface requirements
- Adding logging statements
- Inserting type annotations

---

#### 4. Move
**Concept**: Relocate code between files or directories while maintaining functionality.

**Implemented Tools**:
- `rename_file` - Move files to new locations
- `rename_directory` - Move entire directories
- `extract_module_to_package` - Move module to new package structure

**Key Characteristics**:
- Import/export rewiring
- Reference updating
- Namespace preservation
- Dependency tracking

**Example Use Cases**:
- Reorganizing project structure
- Creating feature modules
- Consolidating related code
- Migrating to new architecture

---

#### 5. Inline
**Concept**: Replace a reference with its value or implementation, reducing indirection.

**Implemented Tools**:
- `inline_variable` - Replace variable references with the variable's value
- `inline_function` - Replace function calls with function body

**Key Characteristics**:
- Scope-aware replacement
- Single/multiple occurrence handling
- Side effect preservation
- Type safety maintenance

**Example Use Cases**:
- Eliminating unnecessary variables
- Simplifying overly abstract code
- Performance optimization
- Removing dead abstractions

---

#### 6. Reorder
**Concept**: Change the sequence of code elements for clarity or convention compliance.

**Implemented Tools**:
- Code actions from `get_code_actions` - Organize imports, reorder members
- `format_document` - Format code according to style guidelines

**Key Characteristics**:
- Semantic preservation
- Convention awareness
- Dependency respect
- Style guide compliance

**Example Use Cases**:
- Organizing imports alphabetically
- Grouping related methods
- Following language conventions
- Improving readability

---

#### 7. Transform
**Concept**: Modify code structure while preserving behavior (control flow, data structures).

**Implemented Tools**:
- `get_code_actions` - Provides transformation suggestions
- `format_document` - Apply formatting transformations
- `apply_edits` - Execute complex transformations

**Key Characteristics**:
- Behavior preservation
- Pattern recognition
- Idiomatic conversion
- Type preservation

**Example Use Cases**:
- Converting loops to functional patterns
- Modernizing syntax
- Applying design patterns
- Refactoring for performance

---

### Optional: Delete and Duplicate

#### Delete
**Implemented Tools**:
- `delete_file` - Remove files from the workspace
- Code actions - Remove unused imports, dead code elimination

**Key Characteristics**:
- Dependency detection
- Safe removal validation
- Cascade cleanup

---

#### Duplicate
**Concept**: Copy code snippets or structures.

**Implemented Through**:
- `read_file` + `write_file` combinations
- `apply_edits` for targeted duplication

**Key Characteristics**:
- Conflict avoidance
- Namespace collision detection
- Reference independence

---

## Pillar 2: Analysis Primitives

Analysis primitives provide the intelligence layer that informs refactoring decisions. These tools scan, measure, and report on code structure, quality, and relationships.

### Core Analysis Operations

#### 1. Linting
**Concept**: Enforce style and detect simple errors.

**Implemented Tools**:
- `get_diagnostics` - Real-time error and warning detection
- `get_code_actions` - Quick fixes for linting issues

**Key Characteristics**:
- Real-time feedback
- Configurable rule sets
- Actionable suggestions
- Integration with language servers

**Example Use Cases**:
- Enforcing code style
- Detecting common mistakes
- Ensuring type safety
- Maintaining code quality

---

#### 2. Complexity Analysis
**Concept**: Measure how complicated a function or module is (cyclomatic complexity, nesting depth).

**Implemented Tools**:
- `get_document_symbols` - Analyze code structure
- `prepare_call_hierarchy` - Understand call complexity
- `find_references` - Measure usage complexity

**Key Characteristics**:
- Quantitative metrics
- Threshold-based warnings
- Hotspot identification
- Refactoring prioritization

**Example Use Cases**:
- Identifying refactoring candidates
- Code review guidance
- Technical debt measurement
- Maintainability tracking

---

#### 3. Dead Code Detection
**Concept**: Find unused or unreachable code.

**Implemented Tools**:
- `find_dead_code` - Identify unused exports and functions
- `find_references` - Verify symbol usage
- `analyze_imports` - Detect unused imports

**Key Characteristics**:
- Whole-program analysis
- Export tracking
- Import validation
- Safe removal suggestions

**Example Use Cases**:
- Cleaning up legacy code
- Reducing bundle size
- Improving compile times
- Eliminating technical debt

---

#### 4. Code Smell Detection
**Concept**: Identify patterns suggesting poor structure.

**Implemented Tools**:
- `get_diagnostics` - Detect anti-patterns
- `get_code_actions` - Suggest improvements
- `find_dead_code` - Identify unused code smell

**Common Code Smells**:
- Long functions (extract function candidate)
- Duplicate code (extract to shared function)
- Large classes (split into modules)
- Deep nesting (flatten control flow)

**Key Characteristics**:
- Pattern recognition
- Heuristic-based detection
- Refactoring suggestions
- Context-aware analysis

---

#### 5. Dependency Analysis
**Concept**: Map out relationships between modules, functions, and files.

**Implemented Tools**:
- `analyze_imports` - Build dependency graphs
- `find_references` - Track symbol dependencies
- `prepare_call_hierarchy` - Analyze function call relationships
- `get_call_hierarchy_incoming_calls` / `get_call_hierarchy_outgoing_calls` - Detailed call graphs

**Key Characteristics**:
- Graph construction
- Circular dependency detection
- Impact analysis
- Layering validation

**Example Use Cases**:
- Refactoring impact assessment
- Architectural analysis
- Breaking circular dependencies
- Module boundary definition

---

## Primitive Composition

The power of this framework comes from composing primitives to achieve complex goals.

### Example: Safe Module Extraction

**Goal**: Extract a large file into multiple smaller modules.

**Primitive Sequence**:
1. **Analyze Dependencies** (`analyze_imports`) - Understand current structure
2. **Detect Complexity** (`get_document_symbols`) - Identify extraction candidates
3. **Extract Functions** (`extract_function`) - Pull out logical units
4. **Move to New Files** (`rename_file` + `write_file`) - Create new module structure
5. **Update Imports** (automatic via `rename_file`) - Maintain references
6. **Verify No Dead Code** (`find_dead_code`) - Ensure clean migration
7. **Format All Files** (`format_document`) - Apply consistent style

---

### Example: Performance Optimization Refactor

**Goal**: Optimize a slow function while maintaining behavior.

**Primitive Sequence**:
1. **Analyze Complexity** (`prepare_call_hierarchy`) - Identify hot paths
2. **Inline Hot Variables** (`inline_variable`) - Reduce overhead
3. **Extract Reusable Parts** (`extract_function`) - Enable caching
4. **Verify References** (`find_references`) - Ensure no breaking changes
5. **Run Diagnostics** (`get_diagnostics`) - Check for introduced errors
6. **Transform Patterns** (`get_code_actions`) - Apply optimization patterns

---

## Design Principles

### 1. Atomicity
Each primitive represents a **single, focused operation**. This ensures:
- Clear semantics
- Easy testing
- Predictable composition
- Minimal side effects

### 2. Composability
Primitives **combine to solve complex problems**. This enables:
- Flexible workflows
- Reusable building blocks
- Incremental refactoring
- Custom automation

### 3. Language Independence
Primitives are **conceptually universal** across programming languages. This supports:
- Consistent user experience
- Plugin architecture
- Multi-language projects
- Transferable knowledge

### 4. Safety First
All primitives **preserve correctness**. This guarantees:
- No breaking changes
- Type safety preservation
- Reference integrity
- Atomic transactions

---

## Mapping to Codebuddy Tools

### Refactoring Primitives → MCP Tools

| Primitive | MCP Tools | Handler |
|-----------|-----------|---------|
| **Rename** | `rename_symbol`, `rename_file`, `rename_directory` | EditingHandler, FileOpsHandler, WorkspaceHandler |
| **Extract** | `extract_function`, `extract_variable`, `extract_module_to_package` | RefactoringHandler, WorkspaceHandler |
| **Inject/Insert** | `apply_edits`, `write_file`, code actions | EditingHandler, FileOpsHandler |
| **Move** | `rename_file`, `rename_directory` | FileOpsHandler, WorkspaceHandler |
| **Inline** | `inline_variable`, `inline_function` | RefactoringHandler |
| **Reorder** | `format_document`, code actions | EditingHandler |
| **Transform** | `get_code_actions`, `apply_edits` | EditingHandler |
| **Delete** | `delete_file`, code actions | FileOpsHandler |

### Analysis Primitives → MCP Tools

| Primitive | MCP Tools | Handler |
|-----------|-----------|---------|
| **Linting** | `get_diagnostics`, `get_code_actions` | NavigationHandler, EditingHandler |
| **Complexity** | `get_document_symbols`, `prepare_call_hierarchy` | NavigationHandler |
| **Dead Code** | `find_dead_code`, `find_references`, `analyze_imports` | WorkspaceHandler, NavigationHandler |
| **Code Smells** | `get_diagnostics`, `get_code_actions` | NavigationHandler, EditingHandler |
| **Dependencies** | `analyze_imports`, `find_references`, call hierarchy tools | WorkspaceHandler, NavigationHandler |

---

## Future Primitive Extensions

As Codebuddy evolves, additional primitives may be added:

### Potential Refactoring Primitives
- **Merge** - Combine multiple functions/modules
- **Split** - Break one entity into multiple
- **Wrap** - Add abstraction layer (e.g., try/catch, logging)
- **Unwrap** - Remove abstraction layer

### Potential Analysis Primitives
- **Performance Profiling** - Runtime hotspot detection
- **Security Analysis** - Vulnerability scanning
- **Test Coverage** - Coverage gap identification
- **Documentation Quality** - Comment/doc completeness

---

## Related Documentation

- **[API_REFERENCE.md](../../API_REFERENCE.md)** - Complete MCP tool API reference
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - System architecture and design
- **[WORKFLOWS.md](../features/WORKFLOWS.md)** - Intent-based workflow automation
- **[CONTRIBUTING.md](../../CONTRIBUTING.md)** - Adding new tools and primitives

---

## Summary

Codebuddy's primitive-based architecture provides:

1. **Clear Mental Model** - Easy to understand tool capabilities
2. **Composable Operations** - Build complex workflows from simple parts
3. **Language Agnostic** - Universal concepts across programming languages
4. **Safety Guarantees** - Correctness-preserving transformations
5. **Extensible Design** - Easy to add new primitives

By organizing all 44+ MCP tools into these two pillars (Refactoring and Analysis), Codebuddy provides a complete foundation for AI-assisted code intelligence and transformation.
