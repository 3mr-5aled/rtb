import type { Command } from 'commander';
import chalk from 'chalk';
import fs from 'node:fs';
import path from 'node:path';
import { execSync } from 'node:child_process';
import readline from 'node:readline';
import type { CliContext } from '../types/context.js';
import { findProjectPathFuzzy } from '../navigation/fuzzy.js';
import { isGitClean } from '../utils/git.js';
import { pruneDirectory } from './pause.js';
import { outputError, outputJson } from '../utils/output.js';

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
        outputError('Configuration not loaded', 'CONFIG_MISSING', ctx.isJson);
        process.exitCode = 1;
        return;
      }

      const matches = findProjectPathFuzzy(name, ctx.config);
      if (matches.length === 0) {
        outputError(`Project '${name}' not found!`, 'NOT_FOUND', ctx.isJson);
        process.exitCode = 1;
        return;
      }

      const target = matches[0];
      const projectPath = target.path;
      const projectName = path.basename(projectPath);

      if (!options.force && !isGitClean(projectPath)) {
        outputError(
          `Project has uncommitted git changes! Commit or stash first, or pass --force.`,
          'DIRTY_GIT',
          ctx.isJson
        );
        process.exitCode = 1;
        return;
      }

      const backupRoot = ctx.config.backupRoot || path.join(path.dirname(projectPath), 'backup');
      const snapshotDir = path.join(backupRoot, 'project-snapshots');
      if (!fs.existsSync(snapshotDir)) {
        fs.mkdirSync(snapshotDir, { recursive: true });
      }

      const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19);
      const archiveFilename = `${projectName}-${timestamp}.tar.gz`;
      const archivePath = path.join(snapshotDir, archiveFilename);

      if (!options.force && ctx.isInteractive) {
        console.log('');
        console.log(`  ${chalk.cyan('This will:')}`);
        console.log(`    1. Prune dependency folders (node_modules, .venv, target, etc.)`);
        console.log(`    2. Create archive: ${chalk.gray(archivePath)}`);
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

      // 1. Prune
      pruneDirectory(projectPath, ctx.config.cleanDeps?.targets);

      // 2. Create tar.gz archive
      try {
        const parentDir = path.dirname(projectPath);
        execSync(`tar -czf "${archivePath}" "${projectName}"`, {
          cwd: parentDir,
          stdio: ['ignore', 'pipe', 'ignore'],
        });
      } catch (err: any) {
        outputError(`Failed to create archive: ${err.message}`, 'TAR_FAILED', ctx.isJson);
        process.exitCode = 1;
        return;
      }

      // 3. Delete directory
      fs.rmSync(projectPath, { recursive: true, force: true });

      if (ctx.isJson) {
        outputJson({
          archived: true,
          project: projectName,
          archivePath,
        });
        return;
      }

      console.log(`\n  ${chalk.green('✓')} Successfully archived '${chalk.bold(projectName)}'`);
      console.log(`  Archive: ${chalk.cyan(archivePath)}`);
      console.log(`  Source directory deleted.\n`);
    });
}
