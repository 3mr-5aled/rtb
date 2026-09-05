import type { Command } from 'commander';
import chalk from 'chalk';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { execSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import * as p from '@clack/prompts';
import type { CliContext } from '../types/context.js';
import { getStandardConfigDir, getStandardConfigPath } from '../config/loader.js';
import { outputJson } from '../utils/output.js';
import { getLogo } from '../utils/logo.js';
import { detectCurrentShell } from './shell-init.js';
import { findRtbtuiBinary, getDefaultUserBinDir } from './doctor.js';
import { provisionRtbtuiBinary } from './ui.js';
import { RTB_VERSION } from './version.js';
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
  spinner: p.spinner,
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
      return `$rtbBin = [System.IO.Path]::Combine($HOME, '.config', 'rtb', 'bin')
if ((Test-Path $rtbBin) -and ($env:PATH -notmatch [regex]::Escape($rtbBin))) {
    $env:PATH = "$rtbBin;$env:PATH"
}
if (Get-Command rtb -ErrorAction SilentlyContinue) {
    (& rtb shell-init pwsh | Out-String) | Invoke-Expression
}`;
    case 'zsh':
      return `if [ -d "$HOME/.config/rtb/bin" ] && [[ ":$PATH:" != *":$HOME/.config/rtb/bin:"* ]]; then
  export PATH="$HOME/.config/rtb/bin:$PATH"
fi
if command -v rtb >/dev/null 2>&1; then
  eval "$(rtb shell-init zsh)"
fi`;
    case 'fish':
      return `if test -d "$HOME/.config/rtb/bin"
  contains "$HOME/.config/rtb/bin" $PATH; or set -gx PATH "$HOME/.config/rtb/bin" $PATH
end
if command -v rtb >/dev/null 2>&1
  rtb shell-init fish | source
end`;
    case 'bash':
    default:
      return `if [ -d "$HOME/.config/rtb/bin" ] && [[ ":$PATH:" != *":$HOME/.config/rtb/bin:"* ]]; then
  export PATH="$HOME/.config/rtb/bin:$PATH"
fi
if command -v rtb >/dev/null 2>&1; then
  eval "$(rtb shell-init bash)"
fi`;
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
      let content = fs.readFileSync(profilePath, 'utf8');

      // Check if exact snippet or resilient pattern already exists
      if (content.includes('$rtbBin =') || content.includes('command -v rtb') || content.includes('contains "$HOME/.config/rtb/bin"')) {
        return {
          success: true,
          profilePath,
          message: 'Shell integration already configured in profile.',
          snippet,
        };
      }

      // Upgrade legacy bare hooks
      if (content.includes('rtb shell-init')) {
        const legacyPattern = /(#\s*RTB shell integration\s*)?(\(&\s*rtb\s+shell-init[^\n]+\)|eval\s*"\$\(rtb\s+shell-init[^\)]+\)"|rtb\s+shell-init[^\n]+)/g;
        content = content.replace(legacyPattern, '').trimEnd();
        const prefix = content.length > 0 ? '\n\n' : '';
        fs.writeFileSync(profilePath, `${content}${prefix}# RTB shell integration\n${snippet}\n`, 'utf8');
        return {
          success: true,
          profilePath,
          message: `Upgraded shell integration in ${profilePath}`,
          snippet,
        };
      }
    } else {
      fs.mkdirSync(path.dirname(profilePath), { recursive: true });
    }

    const existingContent = fs.existsSync(profilePath) ? fs.readFileSync(profilePath, 'utf8') : '';
    const prefix = existingContent.trim().length > 0 ? '\n\n' : '';
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

export interface DeployLauncherResult {
  success: boolean;
  binDir: string;
  launcherPath: string;
  pathUpdated: boolean;
  message?: string;
}

export function deployCliLauncher(customBinDir?: string): DeployLauncherResult {
  const binDir = customBinDir || getDefaultUserBinDir();
  const isWindows = process.platform === 'win32';

  try {
    if (!fs.existsSync(binDir)) {
      fs.mkdirSync(binDir, { recursive: true });
    }

    // 1. Locate current bundle
    let sourceBundle = fileURLToPath(import.meta.url);
    if (sourceBundle.endsWith('.ts')) {
      const builtDist = path.resolve(path.dirname(sourceBundle), '../../dist/index.js');
      if (fs.existsSync(builtDist)) {
        sourceBundle = builtDist;
      }
    }

    const destJs = path.join(binDir, 'rtb-cli.js');
    if (fs.existsSync(sourceBundle)) {
      fs.copyFileSync(sourceBundle, destJs);
    }
    // Clean up legacy rtb.js in binDir to prevent Windows PATHEXT collision
    const legacyJs = path.join(binDir, 'rtb.js');
    if (fs.existsSync(legacyJs)) {
      try {
        fs.unlinkSync(legacyJs);
      } catch {}
    }

    const versionDest = path.join(binDir, 'VERSION');
    try {
      fs.writeFileSync(versionDest, RTB_VERSION, 'utf8');
    } catch {}

    let launcherPath = destJs;

    // 2. Create launcher wrappers
    if (isWindows) {
      const cmdContent = `@echo off\r\nnode "%~dp0rtb-cli.js" %*\r\n`;
      fs.writeFileSync(path.join(binDir, 'rtb.cmd'), cmdContent, 'utf8');

      const ps1Content = `& node (Join-Path $PSScriptRoot 'rtb-cli.js') @args\r\n`;
      fs.writeFileSync(path.join(binDir, 'rtb.ps1'), ps1Content, 'utf8');

      launcherPath = path.join(binDir, 'rtb.cmd');
    } else {
      const shContent = `#!/usr/bin/env sh\nRTB_LIB_PATH="$(cd "$(dirname "$0")" && pwd)/rtb-cli.js"\nexec node "$RTB_LIB_PATH" "$@"\n`;
      const shPath = path.join(binDir, 'rtb');
      fs.writeFileSync(shPath, shContent, { mode: 0o755 });
      try {
        fs.chmodSync(shPath, 0o755);
      } catch {}
      launcherPath = shPath;
    }

    // 3. Configure PATH on Windows (skip during automated test runs)
    let pathUpdated = false;
    const isTest = Boolean(process.env.VITEST || process.env.NODE_ENV === 'test');
    if (isWindows && !isTest) {
      try {
        const psScript = `
          $target = '${binDir.replace(/'/g, "''")}';
          $cur = [Environment]::GetEnvironmentVariable('PATH', 'User');
          $parts = if ($cur) { $cur -split ';' | Where-Object { $_ -and $_.Trim() } } else { @() };
          if ($parts -notcontains $target) {
            $new = @($target) + $parts -join ';';
            [Environment]::SetEnvironmentVariable('PATH', $new, 'User');
            try {
              Add-Type -Namespace Win32 -Name NativeMethods -MemberDefinition @'
[DllImport("user32.dll", SetLastError = true, CharSet = CharSet.Auto)]
public static extern IntPtr SendMessageTimeout(IntPtr hWnd, uint Msg, UIntPtr wParam, string lParam, uint fuFlags, uint uTimeout, out UIntPtr lpdwResult);
'@ -ErrorAction SilentlyContinue
              [UIntPtr]$res = [UIntPtr]::Zero
              [Win32.NativeMethods]::SendMessageTimeout([IntPtr]0xffff, 0x001A, [UIntPtr]::Zero, "Environment", 2, 1000, [ref]$res) | Out-Null
            } catch {}
          }
        `.replace(/\r?\n\s*/g, ' ');

        execSync(`powershell.exe -NoProfile -NonInteractive -Command "${psScript}"`, {
          stdio: 'ignore',
          timeout: 5000,
        });
        pathUpdated = true;
      } catch {}
    } else {
      pathUpdated = true;
    }

    if (process.env.PATH) {
      const parts = process.env.PATH.split(path.delimiter);
      if (!parts.includes(binDir)) {
        process.env.PATH = `${binDir}${path.delimiter}${process.env.PATH}`;
      }
    }

    return {
      success: true,
      binDir,
      launcherPath,
      pathUpdated,
    };
  } catch (err: any) {
    return {
      success: false,
      binDir,
      launcherPath: '',
      pathUpdated: false,
      message: err?.message || String(err),
    };
  }
}

export function registerInitCommand(program: Command, getContext: () => CliContext): void {
  const initAction = async (options: {
    force?: boolean;
    root?: string;
    flat?: boolean;
    skipUi?: boolean;
    noUi?: boolean;
    ui?: boolean;
  }) => {
    const ctx = getContext();
    const configDir = getStandardConfigDir();
    const configFile = getStandardConfigPath();

    // Check existing config
    let shouldOverwriteConfig = true;
    let existingConfig: RtbConfig | null = null;

    if (fs.existsSync(configFile)) {
      try {
        existingConfig = JSON.parse(fs.readFileSync(configFile, 'utf8'));
      } catch {}

      if (!options.force) {
        if (ctx.isJson) {
          outputJson({ status: 'already_configured', configPath: configFile });
          return;
        }

        if (ctx.isInteractive) {
          const overwrite = await prompts.confirm({
            message: `Configuration already exists at ${configFile}. Overwrite settings?`,
            initialValue: false,
          });
          if (prompts.isCancel(overwrite)) {
            prompts.cancel('Setup cancelled.');
            return;
          }
          if (!overwrite) {
            shouldOverwriteConfig = false;
            prompts.log.info(chalk.dim('Keeping existing configuration. Proceeding with remaining installation steps...'));
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
    }

    const homeDir = os.homedir();
    let chosenRoot = options.root;

    if (!shouldOverwriteConfig && existingConfig) {
      if (existingConfig.projectRoots?.active?.path) {
        chosenRoot = path.dirname(existingConfig.projectRoots.active.path);
      } else if (existingConfig.projectRoots?.projects?.path) {
        chosenRoot = existingConfig.projectRoots.projects.path;
      } else if (existingConfig.backupRoot) {
        chosenRoot = path.dirname(existingConfig.backupRoot);
      }
    }

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

    let projectRoots: RtbConfig['projectRoots'] = existingConfig?.projectRoots || {};

    if (shouldOverwriteConfig) {
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
    } else {
      if (!chosenRoot) {
        chosenRoot = path.join(homeDir, 'Projects');
      }
      chosenRoot = path.resolve(chosenRoot);
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

    // Step 5: Terminal UI Binary (Interactive or explicit flags)
    const hasTui = Boolean(findRtbtuiBinary());
    const skipUi = Boolean(options.skipUi || options.noUi || process.env.RTB_SKIP_UI === '1' || process.env.RTB_SKIP_UI === 'true');
    let shouldInstallTui = Boolean(options.ui || process.env.RTB_INSTALL_UI === '1' || process.env.RTB_INSTALL_UI === 'true');

    if (!hasTui && !skipUi && !shouldInstallTui && ctx.isInteractive && !ctx.isJson) {
      const tuiChoice = await prompts.select({
        message: 'Download RTB Terminal UI (rtbtui) dashboard?',
        options: [
          { value: 'now', label: 'Download now (Recommended)', hint: 'Prebuilt native dashboard binary' },
          { value: 'later', label: 'Download later', hint: "Downloads automatically on first 'rtb ui' run" },
        ],
        initialValue: 'now',
      });

      if (!prompts.isCancel(tuiChoice) && tuiChoice === 'now') {
        shouldInstallTui = true;
      } else {
        prompts.log.info(chalk.dim("Skipped rtbtui download. Run 'rtb ui' anytime to download on demand."));
      }
    }

    if (shouldInstallTui && !hasTui) {
      const s = prompts.spinner();
      s.start('Downloading prebuilt rtbtui binary...');
      const dest = await provisionRtbtuiBinary();
      if (dest) {
        s.stop(chalk.green(`Installed rtbtui binary: ${dest}`));
      } else {
        s.stop(chalk.yellow('Download failed. You can download later via "rtb ui --download".'));
      }
    }

    // Step 6: Deploy CLI Launcher & Configure PATH
    const launcherRes = deployCliLauncher();

    // Step 7: Completion Outro
    if (shouldOverwriteConfig) {
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
    }

    const finalConfig = shouldOverwriteConfig
      ? (JSON.parse(fs.readFileSync(configFile, 'utf8')) as RtbConfig)
      : (existingConfig || { version: '1.0', projectRoots: {} });

    if (ctx.isJson) {
      outputJson({
        status: 'success',
        configPath: configFile,
        launcherPath: launcherRes.launcherPath,
        config: finalConfig,
      });
      return;
    }

    if (ctx.isInteractive) {
      prompts.outro(chalk.bold.green('✔ RTB installation setup complete!'));
    } else {
      console.log(`\n  ${chalk.green('✔')} ${chalk.bold('RTB installation setup complete!')}`);
    }

    console.log('');
    console.log(`  ${chalk.bold.hex('#FFD700')('rtb')} ${chalk.green('is now installed and ready for use!')}`);
    console.log('');
    console.log(`  ${chalk.cyan('Launcher:')}   ${launcherRes.launcherPath || launcherRes.binDir}`);
    console.log(`  ${chalk.cyan('Workspace:')}  ${chosenRoot}`);
    console.log(`  ${chalk.cyan('Config:')}     ${configFile}${shouldOverwriteConfig ? '' : chalk.dim(' (preserved)')}`);
    console.log(`  ${chalk.cyan('TUI Binary:')} ${hasTui || shouldInstallTui ? chalk.green('Installed') : chalk.gray("Skipped (run 'rtb ui' to download anytime)")}`);
    console.log('');
    console.log(`  ${chalk.bold('Try running:')}`);
    console.log(`    ${chalk.green.bold('rtb')}                  ${chalk.dim('— open RTB workspace cockpit')}`);
    console.log(`    ${chalk.green.bold('rtb menu')}             ${chalk.dim('— interactive prompt launcher')}`);
    console.log(`    ${chalk.green.bold('rtb goto')} ${chalk.cyan('<project>')}   ${chalk.dim('— switch directory into any project')}`);
    console.log(`    ${chalk.green.bold('rtb ui')}               ${chalk.dim('— interactive terminal dashboard')}`);
    console.log(`    ${chalk.green.bold('rtb doctor')}           ${chalk.dim('— verify toolchain health & agents')}`);
    console.log(`    ${chalk.green.bold('rtb help')}             ${chalk.dim('— view all available commands')}\n`);
    };

  program
    .command('init')
    .alias('setup')
    .description('Initialize and configure your RTB workspace')
    .option('-f, --force', 'Overwrite existing configuration', false)
    .option('-r, --root <path>', 'Custom workspace root directory')
    .option('--flat', 'Use flat workspace structure instead of lifecycle folders', false)
    .option('--skip-ui', 'Skip downloading rtbtui binary', false)
    .option('--no-ui', 'Skip downloading rtbtui binary', false)
    .option('--ui', 'Download rtbtui binary during initialization', false)
    .action(initAction);

  program
    .command('install')
    .description('Full interactive installation setup for RTB')
    .option('-f, --force', 'Overwrite existing configuration', false)
    .option('-r, --root <path>', 'Custom workspace root directory')
    .option('--flat', 'Use flat workspace structure instead of lifecycle folders', false)
    .option('--skip-ui', 'Skip downloading rtbtui binary', false)
    .option('--no-ui', 'Skip downloading rtbtui binary', false)
    .option('--ui', 'Download rtbtui binary during installation', false)
    .action(initAction);
}
