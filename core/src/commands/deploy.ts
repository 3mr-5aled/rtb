import type { Command } from 'commander';
import chalk from 'chalk';
import fs from 'node:fs';
import path from 'node:path';
import type { CliContext } from '../types/context.js';
import { toKebabCase } from './new.js';
import { outputError, outputJson } from '../utils/output.js';

export function registerDeployCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('deploy [name]')
    .description('Deploy a project from Active to Production or Staging')
    .option('--prod', 'Deploy to Production (default)', true)
    .option('--staging', 'Deploy to Staging', false)
    .action((name: string | undefined, options: { prod?: boolean; staging?: boolean }) => {
      const ctx = getContext();

      if (!name) {
        console.log(`\n  ${chalk.yellow('Usage:')} rtb deploy <project-name> [--prod|--staging]\n`);
        return;
      }

      if (!ctx.config) {
        outputError('Configuration not loaded', 'CONFIG_MISSING', ctx.isJson);
        process.exitCode = 1;
        return;
      }

      const activeRoot = ctx.config.projectRoots?.active?.path;
      if (!activeRoot || !fs.existsSync(activeRoot)) {
        outputError('Active project root not configured or does not exist', 'CONFIG_INVALID', ctx.isJson);
        process.exitCode = 1;
        return;
      }

      const targetEnvironment = options.staging ? 'staging' : 'production';
      const targetRootEntry = ctx.config.projectRoots?.[targetEnvironment];

      if (!targetRootEntry?.path) {
        outputError(`Target root '${targetEnvironment}' not configured in rtb.config.json`, 'CONFIG_INVALID', ctx.isJson);
        process.exitCode = 1;
        return;
      }

      const kebabName = toKebabCase(name);
      let sourcePath = path.join(activeRoot, kebabName);

      if (!fs.existsSync(sourcePath)) {
        // Fallback: check exact name
        sourcePath = path.join(activeRoot, name);
        if (!fs.existsSync(sourcePath)) {
          outputError(`Project '${kebabName}' not found in Active!`, 'NOT_FOUND', ctx.isJson);
          process.exitCode = 1;
          return;
        }
      }

      const projectName = path.basename(sourcePath);
      const deployRoot = targetRootEntry.path;
      if (!fs.existsSync(deployRoot)) {
        fs.mkdirSync(deployRoot, { recursive: true });
      }

      const destinationPath = path.join(deployRoot, projectName);

      if (fs.existsSync(destinationPath)) {
        outputError(`Destination path already exists: ${destinationPath}`, 'ALREADY_EXISTS', ctx.isJson);
        process.exitCode = 1;
        return;
      }

      fs.renameSync(sourcePath, destinationPath);

      if (ctx.isJson) {
        outputJson({
          deployed: true,
          name: projectName,
          target: targetEnvironment,
          from: sourcePath,
          to: destinationPath,
        });
        return;
      }

      console.log(`\n  ${chalk.cyan('Deploying:')} ${chalk.bold(projectName)} → ${chalk.green(targetEnvironment)}`);
      console.log(`  Target: ${chalk.gray(destinationPath)}`);
      console.log(`  ${chalk.green('✓')} '${chalk.bold(projectName)}' deployed to ${targetEnvironment}!\n`);
    });
}
