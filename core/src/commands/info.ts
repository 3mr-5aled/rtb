import type { Command } from 'commander';
import chalk from 'chalk';
import path from 'node:path';
import fs from 'node:fs';
import type { CliContext } from '../types/context.js';
import { findProjectPathFuzzy } from '../navigation/fuzzy.js';
import { inspectProject } from '../inspector/inspector.js';
import { outputError, outputJson } from '../utils/output.js';

export function registerInfoCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('info [name]')
    .description('Display detailed metadata, stack, git status, and inspection for a project')
    .action((name: string | undefined) => {
      const ctx = getContext();

      if (!name) {
        console.log(`\n  ${chalk.yellow('Usage:')} rtb info <project-name> [--json]\n`);
        return;
      }

      if (!ctx.config) {
        outputError('Configuration not loaded', 'CONFIG_MISSING', ctx.isJson);
        process.exitCode = 1;
        return;
      }

      const matches = findProjectPathFuzzy(name, ctx.config);
      let targetPath: string | null = null;
      let targetStatus = 'Active';

      if (matches.length > 0) {
        targetPath = matches[0].path;
        targetStatus = matches[0].status;
      } else if (fs.existsSync(name)) {
        targetPath = path.resolve(name);
      }

      if (!targetPath || !fs.existsSync(targetPath)) {
        outputError(`Project '${name}' not found.`, 'NOT_FOUND', ctx.isJson);
        process.exitCode = 1;
        return;
      }

      const details = inspectProject(targetPath, targetStatus);
      if (!details) {
        outputError(`Could not inspect project at ${targetPath}`, 'INSPECTION_FAILED', ctx.isJson);
        process.exitCode = 1;
        return;
      }

      if (ctx.isJson) {
        outputJson(details);
        return;
      }

      console.log(`\n${chalk.cyan('══════════════════════════════════════════')}`);
      console.log(`  ${chalk.bold(`rtb (رتّب) » Project Info: ${details.name}`)}`);
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
