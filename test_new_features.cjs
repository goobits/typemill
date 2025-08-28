const { spawn } = require('child_process');
const path = require('path');

// Test configuration
const TEST_FILE = path.join(__dirname, 'playground/src/components/user-form.ts');
const TEST_RESULTS = [];

class MCPTestClient {
  constructor() {
    this.requestId = 1;
    this.process = null;
  }

  async start() {
    return new Promise((resolve, reject) => {
      this.process = spawn('node', ['dist/index.js'], {
        stdio: ['pipe', 'pipe', 'pipe'],
        cwd: __dirname
      });

      this.process.stderr.on('data', (data) => {
        const message = data.toString();
        if (message.includes('CCLSP Server running on stdio')) {
          console.log('✅ CCLSP Server started successfully');
          resolve();
        }
      });

      this.process.on('error', reject);
      setTimeout(() => reject(new Error('Server failed to start')), 10000);
    });
  }

  async sendRequest(method, params) {
    return new Promise((resolve, reject) => {
      const id = this.requestId++;
      const request = {
        jsonrpc: '2.0',
        id,
        method,
        params
      };

      let responseBuffer = '';
      
      const handleData = (data) => {
        responseBuffer += data.toString();
        
        // Look for complete JSON-RPC responses
        const lines = responseBuffer.split('\n');
        for (let i = 0; i < lines.length - 1; i++) {
          const line = lines[i].trim();
          if (line.startsWith('Content-Length:')) continue;
          if (line === '') continue;
          
          try {
            const response = JSON.parse(line);
            if (response.id === id) {
              this.process.stdout.removeListener('data', handleData);
              resolve(response);
              return;
            }
          } catch (e) {
            // Continue parsing
          }
        }
        responseBuffer = lines[lines.length - 1];
      };

      this.process.stdout.on('data', handleData);

      const message = JSON.stringify(request);
      const contentLength = Buffer.byteLength(message, 'utf8');
      const headers = `Content-Length: ${contentLength}\r\n\r\n`;
      
      this.process.stdin.write(headers + message);
      
      setTimeout(() => {
        this.process.stdout.removeListener('data', handleData);
        reject(new Error('Request timeout'));
      }, 15000);
    });
  }

  async stop() {
    if (this.process) {
      this.process.kill();
    }
  }
}

// Test functions
async function testGetFoldingRanges(client) {
  console.log('\n🔍 Testing get_folding_ranges...');
  try {
    const response = await client.sendRequest('tools/call', {
      name: 'get_folding_ranges',
      arguments: { file_path: TEST_FILE }
    });

    if (response.result && response.result.content) {
      const content = response.result.content[0].text;
      console.log('✅ get_folding_ranges response:', content.substring(0, 200) + '...');
      TEST_RESULTS.push({ test: 'get_folding_ranges', status: 'PASS', details: 'Returned folding range data' });
      return true;
    } else {
      console.log('❌ get_folding_ranges failed:', response.error || 'No content returned');
      TEST_RESULTS.push({ test: 'get_folding_ranges', status: 'FAIL', details: response.error || 'No content' });
      return false;
    }
  } catch (error) {
    console.log('❌ get_folding_ranges error:', error.message);
    TEST_RESULTS.push({ test: 'get_folding_ranges', status: 'ERROR', details: error.message });
    return false;
  }
}

async function testGetSignatureHelp(client) {
  console.log('\n🔍 Testing get_signature_help...');
  try {
    const response = await client.sendRequest('tools/call', {
      name: 'get_signature_help',
      arguments: { 
        file_path: TEST_FILE,
        position: { line: 10, character: 15 } // Approximate position
      }
    });

    if (response.result) {
      const content = response.result.content[0].text;
      console.log('✅ get_signature_help response:', content.substring(0, 200) + '...');
      TEST_RESULTS.push({ test: 'get_signature_help', status: 'PASS', details: 'Returned signature help data' });
      return true;
    } else {
      console.log('❌ get_signature_help failed:', response.error || 'No result');
      TEST_RESULTS.push({ test: 'get_signature_help', status: 'FAIL', details: response.error || 'No result' });
      return false;
    }
  } catch (error) {
    console.log('❌ get_signature_help error:', error.message);
    TEST_RESULTS.push({ test: 'get_signature_help', status: 'ERROR', details: error.message });
    return false;
  }
}

async function testGetDocumentLinks(client) {
  console.log('\n🔍 Testing get_document_links...');
  try {
    const response = await client.sendRequest('tools/call', {
      name: 'get_document_links',
      arguments: { file_path: TEST_FILE }
    });

    if (response.result && response.result.content) {
      const content = response.result.content[0].text;
      console.log('✅ get_document_links response:', content.substring(0, 200) + '...');
      TEST_RESULTS.push({ test: 'get_document_links', status: 'PASS', details: 'Returned document links data' });
      return true;
    } else {
      console.log('❌ get_document_links failed:', response.error || 'No content');
      TEST_RESULTS.push({ test: 'get_document_links', status: 'FAIL', details: response.error || 'No content' });
      return false;
    }
  } catch (error) {
    console.log('❌ get_document_links error:', error.message);
    TEST_RESULTS.push({ test: 'get_document_links', status: 'ERROR', details: error.message });
    return false;
  }
}

