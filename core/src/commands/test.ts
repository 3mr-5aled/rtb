import type { Command } from 'commander';
import type { CliContext } from '../types/context.js';
import { runActionCommand } from '../services/runner.js';

export function registerTestCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('test [project] [args...]')
    .description('Run project test suite')
    .allowUnknownOption(true)
    .option('--dry-run', 'Inspect test command resolution without executing', false)
    .action((projectName: string | undefined, extraArgs: string[] | undefined, options: { dryRun?: boolean }) => {
      return runActionCommand('test', projectName, extraArgs, options, getContext());
    });
}
