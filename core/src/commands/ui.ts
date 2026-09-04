import type { Command } from 'commander';
import chalk from 'chalk';
import fs from 'node:fs';
import path from 'node:path';
import readline from 'node:readline';
import { spawn } from 'node:child_process';
import type { CliContext } from '../types/context.js';
import { findRtbtuiBinary, getDefaultUserBinDir, getPlatformBinaryAsset } from './doctor.js';
import { outputError } from '../utils/output.js';

export async function provisionRtbtuiBinary(options?: {
  destDir?: string;
  fetchFn?: typeof fetch;
  platform?: string;
  arch?: string;
}): Promise<string | null> {
  const assetName = getPlatformBinaryAsset(options?.platform, options?.arch);
  if (!assetName) return null;

  const binDir = options?.destDir || getDefaultUserBinDir();
  const isWindows = (options?.platform || process.platform) === 'win32';
  const binName = isWindows ? 'rtbtui.exe' : 'rtbtui';
  const destPath = path.join(binDir, binName);

  const url = `https://github.com/3mr-5aled/rtb/releases/latest/download/${assetName}`;
  const fetchImpl = options?.fetchFn || globalThis.fetch;

  try {
    const res = await fetchImpl(url);
    if (!res.ok) {
      throw new Error(`HTTP ${res.status}: ${res.statusText}`);
    }
    const buffer = Buffer.from(await res.arrayBuffer());
    if (!fs.existsSync(binDir)) {
      fs.mkdirSync(binDir, { recursive: true });
    }
    fs.writeFileSync(destPath, buffer);
    if (!isWindows) {
      fs.chmodSync(destPath, 0o755);
    }
    return destPath;
  } catch {
    return null;
  }
}

export function registerUiCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('ui')
    .description('Launch the Rust-powered interactive TUI (rtbtui)')
    .option('--download', 'Download prebuilt rtbtui binary if not installed locally', false)
    .action(async (options: { download?: boolean }) => {
      const ctx = getContext();
      let binaryPath = findRtbtuiBinary();

      if (!binaryPath) {
        const asset = getPlatformBinaryAsset();
        let shouldDownload = options.download === true;

        if (!shouldDownload && asset && ctx.isInteractive && !ctx.isJson) {
          const rl = readline.createInterface({ input: process.stdin, output: process.stdout });
          const answer = await new Promise<string>((resolve) => {
            rl.question(
              chalk.cyan(`? rtbtui binary not found. Download prebuilt ${asset} from GitHub Releases? (Y/n) `),
              (ans) => {
                rl.close();
                resolve(ans.trim());
              }
            );
          });
          shouldDownload = answer === '' || answer.toLowerCase().startsWith('y');
        }

        if (shouldDownload && asset) {
          if (!ctx.isJson) {
            console.log(chalk.cyan(`⬇ Downloading prebuilt rtbtui (${asset})...`));
          }
          const downloaded = await provisionRtbtuiBinary();
          if (downloaded) {
            if (!ctx.isJson) {
              console.log(chalk.green(`✓ Installed rtbtui binary to ${downloaded}`));
            }
            binaryPath = downloaded;
          } else {
            if (!ctx.isJson) {
              console.error(chalk.yellow(`⚠ Download failed. Falling back to manual build instructions.`));
            }
          }
        }
      }

      if (!binaryPath) {
        outputError(
          'rtbtui binary not found. Build with: cargo build --release -p rtbtui in tui/, or download prebuilt binaries from https://github.com/3mr-5aled/rtb/releases',
          'TUI_NOT_FOUND',
          ctx.isJson
        );
        process.exitCode = 1;
        return;
      }

      const isWindows = process.platform === 'win32';
      const args: string[] = [];
      const env = { ...process.env };

      if (ctx.configPath) {
        args.push('--config', ctx.configPath);
        env.RTB_CONFIG = ctx.configPath;
      }

      const child = isWindows
        ? spawn(
            [
              binaryPath.includes(' ') ? `"${binaryPath}"` : binaryPath,
              ...args.map((a) => (a.includes(' ') ? `"${a}"` : a)),
            ].join(' '),
            {
              stdio: 'inherit',
              shell: true,
              env,
            }
          )
        : spawn(binaryPath, args, {
            stdio: 'inherit',
            shell: false,
            env,
          });

      child.on('error', (err) => {
        outputError(`Failed to spawn rtbtui: ${err.message}`, 'TUI_SPAWN_FAILED', ctx.isJson);
        process.exitCode = 1;
      });

      await new Promise<void>((resolve) => {
        child.on('close', (code) => {
          process.exitCode = code ?? 0;
          resolve();
        });
      });
    });
}
