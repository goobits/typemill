<h1>Architecture Overview</h1>
<p>TypeMill is a pure Rust MCP server that bridges Model Context Protocol (MCP) with Language Server Protocol (LSP) functionality.</p>

<section>
	<h2>High-Level Architecture</h2>
	<p>The system follows a service-oriented design with clear crate separation and AI-friendly boundaries.</p>
	<pre><code>┌─────────────────────────────────────────┐
│         Application Layer               │
│         (apps/mill)                     │
└─────────────────────────────────────────┘
              ↓
┌─────────────────────────────────────────┐
│         Handlers Layer                  │
│         (mill-handlers)                 │
│  - NavigationHandler                    │
│  - RefactoringHandler                   │
│  - AnalysisHandler                      │
└─────────────────────────────────────────┘
              ↓
┌─────────────────────────────────────────┐
│         Services Layer                  │
│  - mill-services                        │
│  - mill-lsp                             │
│  - mill-ast                             │
│  - mill-plugins                         │
└─────────────────────────────────────────┘
              ↓
┌─────────────────────────────────────────┐
│         Language Plugins                │
│  - mill-lang-rust                       │
│  - mill-lang-typescript                 │
│  - mill-lang-python                     │
└─────────────────────────────────────────┘</code></pre>
</section>

<section>
	<h2>Core Components</h2>

	<h3>1. Handler System</h3>
	<p>All 28 MCP tools are implemented through specialized handlers:</p>
	<ul>
		<li><strong>NavigationHandler:</strong> Symbol navigation and code intelligence (8 tools)</li>
		<li><strong>RefactoringHandler:</strong> Code transformations with dryRun API (7 tools)</li>
		<li><strong>AnalysisHandler:</strong> Code quality and dead code analysis (8 tools)</li>
		<li><strong>WorkspaceHandler:</strong> Package management and text operations (4 tools)</li>
		<li><strong>SystemHandler:</strong> Health checks and diagnostics (1 tool)</li>
	</ul>

	<h3>2. Plugin Dispatcher</h3>
	<p>Central request orchestrator that manages plugin lifecycle and routing.</p>
	<pre><code>MCP Request → Plugin Dispatcher → Tool Registry
                                    ↓
                              Handler Lookup
                                    ↓
                            Handler.handle_tool_call()
                                    ↓
                         LSP/Plugin/Service Layer</code></pre>

	<h3>3. Language Plugins</h3>
	<p>Each language plugin implements:</p>
	<ul>
		<li>AST parsing and symbol extraction</li>
		<li>Import analysis and manipulation</li>
		<li>Refactoring operations (extract, inline, etc.)</li>
		<li>Workspace operations</li>
		<li>Project creation</li>
	</ul>

	<h3>4. LSP Integration</h3>
	<p>Direct communication with language servers for:</p>
	<ul>
		<li>Symbol navigation (definitions, references)</li>
		<li>Code intelligence (hover, completion)</li>
		<li>Diagnostics (errors, warnings)</li>
		<li>Formatting and code actions</li>
	</ul>
</section>

<section>
	<h2>Request Lifecycle</h2>
	<ol>
		<li><strong>Request Reception:</strong> Transport Layer (stdio/WebSocket) → JSON parsing</li>
		<li><strong>Plugin Dispatch:</strong> PluginDispatcher → Tool lookup</li>
		<li><strong>Handler Execution:</strong> Tool handler → Service layer</li>
		<li><strong>Service Processing:</strong> LSP/AST/File services</li>
		<li><strong>Response Generation:</strong> Result → JSON serialization → Transport</li>
	</ol>
</section>

<section>
	<h2>Unified Refactoring API</h2>
	<p>All refactoring operations use a consistent <code>dryRun</code> pattern:</p>
	<ul>
		<li><strong>Default (dryRun: true):</strong> Generate plan, preview changes, never modify files</li>
		<li><strong>Execute (dryRun: false):</strong> Apply changes with checksum validation and rollback</li>
	</ul>
	<p>Refactoring flow:</p>
	<pre><code>1. User calls tool with dryRun: true (default)
