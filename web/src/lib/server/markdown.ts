import { readFile, access, stat } from 'fs/promises';
import { join, isAbsolute, dirname, resolve } from 'path';
import { fileURLToPath } from 'url';
import { marked } from 'marked';
import hljs from 'highlight.js';
import DOMPurify from 'isomorphic-dompurify';

// Resolve project root by trying multiple strategies
const __dirname = dirname(fileURLToPath(import.meta.url));

async function findProjectRoot(): Promise<string> {
	// Strategy 1: WORKSPACE_ROOT env var
	if (process.env.WORKSPACE_ROOT) {
		return process.env.WORKSPACE_ROOT;
	}

	// Strategy 2: Navigate up from this file's location
	// This file is at: web/src/lib/server/markdown.ts
	// Project root is 4 levels up
	const fromFile = resolve(__dirname, '..', '..', '..', '..');

	// Strategy 3: From cwd, check if we're in web/ or project root
	const cwd = process.cwd();
	const fromCwdParent = resolve(cwd, '..');

	// Test each potential root by checking for docs/ directory
	const candidates = [fromFile, cwd, fromCwdParent];

	for (const candidate of candidates) {
		try {
			await access(join(candidate, 'docs'));
			return candidate;
		} catch {
			// Continue to next candidate
		}
	}

	// Fallback to the file-based calculation
	return fromFile;
}

// Cache the project root
let projectRootPromise: Promise<string> | null = null;

function getProjectRoot(): Promise<string> {
	if (!projectRootPromise) {
		projectRootPromise = findProjectRoot();
	}
	return projectRootPromise;
}

// Configure marked once
const renderer = {
	code({ text, lang }: { text: string; lang?: string }) {
		const validLang = !!(lang && hljs.getLanguage(lang));
		const highlighted = validLang
			? hljs.highlight(text, { language: lang }).value
			: hljs.highlight(text, { language: 'plaintext' }).value;

		return `<pre><code class="hljs${validLang ? ` language-${lang}` : ''}">${highlighted}</code></pre>`;
	}
};

marked.use({
	gfm: true,
	breaks: false,
	renderer
});

// Cache for markdown content and rendered HTML
interface CacheEntry {
	mtimeMs: number;
	content: string;
	metadata: Record<string, any>;
	html?: string;
}

const postCache = new Map<string, CacheEntry>();

// Parse frontmatter from markdown content
function parseFrontmatter(content: string): { metadata: Record<string, any>; content: string } {
	const frontmatterRegex = /^---\s*\n([\s\S]*?)\n---\s*\n([\s\S]*)$/;
	const match = content.match(frontmatterRegex);

	if (!match) {
		return { metadata: {}, content };
	}

	const [, frontmatterStr, mainContent] = match;
	const metadata: Record<string, any> = {};

	// Parse YAML-like frontmatter
	frontmatterStr.split('\n').forEach(line => {
		const colonIndex = line.indexOf(':');
		if (colonIndex > 0) {
			const key = line.slice(0, colonIndex).trim();
			const value = line.slice(colonIndex + 1).trim();
			metadata[key] = value.replace(/^["']|["']$/g, ''); // Remove quotes
		}
	});

	return { metadata, content: mainContent };
}

async function resolveMarkdownPath(relativePath: string): Promise<string> {
	// Security: Prevent directory traversal and arbitrary file read
	// Check for parent directory references and absolute paths (including Windows drive paths)
	if (relativePath.includes('..') || isAbsolute(relativePath) || relativePath.startsWith('/')) {
		throw new Error('Invalid path');
	}

	// Security: Only allow reading markdown files
	if (!relativePath.toLowerCase().endsWith('.md')) {
		throw new Error('Invalid path');
	}

	const root = await getProjectRoot();
	return join(root, relativePath);
}

export async function readMarkdownFile(relativePath: string): Promise<{ content: string; metadata: Record<string, any> }> {
	const filePath = await resolveMarkdownPath(relativePath);

	try {
		const stats = await stat(filePath);
		const cached = postCache.get(filePath);

		if (cached && cached.mtimeMs === stats.mtimeMs) {
			return { content: cached.content, metadata: cached.metadata };
		}

		const fileContent = await readFile(filePath, 'utf-8');
		const { metadata, content } = parseFrontmatter(fileContent);

		postCache.set(filePath, {
			mtimeMs: stats.mtimeMs,
			content,
			metadata,
			html: undefined // Invalidate HTML cache
		});

		return { content, metadata };
	} catch (e: any) {
		if (e.code === 'ENOENT') {
			throw new Error('File not found');
		}
		throw e;
	}
}

export async function renderMarkdown(markdown: string): Promise<string> {
	const result = marked.parse(markdown);
	const html = result instanceof Promise ? await result : result;
	return DOMPurify.sanitize(html);
}

export async function getRenderedPost(relativePath: string): Promise<{ html: string; metadata: Record<string, any>; filePath: string }> {
	const filePath = await resolveMarkdownPath(relativePath);

	try {
		const stats = await stat(filePath);
		const cached = postCache.get(filePath);

		if (cached && cached.mtimeMs === stats.mtimeMs && cached.html) {
			return { html: cached.html, metadata: cached.metadata, filePath };
		}

		// Check if content is cached but HTML is missing
		let content: string;
		let metadata: Record<string, any>;

		if (cached && cached.mtimeMs === stats.mtimeMs) {
			content = cached.content;
			metadata = cached.metadata;
		} else {
			// Read fresh
			const fileContent = await readFile(filePath, 'utf-8');
			const parsed = parseFrontmatter(fileContent);
			content = parsed.content;
			metadata = parsed.metadata;
		}

		const html = await renderMarkdown(content);

		postCache.set(filePath, {
			mtimeMs: stats.mtimeMs,
			content,
			metadata,
			html
		});

		return { html, metadata, filePath };
	} catch (e: any) {
		if (e.code === 'ENOENT') {
			throw new Error('File not found');
		}
		throw e;
	}
}

// Export for use in entries generator
export { getProjectRoot };
