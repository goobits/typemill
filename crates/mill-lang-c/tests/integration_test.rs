//! Integration tests for c plugin

use mill_lang_c::CPlugin;
use mill_plugin_api::LanguagePlugin;

#[tokio::test]
async fn test_plugin_metadata() {
    let plugin = CPlugin::new();
    let metadata = plugin.metadata();

    assert_eq!(metadata.name, "C");
    assert!(!metadata.extensions.is_empty());
}

use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

#[tokio::test]
async fn test_parse_integration() {
    // This is a placeholder. More comprehensive integration tests
    // will be added in a later step.
}

#[tokio::test]
async fn test_makefile_integration() {
    let plugin = CPlugin::new();
    let dir = tempdir().unwrap();
    let makefile_path = dir.path().join("Makefile");
    let mut file = File::create(&makefile_path).unwrap();
    writeln!(
        file,
        r#"
        CC = gcc
        CFLAGS = -Wall -Wextra

        SRCS = main.c
        TARGET = my_app

        all: $(TARGET)

        $(TARGET): $(SRCS)
	        $(CC) $(CFLAGS) -o $(TARGET) $(SRCS)
        "#
    )
    .unwrap();

    File::create(dir.path().join("main.c")).unwrap();

    let result = plugin.analyze_manifest(&makefile_path).await;
    assert!(result.is_ok());
    let manifest_data = result.unwrap();
    assert_eq!(manifest_data.name, "my_app");
    assert_eq!(manifest_data.dependencies.len(), 1);
    assert_eq!(manifest_data.dependencies[0].name, "main.c");
}

#[tokio::test]
async fn test_cmake_integration() {
    let plugin = CPlugin::new();
    let dir = tempdir().unwrap();
    let cmake_path = dir.path().join("CMakeLists.txt");
    let mut file = File::create(&cmake_path).unwrap();
    writeln!(
        file,
        r#"
        cmake_minimum_required(VERSION 3.10)
        project(my_cmake_app)
        add_executable(my_cmake_app main.c)
        "#
    )
    .unwrap();

    File::create(dir.path().join("main.c")).unwrap();

    let result = plugin.analyze_manifest(&cmake_path).await;
    assert!(result.is_ok());
    let manifest_data = result.unwrap();
    assert_eq!(manifest_data.name, "my_cmake_app");
    assert_eq!(manifest_data.dependencies.len(), 1);
    assert_eq!(manifest_data.dependencies[0].name, "main.c");
}