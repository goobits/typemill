# Mill Dogfooding Process

Testing mill refactoring tools on real-world projects to validate correctness and robustness.

## Directory Structure

```
.dogfood/
  ├── README.md              # This file
  ├── SYSTEM_DEPS.md         # System dependencies installed per project
  └── project-*/             # One directory per test project
      ├── repo/              # Cloned repository (gitignored)
      ├── docker-compose.mill-test.yml  # Services (DB, Redis, etc.)
      └── SETUP_NOTES.md     # Project-specific setup and findings
```

## Setup New Test Project

1. **Create project directory**:
   ```bash
   mkdir -p .dogfood/project-name
   cd .dogfood/project-name
   ```

2. **Clone repository**:
   ```bash
   git clone <repo-url> repo
   cd repo
   ```

3. **Create docker-compose.mill-test.yml** (if project needs services):
   ```bash
   cd ..  # Back to project directory
   # Copy template from project-a or create custom
   ```

4. **Start services** (if needed):
   ```bash
   docker-compose -f docker-compose.mill-test.yml up -d
   ```

5. **Install system dependencies** (if needed):
   ```bash
   apt-get update && apt-get install -y <packages>
   # Document in ../SYSTEM_DEPS.md
   ```

6. **Configure mill**:
   ```bash
   cd repo
   mill setup  # Auto-detects languages and installs LSP servers
   mill status # Verify configuration
   ```

## Test Workflow

### 1. Baseline

Ensure project builds and tests pass before any refactoring:

```bash
cd .dogfood/project-name/repo

# Language-specific commands
cargo test           # Rust
npm test            # TypeScript/JavaScript
pytest              # Python
go test ./...       # Go
mvn test            # Java
dotnet test         # C#
```

### 2. Run Refactorings

Test mill tools on the codebase:

```bash
# Preview changes first (dryRun: true is default)
mill tool rename --target directory:old-name --new-name new-name

# Execute if preview looks good
mill tool rename --target directory:old-name --new-name new-name --dry-run false

# Other operations
mill tool move --target file:src/old.rs --destination src/new/
mill tool extract --target function:src/lib.rs:45:10 --new-name extracted_fn
```

### 3. Validate

After refactoring, ensure project still works:

```bash
# Re-run tests
cargo test  # or npm test, pytest, etc.

# Check build
cargo build --release

# Manual verification if needed
```

### 4. Document

Record findings in `SETUP_NOTES.md`:
- What refactorings were tested
- Success/failure cases
- Edge cases discovered
- Bugs or improvements identified

## Cleanup

```bash
# Stop services
cd .dogfood/project-name
docker-compose -f docker-compose.mill-test.yml down -v

# Option 1: Keep for regression testing
# (just stop services, keep repo)

# Option 2: Full cleanup
cd ../..
rm -rf .dogfood/project-name
```

## Tips

- **Port conflicts**: Each project should use unique ports in docker-compose
- **Database isolation**: Use separate database names per project
- **System packages**: Document in SYSTEM_DEPS.md to track contamination
- **LSP servers**: Automatically isolated per project workspace
- **Git state**: Keep repo in clean state or create test branches

## Typical Project Categories

### Simple (No External Services)
- Pure library projects
- CLI tools
- Static site generators

**Setup**: Just `mill setup` and start testing

### Medium (Database/Cache)
- Web applications
- APIs with persistence
- Services with Redis/PostgreSQL

**Setup**: docker-compose.mill-test.yml with services

### Complex (Multiple Services)
- Microservices
- Full-stack applications
- Distributed systems

**Setup**: Comprehensive docker-compose or consider dedicated VM

## Notes

- We're already in Docker, so projects are isolated from host
- LSP binary cache (~/.mill/lsp/) is shared safely across projects
- System package contamination is acceptable (documented in SYSTEM_DEPS.md)
- Consider creating test branches in repos to avoid polluting main branch
