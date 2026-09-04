import type { Command } from 'commander';
import chalk from 'chalk';
import fs from 'node:fs';
import path from 'node:path';
import type { CliContext } from '../types/context.js';
import { outputError, outputJson } from '../utils/output.js';

export interface CleanTargetItem {
  project: string;
  targetName: string;
  path: string;
  lastModified: string;
  daysInactive: number;
}

export function scanCleanTargets(searchPaths: string[], targets: string[], daysInactiveThreshold: number): CleanTargetItem[] {
  const cutoffTime = Date.now() - daysInactiveThreshold * 24 * 60 * 60 * 1000;
  const flagged: CleanTargetItem[] = [];

  for (const root of searchPaths) {
    if (!root || !fs.existsSync(root)) continue;

    let projects: string[] = [];
    try {
      projects = fs.readdirSync(root);
    } catch {
      continue;
    }

    for (const proj of projects) {
      const projPath = path.join(root, proj);
      try {
        if (!fs.statSync(projPath).isDirectory()) continue;
      } catch {
        continue;
      }

      for (const target of targets) {
        const targetPath = path.join(projPath, target);
        if (!fs.existsSync(targetPath)) continue;

        try {
          const stat = fs.statSync(targetPath);
          if (stat.mtimeMs < cutoffTime) {
            const daysAgo = Math.floor((Date.now() - stat.mtimeMs) / (24 * 60 * 60 * 1000));
            flagged.push({
              project: proj,
              targetName: target,
              path: targetPath,
              lastModified: stat.mtime.toISOString().slice(0, 10),
              daysInactive: daysAgo,
            });
          }
        } catch {}
      }
    }
  }

  return flagged;
}

export function registerCleanCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('clean')
    .description('Prune inactive dependency directories (node_modules, .venv, target, dist, etc.)')
    .option('-c, --commit', 'Perform actual deletion (defaults to dry-run)', false)
    .option('-d, --days <days>', 'Inactivity threshold in days', '60')
    .action((options: { commit?: boolean; days?: string }) => {
      const ctx = getContext();

      if (!ctx.config) {
        outputError('Configuration not loaded', 'CONFIG_MISSING', ctx.isJson);
        process.exitCode = 1;
        return;
      }

      const daysThreshold = parseInt(options.days || '60', 10);
      const targets = ctx.config.cleanDeps?.targets || [
        'node_modules',
        '.venv',
        '.next',
        '__pycache__',
        'dist',
        'build',
        'target',
      ];

      const searchPaths: string[] = [];
      for (const key of ['active', 'paused', 'vibe', 'sandbox']) {
        const p = ctx.config.projectRoots[key]?.path;
        if (p) searchPaths.push(p);
      }

      const flagged = scanCleanTargets(searchPaths, targets, daysThreshold);

      if (ctx.isJson) {
        outputJson({
          dryRun: !options.commit,
          thresholdDays: daysThreshold,
          totalTargets: flagged.length,
          targets: flagged,
        });
        return;
      }

      console.log(`\n${chalk.cyan('══════════════════════════════════════════')}`);
      console.log(`  ${chalk.bold(`rtb (ﺐﺗر) » Dependency Pruning (${daysThreshold}d threshold)`)}`);
      console.log(`${chalk.cyan('══════════════════════════════════════════')}\n`);

      if (!options.commit) {
        console.log(`  ${chalk.cyan('[DRY RUN MODE]')} No files will be deleted. Use ${chalk.yellow('--commit')} to perform deletion.\n`);
      }

      if (flagged.length === 0) {
        console.log(`  ${chalk.green('✓')} No inactive dependency folders found older than ${daysThreshold} days.\n`);
        return;
      }

      for (const item of flagged) {
        console.log(`  ${chalk.red('•')} ${chalk.bold(item.project)} / ${chalk.yellow(item.targetName)} ${chalk.gray(`(inactive ${item.daysInactive}d, ${item.lastModified})`)}`);
      }

      if (options.commit) {
        console.log(`\n  ${chalk.yellow('Pruning directories...')}`);
        let deleted = 0;
        for (const item of flagged) {
          try {
            fs.rmSync(item.path, { recursive: true, force: true });
            deleted++;
          } catch {}
        }
        console.log(`  ${chalk.green('✓')} Successfully pruned ${deleted} folders.\n`);
      } else {
        console.log(`\n  Found ${flagged.length} target directories to reclaim.`);
        console.log(`  Run ${chalk.cyan(`rtb clean --commit --days ${daysThreshold}`)} to prune them.\n`);
      }
    });
}
