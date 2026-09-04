import type { Command } from 'commander';
import chalk from 'chalk';
import fs from 'node:fs';
import path from 'node:path';
import { execSync } from 'node:child_process';
import type { CliContext } from '../types/context.js';
import { outputError, outputJson } from '../utils/output.js';

export function registerUnarchiveCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('unarchive [archive]')
    .description('Extract a previously archived project back into the Active root')
    .action((archiveName: string | undefined) => {
      const ctx = getContext();

      if (!archiveName) {
        console.log(`\n  ${chalk.yellow('Usage:')} rtb unarchive <archive-name.tar.gz>\n`);
        return;
      }

      if (!ctx.config || !ctx.config.projectRoots.active?.path) {
        outputError('Active project root not configured', 'CONFIG_MISSING', ctx.isJson);
        process.exitCode = 1;
        return;
      }

      const activeDir = ctx.config.projectRoots.active.path;
      const backupRoot = ctx.config.backupRoot || path.join(path.dirname(activeDir), 'backup');
      const snapshotDir = path.join(backupRoot, 'project-snapshots');

      let targetArchive = archiveName;
      let archivePath = path.join(snapshotDir, targetArchive);

      // Search by partial name if exact match not found
      if (!fs.existsSync(archivePath) && fs.existsSync(snapshotDir)) {
        try {
          const files = fs.readdirSync(snapshotDir);
          const match = files.find((f) => f.toLowerCase().includes(targetArchive.toLowerCase()));
          if (match) {
            archivePath = path.join(snapshotDir, match);
            targetArchive = match;
          }
        } catch {}
      }

      if (!fs.existsSync(archivePath)) {
        outputError(`Archive '${archiveName}' not found in: ${snapshotDir}`, 'ARCHIVE_NOT_FOUND', ctx.isJson);
        process.exitCode = 1;
        return;
      }

      if (!fs.existsSync(activeDir)) {
        fs.mkdirSync(activeDir, { recursive: true });
      }

      try {
        execSync(`tar -xzf "${archivePath}"`, {
          cwd: activeDir,
          stdio: ['ignore', 'pipe', 'ignore'],
        });
      } catch (err: any) {
        outputError(`Failed to extract archive: ${err.message}`, 'TAR_FAILED', ctx.isJson);
        process.exitCode = 1;
        return;
      }

      if (ctx.isJson) {
        outputJson({
          unarchived: true,
          archive: archivePath,
          destination: activeDir,
        });
        return;
      }

      console.log(`\n  ${chalk.green('✓')} Unarchived: ${chalk.bold(archiveName)}`);
      console.log(`  Extracted to: ${chalk.cyan(activeDir)}`);
      console.log(`  Run: ${chalk.cyan('rtb list --active')}\n`);
    });
}
