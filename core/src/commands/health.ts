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
    .action((cmdOpts: { json?: boolean }) => {
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

      const staleThreshold = ctx.config?.staleThresholdDays || 30;
      const report = scanGitHealth(scanRoots, staleThreshold);

      if (isJson) {
        outputJson(report);
        return;
      }

      console.log('');
      console.log(chalk.cyan('═'.repeat(60)));
      console.log(`  ${chalk.bold.cyan('rtb (رتّب)')} » Git Repository Health`);
      console.log(chalk.cyan('═'.repeat(60)));

      for (const repo of report.repos) {
        if (repo.issues.length > 0) {
          console.log(`\n  ${chalk.bold.yellow(repo.repoPath)}`);
          console.log(`    Last commit: ${chalk.gray(repo.lastCommitRelative)}`);
          for (const issue of repo.issues) {
            const color = issue.isCritical ? chalk.red : chalk.yellow;
            console.log(`    ${color(`⚠ ${issue.message}`)}`);
          }
        }
      }

      const summaryColor = report.issuesCount > 0 ? chalk.yellow : chalk.green;
      console.log(`\n  ${summaryColor(`Scanned: ${report.scannedCount} repos | Issues: ${report.issuesCount}`)}\n`);
    });
}
