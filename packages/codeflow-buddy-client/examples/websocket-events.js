// Example: WebSocket client with event handling

const { WebSocketClient } = require('../dist/index.js');

async function main() {
  const client = new WebSocketClient('ws://localhost:3000', {
    token: process.env.CODEFLOW_TOKEN,
    reconnect: true,
    reconnectMaxRetries: 5,
    requestTimeout: 30000,
  });

  // Setup event listeners
  client.on('connected', () => {
    console.log('✅ Connected to server');
  });

  client.on('disconnected', ({ code, reason }) => {
    console.log(`❌ Disconnected: ${code} - ${reason}`);
  });

  client.on('reconnecting', ({ attempt, delay }) => {
    console.log(`🔄 Reconnecting (attempt ${attempt}) in ${delay}ms...`);
  });

  client.on('error', (error) => {
    console.error('❗ Error:', error.message);
  });

  client.on('notification', (notification) => {
    console.log('📢 Server notification:', notification);
  });

  client.on('status', (status) => {
    console.log(`📊 Status changed to: ${status}`);
  });

  try {
    // Connect to server
    await client.connect();

    // Send some requests
    console.log('\nSending test requests...');
    
    const result1 = await client.send('tools/list');
    console.log(`Found ${result1?.tools?.length || 0} tools`);

    const result2 = await client.send('health_check', {
      include_details: true,
    });
    console.log('Health check:', result2);

    // Wait a bit to see any notifications
    await new Promise(resolve => setTimeout(resolve, 2000));

  } catch (error) {
    console.error('Operation failed:', error);
  } finally {
    // Clean disconnect
    await client.disconnect();
    console.log('\nClean shutdown completed');
  }
}

main().catch(console.error);