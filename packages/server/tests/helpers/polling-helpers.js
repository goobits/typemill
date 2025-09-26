/**
 * Advanced polling helpers for reliable test execution
 * These replace setTimeout calls with intelligent condition-based waiting
 */

/**
 * Generic condition polling helper
 * @param {() => boolean | Promise<boolean>} condition - Condition to check
 * @param {object} options - Options for timeout and interval
 * @returns {Promise<void>}
 */
export async function waitForCondition(condition, options = {}) {
  const { timeout = 10000, interval = 100 } = options;
  const startTime = Date.now();

  while (Date.now() - startTime < timeout) {
    const result = await condition();
    if (result) {
      return;
    }
    await new Promise((resolve) => setTimeout(resolve, interval));
  }

  throw new Error(`Condition not met within ${timeout}ms`);
}

/**
 * Wait for LSP servers to initialize properly
 * @param {import('./mcp-test-client.js').MCPTestClient} client - MCP test client
 * @param {object} options - Options for timeout and interval
 * @returns {Promise<void>}
 */
export async function waitForLSPInitialization(client, options = {}) {
  const { timeout = 8000, interval = 500 } = options;

  await waitForCondition(async () => {
    try {
      // Try a simple health check or lightweight operation
      const result = await client.callTool('health_check', { include_details: false });
      return result && result.content && result.content.length > 0;
    } catch (error) {
      // LSP not ready yet
      return false;
    }
  }, { timeout, interval });
}

/**
 * Wait for a file system operation to complete
 * @param {() => boolean | Promise<boolean>} condition - File system condition to check
 * @param {object} options - Options for timeout and interval
 * @returns {Promise<void>}
 */
export async function waitForFileOperation(condition, options = {}) {
  const { timeout = 2000, interval = 100 } = options;

  await waitForCondition(condition, { timeout, interval });
}