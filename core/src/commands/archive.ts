import type { Command } from 'commander';
import chalk from 'chalk';
import path from 'node:path';
import readline from 'node:readline';
import type { CliContext } from '../types/context.js';
import { ProjectLifecycle } from '../services/lifecycle.js';
import { ConfigMissingError, ProjectNotFoundError } from '../errors.js';
import { findProjectPathFuzzy } from '../navigation/fuzzy.js';
import { outputJson } from '../utils/output.js';

export function registerArchiveCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('archive [name]')
    .description('Compress a project into a tar.gz snapshot and permanently delete the source folder')
    .option('-f, --force', 'Bypass git check and confirmation prompt', false)
    .action(async (name: string | undefined, options: { force?: boolean }) => {
      const ctx = getContext();

      if (!name) {
        console.log(`\n  ${chalk.yellow('Usage:')} rtb archive <project-name> [--force]\n`);
        return;
      }

      if (!ctx.config) {
        throw new ConfigMissingError('Configuration not loaded');
      }

      const matches = findProjectPathFuzzy(name, ctx.config);
      if (matches.length === 0) {
        throw new ProjectNotFoundError(`Project '${name}' not found!`);
      }

      const target = matches[0];
      const projectPath = target.path;
      const projectName = path.basename(projectPath);

      if (!options.force && ctx.isInteractive) {
        const backupRoot = ctx.config.backupRoot || path.join(path.dirname(projectPath), 'backup');
        const snapshotDir = path.join(backupRoot, 'project-snapshots');
        console.log('');
        console.log(`  ${chalk.cyan('This will:')}`);
        console.log(`    1. Prune dependency folders (node_modules, .venv, target, etc.)`);
        console.log(`    2. Create archive in: ${chalk.gray(snapshotDir)}`);
        console.log(`    3. ${chalk.red.bold('PERMANENTLY DELETE:')} ${chalk.red(projectPath)}`);
        console.log('');

        const rl = readline.createInterface({
          input: process.stdin,
          output: process.stdout,
        });

        const answer: string = await new Promise((resolve) => {
          rl.question(chalk.yellow('  Are you sure? Type project name to confirm: '), (ans) => {
            rl.close();
            resolve(ans.trim());
          });
        });

        if (answer !== projectName) {
          console.log(chalk.gray('  Archive cancelled.\n'));
          return;
        }
      }

      const lifecycle = new ProjectLifecycle();
      const result = lifecycle.archive({
        name,
        config: ctx.config,
        force: options.force,
      });

      if (ctx.isJson) {
        outputJson({
          archived: true,
          project: result.project,
          archivePath: result.archivePath,
        });
        return;
      }

      console.log(`\n  ${chalk.green('✓')} Successfully archived '${chalk.bold(result.project)}'`);
      console.log(`  Archive: ${chalk.cyan(result.archivePath)}`);
      console.log(`  Source directory deleted.\n`);
    });
}
