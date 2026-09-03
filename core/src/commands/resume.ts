import type { Command } from 'commander';
import chalk from 'chalk';
import fs from 'node:fs';
import path from 'node:path';
import { execSync } from 'node:child_process';
import type { CliContext } from '../types/context.js';
import { toKebabCase } from './new.js';
import { outputError, outputJson } from '../utils/output.js';

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
      let sourcePath = path.join(pausedRoot, kebab);

      if (!fs.existsSync(sourcePath)) {
        sourcePath = path.join(pausedRoot, name);
        if (!fs.existsSync(sourcePath)) {
          outputError(`Project '${name}' not found in Paused!`, 'NOT_FOUND', ctx.isJson);
          process.exitCode = 1;
          return;
        }
      }

      const projectName = path.basename(sourcePath);
      const targetPath = path.join(activeRoot, projectName);

      if (fs.existsSync(targetPath)) {
        outputError(`A project named '${projectName}' already exists in Active!`, 'ALREADY_EXISTS', ctx.isJson);
        process.exitCode = 1;
        return;
      }

      fs.renameSync(sourcePath, targetPath);

      let installRan = false;
      if (options.install) {
        if (fs.existsSync(path.join(targetPath, 'package.json'))) {
          console.log(`  ${chalk.gray('Running npm install...')}`);
          try {
            execSync('npm install', { cwd: targetPath, stdio: 'inherit' });
            installRan = true;
          } catch {}
        } else if (fs.existsSync(path.join(targetPath, 'requirements.txt'))) {
          console.log(`  ${chalk.gray('Running pip install...')}`);
          try {
            execSync('pip install -r requirements.txt', { cwd: targetPath, stdio: 'inherit' });
            installRan = true;
          } catch {}
        }
      }

      if (ctx.isJson) {
        outputJson({
          resumed: true,
          name: projectName,
          from: sourcePath,
          to: targetPath,
          installed: installRan,
        });
        return;
      }

      console.log(`\n  ${chalk.green('✓')} '${chalk.bold(projectName)}' moved to Active`);
      console.log(`  Run: ${chalk.cyan(`rtb goto ${projectName}`)}\n`);
    });
}
