import type { Command } from 'commander';
import chalk from 'chalk';
import type { CliContext } from '../types/context.js';
import { outputJson } from '../utils/output.js';

export const RTB_VERSION = '0.5.0';

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