2. System generates RefactorPlan with edits
3. User reviews plan
4. User calls same tool with dryRun: false
5. System validates checksums
6. System applies changes atomically
7. On error: automatic rollback</code></pre>
</section>

<section>
	<h2>Multi-Tenancy</h2>
	<p>Production-ready multi-tenancy with JWT authentication:</p>
	<ul>
		<li>User-scoped workspaces</li>
		<li>JWT-based authentication</li>
		<li>Workspace isolation</li>
		<li>Session management</li>
	</ul>
</section>

<section>
	<h2>Performance Features</h2>
	<h3>Async Runtime</h3>
	<ul>
		<li>Tokio-based efficient async I/O</li>
		<li>Concurrent request processing</li>
		<li>Resource pooling</li>
	</ul>

	<h3>Memory Management</h3>
	<ul>
		<li>Arc-based sharing for services</li>
		<li>Lazy initialization</li>
		<li>Bounded caching with TTL</li>
	</ul>

	<h3>Zero-Cost Abstractions</h3>
	<ul>
		<li>Compile-time optimizations</li>
		<li>No garbage collection overhead</li>
		<li>Native compilation</li>
	</ul>
</section>

<section>
	<h2>Security Model</h2>
	<h3>Process Isolation</h3>
	<ul>
		<li>LSP servers run as separate child processes</li>
		<li>Workspace boundaries enforced</li>
		<li>Command validation</li>
	</ul>

	<h3>Input Validation</h3>
	<ul>
		<li>JSON Schema validation</li>
		<li>Path sanitization</li>
		<li>Type safety (Rust's type system)</li>
	</ul>
</section>

<section>
	<h2>Language Plugin Parity</h2>
	<p>100% feature parity across TypeScript, Rust, and Python:</p>
	<table>
		<thead>
			<tr>
				<th>Capability</th>
				<th>TypeScript</th>
				<th>Rust</th>
				<th>Python</th>
			</tr>
		</thead>
		<tbody>
			<tr>
				<td>Core LanguagePlugin</td>
				<td>✅</td>
				<td>✅</td>
				<td>✅</td>
			</tr>
			<tr>
				<td>Import Support (5 traits)</td>
				<td>✅</td>
				<td>✅</td>
				<td>✅</td>
			</tr>
			<tr>
				<td>Workspace Operations</td>
				<td>✅</td>
				<td>✅</td>
				<td>✅</td>
			</tr>
			<tr>
				<td>Refactoring (3 operations)</td>
				<td>✅</td>
				<td>✅</td>
				<td>✅</td>
			</tr>
			<tr>
				<td>Analysis (2 traits)</td>
				<td>✅</td>
				<td>✅</td>
				<td>✅</td>
			</tr>
			<tr>
				<td>Manifest Management</td>
				<td>✅</td>
				<td>✅</td>
				<td>✅</td>
			</tr>
			<tr>
				<td>Project Creation</td>
				<td>✅</td>
				<td>✅</td>
				<td>✅</td>
			</tr>
		</tbody>
	</table>
</section>

<section>
	<h2>Layer Architecture</h2>
	<p>The codebase follows a strict 7-layer architecture:</p>
	<ol>
		<li><strong>Support:</strong> Testing and tooling</li>
		<li><strong>Foundation:</strong> Core types, protocol, config</li>
		<li><strong>Plugin API:</strong> Language plugin contracts</li>
		<li><strong>Language Plugins:</strong> Language-specific implementations</li>
		<li><strong>Services:</strong> Business logic, LSP integration, AST</li>
		<li><strong>Handlers:</strong> MCP tool implementations</li>
		<li><strong>Application:</strong> Server, client, transport</li>
	</ol>
</section>

<section>
	<h2>Learn More</h2>
	<ul>
		<li><a href="https://github.com/goobits/typemill/tree/main/docs/architecture" target="_blank" rel="noopener">Complete Architecture Documentation</a></li>
		<li><a href="/contributing">Contributing Guide</a></li>
		<li><a href="/tools">MCP Tools Reference</a></li>
	</ul>
</section>
