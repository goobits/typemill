const { spawn } = require('child_process');
const path = require('path');

class MCPTestClient {
  constructor() {
    this.process = null;
    this.buffer = '';
  }

  async start() {
    return new Promise((resolve, reject) => {
      this.process = spawn('node', ['dist/index.js'], {
        stdio: ['pipe', 'pipe', 'pipe'],
        cwd: __dirname,
        env: { ...process.env, CCLSP_CONFIG_PATH: path.join(__dirname, 'test-config.json') },
      });

      this.process.stderr.on('data', (data) => {
        const message = data.toString();
        console.log('SERVER:', message.trim());
        if (message.includes('LSP server preloading completed')) {
          setTimeout(resolve, 1000); // Give a moment after completion
        }
      });

      this.process.on('error', reject);
      setTimeout(() => reject(new Error('Server failed to start')), 15000);
    });
  }

  sendRequest(method, params) {
    return new Promise((resolve, reject) => {
      const request = {
        jsonrpc: '2.0',
        id: 1,
        method,
        params,
      };

      let responseBuffer = '';
      let responseReceived = false;

      const handleData = (data) => {
        if (responseReceived) return;

        responseBuffer += data.toString();
        console.log('RAW RESPONSE:', responseBuffer);

        // Look for Content-Length header
        const headerMatch = responseBuffer.match(/Content-Length: (\d+)\r?\n\r?\n(.*)$/s);
        if (headerMatch) {
          const contentLength = Number.parseInt(headerMatch[1]);
          const content = headerMatch[2];

          if (content.length >= contentLength) {
            const jsonResponse = content.substring(0, contentLength);
            try {
              const response = JSON.parse(jsonResponse);
              responseReceived = true;
              this.process.stdout.removeListener('data', handleData);
              resolve(response);
            } catch (e) {
              console.log('JSON parse error:', e);
              reject(e);
            }
          }
        }
      };

      this.process.stdout.on('data', handleData);

      const message = JSON.stringify(request);
      const contentLength = Buffer.byteLength(message, 'utf8');
      const headers = `Content-Length: ${contentLength}\r\n\r\n`;

      console.log('SENDING:', headers + message);
      this.process.stdin.write(headers + message);

      setTimeout(() => {
        if (!responseReceived) {
          this.process.stdout.removeListener('data', handleData);
          reject(new Error('Request timeout'));
        }
      }, 10000);
    });
  }

  async stop() {
    if (this.process) {
      this.process.kill();
    }
  }
}

async function runTest() {
  const client = new MCPTestClient();

  try {
    console.log('🚀 Starting CCLSP Test...');
    await client.start();
    console.log('✅ Server started successfully');

    // Test 1: List tools
    console.log('\n📋 Testing tools/list...');
    const toolsResponse = await client.sendRequest('tools/list', {});
    console.log('✅ Tools response:', JSON.stringify(toolsResponse, null, 2));

    // Test 2: Test get_folding_ranges
    console.log('\n🔍 Testing get_folding_ranges...');
    const foldingResponse = await client.sendRequest('tools/call', {
      name: 'get_folding_ranges',
      arguments: {
        file_path: path.join(__dirname, 'playground/src/components/user-form.ts'),
      },
    });
    console.log('✅ Folding ranges response:', JSON.stringify(foldingResponse, null, 2));
  } catch (error) {
    console.error('❌ Test failed:', error);
  } finally {
    await client.stop();
  }
}

runTest();
