# Jules SvelteKit Frontend Proposal

**Status**: Proposal
**Date**: 2025-10-09
**Priority**: 70 (Medium - Enhancement)

## Executive Summary

Replace the `jules-cli` command-line tool with a modern **SvelteKit web frontend** powered by a Bun server. This will provide a better user experience for interacting with the Google Jules API through a visual interface while maintaining the same functionality.

## Current State

```
jules/
├── crates/
│   ├── jules-api/          # Rust client library (KEEP)
│   ├── jules-cli/          # CLI tool (REMOVE)
│   └── jules-mcp-server/   # MCP server (KEEP)
```

**Current CLI routes:**
- `sources list` - List code repositories
- `sources get <id>` - Get repository details
- `sessions create` - Create new Jules session
- `sessions list` - List all sessions
- `sessions get <id>` - Get session details
- `sessions delete <id>` - Delete session
- `activities list <session-id>` - List session activities
- `activities send` - Send message to Jules

## Proposed Architecture

```
jules/
├── crates/
│   ├── jules-api/          # Rust client library (unchanged)
│   └── jules-mcp-server/   # MCP server (unchanged)
└── web/                    # NEW: SvelteKit frontend
    ├── src/
    │   ├── routes/
    │   │   ├── +page.svelte              # Dashboard
    │   │   ├── sources/
    │   │   │   ├── +page.svelte          # Sources list
    │   │   │   └── [id]/+page.svelte     # Source details
    │   │   ├── sessions/
    │   │   │   ├── +page.svelte          # Sessions list
    │   │   │   ├── new/+page.svelte      # Create session
    │   │   │   └── [id]/+page.svelte     # Session chat
    │   │   └── api/
    │   │       ├── sources/+server.ts
    │   │       ├── sessions/+server.ts
    │   │       └── activities/+server.ts
    │   ├── lib/
    │   │   ├── api/
    │   │   │   └── jules-client.ts       # Jules API wrapper
    │   │   └── components/
    │   │       ├── SourceCard.svelte
    │   │       ├── SessionList.svelte
    │   │       ├── ChatInterface.svelte
    │   │       └── PlanApproval.svelte
    │   └── app.html
    ├── package.json
    ├── svelte.config.js
    ├── tsconfig.json
    └── vite.config.ts
```

## Technology Stack

**Runtime:** Bun (fast, TypeScript-native)
**Framework:** SvelteKit (SSR + API routes)
**Styling:** TailwindCSS + shadcn-svelte
**API Client:** Fetch (native Bun support)
**State:** Svelte stores
**WebSockets:** For real-time activity updates (optional enhancement)

## Route Mapping

### API Routes (SvelteKit server endpoints)

| Endpoint | Method | Maps to Google Jules API |
|----------|--------|--------------------------|
| `/api/sources` | GET | `GET /sources` |
| `/api/sources?filter=X` | GET | `GET /sources?filter=X` |
| `/api/sources/[id]` | GET | `GET /sources/{id}` |
| `/api/sessions` | GET | `GET /sessions` |
| `/api/sessions` | POST | `POST /sessions` |
| `/api/sessions/[id]` | GET | `GET /sessions/{id}` |
| `/api/sessions/[id]` | DELETE | `DELETE /sessions/{id}` |
| `/api/sessions/[id]/activities` | GET | `GET /sessions/{id}/activities` |
| `/api/sessions/[id]/activities` | POST | `POST /sessions/{id}/activities` |
| `/api/sessions/[id]/plans/[planId]/approve` | POST | `POST /sessions/{id}/plans/{planId}:approve` |

### UI Routes (SvelteKit pages)

| Route | Description | Key Features |
|-------|-------------|--------------|
| `/` | Dashboard | Overview, recent sessions, quick actions |
| `/sources` | Sources list | All connected repos, filter, search |
| `/sources/[id]` | Source details | Repo info, sessions for this source |
| `/sessions` | Sessions list | All sessions, filter by state |
| `/sessions/new` | Create session | Select source, set prompt |
| `/sessions/[id]` | Session chat | Chat interface, activity timeline, plan approvals |

## UI/UX Design

### Dashboard (`/`)
```
┌─────────────────────────────────────┐
│ Jules Control Panel                 │
├─────────────────────────────────────┤
│ Quick Actions                       │
│ [+ New Session]  [View Sources]     │
│                                     │
│ Recent Sessions                     │
│ ┌─────────────────────────────────┐│
│ │ Fix login bug - In Progress     ││
│ │ Add dark mode - Completed       ││
│ └─────────────────────────────────┘│
│                                     │
│ Connected Sources                   │
│ [my-app] [backend-api] [docs]      │
└─────────────────────────────────────┘
```

