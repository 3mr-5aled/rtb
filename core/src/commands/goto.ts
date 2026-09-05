import type { Command } from 'commander';
import chalk from 'chalk';
import readline from 'node:readline';
import type { CliContext } from '../types/context.js';
import { findProjectPathFuzzy } from '../navigation/fuzzy.js';
import { AgentOrchestrator } from '../services/agent.js';
import { outputError, outputJson } from '../utils/output.js';
import { ConfigMissingError } from '../errors.js';

export function registerGotoCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('goto [project]')
    .description('Navigate to a project directory across registered project roots')
    .option('--print', 'Print resolved target path directly to stdout without formatting')
    .option('--choice <number>', 'Direct selection index for multi-match resolution')
    .option('--agy', 'Launch Antigravity agent upon arrival')
    .option('--claude', 'Launch Claude Code agent upon arrival')
    .option('--gemini', 'Launch Gemini CLI agent upon arrival')
    .option('--cursor', 'Open project in Cursor')
    .option('--windsurf', 'Open project in Windsurf')
    .option('--aider', 'Launch Aider upon arrival')
    .option('--openhands', 'Launch OpenHands upon arrival')
    .option('--no-launch', 'Generate context without launching agent')
    .action(async (projectName: string | undefined, options: {
      print?: boolean;
      choice?: string;
      agy?: boolean;
      claude?: boolean;
      gemini?: boolean;
      cursor?: boolean;
      windsurf?: boolean;
      aider?: boolean;
      openhands?: boolean;
      launch?: boolean;
      noLaunch?: boolean;
    }) => {
      const ctx = getContext();

      if (!projectName) {
        if (options.print) {
          process.exitCode = 1;
          return;
        }
        console.log(`\n  ${chalk.yellow('Usage:')} rtb goto <project-name> [--agy|--claude|...]`);
        console.log(`  ${chalk.gray('Tip: Use fuzzy matching or partial names to jump instantly.')}\n`);
        return;
      }

      if (!ctx.config) {
        throw new ConfigMissingError('Configuration not loaded');
      }

      const matches = findProjectPathFuzzy(projectName, ctx.config);

      if (matches.length === 0) {
        if (options.print) {
          process.exitCode = 1;
          return;
        }
        if (ctx.isJson) {
          outputJson({ found: false, matches: [] });
          process.exitCode = 1;
          return;
        } else {
          console.error(`\n  ${chalk.red('✗')} No project matching '${chalk.bold(projectName)}' found.\n`);
          process.exitCode = 1;
          return;
        }
      }

      let selected = matches[0];

      // If top match is 100 or unique, select it immediately
      if (matches.length > 1 && !(matches[0].score === 100 && matches[1].score < 100)) {
        if (options.choice) {
          const idx = parseInt(options.choice, 10) - 1;
          if (idx >= 0 && idx < matches.length) {
            selected = matches[idx];
          }
        } else if (options.print || !ctx.isInteractive) {
          // In non-interactive or print mode, default to best match
          selected = matches[0];
        } else {
          console.log(`\n  ${chalk.yellow('Multiple projects match')} '${chalk.bold(projectName)}':`);
          const limit = Math.min(matches.length, 9);
          for (let i = 0; i < limit; i++) {
            const m = matches[i];
            console.log(`  [${chalk.cyan(i + 1)}] ${m.name.padEnd(35)} ${chalk.gray(`(${m.status})`)}`);
          }
          console.log('');

          const rl = readline.createInterface({
            input: process.stdin,
            output: process.stdout,
          });

          const choiceStr: string = await new Promise((resolve) => {
            rl.question(chalk.yellow('  Select [1-9] or Enter to cancel: '), (ans) => {
              rl.close();
              resolve(ans.trim());
            });
          });

          const selNum = parseInt(choiceStr, 10);
          if (!isNaN(selNum) && selNum >= 1 && selNum <= limit) {
            selected = matches[selNum - 1];
          } else {
            console.log(chalk.gray('  Cancelled.\n'));
            return;
          }
        }
      }

      if (options.print) {
        // Raw stdout output for shell cd hook
        process.stdout.write(selected.path);
        return;
      }

      const agentName = options.agy
        ? 'agy'
        : options.claude
        ? 'claude'
        : options.gemini
        ? 'gemini'
        : options.cursor
        ? 'cursor'
        : options.windsurf
        ? 'windsurf'
        : options.aider
        ? 'aider'
        : options.openhands
        ? 'openhands'
        : undefined;

      let orchResult;
      if (agentName) {
        const orchestrator = new AgentOrchestrator();
        orchResult = await orchestrator.orchestrate({
          projectPath: selected.path,
          projectName: selected.name,
          agent: agentName,
          config: ctx.config,
          launch: options.noLaunch || options.launch === false ? false : true,
          quiet: ctx.isJson,
        });

        if (orchResult.exitCode !== undefined && orchResult.exitCode !== 0) {
          process.exitCode = orchResult.exitCode;
        }
      }

      if (ctx.isJson) {
        outputJson({
          found: true,
          project: selected,
          agent: orchResult?.agent,
          contextFile: orchResult?.contextPath,
        });
        return;
      }

      if (!agentName) {
        console.log(`  ${chalk.cyan(selected.status)} » ${chalk.green(selected.path)}`);
      }
    });
}
