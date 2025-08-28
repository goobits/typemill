const { spawn } = require('child_process');
const path = require('path');

async function testServer() {
  console.log('Starting simple server test...');
  
  const serverProcess = spawn('node', ['dist/index.js'], {
    stdio: ['pipe', 'pipe', 'pipe'],
    cwd: __dirname,
    env: { ...process.env, CCLSP_CONFIG_PATH: path.join(__dirname, 'test-config.json') }
  });

  serverProcess.stderr.on('data', (data) => {
    console.log('STDERR:', data.toString());
  });

  serverProcess.stdout.on('data', (data) => {
    console.log('STDOUT:', data.toString());
  });

  // Wait for server to start
  await new Promise(resolve => setTimeout(resolve, 3000));

  // Send a tools/list request first to see if basic MCP is working
  const listRequest = {
    jsonrpc: '2.0',
    id: 1,
    method: 'tools/list',
    params: {}
  };

  const message = JSON.stringify(listRequest);
  const contentLength = Buffer.byteLength(message, 'utf8');
  const headers = `Content-Length: ${contentLength}\r\n\r\n`;
  
  console.log('Sending request:', headers + message);
  serverProcess.stdin.write(headers + message);

  // Wait for response
  await new Promise(resolve => setTimeout(resolve, 5000));
  
  serverProcess.kill();
}

testServer().catch(console.error);