import { Command } from 'commander';
import chalk from 'chalk';
import readline from 'node:readline';
import { loadConfig } from './config/loader.js';
import type { CliContext } from './types/context.js';
import { registerVersionCommand, RTB_VERSION } from './commands/version.js';
import { registerConfigCommand } from './commands/config.js';
import { registerGotoCommand } from './commands/goto.js';
import { registerListCommand } from './commands/list.js';
import { registerIndexCommand } from './commands/project-index.js';
import { registerShellInitCommand } from './commands/shell-init.js';
import { registerNewCommand } from './commands/new.js';
import { registerPauseCommand } from './commands/pause.js';
import { registerResumeCommand } from './commands/resume.js';
import { registerArchiveCommand } from './commands/archive.js';
import { registerUnarchiveCommand } from './commands/unarchive.js';
import { registerCleanCommand } from './commands/clean.js';
import { registerAgentCommand } from './commands/agent.js';
import { registerDoctorCommand } from './commands/doctor.js';
import { registerStatusCommand } from './commands/status.js';
import { registerInfoCommand } from './commands/info.js';
import { registerUiCommand } from './commands/ui.js';
import { registerInitCommand } from './commands/init.js';
import { registerHelpCommand } from './commands/help.js';
import { registerRunnerCommands } from './services/runner.js';
import { registerDepsCommand } from './commands/deps.js';
import { registerWorkspaceCommand } from './commands/workspace.js';
import { registerOpenCommand } from './commands/open.js';
import { registerHealthCommand } from './commands/health.js';
import { registerMaintenanceCommands } from './services/maintenance.js';
import { registerDeployCommand } from './commands/deploy.js';
import { registerUninstallCommand } from './commands/uninstall.js';
import { registerUpgradeCommand } from './commands/upgrade.js';
import { registerCompletionCommand } from './commands/completion.js';
import { outputError } from './utils/output.js';
import { wrapAction } from './utils/envelope.js';


export const EXEMPT_COMMANDS = new Set([
  'help',
  'version',
  'init',
  'config',
  'doctor',
  'shell-init',
  'uninstall',
  'upgrade',
  'completion',
  '__complete',
]);

export function createCli(argv: string[] = process.argv): Command {
  const program = new Command();

  program
    .name('rtb')
    .description('RTB (ﺐﺗر) — Unified developer workspace & project manager')
    .version(RTB_VERSION, '-v, --version', 'Output the current version')
    .option('-c, --config <path>', 'Custom path to rtb.config.json')
    .option('--json', 'Output result in JSON format where supported', false)
    .option('-q, --quiet', 'Suppress non-essential progress output', false);

  // Lazy context resolver updated on command execution
  let currentContext: CliContext = {
    config: null,
    configPath: '',
    isConfigured: false,
    isJson: false,
    isQuiet: false,
    isInteractive: process.stdin.isTTY && !process.env.CI && !process.env.RTB_NON_INTERACTIVE,
  };

  const getContext = (): CliContext => currentContext;

  // Automatically wrap all registered command action handlers in CommandEnvelope
  const originalCommand = program.command.bind(program);
  (program as any).command = function (...args: any[]) {
    const cmd = (originalCommand as any)(...args);
    const originalAction = cmd.action.bind(cmd);
    cmd.action = function (fn: (...a: any[]) => any) {
      const isExempt = EXEMPT_COMMANDS.has(cmd.name());
      return originalAction(wrapAction(getContext, fn, { exemptFromConfig: isExempt }));
    };
    return cmd;
  };

  // Config Gate Middleware via preAction hook
  program.hook('preAction', async (thisCommand, actionCommand) => {
    const opts = thisCommand.opts<{ config?: string; json?: boolean; quiet?: boolean }>();
    const resolution = loadConfig(opts.config);

    currentContext = {
      config: resolution.config,
      configPath: resolution.configPath,
      isConfigured: resolution.isConfigured,
      isJson: Boolean(opts.json),
      isQuiet: Boolean(opts.quiet),
      isInteractive: process.stdin.isTTY && !process.env.CI && !process.env.RTB_NON_INTERACTIVE,
    };

    const cmdName = actionCommand.name();
    const isExempt = EXEMPT_COMMANDS.has(cmdName);

    if (!isExempt && !resolution.isConfigured) {
      if (currentContext.isJson) {
        outputError(
          'RTB is not configured yet. Run "rtb init" to initialize workspace.',
          'NOT_CONFIGURED',
          true
        );
        process.exitCode = 1;
        return;
      }

      console.log('');
      console.log(`  ${chalk.yellow('⚠')}  ${chalk.yellow('RTB is not configured yet.')}`);
      console.log(`     Run '${chalk.cyan('rtb init')}' to set up your workspace (or edit ${chalk.gray(resolution.configPath)} directly).`);
      console.log('');

      if (currentContext.isInteractive) {
        const rl = readline.createInterface({
          input: process.stdin,
          output: process.stdout,
        });

        const answer: string = await new Promise((resolve) => {
          rl.question('  Would you like to configure now? (Y/n) ', (ans) => {
            rl.close();
            resolve(ans.trim().toLowerCase());
          });
        });

        if (answer === '' || answer === 'y' || answer === 'yes') {
          // Future: launch init
          console.log(`\n  Please run '${chalk.cyan('rtb init')}' to proceed.\n`);
          process.exitCode = 0;
          return;
        }
      }

      process.exitCode = 1;
      return;
    }
  });

  // Register commands
  registerVersionCommand(program, getContext);
  registerConfigCommand(program, getContext);
  registerGotoCommand(program, getContext);
  registerListCommand(program, getContext);
  registerIndexCommand(program, getContext);
  registerShellInitCommand(program, getContext);
  registerNewCommand(program, getContext);
  registerPauseCommand(program, getContext);
  registerResumeCommand(program, getContext);
  registerArchiveCommand(program, getContext);
  registerUnarchiveCommand(program, getContext);
  registerCleanCommand(program, getContext);
  registerAgentCommand(program, getContext);
  registerDoctorCommand(program, getContext);
  registerStatusCommand(program, getContext);
  registerInfoCommand(program, getContext);
  registerUiCommand(program, getContext);
  registerInitCommand(program, getContext);
  registerRunnerCommands(program, getContext);
  registerDepsCommand(program, getContext);
  registerWorkspaceCommand(program, getContext);
  registerOpenCommand(program, getContext);
  registerHealthCommand(program, getContext);
  registerMaintenanceCommands(program, getContext);
  registerDeployCommand(program, getContext);
  registerUninstallCommand(program, getContext);
  registerUpgradeCommand(program, getContext);
  registerCompletionCommand(program, getContext);
  registerHelpCommand(program);

  return program;
}
