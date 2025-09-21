# 🧪 Testing Guide for CodeFlow-Buddy

## 🚀 Quick Start

```bash
# Clone and run tests
git clone <repo>
cd codeflow-buddy
bun install
bun test:fast
```

That's it! The test runner automatically:
- ✅ **Checks dependencies** and installs if missing
- ✅ **Builds the project** if needed or outdated
- ✅ **Validates language servers** and gives helpful install instructions
- ✅ **Runs tests** with everything ready

No manual setup needed!

## 🎯 Test Commands

```bash
bun test              # Unit tests
bun test:fast         # Quick test suite (recommended)
bun test:minimal      # For slower systems
bun test:all          # Everything
```

## 📊 How It Works

1. **Pre-test validation** - Checks and auto-fixes common issues
2. **Auto-build** - Builds project if `dist/index.js` is missing or outdated
3. **Language server check** - Validates TypeScript LSP and lists optional servers
4. **Smart defaults** - Uses `npx` for language servers from node_modules

The system ensures everything needed for tests is ready before running them.

## 🔧 Troubleshooting

### "Command not found: bun"
Install Bun: `curl -fsSL https://bun.sh/install | bash`

### Pre-test validation fails
The test runner will show you exactly what's wrong and how to fix it. Common issues:
- Missing dependencies: Run `bun install`
- TypeScript LSP missing: Already handled automatically
- Build issues: Auto-fixed by the system

### Tests fail on first run
LSP servers need a moment to warm up - run again.

### Want more language servers?
The pre-test check shows install commands for Python, Rust, Go, C++, etc.

## 💡 Why So Simple?

- **Pre-test validation** auto-fixes common issues
- **Auto-build** handles the required build step
- **Bun + NPX** handle all package management
- **Smart defaults** work out of the box
- **Clear guidance** when manual action is needed

The system is designed to eliminate friction for new contributors. Tests "just work" after `bun install`!