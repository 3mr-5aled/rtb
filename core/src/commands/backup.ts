import type { Command } from 'commander';
import chalk from 'chalk';
import type { CliContext } from '../types/context.js';
import { MaintenanceTaskRegistry } from '../services/maintenance.js';
import { outputJson, outputError } from '../utils/output.js';

export function registerBackupCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('backup')
    .description('Full workspace configuration backup')
    .option('--json', 'Output backup results in JSON format')
    .action(async (cmdOpts: { json?: boolean }) => {
      const ctx = getContext();
      const isJson = Boolean(cmdOpts.json || ctx.isJson);

      if (!ctx.config) {
        outputError('Configuration not loaded', 'CONFIG_MISSING', isJson);
        if (process.env.VITEST) return;
        process.exit(1);
      }

      const registry = new MaintenanceTaskRegistry();
      const result = await registry.runTask('backup', {
        config: ctx.config,
        configPath: ctx.configPath,
        isJson,
      });

      if (isJson) {
        outputJson(result);
        return;
      }

      console.log('');
      console.log(chalk.cyan('═'.repeat(60)));
      console.log(`  ${chalk.bold.cyan('rtb (رتّب)')} » Configuration Backup`);
      console.log(chalk.cyan('═'.repeat(60)));
      console.log(`  ${result.success ? chalk.green('✓') : chalk.red('✗')} ${result.message}\n`);
    });
}
