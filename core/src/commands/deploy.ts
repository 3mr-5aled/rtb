import type { Command } from 'commander';
import chalk from 'chalk';
import type { CliContext } from '../types/context.js';
import { ProjectLifecycle } from '../services/lifecycle.js';
import { ConfigMissingError, RtbError } from '../errors.js';
import { outputJson } from '../utils/output.js';

export function registerDeployCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('deploy [name]')
    .description('Deploy a project from Active to Production or Staging')
    .option('--prod', 'Deploy to Production (default)', true)
    .option('--staging', 'Deploy to Staging', false)
    .option('--from <root>', 'Specify source project root (e.g. active, staging)')
    .option('--json', 'Output deploy result in JSON format')
    .action((name: string | undefined, options: { prod?: boolean; staging?: boolean; from?: string; json?: boolean }) => {
      const ctx = getContext();
      const isJson = Boolean(options.json || ctx.isJson);

      if (!name) {
        throw new RtbError('Usage: rtb deploy <project-name> [--prod|--staging] [--from <root>]', 'USAGE_ERROR');
      }

      if (!ctx.config) {
        throw new ConfigMissingError('Configuration not loaded');
      }

      const targetEnvironment = options.staging ? 'staging' : 'production';
      const lifecycle = new ProjectLifecycle();
      const result = lifecycle.deploy({
        name,
        config: ctx.config,
        targetEnvironment,
        from: options.from,
      });

      if (isJson) {
        outputJson({
          deployed: true,
          name: result.name,
          target: result.target,
          from: result.from,
          to: result.to,
        });
        return;
      }

      console.log(`\n  ${chalk.cyan('Deploying:')} ${chalk.bold(result.name)} (${options.from || 'Active'}) → ${chalk.green(result.target)}`);
      console.log(`  Target: ${chalk.gray(result.to)}`);
      console.log(`  ${chalk.green('✓')} '${chalk.bold(result.name)}' deployed to ${result.target}!\n`);
    });
}
