import type { Command } from 'commander';
import chalk from 'chalk';
import path from 'node:path';
import type { CliContext } from '../types/context.js';
import { inspectWorkspace } from '../inspector/workspace.js';
import { resolveProjectTarget } from '../navigation/fuzzy.js';
import { outputJson, outputError } from '../utils/output.js';

export function registerWorkspaceCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('workspace [project]')
    .description('Inspect monorepo workspace packages (pnpm, npm/yarn, Cargo)')
    .option('--json', 'Output workspace details in JSON format')
    .action((projectName: string | undefined, cmdOpts: { json?: boolean }) => {
      const ctx = getContext();
      const isJson = Boolean(cmdOpts.json || ctx.isJson);

      const target = resolveProjectTarget(projectName, ctx.config);
      if (!target) {
        outputError(`Project or path '${projectName}' not found.`, 'NOT_FOUND', isJson);
        if (process.env.VITEST) return;
        process.exit(1);
      }

      const { targetPath } = target;
      const info = inspectWorkspace(targetPath);

      if (isJson) {
        outputJson(info);
        return;
      }

      console.log('');
      console.log(chalk.cyan('═'.repeat(60)));
      console.log(`  ${chalk.bold.cyan('rtb (رتّب)')} » Monorepo Workspace Inspector (${path.basename(targetPath)})`);
      console.log(chalk.cyan('═'.repeat(60)));
      console.log(`  Path:          ${chalk.gray(info.projectPath)}`);
      console.log(`  Monorepo Type: ${chalk.cyan(info.workspaceType)}`);
      console.log('');

      if (info.packages.length > 0) {
        console.log(`  ${chalk.bold.green('Declared Workspace Packages:')}`);
        const maxPatternLen = Math.max(14, ...info.packages.map((p) => p.packagePattern.length));
        console.log(`  ${chalk.gray('Package Pattern'.padEnd(maxPatternLen + 4))}${chalk.gray('Type')}`);
        console.log(`  ${chalk.gray('─'.repeat(maxPatternLen + 10))}`);
        for (const pkg of info.packages) {
          console.log(`  ${pkg.packagePattern.padEnd(maxPatternLen + 4)}${chalk.yellow(pkg.type)}`);
        }
        console.log('');
      } else {
        console.log(`  ${chalk.yellow('ℹ')} No active monorepo workspace configurations detected.`);
        console.log('');
      }
    });
}
