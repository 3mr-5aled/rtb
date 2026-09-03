import type { Command } from 'commander';
import chalk from 'chalk';
import fs from 'node:fs';
import path from 'node:path';
import type { CliContext } from '../types/context.js';
import { scanAllProjects } from '../inspector/inspector.js';
import { outputJson } from '../utils/output.js';

export function registerListCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('list')
    .description('List registered projects across workspace roots')
    .option('--active', 'Filter active projects only')
    .option('--paused', 'Filter paused projects only')
    .option('--deployed', 'Filter production and staging projects')
    .option('--vibe', 'Filter vibe projects')
    .option('--all', 'List all projects across all roots (default)')
    .action((options: { active?: boolean; paused?: boolean; deployed?: boolean; vibe?: boolean; all?: boolean }) => {
      const ctx = getContext();
      if (!ctx.config) {
        if (ctx.isJson) {
          outputJson([]);
        } else {
          console.error(chalk.red('  ✗ Configuration not found.'));
        }
        return;
      }

      let filter = 'all';
      if (options.active) filter = 'active';
      else if (options.paused) filter = 'paused';
      else if (options.deployed) filter = 'deployed';
      else if (options.vibe) filter = 'vibe';

      const projects = scanAllProjects(ctx.config, filter);

      if (ctx.isJson) {
        outputJson(projects);
        return;
      }

      console.log(`\n${chalk.cyan('══════════════════════════════════════════')}`);
      console.log(`  ${chalk.bold('rtb (رتّب) » Project List')}`);
      console.log(`${chalk.cyan('══════════════════════════════════════════')}\n`);

      const categories = [
        { key: 'active', label: ctx.config.projectRoots.active?.label || 'Active', emoji: ctx.config.projectRoots.active?.emoji || '📁', path: ctx.config.projectRoots.active?.path },
        { key: 'paused', label: ctx.config.projectRoots.paused?.label || 'Paused', emoji: ctx.config.projectRoots.paused?.emoji || '⏸️', path: ctx.config.projectRoots.paused?.path },
        { key: 'production', label: ctx.config.projectRoots.production?.label || 'Production', emoji: ctx.config.projectRoots.production?.emoji || '🚀', path: ctx.config.projectRoots.production?.path },
        { key: 'staging', label: ctx.config.projectRoots.staging?.label || 'Staging', emoji: ctx.config.projectRoots.staging?.emoji || '🚀', path: ctx.config.projectRoots.staging?.path },
        { key: 'vibe', label: ctx.config.projectRoots.vibe?.label || 'Vibe', emoji: ctx.config.projectRoots.vibe?.emoji || '✨', path: ctx.config.projectRoots.vibe?.path },
      ];

      let total = 0;
      for (const cat of categories) {
        if (!cat.path || !fs.existsSync(cat.path)) continue;

        const catProjects = projects.filter((p) => path.dirname(p.path).toLowerCase() === cat.path!.toLowerCase());
        if (catProjects.length === 0) continue;

        console.log(`  ${cat.emoji} ${chalk.yellow.bold(cat.label)} (${catProjects.length})`);
        for (const p of catProjects) {
          total++;
          const modDate = p.last_modified ? p.last_modified.slice(0, 10) : '-';
          console.log(`    ${chalk.white(p.name.padEnd(35))} ${chalk.gray(`(${modDate})`)}`);
        }
        console.log('');
      }

      console.log(`  ${chalk.gray(`Total: ${total} projects`)}\n`);
    });
}
