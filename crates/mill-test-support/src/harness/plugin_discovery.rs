//! Plugin discovery helpers for integration testing
//!
//! This module provides utilities for discovering language plugins
//! and their test fixtures at runtime. It enables truly dynamic
//! plugin testing where adding a new language plugin automatically
//! includes it in the test suite.

use mill_plugin_api::{LanguagePlugin, LanguageTestFixtures, PluginDiscovery};
use mill_services::services::registry_builder::build_language_plugin_registry;
use std::sync::{Arc, OnceLock};

// Create a single, static instance of the plugin registry.
// This is initialized once and lives for the entire test run.
static REGISTRY: OnceLock<Arc<PluginDiscovery>> = OnceLock::new();

fn get_or_init_registry() -> &'static Arc<PluginDiscovery> {
    REGISTRY.get_or_init(|| {
        // Get all available plugins from the bundle
        // Convert Arc<dyn LanguagePlugin> to Box<dyn LanguagePlugin> via cloning
        // This is needed because all_plugins returns Arcs but registry_builder expects Boxes (ownership transfer)
        // In a real app, we'd move the boxes, but here we're coming from a static list of Arcs.
        // Note: LanguagePlugin implements Clone via Arc internally usually, but here we need to
        // deep clone or just accept that we are creating new instances if possible,
        // OR we need to update registry_builder to take Arcs.
        // Since registry_builder takes Box, let's instantiate fresh ones if we can, or
        // cheat by using the fact that we can't easily clone trait objects.
        // Actually, mill_plugin_bundle::all_plugins() returns Vec<Arc<dyn LanguagePlugin>>.
        // Registry builder expects Vec<Box<dyn LanguagePlugin>>.
        // We cannot convert Arc to Box easily if it's shared.
        //
        // Let's use the fact that we are in test support.
        // We will re-instantiate them.
        // But mill_plugin_bundle::all_plugins() is our only entry point.
        //
        // Wait, mill_plugin_bundle::all_plugins() returns instances.
        // If we can't convert, we might need to change registry_builder to accept Arcs,
        // which is actually better for shared ownership anyway.
        //
        // Let's update registry_builder to accept Vec<Arc<dyn LanguagePlugin>> instead.
        // But wait, `register` takes `Arc`.
        // So if we pass `Box`, we convert to `Arc`. If we pass `Arc`, we just use it.
        // This suggests changing registry_builder signature is the right path.
        //
        // HOWEVER, for this specific patch, I will try to resolve the type mismatch
        // by modifying this call site if possible, or falling back to changing the signature.
        //
        // Since `mill_plugin_bundle` gives Arcs, and `registry_builder` creates Arcs,
        // passing Arcs is logical.

        // TEMPORARY FIX: Change registry_builder to take Arcs?
        // No, I can't change registry_builder signature easily without breaking other calls I just fixed.
        //
        // Let's see if we can get Boxes from plugin bundle. No, it returns Arcs.
        //
        // Okay, I will update `registry_builder` to accept `Vec<Arc<dyn LanguagePlugin>>`.
        // This is a better API design anyway.
        let plugins = mill_plugin_bundle::all_plugins();

        // Note: I will apply this change to registry_builder.rs first.
        // But here I am editing plugin_discovery.rs.
        // I will comment this out and rely on the next step to fix the signature match.
        // For now, to make THIS file compile, I need to assume the signature change happens.
        // But I can't assume.
        //
        // Let's try to clone the Arcs into new Boxes? No, unsized.
        //
        // Okay, I will assume `registry_builder` will be updated to `Vec<Arc<...>>`.
        build_language_plugin_registry(plugins)
    })
}

/// Discover all installed language plugins that provide test fixtures.
///
/// This function queries the plugin registry and returns all plugins
/// that have implemented the `test_fixtures()` method.
///
/// # Returns
///
/// A vector of tuples containing:
/// - An `Arc` clone of the plugin.
/// - The test fixtures it provides.
pub fn discover_plugins_with_fixtures() -> Vec<(Arc<dyn LanguagePlugin>, LanguageTestFixtures)> {
    let registry = get_or_init_registry();
    registry
        .all()
        .iter()
        .filter_map(|plugin| {
            plugin
                .test_fixtures()
                .map(|fixtures| (plugin.clone(), fixtures))
        })
        .collect()
}

/// Get the display name of a language plugin.
///
/// Useful for logging and error messages.
pub fn plugin_language_name(plugin: &dyn LanguagePlugin) -> &str {
    plugin.metadata().name
}

/// Get the file extension for a language plugin.
///
/// Returns the first registered extension (e.g., "py", "ts", "rs").
pub fn plugin_file_extension(plugin: &dyn LanguagePlugin) -> &str {
    // It's safe to unwrap because the contract tests ensure extensions are not empty.
    plugin.metadata().extensions.first().unwrap()
}

/// Returns a reference to the global plugin registry for tests.
pub fn get_test_registry() -> &'static Arc<PluginDiscovery> {
    get_or_init_registry()
}
