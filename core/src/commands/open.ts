import type { Command } from 'commander';
import chalk from 'chalk';
import path from 'node:path';
import fs from 'node:fs';
import type { CliContext } from '../types/context.js';
import { findProjectPathFuzzy } from '../navigation/fuzzy.js';
import { openPath } from '../utils/opener.js';
import { outputError } from '../utils/output.js';

export function registerOpenCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('open [project]')
    .description('Open project folder in File Explorer or OS file manager')
    .action((projectName: string | undefined) => {
      const ctx = getContext();
      let targetPath = process.cwd();
      let targetName = path.basename(targetPath);

      if (projectName) {
        if (fs.existsSync(projectName)) {
          targetPath = path.resolve(projectName);
          targetName = path.basename(targetPath);
        } else if (ctx.config) {
          const matches = findProjectPathFuzzy(projectName, ctx.config);
          if (matches.length > 0) {
            targetPath = matches[0].path;
            targetName = matches[0].name;
          } else {
            outputError(`Project or path '${projectName}' not found.`, 'NOT_FOUND', ctx.isJson);
            if (process.env.VITEST) return;
            process.exit(1);
          }
        } else {
          outputError(`Project or path '${projectName}' not found.`, 'NOT_FOUND', ctx.isJson);
          if (process.env.VITEST) return;
          process.exit(1);
        }
      }

      if (!ctx.isQuiet && !ctx.isJson) {
        console.log(`Opening project '${chalk.green(targetName)}' in file explorer...`);
        console.log(`  Path: ${chalk.gray(targetPath)}`);
      }

      try {
        openPath(targetPath);
      } catch (err: unknown) {
        outputError(
          `Failed to open '${targetPath}': ${err instanceof Error ? err.message : String(err)}`,
          'OPEN_FAILED',
          ctx.isJson
        );
      }
    });
}
