//! User input helpers for interactive CLI prompts
//!
//! This module provides functions for reading user input in interactive CLI mode.
//! All functions support both interactive and non-interactive modes:
//! - Interactive mode: Prompts the user and reads from stdin
//! - Non-interactive mode: Returns default values automatically (for CI/scripts)
//!
//! Non-interactive mode is enabled when:
//! - MILL_NON_INTERACTIVE environment variable is set
//! - CI environment variables are detected (CI, GITHUB_ACTIONS, etc.)
//! - stdin is not a TTY

use mill_foundation::core::utils::system;
use std::io::{self, BufRead, Write};

/// Check if stdin is a TTY (interactive terminal)
///
/// Returns false if:
/// - stdin is not a TTY
/// - CI environment is detected
/// - MILL_NON_INTERACTIVE environment variable is set
pub fn is_interactive() -> bool {
    // Check for explicit non-interactive flag
    if std::env::var("MILL_NON_INTERACTIVE").is_ok() {
        return false;
    }

    // Check if running in CI
    if system::is_ci() {
        return false;
    }

    // Check if stdin is a TTY using std::io::IsTerminal (Rust 1.70+)
    use std::io::IsTerminal;
    io::stdin().is_terminal()
}

/// Read a yes/no response from the user
///
/// Returns true for y/yes/empty, false for n/no
/// In non-interactive mode, returns default_value
///
/// # Arguments
/// * `prompt` - The prompt to display to the user
/// * `default_value` - The value to return in non-interactive mode
///
/// # Examples
/// ```no_run
/// use mill::cli::user_input;
/// let answer = user_input::read_yes_no("Continue?", true).unwrap();
/// ```
pub fn read_yes_no(prompt: &str, default_value: bool) -> io::Result<bool> {
    if !is_interactive() {
        return Ok(default_value);
    }

    let default_indicator = if default_value { "[Y/n]" } else { "[y/N]" };

    loop {
        print!("{} {} ", prompt, default_indicator);
        io::stdout().flush()?;

        let stdin = io::stdin();
        let mut input = String::new();
        stdin.lock().read_line(&mut input)?;

        let input = input.trim().to_lowercase();

        match input.as_str() {
            "" => return Ok(default_value),
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => {
                println!("Please enter 'y' or 'n'");
                continue;
            }
        }
    }
}

/// Read a numeric choice from the user (1-N)
///
/// Returns the chosen index (0-based)
/// In non-interactive mode, returns default_choice
///
/// # Arguments
/// * `prompt` - The prompt to display to the user
/// * `options` - Array of option strings to display
/// * `default_choice` - The default choice index (0-based) for non-interactive mode
///
/// # Examples
/// ```no_run
/// use mill::cli::user_input;
/// let options = &["Option A", "Option B", "Option C"];
/// let choice = user_input::read_choice("Select an option:", options, 0).unwrap();
/// ```
pub fn read_choice(prompt: &str, options: &[&str], default_choice: usize) -> io::Result<usize> {
    if !is_interactive() {
        return Ok(default_choice);
    }

    // Display prompt and options
    println!("{}", prompt);
    for (i, option) in options.iter().enumerate() {
        println!("  {}. {}", i + 1, option);
    }

    loop {
        print!("Enter choice (1-{}): ", options.len());
        io::stdout().flush()?;

        let stdin = io::stdin();
        let mut input = String::new();
        stdin.lock().read_line(&mut input)?;

        let input = input.trim();

        // Parse the number
        match input.parse::<usize>() {
            Ok(choice) if choice >= 1 && choice <= options.len() => {
                return Ok(choice - 1); // Convert to 0-based index
            }
            _ => {
                println!("Please enter a number between 1 and {}", options.len());
                continue;
            }
        }
    }
}

/// Read a free-form string from the user
///
/// In non-interactive mode, returns default_value
///
/// # Arguments
/// * `prompt` - The prompt to display to the user
/// * `default_value` - The value to return in non-interactive mode
///
/// # Examples
/// ```no_run
/// use mill::cli::user_input;
/// let name = user_input::read_string("Enter name:", "default").unwrap();
/// ```
pub fn read_string(prompt: &str, default_value: &str) -> io::Result<String> {
    if !is_interactive() {
        return Ok(default_value.to_string());
    }

    print!("{} ", prompt);
    io::stdout().flush()?;

    let stdin = io::stdin();
    let mut input = String::new();
    stdin.lock().read_line(&mut input)?;

    Ok(input.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_interactive_with_env_var() {
        // Set non-interactive flag
        std::env::set_var("MILL_NON_INTERACTIVE", "1");
        assert!(!is_interactive());
        std::env::remove_var("MILL_NON_INTERACTIVE");
    }

    #[test]
    fn test_read_yes_no_non_interactive() {
        std::env::set_var("MILL_NON_INTERACTIVE", "1");

        // Should return default without prompting
        let result = read_yes_no("Test?", true).unwrap();
        assert!(result);

        let result = read_yes_no("Test?", false).unwrap();
        assert!(!result);

        std::env::remove_var("MILL_NON_INTERACTIVE");
    }

    #[test]
    fn test_read_choice_non_interactive() {
        std::env::set_var("MILL_NON_INTERACTIVE", "1");

        let options = &["Option A", "Option B", "Option C"];
        let result = read_choice("Choose:", options, 1).unwrap();
        assert_eq!(result, 1);

        std::env::remove_var("MILL_NON_INTERACTIVE");
    }

    #[test]
    fn test_read_string_non_interactive() {
        std::env::set_var("MILL_NON_INTERACTIVE", "1");

        let result = read_string("Enter name:", "default_name").unwrap();
        assert_eq!(result, "default_name");

        std::env::remove_var("MILL_NON_INTERACTIVE");
    }

    // Note: Full interactive testing would require stdin mocking
    // These tests verify the non-interactive mode works correctly
}
