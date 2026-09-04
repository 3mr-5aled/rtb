import type { Command } from 'commander';
import chalk from 'chalk';
import path from 'node:path';
import fs from 'node:fs';
import type { CliContext } from '../types/context.js';
import { scanGitHealth } from '../inspector/health.js';
import { outputJson } from '../utils/output.js';

export function registerHealthCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('health')
    .description('Git repository health scan (uncommitted, unpushed, stale, remote status)')
    .option('--json', 'Output health scan results in JSON format')
    .option('-v, --verbose', 'Show detailed verbose scan information')
    .action((cmdOpts: { json?: boolean; verbose?: boolean }) => {
      const ctx = getContext();
      const isJson = Boolean(cmdOpts.json || ctx.isJson);

      const scanRoots: string[] = [];

      if (ctx.config?.gitHealth?.scanRoots && ctx.config.gitHealth.scanRoots.length > 0) {
        for (const root of ctx.config.gitHealth.scanRoots) {
          if (fs.existsSync(root)) {
            scanRoots.push(root);
          }
        }
      } else if (ctx.config?.projectRoots) {
        for (const entry of Object.values(ctx.config.projectRoots)) {
          if (entry.path && fs.existsSync(entry.path)) {
            scanRoots.push(entry.path);
          }
        }
      }

      if (scanRoots.length === 0) {
        scanRoots.push(process.cwd());
      }

      if (!isJson) {
        console.log('');
        console.log(chalk.cyan('═'.repeat(60)));
        console.log(`  ${chalk.bold.cyan('rtb (ﺐﺗر)')} » Git Repository Health`);
        console.log(chalk.cyan('═'.repeat(60)));
      }

      const staleThreshold = ctx.config?.staleThresholdDays || 30;
      const report = scanGitHealth(scanRoots, staleThreshold, (repo) => {
        if (isJson) return;

        const branchPart = repo.branch ? chalk.magenta(`[${repo.branch}]`) : '';

        if (repo.issues.length > 0) {
          console.log(`\n  ${chalk.bold.yellow('⚠')} ${chalk.bold.yellow(repo.repoName.padEnd(25))} ${branchPart} ${chalk.gray(`(${repo.repoPath})`)}`);
          console.log(`    Last commit: ${chalk.gray(repo.lastCommitRelative)}`);
          for (const issue of repo.issues) {
            const color = issue.isCritical ? chalk.red : chalk.yellow;
            console.log(`    ${color(`⚠ ${issue.message}`)}`);
          }
          if (cmdOpts.verbose) {
            console.log(`    ${chalk.gray('Path:')} ${chalk.gray(repo.repoPath)}`);
          }
        } else {
          console.log(
            `  ${chalk.green('✓')} ${chalk.white(repo.repoName.padEnd(25))} ${branchPart} ${chalk.gray(`(${repo.lastCommitRelative})`)} ${chalk.green('• Clean')}`
          );
          if (cmdOpts.verbose) {
            console.log(`    ${chalk.gray('Path:')} ${chalk.gray(repo.repoPath)}`);
          }
        }
      });

      if (isJson) {
        outputJson(report);
        return;
      }

      const summaryColor = report.issuesCount > 0 ? chalk.yellow : chalk.green;
      console.log(`\n  ${summaryColor(`Scanned: ${report.scannedCount} repos | Issues: ${report.issuesCount}`)}\n`);
    });
}
