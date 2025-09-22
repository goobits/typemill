# Release Notes - v1.1.0

## 🎉 ARM64 Native FUSE Support

This release brings full native FUSE support for ARM64 architecture, enabling codeflow-buddy to run on modern ARM-based systems including Apple Silicon Macs and ARM64 Linux servers.

### Key Features

#### 🏗️ Native FUSE Implementation
- Replaced `fuse-native` with `@cocalc/fuse-native` for ARM64 compatibility
- Removed all mock implementations - now using 100% native FUSE
- Full callback-style API implementation for better compatibility

#### 🐳 Multi-Tenant Docker Support
- Production-ready Docker Compose configuration
- Multi-tenant FUSE folder mounting capabilities
- Session-based workspace isolation
- Automatic cleanup on client disconnect

#### 🛠️ Stability Improvements
- Fixed duplicate method definitions in WebSocket server
- Resolved TypeScript type errors in FUSE operations
- Improved test isolation for FUSE integration tests
- Better error handling in session cleanup

### Breaking Changes
None - this release maintains full backward compatibility.

### Installation

```bash
# Install globally
npm install -g @goobits/codeflow-buddy@1.1.0

# Or use with npx
npx @goobits/codeflow-buddy@1.1.0 setup
```

### Docker Deployment

```bash
# Quick start multi-tenant service
docker-compose up --build

# Or use the production configuration
docker-compose -f docker-compose.production.yml up -d
```

### Requirements
- Node.js 18+
- FUSE support in kernel (for FUSE features)
- Docker with privileged mode (for containerized FUSE)

### Platform Support
- ✅ x86_64 Linux
- ✅ ARM64 Linux
- ✅ macOS (Intel)
- ✅ macOS (Apple Silicon)
- ✅ Windows (via WSL2)

### Contributors
Special thanks to everyone who helped test and validate ARM64 support!

---

For more information, see the [CHANGELOG](CHANGELOG.md) and [README](README.md).