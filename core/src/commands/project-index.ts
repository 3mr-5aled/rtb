import type { Command } from 'commander';
import chalk from 'chalk';
import fs from 'node:fs';
import path from 'node:path';
import type { CliContext } from '../types/context.js';
import { scanAllProjects } from '../inspector/inspector.js';
import { outputJson } from '../utils/output.js';

export function registerIndexCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('index')
    .description('Generate PROJECT-INDEX.md inventory of all projects and stacks')
    .option('-o, --out <path>', 'Custom path for output file')
    .action((options: { out?: string }) => {
      const ctx = getContext();
      if (!ctx.config) {
        if (ctx.isJson) {
          outputJson({ error: true, message: 'Configuration not found' });
        } else {
          console.error(chalk.red('  ✗ Configuration not found.'));
        }
        return;
      }

      if (!ctx.isJson) {
        console.log(`\n${chalk.cyan('══════════════════════════════════════════')}`);
        console.log(`  ${chalk.bold('rtb (ﺐﺗر) » Project Index Generator')}`);
        console.log(`${chalk.cyan('══════════════════════════════════════════')}\n`);
      }

      const projects = scanAllProjects(ctx.config, 'all', {
        onProject: (p) => {
          if (!ctx.isJson) {
            const stackStr = p.stack.filter((s) => s !== '-').join(', ') || 'General';
            console.log(`  ${chalk.green('✓')} Discovered: ${chalk.white.bold(p.name.padEnd(28))} ${chalk.cyan(`[${stackStr}]`)}`);
          }
        },
      });

      if (ctx.isJson) {
        outputJson({
          timestamp: new Date().toISOString(),
          total: projects.length,
          projects,
        });
        return;
      }

      const nowStr = new Date().toISOString().replace('T', ' ').slice(0, 16);
      let md = `# Project Index\n\n> Generated ${nowStr}\n\n| Project | Status | Stack | Last Modified |\n|:---|:---|:---|:---|\n`;

      for (const p of projects) {
        const stackStr = p.stack.length > 0 ? p.stack.join(', ') : '-';
        const modStr = p.last_modified ? p.last_modified.slice(0, 10) : '-';
        md += `| ${p.name} | ${p.status} | ${stackStr} | ${modStr} |\n`;
      }

      md += `\n---\n*Total: ${projects.length} projects*\n`;

      let targetPath = options.out;
      if (!targetPath) {
        const activeRoot = ctx.config.projectRoots.active?.path;
        if (activeRoot && fs.existsSync(activeRoot)) {
          targetPath = path.join(path.dirname(activeRoot), 'PROJECT-INDEX.md');
        } else {
          targetPath = path.resolve('PROJECT-INDEX.md');
        }
      } else {
        targetPath = path.resolve(targetPath);
      }

      fs.writeFileSync(targetPath, md, 'utf-8');
      console.log(`  ${chalk.green('✓')} Generated index: ${projects.length} projects → ${chalk.cyan(targetPath)}\n`);
    });
}
