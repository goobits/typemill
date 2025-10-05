#!/usr/bin/env bash
set -euo pipefail

# new-lang.sh - Scaffold a new language plugin for Codebuddy
#
# Usage: ./new-lang.sh <language-name>
# Example: ./new-lang.sh java

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LANGUAGES_DIR="$SCRIPT_DIR"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

usage() {
    echo "Usage: $0 <language-name>"
    echo ""
    echo "Examples:"
    echo "  $0 java"
    echo "  $0 kotlin"
    echo "  $0 ruby"
    exit 1
}

if [ $# -ne 1 ]; then
    usage
fi

LANG_NAME="$1"
LANG_LOWER=$(echo "$LANG_NAME" | tr '[:upper:]' '[:lower:]')
LANG_UPPER=$(echo "$LANG_NAME" | tr '[:lower:]' '[:upper:]')
LANG_TITLE=$(echo "$LANG_LOWER" | sed 's/.*/\u&/')

PLUGIN_NAME="cb-lang-${LANG_LOWER}"
PLUGIN_DIR="${LANGUAGES_DIR}/${PLUGIN_NAME}"

echo -e "${BLUE}Creating ${LANG_TITLE} language plugin...${NC}"

# Check if plugin already exists
if [ -d "$PLUGIN_DIR" ]; then
    echo -e "${RED}Error: Plugin directory already exists: ${PLUGIN_DIR}${NC}"
    exit 1
fi

# Create directory structure
echo -e "${GREEN}✓${NC} Creating directory structure..."
mkdir -p "$PLUGIN_DIR/src"
mkdir -p "$PLUGIN_DIR/resources"

# Create Cargo.toml
echo -e "${GREEN}✓${NC} Generating Cargo.toml..."
cat > "$PLUGIN_DIR/Cargo.toml" << EOF
[package]
name = "${PLUGIN_NAME}"
version.workspace = true
edition.workspace = true
license.workspace = true
repository.workspace = true
homepage.workspace = true

[dependencies]
# Codebuddy workspace dependencies
cb-plugin-api = { path = "../../cb-plugin-api" }
cb-protocol = { path = "../../cb-protocol" }
cb-core = { path = "../../cb-core" }

# Async operations
async-trait = { workspace = true }
tokio = { workspace = true }

# Serialization/Deserialization
serde = { workspace = true }
serde_json = { workspace = true }

# Error handling
thiserror = { workspace = true }

# Logging
tracing = { workspace = true }

# Utilities (uncomment as needed)
# regex = "1.10"
# tempfile = "3.10"
# chrono = { version = "0.4", features = ["serde"] }
EOF

# Create lib.rs with skeleton implementation
echo -e "${GREEN}✓${NC} Generating src/lib.rs..."
cat > "$PLUGIN_DIR/src/lib.rs" << EOF
//! ${LANG_TITLE} language plugin for Codebuddy
//!
//! Provides AST parsing, symbol extraction, and manifest analysis for ${LANG_TITLE}.

mod parser;
mod manifest;

use cb_plugin_api::{
    LanguageIntelligencePlugin, ManifestData, ParsedSource, PluginError, PluginResult,
};
use async_trait::async_trait;
use std::path::Path;

/// ${LANG_TITLE} language plugin
pub struct ${LANG_TITLE}Plugin;

impl ${LANG_TITLE}Plugin {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ${LANG_TITLE}Plugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LanguageIntelligencePlugin for ${LANG_TITLE}Plugin {
    fn name(&self) -> &'static str {
        "${LANG_TITLE}"
    }

    fn file_extensions(&self) -> Vec<&'static str> {
        // TODO: Add ${LANG_TITLE} file extensions
        // Example: vec!["java"] for Java, vec!["kt", "kts"] for Kotlin
        vec!["${LANG_LOWER}"]
    }

    async fn parse(&self, source: &str) -> PluginResult<ParsedSource> {
        parser::parse_source(source)
    }

    async fn analyze_manifest(&self, path: &Path) -> PluginResult<ManifestData> {
        manifest::analyze_manifest(path).await
    }

    fn manifest_filename(&self) -> &'static str {
        // TODO: Specify manifest filename
        // Examples: "pom.xml", "build.gradle", "Gemfile", "pyproject.toml"
        "manifest.${LANG_LOWER}"
    }

    fn source_dir(&self) -> &'static str {
        // TODO: Specify source directory (empty string for project root)
        // Examples: "src" for Java/Kotlin, "" for Python/Ruby
        "src"
    }

    fn entry_point(&self) -> &'static str {
        // TODO: Specify entry point filename
        // Examples: "Main.java", "main.kt", "__init__.py"
        "main.${LANG_LOWER}"
    }

    fn module_separator(&self) -> &'static str {
        // TODO: Specify module path separator
        // Examples: "." for Java/Python, "::" for Rust, "/" for Go
        "."
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let plugin = ${LANG_TITLE}Plugin::new();
        assert_eq!(plugin.name(), "${LANG_TITLE}");
    }

    #[test]
    fn test_file_extensions() {
        let plugin = ${LANG_TITLE}Plugin::new();
        let extensions = plugin.file_extensions();
        assert!(!extensions.is_empty());
    }
}
EOF

