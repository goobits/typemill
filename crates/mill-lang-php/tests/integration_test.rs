//! Integration tests for php plugin

use mill_lang_php::PhpPlugin;
use mill_plugin_api::LanguagePlugin;

#[tokio::test]
async fn test_php_plugin_loads() {
    let plugin = PhpPlugin::new();
    let metadata = plugin.metadata();
    assert_eq!(metadata.name, "PHP");
}

#[tokio::test]
async fn test_parse_sample_code() {
    let plugin = PhpPlugin::new();
    // A simple php script
    let source = "<?php\n\nfunction hello() {\n    echo \"Hello, world!\";\n}\n";
    let result = plugin.parse(source).await;
    assert!(result.is_ok());
}