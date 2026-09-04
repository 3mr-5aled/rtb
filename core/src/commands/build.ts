import type { Command } from 'commander';
import type { CliContext } from '../types/context.js';
import { runActionCommand } from '../services/runner.js';

export function registerBuildCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('build [project] [args...]')
    .description('Build project for release or compilation')
    .allowUnknownOption(true)
    .option('--dry-run', 'Inspect build command resolution without executing', false)
    .action((projectName: string | undefined, extraArgs: string[] | undefined, options: { dryRun?: boolean }) => {
      return runActionCommand('build', projectName, extraArgs, options, getContext());
    });
}
