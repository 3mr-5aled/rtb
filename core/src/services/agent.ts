import path from 'node:path';
import { spawn } from 'node:child_process';
import chalk from 'chalk';
import type { RtbConfig } from '../types/config.js';
import { getInstalledAgents, type AgentDefinition } from '../agent/discovery.js';
import { generateAgentContextFile } from '../agent/context.js';
import { resolveProjectTarget } from '../navigation/fuzzy.js';
import { inspectProject } from '../inspector/inspector.js';
import { RtbError, ProjectNotFoundError } from '../errors.js';

export function launchAgentProcess(agent: AgentDefinition, projectPath: string): Promise<number> {
  return new Promise((resolve) => {
    const isWindows = process.platform === 'win32';
    const child = isWindows
      ? spawn(agent.command, {
          cwd: projectPath,
          stdio: 'inherit',
          shell: true,
        })
      : spawn(agent.command, [], {
          cwd: projectPath,
          stdio: 'inherit',
          shell: false,
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

export interface OrchestrateOptions {
  projectName?: string;
  projectPath?: string;
  agent?: string;
  config?: RtbConfig | null;
  launch?: boolean;
  quiet?: boolean;
}

export interface OrchestrateResult {
  agent: AgentDefinition;
  projectName: string;
  projectPath: string;
  contextPath: string;
  launched: boolean;
  exitCode?: number;
}

export class AgentOrchestrator {
  public listAgents(): AgentDefinition[] {
    return getInstalledAgents();
  }

  public resolveAgent(targetAgentName?: string, allowFallback = true): AgentDefinition {
    const installedAgents = this.listAgents();

    if (targetAgentName) {
      const q = targetAgentName.toLowerCase();
      const match = installedAgents.find(
        (a) => a.command.toLowerCase() === q || a.name.toLowerCase().includes(q)
      );
      if (!match) {
        throw new RtbError(`Specified agent '${targetAgentName}' is not recognized.`, 'AGENT_UNKNOWN');
      }
      if (!match.installed) {
        throw new RtbError(
          `Agent '${match.name}' (${match.command}) is not installed or not in PATH.`,
          'AGENT_NOT_INSTALLED'
        );
      }
      return match;
    }

    // Default: prefer 'agy' if installed, otherwise first available installed agent
    let selectedAgent = installedAgents.find((a) => a.command === 'agy' && a.installed);
    if (!selectedAgent) {
      selectedAgent = installedAgents.find((a) => a.installed);
    }

    if (!selectedAgent && allowFallback) {
      selectedAgent = installedAgents[0];
    }

    if (!selectedAgent) {
      throw new RtbError(
        'No installed AI agent found in PATH (agy, claude, gemini, codex, cursor, windsurf, aider, openhands).',
        'NO_AGENTS'
      );
    }

    return selectedAgent;
  }

  public async orchestrate(options: OrchestrateOptions): Promise<OrchestrateResult> {
    let targetPath = options.projectPath;
    let targetName = options.projectName;

    if (!targetPath) {
      const resolved = resolveProjectTarget(options.projectName, options.config || null);
      if (!resolved) {
        throw new ProjectNotFoundError(`Project or path '${options.projectName}' not found.`);
      }
      targetPath = resolved.targetPath;
      targetName = resolved.targetName;
    } else if (!targetName) {
      targetName = path.basename(targetPath);
    }

    const shouldLaunch = options.launch !== false;
    const agent = this.resolveAgent(options.agent, !shouldLaunch);

    const details = inspectProject(targetPath);
    const contextPath = generateAgentContextFile(targetPath, details);

    let exitCode = 0;
    if (shouldLaunch) {
      if (!options.quiet) {
        console.log(`\n${chalk.cyan('══════════════════════════════════════════')}`);
        console.log(`  ${chalk.bold(`rtb (ﺐﺗر) » Launching AI Agent: ${agent.name} (${agent.command})`)}`);
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
        console.log(`\n  Launching process '${chalk.green(agent.command)}' in ${chalk.gray(targetPath)}...\n`);
      }

      exitCode = await launchAgentProcess(agent, targetPath);
    }

    return {
      agent,
      projectName: targetName,
      projectPath: targetPath,
      contextPath,
      launched: shouldLaunch,
      exitCode,
    };
  }
}
