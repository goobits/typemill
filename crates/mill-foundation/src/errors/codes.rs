//! Error code constants

pub mod error_codes {
    // Existing codes (from old error.rs)
    /// Internal server error (500)
    pub const E1000_INTERNAL_SERVER_ERROR: &str = "E1000";
    /// Invalid request parameters (400)
    pub const E1001_INVALID_REQUEST: &str = "E1001";
    /// File not found (404)
    pub const E1002_FILE_NOT_FOUND: &str = "E1002";
    /// LSP server error
    pub const E1003_LSP_ERROR: &str = "E1003";
    /// Operation timeout
    pub const E1004_TIMEOUT: &str = "E1004";
    /// Permission denied (403)
    pub const E1005_PERMISSION_DENIED: &str = "E1005";
    /// Resource not found (404)
    pub const E1006_RESOURCE_NOT_FOUND: &str = "E1006";
    /// Operation not supported
    pub const E1007_NOT_SUPPORTED: &str = "E1007";
    /// Invalid data format
    pub const E1008_INVALID_DATA: &str = "E1008";

    // New codes for expanded error variants
    /// Parse error
    pub const E1009_PARSE_ERROR: &str = "E1009";
    /// Validation error
    pub const E1010_VALIDATION_ERROR: &str = "E1010";
    /// Serialization error
    pub const E1011_SERIALIZATION_ERROR: &str = "E1011";
    /// Connection error
    pub const E1012_CONNECTION_ERROR: &str = "E1012";
    /// Authentication error
    pub const E1013_AUTH_ERROR: &str = "E1013";
    /// Plugin error
    pub const E1014_PLUGIN_ERROR: &str = "E1014";
    /// Manifest error
    pub const E1015_MANIFEST_ERROR: &str = "E1015";
    /// AST error
    pub const E1016_AST_ERROR: &str = "E1016";
    /// Analysis error
    pub const E1017_ANALYSIS_ERROR: &str = "E1017";
    /// Transformation error
    pub const E1018_TRANSFORMATION_ERROR: &str = "E1018";
    /// Bootstrap error
    pub const E1019_BOOTSTRAP_ERROR: &str = "E1019";
    /// Runtime error
    pub const E1020_RUNTIME_ERROR: &str = "E1020";
    /// Unsupported syntax
    pub const E1021_UNSUPPORTED_SYNTAX: &str = "E1021";
    /// Resource already exists
    pub const E1022_ALREADY_EXISTS: &str = "E1022";
    /// Transport error
    pub const E1023_TRANSPORT_ERROR: &str = "E1023";
    /// Plugin not found
    pub const E1024_PLUGIN_NOT_FOUND: &str = "E1024";
}
