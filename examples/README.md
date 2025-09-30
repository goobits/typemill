# Docker Development Environment

## Quick Start

### Development Mode
```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f codebuddy

# Stop services
docker-compose down
```

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

- **codebuddy**: Main WebSocket server with FUSE support
- **frontend-workspace**: Example Node.js development container
- **backend-workspace**: Example Python development container
- **nginx**: Reverse proxy (production only)

## FUSE Mounts

FUSE filesystems are mounted at `/tmp/codeflow-mounts` inside containers and shared across services using Docker's `:shared` mount propagation.

## Configuration

- Development: `./config/development.json`
- Production: `./config/production.json`

Override via environment variables:
```bash
CODEBUDDY__SERVER__PORT=3001
CODEBUDDY__LOGGING__LEVEL=debug
```

## Troubleshooting

### FUSE not working
- Ensure `/dev/fuse` exists on host
- Check Docker has `SYS_ADMIN` capability
- Verify `apparmor:unconfined` security opt

### LSP servers not found
- LSP servers are pre-installed in the container
- Check `which typescript-language-server` inside container

### Permission errors
- Ensure the `codebuddy` user is in the `fuse` group
- Check volume mount permissions

## WebSocket Connection

Connect to the server from your applications:
```javascript
const ws = new WebSocket('ws://localhost:3000');
```

Or via nginx proxy (production):
```javascript
const ws = new WebSocket('ws://your-domain.com/ws');
```

## Health Checks

Development:
```bash
curl http://localhost:4000/health
```

Production (via nginx):
```bash
curl http://localhost/health
```

## Logs

View server logs:
```bash
# Development
docker-compose logs -f codebuddy

# Production
docker-compose -f docker-compose.production.yml logs -f codebuddy
```

## Building from Source

Rebuild the Docker image after code changes:
```bash
docker-compose build --no-cache
```
