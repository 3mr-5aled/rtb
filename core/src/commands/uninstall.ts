import type { Command } from 'commander';
import chalk from 'chalk';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import readline from 'node:readline';
import { spawnSync } from 'node:child_process';
import type { CliContext } from '../types/context.js';
import { outputError, outputJson } from '../utils/output.js';

export function cleanProfileContent(content: string): string {
  const lines = content.split(/\r?\n/);
  const filtered = lines.filter((line) => {
    if (/rtb\s+shell-init/i.test(line)) return false;
    if (/#\s*RTB\s+Shell\s+Integration/i.test(line)) return false;
    if (/Import-Module.*?(rtb|dev-tools|dev-cli|rtb-command-tool).*?\.psd1/i.test(line)) return false;
    if (/#\s*RTB.*?Module/i.test(line)) return false;
    return true;
  });

  const sep = content.includes('\r\n') ? '\r\n' : '\n';
  return filtered.join(sep);
}

export function cleanShellProfiles(customCandidates?: string[]): string[] {
  const cleaned: string[] = [];
  const homeDir = os.homedir();

  const candidates: string[] = customCandidates ?? [];
  if (!customCandidates) {
    if (process.platform === 'win32') {
      const docs = path.join(homeDir, 'Documents');
      candidates.push(
        path.join(docs, 'PowerShell', 'Microsoft.PowerShell_profile.ps1'),
        path.join(docs, 'WindowsPowerShell', 'Microsoft.PowerShell_profile.ps1')
      );
    }

    candidates.push(
      path.join(homeDir, '.bashrc'),
      path.join(homeDir, '.bash_profile'),
      path.join(homeDir, '.zshrc'),
      path.join(homeDir, '.config', 'fish', 'config.fish')
    );
  }

  for (const prof of candidates) {
    if (fs.existsSync(prof)) {
      try {
        const content = fs.readFileSync(prof, 'utf-8');
        const newContent = cleanProfileContent(content);

        if (newContent !== content) {
          fs.writeFileSync(prof, newContent, 'utf-8');
          cleaned.push(prof);
        }
      } catch {}
    }
  }

  return cleaned;
}

export function findRtbExecutables(): string[] {
  const executables: string[] = [];
  const candidateDirs: string[] = [];

  if (process.platform === 'win32') {
    if (fs.existsSync('D:\\bin')) {
      candidateDirs.push('D:\\bin');
    }
    if (process.env.LOCALAPPDATA) {
      candidateDirs.push(path.join(process.env.LOCALAPPDATA, 'Programs', 'rtb', 'bin'));
    }
  }

  // Probe via where.exe / which
  try {
    const isWindows = process.platform === 'win32';
    const cmd = isWindows ? 'where.exe' : 'which';
    for (const binName of ['rtb', 'rtbtui']) {
      const binArgs = isWindows ? [binName] : ['-a', binName];
      const res = spawnSync(cmd, binArgs, { encoding: 'utf-8' });
      if (res.status === 0 && res.stdout) {
        const found = res.stdout.split(/\r?\n/).map((l) => l.trim()).filter(Boolean);
        for (const f of found) {
          const dir = path.dirname(f);
          if (!candidateDirs.includes(dir)) {
            candidateDirs.push(dir);
          }
        }
      }
    }
  } catch {}

  const targetFilenames = ['rtb', 'rtb.cmd', 'rtb.ps1', 'rtb.js', 'rtbtui', 'rtbtui.exe'];

  for (const dir of candidateDirs) {
    for (const file of targetFilenames) {
      const fullPath = path.join(dir, file);
      if (fs.existsSync(fullPath)) {
        const norm = path.normalize(fullPath).toLowerCase();
        // Prevent deleting source or build files inside the development repository
        if (
          !norm.includes(path.normalize('rtb-command-tool').toLowerCase()) &&
          !executables.includes(fullPath)
        ) {
          executables.push(fullPath);
        }
      }
    }
  }

  return executables;
}

export function performUninstall(options: { keepConfig?: boolean; customConfigDir?: string } = {}): {
  removedPaths: string[];
  cleanedProfiles: string[];
} {
  const removedPaths: string[] = [];
  const homeDir = os.homedir();
  const userConfigDir = options.customConfigDir || path.join(homeDir, '.config', 'rtb');
  const binDir = process.env.RTB_BIN_DIR || path.join(userConfigDir, 'bin');

  // 1. Clean shell profiles
  const cleanedProfiles = cleanShellProfiles();

  // 2. Remove binary directory
  if (fs.existsSync(binDir)) {
    try {
      fs.rmSync(binDir, { recursive: true, force: true });
      removedPaths.push(binDir);
    } catch {}
  }

  // 2b. Remove discovered standalone binary wrappers from PATH (e.g. D:\bin, etc.)
  const discoveredExecutables = findRtbExecutables();
  for (const exe of discoveredExecutables) {
    try {
      if (fs.existsSync(exe)) {
        fs.unlinkSync(exe);
        removedPaths.push(exe);
      }
    } catch {}
  }

  // 2c. Check and uninstall global npm package if present
  try {
    const isWindows = process.platform === 'win32';
    const npmCmd = isWindows ? 'npm.cmd' : 'npm';
    spawnSync(npmCmd, ['uninstall', '-g', '@3mr5aled/rtb', '@3mr-5aled/rtb'], { stdio: 'ignore' });
  } catch {}

  // 3. Remove user configuration directory if not keepConfig
  if (!options.keepConfig && fs.existsSync(userConfigDir)) {
    const normConfigDir = path.normalize(userConfigDir).toLowerCase();
    const isInsideRepo = normConfigDir.includes(path.normalize('rtb-command-tool').toLowerCase());
    const isTargetConfig =
      path.basename(userConfigDir) === 'rtb' ||
      normConfigDir.endsWith(path.normalize('.config/rtb').toLowerCase()) ||
      normConfigDir.includes(path.normalize('rtb-uninstall-test').toLowerCase());

    if (!isInsideRepo && isTargetConfig) {
      try {
        fs.rmSync(userConfigDir, { recursive: true, force: true });
        removedPaths.push(userConfigDir);
      } catch {}
    } else if (!isInsideRepo && fs.existsSync(path.join(userConfigDir, 'rtb.config.json'))) {
      try {
        fs.unlinkSync(path.join(userConfigDir, 'rtb.config.json'));
        removedPaths.push(path.join(userConfigDir, 'rtb.config.json'));
      } catch {}
    }
  }

  // 4. Legacy AppData cleanup on Windows
  if (process.platform === 'win32' && process.env.APPDATA) {
    const legacyRoaming = path.join(process.env.APPDATA, 'rtb');
    if (fs.existsSync(legacyRoaming)) {
      try {
        fs.rmSync(legacyRoaming, { recursive: true, force: true });
        removedPaths.push(legacyRoaming);
      } catch {}
    }
  }

  // 5. Clean PATH on Windows
  if (process.platform === 'win32') {
    try {
      const cleanPs = `
        $cur = [Environment]::GetEnvironmentVariable('PATH', 'User')
        if ($cur) {
          $parts = @($cur -split ';' | Where-Object { $_ -and $_ -notmatch '(?i)\\.config[\\\\/]rtb[\\\\/]bin' -and $_ -notmatch '(?i)AppData[\\\\/]Roaming[\\\\/]rtb' })
          [Environment]::SetEnvironmentVariable('PATH', ($parts -join ';'), 'User')
        }
      `;
      spawnSync('powershell.exe', ['-NoProfile', '-Command', cleanPs], { stdio: 'ignore' });
    } catch {}
  }

  return { removedPaths, cleanedProfiles };
}

export function registerUninstallCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('uninstall')
    .description('Cleanly uninstall RTB from system, removing binaries, profile hooks, and config')
    .option('-f, --force', 'Skip interactive confirmation prompt', false)
    .option('--keep-config', 'Preserve workspace configuration and projects', false)
    .action(async (options: { force?: boolean; keepConfig?: boolean }) => {
      const ctx = getContext();

      if (!options.force) {
        if (!ctx.isInteractive) {
          outputError(
            "Uninstallation requires confirmation. Run 'rtb uninstall --force' to proceed non-interactively.",
            'CONFIRMATION_REQUIRED',
            ctx.isJson
          );
          process.exitCode = 1;
          return;
        }

        const rl = readline.createInterface({
          input: process.stdin,
          output: process.stdout,
        });

        const answer: string = await new Promise((resolve) => {
          rl.question(chalk.yellow('  Are you sure you want to uninstall RTB from your system? (y/N) '), (ans) => {
            rl.close();
            resolve(ans.trim().toLowerCase());
          });
        });

        if (answer !== 'y' && answer !== 'yes') {
          console.log(chalk.gray('\n  Uninstallation canceled.\n'));
          return;
        }
      }

      const customConfigDir = ctx.configPath ? path.dirname(ctx.configPath) : undefined;
      const result = performUninstall({
        keepConfig: options.keepConfig,
        customConfigDir,
      });

      if (ctx.isJson) {
        outputJson({
          uninstalled: true,
          removedPaths: result.removedPaths,
          cleanedProfiles: result.cleanedProfiles,
          keptConfig: Boolean(options.keepConfig),
        });
        return;
      }

      console.log(`\n${chalk.cyan('══════════════════════════════════════════')}`);
      console.log(`  ${chalk.bold('rtb (ﺐﺗر) » Uninstallation')}`);
      console.log(`${chalk.cyan('══════════════════════════════════════════')}\n`);

      if (result.removedPaths.length > 0) {
        for (const p of result.removedPaths) {
          console.log(`  ${chalk.green('✓')} Removed: ${chalk.gray(p)}`);
        }
      }

      if (result.cleanedProfiles.length > 0) {
        for (const prof of result.cleanedProfiles) {
          console.log(`  ${chalk.green('✓')} Cleaned profile: ${chalk.gray(prof)}`);
        }
      }

      if (options.keepConfig) {
        console.log(`  ${chalk.yellow('ℹ')} Configuration directory preserved.`);
      }

      console.log(`\n  ${chalk.green('✓ RTB has been successfully uninstalled from your system.')}\n`);
    });
}
