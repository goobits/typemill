use std::path::Path;

fn main() {
    let dir = Path::new("tree-sitter-go/src");

    let mut build = cc::Build::new();

    if let Ok(include_path) = std::env::var("DEP_TREE_SITTER_INCLUDE") {
        build.include(include_path);
    }

    build
        .include(dir)
        .file(dir.join("parser.c"))
        .compile("tree_sitter_go");

    println!("cargo:rerun-if-changed=tree-sitter-go/src/parser.c");
}