import type { Command } from 'commander';
import chalk from 'chalk';
import type { CliContext } from '../types/context.js';
import { ProjectLifecycle } from '../services/lifecycle.js';
import { ConfigMissingError } from '../errors.js';
import { outputJson } from '../utils/output.js';

export function registerResumeCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('resume [name]')
    .description('Move a project from Paused back to Active')
    .option('-i, --install', 'Run package manager install after restoring project', false)
    .action((name: string | undefined, options: { install?: boolean }) => {
      const ctx = getContext();

      if (!name) {
        console.log(`\n  ${chalk.yellow('Usage:')} rtb resume <project-name> [--install]\n`);
        return;
      }

      if (!ctx.config) {
        throw new ConfigMissingError('Configuration not loaded');
      }

      const lifecycle = new ProjectLifecycle();
      const result = lifecycle.resume({
        name,
        config: ctx.config,
        install: options.install,
      });

      if (ctx.isJson) {
        outputJson({
          resumed: true,
          name: result.name,
          from: result.from,
          to: result.to,
          installed: result.installed,
        });
        return;
      }

      console.log(`\n  ${chalk.green('✓')} '${chalk.bold(result.name)}' moved to Active`);
      console.log(`  Run: ${chalk.cyan(`rtb goto ${result.name}`)}\n`);
    });
}
