import type { Command } from 'commander';
import chalk from 'chalk';
import * as p from '@clack/prompts';
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import type { CliContext } from '../types/context.js';
import { outputJson } from '../utils/output.js';
import { getProjectNames } from './completion.js';
import { resolveProjectTarget } from '../navigation/fuzzy.js';
import {
  resolveProjectAction,
  executeProjectAction,
  type ProjectAction,
} from '../services/runner.js';
import { findRtbtuiBinary } from './doctor.js';
import { AgentOrchestrator } from '../services/agent.js';
import { openInEditor } from './config.js';
import { getStandardConfigPath } from '../config/loader.js';
import { scanGitHealth } from '../inspector/health.js';
import { TaskSpinner, withSpinner } from '../utils/spinner.js';
import { runDoctorChecks } from './doctor.js';

export const prompts = {
  intro: p.intro,
  outro: p.outro,
  select: p.select,
  confirm: p.confirm,
  text: p.text,
  cancel: p.cancel,
  isCancel: p.isCancel,
  log: p.log,
};

export interface MenuActionItem {
  value: string;
  label: string;
  hint: string;
}

export const MENU_ACTIONS: MenuActionItem[] = [
  {
    value: 'run_build_test',
    label: '🚀 Run / Build / Test',
    hint: 'Execute dev, start, build, or test scripts',
  },
  {
    value: 'goto',
    label: '📂 Goto Project',
    hint: 'Find and navigate to a project directory',
  },
  {
    value: 'ui',
    label: '🖥️  Launch TUI',
    hint: 'Launch the Ratatui interactive operations center',
  },
  {
    value: 'health_doctor',
    label: '🩺 Health Scan & Diagnostics',
    hint: 'Scan Git repositories or run system doctor',
  },
  {
    value: 'agent',
    label: '🤖 AI Agent Cockpit',
    hint: 'Launch AI agents (agy, claude, gemini, cursor, etc.) with context',
  },
  {
    value: 'config',
    label: '⚙️  Configuration',
    hint: 'View or edit rtb.config.json',
  },
  {
    value: 'exit',
    label: '❌ Exit',
    hint: 'Return to shell',
  },
];

async function handleRunBuildTest(ctx: CliContext): Promise<void> {
  const actionType = await prompts.select({
    message: 'What action would you like to run?',
    options: [
      { value: 'run', label: '⚡ Run (dev/start script)', hint: 'npm run dev / cargo run / python main.py' },
      { value: 'build', label: '🔨 Build project', hint: 'npm run build / cargo build --release' },
      { value: 'test', label: '🧪 Run test suite', hint: 'npm test / cargo test / pytest' },
    ],
  });

  if (prompts.isCancel(actionType)) {
    prompts.outro('Action cancelled.');
    return;
  }

  const projects = getProjectNames(ctx.config);
  if (projects.length === 0) {
    prompts.log.warn('No managed projects found in workspace.');
    prompts.outro('Completed.');
    return;
  }

  const chosenProj = await prompts.select({
    message: `Select project to ${actionType}:`,
    options: projects.map((p) => ({ value: p, label: p })),
  });

  if (prompts.isCancel(chosenProj)) {
    prompts.outro('Action cancelled.');
    return;
  }

  const target = resolveProjectTarget(chosenProj as string, ctx.config);
  if (!target) {
    prompts.log.warn(`Project '${chosenProj}' not found.`);
    prompts.outro('Completed.');
    return;
  }

  const resolved = resolveProjectAction(actionType as ProjectAction, target.targetPath);
  if (!resolved) {
    prompts.log.warn(`No standard ${actionType} script detected in ${target.targetName}.`);
    prompts.outro('Completed.');
    return;
  }

  prompts.outro(`Executing: ${resolved.executable} ${resolved.args.join(' ')} in ${target.targetName}`);
  await executeProjectAction(target.targetPath, resolved);
}

