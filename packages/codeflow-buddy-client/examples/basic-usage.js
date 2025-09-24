// Example: Basic usage of Codeflow Buddy Client

const { MCPProxy } = require('../dist/index.js');

async function main() {
  // Create proxy instance
  const proxy = new MCPProxy('ws://localhost:3000', {
    token: process.env.CODEFLOW_TOKEN,
  });

  try {
    // List available tools
    console.log('Fetching available tools...');
    const tools = await proxy.listTools();
    console.log(`Found ${tools?.tools?.length || 0} tools\n`);

    // Example: Find definition
    console.log('Finding definition...');
    const definition = await proxy.send({
      method: 'find_definition',
      params: {
        file_path: 'src/index.ts',
        symbol_name: 'main',
      },
    });
    console.log('Definition result:', definition);

    // Example: Batch operations
    console.log('\nRunning batch operations...');
    const results = await proxy.sendBatch([
      {
        method: 'get_diagnostics',
        params: { file_path: 'src/index.ts' },
      },
      {
        method: 'get_document_symbols',
        params: { file_path: 'src/index.ts' },
      },
    ]);
    console.log('Batch results:', results);

  } catch (error) {
    console.error('Error:', error.message);
  } finally {
    // Always disconnect when done
    await proxy.disconnect();
    console.log('\nDisconnected from server');
  }
}

// Run the example
main().catch(console.error);