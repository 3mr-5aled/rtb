import type { Command } from 'commander';
import chalk from 'chalk';
import path from 'node:path';
import fs from 'node:fs';
import { spawn } from 'node:child_process';
import type { CliContext } from '../types/context.js';
import { getInstalledAgents, type AgentDefinition } from '../agent/discovery.js';
import { generateAgentContextFile } from '../agent/context.js';
import { findProjectPathFuzzy } from '../navigation/fuzzy.js';
import { inspectProject } from '../inspector/inspector.js';
import { outputError, outputJson } from '../utils/output.js';

export function launchAgentProcess(agent: AgentDefinition, projectPath: string): Promise<number> {
  return new Promise((resolve) => {
    const isWindows = process.platform === 'win32';
    // Use shell on Windows for resolving .cmd, .bat, or path lookup cleanly
    const child = spawn(agent.command, [], {
      cwd: projectPath,
      stdio: 'inherit',
      shell: isWindows,
    });

    child.on('error', (err) => {
      console.error(`Failed to launch agent ${agent.command}: ${err.message}`);
      resolve(1);
    });

    child.on('close', (code) => {
      resolve(code ?? 0);
    });
  });
}

export function registerAgentCommand(program: Command, getContext: () => CliContext): void {
  const handleAgent = async (
    projectName: string | undefined,
    agentArg: string | undefined,
    options: {
      list?: boolean;
      agy?: boolean;
      claude?: boolean;
      gemini?: boolean;
      codex?: boolean;
      cursor?: boolean;
      windsurf?: boolean;
      aider?: boolean;
      openhands?: boolean;
      launch?: boolean;
    }
  ) => {
    const ctx = getContext();
    const installedAgents = getInstalledAgents();

    if (options.list) {
      if (ctx.isJson) {
        outputJson(installedAgents);
        return;
      }

      console.log(`\n${chalk.cyan('══════════════════════════════════════════')}`);
      console.log(`  ${chalk.bold('rtb (رتّب) » Installed AI Agents')}`);
      console.log(`${chalk.cyan('══════════════════════════════════════════')}\n`);

      for (const a of installedAgents) {
        const statusStr = a.installed ? chalk.green('[Installed]') : chalk.gray('[Not Found]');
        console.log(`  ${a.name.padEnd(22)} (${chalk.yellow(a.command.padEnd(10))}) ${statusStr}`);
      }
      console.log('');
      return;
    }

    // Determine target agent
    let targetAgentName = agentArg;
    if (!targetAgentName) {
      if (options.agy) targetAgentName = 'agy';
      else if (options.claude) targetAgentName = 'claude';
      else if (options.gemini) targetAgentName = 'gemini';
      else if (options.codex) targetAgentName = 'codex';
      else if (options.cursor) targetAgentName = 'cursor';
      else if (options.windsurf) targetAgentName = 'windsurf';
      else if (options.aider) targetAgentName = 'aider';
      else if (options.openhands) targetAgentName = 'openhands';
    }

    let selectedAgent: AgentDefinition | undefined;
    if (targetAgentName) {
      const q = targetAgentName.toLowerCase();
      selectedAgent = installedAgents.find(
        (a) => a.command.toLowerCase() === q || a.name.toLowerCase().includes(q)
      );
      if (!selectedAgent) {
        outputError(`Specified agent '${targetAgentName}' is not recognized.`, 'AGENT_UNKNOWN', ctx.isJson);
        process.exitCode = 1;
        return;
      }
      if (!selectedAgent.installed) {
        outputError(`Agent '${selectedAgent.name}' (${selectedAgent.command}) is not installed or not in PATH.`, 'AGENT_NOT_INSTALLED', ctx.isJson);
        process.exitCode = 1;
        return;
      }
    } else {
      // Default: prefer 'agy' if installed, otherwise first available installed agent
      selectedAgent = installedAgents.find((a) => a.command === 'agy' && a.installed);
      if (!selectedAgent) {
        selectedAgent = installedAgents.find((a) => a.installed);
      }
    }

    if (!selectedAgent || !selectedAgent.installed) {
      outputError('No installed AI agent found in PATH (agy, claude, gemini, codex, cursor, windsurf, aider, openhands).', 'NO_AGENTS', ctx.isJson);
      console.error(chalk.gray("  Run 'rtb agent --list' to check agent status.\n"));
      process.exitCode = 1;
      return;
    }

    // Resolve target project path
    let targetPath = process.cwd();
    let targetName = path.basename(targetPath);

    if (projectName) {
      if (ctx.config) {
        const matches = findProjectPathFuzzy(projectName, ctx.config);
        if (matches.length > 0) {
          targetPath = matches[0].path;
          targetName = matches[0].name;
        } else if (fs.existsSync(projectName)) {
          targetPath = path.resolve(projectName);
          targetName = path.basename(targetPath);
        } else {
          outputError(`Project or path '${projectName}' not found.`, 'NOT_FOUND', ctx.isJson);
          process.exitCode = 1;
          return;
        }
      } else if (fs.existsSync(projectName)) {
        targetPath = path.resolve(projectName);
        targetName = path.basename(targetPath);
      }
    }

    // Generate .rtb_context.md
    const details = inspectProject(targetPath);
    const contextPath = generateAgentContextFile(targetPath, details);

    if (ctx.isJson) {
      outputJson({
        agent: selectedAgent,
        project: targetName,
        projectPath: targetPath,
        contextFile: contextPath,
        launched: options.launch !== false,
      });
      return;
    }

    console.log(`\n${chalk.cyan('══════════════════════════════════════════')}`);
    console.log(`  ${chalk.bold(`rtb (رتّب) » Launching AI Agent: ${selectedAgent.name} (${selectedAgent.command})`)}`);
    console.log(`${chalk.cyan('══════════════════════════════════════════')}\n`);

    console.log(`  Project Name:  ${chalk.white.bold(targetName)}`);
    console.log(`  Project Path:  ${chalk.gray(targetPath)}`);
    if (details?.stack) {
      console.log(`  Stack:         ${chalk.yellow(details.stack.join(', '))}`);
    }
    if (details?.git) {
      console.log(`  Git Branch:    ${chalk.cyan(details.git.branch)}`);
    }
    console.log(`  Context File:  ${chalk.cyan('.rtb_context.md')}`);
    console.log(`\n  Launching process '${chalk.green(selectedAgent.command)}' in ${chalk.gray(targetPath)}...\n`);

    if (options.launch !== false) {
      const exitCode = await launchAgentProcess(selectedAgent, targetPath);
      process.exitCode = exitCode;
    }
  };

  const agentCmd = program
    .command('agent [project] [agent]')
    .description('Discover installed AI agents, generate .rtb_context.md, and launch agent')
    .option('-l, --list', 'List installed AI agents and status')
    .option('--agy', 'Select Google Antigravity')
    .option('--claude', 'Select Claude Code')
    .option('--gemini', 'Select Gemini CLI')
    .option('--codex', 'Select Codex CLI')
    .option('--cursor', 'Select Cursor')
    .option('--windsurf', 'Select Windsurf')
    .option('--aider', 'Select Aider')
    .option('--openhands', 'Select OpenHands')
    .option('--no-launch', 'Generate .rtb_context.md without spawning agent process')
    .action((proj, agent, opts) => handleAgent(proj, agent, opts));

  // Shorthands: rtb agy [project], rtb claude [project], etc.
  const shorthands = [
    { name: 'agy', label: 'Google Antigravity' },
    { name: 'claude', label: 'Claude Code' },
    { name: 'gemini', label: 'Gemini CLI' },
    { name: 'codex', label: 'Codex CLI' },
    { name: 'cursor', label: 'Cursor' },
    { name: 'windsurf', label: 'Windsurf' },
    { name: 'aider', label: 'Aider' },
    { name: 'openhands', label: 'OpenHands' },
  ];

  for (const s of shorthands) {
    program
      .command(`${s.name} [project]`)
      .description(`Launch ${s.label} with generated .rtb_context.md`)
      .option('--no-launch', 'Generate context without launching')
      .action((proj, opts) => handleAgent(proj, s.name, opts));
  }
}