async function handleGoto(ctx: CliContext): Promise<void> {
  const projects = getProjectNames(ctx.config);
  if (projects.length === 0) {
    prompts.log.warn('No managed projects found in workspace.');
    prompts.outro('Completed.');
    return;
  }

  const chosenProj = await prompts.select({
    message: 'Select project to navigate to:',
    options: projects.map((p) => ({ value: p, label: p })),
  });

  if (prompts.isCancel(chosenProj)) {
    prompts.outro('Action cancelled.');
    return;
  }

  const target = resolveProjectTarget(chosenProj as string, ctx.config);
  prompts.outro(`Selected: ${chalk.bold.green(chosenProj as string)}`);
  console.log(`\n  ${chalk.cyan('Path:')} ${target?.targetPath}`);
  console.log(`  ${chalk.dim(`Tip: Run 'rtb goto ${chosenProj}' in your shell to cd into this directory.`)}\n`);
}

async function handleUi(ctx: CliContext): Promise<void> {
  const binaryPath = findRtbtuiBinary();
  if (!binaryPath) {
    prompts.log.warn('rtbtui binary not found. Build with cargo build --release -p rtbtui in tui/');
    prompts.outro('Completed.');
    return;
  }

  prompts.outro('Launching RTB TUI...');
  const isWindows = process.platform === 'win32';
  const args: string[] = [];
  const env = { ...process.env };
  if (ctx.configPath) {
    args.push('--config', ctx.configPath);
    env.RTB_CONFIG = ctx.configPath;
  }

  await new Promise<void>((resolve) => {
    const child = isWindows
      ? spawn(
          [
            binaryPath.includes(' ') ? `"${binaryPath}"` : binaryPath,
            ...args.map((a) => (a.includes(' ') ? `"${a}"` : a)),
          ].join(' '),
          { stdio: 'inherit', shell: true, env }
        )
      : spawn(binaryPath, args, { stdio: 'inherit', shell: false, env });

    child.on('close', () => resolve());
    child.on('error', () => resolve());
  });
}

async function handleHealthDoctor(ctx: CliContext): Promise<void> {
  const diagChoice = await prompts.select({
    message: 'Select diagnostic tool:',
    options: [
      { value: 'health', label: '🏥 Git Repository Health Scan', hint: 'Scan uncommitted, unpushed, and stale repos' },
      { value: 'doctor', label: '🩺 System Doctor', hint: 'Verify environment, agents, and toolchains' },
    ],
  });

  if (prompts.isCancel(diagChoice)) {
    prompts.outro('Action cancelled.');
    return;
  }

  if (diagChoice === 'health') {
    prompts.outro('Scanning workspace Git health...');
    const scanRoots: string[] = [];
    if (ctx.config?.gitHealth?.scanRoots && ctx.config.gitHealth.scanRoots.length > 0) {
      for (const root of ctx.config.gitHealth.scanRoots) {
        if (fs.existsSync(root)) scanRoots.push(root);
      }
    } else if (ctx.config?.projectRoots) {
      for (const entry of Object.values(ctx.config.projectRoots)) {
        if (entry.path && fs.existsSync(entry.path)) scanRoots.push(entry.path);
      }
    }
    if (scanRoots.length === 0) scanRoots.push(process.cwd());

    const spinner = new TaskSpinner('Scanning Git repositories...', { quiet: false });
    spinner.start();
    const staleThreshold = ctx.config?.staleThresholdDays || 30;
    const report = scanGitHealth(scanRoots, staleThreshold);
    spinner.stop();

    const cleanCount = report.repos.filter((r) => r.issues.length === 0).length;
    console.log(`\n  ${chalk.bold('Scanned:')} ${report.scannedCount} repositories`);
    console.log(`  ${chalk.bold('Clean:')}   ${chalk.green(cleanCount)}`);
    console.log(`  ${chalk.bold('Issues:')}  ${report.issuesCount > 0 ? chalk.yellow(report.issuesCount) : chalk.green(0)}\n`);
  } else {
    prompts.outro('Running System Doctor...');
    const { allGood, checks } = await withSpinner(
      'Diagnosing system environment and toolchains...',
      () => runDoctorChecks(ctx),
      { quiet: false }
    );

    console.log('');
    for (const c of checks) {
      const sym = c.passed ? chalk.green('✓') : c.optional ? chalk.yellow('⚠') : chalk.red('✗');
      console.log(`  ${sym} ${c.name} ${c.detail ? chalk.dim(`(${c.detail})`) : ''}`);
    }
    console.log('');
    if (allGood) {
      console.log(`  ${chalk.green('✓ All checks passed — RTB is healthy!\n')}`);
    }
  }
}