# Create parser.rs
echo -e "${GREEN}✓${NC} Generating src/parser.rs..."
cat > "$PLUGIN_DIR/src/parser.rs" << EOF
//! ${LANG_TITLE} source code parsing and symbol extraction

use cb_plugin_api::{ParsedSource, PluginError, PluginResult, Symbol, SymbolKind};
use cb_protocol::SourceLocation;

/// Parse ${LANG_TITLE} source code and extract symbols
pub fn parse_source(source: &str) -> PluginResult<ParsedSource> {
    // TODO: Implement ${LANG_TITLE} parsing
    //
    // Two approaches:
    // 1. Dual-mode (recommended): Native AST parser subprocess + regex fallback
    //    See cb-lang-go or cb-lang-typescript for examples
    //
    // 2. Pure Rust: Use a Rust parser crate if available
    //    See cb-lang-rust for example using syn crate
    //
    // For now, return empty symbols
    tracing::warn!("${LANG_TITLE} parsing not yet implemented");

    Ok(ParsedSource {
        data: serde_json::json!({}),
        symbols: vec![],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_source() {
        let result = parse_source("");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_simple_source() {
        let source = r#"
            // TODO: Add ${LANG_TITLE} source example
        "#;
        let result = parse_source(source);
        assert!(result.is_ok());
    }
}
EOF

# Create manifest.rs
echo -e "${GREEN}✓${NC} Generating src/manifest.rs..."
cat > "$PLUGIN_DIR/src/manifest.rs" << EOF
//! ${LANG_TITLE} manifest file parsing
//!
//! Handles manifest files for ${LANG_TITLE} projects.
//! TODO: Specify manifest format (e.g., pom.xml, build.gradle, Gemfile)

use cb_plugin_api::{Dependency, DependencySource, ManifestData, PluginError, PluginResult};
use std::path::Path;

/// Analyze ${LANG_TITLE} manifest file
pub async fn analyze_manifest(path: &Path) -> PluginResult<ManifestData> {
    // TODO: Implement manifest parsing
    //
    // Examples:
    // - Java: Parse pom.xml (Maven) or build.gradle (Gradle)
    // - Kotlin: Parse build.gradle.kts
    // - Ruby: Parse Gemfile
    // - Python: Parse pyproject.toml or requirements.txt
    //
    // For now, return minimal data
    tracing::warn!(
        manifest_path = %path.display(),
        "${LANG_TITLE} manifest parsing not yet implemented"
    );

    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| PluginError::io(format!("Failed to read manifest: {}", e)))?;

    Ok(ManifestData {
        name: "unknown".to_string(),
        version: "0.0.0".to_string(),
        dependencies: vec![],
        dev_dependencies: vec![],
        raw_data: serde_json::json!({ "content": content }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[tokio::test]
    async fn test_analyze_empty_manifest() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "# Empty manifest").unwrap();

        let result = analyze_manifest(temp_file.path()).await;
        assert!(result.is_ok());
    }
}
EOF

# Create README.md
echo -e "${GREEN}✓${NC} Generating README.md..."
cat > "$PLUGIN_DIR/README.md" << EOF
# ${LANG_TITLE} Language Plugin

${LANG_TITLE} language support for Codebuddy via the \`LanguageIntelligencePlugin\` trait.

## Features

- [ ] AST parsing and symbol extraction
- [ ] Import/dependency analysis
- [ ] Manifest file parsing (TODO: specify format)
- [ ] Refactoring support (rename, extract, etc.)

## Implementation Status

🚧 **Under Development**

### Completed
- ✅ Plugin scaffolding
- ✅ Basic trait implementation

### TODO
- [ ] Implement parser (see \`src/parser.rs\`)
- [ ] Implement manifest analyzer (see \`src/manifest.rs\`)
- [ ] Add comprehensive tests
- [ ] Document ${LANG_TITLE}-specific behavior
- [ ] Register in \`registry_builder.rs\`

## Parser Strategy

TODO: Choose and implement one of:

### Option 1: Dual-Mode (Recommended)
- **AST Mode**: Native ${LANG_TITLE} parser via subprocess
- **Fallback Mode**: Regex-based parsing when native parser unavailable
- **Examples**: See \`cb-lang-go\` or \`cb-lang-typescript\`

### Option 2: Pure Rust Parser
- Use a Rust ${LANG_TITLE} parser crate (if available)
- **Example**: See \`cb-lang-rust\` using \`syn\` crate

## Manifest Format

TODO: Document manifest file format and location

Examples:
- Java: \`pom.xml\` (Maven) or \`build.gradle\` (Gradle)
- Kotlin: \`build.gradle.kts\`
- Python: \`pyproject.toml\`, \`requirements.txt\`, or \`setup.py\`
- Ruby: \`Gemfile\`

## Testing

\`\`\`bash
# Run plugin tests
cargo test -p ${PLUGIN_NAME}

# Run with output
cargo test -p ${PLUGIN_NAME} -- --nocapture
\`\`\`

## Registration

After implementation, register in \`crates/cb-services/src/services/registry_builder.rs\`:

\`\`\`rust
// Register ${LANG_TITLE} plugin
#[cfg(feature = "lang-${LANG_LOWER}")]
{
    registry.register(Arc::new(${PLUGIN_NAME}::${LANG_TITLE}Plugin::new()));
    plugin_count += 1;
}
\`\`\`

## References

- [Language Plugin Guide](../README.md)
- [API Documentation](../../cb-plugin-api/src/lib.rs)
- Reference implementations: \`cb-lang-rust\`, \`cb-lang-go\`, \`cb-lang-typescript\`
EOF

echo ""
echo -e "${GREEN}✓${NC} Successfully created ${LANG_TITLE} language plugin at:"
echo -e "  ${BLUE}${PLUGIN_DIR}${NC}"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo ""
echo -e "1. ${BLUE}Add to workspace dependencies${NC} in ${WORKSPACE_ROOT}/Cargo.toml:"
echo -e "   ${GREEN}[features]${NC}"
echo -e "   ${GREEN}lang-${LANG_LOWER} = [\"${PLUGIN_NAME}\"]${NC}"
echo -e ""
echo -e "   ${GREEN}[workspace.dependencies]${NC}"
echo -e "   ${GREEN}${PLUGIN_NAME} = { path = \"crates/languages/${PLUGIN_NAME}\" }${NC}"
echo ""
echo -e "2. ${BLUE}Add to cb-handlers${NC} in ${WORKSPACE_ROOT}/crates/cb-handlers/Cargo.toml:"
echo -e "   ${GREEN}[dependencies]${NC}"
echo -e "   ${GREEN}${PLUGIN_NAME} = { workspace = true, optional = true }${NC}"
echo -e ""
echo -e "   ${GREEN}[features]${NC}"
echo -e "   ${GREEN}lang-${LANG_LOWER} = [\"dep:${PLUGIN_NAME}\"]${NC}"
echo ""
echo -e "3. ${BLUE}Register plugin${NC} in ${WORKSPACE_ROOT}/crates/cb-services/src/services/registry_builder.rs:"
echo -e "   Add around line 88:"
echo ""
echo -e "   ${GREEN}// Register ${LANG_TITLE} plugin${NC}"
echo -e "   ${GREEN}#[cfg(feature = \"lang-${LANG_LOWER}\")]${NC}"
echo -e "   ${GREEN}{${NC}"
echo -e "   ${GREEN}    registry.register(Arc::new(${PLUGIN_NAME}::${LANG_TITLE}Plugin::new()));${NC}"
echo -e "   ${GREEN}    plugin_count += 1;${NC}"
echo -e "   ${GREEN}}${NC}"
echo ""
echo -e "4. ${BLUE}Implement parsing logic${NC} in:"
echo -e "   - ${PLUGIN_DIR}/src/parser.rs"
echo -e "   - ${PLUGIN_DIR}/src/manifest.rs"
echo ""
echo -e "5. ${BLUE}Add tests${NC} and run:"
echo -e "   ${GREEN}cargo test -p ${PLUGIN_NAME}${NC}"
echo ""
echo -e "6. ${BLUE}Verify configuration${NC}:"
echo -e "   ${GREEN}./crates/languages/check-features.sh${NC}"
echo ""
echo -e "${BLUE}For examples, see:${NC}"
echo -e "  - Pure Rust parser: ${GREEN}crates/languages/cb-lang-rust${NC}"
echo -e "  - Dual-mode parser: ${GREEN}crates/languages/cb-lang-go${NC}"
echo ""
