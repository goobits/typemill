# Docker Deployment Guide

## Quick Start

### Production Mode (Secure - Recommended)
```bash
# Start with default secure configuration (no FUSE, no elevated privileges)
docker-compose up codebuddy

# View logs
docker-compose logs -f codebuddy

# Stop services
docker-compose down
```

### Development Mode with FUSE (⚠️ EXPERIMENTAL)
```bash
# Start with FUSE support (requires SYS_ADMIN capability)
docker-compose --profile fuse up codebuddy-fuse

# View logs
docker-compose logs -f codebuddy-fuse
```

**⚠️ SECURITY WARNING:** FUSE mode requires `SYS_ADMIN` capability and disabled AppArmor security, effectively removing container security boundaries. **Only use in trusted development environments. Never use in production.**

### Production Mode
```bash
# Set JWT secret
export JWT_SECRET="your-secure-secret-key"

# Start production stack
docker-compose -f docker-compose.production.yml up -d

# Check health
curl http://localhost/health
```

## Architecture

- **codebuddy**: Main WebSocket server with FUSE support (ports 3000, 4000)
- **nginx**: Reverse proxy with SSL/TLS termination (production only)

### Optional Development Containers
The default `docker-compose.yml` includes example workspace containers:
- **frontend-workspace**: Node.js development container
- **backend-workspace**: Python development container

These can be removed if not needed, or customized for your stack.

## FUSE Support (⚠️ EXPERIMENTAL)

**Default:** FUSE is **disabled** for security. The default `codebuddy` service runs without elevated privileges.

**To enable FUSE (development only):**
```bash
docker-compose --profile fuse up codebuddy-fuse
```

FUSE filesystems are mounted at `/tmp/codeflow-mounts` inside containers with shared mount propagation.

**Security Requirements:**
- `/dev/fuse` device on host
- `SYS_ADMIN` capability (elevated privileges)
- `apparmor:unconfined` security option (disables container isolation)

**⚠️ This configuration is NOT secure for production environments.**

**To disable FUSE in configuration:**
Edit `.codebuddy/config.json`:
```json
{
  "fuse": null
}
```

## Configuration

### Development: `./config/development.json`
```json
{
  "server": {"host": "0.0.0.0", "port": 3000},
  "logging": {"level": "debug", "format": "pretty"}
}
```

### Production: `./config/production.json`
```json
{
  "server": {"host": "0.0.0.0", "port": 3000, "maxClients": 100},
  "logging": {"level": "info", "format": "json"}
}
```

### Environment Variable Overrides
```bash
CODEBUDDY__SERVER__PORT=3001
CODEBUDDY__LOGGING__LEVEL=debug
CODEBUDDY__SERVER__MAX_CLIENTS=50
```

## WebSocket Connection

### Development
```javascript
const ws = new WebSocket('ws://localhost:3000');
```

### Production (via nginx)
```javascript
const ws = new WebSocket('ws://your-domain.com/ws');
// or wss://your-domain.com/ws for SSL
```

## Health Checks

```bash
# Development - direct to server
curl http://localhost:4000/health

# Production - via nginx
curl http://localhost/health
```

## Logs

```bash
# Development
docker-compose logs -f codebuddy

# Production
docker-compose -f docker-compose.production.yml logs -f codebuddy
```

## Building

```bash
# Rebuild after code changes
docker-compose build --no-cache

# Build specific service
docker-compose build codebuddy
```

## Troubleshooting

### FUSE not working
- Ensure `/dev/fuse` exists: `ls -la /dev/fuse`
- Check kernel module: `lsmod | grep fuse`
- Verify Docker has required capabilities

### LSP servers not found
LSP servers are pre-installed:
- `typescript-language-server` + `typescript`
- `python-lsp-server`

Check installation inside container:
```bash
docker-compose exec codebuddy which typescript-language-server
```

### Permission errors
Ensure proper ownership:
```bash
docker-compose exec codebuddy ls -la /workspace
```

### Connection refused
Check if service is healthy:
```bash
docker-compose ps
docker-compose logs codebuddy
```

## SSL/TLS Setup (Production)

1. Obtain certificates (Let's Encrypt, etc.)
2. Place in `./certs/` directory:
   - `fullchain.pem`
   - `privkey.pem`
3. Uncomment HTTPS server block in `nginx.conf`
4. Restart nginx:
   ```bash
   docker-compose -f docker-compose.production.yml restart nginx
   ```

## Resource Limits

Add to `docker-compose.yml` if needed:
```yaml
services:
  codebuddy:
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 2G
        reservations:
          memory: 512M
```

## Security Hardening

### Production Checklist
- ✅ **Use default `codebuddy` service (no FUSE, no elevated privileges)**
- ✅ Disable FUSE in config: `"fuse": null`
- ✅ Set strong `JWT_SECRET`
- ✅ Enable HTTPS/WSS
- ✅ Restrict `/metrics` endpoint by IP
- ✅ Use read-only config mounts (`:ro`)
- ✅ Enable Docker Content Trust
- ✅ Scan images for vulnerabilities
- ✅ Run as non-root user (already configured)

### ⚠️ NEVER in Production
- ❌ Do not use `codebuddy-fuse` service in production
- ❌ Do not enable `SYS_ADMIN` capability
- ❌ Do not disable AppArmor security (`apparmor:unconfined`)
- ❌ Do not enable FUSE configuration

## Kubernetes Deployment

For Kubernetes, convert compose files:
```bash
kompose convert -f docker-compose.production.yml
```

Or see example manifests in `/kubernetes/` (if available).