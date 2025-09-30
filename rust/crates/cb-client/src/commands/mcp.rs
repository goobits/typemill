//! MCP preset management commands

use anyhow::{bail, Result};
use cb_core::config::{AppConfig, ExternalMcpConfig, ExternalMcpServerConfig};
use std::path::Path;

#[cfg(feature = "mcp-proxy")]
use cb_mcp_proxy::presets;

/// List available MCP presets
pub fn list_presets() -> Result<()> {
    #[cfg(not(feature = "mcp-proxy"))]
    {
        bail!("MCP proxy feature not enabled. Rebuild with --features mcp-proxy");
    }

    #[cfg(feature = "mcp-proxy")]
    {
        let presets = presets::get_presets();

        println!("Available MCP Presets:\n");
        for (id, preset) in presets {
            println!("  {} - {}", id, preset.description);
        }
        println!("\nUsage: codebuddy mcp add <preset>");

        Ok(())
    }
}

/// Add an MCP preset to config
pub fn add_preset(preset_id: &str) -> Result<()> {
    #[cfg(not(feature = "mcp-proxy"))]
    {
        bail!("MCP proxy feature not enabled. Rebuild with --features mcp-proxy");
    }

    #[cfg(feature = "mcp-proxy")]
    {
        // Get preset
        let preset = presets::get_preset(preset_id).ok_or_else(|| {
            anyhow::anyhow!(
                "Preset '{}' not found. Run 'codebuddy mcp list' to see available presets.",
                preset_id
            )
        })?;

        // Load config
        let config_path = Path::new(".codebuddy/config.json");
        let mut config = if config_path.exists() {
            AppConfig::load()?
        } else {
            AppConfig::default()
        };

        // Initialize external_mcp if needed
        if config.external_mcp.is_none() {
            config.external_mcp = Some(ExternalMcpConfig { servers: vec![] });
        }

        let external_mcp = config.external_mcp.as_mut().unwrap();

        // Check if already exists
        if external_mcp.servers.iter().any(|s| s.name == preset.id) {
            println!("✓ {} is already configured", preset.name);
            return Ok(());
        }

        // Add preset
        external_mcp.servers.push(ExternalMcpServerConfig {
            name: preset.id.clone(),
            command: preset.command.clone(),
            env: if preset.env.is_empty() {
                None
            } else {
                Some(preset.env.clone())
            },
            auto_start: preset.auto_start,
        });

        // Save config
        config.save(config_path)?;

        println!("✓ Added {} to .codebuddy/config.json", preset.name);

        Ok(())
    }
}
