import type { Command } from 'commander';
import type { CliContext } from '../types/context.js';
import { runActionCommand } from '../services/runner.js';

export function registerRunCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('run [project] [args...]')
    .description('Run project dev server or main entrypoint')
    .allowUnknownOption(true)
    .option('--dry-run', 'Inspect command resolution without executing', false)
    .action((projectName: string | undefined, extraArgs: string[] | undefined, options: { dryRun?: boolean }) => {
      return runActionCommand('run', projectName, extraArgs, options, getContext());
    });
}
