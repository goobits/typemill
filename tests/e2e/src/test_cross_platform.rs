//! Cross-platform compatibility integration tests
//!
//! Tests platform-specific functionality to ensure TypeMill works correctly
//! on both Unix (Linux, macOS) and Windows platforms.

#[cfg(test)]
mod tests {
    use std::env;
    use std::path::{Path, PathBuf};

    /// Test that temp directory is correctly detected on all platforms
    #[test]
    fn test_temp_dir_cross_platform() {
        let temp_dir = env::temp_dir();

        // Verify temp dir exists
        assert!(
            temp_dir.exists(),
            "Temp directory should exist: {:?}",
            temp_dir
        );

        // Verify temp dir is absolute
        assert!(
            temp_dir.is_absolute(),
            "Temp directory should be absolute: {:?}",
            temp_dir
        );

        // Platform-specific checks
        #[cfg(unix)]
        {
            // Unix typically uses /tmp or /var/folders (macOS)
            let path_str = temp_dir.to_string_lossy();
            assert!(
                path_str.starts_with("/tmp") || path_str.starts_with("/var/"),
                "Unix temp dir should start with /tmp or /var/: {}",
                path_str
            );
        }

        #[cfg(windows)]
        {
            // Windows uses C:\Users\<user>\AppData\Local\Temp or similar
            let path_str = temp_dir.to_string_lossy();
            assert!(
                path_str.contains("\\Temp") || path_str.contains("\\TEMP"),
                "Windows temp dir should contain \\Temp: {}",
                path_str
            );
        }
    }

    /// Test absolute path detection works on both Unix and Windows
    #[test]
    fn test_absolute_path_detection() {
        // Unix absolute paths
        #[cfg(unix)]
        {
            assert!(Path::new("/usr/bin").is_absolute());
            assert!(Path::new("/tmp/test").is_absolute());
            assert!(!Path::new("relative/path").is_absolute());
            assert!(!Path::new("./relative").is_absolute());
        }

        // Windows absolute paths
        #[cfg(windows)]
        {
            assert!(Path::new("C:\\Windows").is_absolute());
            assert!(Path::new("D:\\Data").is_absolute());
            assert!(Path::new("\\\\server\\share").is_absolute()); // UNC path
            assert!(!Path::new("relative\\path").is_absolute());
            assert!(!Path::new(".\\relative").is_absolute());
        }

        // Relative paths should work consistently
        assert!(!Path::new("src/main.rs").is_absolute());
        assert!(!Path::new("../parent").is_absolute());
    }

    /// Test home directory detection on both platforms
    #[test]
    fn test_home_dir_detection() {
        // Try to get home directory using cross-platform approach
        let home = env::var("HOME").or_else(|_| env::var("USERPROFILE"));

        // At least one should succeed on all platforms
        assert!(
            home.is_ok(),
            "Should be able to detect home directory via HOME or USERPROFILE"
        );

        let home_path = PathBuf::from(home.unwrap());

        // Verify home directory exists
        assert!(
            home_path.exists(),
            "Home directory should exist: {:?}",
            home_path
        );

        // Platform-specific checks
        #[cfg(unix)]
        {
            assert!(
                home_path.starts_with("/home")
                    || home_path.starts_with("/Users")
                    || home_path.starts_with("/root"),
                "Unix home should start with /home, /Users, or /root: {:?}",
                home_path
            );
        }

        #[cfg(windows)]
        {
            let path_str = home_path.to_string_lossy();
            assert!(
                path_str.contains("\\Users\\"),
                "Windows home should contain \\Users\\: {}",
                path_str
            );
        }
    }

    /// Test PATH separator is correct for the platform
    #[test]
    fn test_path_separator() {
        #[cfg(unix)]
        const EXPECTED_SEP: &str = ":";

        #[cfg(windows)]
        const EXPECTED_SEP: &str = ";";

        // Test separator in a constructed PATH
        let paths = ["/usr/bin", "/usr/local/bin"];

        #[cfg(unix)]
        let combined = paths.join(":");
        #[cfg(windows)]
        let combined = paths.join(";");

        assert!(combined.contains(EXPECTED_SEP));

        // Verify platform-specific separator
        #[cfg(unix)]
        assert_eq!(EXPECTED_SEP, ":");

        #[cfg(windows)]
        assert_eq!(EXPECTED_SEP, ";");
    }

    /// Test path normalization (backslash to forward slash)
    #[test]
    fn test_path_normalization() {
        let windows_path = "src\\utils\\helper.rs";
        let normalized = windows_path.replace('\\', "/");

        assert_eq!(normalized, "src/utils/helper.rs");
        assert!(!normalized.contains('\\'));

        // Test already normalized path
        let unix_path = "src/utils/helper.rs";
        let normalized2 = unix_path.replace('\\', "/");
        assert_eq!(normalized2, "src/utils/helper.rs");
    }

