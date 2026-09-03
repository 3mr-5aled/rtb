import type { Command } from 'commander';
import chalk from 'chalk';
import fs from 'node:fs';
import path from 'node:path';
import type { CliContext } from '../types/context.js';
import { isGitClean } from '../utils/git.js';
import { toKebabCase } from './new.js';
import { outputError, outputJson } from '../utils/output.js';

export function pruneDirectory(dir: string, targets: string[] = ['node_modules', '.venv', '.next', '__pycache__', 'dist', 'build', 'target']): void {
  for (const target of targets) {
    const targetPath = path.join(dir, target);
    if (fs.existsSync(targetPath)) {
      try {
        fs.rmSync(targetPath, { recursive: true, force: true });
      } catch {}
    }
  }
}

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
        outputError('Configuration not loaded', 'CONFIG_MISSING', ctx.isJson);
        process.exitCode = 1;
        return;
      }

      const activeRoot = ctx.config.projectRoots.active?.path;
      const pausedRoot = ctx.config.projectRoots.paused?.path;

      if (!activeRoot || !pausedRoot) {
        outputError('Active or Paused roots not defined in config', 'CONFIG_INVALID', ctx.isJson);
        process.exitCode = 1;
        return;
      }

      const kebab = toKebabCase(name);
      let sourcePath = path.join(activeRoot, kebab);

      if (!fs.existsSync(sourcePath)) {
        // Fallback: check exact name
        sourcePath = path.join(activeRoot, name);
        if (!fs.existsSync(sourcePath)) {
          outputError(`Project '${name}' not found in Active!`, 'NOT_FOUND', ctx.isJson);
          process.exitCode = 1;
          return;
        }
      }

      const projectName = path.basename(sourcePath);
      const targetPath = path.join(pausedRoot, projectName);

      if (!options.force && !isGitClean(sourcePath)) {
        outputError(
          `Project has uncommitted git changes! Commit/stash first, or pass --force.`,
          'DIRTY_GIT',
          ctx.isJson
        );
        process.exitCode = 1;
        return;
      }

      if (!fs.existsSync(pausedRoot)) {
        fs.mkdirSync(pausedRoot, { recursive: true });
      }

      if (options.prune) {
        pruneDirectory(sourcePath, ctx.config.cleanDeps?.targets);
      }

      fs.renameSync(sourcePath, targetPath);

      if (ctx.isJson) {
        outputJson({
          paused: true,
          name: projectName,
          from: sourcePath,
          to: targetPath,
          pruned: Boolean(options.prune),
        });
        return;
      }

      console.log(`\n  ${chalk.green('✓')} '${chalk.bold(projectName)}' moved to Paused`);
      console.log(`  Target: ${chalk.gray(targetPath)}`);
      console.log(`  Run: ${chalk.cyan(`rtb resume ${projectName}`)} to restore.\n`);
    });
}
