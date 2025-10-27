<h1>MCP Tools Reference</h1>
<p>Complete API reference for all 28 TypeMill MCP tools organized by category.</p>

<nav class="tools-nav">
	<a href="#navigation">Navigation (8)</a>
	<a href="#refactoring">Refactoring (7)</a>
	<a href="#analysis">Analysis (8)</a>
	<a href="#workspace">Workspace (4)</a>
	<a href="#system">System (1)</a>
</nav>

<section id="navigation">
	<h2>🔍 Navigation & Intelligence (8 tools)</h2>
	<p>LSP-based tools for code navigation and symbol information.</p>

	<div class="tool">
		<h3><code>find_definition</code></h3>
		<p><strong>Purpose:</strong> Find where a symbol is defined in the codebase.</p>
		<p><strong>Parameters:</strong></p>
		<table>
			<thead>
				<tr>
					<th>Name</th>
					<th>Type</th>
					<th>Required</th>
					<th>Description</th>
				</tr>
			</thead>
			<tbody>
				<tr>
					<td>file_path</td>
					<td>string</td>
					<td>Yes</td>
					<td>Absolute path to the source file</td>
				</tr>
				<tr>
					<td>line</td>
					<td>number</td>
					<td>Yes</td>
					<td>Line number (0-indexed)</td>
				</tr>
				<tr>
					<td>character</td>
					<td>number</td>
					<td>Yes</td>
					<td>Character offset in the line (0-indexed)</td>
				</tr>
			</tbody>
		</table>
		<p><strong>Example:</strong></p>
		<pre><code>{JSON.stringify({
  "method": "tools/call",
  "params": {
    "name": "find_definition",
    "arguments": {
      "file_path": "/workspace/src/main.rs",
      "line": 10,
      "character": 5
    }
  }
}, null, 2)}</code></pre>
	</div>

	<div class="tool">
		<h3><code>find_references</code></h3>
		<p><strong>Purpose:</strong> Find all references to a symbol across the project.</p>
		<p><strong>Parameters:</strong> Same as find_definition (file_path, line, character)</p>
		<p><strong>Returns:</strong> List of locations where the symbol is referenced</p>
	</div>

	<div class="tool">
		<h3><code>search_symbols</code></h3>
		<p><strong>Purpose:</strong> Search for symbols across the entire workspace.</p>
		<p><strong>Parameters:</strong></p>
		<table>
			<thead>
				<tr>
					<th>Name</th>
					<th>Type</th>
					<th>Required</th>
					<th>Description</th>
				</tr>
			</thead>
			<tbody>
				<tr>
					<td>query</td>
					<td>string</td>
					<td>Yes</td>
					<td>Search query (supports fuzzy matching)</td>
				</tr>
			</tbody>
		</table>
	</div>

	<div class="tool">
		<h3><code>find_implementations</code></h3>
		<p><strong>Purpose:</strong> Find all implementations of an interface or trait.</p>
	</div>

	<div class="tool">
		<h3><code>find_type_definition</code></h3>
		<p><strong>Purpose:</strong> Find the underlying type definition of a symbol.</p>
	</div>

	<div class="tool">
		<h3><code>get_symbol_info</code></h3>
		<p><strong>Purpose:</strong> Get detailed information about a symbol (hover info, documentation).</p>
	</div>

	<div class="tool">
		<h3><code>get_diagnostics</code></h3>
		<p><strong>Purpose:</strong> Get errors, warnings, and hints for a file.</p>
		<p><strong>Parameters:</strong></p>
		<table>
			<thead>
				<tr>
					<th>Name</th>
					<th>Type</th>
					<th>Required</th>
					<th>Description</th>
				</tr>
			</thead>
			<tbody>
				<tr>
					<td>file_path</td>
					<td>string</td>
					<td>Yes</td>
					<td>Path to the file to check</td>
				</tr>
			</tbody>
		</table>
	</div>

	<div class="tool">
		<h3><code>get_call_hierarchy</code></h3>
		<p><strong>Purpose:</strong> Get the call hierarchy (callers and callees) for a function.</p>
	</div>
</section>

