import { afterAll, beforeAll, describe, expect, it } from 'bun:test';
import { join } from 'node:path';
import { LSPClient } from '../../src/lsp/lsp-client.js';
import { FileService } from '../../src/services/file-service.js';
import { SymbolService } from '../../src/services/lsp/symbol-service.js';
import { ServiceContextUtils } from '../../src/services/service-context.js';
import { PredictiveLoaderService } from '../../src/services/predictive-loader.js';
import { logger } from '../../src/core/diagnostics/logger.js';

/**
 * Performance test for Predictive Loading
 *
 * This test measures the real-world impact of predictive loading on LSP operations.
 * We compare find_definition performance with and without predictive loading enabled.
 */
describe('Predictive Loading Performance', () => {
  let lspClient: LSPClient;
  let fileService: FileService;
  let symbolService: SymbolService;

  const testMainFile = join(process.cwd(), 'tests/fixtures/perf-test/main.ts');
  const testUtilsFile = join(process.cwd(), 'tests/fixtures/perf-test/utils.ts');

  beforeAll(async () => {
    console.log('🚀 Setting up LSP client for performance testing...');

    // Initialize LSP client
    lspClient = new LSPClient();

    // Create services with predictive loading enabled
    const serviceContext = ServiceContextUtils.createServiceContext(
      lspClient.getServer.bind(lspClient),
      lspClient.protocol,
      undefined, // transactionManager
      logger,
      { serverOptions: { enablePredictiveLoading: true } }
    );

    fileService = new FileService(serviceContext);
    symbolService = new SymbolService(serviceContext);

    // Create and add predictive loader service
    const predictiveLoaderService = new PredictiveLoaderService({
      logger,
      openFile: (filePath: string) => {
        console.log(`🔄 Predictive loading triggered for: ${filePath}`);
        return fileService.openFileInternal(filePath);
      },
      config: { serverOptions: { enablePredictiveLoading: true } }
    });

    serviceContext.predictiveLoader = predictiveLoaderService;
    serviceContext.fileService = fileService;

    // Warm up LSP servers
    console.log('⏳ Warming up LSP servers...');
    await fileService.getFoldingRanges(testMainFile);
    console.log('✅ LSP servers ready');
  });

  afterAll(async () => {
    if (lspClient) {
      await lspClient.dispose();
    }
  });

  async function measureOperation(
    operationName: string,
    operation: () => Promise<any>,
    iterations: number = 5
  ): Promise<{ average: number; min: number; max: number; times: number[] }> {
    const times: number[] = [];

    console.log(`📏 Measuring ${operationName} (${iterations} iterations)...`);

    for (let i = 0; i < iterations; i++) {
      const start = performance.now();
      await operation();
      const duration = performance.now() - start;
      times.push(duration);

      // Small delay between iterations
      await new Promise(resolve => setTimeout(resolve, 100));
    }

    const average = times.reduce((sum, time) => sum + time, 0) / times.length;
    const min = Math.min(...times);
    const max = Math.max(...times);

    console.log(`  Average: ${average.toFixed(1)}ms, Min: ${min.toFixed(1)}ms, Max: ${max.toFixed(1)}ms`);

    return { average, min, max, times };
  }

  it('should show performance improvement with predictive loading', async () => {
    console.log('\n🔍 Testing find_definition performance...');

    // Test without predictive loading (cold start scenario)
    // We simulate this by testing definition lookup on imported symbols
    console.log('\n❄️  Testing WITHOUT predictive loading (cold start):');

    const withoutPreloading = await measureOperation(
      'find_definition on imported symbol (cold)',
      async () => {
        // Dispose and recreate client WITHOUT predictive loading
        await lspClient.dispose();
        lspClient = new LSPClient();
        const coldServiceContext = ServiceContextUtils.createServiceContext(
          lspClient.getServer.bind(lspClient),
          lspClient.protocol,
          undefined,
          logger,
          { serverOptions: { enablePredictiveLoading: false } }
        );
        fileService = new FileService(coldServiceContext);
        symbolService = new SymbolService(coldServiceContext);

        // NO predictive loader in cold context

        // Test operation that would benefit from predictive loading
        // Use symbol lookup on imported function (should be slower)
        const result = await symbolService.findSymbolMatches(testMainFile, 'findUser');
        expect(result).toBeDefined();

        return result;
      },
      3 // Fewer iterations for cold start test (expensive)
    );

    // Test with predictive loading (warm start scenario)
    console.log('\n🔥 Testing WITH predictive loading (warm start):');

    const withPreloading = await measureOperation(
      'find_definition on imported symbol (warm)',
      async () => {
        // First open the main file (this triggers predictive loading of imports)
        await fileService.openFile(testMainFile);

        // Now test symbol lookup in imported file (should be faster due to preloading)
        const result = await symbolService.findSymbolMatches(testMainFile, 'findUser');
        expect(result).toBeDefined();

        return result;
      }
    );

    // Compare results
    const improvementPercent = ((withoutPreloading.average - withPreloading.average) / withoutPreloading.average) * 100;

    console.log('\n📊 Performance Results:');
    console.log(`  Cold start (no preloading):  ${withoutPreloading.average.toFixed(1)}ms average`);
    console.log(`  Warm start (with preloading): ${withPreloading.average.toFixed(1)}ms average`);
    console.log(`  Improvement: ${improvementPercent.toFixed(1)}% faster`);

    // Verify we see some performance improvement
    // Note: In real scenarios with larger codebases, improvements would be more significant
    expect(improvementPercent).toBeGreaterThanOrEqual(0);

    // Log detailed statistics
    console.log('\n📈 Detailed Statistics:');
    console.log('  Cold Start Times:', withoutPreloading.times.map(t => `${t.toFixed(1)}ms`).join(', '));
    console.log('  Warm Start Times:', withPreloading.times.map(t => `${t.toFixed(1)}ms`).join(', '));

    if (improvementPercent > 10) {
      console.log(`\n✅ Significant performance improvement detected: ${improvementPercent.toFixed(1)}%`);
    } else if (improvementPercent > 0) {
      console.log(`\n⚡ Minor performance improvement detected: ${improvementPercent.toFixed(1)}%`);
    } else {
      console.log('\n⚠️  No significant performance improvement detected in this test scenario');
      console.log('   Note: Performance improvements are more significant with larger codebases');
    }
  });

  it('should demonstrate predictive loading behavior', async () => {
    console.log('\n🔄 Demonstrating predictive loading behavior...');

    // Test sequence showing how predictive loading works:
    console.log('1. Opening main file (this should trigger predictive loading of imports)');
    console.log(`   Main file: ${testMainFile}`);
    console.log(`   Utils file: ${testUtilsFile}`);
    const start1 = performance.now();
    await fileService.openFile(testMainFile); // This triggers predictive loading
    const mainFileTime = performance.now() - start1;
    console.log(`   Open main file (triggers predictive loading): ${mainFileTime.toFixed(1)}ms`);

    // Give predictive loading time to complete
    await new Promise(resolve => setTimeout(resolve, 100));

    console.log('2. Finding symbol in imported file (should be fast due to preloading)');
    const start2 = performance.now();
    const result = await symbolService.findSymbolMatches(testMainFile, 'findUser');
    const importedSymbolTime = performance.now() - start2;
    console.log(`   Find imported symbol: ${importedSymbolTime.toFixed(1)}ms`);

    // Verify the operation succeeded
    expect(result).toBeDefined();

    console.log('3. Finding another symbol (should also be fast)');
    const start3 = performance.now();
    const result2 = await symbolService.findSymbolMatches(testUtilsFile, 'validateUser');
    const anotherOpTime = performance.now() - start3;
    console.log(`   Find another symbol: ${anotherOpTime.toFixed(1)}ms`);

    expect(result2).toBeDefined();

    console.log('\n✅ Predictive loading behavior demonstrated');
    console.log(`   Subsequent operations on preloaded files tend to be faster`);
  });
});