### Session Chat (`/sessions/[id]`)
```
┌─────────────────────────────────────┐
│ Session: Fix login bug              │
│ Source: my-app                      │
├─────────────────────────────────────┤
│ Activity Timeline                   │
│                                     │
│ You: Fix the auth bug in login.js  │
│                                     │
│ Jules: Analyzing code...            │
│ Jules: Found issue in login.js:42   │
│ Jules: Created fix plan             │
│                                     │
│ ┌─────────────────────────────────┐│
│ │ Plan: Fix authentication        ││
│ │ - login.js: Change token check  ││
│ │ [View Diff] [Approve] [Reject]  ││
│ └─────────────────────────────────┘│
│                                     │
│ [Type message...]            [Send] │
└─────────────────────────────────────┘
```

## Implementation Plan

### Phase 1: Project Setup (2-3 hours)
- [x] Create proposal document
- [ ] Initialize SvelteKit project in `jules/web/`
- [ ] Configure Bun as runtime
- [ ] Set up TailwindCSS + shadcn-svelte
- [ ] Create basic layout and navigation

### Phase 2: API Client (3-4 hours)
- [ ] Create TypeScript client for Google Jules API
- [ ] Implement authentication (X-Goog-Api-Key)
- [ ] Add error handling and retry logic
- [ ] Create SvelteKit API routes (server endpoints)
- [ ] Add environment variable support (JULES_API_KEY)

### Phase 3: Core UI (6-8 hours)
- [ ] Dashboard page with overview
- [ ] Sources list and detail pages
- [ ] Sessions list page
- [ ] Session creation form
- [ ] Session chat interface
- [ ] Activity timeline component

### Phase 4: Advanced Features (4-5 hours)
- [ ] Plan approval UI with diff viewer
- [ ] Filtering and search
- [ ] Pagination support
- [ ] Error states and loading indicators
- [ ] Real-time updates (WebSocket or polling)

### Phase 5: Polish & Deployment (2-3 hours)
- [ ] Responsive design (mobile-friendly)
- [ ] Dark mode support
- [ ] Performance optimization
- [ ] Build and deployment configuration
- [ ] Documentation

### Phase 6: Cleanup (1 hour)
- [ ] Remove `jules-cli` crate
- [ ] Update main README
- [ ] Update workspace Cargo.toml

**Total Estimated Time:** 18-24 hours

## Migration Strategy

### 1. Parallel Development
Keep `jules-cli` during SvelteKit development. Users can test web UI while CLI remains available.

### 2. Feature Parity Checklist
- [ ] All CLI commands have web equivalents
- [ ] Filtering works (sources list)
- [ ] Pagination works (all lists)
- [ ] Error messages are clear
- [ ] Authentication works
- [ ] Session creation matches CLI behavior

### 3. Removal Criteria
Only remove `jules-cli` when:
- ✅ Web UI has 100% feature parity
- ✅ Documentation is complete
- ✅ At least 2 users have tested it
- ✅ Deployment guide is written

## Configuration

### Environment Variables
```bash
# .env.local (development)
JULES_API_KEY=your-api-key-here
PUBLIC_APP_NAME=Jules Control Panel

# Optional
JULES_API_BASE_URL=https://jules.googleapis.com/v1alpha
```

### Bun Server
```bash
# Development
bun run dev

# Production
bun run build
bun run preview  # or deploy to Vercel/Cloudflare
```

## Deployment Options

1. **Self-hosted** (Bun server)
   ```bash
   cd jules/web
   bun install
   bun run build
   bun run preview --host 0.0.0.0 --port 3000
   ```

2. **Vercel** (recommended for SvelteKit)
   - Zero config deployment
   - Automatic HTTPS
   - Edge functions for API routes

3. **Cloudflare Pages**
   - Similar to Vercel
   - Good for global distribution

4. **Docker** (future consideration)
   ```dockerfile
   FROM oven/bun:1
   WORKDIR /app
   COPY . .
   RUN bun install
   RUN bun run build
   CMD ["bun", "run", "preview", "--host", "0.0.0.0"]
   ```

## Advantages Over CLI

### User Experience
- ✅ Visual timeline of session activities
- ✅ Syntax-highlighted code diffs
- ✅ One-click plan approvals
- ✅ No need to remember command syntax
- ✅ Works on mobile devices

