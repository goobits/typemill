use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::path::Path;

/// Test client for interacting with the cb-server binary.
/// Manages process lifecycle and JSON-RPC communication.
pub struct TestClient {
    pub process: Child,
    pub stdin: ChildStdin,
    pub stdout_receiver: mpsc::Receiver<String>,
    pub stderr_receiver: mpsc::Receiver<String>,
}

impl TestClient {
    /// Spawns cb-server in stdio mode with the given working directory.
    pub fn new(working_dir: &Path) -> Self {
        // Determine the path to the cb-server binary
        // Use absolute paths for reliability
        let possible_paths = vec![
            "/workspace/rust/target/release/cb-server",
            "/workspace/rust/target/debug/cb-server",
            "target/release/cb-server",
            "target/debug/cb-server",
        ];

        let server_path = possible_paths
            .iter()
            .find(|path| Path::new(path).exists())
            .unwrap_or(&"cb-server");

        eprintln!("DEBUG: TestClient using server path: {}", server_path);

        let mut process = Command::new(server_path)
            .arg("start")
            .current_dir(working_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to start cb-server binary");

        let stdin = process.stdin.take().unwrap();
        let stdout = process.stdout.take().unwrap();
        let stderr = process.stderr.take().unwrap();

        // Spawn thread to read stdout
        let (stdout_sender, stdout_receiver) = mpsc::channel();
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(line) = line {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() && trimmed.starts_with('{') {
                        if stdout_sender.send(line).is_err() {
                            break;
                        }
                    }
                }
            }
        });

        // Spawn thread to read stderr (for debugging crashes)
        let (stderr_sender, stderr_receiver) = mpsc::channel();
        thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                if let Ok(line) = line {
                    if stderr_sender.send(line).is_err() {
                        break;
                    }
                }
            }
        });

        // Wait for server startup
        thread::sleep(Duration::from_millis(1500));

        TestClient {
            process,
            stdin,
            stdout_receiver,
            stderr_receiver,
        }
    }

    /// Send a JSON-RPC request and wait for response.
    pub fn send_request(&mut self, request: Value) -> Result<Value, Box<dyn std::error::Error>> {
        let request_str = serde_json::to_string(&request)?;
        writeln!(self.stdin, "{}", request_str)?;
        self.stdin.flush()?;

        // Wait for response with extended timeout for resilience tests
        let response_str = self.stdout_receiver.recv_timeout(Duration::from_secs(15))?;
        let response: Value = serde_json::from_str(&response_str)?;
        Ok(response)
    }

    /// Send a tools/call request with the given tool name and arguments.
    pub fn call_tool(&mut self, tool_name: &str, arguments: Value) -> Result<Value, Box<dyn std::error::Error>> {
        static mut REQUEST_ID: i32 = 0;
        let id = unsafe {
            REQUEST_ID += 1;
            REQUEST_ID
        };

        let request = json!({
            "jsonrpc": "2.0",
            "id": format!("test-{}", id),
            "method": "tools/call",
            "params": {
                "name": tool_name,
                "arguments": arguments
            }
        });

        self.send_request(request)
    }

    /// Check if the server process is still alive.
    pub fn is_alive(&mut self) -> bool {
        match self.process.try_wait() {
            Ok(Some(_)) => false, // Process has exited
            Ok(None) => true,     // Process is still running
            Err(_) => false,      // Error checking status
        }
    }

    /// Get stderr logs for debugging.
    pub fn get_stderr_logs(&self) -> Vec<String> {
        let mut logs = Vec::new();
        while let Ok(line) = self.stderr_receiver.try_recv() {
            logs.push(line);
        }
        logs
    }

    /// Get child processes (LSP servers spawned by cb-server).
    pub fn get_child_processes(&self) -> Vec<u32> {
        // Find child processes (LSP servers spawned by cb-server)
        let output = Command::new("pgrep")
            .arg("-P")
            .arg(self.process.id().to_string())
            .output();

        if let Ok(output) = output {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .filter_map(|line| line.trim().parse::<u32>().ok())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Gracefully shutdown the server.
    pub fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Try to send a shutdown request first
        let shutdown_request = json!({
            "jsonrpc": "2.0",
            "id": "shutdown",
            "method": "shutdown",
            "params": {}
        });

        // Attempt graceful shutdown
        if let Ok(_) = self.send_request(shutdown_request) {
            // Give the server time to shut down gracefully
            thread::sleep(Duration::from_millis(500));
        }

        // Kill the process if it's still alive
        if self.is_alive() {
            self.process.kill()?;
            self.process.wait()?;
        }

        Ok(())
    }
}

impl Drop for TestClient {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}