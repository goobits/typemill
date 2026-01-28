# TypeMill Extra Language Plugins

Optional language support plugins for TypeMill. These are not included in the default build.

## Languages

| Language | Plugin | LSP Server | Status |
|----------|--------|------------|--------|
| C | mill-lang-c | clangd | Basic |
| C++ | mill-lang-cpp | clangd | Basic |
| C# | mill-lang-csharp | omnisharp | Requires .NET SDK |
| Go | mill-lang-go | gopls | Full |
| Java | mill-lang-java | jdtls | Full |
| Swift | mill-lang-swift | sourcekit-lsp | Full |

## Building with Extra Languages

```bash
# Build with specific extra language
cargo build -F lang-java

# Build with all extra languages
cargo build -F lang-c,lang-cpp,lang-csharp,lang-go,lang-java,lang-swift
```

## Future: Standalone Repository

This directory is structured to allow easy extraction into a standalone repository:

```bash
git subtree split --prefix=languages-extra -b typemill-langs-extra
```
