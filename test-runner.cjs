#!/usr/bin/env node

const { spawn } = require('node:child_process');
const { cpus } = require('node:os');

// Detect system capabilities
const cpuCount = cpus().length;
const totalMemory = require('node:os').totalmem();
const isSlowSystem = cpuCount <= 2 || totalMemory < 4 * 1024 * 1024 * 1024;

console.log('System Detection:');
console.log(`  CPUs: ${cpuCount}`);
console.log(`  RAM: ${(totalMemory / (1024 * 1024 * 1024)).toFixed(1)}GB`);
console.log(`  Mode: ${isSlowSystem ? 'SLOW' : 'FAST'}`);
console.log('');

// Test configuration based on system
const config = {
  timeout: isSlowSystem ? 180000 : 60000,
  parallel: !isSlowSystem,
  sharedServer: true,
  warmupDelay: isSlowSystem ? 10000 : 3000,
};

// Environment variables for tests
const testEnv = {
  ...process.env,
  TEST_MODE: isSlowSystem ? 'slow' : 'fast',
  TEST_SHARED_SERVER: 'true',
  TEST_TIMEOUT: config.timeout.toString(),
  BUN_TEST_TIMEOUT: config.timeout.toString(),
};

// Run tests with appropriate configuration
async function runTests() {
  const args = [
    'test',
    'tests/integration/call-hierarchy.test.ts',
    '--timeout',
    config.timeout.toString(),
  ];

  if (!config.parallel) {
    console.log('Running tests sequentially (slow system mode)...\n');
    // Force sequential execution in Bun
    args.push('--bail', '1'); // Stop on first failure
  } else {
    console.log('Running tests in parallel (fast system mode)...\n');
  }

  const proc = spawn('bun', args, {
    env: testEnv,
    stdio: 'inherit',
  });

  return new Promise((resolve, reject) => {
    proc.on('exit', (code) => {
      if (code === 0) {
        resolve(code);
      } else {
        reject(new Error(`Tests failed with code ${code}`));
      }
    });
  });
}

// Main execution
(async () => {
  try {
    await runTests();
    console.log('\n✅ All tests passed!');
    process.exit(0);
  } catch (error) {
    console.error('\n❌ Tests failed:', error.message);
    process.exit(1);
  }
})();
