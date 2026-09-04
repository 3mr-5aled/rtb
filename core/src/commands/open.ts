import type { Command } from 'commander';
import chalk from 'chalk';
import type { CliContext } from '../types/context.js';
import { resolveProjectTarget } from '../navigation/fuzzy.js';
import { openPath } from '../utils/opener.js';
import { ProjectNotFoundError, RtbError } from '../errors.js';

export function registerOpenCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('open [project]')
    .description('Open project folder in File Explorer or OS file manager')
    .action((projectName: string | undefined) => {
      const ctx = getContext();
      const target = resolveProjectTarget(projectName, ctx.config);

      if (!target) {
        throw new ProjectNotFoundError(`Project or path '${projectName}' not found.`, 'NOT_FOUND');
      }

      const { targetPath, targetName } = target;

      if (!ctx.isQuiet && !ctx.isJson) {
        console.log(`Opening project '${chalk.green(targetName)}' in file explorer...`);
        console.log(`  Path: ${chalk.gray(targetPath)}`);
      }

      try {
        openPath(targetPath);
      } catch (err: unknown) {
        throw new RtbError(
          `Failed to open '${targetPath}': ${err instanceof Error ? err.message : String(err)}`,
          'OPEN_FAILED'
        );
      }
    });
}