async function testCreateFile(client) {
  console.log('\n🔍 Testing create_file...');
  const testFilePath = path.join(__dirname, 'playground/src/test-created-file.ts');
  
  try {
    const response = await client.sendRequest('tools/call', {
      name: 'create_file',
      arguments: { 
        file_path: testFilePath,
        content: '// This is a test file\nexport const testValue = 42;\n'
      }
    });

    if (response.result && response.result.content) {
      const content = response.result.content[0].text;
      console.log('✅ create_file response:', content);
      TEST_RESULTS.push({ test: 'create_file', status: 'PASS', details: 'Successfully created file' });
      return true;
    } else {
      console.log('❌ create_file failed:', response.error || 'No content');
      TEST_RESULTS.push({ test: 'create_file', status: 'FAIL', details: response.error || 'No content' });
      return false;
    }
  } catch (error) {
    console.log('❌ create_file error:', error.message);
    TEST_RESULTS.push({ test: 'create_file', status: 'ERROR', details: error.message });
    return false;
  }
}

async function testDeleteFile(client) {
  console.log('\n🔍 Testing delete_file...');
  const testFilePath = path.join(__dirname, 'playground/src/test-created-file.ts');
  
  try {
    const response = await client.sendRequest('tools/call', {
      name: 'delete_file',
      arguments: { file_path: testFilePath }
    });

    if (response.result && response.result.content) {
      const content = response.result.content[0].text;
      console.log('✅ delete_file response:', content);
      TEST_RESULTS.push({ test: 'delete_file', status: 'PASS', details: 'Successfully deleted file' });
      return true;
    } else {
      console.log('❌ delete_file failed:', response.error || 'No content');
      TEST_RESULTS.push({ test: 'delete_file', status: 'FAIL', details: response.error || 'No content' });
      return false;
    }
  } catch (error) {
    console.log('❌ delete_file error:', error.message);
    TEST_RESULTS.push({ test: 'delete_file', status: 'ERROR', details: error.message });
    return false;
  }
}

async function testApplyWorkspaceEdit(client) {
  console.log('\n🔍 Testing apply_workspace_edit...');
  const testFile = path.join(__dirname, 'playground/src/index.ts');
  
  try {
    const response = await client.sendRequest('tools/call', {
      name: 'apply_workspace_edit',
      arguments: {
        changes: {
          [testFile]: [{
            range: {
              start: { line: 0, character: 0 },
              end: { line: 0, character: 0 }
            },
            newText: '// Test comment added by workspace edit\n'
          }]
        }
      }
    });

    if (response.result && response.result.content) {
      const content = response.result.content[0].text;
      console.log('✅ apply_workspace_edit response:', content);
      TEST_RESULTS.push({ test: 'apply_workspace_edit', status: 'PASS', details: 'Successfully applied workspace edit' });
      return true;
    } else {
      console.log('❌ apply_workspace_edit failed:', response.error || 'No content');
      TEST_RESULTS.push({ test: 'apply_workspace_edit', status: 'FAIL', details: response.error || 'No content' });
      return false;
    }
  } catch (error) {
    console.log('❌ apply_workspace_edit error:', error.message);
    TEST_RESULTS.push({ test: 'apply_workspace_edit', status: 'ERROR', details: error.message });
    return false;
  }
}

async function runAllTests() {
  const client = new MCPTestClient();
  
  try {
    console.log('🚀 Starting CCLSP Feature Tests...');
    await client.start();
    
    // Give server time to initialize
    await new Promise(resolve => setTimeout(resolve, 2000));

    // Run all tests
    await testGetFoldingRanges(client);
    await testGetSignatureHelp(client);
    await testGetDocumentLinks(client);
    await testCreateFile(client);
    await testDeleteFile(client);
    await testApplyWorkspaceEdit(client);

    // Print summary
    console.log('\n📊 TEST SUMMARY:');
    console.log('================');
    let passCount = 0;
    let failCount = 0;
    let errorCount = 0;

    TEST_RESULTS.forEach(result => {
      const status = result.status === 'PASS' ? '✅' : result.status === 'FAIL' ? '❌' : '⚠️';
      console.log(`${status} ${result.test}: ${result.status} - ${result.details}`);
      
      if (result.status === 'PASS') passCount++;
      else if (result.status === 'FAIL') failCount++;
      else errorCount++;
    });

    console.log(`\nResults: ${passCount} passed, ${failCount} failed, ${errorCount} errors`);
    
  } catch (error) {
    console.error('💥 Test suite failed:', error);
  } finally {
    await client.stop();
  }
}

// Run the tests
runAllTests().catch(console.error);