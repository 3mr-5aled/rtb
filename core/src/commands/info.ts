import type { Command } from 'commander';
import chalk from 'chalk';
import path from 'node:path';
import fs from 'node:fs';
import type { CliContext } from '../types/context.js';
import { resolveProjectTarget } from '../navigation/fuzzy.js';
import { inspectProject } from '../inspector/inspector.js';
import { outputError, outputJson } from '../utils/output.js';
import { ConfigMissingError, ProjectNotFoundError } from '../errors.js';

export function registerInfoCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('info [name]')
    .description('Display detailed health, dependency, and git metadata for a project')
    .action((name: string | undefined) => {
      const ctx = getContext();

      if (!name) {
        console.log(`\n  ${chalk.yellow('Usage:')} rtb info <project-name> [--json]\n`);
        return;
      }

      if (!ctx.config) {
        throw new ConfigMissingError('Configuration not loaded');
      }

      const target = resolveProjectTarget(name, ctx.config);
      if (!target || !fs.existsSync(target.targetPath)) {
        throw new ProjectNotFoundError(`Project '${name}' not found.`, 'NOT_FOUND');
      }

      const details = inspectProject(target.targetPath, target.status || 'Active');
      if (!details) {
        outputError(`Could not inspect project at ${target.targetPath}`, 'INSPECTION_FAILED', ctx.isJson);
        process.exitCode = 1;
        return;
      }

      if (ctx.isJson) {
        outputJson(details);
        return;
      }

      console.log(`\n${chalk.cyan('══════════════════════════════════════════')}`);
      console.log(`  ${chalk.bold(`rtb (ﺐﺗر) » Project Info: ${details.name}`)}`);
      console.log(`${chalk.cyan('══════════════════════════════════════════')}\n`);

      console.log(`  Name:            ${chalk.white.bold(details.name)}`);
      console.log(`  Status:          ${chalk.cyan(details.status)}`);
      console.log(`  Path:            ${chalk.gray(details.path)}`);
      console.log(`  Stack:           ${chalk.yellow(details.stack.filter((s) => s !== '-').join(', ') || 'Unknown')}`);
      console.log(`  Monorepo:        ${details.is_monorepo ? chalk.green('Yes') : 'No'}`);
      console.log(`  CI/CD:           ${details.ci_cd ? chalk.green(details.ci_cd) : chalk.gray('None')}`);
      console.log(`  Runtime Version: ${details.runtime_version ? chalk.magenta(details.runtime_version) : chalk.gray('N/A')}`);

      if (details.git) {
        console.log(`\n  ${chalk.yellow('Git Info:')}`);
        console.log(`    Branch:        ${chalk.white(details.git.branch)}`);
        console.log(`    Uncommitted:   ${details.git.uncommitted > 0 ? chalk.yellow(details.git.uncommitted) : chalk.green('0 (clean)')}`);
        console.log(`    Unpushed:      ${details.git.unpushed > 0 ? chalk.yellow(details.git.unpushed) : '0'}`);
        console.log(`    Has Remote:    ${details.git.has_remote ? chalk.green('Yes') : chalk.gray('No')}`);
        if (details.git.last_commit_msg) {
          console.log(`    Last Commit:   ${chalk.gray(`${details.git.last_commit_msg} (${details.git.last_commit_relative})`)}`);
        }
      }

      if (details.readme_preview) {
        console.log(`\n  ${chalk.yellow('README Preview:')}`);
        for (const line of details.readme_preview.split('\n')) {
          console.log(`    ${chalk.gray(line)}`);
        }
      }

      console.log('');
    });
}
