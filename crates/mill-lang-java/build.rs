use std::process::Command;
use std::path::Path;

fn main() {
    // Declare the custom cfg flag for conditional compilation
    println!("cargo::rustc-check-cfg=cfg(java_parser_jar_exists)");

    // Check if the JAR file exists
    let jar_path = Path::new("resources/java-parser/target/java-parser-1.0.0.jar");
    if jar_path.exists() {
        println!("cargo:rustc-cfg=java_parser_jar_exists");
        println!("cargo:rerun-if-changed=resources/java-parser/target/java-parser-1.0.0.jar");
    } else {
        println!("cargo:warning=Java parser JAR not found. The plugin will work with limited functionality (no AST parsing).");

        // Check if Maven and Java are available to build the JAR
        let has_mvn = is_command_in_path("mvn");
        let has_java = is_command_in_path("java");

        if !has_mvn {
            println!("cargo:warning=Maven is required to build the Java parser. Please install it and ensure it's in your PATH. You can run 'make check-parser-deps' for more details.");
        }
        if !has_java {
            println!("cargo:warning=A Java runtime is required to build the Java parser. Please install it and ensure it's in your PATH. You can run 'make check-parser-deps' for more details.");
        }

        if has_mvn && has_java {
            println!("cargo:warning=To build the Java parser JAR, run: cd resources/java-parser && mvn package");
        }
    }
}

fn is_command_in_path(command: &str) -> bool {
    Command::new(command)
        .arg("--version")
        .output()
        .is_ok()
}