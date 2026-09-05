import type { Command } from 'commander';
import chalk from 'chalk';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import * as p from '@clack/prompts';
import type { CliContext } from '../types/context.js';
import { getStandardConfigDir, getStandardConfigPath } from '../config/loader.js';
import { outputJson } from '../utils/output.js';
import { getLogo } from '../utils/logo.js';
import { detectCurrentShell } from './shell-init.js';
import type { RtbConfig } from '../types/config.js';

export const prompts = {
  intro: p.intro,
  outro: p.outro,
  select: p.select,
  multiselect: p.multiselect,
  confirm: p.confirm,
  text: p.text,
  cancel: p.cancel,
  isCancel: p.isCancel,
  log: p.log,
};

export interface LifecycleOption {
  key: string;
  label: string;
  hint: string;
  dirName: string;
  folderLabel: string;
  emoji: string;
}

export const LIFECYCLE_OPTIONS: LifecycleOption[] = [
  {
    key: 'active',
    label: '01-Active 🟢',
    hint: 'In-flight daily development',
    dirName: '01-Active',
    folderLabel: 'Active Projects',
    emoji: '🟢',
  },
  {
    key: 'planning',
    label: '02-Planning 📝',
    hint: 'Specs, RFCs, roadmaps',
    dirName: '02-Planning',
    folderLabel: 'Planning Projects',
    emoji: '📝',
  },
  {
    key: 'testing',
    label: '03-Testing 🧪',
    hint: 'Spikes, benchmarks, test suites',
    dirName: '03-Testing',
    folderLabel: 'Testing Projects',
    emoji: '🧪',
  },
  {
    key: 'paused',
    label: '04-Paused ⏸️',
    hint: 'Dormant projects temporarily on hold',
    dirName: '04-Paused',
    folderLabel: 'Paused Projects',
    emoji: '⏸️',
  },
  {
    key: 'abandoned',
    label: '05-Abandoned 🗑️',
    hint: 'Discarded experiments and deprecated code',
    dirName: '05-Abandoned',
    folderLabel: 'Abandoned Projects',
    emoji: '🗑️',
  },
  {
    key: 'production',
    label: '06-Production 🚀',
    hint: 'Deployed live systems',
    dirName: '06-Production',
    folderLabel: 'Production Projects',
    emoji: '🚀',
  },
  {
    key: 'staging',
    label: '07-Staging 🪜',
    hint: 'Pre-release staging environments',
    dirName: '07-Staging',
    folderLabel: 'Staging Projects',
    emoji: '🪜',
  },
  {
    key: 'vibe',
    label: '08-Vibe 🤖',
    hint: 'Agentic and exploratory coding',
    dirName: '08-Vibe',
    folderLabel: 'Vibe Coding',
    emoji: '🤖',
  },
];

export interface ShellIntegrationResult {
  success: boolean;
  profilePath?: string;
  message: string;
  snippet: string;
}

export function getShellProfilePath(shell: string): string | null {
  const norm = shell.trim().toLowerCase();
  const home = os.homedir();

  switch (norm) {
    case 'pwsh':
    case 'powershell':
    case 'posh': {
      if (process.env.PROFILE && fs.existsSync(process.env.PROFILE)) {
        return process.env.PROFILE;
      }
      if (process.platform === 'win32') {
        const candidates = [
          path.join(home, 'Documents', 'PowerShell', 'Microsoft.PowerShell_profile.ps1'),
          path.join(home, 'OneDrive', 'Documents', 'PowerShell', 'Microsoft.PowerShell_profile.ps1'),
          path.join(home, 'Documents', 'WindowsPowerShell', 'Microsoft.PowerShell_profile.ps1'),
        ];
        for (const cand of candidates) {
          if (fs.existsSync(cand)) return cand;
        }
        return path.join(home, 'Documents', 'PowerShell', 'Microsoft.PowerShell_profile.ps1');
      } else {
        return path.join(home, '.config', 'powershell', 'Microsoft.PowerShell_profile.ps1');
      }
    }
    case 'bash':
      return path.join(home, '.bashrc');
    case 'zsh':
      return path.join(home, '.zshrc');
    case 'fish':
      return path.join(home, '.config', 'fish', 'config.fish');
    default:
      return null;
  }
}

export function getShellIntegrationSnippet(shell: string): string {
  const norm = shell.trim().toLowerCase();
  switch (norm) {
    case 'pwsh':
    case 'powershell':
    case 'posh':
      return '(& rtb shell-init pwsh | Out-String) | Invoke-Expression';
    case 'zsh':
      return 'eval "$(rtb shell-init zsh)"';
    case 'fish':
      return 'rtb shell-init fish | source';
    case 'bash':
    default:
      return 'eval "$(rtb shell-init bash)"';
  }
}

