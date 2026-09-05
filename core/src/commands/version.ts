import type { Command } from 'commander';
import chalk from 'chalk';
import type { CliContext } from '../types/context.js';
import { outputJson } from '../utils/output.js';

import { existsSync, readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

export function resolveVersion(): string {
  try {
    const currentDir = dirname(fileURLToPath(import.meta.url));

    // 1. Check directly adjacent VERSION file (e.g. in bin directory alongside rtb.js)
    const adjacentVersion = join(currentDir, 'VERSION');
    if (existsSync(adjacentVersion)) {
      const content = readFileSync(adjacentVersion, 'utf-8').trim();
      if (content && /^\d+\.\d+\.\d+/.test(content.replace(/^v/, ''))) {
        return content.replace(/^v/, '');
      }
    }

    // 2. Check if running inside the RTB repository checkout
    let searchDir = currentDir;
    for (let i = 0; i < 5; i++) {
      const parentDir = dirname(searchDir);
      if (!parentDir || parentDir === searchDir) break;
      searchDir = parentDir;

      const pkgPath = join(searchDir, 'core', 'package.json');
      const rootPkgPath = join(searchDir, 'package.json');
      let isRtbRepo = false;

      if (existsSync(pkgPath)) {
        try {
          const pkg = JSON.parse(readFileSync(pkgPath, 'utf-8'));
          if (pkg.name === '@3mr5aled/rtb' || pkg.name === '@3mr-5aled/rtb') isRtbRepo = true;
        } catch {}
      } else if (existsSync(rootPkgPath)) {
        try {
          const pkg = JSON.parse(readFileSync(rootPkgPath, 'utf-8'));
          if (pkg.name === '@3mr5aled/rtb' || pkg.name === '@3mr-5aled/rtb') isRtbRepo = true;
        } catch {}
      }

      if (isRtbRepo) {
        const repoVersionFile = join(searchDir, 'VERSION');
        if (existsSync(repoVersionFile)) {
          const content = readFileSync(repoVersionFile, 'utf-8').trim();
          if (content && /^\d+\.\d+\.\d+/.test(content.replace(/^v/, ''))) {
            return content.replace(/^v/, '');
          }
        }
      }
    }
  } catch {}
  return '0.11.4';
}

export const RTB_VERSION = resolveVersion();

export function registerVersionCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('version')
    .description('Display RTB version information')
    .action(() => {
      const ctx = getContext();
      if (ctx.isJson) {
        outputJson({
          name: 'rtb',
          version: RTB_VERSION,
          engine: 'node',
          platform: process.platform,
          arch: process.arch,
          nodeVersion: process.version,
        });
      } else {
        console.log(`RTB (${chalk.cyan('ﺐﺗر')}) CLI ${chalk.green(`v${RTB_VERSION}`)} [Node]`);
      }
    });
}
