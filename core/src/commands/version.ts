import type { Command } from 'commander';
import chalk from 'chalk';
import type { CliContext } from '../types/context.js';
import { outputJson } from '../utils/output.js';

import { existsSync, readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

export function resolveVersion(): string {
  try {
    const currentDir = dirname(fileURLToPath(import.meta.url));
    const candidates = [
      join(currentDir, 'VERSION'),
      join(currentDir, '..', 'VERSION'),
      join(currentDir, '..', '..', 'VERSION'),
      join(currentDir, '..', '..', '..', 'VERSION'),
      join(process.cwd(), 'VERSION'),
    ];
    for (const file of candidates) {
      if (existsSync(file)) {
        const content = readFileSync(file, 'utf-8').trim();
        if (content) return content.replace(/^v/, '');
      }
    }
  } catch {}
  return '0.8.4';
}

export const RTB_VERSION = resolveVersion();

export function registerVersionCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('version')
    .description('Display RTB version information')
    .action(() => {
      const ctx = getContext();
      if (ctx.isJson) {
        outputJson({
          name: 'rtb',
          version: RTB_VERSION,
          engine: 'node',
          platform: process.platform,
          arch: process.arch,
          nodeVersion: process.version,
        });
      } else {
        console.log(`RTB (${chalk.cyan('رتّب')}) CLI ${chalk.green(`v${RTB_VERSION}`)} [Node]`);
      }
    });
}