<section id="refactoring">
	<h2>✂️ Editing & Refactoring (7 tools)</h2>
	<p>All refactoring tools use a unified <code>options.dryRun</code> API:</p>
	<ul>
		<li><strong>Default behavior:</strong> <code>dryRun: true</code> (preview only, safe)</li>
		<li><strong>Execution mode:</strong> <code>dryRun: false</code> (applies changes with validation and rollback)</li>
	</ul>

	<div class="tool">
		<h3><code>rename</code></h3>
		<p><strong>Purpose:</strong> Rename symbols, files, or directories with automatic import updates.</p>
		<p><strong>Parameters:</strong></p>
		<table>
			<thead>
				<tr>
					<th>Name</th>
					<th>Type</th>
					<th>Required</th>
					<th>Description</th>
				</tr>
			</thead>
			<tbody>
				<tr>
					<td>target</td>
					<td>object</td>
					<td>Yes</td>
					<td>Target to rename (kind: symbol/file/directory)</td>
				</tr>
				<tr>
					<td>newName</td>
					<td>string</td>
					<td>Yes</td>
					<td>New name for the target</td>
				</tr>
				<tr>
					<td>options.dryRun</td>
					<td>boolean</td>
					<td>No</td>
					<td>Default: true (preview), false (execute)</td>
				</tr>
			</tbody>
		</table>
		<p><strong>Example (Preview):</strong></p>
		<pre><code>{JSON.stringify({
  "name": "rename",
  "arguments": {
    "target": {
      "kind": "symbol",
      "path": "src/app.ts",
      "selector": { "position": { "line": 15, "character": 8 } }
    },
    "newName": "newUser"
    // options.dryRun defaults to true
  }
}, null, 2)}</code></pre>
		<p><strong>Example (Execute):</strong></p>
		<pre><code>{JSON.stringify({
  "name": "rename",
  "arguments": {
    "target": { "kind": "file", "path": "src/old.rs" },
    "newName": "src/new.rs",
    "options": { "dryRun": false }
  }
}, null, 2)}</code></pre>
	</div>

	<div class="tool">
		<h3><code>extract</code></h3>
		<p><strong>Purpose:</strong> Extract code into functions, variables, or constants.</p>
		<p><strong>Kinds:</strong> function, variable, constant, module</p>
	</div>

	<div class="tool">
		<h3><code>inline</code></h3>
		<p><strong>Purpose:</strong> Inline variables or functions at their usage sites.</p>
		<p><strong>Kinds:</strong> variable, function, constant</p>
	</div>

	<div class="tool">
		<h3><code>move</code></h3>
		<p><strong>Purpose:</strong> Move symbols to another file or module.</p>
		<p><strong>Kinds:</strong> symbol, file, directory, module</p>
	</div>

	<div class="tool">
		<h3><code>reorder</code></h3>
		<p><strong>Purpose:</strong> Reorder function parameters or imports.</p>
		<p><strong>Kinds:</strong> parameters</p>
	</div>

	<div class="tool">
		<h3><code>transform</code></h3>
		<p><strong>Purpose:</strong> Apply code transformations (e.g., convert to async).</p>
		<p><strong>Kinds:</strong> async, patterns</p>
	</div>

	<div class="tool">
		<h3><code>delete</code></h3>
		<p><strong>Purpose:</strong> Delete symbols, files, or directories with smart import removal.</p>
		<p><strong>Kinds:</strong> symbol, file, directory, dead_code</p>
	</div>
</section>

<section id="analysis">
	<h2>📊 Analysis (8 tools)</h2>
	<p>Unified analysis API with consistent <code>kind</code> and <code>scope</code> parameters.</p>

	<div class="tool">
		<h3><code>analyze.quality</code></h3>
		<p><strong>Purpose:</strong> Analyze code quality metrics.</p>
		<p><strong>Kinds:</strong> complexity, smells, maintainability, readability</p>
		<p><strong>Example:</strong></p>
		<pre><code>{JSON.stringify({
  "name": "analyze.quality",
  "arguments": {
    "kind": "complexity",
    "scope": { "type": "file", "path": "src/app.ts" }
  }
}, null, 2)}</code></pre>
	</div>

	<div class="tool">
		<h3><code>analyze.dead_code</code></h3>
		<p><strong>Purpose:</strong> Detect unused code.</p>
		<p><strong>Kinds:</strong> unused_imports, unused_symbols, unused_parameters, unused_variables, unused_types, unreachable_code</p>
	</div>

	<div class="tool">
		<h3><code>analyze.dependencies</code></h3>
		<p><strong>Purpose:</strong> Analyze code dependencies.</p>
		<p><strong>Kinds:</strong> imports, graph, circular, coupling, cohesion, depth</p>
	</div>

	<div class="tool">
		<h3><code>analyze.structure</code></h3>
		<p><strong>Purpose:</strong> Analyze code structure.</p>
		<p><strong>Kinds:</strong> symbols, hierarchy, interfaces, inheritance, modules</p>
	</div>

	<div class="tool">
		<h3><code>analyze.documentation</code></h3>
		<p><strong>Purpose:</strong> Analyze documentation quality.</p>
		<p><strong>Kinds:</strong> coverage, quality, style, examples, todos</p>
	</div>

	<div class="tool">
		<h3><code>analyze.tests</code></h3>
		<p><strong>Purpose:</strong> Analyze test coverage and quality.</p>
		<p><strong>Kinds:</strong> coverage, quality, assertions, organization</p>
	</div>

	<div class="tool">
		<h3><code>analyze.batch</code></h3>
		<p><strong>Purpose:</strong> Multi-file batch analysis with optimized AST caching.</p>
	</div>

	<div class="tool">
		<h3><code>analyze.module_dependencies</code></h3>
		<p><strong>Purpose:</strong> Analyze Rust module dependencies for crate extraction.</p>
	</div>
