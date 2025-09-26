//! Authentication module

pub mod jwt;

pub use jwt::{validate_token, validate_token_with_project, Claims};