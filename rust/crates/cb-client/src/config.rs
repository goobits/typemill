//! Client configuration

use cb_core::AppConfig;
use crate::error::{ClientError, ClientResult};

/// Load client configuration
pub fn load_client_config() -> ClientResult<AppConfig> {
    AppConfig::load().map_err(|err| ClientError::config(format!("Failed to load config: {}", err)))
}