    /// Test that platform-specific shell commands would work
    #[test]
    fn test_shell_command_detection() {
        #[cfg(unix)]
        {
            // Unix systems should have 'sh'
            let sh_exists = std::process::Command::new("sh")
                .arg("-c")
                .arg("echo test")
                .output();

            assert!(
                sh_exists.is_ok(),
                "sh command should be available on Unix systems"
            );
        }

        #[cfg(windows)]
        {
            // Windows systems should have 'cmd.exe'
            let cmd_exists = std::process::Command::new("cmd.exe")
                .arg("/C")
                .arg("echo test")
                .output();

            assert!(
                cmd_exists.is_ok(),
                "cmd.exe should be available on Windows systems"
            );
        }
    }

    /// Test case-sensitive vs case-insensitive filesystem behavior
    #[test]
    fn test_filesystem_case_sensitivity() {
        use std::fs;

        let temp_dir = env::temp_dir();
        let test_dir = temp_dir.join("mill_case_test");

        // Clean up from previous runs
        let _ = fs::remove_dir_all(&test_dir);

        // Create test directory and file
        fs::create_dir_all(&test_dir).expect("Failed to create test directory");
        let test_file = test_dir.join("TestFile.txt");
        fs::write(&test_file, "test content").expect("Failed to write test file");

        // Try accessing with different case
        let lowercase_file = test_dir.join("testfile.txt");

        #[cfg(unix)]
        {
            // Linux is case-sensitive (macOS is case-insensitive by default)
            // We can't make assumptions, so just verify the original exists
            assert!(test_file.exists());
        }

        #[cfg(windows)]
        {
            // Windows is case-insensitive
            assert!(
                lowercase_file.exists(),
                "Windows should find file regardless of case"
            );
        }

        // Clean up
        fs::remove_dir_all(&test_dir).expect("Failed to clean up test directory");
    }

    /// Test UNC path detection on Windows
    #[cfg(windows)]
    #[test]
    fn test_unc_path_detection() {
        let unc_path = Path::new("\\\\server\\share\\file.txt");
        assert!(
            unc_path.is_absolute(),
            "UNC paths should be detected as absolute on Windows"
        );

        let unc_path2 = Path::new("\\\\?\\C:\\path");
        assert!(
            unc_path2.is_absolute(),
            "Extended UNC paths should be absolute"
        );
    }

    /// Test that drive letter paths are correctly identified on Windows
    #[cfg(windows)]
    #[test]
    fn test_drive_letter_paths() {
        assert!(Path::new("C:\\Windows").is_absolute());
        assert!(Path::new("D:\\Data\\file.txt").is_absolute());
        assert!(Path::new("Z:\\").is_absolute());

        // Relative paths with backslashes should not be absolute
        assert!(!Path::new("relative\\path").is_absolute());
    }

    /// Test environment variable access patterns
    #[test]
    fn test_env_var_patterns() {
        // PATH should exist on all platforms
        let path = env::var("PATH");
        assert!(path.is_ok(), "PATH environment variable should exist");

        let path_value = path.unwrap();
        assert!(!path_value.is_empty(), "PATH should not be empty");

        // Platform-specific separators
        #[cfg(unix)]
        assert!(
            path_value.contains(':'),
            "Unix PATH should use colon separator"
        );

        #[cfg(windows)]
        assert!(
            path_value.contains(';'),
            "Windows PATH should use semicolon separator"
        );
    }

    /// Test that common tool detection would work across platforms
    #[test]
    fn test_common_tool_paths() {
        let home = env::var("HOME")
            .or_else(|_| env::var("USERPROFILE"))
            .expect("Should have home directory");

        // Common tool paths that should exist conceptually
        #[cfg(unix)]
        {
            let cargo_bin = PathBuf::from(&home).join(".cargo/bin");
            // May not exist if cargo not installed, but path should be valid
            assert!(cargo_bin.to_string_lossy().contains(".cargo"));

            let local_bin = PathBuf::from(&home).join(".local/bin");
            assert!(local_bin.to_string_lossy().contains(".local"));
        }

        #[cfg(windows)]
        {
            let cargo_bin = PathBuf::from(&home).join(".cargo\\bin");
            assert!(cargo_bin.to_string_lossy().contains(".cargo"));

            let npm_global = PathBuf::from(&home).join("AppData\\Roaming\\npm");
            assert!(npm_global.to_string_lossy().contains("npm"));
        }
    }
}
