import { afterAll, beforeAll, beforeEach, describe, expect, it } from 'bun:test';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { FileBackupManager } from '../helpers/file-backup-manager.js';
import { MCPTestClient, assertToolResult } from '../helpers/mcp-test-client.js';

describe('Multi-File Rename Integration Tests', () => {
  let client: MCPTestClient;
  let backupManager: FileBackupManager;

  // Test files - all files that reference AccountService
  const testFiles = [
    '/workspace/plugins/cclsp/playground/src/services/user-service.ts',
    '/workspace/plugins/cclsp/playground/src/index.ts',
    '/workspace/plugins/cclsp/playground/src/components/user-list.ts',
    '/workspace/plugins/cclsp/playground/src/components/user-form.ts',
    '/workspace/plugins/cclsp/playground/src/utils/user-helpers.ts',
    '/workspace/plugins/cclsp/playground/src/test-file.ts',
  ];

  beforeAll(async () => {
    console.log('🔍 Multi-File Rename Integration Test');
    console.log('=====================================\n');

    // Initialize backup manager
    backupManager = new FileBackupManager();

    // Create backups of all test files
    console.log('📋 Creating backups of playground files...');
    for (const filePath of testFiles) {
      backupManager.backupFile(filePath);
      console.log(`  ✓ Backed up: ${filePath}`);
    }

    // Initialize MCP client
    client = new MCPTestClient();
    await client.start();

    // Wait for LSP servers to initialize
    console.log('⏳ Waiting for LSP servers to initialize...');
    await new Promise((resolve) => setTimeout(resolve, 3000));
    console.log('✅ Setup complete\n');
  });

  afterAll(async () => {
    // Stop MCP client
    await client.stop();

    // Restore all files from backups
    console.log('\n🔄 Restoring original files...');
    const restored = backupManager.restoreAll();
    console.log(`✅ Restored ${restored} files from backups`);

    // Cleanup backup manager
    backupManager.cleanup();
  });

  beforeEach(async () => {
    // Restore files before each test to ensure clean state
    backupManager.restoreAll();
    console.log('🔄 Files restored to original state');
  });

  describe('AccountService → CustomerService Rename', () => {
    it('should preview multi-file rename with dry_run', async () => {
      console.log('🔍 Testing dry-run rename preview...');

      const result = await client.callTool('rename_symbol', {
        file_path: '/workspace/plugins/cclsp/playground/src/services/user-service.ts',
        symbol_name: 'AccountService',
        new_name: 'CustomerService',
        dry_run: true,
      });

      expect(result).toBeDefined();
      const toolResult = assertToolResult(result);
      const content = toolResult.content?.[0]?.text || '';

      console.log('📋 Dry-run result preview:');
      console.log(content);

      // Should indicate it's a dry run
      expect(content).toMatch(/DRY RUN|Would rename|preview/i);

      // Should mention the symbol being renamed
      expect(content).toMatch(/AccountService.*CustomerService/);

      // Verify files are unchanged after dry run
      for (const filePath of testFiles) {
        const fileContent = readFileSync(filePath, 'utf-8');
        expect(fileContent).toContain('AccountService');
        expect(fileContent).not.toContain('CustomerService');
      }

      console.log('✅ Dry-run preview successful - no files modified');
    });

    it('should execute multi-file rename and verify all file changes', async () => {
      console.log('🔧 Executing actual multi-file rename...');

      // Record original content for comparison
      const originalContents = new Map<string, string>();
      for (const filePath of testFiles) {
        originalContents.set(filePath, readFileSync(filePath, 'utf-8'));
      }

      // Execute the rename
      const result = await client.callTool('rename_symbol', {
        file_path: '/workspace/plugins/cclsp/playground/src/services/user-service.ts',
        symbol_name: 'AccountService',
        new_name: 'CustomerService',
        dry_run: false,
      });

      expect(result).toBeDefined();
      const toolResult = assertToolResult(result);
      const content = toolResult.content?.[0]?.text || '';

      console.log('📋 Rename execution result:');
      console.log(content);

      // Should indicate successful rename
      expect(content).toMatch(/renamed|success|applied/i);
      expect(content).toMatch(/AccountService.*CustomerService/);

      console.log('🔍 Verifying file changes...');

      // Wait a moment for file system operations to complete
      await new Promise((resolve) => setTimeout(resolve, 500));

      // Verify specific changes in each file
      const verifications = [
        {
          file: '/workspace/plugins/cclsp/playground/src/services/user-service.ts',
          expectedChanges: ['export class CustomerService {'],
          description: 'Class definition',
        },
        {
          file: '/workspace/plugins/cclsp/playground/src/index.ts',
          expectedChanges: ['export { CustomerService as UserService }'],
          description: 'Re-export with alias',
        },
        {
          file: '/workspace/plugins/cclsp/playground/src/components/user-list.ts',
          expectedChanges: [
            'import type { CustomerService }',
            'private userService: CustomerService',
          ],
          description: 'Type import and constructor parameter',
        },
        {
          file: '/workspace/plugins/cclsp/playground/src/components/user-form.ts',
          expectedChanges: ['import type { CustomerService }', 'private service: CustomerService'],
          description: 'Type import and constructor parameter',
        },
        {
          file: '/workspace/plugins/cclsp/playground/src/utils/user-helpers.ts',
          expectedChanges: [
            'import { CustomerService }',
            'return new CustomerService(db);',
            ').then((m) => m.CustomerService);',
          ],
          description: 'Regular import, constructor, and dynamic import',
        },
        {
          file: '/workspace/plugins/cclsp/playground/src/test-file.ts',
          expectedChanges: ['import { CustomerService }'],
          description: 'Regular import',
        },
      ];

      let totalExpectedChanges = 0;
      let totalFoundChanges = 0;

      for (const verification of verifications) {
        const newContent = readFileSync(verification.file, 'utf-8');
        const relativeFile = verification.file.replace(
          '/workspace/plugins/cclsp/playground/src/',
          ''
        );

        console.log(`\n📄 ${relativeFile} (${verification.description}):`);

        for (const expectedChange of verification.expectedChanges) {
          totalExpectedChanges++;
          if (newContent.includes(expectedChange)) {
            console.log(`  ✅ Found: "${expectedChange}"`);
            totalFoundChanges++;
          } else {
            console.log(`  ❌ Missing: "${expectedChange}"`);
            console.log('  📝 File content preview:');
            console.log(`     ${newContent.split('\n').slice(0, 5).join('\\n     ')}`);
          }
        }

        // Verify old name is completely gone
        if (!newContent.includes('AccountService')) {
          console.log(`  ✅ Old name 'AccountService' successfully replaced`);
        } else {
          console.log(`  ⚠️  Old name 'AccountService' still present in file`);
          // Show where it still appears
          const lines = newContent.split('\\n');
          for (let i = 0; i < lines.length; i++) {
            if (lines[i].includes('AccountService')) {
              console.log(`      Line ${i + 1}: ${lines[i].trim()}`);
            }
          }
        }
      }

      console.log(
        `\n📊 Summary: ${totalFoundChanges}/${totalExpectedChanges} expected changes found`
      );

      // Assert that all expected changes were found
      expect(totalFoundChanges).toBeGreaterThan(0);
      expect(totalFoundChanges).toBe(totalExpectedChanges);

      console.log('✅ Multi-file rename verification complete');
    }, 30000); // Extended timeout for LSP operations

    it('should handle rename of non-existent symbol gracefully', async () => {
      console.log('🔍 Testing rename of non-existent symbol...');

      const result = await client.callTool('rename_symbol', {
        file_path: '/workspace/plugins/cclsp/playground/src/services/user-service.ts',
        symbol_name: 'NonExistentService',
        new_name: 'SomeOtherService',
        dry_run: true,
      });

      expect(result).toBeDefined();
      const toolResult = assertToolResult(result);
      const content = toolResult.content?.[0]?.text || '';

      console.log('📋 Non-existent symbol result:');
      console.log(content);

      // Should indicate no symbol found or similar
      expect(content).toMatch(/No.*found|not found|No symbols/i);

      console.log('✅ Non-existent symbol handled gracefully');
    });

    it('should validate rename with same name fails appropriately', async () => {
      console.log('🔍 Testing rename with same name...');

      const result = await client.callTool('rename_symbol', {
        file_path: '/workspace/plugins/cclsp/playground/src/services/user-service.ts',
        symbol_name: 'AccountService',
        new_name: 'AccountService',
        dry_run: true,
      });

      expect(result).toBeDefined();
      const toolResult = assertToolResult(result);
      const content = toolResult.content?.[0]?.text || '';

      console.log('📋 Same name rename result:');
      console.log(content);

      // Should indicate same name issue or no changes
      expect(content).toMatch(/same|no changes|identical|already named/i);

      console.log('✅ Same name rename handled appropriately');
    });
  });

  describe('Position-Based Rename (rename_symbol_strict)', () => {
    it('should rename using exact position coordinates', async () => {
      console.log('🎯 Testing position-based rename...');

      // Use exact coordinates of "AccountService" in the class definition
      // Line 2, character 14 should be the "A" in "AccountService"
      const result = await client.callTool('rename_symbol_strict', {
        file_path: '/workspace/plugins/cclsp/playground/src/services/user-service.ts',
        line: 2,
        character: 14,
        new_name: 'OrderService',
        dry_run: true,
      });

      expect(result).toBeDefined();
      const toolResult = assertToolResult(result);
      const content = toolResult.content?.[0]?.text || '';

      console.log('📋 Position-based rename result:');
      console.log(content);

      // Should indicate successful rename preview
      expect(content).toMatch(/DRY RUN|Would rename|preview/i);
      expect(content).toMatch(/OrderService/);

      console.log('✅ Position-based rename preview successful');
    });
  });

  describe('Cross-File Reference Verification', () => {
    it('should verify find_references works across all files before rename', async () => {
      console.log('🔍 Verifying cross-file references before rename...');

      const result = await client.callTool('find_references', {
        file_path: '/workspace/plugins/cclsp/playground/src/services/user-service.ts',
        symbol_name: 'AccountService',
        include_declaration: true,
      });

      expect(result).toBeDefined();
      const toolResult = assertToolResult(result);
      const content = toolResult.content?.[0]?.text || '';

      console.log('📋 References found:');
      console.log(content);

      // Should find references in multiple files
      expect(content).not.toMatch(/No.*found/i);
      expect(content).toMatch(/AccountService/);

      // Count expected files mentioned (should be at least 3-4 files)
      const fileMatches = content.match(/\.ts/g) || [];
      expect(fileMatches.length).toBeGreaterThan(2);

      console.log(`✅ Found references in ${fileMatches.length} files`);
    });
  });
});
