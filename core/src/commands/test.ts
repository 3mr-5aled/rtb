import type { Command } from 'commander';
import chalk from 'chalk';
import path from 'node:path';
import fs from 'node:fs';
import type { CliContext } from '../types/context.js';
import { resolveProjectAction, executeProjectAction } from '../services/runner.js';
import { findProjectPathFuzzy } from '../navigation/fuzzy.js';
import { outputError, outputJson } from '../utils/output.js';

export function registerTestCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('test [project] [args...]')
    .description('Run project test suite')
    .allowUnknownOption(true)
    .option('--dry-run', 'Inspect test command resolution without executing', false)
    .action(async (projectName: string | undefined, extraArgs: string[] | undefined, options: { dryRun?: boolean }) => {
      const ctx = getContext();
      let targetPath = process.cwd();

      const finalExtraArgs = Array.isArray(extraArgs) ? extraArgs : [];

      if (projectName && ctx.config) {
        if (fs.existsSync(projectName)) {
          targetPath = path.resolve(projectName);
        } else {
          const matches = findProjectPathFuzzy(projectName, ctx.config);
          if (matches.length > 0) {
            targetPath = matches[0].path;
          } else {
            if (ctx.isJson) {
              outputError(`Project '${projectName}' not found.`, 'PROJECT_NOT_FOUND', true);
            } else {
              console.error(chalk.red(`\n  ✗ Project '${projectName}' not found.\n`));
            }
            process.exit(1);
            return;
          }
        }
      }

      const resolved = resolveProjectAction('test', targetPath, finalExtraArgs);
      if (!resolved) {
        if (ctx.isJson) {
          outputError(`No test configuration detected in ${targetPath}`, 'NO_TEST_TARGET', true);
        } else {
          console.log(chalk.yellow(`\n  ⚠ No test configuration detected in ${targetPath}\n`));
        }
        process.exit(1);
        return;
      }

      if (ctx.isJson) {
        outputJson({
          targetPath,
          executable: resolved.executable,
          args: resolved.args,
          dryRun: Boolean(options.dryRun),
        });
        return;
      }

      if (!ctx.isQuiet) {
        const leaf = path.basename(targetPath);
        console.log(`\n${chalk.cyan('══════════════════════════════════════════')}`);
        console.log(`  ${chalk.bold(`rtb (رتّب) » Test (${leaf})`)}`);
        console.log(`${chalk.cyan('══════════════════════════════════════════')}`);
        console.log(chalk.green(`  Running: ${resolved.executable} ${resolved.args.join(' ')}\n`));
      }

      const exitCode = await executeProjectAction(targetPath, resolved, { dryRun: options.dryRun });
      process.exit(exitCode);
    });
}
