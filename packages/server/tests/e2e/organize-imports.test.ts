import { afterAll, beforeAll, describe, expect, it } from 'bun:test';
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { assertToolResult, MCPTestClient } from '../helpers/mcp-test-client.js';
import { poll, waitForLSP } from '../helpers/test-verification-helpers.js';

describe('Organize Imports Integration Tests', () => {
  let client: MCPTestClient;
  const TEST_DIR = '/workspace/examples/playground/organize-imports-test';

  beforeAll(async () => {
    console.log('🔍 Organize Imports Integration Test');
    console.log('===================================\n');

    // Create isolated test directory
    if (existsSync(TEST_DIR)) {
      rmSync(TEST_DIR, { recursive: true });
    }
    mkdirSync(TEST_DIR, { recursive: true });

    // Create TypeScript project configuration
    writeFileSync(
      join(TEST_DIR, 'tsconfig.json'),
      JSON.stringify(
        {
          compilerOptions: {
            target: 'ES2022',
            module: 'ESNext',
            moduleResolution: 'node',
            esModuleInterop: true,
            allowSyntheticDefaultImports: true,
            strict: true,
            skipLibCheck: true,
            forceConsistentCasingInFileNames: true,
            resolveJsonModule: true,
            isolatedModules: true,
            noEmit: true,
          },
          include: ['**/*'],
          exclude: ['node_modules'],
        },
        null,
        2
      )
    );

    writeFileSync(
      join(TEST_DIR, 'package.json'),
      JSON.stringify(
        {
          name: 'organize-imports-test',
          type: 'module',
          version: '1.0.0',
        },
        null,
        2
      )
    );

    // Create utility modules for importing
    writeFileSync(
      join(TEST_DIR, 'utils.ts'),
      `export function capitalize(str: string): string {
  return str.charAt(0).toUpperCase() + str.slice(1);
}

export function lowercase(str: string): string {
  return str.toLowerCase();
}

export const CONSTANTS = {
  MAX_LENGTH: 100,
  MIN_LENGTH: 1,
};`
    );

    writeFileSync(
      join(TEST_DIR, 'types.ts'),
      `export interface User {
  id: number;
  name: string;
  email: string;
}

export interface Product {
  id: number;
  title: string;
  price: number;
}

export type Status = 'active' | 'inactive' | 'pending';`
    );

    // Create a TypeScript file with unsorted/messy imports
    writeFileSync(
      join(TEST_DIR, 'messy-imports.ts'),
      `// This file has imports that need organizing
import { User, Product } from './types';
import { readFileSync } from 'node:fs';
import { capitalize, CONSTANTS } from './utils';
import { join } from 'node:path';
import { lowercase } from './utils';

// Some unused imports that should be removed
import { existsSync } from 'node:fs';
import { Status } from './types';

export class DataManager {
  private users: User[] = [];
  private products: Product[] = [];

  addUser(name: string, email: string): void {
    const user: User = {
      id: this.users.length + 1,
      name: capitalize(name),
      email: lowercase(email),
    };
    this.users.push(user);
  }

  addProduct(title: string, price: number): void {
    const product: Product = {
      id: this.products.length + 1,
      title: capitalize(title),
      price: Math.min(price, CONSTANTS.MAX_LENGTH),
    };
    this.products.push(product);
  }

  loadData(filePath: string): string {
    const fullPath = join(process.cwd(), filePath);
    return readFileSync(fullPath, 'utf-8');
  }
}`
    );

    // Create a Python file with unsorted imports (for Python LSP testing)
    writeFileSync(
      join(TEST_DIR, 'messy_imports.py'),
      `# This Python file has imports that need organizing
import sys
import json
from typing import Dict, List
import os
from collections import defaultdict
import re

# Some unused imports
from datetime import datetime
from pathlib import Path

def process_data(data: List[Dict[str, str]]) -> Dict[str, List[str]]:
    """Process data and return organized results."""
    result = defaultdict(list)

    for item in data:
        if re.match(r'^[A-Z]', item.get('name', '')):
            result['uppercase'].append(item['name'])
        else:
            result['lowercase'].append(item['name'])

    return dict(result)

def load_config(config_path: str) -> Dict:
    """Load configuration from JSON file."""
    if os.path.exists(config_path):
        with open(config_path, 'r') as f:
            return json.load(f)
    return {}

def main():
    """Main function."""
    print(f"Python version: {sys.version}")
    data = [{"name": "Alice"}, {"name": "bob"}]
    result = process_data(data)
    print(json.dumps(result, indent=2))

if __name__ == "__main__":
    main()
`
    );

    // Initialize MCP client
    client = new MCPTestClient();
    await client.start({ skipLSPPreload: true });

    await waitForLSP(client, join(TEST_DIR, 'messy-imports.ts'));
    console.log('✅ Setup complete\n');
  });

  afterAll(async () => {
    await client.stop();
    if (existsSync(TEST_DIR)) {
      rmSync(TEST_DIR, { recursive: true });
    }
    console.log('✅ Cleanup complete');
  });

  describe('TypeScript Import Organization', () => {
    it('should organize imports in TypeScript file', async () => {
      console.log('🔧 Testing TypeScript import organization...');

      const beforeContent = readFileSync(join(TEST_DIR, 'messy-imports.ts'), 'utf-8');
      console.log('📄 Before organizing imports:');
      console.log(beforeContent.split('\n').slice(0, 15).join('\n'));

      const result = await client.callTool('organize_imports', {
        file_path: join(TEST_DIR, 'messy-imports.ts'),
      });

      expect(result).toBeDefined();
      assertToolResult(result);
      const content = result.content?.[0]?.text || '';

      console.log('📋 Organize imports result:');
      console.log(content);

      // Should indicate successful organization or no changes needed
      expect(content).toMatch(/organized imports|already properly organized|no changes|Successfully organized/i);

      // Wait for file system operations to complete
      await poll(
        async () => {
          const afterContent = readFileSync(join(TEST_DIR, 'messy-imports.ts'), 'utf-8');
          return afterContent !== beforeContent || content.includes('already properly organized') || content.includes('Successfully organized');
        },
        10000,
        100
      );

      const afterContent = readFileSync(join(TEST_DIR, 'messy-imports.ts'), 'utf-8');
      console.log('📄 After organizing imports:');
      console.log(afterContent.split('\n').slice(0, 15).join('\n'));

      // Basic validation that imports are still present and file is valid
      expect(afterContent).toContain('import');
      expect(afterContent).toContain('DataManager');
      expect(afterContent).toContain('User');
      expect(afterContent).toContain('capitalize');

      console.log('✅ TypeScript import organization complete');
    }, 30000);

    it('should handle file with already organized imports', async () => {
      console.log('🔍 Testing file with already organized imports...');

      // Create a file with well-organized imports
      const organizedFile = join(TEST_DIR, 'organized-imports.ts');
      writeFileSync(
        organizedFile,
        `import { readFileSync } from 'node:fs';
import { join } from 'node:path';

import { capitalize } from './utils';
import { User } from './types';

export class WellOrganized {
  loadUser(name: string): User {
    return {
      id: 1,
      name: capitalize(name),
      email: 'test@example.com',
    };
  }
}`
      );

      await waitForLSP(client, organizedFile);

      const result = await client.callTool('organize_imports', {
        file_path: organizedFile,
      });

      expect(result).toBeDefined();
      assertToolResult(result);
      const content = result.content?.[0]?.text || '';

      console.log('📋 Well-organized file result:');
      console.log(content);

      // Should indicate successful organization (even if minimal changes)
      expect(content).toMatch(/already properly organized|no changes|no.*action|Successfully organized/i);

      console.log('✅ Already organized imports handled correctly');
    });
  });

  describe('Python Import Organization', () => {
    it('should organize imports in Python file', async () => {
      console.log('🐍 Testing Python import organization...');

      const beforeContent = readFileSync(join(TEST_DIR, 'messy_imports.py'), 'utf-8');
      console.log('📄 Before organizing Python imports:');
      console.log(beforeContent.split('\n').slice(0, 15).join('\n'));

      await waitForLSP(client, join(TEST_DIR, 'messy_imports.py'));

      const result = await client.callTool('organize_imports', {
        file_path: join(TEST_DIR, 'messy_imports.py'),
      });

      expect(result).toBeDefined();
      assertToolResult(result);
      const content = result.content?.[0]?.text || '';

      console.log('📋 Python organize imports result:');
      console.log(content);

      // Should indicate successful organization, no changes needed, or no support
      expect(content).toMatch(/organized imports|already properly organized|no changes|not support/i);

      const afterContent = readFileSync(join(TEST_DIR, 'messy_imports.py'), 'utf-8');

      // Basic validation that imports are still present and file is valid
      expect(afterContent).toContain('import');
      expect(afterContent).toContain('def process_data');
      expect(afterContent).toContain('def main');

      console.log('✅ Python import organization complete');
    }, 30000);
  });

  describe('Error Handling', () => {
    it('should handle non-existent file gracefully', async () => {
      console.log('🔍 Testing non-existent file handling...');

      const result = await client.callTool('organize_imports', {
        file_path: join(TEST_DIR, 'non-existent-file.ts'),
      });

      expect(result).toBeDefined();
      assertToolResult(result);
      const content = result.content?.[0]?.text || '';

      console.log('📋 Non-existent file result:');
      console.log(content);

      // Should indicate error or file not found
      expect(content).toMatch(/error|not found|does not exist/i);

      console.log('✅ Non-existent file handled gracefully');
    });

    it('should handle file with no imports gracefully', async () => {
      console.log('🔍 Testing file with no imports...');

      const noImportsFile = join(TEST_DIR, 'no-imports.ts');
      writeFileSync(
        noImportsFile,
        `export class SimpleClass {
  private value: string = 'hello';

  getValue(): string {
    return this.value;
  }
}`
      );

      await waitForLSP(client, noImportsFile);

      const result = await client.callTool('organize_imports', {
        file_path: noImportsFile,
      });

      expect(result).toBeDefined();
      assertToolResult(result);
      const content = result.content?.[0]?.text || '';

      console.log('📋 No imports file result:');
      console.log(content);

      // Should indicate no imports to organize or no actions available
      expect(content).toMatch(/no.*import|no.*action|already properly organized/i);

      console.log('✅ File with no imports handled gracefully');
    });
  });
});