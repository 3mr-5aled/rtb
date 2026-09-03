import type { Command } from 'commander';
import chalk from 'chalk';
import path from 'node:path';
import fs from 'node:fs';
import os from 'node:os';
import { spawn, execSync } from 'node:child_process';
import type { CliContext } from '../types/context.js';
import { outputJson, outputError } from '../utils/output.js';
import { getStandardConfigPath } from '../config/loader.js';

export function openInEditor(filePath: string): void {
  const editor = process.env.EDITOR || process.env.VISUAL;
  if (editor) {
    try {
      const child = spawn(editor, [filePath], {
        detached: true,
        stdio: 'ignore',
        shell: process.platform === 'win32',
      });
      child.unref();
      return;
    } catch {}
  }

  if (process.platform === 'win32') {
    try {
      execSync('where code', { stdio: 'ignore' });
      const child = spawn('cmd.exe', ['/c', 'code', filePath], {
        detached: true,
        stdio: 'ignore',
      });
      child.unref();
      return;
    } catch {}

    try {
      const child = spawn('notepad.exe', [filePath], {
        detached: true,
        stdio: 'ignore',
      });
      child.unref();
      return;
    } catch {}

    try {
      const child = spawn('cmd.exe', ['/c', 'start', '', filePath], {
        detached: true,
        stdio: 'ignore',
      });
      child.unref();
      return;
    } catch {}
  }

  if (process.platform === 'darwin') {
    try {
      const child = spawn('open', [filePath], { detached: true, stdio: 'ignore' });
      child.unref();
      return;
    } catch {}
  }

  try {
    const child = spawn('xdg-open', [filePath], { detached: true, stdio: 'ignore' });
    child.unref();
  } catch {}
}

function ensureConfigFile(configPath: string): void {
  if (!fs.existsSync(configPath)) {
    const dir = path.dirname(configPath);
    if (!fs.existsSync(dir)) {
      fs.mkdirSync(dir, { recursive: true });
    }
    const defaultStructure = {
      version: '1.0.0',
      projectRoots: {
        active: { path: '', label: 'Active', emoji: '📁' },
        paused: { path: '', label: 'Paused', emoji: '⏸️' },
        production: { path: '', label: 'Production', emoji: '🚀' },
      },
    };
    fs.writeFileSync(configPath, JSON.stringify(defaultStructure, null, 2), 'utf8');
  }
}

export function registerConfigCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('config')
    .description('Open or view RTB configuration')
    .option('--path', 'Display the path to the active configuration file')
    .option('--view, --show', 'Display configuration in terminal instead of opening in editor')
    .action((options: { path?: boolean; view?: boolean; show?: boolean }) => {
      const ctx = getContext();
      const targetPath = ctx.configPath || getStandardConfigPath();

      if (options.path) {
        if (ctx.isJson) {
          outputJson({
            configPath: targetPath,
            exists: Boolean(ctx.config),
            isConfigured: ctx.isConfigured,
          });
        } else {
          console.log(targetPath);
        }
        return;
      }

      if (options.view || options.show) {
        if (!ctx.config) {
          outputError(`No valid configuration found at: ${targetPath}`, 'CONFIG_NOT_FOUND', ctx.isJson);
          process.exitCode = 1;
          return;
        }

        if (ctx.isJson) {
          outputJson(ctx.config);
        } else {
          console.log(`\n${chalk.bold('RTB Configuration')} (${chalk.gray(targetPath)}):`);
          console.log(`  Version: ${chalk.cyan(ctx.config.version)}`);
          console.log(`  Project Roots:`);
          for (const [key, entry] of Object.entries(ctx.config.projectRoots)) {
            console.log(`    ${entry.emoji} ${chalk.bold(entry.label)} (${chalk.yellow(key)}): ${chalk.gray(entry.path)}`);
          }
          if (ctx.config.backupRoot) {
            console.log(`  Backup Root: ${chalk.gray(ctx.config.backupRoot)}`);
          }
          console.log('');
        }
        return;
      }

      // If --json was specified without --view or --path, still output JSON
      if (ctx.isJson) {
        if (!ctx.config) {
          outputError(`No valid configuration found at: ${targetPath}`, 'CONFIG_NOT_FOUND', true);
          process.exitCode = 1;
          return;
        }
        outputJson(ctx.config);
        return;
      }

      // Default behavior: open config file in editor
      ensureConfigFile(targetPath);
      console.log(chalk.cyan('Opening RTB configuration...'));
      console.log(`  Config file: ${chalk.white(targetPath)}`);
      openInEditor(targetPath);
    });
}
