//! Virtual filesystem implementation using FUSE
//!
//! This crate provides FUSE (Filesystem in Userspace) functionality
//! for mounting virtual filesystems that integrate with the codebuddy
//! code intelligence system.

// Prevent technical debt accumulation
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]

pub mod driver;

pub use driver::{start_fuse_mount, CodeflowFS};