export function configureShellIntegration(
  shell: string = detectCurrentShell(),
  customProfilePath?: string
): ShellIntegrationResult {
  const profilePath = customProfilePath || getShellProfilePath(shell);
  const snippet = getShellIntegrationSnippet(shell);

  if (!profilePath) {
    return {
      success: false,
      message: `Unknown shell profile location for shell: ${shell}`,
      snippet,
    };
  }

  try {
    if (fs.existsSync(profilePath)) {
      const content = fs.readFileSync(profilePath, 'utf8');
      if (content.includes('rtb shell-init')) {
        return {
          success: true,
          profilePath,
          message: 'Shell integration already configured in profile.',
          snippet,
        };
      }
    } else {
      fs.mkdirSync(path.dirname(profilePath), { recursive: true });
    }

    const prefix = fs.existsSync(profilePath) ? '\n' : '';
    fs.appendFileSync(
      profilePath,
      `${prefix}# RTB shell integration\n${snippet}\n`,
      'utf8'
    );

    return {
      success: true,
      profilePath,
      message: `Successfully configured shell integration in ${profilePath}`,
      snippet,
    };
  } catch (err: any) {
    return {
      success: false,
      profilePath,
      message: err?.message || String(err),
      snippet,
    };
  }
}

export function registerInitCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('init')
    .description('Initialize and configure your RTB workspace')
    .option('-f, --force', 'Overwrite existing configuration', false)
    .option('-r, --root <path>', 'Custom workspace root directory')
    .option('--flat', 'Use flat workspace structure instead of lifecycle folders', false)
    .action(async (options: { force?: boolean; root?: string; flat?: boolean }) => {
      const ctx = getContext();
      const configDir = getStandardConfigDir();
      const configFile = getStandardConfigPath();

      // Check existing config
      if (fs.existsSync(configFile) && !options.force) {
        if (ctx.isJson) {
          outputJson({ status: 'already_configured', configPath: configFile });
          return;
        }

        if (ctx.isInteractive) {
          const overwrite = await prompts.confirm({
            message: `Configuration already exists at ${configFile}. Overwrite?`,
            initialValue: false,
          });
          if (prompts.isCancel(overwrite) || !overwrite) {
            prompts.cancel('Setup cancelled.');
            return;
          }
        } else {
          console.log('');
          console.log(`  ${chalk.yellow('⚠')}  Configuration already exists at:`);
          console.log(`     ${chalk.white(configFile)}`);
          console.log(`     Run '${chalk.cyan('rtb config')}' to view or edit.`);
          console.log(`     Use '${chalk.gray('rtb init --force')}' to overwrite.\n`);
          return;
        }
      }

      const homeDir = os.homedir();
      let chosenRoot = options.root;

      // Detect candidate paths
      const candidateRoots = [
        path.join(homeDir, 'Projects'),
        path.join(homeDir, 'dev'),
        path.join(homeDir, 'code'),
        path.join(homeDir, 'repos'),
        path.join(homeDir, 'workspace'),
        'D:\\02-Projects',
        'D:\\Projects',
      ];
      const existing = candidateRoots.filter((targetPath) => {
        try {
          return fs.existsSync(targetPath);
        } catch {
          return false;
        }
      });

      // Step 1: Welcome & Brand Intro (Interactive only)
      if (ctx.isInteractive && !ctx.isJson && !ctx.isQuiet) {
        const logo = getLogo({ color: true });
        if (logo) {
          console.log(logo);
        }
        prompts.intro(chalk.bold.hex('#FFD700')('rtb workspace setup') + chalk.dim(' — Next-gen project orchestrator'));
      }

      // Step 2: Workspace Root Selection
      if (!chosenRoot) {
        if (ctx.isInteractive) {
          const selectOptions: Array<{ value: string; label: string; hint?: string }> = existing.map((p) => ({
            value: p,
            label: p,
            hint: 'detected',
          }));

          if (selectOptions.length === 0) {
            selectOptions.push({
              value: path.join(homeDir, 'Projects'),
              label: path.join(homeDir, 'Projects'),
              hint: 'default',
            });
          }

          selectOptions.push({
            value: '__custom__',
            label: 'Custom path...',
            hint: 'Enter an absolute path',
          });

          const rootAnswer = await prompts.select({
            message: 'Where do you want to keep and manage your projects?',
            options: selectOptions,
          });

          if (prompts.isCancel(rootAnswer)) {
            prompts.cancel('Setup cancelled.');
            return;
          }

          if (rootAnswer === '__custom__') {
            const customPath = await prompts.text({
              message: 'Enter workspace root path:',
              placeholder: path.join(homeDir, 'Projects'),
              validate(val) {
                if (!val || val.trim().length === 0) return 'Path cannot be empty';
              },
            });

            if (prompts.isCancel(customPath)) {
              prompts.cancel('Setup cancelled.');
              return;
            }
            chosenRoot = path.resolve(customPath.trim());
          } else {
            chosenRoot = path.resolve(rootAnswer as string);
          }
        } else {
          chosenRoot = existing.length > 0 ? existing[0] : path.join(homeDir, 'Projects');
        }
      }

      if (!chosenRoot) {
        chosenRoot = path.join(homeDir, 'Projects');
      }
      chosenRoot = path.resolve(chosenRoot);

      // Step 3: Lifecycle Folder Scaffolding
      let projectRoots: RtbConfig['projectRoots'] = {};

      if (options.flat) {
        projectRoots = {
          projects: {
            path: chosenRoot,
            label: 'Projects',
            emoji: '📁',
          },
          active: {
            path: chosenRoot,
            label: 'Projects',
            emoji: '📁',
          },
        };
      } else if (ctx.isInteractive) {
        const selected = await prompts.multiselect({
          message: 'Select project lifecycle folders to scaffold:',
          options: LIFECYCLE_OPTIONS.map((opt) => ({
            value: opt.key,
            label: opt.label,
            hint: opt.hint,
          })),
          initialValues: ['active', 'paused'],
          required: false,
        });

        if (prompts.isCancel(selected)) {
          prompts.cancel('Setup cancelled.');
          return;
        }

        const selectedList = selected as string[];
        const finalKeys = Array.from(new Set(['active', ...selectedList]));
        projectRoots = {};

        for (const k of finalKeys) {
          const item = LIFECYCLE_OPTIONS.find((o) => o.key === k);
          if (item) {
            const folderPath = path.join(chosenRoot, item.dirName);
            fs.mkdirSync(folderPath, { recursive: true });
            projectRoots[item.key] = {
              path: folderPath,
              label: item.folderLabel,
              emoji: item.emoji,
            };
          }
        }
      } else {
        const activeDir = path.join(chosenRoot, '01-Active');
        const pausedDir = path.join(chosenRoot, '04-Paused');
        const archiveDir = path.join(chosenRoot, '03-Archive');

        fs.mkdirSync(activeDir, { recursive: true });
        fs.mkdirSync(pausedDir, { recursive: true });
        fs.mkdirSync(archiveDir, { recursive: true });

        projectRoots = {
          active: {
            path: activeDir,
            label: 'Active Projects',
            emoji: '⚡',
          },
          paused: {
            path: pausedDir,
            label: 'Paused Projects',
            emoji: '⏸️',
          },
          archive: {
            path: archiveDir,
            label: 'Archived Projects',
            emoji: '📦',
          },
        };
      }

      // Step 4: Shell Integration Hook (Interactive only)
      if (ctx.isInteractive && !ctx.isJson) {
        const shell = detectCurrentShell();
        const shouldConfigure = await prompts.confirm({
          message: `Configure shell integration hook for ${shell}?`,
          initialValue: true,
        });

        if (prompts.isCancel(shouldConfigure)) {
          prompts.cancel('Setup cancelled.');
          return;
        }

        if (shouldConfigure) {
          const shellResult = configureShellIntegration(shell);
          if (shellResult.success) {
            prompts.log.success(chalk.green(`Shell integration configured: ${shellResult.profilePath}`));
          } else {
            prompts.log.warn(chalk.yellow(`Could not auto-configure shell: ${shellResult.message}`));
            prompts.log.info(chalk.dim(`Add manually:\n${shellResult.snippet}`));
          }
        }
      }

      // Step 5: Completion Outro
      const newConfig: RtbConfig = {
        version: '1.0',
        projectRoots,
        backupRoot: path.join(chosenRoot, 'Backups'),
        cleanDeps: {
          daysInactive: 14,
          targets: ['node_modules', '.venv', 'target', 'dist'],
        },
        gitHealth: {
          scanRoots: ['active'],
        },
      };

      fs.mkdirSync(configDir, { recursive: true });
      fs.writeFileSync(configFile, JSON.stringify(newConfig, null, 2) + '\n', 'utf-8');

      if (ctx.isJson) {
        outputJson({ status: 'success', configPath: configFile, config: newConfig });
        return;
      }

      if (ctx.isInteractive) {
        prompts.outro(chalk.bold.green('✨ Workspace initialized successfully!'));
      } else {
        console.log(`\n  ${chalk.green('✔')} ${chalk.bold('RTB workspace successfully initialized!')}`);
      }

      console.log(`  ${chalk.cyan('Config:')}  ${configFile}`);
      console.log(`  ${chalk.cyan('Root:')}    ${chosenRoot}`);
      console.log(`\n  ${chalk.bold('Next steps:')}`);
      console.log(`    ${chalk.green('rtb list')}        - list registered projects`);
      console.log(`    ${chalk.green('rtb new <name>')}  - scaffold a new project`);
      console.log(`    ${chalk.green('rtb doctor')}      - verify toolchain health`);
      console.log(`    ${chalk.green('rtb help')}        - view all available commands\n`);
    });
}