### Features
- ✅ Real-time updates (see Jules typing)
- ✅ Multiple sessions in tabs
- ✅ Search across all sessions
- ✅ Visual indicators (in progress, completed, failed)

### Accessibility
- ✅ No CLI knowledge required
- ✅ Shareable URLs for sessions
- ✅ Works in any browser
- ✅ Better for non-technical stakeholders

## Disadvantages / Trade-offs

### CLI Advantages We Lose
- ❌ Scriptability (can't pipe output)
- ❌ SSH-friendly (need GUI)
- ❌ Lower resource usage

### Mitigation
- Keep `jules-api` Rust crate - can build new CLI later
- Keep `jules-mcp-server` - provides programmatic access
- Could add REST API documentation for curl users

## API Client Example

```typescript
// src/lib/api/jules-client.ts
export class JulesClient {
  private apiKey: string;
  private baseUrl: string;

  constructor(apiKey: string, baseUrl = 'https://jules.googleapis.com/v1alpha') {
    this.apiKey = apiKey;
    this.baseUrl = baseUrl;
  }

  private async fetch(path: string, options: RequestInit = {}) {
    const response = await fetch(`${this.baseUrl}${path}`, {
      ...options,
      headers: {
        'X-Goog-Api-Key': this.apiKey,
        'Content-Type': 'application/json',
        ...options.headers,
      },
    });

    if (!response.ok) {
      throw new Error(`Jules API error: ${response.statusText}`);
    }

    return response.json();
  }

  async listSources(filter?: string, pageSize?: number, pageToken?: string) {
    const params = new URLSearchParams();
    if (filter) params.set('filter', filter);
    if (pageSize) params.set('pageSize', String(pageSize));
    if (pageToken) params.set('pageToken', pageToken);

    return this.fetch(`/sources?${params}`);
  }

  async createSession(sourceId: string, prompt?: string) {
    return this.fetch('/sessions', {
      method: 'POST',
      body: JSON.stringify({ source_id: sourceId, prompt }),
    });
  }

  async sendMessage(sessionId: string, content: string) {
    return this.fetch(`/sessions/${sessionId}/activities`, {
      method: 'POST',
      body: JSON.stringify({ content }),
    });
  }

  // ... other methods
}
```

## Success Criteria

- [ ] All CLI functionality available in web UI
- [ ] Web UI is faster to use than CLI for common tasks
- [ ] Documentation is complete
- [ ] Can deploy to Vercel/Cloudflare in < 5 minutes
- [ ] Works on mobile Safari and Chrome
- [ ] No regressions in `jules-api` or `jules-mcp-server`

## Future Enhancements

1. **WebSocket Support** - Real-time activity streaming
2. **Code Editor** - Edit proposed changes before approval
3. **Diff Viewer** - Side-by-side code comparison
4. **Session Templates** - Common prompts (fix bugs, add tests, etc.)
5. **Multi-source Sessions** - Work across multiple repos
6. **Collaboration** - Share session URLs with team
7. **Analytics** - Track Jules's success rate

## Questions / Decisions Needed

1. **Should we keep `jules-cli` as optional?**
   - Pro: Power users still have CLI
   - Con: Maintenance burden
   - **Recommendation:** Remove CLI, keep `jules-api` for future CLI v2

2. **Authentication: Browser storage?**
   - Option A: API key in environment (server-side only)
   - Option B: API key in browser localStorage (client-side)
   - **Recommendation:** Server-side only (more secure)

3. **Real-time updates: WebSocket or polling?**
   - WebSocket: Better UX, more complex
   - Polling: Simpler, works everywhere
   - **Recommendation:** Start with polling, add WebSocket later

4. **Component library?**
   - shadcn-svelte: Modern, customizable
   - Skeleton UI: Svelte-native
   - Material UI: Enterprise feel
   - **Recommendation:** shadcn-svelte (best for quick prototyping)

## Open Questions

- [ ] Does Google Jules API support WebSocket connections?
- [ ] What's the rate limit for polling activities?
- [ ] Should we persist session state locally (IndexedDB)?
- [ ] Do we need user authentication (beyond API key)?

## References

- Google Jules API: https://developers.google.com/jules/api
- SvelteKit Docs: https://kit.svelte.dev/docs
- Bun Runtime: https://bun.sh/docs
- shadcn-svelte: https://www.shadcn-svelte.com/

---

**Next Steps:**
1. Get approval for this proposal
2. Create `jules/web/` directory structure
3. Initialize SvelteKit + Bun project
4. Implement Phase 1 (project setup)