</section>

<section id="workspace">
	<h2>📦 Workspace (4 tools)</h2>
	<p>Workspace management tools for package operations and text search.</p>

	<div class="tool">
		<h3><code>workspace.create_package</code></h3>
		<p><strong>Purpose:</strong> Create a new package in the workspace.</p>
		<p><strong>Parameters:</strong></p>
		<table>
			<thead>
				<tr>
					<th>Name</th>
					<th>Type</th>
					<th>Required</th>
					<th>Description</th>
				</tr>
			</thead>
			<tbody>
				<tr>
					<td>name</td>
					<td>string</td>
					<td>Yes</td>
					<td>Package name</td>
				</tr>
				<tr>
					<td>path</td>
					<td>string</td>
					<td>Yes</td>
					<td>Package directory path</td>
				</tr>
				<tr>
					<td>template</td>
					<td>string</td>
					<td>No</td>
					<td>Template: minimal, full (default: minimal)</td>
				</tr>
			</tbody>
		</table>
	</div>

	<div class="tool">
		<h3><code>workspace.extract_dependencies</code></h3>
		<p><strong>Purpose:</strong> Extract module dependencies for crate extraction planning.</p>
	</div>

	<div class="tool">
		<h3><code>workspace.update_members</code></h3>
		<p><strong>Purpose:</strong> Update workspace member list in Cargo.toml/package.json.</p>
	</div>

	<div class="tool">
		<h3><code>workspace.find_replace</code></h3>
		<p><strong>Purpose:</strong> Find and replace text across the entire workspace.</p>
		<p><strong>Parameters:</strong></p>
		<table>
			<thead>
				<tr>
					<th>Name</th>
					<th>Type</th>
					<th>Required</th>
					<th>Description</th>
				</tr>
			</thead>
			<tbody>
				<tr>
					<td>pattern</td>
					<td>string</td>
					<td>Yes</td>
					<td>Search pattern (supports regex)</td>
				</tr>
				<tr>
					<td>replacement</td>
					<td>string</td>
					<td>Yes</td>
					<td>Replacement text</td>
				</tr>
			</tbody>
		</table>
	</div>
</section>

<section id="system">
	<h2>💚 System (1 tool)</h2>

	<div class="tool">
		<h3><code>health_check</code></h3>
		<p><strong>Purpose:</strong> Get server health status, LSP server diagnostics, and statistics.</p>
		<p><strong>Returns:</strong></p>
		<ul>
			<li>Server status and uptime</li>
			<li>LSP server health (per language)</li>
			<li>Memory usage</li>
			<li>Active connections</li>
		</ul>
	</div>
</section>

<style>
	.tools-nav {
		display: flex;
		gap: 1rem;
		flex-wrap: wrap;
		margin: 2rem 0;
		padding: 1rem;
		background: var(--surface);
		border-radius: 8px;
	}

	.tools-nav a {
		padding: 0.5rem 1rem;
		background: var(--background);
		border: 1px solid var(--border);
		border-radius: 4px;
		text-decoration: none;
		color: var(--text);
		transition: all 0.2s;
	}

	.tools-nav a:hover {
		background: var(--accent);
		color: white;
		border-color: var(--accent);
	}

	.tool {
		margin: 2rem 0;
		padding: 1.5rem;
		background: var(--surface);
		border-left: 4px solid var(--accent);
		border-radius: 4px;
	}

	.tool h3 {
		margin-top: 0;
		color: var(--accent);
	}

	.tool h3 code {
		font-size: 1.25rem;
	}
</style>
