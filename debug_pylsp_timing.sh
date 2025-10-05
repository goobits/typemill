#!/bin/bash
# Debug script to test pylsp initialization timing in a test-like environment

set -e

echo "=== Testing pylsp Initialization Timing ==="
echo ""

# Create a temporary test workspace similar to the integration test
TEST_DIR=$(mktemp -d)
echo "Test workspace: $TEST_DIR"
cd "$TEST_DIR"

# Create a .codebuddy directory and config
mkdir -p .codebuddy

# Create the same config that the test uses
cat > .codebuddy/config.json <<'EOF'
{
  "server": {
    "host": "127.0.0.1",
    "port": 3000,
    "timeoutMs": 30000
  },
  "lsp": {
    "servers": [
      {
        "extensions": ["py"],
        "command": ["/home/developer/.local/bin/pylsp"],
        "rootDir": null,
        "restartInterval": 5
      }
    ],
    "defaultTimeoutMs": 30000,
    "enablePreload": true
  },
  "logging": {
    "level": "debug",
    "format": "json"
  }
}
EOF

# Create a simple Python file like in the test
cat > validate.py <<'EOF'
def validate_user_data(user_data):
    """Validate user data structure"""
    required_fields = ['name', 'email', 'age']
    return all(field in user_data for field in required_fields)

def process_user_data(user_data):
    """Process user data"""
    if validate_user_data(user_data):
        return {
            'status': 'success',
            'processed_data': user_data
        }
    return {'status': 'error', 'message': 'Invalid data'}
EOF

echo "Created test files:"
ls -la

echo ""
echo "=== Starting codebuddy server with RUST_LOG=debug ==="
echo "This will show LSP initialization timing logs"
echo ""

# Run the server in the background and capture logs
RUST_LOG=debug /workspace/target/release/codebuddy serve 2>&1 | tee /tmp/codebuddy_debug.log &
SERVER_PID=$!

echo "Server PID: $SERVER_PID"
echo "Waiting 5 seconds for server to start..."
sleep 5

echo ""
echo "=== Making a test request to trigger pylsp initialization ==="
echo ""

# Use the CLI client to make a request
timeout 90 /workspace/target/release/codebuddy-cli call get_document_symbols "{\"file_path\": \"$TEST_DIR/validate.py\"}" 2>&1 | tee /tmp/client_output.log || {
  EXIT_CODE=$?
  if [ $EXIT_CODE -eq 124 ]; then
    echo ""
    echo "❌ TIMEOUT: Request took more than 90 seconds!"
  else
    echo ""
    echo "❌ Request failed with exit code: $EXIT_CODE"
  fi
}

echo ""
echo "=== Cleanup ==="
kill $SERVER_PID 2>/dev/null || true
sleep 2
rm -rf "$TEST_DIR"

echo ""
echo "=== Analyzing logs for LSP initialization timing ==="
echo ""
echo "Looking for initialization messages in /tmp/codebuddy_debug.log:"
grep -E "(Sending LSP initialize|initialized successfully|TIMEOUT)" /tmp/codebuddy_debug.log || echo "No initialization messages found"

echo ""
echo "Full debug log saved to: /tmp/codebuddy_debug.log"
echo "Client output saved to: /tmp/client_output.log"
