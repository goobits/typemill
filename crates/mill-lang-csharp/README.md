# C# Language Plugin for TypeMill

This crate provides C# language support for TypeMill, including AST parsing, symbol extraction, and manifest analysis.

## Features

- **AST Parsing**: Parses C# source code into an AST using `tree-sitter-c-sharp`.
- **Symbol Extraction**: Extracts symbols (classes, methods, etc.) from the AST.
- **Manifest Parsing**: Parses `.csproj` files to extract project metadata like package name and dependencies.
- **Import Rewriting**: Supports updating `using` statements for file moves and renames.
- **Refactoring**: Provides capabilities for `extract_function`, `extract_variable`, and `inline_variable`.
- **Project Creation**: Implements the `ProjectFactory` trait to create new C# projects using `dotnet` templates.

## LSP Configuration

The plugin is configured to use `csharp-ls` as the Language Server Protocol (LSP) server. Ensure that `csharp-ls` is installed and available in your system's `PATH`.

## `.csproj` Parsing

The plugin includes a basic parser for `.csproj` (MSBuild XML) files. It extracts the following information:
- **Package Name**: From the `<AssemblyName>` or `<PackageId>` tags. If not present, it falls back to the project file name.
- **Dependencies**: From `<PackageReference>` tags.

The parser is designed to be resilient to variations in `.csproj` file structure.