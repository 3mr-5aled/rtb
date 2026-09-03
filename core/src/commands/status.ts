import type { Command } from 'commander';
import chalk from 'chalk';
import path from 'node:path';
import fs from 'node:fs';
import { execSync } from 'node:child_process';
import type { CliContext } from '../types/context.js';
import { inspectProject } from '../inspector/inspector.js';
import { outputJson } from '../utils/output.js';

export interface WorkspaceStatus {
  cwd: string;
  inWorkspace: boolean;
  project?: {
    name: string;
    path: string;
    rootCategory: string;
    stack: string[];
    git: {
      branch: string;
      uncommitted: number;
    } | null;
  };
}

export function detectWorkspaceStatus(cwd: string, ctx: CliContext): WorkspaceStatus {
  let inWorkspace = false;
  let matchedProject: WorkspaceStatus['project'];

  if (ctx.config?.projectRoots) {
    for (const [key, entry] of Object.entries(ctx.config.projectRoots)) {
      if (!entry.path) continue;
      const rootPath = path.resolve(entry.path);
      const normalizedCwd = path.resolve(cwd);

      if (normalizedCwd.toLowerCase().startsWith(rootPath.toLowerCase())) {
        inWorkspace = true;
        const relative = path.relative(rootPath, normalizedCwd);
        const parts = relative.split(path.sep).filter(Boolean);
        if (parts.length > 0) {
          const projName = parts[0];
          const projDir = path.join(rootPath, projName);
          const details = inspectProject(projDir);

          let branch = 'unknown';
          let uncommitted = 0;
          if (fs.existsSync(path.join(projDir, '.git'))) {
            try {
              branch = execSync('git branch --show-current', {
                cwd: projDir,
                stdio: ['ignore', 'pipe', 'ignore'],
              }).toString().trim() || 'HEAD';
            } catch {}
            try {
              const statusRaw = execSync('git status --porcelain', {
                cwd: projDir,
                stdio: ['ignore', 'pipe', 'ignore'],
              }).toString().trim();
              uncommitted = statusRaw ? statusRaw.split('\n').filter(Boolean).length : 0;
            } catch {}
          }

          matchedProject = {
            name: projName,
            path: projDir,
            rootCategory: entry.label || key,
            stack: details?.stack.filter((s) => s !== '-') || [],
            git: fs.existsSync(path.join(projDir, '.git')) ? { branch, uncommitted } : null,
          };
        }
        break;
      }
    }
  }

  return {
    cwd,
    inWorkspace,
    project: matchedProject,
  };
}

export function registerStatusCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('status')
    .description('Display workspace and git status for current location')
    .action(() => {
      const ctx = getContext();
      const status = detectWorkspaceStatus(process.cwd(), ctx);

      if (ctx.isJson) {
        outputJson(status);
        return;
      }

      console.log(`\n${chalk.cyan('══════════════════════════════════════════')}`);
      console.log(`  ${chalk.bold('rtb (رتّب) » Workspace Status')}`);
      console.log(`${chalk.cyan('══════════════════════════════════════════')}\n`);

      console.log(`  Current Directory: ${chalk.gray(status.cwd)}`);

      if (status.project) {
        console.log(`  Project:           ${chalk.white.bold(status.project.name)}`);
        console.log(`  Category:          ${chalk.cyan(status.project.rootCategory)}`);
        if (status.project.stack.length > 0) {
          console.log(`  Detected Stack:    ${chalk.yellow(status.project.stack.join(', '))}`);
        }
        if (status.project.git) {
          const dirtyStr = status.project.git.uncommitted > 0
            ? chalk.yellow(`${status.project.git.uncommitted} uncommitted changes`)
            : chalk.green('clean');
          console.log(`  Git Branch:        ${chalk.magenta(status.project.git.branch)} (${dirtyStr})`);
        }
      } else if (status.inWorkspace) {
        console.log(`  Status:            Inside workspace root (outside individual project)`);
      } else {
        console.log(`  Status:            ${chalk.gray('Outside RTB workspace roots')}`);
      }
      console.log('');
    });
}
