import type { Command } from 'commander';
import chalk from 'chalk';
import type { CliContext } from '../types/context.js';
import { ProjectLifecycle } from '../services/lifecycle.js';
import { ConfigMissingError } from '../errors.js';
import { outputJson } from '../utils/output.js';

export function registerUnarchiveCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('unarchive [archive]')
    .description('Extract a previously archived project back into the Active root')
    .action((archiveName: string | undefined) => {
      const ctx = getContext();

      if (!archiveName) {
        console.log(`\n  ${chalk.yellow('Usage:')} rtb unarchive <archive-name.tar.gz>\n`);
        return;
      }

      if (!ctx.config) {
        throw new ConfigMissingError('Configuration not loaded');
      }

      const lifecycle = new ProjectLifecycle();
      const result = lifecycle.unarchive({
        archiveName,
        config: ctx.config,
      });

      if (ctx.isJson) {
        outputJson({
          unarchived: true,
          archive: result.archive,
          destination: result.destination,
        });
        return;
      }

      console.log(`\n  ${chalk.green('✓')} Unarchived: ${chalk.bold(archiveName)}`);
      console.log(`  Extracted to: ${chalk.cyan(result.destination)}`);
      console.log(`  Run: ${chalk.cyan('rtb list --active')}\n`);
    });
}
