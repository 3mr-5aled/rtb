import type { Command } from 'commander';
import chalk from 'chalk';
import { spawn } from 'node:child_process';
import type { CliContext } from '../types/context.js';
import { findRtbtuiBinary } from './doctor.js';
import { outputError } from '../utils/output.js';

export function registerUiCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('ui')
    .description('Launch the Rust-powered interactive TUI (rtbtui)')
    .action(async () => {
      const ctx = getContext();
      const binaryPath = findRtbtuiBinary();

      if (!binaryPath) {
        outputError(
          'rtbtui binary not found. Build with: cargo build --release -p rtbtui in tui/',
          'TUI_NOT_FOUND',
          ctx.isJson
        );
        process.exitCode = 1;
        return;
      }

      const isWindows = process.platform === 'win32';
      const child = spawn(binaryPath, [], {
        stdio: 'inherit',
        shell: isWindows,
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
