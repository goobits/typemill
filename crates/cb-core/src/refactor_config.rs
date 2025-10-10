use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RefactorConfig {
    #[serde(default)]
    pub presets: HashMap<String, RefactorPreset>,
    #[serde(default)]
    pub defaults: RefactorDefaults,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RefactorPreset {
    pub strict: Option<bool>,
    pub validate_scope: Option<bool>,
    pub update_imports: Option<bool>,
    // ... other options
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefactorDefaults {
    pub dry_run: bool,
    pub rollback_on_error: bool,
    pub validate_checksums: bool,
}

impl Default for RefactorDefaults {
    fn default() -> Self {
        Self {
            dry_run: false,
            rollback_on_error: true,
            validate_checksums: true,
        }
    }
}

impl RefactorConfig {
    pub fn load(project_root: &PathBuf) -> Result<Self> {
        let config_path = project_root.join(".codebuddy/refactor.toml");
        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(config_path)?;
        let config: RefactorConfig = toml::from_str(&content)?;
        Ok(config)
    }

    // pub fn apply_preset(&self, preset_name: &str, options: &mut PlanOptions) -> Result<()> {
    //     // Implementation will be added once PlanOptions is defined
    //     Ok(())
    // }
}
