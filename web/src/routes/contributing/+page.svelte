<h1>Contributing to TypeMill</h1>
<p>Thank you for considering contributing! It's people like you that make TypeMill such a great tool.</p>

<section>
	<h2>Getting Started</h2>

	<h3>Prerequisites</h3>
	<ul>
		<li><strong>Rust Toolchain:</strong> Get it from <a href="https://rustup.rs" target="_blank" rel="noopener">rustup.rs</a></li>
		<li><strong>Java SDK & Maven:</strong> Required for Java parser</li>
		<li><strong>.NET SDK:</strong> Required for C# parser</li>
		<li><strong>Node.js & npm:</strong> Required for TypeScript parser</li>
		<li><strong>Git:</strong> For cloning the repository</li>
	</ul>

	<h3>Developer Setup</h3>
	<pre><code># Clone the repository
git clone https://github.com/goobits/typemill.git
cd mill

# Run the first-time setup command
make first-time-setup

# This single command will:
# - Check prerequisites
# - Install dev tools (sccache, mold)
# - Build language parsers
# - Build the main project
# - Run tests</code></pre>
</section>

<section>
	<h2>Running Tests</h2>
	<p>This project uses <a href="https://nexte.st/" target="_blank" rel="noopener">cargo-nextest</a> for faster test execution.</p>

	<pre><code># Run fast tests (recommended for local development)
make test

# Run the full test suite, including skipped tests
make test-full

# Run tests that require LSP servers
make test-lsp

# Run specific test file
cargo nextest run --test lsp_features

# Run with detailed output
cargo nextest run --no-capture</code></pre>

	<h3>Focused Development Workflows</h3>
	<pre><code># Analysis crates only (extremely fast: ~0.02s)
make test-analysis

# Handlers only (fast: ~0.1s for 37 tests)
make test-handlers

# Core libraries (excludes integration tests)
make test-core

# Language plugins only (fast: ~0.3s for 193 tests)
make test-lang

# Watch mode (auto-rebuild on file changes)
make dev-handlers
make dev-analysis
make dev-core
make dev-lang</code></pre>
</section>

<section>
	<h2>Code Style and Linting</h2>
	<pre><code># Format code
cargo fmt --all

# Run clippy
cargo clippy --all-targets -- -D warnings

# Or use Makefile
make clippy

# Run all checks (fmt + clippy + test + audit + deny)
make check

# Check for duplicate code & complexity
make check-duplicates</code></pre>
</section>

<section>
	<h2>Build Automation (xtask)</h2>
	<p>This project uses the <strong>xtask pattern</strong> for build automation. Instead of shell scripts, we write automation tasks in Rust for cross-platform compatibility.</p>

	<pre><code># Install mill
cargo xtask install

# Run all checks
cargo xtask check-all

# Check for duplicate code
cargo xtask check-duplicates

# Check cargo features
cargo xtask check-features

# Create new language plugin
cargo xtask new-lang python

# See all available commands
cargo xtask --help</code></pre>
</section>

<section>
	<h2>Adding New MCP Tools</h2>

	<h3>Adding a Tool to an Existing Handler</h3>
	<p>For step-by-step guide, see the <a href="https://github.com/goobits/typemill/blob/main/contributing.md#adding-a-tool-to-an-existing-handler" target="_blank" rel="noopener">contributing guide</a>.</p>

	<ol>
		<li>Open the appropriate handler file (e.g., <code>crates/mill-handlers/src/handlers/tools/navigation.rs</code>)</li>
		<li>Add tool name to <code>TOOL_NAMES</code> constant</li>
		<li>Add match arm in <code>handle_tool_call</code> method</li>
		<li>Implement the tool method</li>
		<li>Add tests</li>
	</ol>

	<h3>Creating a New Handler</h3>
	<p>Create a new handler when adding a category of related tools:</p>
	<ol>
		<li>Create handler file: <code>crates/mill-handlers/src/handlers/tools/diagnostics.rs</code></li>
		<li>Implement <code>ToolHandler</code> trait</li>
		<li>Register handler in <code>plugin_dispatcher.rs</code></li>
		<li>Add documentation and tests</li>
	</ol>
</section>

<section>
	<h2>Adding New Language Plugins</h2>
	<p>See the <a href="https://github.com/goobits/typemill/blob/main/docs/DEVELOPMENT.md" target="_blank" rel="noopener">Language Plugins Guide</a> for complete instructions.</p>

	<h3>Quick Start</h3>
	<pre><code># Generate plugin scaffolding
cargo xtask new-lang go

# Implement the LanguagePlugin trait
# - metadata(): Language information
# - parse(): AST parsing
# - analyze_manifest(): Package manifest parsing
# - list_functions(): Function extraction

