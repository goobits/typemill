// Note: These extern crate declarations ensure language plugins are linked
// for detailed import analysis. Future work: abstract via LanguagePlugin trait.
extern crate cb_lang_markdown;
extern crate cb_lang_rust;
extern crate cb_lang_typescript;
extern crate cb_lang_toml;
extern crate cb_lang_yaml;

pub mod services;

// Re-export commonly used types at crate root for convenience
pub use services::{
    ChecksumValidator, DryRunGenerator, DryRunResult, PlanConverter, PostApplyValidator,
    ValidationConfig, ValidationResult,
};
