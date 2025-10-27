<h1>Documentation</h1>
<p>Complete guide to using and deploying TypeMill</p>

<section>
	<h2>Installation</h2>
	<h3>Recommended Method</h3>
	<pre><code>curl -fsSL https://raw.githubusercontent.com/goobits/mill/main/install.sh | bash</code></pre>

	<h3>Build from Source</h3>
	<pre><code>cargo install mill --locked</code></pre>
</section>

<section>
	<h2>Configuration</h2>
	<h3>Auto-Detection Setup</h3>
	<p>The smart setup wizard automatically detects your project languages and configures LSP servers:</p>
	<pre><code>mill setup</code></pre>

	<h3>Manual Configuration</h3>
	<p>Create <code>.typemill/config.json</code> in your project:</p>
	<pre><code>{JSON.stringify({
  "servers": [
    {
      "extensions": ["ts", "tsx", "js", "jsx"],
      "command": ["typescript-language-server", "--stdio"],
      "restartInterval": 10
    },
    {
      "extensions": ["rs"],
      "command": ["rust-analyzer"],
      "restartInterval": 30
    },
    {
      "extensions": ["py"],
      "command": ["pylsp"],
      "restartInterval": 5
    }
  ]
}, null, 2)}</code></pre>
</section>

<section>
	<h2>Environment Variables</h2>
	<p>Override any configuration value using the <code>TYPEMILL__</code> prefix (double underscores):</p>
	<pre><code># Server configuration
export TYPEMILL__SERVER__PORT=3000
export TYPEMILL__SERVER__HOST="127.0.0.1"

# Authentication (use env vars for secrets!)
export TYPEMILL__SERVER__AUTH__JWT_SECRET="your-secret-key"

# Cache settings
export TYPEMILL__CACHE__ENABLED=true
export TYPEMILL__CACHE__TTL_SECONDS=3600

# Or use a .env file (gitignored)
echo 'TYPEMILL__SERVER__AUTH__JWT_SECRET=dev-secret' > .env</code></pre>

	<h3>Security Best Practices</h3>
	<ul>
		<li>✅ Never commit secrets to config files - use environment variables</li>
		<li>✅ Keep server on <code>127.0.0.1</code> for local development (not <code>0.0.0.0</code>)</li>
		<li>✅ Enable TLS when binding to non-loopback addresses for production</li>
		<li>✅ Use secret management services (Vault, AWS Secrets Manager) in production</li>
	</ul>
</section>

<section>
	<h2>Running TypeMill</h2>

	<h3>Stdio Mode (for MCP clients)</h3>
	<pre><code>mill start</code></pre>
	<p>Use this for integration with AI assistants like Claude Code.</p>

	<h3>WebSocket Server Mode</h3>
	<pre><code>mill serve</code></pre>
	<p>Runs on <code>ws://127.0.0.1:3040</code> by default. Suitable for production deployments.</p>

	<h3>Check Status</h3>
	<pre><code>mill status</code></pre>
</section>

<section>
	<h2>Connecting to AI Assistants</h2>
	<h3>Claude Desktop Configuration</h3>
	<p>Add to your Claude Desktop configuration file:</p>
	<pre><code>{JSON.stringify({
  "mcpServers": {
    "mill": {
      "command": "mill",
      "args": ["start"]
    }
  }
}, null, 2)}</code></pre>
</section>

<section>
	<h2>CLI Usage</h2>
	<h3>File Operations</h3>
	<pre><code># Rename a file (no position needed)
mill tool rename --target file:src/old.rs --new-name src/new.rs

# Rename a directory
mill tool rename --target directory:old-dir --new-name new-dir</code></pre>

	<h3>Code Operations</h3>
	<pre><code># Move code (requires line:char position)
mill tool move --source src/app.rs:10:5 --destination src/utils.rs

# Extract function (requires position)
mill tool extract --kind function --source src/app.rs:10:5 --name handleLogin</code></pre>

	<h3>Analysis</h3>
	<pre><code># Analyze complexity
mill tool analyze.quality --kind complexity --scope workspace

# Find unused imports
mill tool analyze.dead_code --kind unused_imports --scope file:src/app.rs</code></pre>

	<h3>Workspace Operations</h3>
	<pre><code># Find and replace across workspace
mill tool workspace.find_replace --pattern "oldName" --replacement "newName"</code></pre>
</section>

<section>
	<h2>Troubleshooting</h2>

	<h3>Server won't start</h3>
	<pre><code># Check LSP server availability
mill status

# Verify language servers are installed
which typescript-language-server
which rust-analyzer
which pylsp

# Review config file
cat .typemill/config.json</code></pre>

	<h3>Tools not working</h3>
	<ul>
		<li>Ensure file extensions match config (<code>.rs</code> → <code>rust-analyzer</code>)</li>
		<li>Check MCP connection with AI assistant</li>
		<li>Review server logs for errors</li>
	</ul>

	<h3>Performance issues</h3>
	<ul>
		<li>Enable cache: <code>unset TYPEMILL_DISABLE_CACHE</code></li>
		<li>Adjust <code>restartInterval</code> in config (recommended: 10-30 minutes)</li>
		<li>Check system resources (LSP servers can be memory-intensive)</li>
	</ul>
</section>

<section>
	<h2>Docker Deployment</h2>
	<h3>Quick Start</h3>
	<pre><code># Build and run
docker build -t typemill .
docker run -p 3040:3040 typemill

# Or use docker-compose
docker-compose up</code></pre>

	<h3>Environment Configuration</h3>
	<pre><code>docker run -e TYPEMILL__SERVER__PORT=3000 \\
           -e TYPEMILL__SERVER__AUTH__JWT_SECRET=secret \\
           -p 3000:3000 typemill</code></pre>
</section>

<section>
	<h2>Language Support</h2>
	<table>
		<thead>
			<tr>
				<th>Language</th>
				<th>LSP Server</th>
				<th>Installation</th>
			</tr>
		</thead>
		<tbody>
			<tr>
				<td>TypeScript/JavaScript</td>
				<td>typescript-language-server</td>
				<td><code>npm install -g typescript-language-server typescript</code></td>
			</tr>
			<tr>
				<td>Rust</td>
				<td>rust-analyzer</td>
				<td><code>rustup component add rust-analyzer</code></td>
			</tr>
			<tr>
				<td>Python</td>
				<td>pylsp</td>
				<td><code>pip install python-lsp-server</code></td>
			</tr>
		</tbody>
	</table>
</section>

<section>
	<h2>Next Steps</h2>
	<ul>
		<li><a href="/tools">Explore all MCP tools</a></li>
		<li><a href="/architecture">Understand the architecture</a></li>
		<li><a href="/contributing">Contribute to TypeMill</a></li>
	</ul>
</section>
