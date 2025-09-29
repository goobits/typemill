//! Virtual filesystem implementation using FUSE
//!
//! This crate provides FUSE (Filesystem in Userspace) functionality
//! for mounting virtual filesystems that integrate with the codebuddy
//! code intelligence system.

pub mod driver;

pub use driver::{start_fuse_mount, CodeflowFS};
