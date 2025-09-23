#!/usr/bin/env node

/**
 * CLI tool to run dead code analysis using MCP tools
 * This demonstrates Phase 3 capabilities
 */

import { writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { findDeadCode, generateDeadCodeReport } from './dead-code-detector.js';

async function main() {
  console.log('🚀 Starting Dead Code Analysis...\n');

  try {
    // Run the analysis
    const { deadCode, stats } = await findDeadCode();

    // Generate report
    const report = generateDeadCodeReport(deadCode, stats);

    // Output to console
    console.log(report);

    // Save to file
    const reportPath = join(process.cwd(), 'dead-code-report.md');
    writeFileSync(reportPath, report);

    console.log(`\n📝 Report saved to: ${reportPath}`);

    // Exit with appropriate code
    process.exit(deadCode.length > 0 ? 1 : 0);
  } catch (error) {
    console.error('❌ Analysis failed:', error);
    process.exit(1);
  }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main();
}
