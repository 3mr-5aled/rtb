import type { Command } from 'commander';
import chalk from 'chalk';
import type { CliContext } from '../types/context.js';
import { MaintenanceTaskRegistry } from '../services/maintenance.js';
import { outputJson, outputError } from '../utils/output.js';

export function registerGuardCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('guard')
    .description('D drive root guardrail inspection')
    .option('--json', 'Output guard results in JSON format')
    .action(async (cmdOpts: { json?: boolean }) => {
      const ctx = getContext();
      const isJson = Boolean(cmdOpts.json || ctx.isJson);

      if (!ctx.config) {
        outputError('Configuration not loaded', 'CONFIG_MISSING', isJson);
        if (process.env.VITEST) return;
        process.exit(1);
      }

      const registry = new MaintenanceTaskRegistry();
      const result = await registry.runTask('guard', {
        config: ctx.config,
        isReportOnly: true,
        isJson,
      });

      if (isJson) {
        outputJson(result);
        return;
      }

      console.log('');
      console.log(chalk.cyan('═'.repeat(60)));
      console.log(`  ${chalk.bold.cyan('rtb (ﺐﺗر)')} » Root Guardrail`);
      console.log(chalk.cyan('═'.repeat(60)));
      console.log(`  ${result.success ? chalk.green('✓') : chalk.red('✗')} ${result.message}\n`);
    });
}
