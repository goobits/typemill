//! Built-in MCP server presets

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPreset {
    pub id: String,
    pub name: String,
    pub description: String,
    pub command: Vec<String>,
    pub env: HashMap<String, String>,
    pub auto_start: bool,
}

/// Get all built-in presets
pub fn get_presets() -> HashMap<String, McpPreset> {
    let mut presets = HashMap::new();

    // context7
    presets.insert(
        "context7".to_string(),
        McpPreset {
            id: "context7".to_string(),
            name: "Context7".to_string(),
            description: "Up-to-date documentation for any library".to_string(),
            command: vec![
                "npx".to_string(),
                "-y".to_string(),
                "@upstash/context7-mcp".to_string(),
            ],
            env: HashMap::new(),
            auto_start: true,
        },
    );

    // git
    presets.insert(
        "git".to_string(),
        McpPreset {
            id: "git".to_string(),
            name: "Git MCP".to_string(),
            description: "Git operations and history".to_string(),
            command: vec![
                "npx".to_string(),
                "-y".to_string(),
                "@modelcontextprotocol/server-git".to_string(),
            ],
            env: HashMap::new(),
            auto_start: true,
        },
    );

    // filesystem
    presets.insert(
        "filesystem".to_string(),
        McpPreset {
            id: "filesystem".to_string(),
            name: "Filesystem MCP".to_string(),
            description: "Enhanced filesystem operations".to_string(),
            command: vec![
                "npx".to_string(),
                "-y".to_string(),
                "@modelcontextprotocol/server-filesystem".to_string(),
                ".".to_string(),
            ],
            env: HashMap::new(),
            auto_start: true,
        },
    );

    presets
}

/// Get a specific preset by ID
pub fn get_preset(id: &str) -> Option<McpPreset> {
    get_presets().get(id).cloned()
}

/// List all preset IDs
pub fn list_preset_ids() -> Vec<String> {
    get_presets().keys().cloned().collect()
}