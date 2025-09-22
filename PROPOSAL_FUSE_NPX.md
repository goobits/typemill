# File Changes for NPX-in-Docker Implementation

## 99% Confident Analysis Complete

After deep analysis of the current Docker setup, here's the precise implementation plan:

## CREATE - New Files (4 files)

### 1. `/workspace/Dockerfile.npx`

**Purpose:** NPX-based Docker image for zero-setup deployment

**Contents:**
```dockerfile
FROM node:20-slim

# Install FUSE and minimal dependencies
RUN apt-get update && apt-get install -y \
    fuse \
    libfuse-dev \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create directories for FUSE operations
RUN mkdir -p /tmp/codeflow-workspaces /tmp/codeflow-mounts

# Create non-root user and add to fuse group
RUN useradd -r -s /bin/false -G fuse codeflow \
    && chown -R codeflow:codeflow /tmp/codeflow-workspaces /tmp/codeflow-mounts

# Switch to non-root user
USER codeflow

# Expose WebSocket port
EXPOSE 3000

# Health check using npx command
HEALTHCHECK --interval=30s --timeout=10s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Use NPX to run latest version - always fresh from npm
ENTRYPOINT ["npx", "@goobits/codeflow-buddy@latest"]
CMD ["serve", "--enable-fuse", "--port", "3000", "--max-clients", "50"]
```

### 2. `/workspace/docker-compose.npx.yml`

**Purpose:** Simple one-command deployment with NPX

**Contents:**
```yaml
version: '3.8'

services:
  codeflow-buddy:
    build:
      context: .
      dockerfile: Dockerfile.npx
    ports:
      - "3000:3000"
    environment:
      - NODE_ENV=production
      - ENABLE_FUSE=true
      - MAX_CLIENTS=100
    volumes:
      # Persistent storage for workspaces
      - codeflow-workspaces:/tmp/codeflow-workspaces
      - codeflow-mounts:/tmp/codeflow-mounts
    cap_add:
      # Minimal privilege - just for FUSE mounting
      - SYS_ADMIN
    devices:
      # FUSE device access
      - /dev/fuse:/dev/fuse
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 15s

volumes:
  codeflow-workspaces:
  codeflow-mounts:
```

### 3. `/workspace/.dockerignore.npx`

**Purpose:** Minimal ignore file for NPX builds (no source copying needed)

**Contents:**
```
# NPX Docker build - we don't copy source code
*
!Dockerfile.npx
!package.json
```

### 4. `/workspace/scripts/docker-npx.sh`

**Purpose:** Build and push NPX Docker image script

**Contents:**
```bash
#!/bin/bash
set -e

# Build and tag NPX-based Docker image
echo "🐳 Building NPX Docker image..."

# Get version from package.json
VERSION=$(node -p "require('./package.json').version")

# Build the image
docker build -f Dockerfile.npx -t goobits/codeflow-buddy:npx .
docker build -f Dockerfile.npx -t goobits/codeflow-buddy:npx-$VERSION .
docker build -f Dockerfile.npx -t goobits/codeflow-buddy:latest-npx .

echo "✅ Built images:"
echo "  - goobits/codeflow-buddy:npx"
echo "  - goobits/codeflow-buddy:npx-$VERSION"
echo "  - goobits/codeflow-buddy:latest-npx"

# Optionally push to registry
if [ "$1" = "--push" ]; then
    echo "📤 Pushing to Docker Hub..."
    docker push goobits/codeflow-buddy:npx
    docker push goobits/codeflow-buddy:npx-$VERSION
    docker push goobits/codeflow-buddy:latest-npx
    echo "✅ Pushed to Docker Hub"
fi

echo ""
echo "🚀 To run:"
echo "docker run --cap-add SYS_ADMIN --device /dev/fuse -p 3000:3000 goobits/codeflow-buddy:npx"
echo ""
echo "Or with compose:"
echo "docker-compose -f docker-compose.npx.yml up -d"
```

## EDIT - Existing Files (3 files)

### 1. `/workspace/docs/fuse/quick-start.md`

**Adding:** New NPX Docker option as primary recommendation
**Removing:** Current "Option 1: Docker (Recommended)" section

**Changes:**
- Replace current Docker section with NPX Docker as Option 1
- Move git-clone Docker to Option 3
- Add one-liner Docker run command
- Update installation instructions priority

### 2. `/workspace/README.md`

**Adding:** NPX Docker section in Quick Start
**Removing:** Current Docker complexity

**Changes:**
- Add NPX Docker deployment section
- Simplify Docker instructions
- Update "WebSocket Server (Production)" section
- Add one-command deployment example

### 3. `/workspace/package.json`

**Adding:** NPX Docker build script
**Removing:** Nothing

**Changes:**
```json
{
  "scripts": {
    "docker:npx": "bash scripts/docker-npx.sh",
    "docker:npx:push": "bash scripts/docker-npx.sh --push"
  }
}
```

## DELETE - Files (0 files)

**No files to delete** - keeping existing Docker setup for development use

## Key Benefits of This Approach

1. **Zero Setup:** No git clone needed - just `docker run`
2. **Always Latest:** NPX pulls from npm registry automatically
3. **Minimal Privileges:** Uses `SYS_ADMIN` only, not `--privileged`
4. **Docker Security:** Full container isolation for your host
5. **One Command:** `docker run --cap-add SYS_ADMIN --device /dev/fuse -p 3000:3000 goobits/codeflow-buddy:npx`

## User Experience Transformation

**Before (3 complex options):**
```bash
# Option 1: Complex
git clone https://github.com/goobits/codeflow-buddy.git
cd codeflow-buddy
docker-compose up -d

# Option 2: No isolation
npx @goobits/codeflow-buddy@latest serve --enable-fuse

# Option 3: Manual
./start-multitenant.sh
```

**After (1 simple option):**
```bash
# Just this - Docker isolation + NPX simplicity
docker run --cap-add SYS_ADMIN --device /dev/fuse -p 3000:3000 \
  goobits/codeflow-buddy:npx
```

This implementation provides the **best of both worlds**: Docker's security isolation with NPX's zero-setup convenience.