async function handleAgent(ctx: CliContext): Promise<void> {
  const orchestrator = new AgentOrchestrator();
  const installed = orchestrator.listAgents();

  const agentChoice = await prompts.select({
    message: 'Select AI agent to launch:',
    options: installed.map((a) => ({
      value: a.command,
      label: a.name,
      hint: a.installed ? 'installed' : 'not found',
    })),
  });

  if (prompts.isCancel(agentChoice)) {
    prompts.outro('Action cancelled.');
    return;
  }

  const projects = getProjectNames(ctx.config);
  if (projects.length === 0) {
    prompts.log.warn('No managed projects found in workspace.');
    prompts.outro('Completed.');
    return;
  }

  const chosenProj = await prompts.select({
    message: `Select project for ${agentChoice}:`,
    options: projects.map((p) => ({ value: p, label: p })),
  });

  if (prompts.isCancel(chosenProj)) {
    prompts.outro('Action cancelled.');
    return;
  }

  prompts.outro(`Launching ${agentChoice} for ${chosenProj}...`);
  await orchestrator.orchestrate({
    projectName: chosenProj as string,
    agent: agentChoice as string,
    config: ctx.config,
    launch: true,
  });
}

async function handleConfig(ctx: CliContext): Promise<void> {
  const configPath = ctx.configPath || getStandardConfigPath();

  const cfgAction = await prompts.select({
    message: `Configuration (${path.basename(configPath)}):`,
    options: [
      { value: 'view', label: '👀 View Configuration', hint: 'Display registered project roots' },
      { value: 'open', label: '📝 Open in Editor', hint: 'Open configuration file in default editor' },
    ],
  });

  if (prompts.isCancel(cfgAction)) {
    prompts.outro('Action cancelled.');
    return;
  }

  if (cfgAction === 'open') {
    openInEditor(configPath);
    prompts.outro(`Opened ${configPath} in editor.`);
  } else {
    console.log(`\n  ${chalk.cyan('Configuration File:')} ${configPath}`);
    if (ctx.config?.projectRoots) {
      console.log(`\n  ${chalk.bold('Project Roots:')}`);
      for (const [key, entry] of Object.entries(ctx.config.projectRoots)) {
        console.log(`    ${entry.emoji || '📁'} ${chalk.bold((entry.label || key).padEnd(20))} ${chalk.gray(entry.path)}`);
      }
    }
    console.log('');
    prompts.outro('Configuration displayed.');
  }
}

export async function runMenuAction(actionKey: string | symbol, ctx: CliContext): Promise<void> {
  if (prompts.isCancel(actionKey) || actionKey === 'exit') {
    prompts.outro('Goodbye!');
    return;
  }

  switch (actionKey) {
    case 'run_build_test':
      await handleRunBuildTest(ctx);
      break;
    case 'goto':
      await handleGoto(ctx);
      break;
    case 'ui':
      await handleUi(ctx);
      break;
    case 'health_doctor':
      await handleHealthDoctor(ctx);
      break;
    case 'agent':
      await handleAgent(ctx);
      break;
    case 'config':
      await handleConfig(ctx);
      break;
    default:
      prompts.outro('Goodbye!');
      break;
  }
}

export function registerMenuCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('menu')
    .description('Interactive quick-action command launcher')
    .action(async () => {
      const ctx = getContext();

      if (!ctx.isInteractive || ctx.isJson) {
        if (ctx.isJson) {
          outputJson({
            status: 'interactive_only',
            message: 'rtb menu requires an interactive terminal',
          });
        } else {
          console.log('rtb menu requires an interactive terminal.');
        }
        return;
      }

      prompts.intro(
        chalk.bold.hex('#FFD700')('rtb interactive menu') +
          chalk.dim(' — Arrow keys to navigate, Enter to select')
      );

      const action = await prompts.select({
        message: 'What would you like to do?',
        options: MENU_ACTIONS.map((a) => ({
          value: a.value,
          label: a.label,
          hint: a.hint,
        })),
      });

      await runMenuAction(action, ctx);
    });
}
