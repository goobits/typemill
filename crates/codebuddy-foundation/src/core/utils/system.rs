//! System-level utilities

/// Check if a command exists on the system's PATH
pub fn command_exists(cmd: &str) -> bool {
    std::process::Command::new(if cfg!(target_os = "windows") {
        "where"
    } else {
        "command"
    })
    .arg("-v")
    .arg(cmd)
    .stdout(std::process::Stdio::null())
    .stderr(std::process::Stdio::null())
    .status()
    .is_ok_and(|status| status.success())
}
