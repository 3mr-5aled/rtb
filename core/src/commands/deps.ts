import type { Command } from 'commander';
import chalk from 'chalk';
import path from 'node:path';
import fs from 'node:fs';
import type { CliContext } from '../types/context.js';
import { inspectDependencies } from '../inspector/dependencies.js';
import { findProjectPathFuzzy } from '../navigation/fuzzy.js';
import { outputError, outputJson } from '../utils/output.js';

export function registerDepsCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('deps [project]')
    .description('Audit declared project dependencies across ecosystems')
    .action((projectName?: string) => {
      const ctx = getContext();
      let targetPath = process.cwd();

      if (projectName && ctx.config) {
        if (fs.existsSync(projectName)) {
          targetPath = path.resolve(projectName);
        } else {
          const matches = findProjectPathFuzzy(projectName, ctx.config);
          if (matches.length > 0) {
            targetPath = matches[0].path;
          } else {
            outputError(`Project '${projectName}' not found.`, 'PROJECT_NOT_FOUND', ctx.isJson);
            process.exit(1);
            return;
          }
        }
      }

      const deps = inspectDependencies(targetPath);

      if (ctx.isJson) {
        outputJson(deps);
        return;
      }

      const leaf = path.basename(targetPath);
      console.log(`\n${chalk.cyan('══════════════════════════════════════════')}`);
      console.log(`  ${chalk.bold(`rtb (ﺐﺗر) » Dependency Inspector (${leaf})`)}`);
      console.log(`${chalk.cyan('══════════════════════════════════════════')}\n`);

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