# Implement capability traits
# - ImportParser
# - ImportRenameSupport
# - RefactoringProvider
# - WorkspaceSupport
# - ManifestUpdater
# - ProjectFactory</code></pre>

	<h3>Capability Trait Pattern</h3>
	<p>The codebase uses a capability-based dispatch pattern where plugins expose capabilities via traits:</p>
	<pre><code>// Query for capability
let updater = plugin.manifest_updater()
    .ok_or_else(|| anyhow!("Not supported"))?;

// Use the capability (works for ANY plugin)
updater.update_dependency(manifest_path, old, new).await?;</code></pre>
</section>

<section>
	<h2>Pull Request Process</h2>
	<ol>
		<li>Create a feature branch: <code>git checkout -b your-feature-name</code></li>
		<li>Make your changes and commit with descriptive messages</li>
		<li>Ensure tests pass: <code>make test</code></li>
		<li>Run code quality checks: <code>make check</code></li>
		<li>Push to your branch: <code>git push origin your-feature-name</code></li>
		<li>Open a pull request on GitHub with clear title and description</li>
	</ol>
</section>

<section>
	<h2>Dependency Management</h2>
	<p>Before adding new dependencies:</p>
	<ol>
		<li>Check if functionality already exists in the workspace</li>
		<li>Evaluate maintenance status, license, security, binary size</li>
		<li>Run dependency checks: <code>cargo deny check</code></li>
	</ol>

	<pre><code># Check all: advisories, licenses, bans, sources
cargo deny check
make deny

# Check only security advisories
cargo deny check advisories

# Update advisory database
cargo deny fetch
make deny-update</code></pre>
</section>

<section>
	<h2>Build Performance Tips</h2>
	<p>The project uses several build optimizations configured automatically:</p>
	<ul>
		<li><strong>sccache:</strong> Compilation cache for faster rebuilds</li>
		<li><strong>mold:</strong> Modern, fast linker (3-10x faster)</li>
		<li><strong>Dependency optimization:</strong> Dependencies compiled with <code>-O2</code> in dev mode</li>
	</ul>

	<pre><code># Check sccache statistics
sccache --show-stats

# Fast feedback during development
cargo check

# Build only changed code
cargo build

# Full rebuild (slow, use only when necessary)
cargo clean && cargo build</code></pre>

	<h3>Build Times Reference</h3>
	<table>
		<thead>
			<tr>
				<th>Build Type</th>
				<th>Time (First)</th>
				<th>Time (Incremental)</th>
			</tr>
		</thead>
		<tbody>
			<tr>
				<td><code>cargo check</code></td>
				<td>~30s</td>
				<td>2-5s</td>
			</tr>
			<tr>
				<td><code>cargo build</code></td>
				<td>~2m</td>
				<td>5-20s</td>
			</tr>
			<tr>
				<td><code>cargo build --release</code></td>
				<td>~3m</td>
				<td>30-60s</td>
			</tr>
			<tr>
				<td><code>cargo nextest run</code></td>
				<td>~2m</td>
				<td>8-25s</td>
			</tr>
		</tbody>
	</table>
</section>

<section>
	<h2>Documentation Standards</h2>
	<p>All documentation follows these principles:</p>
	<ul>
		<li><strong>Accuracy First:</strong> Every statement reflects current code reality</li>
		<li><strong>Concise & Dense:</strong> Maximum information density, minimum word count</li>
		<li><strong>Single Source of Truth:</strong> One canonical location per topic</li>
		<li><strong>Up-to-date:</strong> Synchronized with codebase changes</li>
	</ul>
</section>

<section>
	<h2>Best Practices</h2>

	<h3>Structured Logging</h3>
	<pre><code>{`// ✅ Good - structured logging
debug!(tool_name = %tool_call.name, file_path = %path, "Processing tool call");
error!(error = %e, tool = "get_diagnostics", "Tool execution failed");

// ❌ Bad - string interpolation
debug!("Processing tool call {} for file {}", tool_call.name, path);`}</code></pre>

	<h3>Error Handling</h3>
	<pre><code>// ✅ Good
let file_path = args["file_path"]
    .as_str()
    .ok_or_else(|| ServerError::InvalidRequest(
        "Missing required parameter 'file_path'"
    ))?;

// ❌ Bad
let file_path = args["file_path"].as_str().unwrap();</code></pre>
</section>

<section>
	<h2>Need Help?</h2>
	<ul>
		<li><strong>Issues:</strong> <a href="https://github.com/goobits/typemill/issues" target="_blank" rel="noopener">GitHub Issues</a></li>
		<li><strong>Discussions:</strong> <a href="https://github.com/goobits/typemill/discussions" target="_blank" rel="noopener">GitHub Discussions</a></li>
		<li><strong>Documentation:</strong> <a href="https://github.com/goobits/typemill/tree/main/docs" target="_blank" rel="noopener">Complete Docs</a></li>
	</ul>
</section>
