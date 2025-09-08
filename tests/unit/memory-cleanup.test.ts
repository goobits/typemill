import { afterEach, beforeEach, describe, expect, it } from 'bun:test';
import { spawn } from 'node:child_process';
import type { ServerState } from '../../src/lsp-types.js';
import { LSPProtocol } from '../../src/lsp/protocol.js';
import { ServerManager } from '../../src/lsp/server-manager.js';
import type { Diagnostic } from '../../src/types.js';

describe('ServerManager Memory Cleanup', () => {
  let protocol: LSPProtocol;
  let serverManager: ServerManager;

  beforeEach(() => {
    protocol = new LSPProtocol();
    serverManager = new ServerManager(protocol);
  });

  describe('Memory cleanup functionality', () => {
    it('should clean up stale diagnostics older than 5 minutes', async () => {
      // Create a mock server state
      const mockProcess = spawn('echo', ['test'], { stdio: 'pipe' });
      const serverState: ServerState = {
        process: mockProcess,
        initialized: true,
        initializationPromise: Promise.resolve(),
        openFiles: new Set(['file1.ts', 'file2.ts']),
        fileVersions: new Map(),
        startTime: Date.now(),
        config: {
          extensions: ['ts'],
          command: ['mock-server'],
        },
        diagnostics: new Map(),
        lastDiagnosticUpdate: new Map(),
        diagnosticVersions: new Map(),
        capabilities: undefined,
        buffer: '',
      };

      // Add some diagnostics with different timestamps
      const now = Date.now();
      const fiveMinutesAgo = now - 5 * 60 * 1000; // Exactly 5 minutes ago
      const tenMinutesAgo = now - 10 * 60 * 1000; // 10 minutes ago (should be cleaned)
      const twoMinutesAgo = now - 2 * 60 * 1000; // 2 minutes ago (should be kept)

      const mockDiagnostic: Diagnostic = {
        range: { start: { line: 0, character: 0 }, end: { line: 0, character: 10 } },
        message: 'Test diagnostic',
        severity: 1,
      };

      // Set up test data
      serverState.diagnostics.set('old-file.ts', [mockDiagnostic]);
      serverState.lastDiagnosticUpdate.set('old-file.ts', tenMinutesAgo);
      serverState.diagnosticVersions.set('old-file.ts', 1);

      serverState.diagnostics.set('recent-file.ts', [mockDiagnostic]);
      serverState.lastDiagnosticUpdate.set('recent-file.ts', twoMinutesAgo);
      serverState.diagnosticVersions.set('recent-file.ts', 2);

      serverState.diagnostics.set('boundary-file.ts', [mockDiagnostic]);
      serverState.lastDiagnosticUpdate.set('boundary-file.ts', fiveMinutesAgo - 1000); // Just over 5 minutes
      serverState.diagnosticVersions.set('boundary-file.ts', 3);

      // Manually add the server to the manager for testing
      (serverManager as any).servers.set('["mock-server"]', serverState);

      // Initial counts
      expect(serverState.diagnostics.size).toBe(3);
      expect(serverState.lastDiagnosticUpdate.size).toBe(3);
      expect(serverState.diagnosticVersions.size).toBe(3);

      // Trigger cleanup
      serverManager.cleanupMemory();

      // Check that old diagnostics were cleaned up
      expect(serverState.diagnostics.size).toBe(1);
      expect(serverState.lastDiagnosticUpdate.size).toBe(1);
      expect(serverState.diagnosticVersions.size).toBe(1);

      // Verify that only the recent file remains
      expect(serverState.diagnostics.has('recent-file.ts')).toBe(true);
      expect(serverState.diagnostics.has('old-file.ts')).toBe(false);
      expect(serverState.diagnostics.has('boundary-file.ts')).toBe(false);

      // Clean up
      mockProcess.kill();
    });

    it('should limit open files to 100 (LRU behavior)', async () => {
      // Create a mock server state
      const mockProcess = spawn('echo', ['test'], { stdio: 'pipe' });
      const serverState: ServerState = {
        process: mockProcess,
        initialized: true,
        initializationPromise: Promise.resolve(),
        openFiles: new Set(),
        fileVersions: new Map(),
        startTime: Date.now(),
        config: {
          extensions: ['ts'],
          command: ['mock-server'],
        },
        diagnostics: new Map(),
        lastDiagnosticUpdate: new Map(),
        diagnosticVersions: new Map(),
        capabilities: undefined,
        buffer: '',
      };

      // Add 150 files to exceed the limit
      const fileCount = 150;
      for (let i = 0; i < fileCount; i++) {
        serverState.openFiles.add(`file${i}.ts`);
      }

      // Manually add the server to the manager for testing
      (serverManager as any).servers.set('["mock-server"]', serverState);

      // Verify initial count exceeds limit
      expect(serverState.openFiles.size).toBe(fileCount);

      // Trigger cleanup
      serverManager.cleanupMemory();

      // Check that files were limited to 100
      expect(serverState.openFiles.size).toBe(100);

      // Verify that the most recent files are kept (file50.ts to file149.ts should remain)
      expect(serverState.openFiles.has('file149.ts')).toBe(true); // Last file should be kept
      expect(serverState.openFiles.has('file50.ts')).toBe(true); // Should be at the boundary
      expect(serverState.openFiles.has('file49.ts')).toBe(false); // Should be removed
      expect(serverState.openFiles.has('file0.ts')).toBe(false); // First file should be removed

      // Clean up
      mockProcess.kill();
    });

    it('should not clean up recent diagnostics', async () => {
      // Create a mock server state
      const mockProcess = spawn('echo', ['test'], { stdio: 'pipe' });
      const serverState: ServerState = {
        process: mockProcess,
        initialized: true,
        initializationPromise: Promise.resolve(),
        openFiles: new Set(['file1.ts']),
        fileVersions: new Map(),
        startTime: Date.now(),
        config: {
          extensions: ['ts'],
          command: ['mock-server'],
        },
        diagnostics: new Map(),
        lastDiagnosticUpdate: new Map(),
        diagnosticVersions: new Map(),
        capabilities: undefined,
        buffer: '',
      };

      const mockDiagnostic: Diagnostic = {
        range: { start: { line: 0, character: 0 }, end: { line: 0, character: 10 } },
        message: 'Test diagnostic',
        severity: 1,
      };

      // Add recent diagnostic (1 minute ago)
      const oneMinuteAgo = Date.now() - 60 * 1000;
      serverState.diagnostics.set('recent-file.ts', [mockDiagnostic]);
      serverState.lastDiagnosticUpdate.set('recent-file.ts', oneMinuteAgo);
      serverState.diagnosticVersions.set('recent-file.ts', 1);

      // Manually add the server to the manager for testing
      (serverManager as any).servers.set('["mock-server"]', serverState);

      // Trigger cleanup
      serverManager.cleanupMemory();

      // Verify recent diagnostics are preserved
      expect(serverState.diagnostics.size).toBe(1);
      expect(serverState.lastDiagnosticUpdate.size).toBe(1);
      expect(serverState.diagnosticVersions.size).toBe(1);
      expect(serverState.diagnostics.has('recent-file.ts')).toBe(true);

      // Clean up
      mockProcess.kill();
    });

    it('should handle empty diagnostic maps gracefully', async () => {
      // Create a mock server state with no diagnostics
      const mockProcess = spawn('echo', ['test'], { stdio: 'pipe' });
      const serverState: ServerState = {
        process: mockProcess,
        initialized: true,
        initializationPromise: Promise.resolve(),
        openFiles: new Set(['file1.ts']),
        fileVersions: new Map(),
        startTime: Date.now(),
        config: {
          extensions: ['ts'],
          command: ['mock-server'],
        },
        diagnostics: new Map(),
        lastDiagnosticUpdate: new Map(),
        diagnosticVersions: new Map(),
        capabilities: undefined,
        buffer: '',
      };

      // Manually add the server to the manager for testing
      (serverManager as any).servers.set('["mock-server"]', serverState);

      // Trigger cleanup on empty maps
      expect(() => serverManager.cleanupMemory()).not.toThrow();

      // Verify maps remain empty
      expect(serverState.diagnostics.size).toBe(0);
      expect(serverState.lastDiagnosticUpdate.size).toBe(0);
      expect(serverState.diagnosticVersions.size).toBe(0);

      // Clean up
      mockProcess.kill();
    });
  });

  // Clean up the manager after tests
  afterEach(() => {
    serverManager.dispose();
  });
});
