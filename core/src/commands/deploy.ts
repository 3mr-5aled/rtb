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
    .option('--from <root>', 'Specify source project root (e.g. active, staging)')
    .option('--json', 'Output deploy result in JSON format')
    .action((name: string | undefined, options: { prod?: boolean; staging?: boolean; from?: string; json?: boolean }) => {
      const ctx = getContext();
      const isJson = Boolean(options.json || ctx.isJson);

      if (!name) {
        outputError('Usage: rtb deploy <project-name> [--prod|--staging] [--from <root>]', 'USAGE_ERROR', isJson);
        process.exitCode = 1;
        return;
      }

      if (!ctx.config) {
        outputError('Configuration not loaded', 'CONFIG_MISSING', isJson);
        process.exitCode = 1;
        return;
      }

      const projectRoots = ctx.config.projectRoots || {};
      const activeRootEntry = projectRoots.active;
      const targetEnvironment = options.staging ? 'staging' : 'production';
      const targetRootEntry = projectRoots[targetEnvironment];

      if (!targetRootEntry?.path) {
        outputError(`Target root '${targetEnvironment}' not configured in rtb.config.json`, 'CONFIG_INVALID', isJson);
        process.exitCode = 1;
        return;
      }

      const kebabName = toKebabCase(name);

      const findProjectInRoot = (rootPath: string): string | null => {
        if (!fs.existsSync(rootPath)) return null;
        const candidateKebab = path.join(rootPath, kebabName);
        if (fs.existsSync(candidateKebab)) return candidateKebab;
        const candidateExact = path.join(rootPath, name);
        if (fs.existsSync(candidateExact)) return candidateExact;
        return null;
      };

      let sourcePath: string | null = null;
      let sourceLabel = 'Active';

      if (options.from) {
        const fromKey = options.from.toLowerCase();
        const fromEntry = projectRoots[fromKey];
        if (!fromEntry?.path || !fs.existsSync(fromEntry.path)) {
          outputError(`Source root '${options.from}' not configured or does not exist`, 'CONFIG_INVALID', isJson);
          process.exitCode = 1;
          return;
        }
        sourceLabel = fromEntry.label || options.from;
        sourcePath = findProjectInRoot(fromEntry.path);
        if (!sourcePath) {
          outputError(`Project '${kebabName}' not found in ${sourceLabel}!`, 'NOT_FOUND', isJson);
          process.exitCode = 1;
          return;
        }
      } else {
        // Default lookup
        if (activeRootEntry?.path) {
          sourcePath = findProjectInRoot(activeRootEntry.path);
        }

        // Project promotion workflow: if deploying to production and not found in active, check staging
        if (!sourcePath && targetEnvironment === 'production') {
          const stagingEntry = projectRoots.staging;
          if (stagingEntry?.path) {
            const stagingCandidate = findProjectInRoot(stagingEntry.path);
            if (stagingCandidate) {
              sourcePath = stagingCandidate;
              sourceLabel = stagingEntry.label || 'Staging';
            }
          }
        }

        if (!sourcePath) {
          const searchRoots = targetEnvironment === 'production' && projectRoots.staging ? 'Active or Staging' : 'Active';
          outputError(`Project '${kebabName}' not found in ${searchRoots}!`, 'NOT_FOUND', isJson);
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
        outputError(`Destination path already exists: ${destinationPath}`, 'ALREADY_EXISTS', isJson);
        process.exitCode = 1;
        return;
      }

      try {
        fs.renameSync(sourcePath, destinationPath);
      } catch (err: any) {
        if (err.code === 'EXDEV') {
          fs.cpSync(sourcePath, destinationPath, { recursive: true });
          fs.rmSync(sourcePath, { recursive: true, force: true });
        } else {
          throw err;
        }
      }

      if (isJson) {
        outputJson({
          deployed: true,
          name: projectName,
          target: targetEnvironment,
          from: sourcePath,
          to: destinationPath,
        });
        return;
      }

      console.log(`\n  ${chalk.cyan('Deploying:')} ${chalk.bold(projectName)} (${sourceLabel}) → ${chalk.green(targetEnvironment)}`);
      console.log(`  Target: ${chalk.gray(destinationPath)}`);
      console.log(`  ${chalk.green('✓')} '${chalk.bold(projectName)}' deployed to ${targetEnvironment}!\n`);
    });
}
