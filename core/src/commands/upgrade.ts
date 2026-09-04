import type { Command } from 'commander';
import chalk from 'chalk';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { spawnSync } from 'node:child_process';
import type { CliContext } from '../types/context.js';
import { RTB_VERSION } from './version.js';
import { outputError, outputJson } from '../utils/output.js';

export function parseSemver(v: string): [number, number, number] {
  const cleaned = v.trim().replace(/^v/, '').split('-')[0];
  const parts = cleaned.split('.').map((p) => parseInt(p, 10) || 0);
  return [parts[0] ?? 0, parts[1] ?? 0, parts[2] ?? 0];
}

export function compareSemver(v1: string, v2: string): number {
  const p1 = parseSemver(v1);
  const p2 = parseSemver(v2);

  for (let i = 0; i < 3; i++) {
    if (p1[i] > p2[i]) return 1;
    if (p1[i] < p2[i]) return -1;
  }
  return 0;
}

export async function fetchLatestVersion(): Promise<string | null> {
  const sources = [
    async () => {
      const res = await fetch('https://api.github.com/repos/3mr-5aled/rtb/releases/latest', {
        headers: { 'User-Agent': 'rtb-cli' },
      });
      if (!res.ok) return null;
      const data: any = await res.json();
      return data.tag_name ? data.tag_name.replace(/^v/, '') : null;
    },
    async () => {
      const res = await fetch('https://raw.githubusercontent.com/3mr-5aled/rtb/main/VERSION');
      if (!res.ok) return null;
      const text = await res.text();
      return text.trim().replace(/^v/, '') || null;
    },
    async () => {
      const res = await fetch('https://registry.npmjs.org/@3mr-5aled/rtb/latest');
      if (!res.ok) return null;
      const data: any = await res.json();
      return data.version || null;
    },
  ];

  for (const src of sources) {
    try {
      const ver = await src();
      if (ver) return ver;
    } catch {}
  }

  return null;
}

export function executeUpgrade(): { success: boolean; method: string; message: string } {
  // Strategy 1: npm global install
  try {
    const isWindows = process.platform === 'win32';
    const res = isWindows
      ? spawnSync('npm.cmd install -g @3mr-5aled/rtb@latest', {
          stdio: 'inherit',
          shell: true,
        })
      : spawnSync('npm', ['install', '-g', '@3mr-5aled/rtb@latest'], {
          stdio: 'inherit',
          shell: false,
        });
    if (res.status === 0) {
      return { success: true, method: 'npm', message: 'Successfully updated via npm' };
    }
  } catch {}

  // Strategy 2: Standalone bundle download
  const homeDir = os.homedir();
  const configDir = path.join(homeDir, '.config', 'rtb');
  const binDir = process.env.RTB_BIN_DIR || path.join(configDir, 'bin');
  const targetJs = path.join(binDir, 'rtb.js');

  if (fs.existsSync(targetJs) && process.platform === 'win32') {
    try {
      const psCmd = `
        $bundleUrl = 'https://github.com/3mr-5aled/rtb/releases/latest/download/rtb-cli.js'
        $destJs = '${targetJs.replace(/\\/g, '\\\\')}'
        Invoke-WebRequest -Uri $bundleUrl -OutFile $destJs -UseBasicParsing -TimeoutSec 120
      `;
      const res = spawnSync('powershell.exe', ['-NoProfile', '-Command', psCmd], { stdio: 'inherit' });
      if (res.status === 0) {
        return { success: true, method: 'standalone', message: 'Successfully downloaded latest rtb-cli.js bundle' };
      }
    } catch {}
  }

  return {
    success: false,
    method: 'none',
    message: "Automatic upgrade failed. Run 'npm install -g @3mr-5aled/rtb@latest' or run the setup installer.",
  };
}

export const upgradeService = {
  fetchLatestVersion,
  executeUpgrade,
};

export function registerUpgradeCommand(
  program: Command,
  getContext: () => CliContext,
  service: { fetchLatestVersion: typeof fetchLatestVersion; executeUpgrade: typeof executeUpgrade } = upgradeService
): void {
  program
    .command('upgrade')
    .description('Check for updates and self-upgrade RTB')
    .option('--check', 'Check for updates without performing installation', false)
    .option('-f, --force', 'Force upgrade even if current version is up to date', false)
    .action(async (options: { check?: boolean; force?: boolean }) => {
      const ctx = getContext();
      const currentVersion = RTB_VERSION;

      if (!ctx.isQuiet && !ctx.isJson) {
        console.log(`\n${chalk.cyan('══════════════════════════════════════════')}`);
        console.log(`  ${chalk.bold('rtb (رتّب) » Self Upgrade')}`);
        console.log(`${chalk.cyan('══════════════════════════════════════════')}\n`);
        console.log(`  Current version: ${chalk.green(`v${currentVersion}`)}`);
        console.log(chalk.gray('  Checking for updates...'));
      }

      const latestVersion = await service.fetchLatestVersion();

      if (!latestVersion) {
        if (ctx.isJson) {
          outputError('Failed to fetch latest version from remote sources', 'UPDATE_CHECK_FAILED', true);
        } else {
          console.log(chalk.red('\n  ✗ Failed to check for latest RTB version (network offline or source unreachable).\n'));
        }
        process.exitCode = 1;
        return;
      }

      const comparison = compareSemver(latestVersion, currentVersion);
      const updateAvailable = comparison > 0;

      if (!ctx.isQuiet && !ctx.isJson) {
        console.log(`  Latest version:  ${chalk.cyan(`v${latestVersion}`)}\n`);
      }

      if (options.check) {
        if (ctx.isJson) {
          outputJson({
            currentVersion,
            latestVersion,
            updateAvailable,
            checkOnly: true,
          });
          return;
        }

        if (updateAvailable) {
          console.log(`  ${chalk.yellow('★')} An update is available! (v${currentVersion} → v${latestVersion})`);
          console.log(`  Run '${chalk.cyan('rtb upgrade')}' to install the latest version.\n`);
        } else {
          console.log(`  ${chalk.green('✓')} RTB is already up to date (v${currentVersion}).\n`);
        }
        return;
      }

      if (!updateAvailable && !options.force) {
        if (ctx.isJson) {
          outputJson({
            currentVersion,
            latestVersion,
            updateAvailable: false,
            upgraded: false,
            message: 'Already up to date',
          });
          return;
        }

        console.log(`  ${chalk.green('✓')} RTB is already up to date (v${currentVersion}).`);
        console.log(chalk.gray("  (Use 'rtb upgrade --force' to force reinstallation)\n"));
        return;
      }

      if (!ctx.isQuiet && !ctx.isJson) {
        console.log(`  ${chalk.cyan('→')} Upgrading RTB to v${latestVersion}...`);
      }

      const result = service.executeUpgrade();

      if (ctx.isJson) {
        outputJson({
          currentVersion,
          latestVersion,
          upgraded: result.success,
          method: result.method,
          message: result.message,
        });
        if (!result.success) process.exitCode = 1;
        return;
      }

      if (result.success) {
        console.log(`\n  ${chalk.green('✓')} ${result.message}`);
        console.log(`  ${chalk.green('✓')} Upgraded successfully to v${latestVersion}!\n`);
      } else {
        console.log(`\n  ${chalk.red('✗')} ${result.message}\n`);
        process.exitCode = 1;
      }
    });
}
