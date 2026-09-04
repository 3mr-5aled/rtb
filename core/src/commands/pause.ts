import type { Command } from 'commander';
import chalk from 'chalk';
import type { CliContext } from '../types/context.js';
import { ProjectLifecycle } from '../services/lifecycle.js';
import { ConfigMissingError } from '../errors.js';
import { outputJson } from '../utils/output.js';

export { pruneDirectory } from '../services/lifecycle.js';

export function registerPauseCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('pause [name]')
    .description('Move a project from Active to Paused')
    .option('-p, --prune', 'Prune dependency and build directories before moving', false)
    .option('-f, --force', 'Bypass git uncommitted changes check', false)
    .action((name: string | undefined, options: { prune?: boolean; force?: boolean }) => {
      const ctx = getContext();

      if (!name) {
        console.log(`\n  ${chalk.yellow('Usage:')} rtb pause <project-name> [--prune] [--force]\n`);
        return;
      }

      if (!ctx.config) {
        throw new ConfigMissingError('Configuration not loaded');
      }

      const lifecycle = new ProjectLifecycle();
      const result = lifecycle.pause({
        name,
        config: ctx.config,
        prune: options.prune,
        force: options.force,
      });

      if (ctx.isJson) {
        outputJson({
          paused: true,
          name: result.name,
          from: result.from,
          to: result.to,
          pruned: result.pruned,
        });
        return;
      }

      console.log(`\n  ${chalk.green('✓')} '${chalk.bold(result.name)}' moved to Paused`);
      console.log(`  Target: ${chalk.gray(result.to)}`);
      console.log(`  Run: ${chalk.cyan(`rtb resume ${result.name}`)} to restore.\n`);
    });
}
