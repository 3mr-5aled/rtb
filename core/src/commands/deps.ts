import type { Command } from 'commander';
import chalk from 'chalk';
import path from 'node:path';
import fs from 'node:fs';
import type { CliContext } from '../types/context.js';
import { inspectDependencies } from '../inspector/dependencies.js';
import { resolveProjectTarget } from '../navigation/fuzzy.js';
import { outputJson } from '../utils/output.js';
import { ProjectNotFoundError } from '../errors.js';
import { withSpinner } from '../utils/spinner.js';

export function registerDepsCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('deps [project]')
    .description('Audit declared project dependencies across ecosystems')
    .action(async (projectName?: string) => {
      const ctx = getContext();
      let targetPath = process.cwd();

      if (projectName) {
        const target = resolveProjectTarget(projectName, ctx.config);
        if (!target) {
          throw new ProjectNotFoundError(projectName, 'PROJECT_NOT_FOUND');
        }
        targetPath = target.targetPath;
      }

      const leaf = path.basename(targetPath);
      if (!ctx.isJson) {
        console.log(`\n${chalk.cyan('══════════════════════════════════════════')}`);
        console.log(`  ${chalk.bold(`rtb (ﺐﺗر) » Dependency Inspector (${leaf})`)}`);
        console.log(`${chalk.cyan('══════════════════════════════════════════')}\n`);
      }

      const deps = await withSpinner(
        `Auditing dependencies for ${leaf}...`,
        () => inspectDependencies(targetPath),
        { quiet: ctx.isQuiet, json: ctx.isJson }
      );

      if (ctx.isJson) {
        outputJson(deps);
        return;
      }

      if (deps.length === 0) {
        console.log(chalk.yellow(`  No dependencies found in ${targetPath}\n`));
        return;
      }

      console.log(chalk.green(`  Found ${deps.length} declared dependencies:\n`));

      // Calculate column widths
      const colPkg = Math.max(7, ...deps.map((d) => d.package.length));
      const colSpec = Math.max(4, ...deps.map((d) => d.spec.length));
      const colType = Math.max(4, ...deps.map((d) => d.type.length));

      const header = `  ${'Package'.padEnd(colPkg)}  ${'Spec'.padEnd(colSpec)}  ${'Type'.padEnd(colType)}  Status`;
      console.log(chalk.gray(header));
      console.log(chalk.gray(`  ${'─'.repeat(header.length - 2)}`));

      for (const d of deps) {
        const pkgStr = chalk.cyan(d.package.padEnd(colPkg));
        const specStr = chalk.white(d.spec.padEnd(colSpec));
        const typeStr = chalk.yellow(d.type.padEnd(colType));
        const statusStr = chalk.green(d.status);
        console.log(`  ${pkgStr}  ${specStr}  ${typeStr}  ${statusStr}`);
      }
      console.log('');
    });
}
