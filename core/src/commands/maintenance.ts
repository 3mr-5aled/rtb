import type { Command } from 'commander';
import chalk from 'chalk';
import type { CliContext } from '../types/context.js';
import { MaintenanceTaskRegistry } from '../services/maintenance.js';
import { outputJson, outputError } from '../utils/output.js';

export function registerMaintenanceCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('maintenance')
    .description('Run all workspace maintenance tasks (backup, env, guard, clean)')
    .option('--full', 'Run comprehensive full maintenance pass', false)
    .option('--json', 'Output maintenance results in JSON format')
    .action(async (cmdOpts: { full?: boolean; json?: boolean }) => {
      const ctx = getContext();
      const isJson = Boolean(cmdOpts.json || ctx.isJson);

      if (!ctx.config) {
        outputError('Configuration not loaded', 'CONFIG_MISSING', isJson);
        if (process.env.VITEST) return;
        process.exit(1);
      }

      const registry = new MaintenanceTaskRegistry();
      const results = await registry.runAll({
        config: ctx.config,
        configPath: ctx.configPath,
        isFull: Boolean(cmdOpts.full),
        isJson,
      });

      if (isJson) {
        outputJson({ success: results.every((r) => r.success), results });
        return;
      }

      console.log('');
      console.log(chalk.cyan('═'.repeat(60)));
      console.log(`  ${chalk.bold.cyan('rtb (رتّب)')} » Workspace Maintenance`);
      console.log(chalk.cyan('═'.repeat(60)));
      console.log('');

      for (const res of results) {
        const icon = res.success ? chalk.green('✓') : chalk.red('✗');
        console.log(`  ${icon} [${chalk.bold(res.task)}] ${res.message}`);
      }
      console.log('');
    });
}
