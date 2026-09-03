import type { Command } from 'commander';
import chalk from 'chalk';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import readline from 'node:readline';
import type { CliContext } from '../types/context.js';
import { getStandardConfigDir, getStandardConfigPath } from '../config/loader.js';
import { outputJson } from '../utils/output.js';
import type { RtbConfig } from '../types/config.js';

export function registerInitCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('init')
    .description('Initialize and configure your RTB workspace')
    .option('-f, --force', 'Overwrite existing configuration', false)
    .option('-r, --root <path>', 'Custom workspace root directory')
    .option('--flat', 'Use flat workspace structure instead of Active/Paused/Archive', false)
    .action(async (options: { force?: boolean; root?: string; flat?: boolean }) => {
      const ctx = getContext();
      const configDir = getStandardConfigDir();
      const configFile = getStandardConfigPath();

      if (fs.existsSync(configFile) && !options.force) {
        if (ctx.isJson) {
          outputJson({ status: 'already_configured', configPath: configFile });
          return;
        }
        console.log('');
        console.log(`  ${chalk.yellow('⚠')}  Configuration already exists at:`);
        console.log(`     ${chalk.white(configFile)}`);
        console.log(`     Run '${chalk.cyan('rtb config')}' to view or edit.`);
        console.log(`     Use '${chalk.gray('rtb init --force')}' to overwrite.\n`);
        return;
      }

      const homeDir = os.homedir();
      let chosenRoot = options.root;

      if (!chosenRoot) {
        const candidateRoots = [
          path.join(homeDir, 'Projects'),
          path.join(homeDir, 'dev'),
          path.join(homeDir, 'code'),
          path.join(homeDir, 'repos'),
          path.join(homeDir, 'workspace'),
          'D:\\02-Projects',
          'D:\\Projects',
        ];
        const existing = candidateRoots.filter((p) => {
          try {
            return fs.existsSync(p);
          } catch {
            return false;
          }
        });

        if (ctx.isInteractive && existing.length > 0) {
          console.log(`\n  ${chalk.cyan('Step 1: Workspace Root Location')}`);
          console.log(`  Where do you want to keep and manage your projects?\n`);
          existing.forEach((p, idx) => {
            console.log(`    [${idx + 1}] ${p}`);
          });
          console.log(`    [C] Enter custom path`);

          const rl = readline.createInterface({ input: process.stdin, output: process.stdout });
          const answer: string = await new Promise((res) => {
            rl.question(`\n  Select option [1-${existing.length} or C] (Default: 1): `, (ans) => {
              rl.close();
              res(ans.trim());
            });
          });

          if (answer.toLowerCase() === 'c') {
            const rl2 = readline.createInterface({ input: process.stdin, output: process.stdout });
            chosenRoot = await new Promise((res) => {
              rl2.question('  Enter workspace root path: ', (ans) => {
                rl2.close();
                res(ans.trim());
              });
            });
          } else {
            const num = parseInt(answer, 10);
            if (!isNaN(num) && num >= 1 && num <= existing.length) {
              chosenRoot = existing[num - 1];
            } else {
              chosenRoot = existing[0];
            }
          }
        } else {
          chosenRoot = existing.length > 0 ? existing[0] : path.join(homeDir, 'Projects');
        }
      }

      if (!chosenRoot) {
        chosenRoot = path.join(homeDir, 'Projects');
      }
      chosenRoot = path.resolve(chosenRoot);

      // Structure
      const isFlat = options.flat;
      let projectRoots: RtbConfig['projectRoots'] = {};

      if (isFlat) {
        projectRoots = {
          projects: {
            path: chosenRoot,
            label: 'Projects',
            emoji: '📁',
          },
        };
      } else {
        const activeDir = path.join(chosenRoot, '01-Active');
        const pausedDir = path.join(chosenRoot, '02-Paused');
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

      console.log(`\n  ${chalk.green('✔')} ${chalk.bold('RTB workspace successfully initialized!')}`);
      console.log(`  ${chalk.cyan('Config:')}  ${configFile}`);
      console.log(`  ${chalk.cyan('Root:')}    ${chosenRoot}`);
      console.log(`\n  ${chalk.bold('Next steps:')}`);
      console.log(`    ${chalk.green('rtb list')}    - list registered projects`);
      console.log(`    ${chalk.green('rtb doctor')}  - verify toolchain health`);
      console.log(`    ${chalk.green('rtb help')}    - view all available commands\n`);
    });
}
