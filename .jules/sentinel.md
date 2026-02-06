## 2025-05-23 - Path Traversal in Background Workers
**Vulnerability:** The `OperationQueue` worker in `mill-server` executed file operations (create, write, delete, rename) using raw paths from the operation object without validating they were within the project root.
**Learning:** Background workers that process serialized operations are a common bypass for security checks enforced at the API layer. The API layer might validate the request, but if the worker is "dumb" and blindly executes the queued operation, an internal attacker or a buggy component can exploit it.
**Prevention:** Validation must happen at the *execution point* (in the worker), not just at the ingestion point. We introduced `validate_path` in the worker loop to enforce project root containment using `canonicalize` (handling non-existent files correctly).

## 2025-06-02 - Unauthenticated Admin Endpoint
**Vulnerability:** The `/auth/generate-token` endpoint on the admin server (bound to localhost) allowed unauthenticated generation of valid JWT tokens for any user or project.
**Learning:** Admin interfaces bound to localhost are often treated as "trusted," but they are accessible to any local process. Exposing a token minting endpoint without authentication defeats the purpose of the token system, as any local code execution (or SSRF) becomes a full privilege escalation.
**Prevention:** Sensitive administrative actions like token generation should not be exposed via unauthenticated HTTP endpoints, even on localhost. They were moved to a CLI command (`mill generate-token`), which requires shell access and serves as a natural authentication barrier.
