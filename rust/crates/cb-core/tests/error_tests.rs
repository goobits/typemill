//! Tests for error handling

use cb_core::{CoreError};
use cb_core::error::CoreResult;
use std::io;

#[test]
fn test_core_error_creation() {
    let config_error = CoreError::config("Invalid configuration");
    match config_error {
        CoreError::Config { message } => {
            assert_eq!(message, "Invalid configuration");
        }
        _ => panic!("Expected config error"),
    }

    let invalid_data = CoreError::invalid_data("Bad format");
    match invalid_data {
        CoreError::InvalidData { message } => {
            assert_eq!(message, "Bad format");
        }
        _ => panic!("Expected invalid data error"),
    }

    let not_supported = CoreError::not_supported("operation_x");
    match not_supported {
        CoreError::NotSupported { operation } => {
            assert_eq!(operation, "operation_x");
        }
        _ => panic!("Expected not supported error"),
    }
}

#[test]
fn test_core_error_from_io() {
    let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
    let core_error: CoreError = io_error.into();

    match core_error {
        CoreError::Io(_) => {} // Expected
        _ => panic!("Expected IO error conversion"),
    }
}

#[test]
fn test_core_error_from_json() {
    let json_error = serde_json::from_str::<serde_json::Value>("invalid json");
    assert!(json_error.is_err());

    let core_error: CoreError = json_error.unwrap_err().into();
    match core_error {
        CoreError::Json(_) => {} // Expected
        _ => panic!("Expected JSON error conversion"),
    }
}

#[test]
fn test_core_error_display() {
    let config_error = CoreError::config("Test message");
    let display_string = format!("{}", config_error);
    assert!(display_string.contains("Configuration error"));
    assert!(display_string.contains("Test message"));

    let not_found = CoreError::not_found("file.txt");
    let display_string = format!("{}", not_found);
    assert!(display_string.contains("Resource not found"));
    assert!(display_string.contains("file.txt"));

    let timeout = CoreError::timeout("network_request");
    let display_string = format!("{}", timeout);
    assert!(display_string.contains("Timeout occurred during"));
    assert!(display_string.contains("network_request"));
}

#[test]
fn test_core_error_debug() {
    let internal_error = CoreError::internal("Debug message");
    let debug_string = format!("{:?}", internal_error);
    assert!(debug_string.contains("Internal"));
    assert!(debug_string.contains("Debug message"));
}

#[test]
fn test_core_result_type_alias() {
    fn test_function() -> CoreResult<String> {
        Ok("success".to_string())
    }

    fn test_error_function() -> CoreResult<String> {
        Err(CoreError::internal("test error"))
    }

    let success_result = test_function();
    assert!(success_result.is_ok());
    assert_eq!(success_result.unwrap(), "success");

    let error_result = test_error_function();
    assert!(error_result.is_err());

    let error = error_result.unwrap_err();
    match error {
        CoreError::Internal { message } => {
            assert_eq!(message, "test error");
        }
        _ => panic!("Expected internal error"),
    }
}

#[test]
fn test_error_chain() {
    // Test that errors can be chained properly
    fn inner_function() -> Result<(), io::Error> {
        Err(io::Error::new(io::ErrorKind::PermissionDenied, "Access denied"))
    }

    fn outer_function() -> CoreResult<()> {
        inner_function().map_err(CoreError::from)?;
        Ok(())
    }

    let result = outer_function();
    assert!(result.is_err());

    let error = result.unwrap_err();
    match error {
        CoreError::Io(io_error) => {
            assert_eq!(io_error.kind(), io::ErrorKind::PermissionDenied);
        }
        _ => panic!("Expected IO error"),
    }
}

#[test]
fn test_error_helpers() {
    let permission_denied = CoreError::permission_denied("read_file");
    match permission_denied {
        CoreError::PermissionDenied { operation } => {
            assert_eq!(operation, "read_file");
        }
        _ => panic!("Expected permission denied error"),
    }

    let not_found = CoreError::not_found("database.db");
    match not_found {
        CoreError::NotFound { resource } => {
            assert_eq!(resource, "database.db");
        }
        _ => panic!("Expected not found error"),
    }

    let timeout = CoreError::timeout("api_call");
    match timeout {
        CoreError::Timeout { operation } => {
            assert_eq!(operation, "api_call");
        }
        _ => panic!("Expected timeout error"),
    }
}

#[test]
fn test_error_implements_std_error() {
    let error = CoreError::internal("test");

    // Should implement std::error::Error
    let _: &dyn std::error::Error = &error;

    // Should have source method (even if it returns None for most variants)
    use std::error::Error;
    let source = error.source();
    assert!(source.is_none()); // Internal error doesn't have a source

    // IO error should have source
    let io_error = io::Error::new(io::ErrorKind::NotFound, "Not found");
    let core_error: CoreError = io_error.into();

    match core_error {
        CoreError::Io(ref _inner) => {
            let core_error_ref: &dyn std::error::Error = &core_error;
            let source = core_error_ref.source();
            assert!(source.is_some());
        }
        _ => panic!("Expected IO error"),
    }
}

#[test]
fn test_error_serialization() {
    // While CoreError doesn't implement Serialize (because std::error::Error can't be serialized),
    // we can test that error messages can be serialized for transport
    let error = CoreError::config("Invalid port number");
    let error_message = format!("{}", error);

    // Should be able to serialize the error message
    let json = serde_json::to_string(&error_message).unwrap();
    assert!(json.contains("Configuration error"));
    assert!(json.contains("Invalid port number"));

    // Should be able to deserialize back
    let deserialized: String = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, error_message);
}