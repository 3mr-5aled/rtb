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
    .option('-v, --verbose', 'Display detailed project inspection information')
    .action((options: { active?: boolean; paused?: boolean; deployed?: boolean; vibe?: boolean; all?: boolean; verbose?: boolean }) => {
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

      if (ctx.isJson) {
        const projects = scanAllProjects(ctx.config, filter);
        outputJson(projects);
        return;
      }

      console.log(`\n${chalk.cyan('══════════════════════════════════════════')}`);
      console.log(`  ${chalk.bold('rtb (رتّب) » Project List')}`);
      console.log(`${chalk.cyan('══════════════════════════════════════════')}\n`);

      let total = 0;
      scanAllProjects(ctx.config, filter, {
        onCategory: (cat) => {
          console.log(`  ${cat.emoji} ${chalk.yellow.bold(cat.label)} (${cat.count})`);
        },
        onProject: (p) => {
          total++;
          const modDate = p.last_modified ? p.last_modified.slice(0, 10) : '-';
          const stackFiltered = p.stack.filter((s) => s !== '-');
          const stackStr = stackFiltered.length > 0 ? stackFiltered.join(', ') : '-';
          const branch = p.git?.branch;
          const branchStr = branch ? chalk.magenta(`[${branch}]`) : '';
          const uncommittedStr = p.git?.uncommitted ? chalk.yellow(`*${p.git.uncommitted}`) : '';
          const gitPart = [branchStr, uncommittedStr].filter(Boolean).join(' ');

          console.log(
            `    ${chalk.white.bold(p.name.padEnd(28))} ${chalk.cyan(stackStr.padEnd(22))} ${gitPart ? gitPart.padEnd(16) : ''.padEnd(16)} ${chalk.gray(`(${modDate})`)}`
          );

          if (options.verbose) {
            console.log(`      ${chalk.gray('Path:')} ${chalk.gray(p.path)}`);
            if (p.runtime_version) {
              console.log(`      ${chalk.gray('Runtime:')} ${chalk.gray(p.runtime_version)}`);
            }
            if (p.git?.last_commit_msg) {
              console.log(`      ${chalk.gray('Last Commit:')} ${chalk.gray(`${p.git.last_commit_msg} (${p.git.last_commit_relative})`)}`);
            }
          }
        },
        onCategoryEnd: () => {
          console.log('');
        },
      });

      console.log(`  ${chalk.gray(`Total: ${total} projects`)}\n`);
    });
}
