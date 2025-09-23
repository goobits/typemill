import { cpus, loadavg, totalmem } from 'node:os';

export interface SystemSpecs {
  cpuCores: number;
  totalRAM: number; // in GB
  loadAverage: number;
  isSlowSystem: boolean;
  testMode: 'fast' | 'slow';
}

export interface TestConfig {
  mode: 'fast' | 'slow';
  timeouts: {
    initialization: number;
    toolCall: number;
    testCase: number;
  };
  concurrency: {
    maxConcurrentServers: number;
    sequentialExecution: boolean;
  };
  features: {
    enableProjectScanner: boolean;
    enableAllLanguageServers: boolean;
    maxRelatedFiles: number;
  };
}

/**
 * Auto-detect system capabilities and determine test mode
 */
export function detectSystemSpecs(): SystemSpecs {
  const cpuCores = cpus().length;
  const totalRAM = Math.round(totalmem() / (1024 * 1024 * 1024)); // Convert to GB
  const loadAverage = loadavg()[0]; // 1-minute load average

  // Determine if system is slow based on multiple factors
  const isSlowSystem =
    cpuCores < 4 || // Less than 4 CPU cores
    totalRAM < 8 || // Less than 8GB RAM
    loadAverage > cpuCores * 0.8; // High CPU load

  const testMode = isSlowSystem ? 'slow' : 'fast';

  return {
    cpuCores,
    totalRAM,
    loadAverage,
    isSlowSystem,
    testMode,
  };
}

/**
 * Get test configuration based on system capabilities
 */
export function getTestConfig(mode?: 'fast' | 'slow'): TestConfig {
  const specs = detectSystemSpecs();
  const selectedMode = mode || specs.testMode;

  if (selectedMode === 'slow') {
    return {
      mode: 'slow',
      timeouts: {
        initialization: 60000, // 60 seconds for LSP init
        toolCall: 120000, // 2 minutes per MCP tool call
        testCase: 300000, // 5 minutes per test case
      },
      concurrency: {
        maxConcurrentServers: 2, // Only 2 LSP servers at once
        sequentialExecution: true, // Run tests one by one
      },
      features: {
        enableProjectScanner: false, // Disable expensive file scanning
        enableAllLanguageServers: false, // Only essential servers
        maxRelatedFiles: 5, // Limit related file opens
      },
    };
  }
  return {
    mode: 'fast',
    timeouts: {
      initialization: 10000, // 10 seconds for LSP init
      toolCall: 30000, // 30 seconds per MCP tool call
      testCase: 60000, // 1 minute per test case
    },
    concurrency: {
      maxConcurrentServers: 8, // Up to 8 concurrent servers
      sequentialExecution: false, // Parallel test execution
    },
    features: {
      enableProjectScanner: true, // Full project scanning
      enableAllLanguageServers: true, // All available servers
      maxRelatedFiles: 30, // Full related file support
    },
  };
}

/**
 * Create minimal config for slow systems (TypeScript only)
 */
export function getMinimalConfig() {
  return {
    servers: [
      {
        extensions: ['ts', 'tsx', 'js', 'jsx', 'mjs', 'cjs'],
        command: ['npx', '--', 'typescript-language-server', '--stdio'],
      },
    ],
  };
}

/**
 * Log system information for debugging
 */
export function logSystemInfo(): void {
  const specs = detectSystemSpecs();
  const config = getTestConfig();

  console.log('\n🖥️  System Information:');
  console.log(`   CPU Cores: ${specs.cpuCores}`);
  console.log(`   Total RAM: ${specs.totalRAM}GB`);
  console.log(`   Load Average: ${specs.loadAverage.toFixed(2)}`);
  console.log(`   Test Mode: ${specs.testMode.toUpperCase()}`);

  console.log('\n⚙️  Test Configuration:');
  console.log(`   Tool Call Timeout: ${config.timeouts.toolCall / 1000}s`);
  console.log(`   Max Concurrent Servers: ${config.concurrency.maxConcurrentServers}`);
  console.log(`   Sequential Execution: ${config.concurrency.sequentialExecution}`);
  console.log(
    `   Project Scanner: ${config.features.enableProjectScanner ? 'Enabled' : 'Disabled'}`
  );
  console.log('');
}

/**
 * Override test mode via environment variable
 */
export function getTestModeFromEnv(): 'fast' | 'slow' | null {
  const envMode = process.env.TEST_MODE?.toLowerCase();
  if (envMode === 'fast' || envMode === 'slow') {
    return envMode;
  }
  return null;
}
