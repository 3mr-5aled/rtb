import type { Command } from 'commander';
import chalk from 'chalk';
import type { CliContext } from '../types/context.js';
import { outputJson, outputError } from '../utils/output.js';

export function registerConfigCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('config')
    .description('View or manage RTB configuration')
    .option('--path', 'Display the path to the active configuration file')
    .action((options: { path?: boolean }) => {
      const ctx = getContext();

      if (options.path) {
        if (ctx.isJson) {
          outputJson({
            configPath: ctx.configPath,
            exists: Boolean(ctx.config),
            isConfigured: ctx.isConfigured,
          });
        } else {
          console.log(ctx.configPath);
        }
        return;
      }

      if (!ctx.config) {
        outputError(`No valid configuration found at: ${ctx.configPath}`, 'CONFIG_NOT_FOUND', ctx.isJson);
        process.exitCode = 1;
        return;
      }

      if (ctx.isJson) {
        outputJson(ctx.config);
      } else {
        console.log(`\n${chalk.bold('RTB Configuration')} (${chalk.gray(ctx.configPath)}):`);
        console.log(`  Version: ${chalk.cyan(ctx.config.version)}`);
        console.log(`  Project Roots:`);
        for (const [key, entry] of Object.entries(ctx.config.projectRoots)) {
          console.log(`    ${entry.emoji} ${chalk.bold(entry.label)} (${chalk.yellow(key)}): ${chalk.gray(entry.path)}`);
        }
        if (ctx.config.backupRoot) {
          console.log(`  Backup Root: ${chalk.gray(ctx.config.backupRoot)}`);
        }
        console.log('');
      }
    });
}
