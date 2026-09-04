import type { Command } from 'commander';
import chalk from 'chalk';
import type { CliContext } from '../types/context.js';
import { MaintenanceTaskRegistry } from '../services/maintenance.js';
import { outputJson, outputError } from '../utils/output.js';

export function registerMaintenanceCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('maintenance [task]')
    .description('Run all workspace maintenance tasks (backup, env, guard) or a specific task')
    .option('--full', 'Run comprehensive full maintenance pass', false)
    .option('--json', 'Output maintenance results in JSON format')
    .action(async (taskName: string | undefined, cmdOpts: { full?: boolean; json?: boolean }) => {
      const ctx = getContext();
      const isJson = Boolean(cmdOpts.json || ctx.isJson);

      if (!ctx.config) {
        outputError('Configuration not loaded', 'CONFIG_MISSING', isJson);
        if (process.env.VITEST) return;
        process.exit(1);
      }

      const registry = new MaintenanceTaskRegistry();

      if (taskName) {
        const result = await registry.runTask(taskName, {
          config: ctx.config,
          configPath: ctx.configPath,
          isFull: Boolean(cmdOpts.full),
          isJson,
        });

        if (isJson) {
          outputJson(result);
          return;
        }

        const icon = result.success ? chalk.green('✓') : chalk.red('✗');
        console.log(`\n  ${icon} [${chalk.bold(result.task)}] ${result.message}\n`);
        return;
      }

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
      console.log(`  ${chalk.bold.cyan('rtb (ﺐﺗر)')} » Workspace Maintenance`);
      console.log(chalk.cyan('═'.repeat(60)));
      console.log('');

      for (const res of results) {
        const icon = res.success ? chalk.green('✓') : chalk.red('✗');
        console.log(`  ${icon} [${chalk.bold(res.task)}] ${res.message}`);
      }
      console.log('');
    });
}
