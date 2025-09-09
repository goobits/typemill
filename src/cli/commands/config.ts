#!/usr/bin/env node

import { spawn } from 'node:child_process';
import * as DirectoryUtils from '../directory-utils.js';

interface ConfigOptions {
  show?: boolean;
  edit?: boolean;
}

/**
 * Config command - direct access to configuration
 * Bob's simple approach
 */
export async function configCommand(options: ConfigOptions = {}): Promise<void> {
  // Auto-migrate if needed
  DirectoryUtils.migrateOldConfig();

  const configPath = DirectoryUtils.getConfigPath();

  // Default: just show the config path
  if (!options.show && !options.edit) {
    console.log(`Config: ${configPath}`);
    return;
  }

  // --show: print config contents
  if (options.show) {
    try {
      const config = DirectoryUtils.readConfig();
      if (config) {
        console.log(JSON.stringify(config, null, 2));
      } else {
        console.log('No configuration found');
        console.log('Run: codebuddy init');
      }
    } catch (error) {
      console.error(`Error reading config: ${error}`);
      return;
    }
    return;
  }

  // --edit: open in $EDITOR
  if (options.edit) {
    const editor = process.env.EDITOR || process.env.VISUAL || 'vi';

    // Ensure config exists
    const config = DirectoryUtils.readConfig();
    if (!config) {
      console.log('No configuration found');
      console.log('Run: codebuddy init');
      return;
    }

    try {
      const proc = spawn(editor, [configPath], {
        stdio: 'inherit',
      });

      proc.on('exit', (code) => {
        if (code === 0) {
          console.log('Configuration updated');
        } else {
          console.error('Editor exited with error');
        }
        process.exit(code || 0);
      });

      proc.on('error', (error) => {
        console.error(`Failed to open editor: ${error.message}`);
        console.error('Set EDITOR environment variable or use --show to view config');
        process.exit(1);
      });
    } catch (error) {
      console.error(`Failed to open editor: ${error}`);
      process.exit(1);
    }
  }
}
