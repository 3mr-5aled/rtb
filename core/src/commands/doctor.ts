import type { Command } from 'commander';
import chalk from 'chalk';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import type { CliContext } from '../types/context.js';
import { findExecutableInPath, getInstalledAgents } from '../agent/discovery.js';
import { resolveConfigPath } from '../config/loader.js';
import { outputJson } from '../utils/output.js';

export interface DoctorCheck {
  category: string;
  name: string;
  passed: boolean;
  optional?: boolean;
  detail?: string;
  path?: string;
}

export function findRtbtuiBinary(): string | null {
  const isWindows = process.platform === 'win32';
  const binName = isWindows ? 'rtbtui.exe' : 'rtbtui';

  // 1. PATH
  const fromPath = findExecutableInPath('rtbtui');
  if (fromPath) return fromPath;

  // 2. RTB_BIN_DIR
  if (process.env.RTB_BIN_DIR) {
    const candidate = path.join(process.env.RTB_BIN_DIR, binName);
    if (fs.existsSync(candidate)) return candidate;
  }

  // 3. User config bin directory: ~/.config/rtb/bin/
  const userConfigBin = path.join(os.homedir(), '.config', 'rtb', 'bin', binName);
  if (fs.existsSync(userConfigBin)) return userConfigBin;

  // 4. Windows AppData fallback
  if (isWindows && process.env.APPDATA) {
    const appDataBin = path.join(process.env.APPDATA, 'rtb', 'bin', binName);
    if (fs.existsSync(appDataBin)) return appDataBin;
  }

  // 5. Local build fallback
  const localTargetRelease = path.resolve(process.cwd(), '../tui/target/release', binName);
  if (fs.existsSync(localTargetRelease)) return localTargetRelease;

  const localTargetDebug = path.resolve(process.cwd(), '../tui/target/debug', binName);
  if (fs.existsSync(localTargetDebug)) return localTargetDebug;

  return null;
}

export function runDoctorChecks(ctx: CliContext): { allGood: boolean; checks: DoctorCheck[] } {
  const checks: DoctorCheck[] = [];
  let allGood = true;

  // 1. Config Check
  const cfgRes = resolveConfigPath(ctx.configPath);
  const cfgPath = cfgRes.path;
  const cfgLoaded = ctx.config !== null;
  checks.push({
    category: 'Config',
    name: 'rtb.config.json',
    passed: cfgLoaded,
    detail: cfgLoaded ? `Loaded from ${cfgPath}` : `Not found or invalid at ${cfgPath}. Run 'rtb config' to inspect.`,
    path: cfgPath,
  });
  if (!cfgLoaded) allGood = false;

  // 2. Project Roots
  if (ctx.config?.projectRoots) {
    for (const [key, entry] of Object.entries(ctx.config.projectRoots)) {
      const exists = entry.path ? fs.existsSync(entry.path) : false;
      checks.push({
        category: 'Project Roots',
        name: `${entry.emoji} ${entry.label} (${key})`,
        passed: exists,
        detail: exists ? entry.path : `Directory does not exist: ${entry.path}`,
        path: entry.path,
      });
      if (!exists) allGood = false;
    }
  } else {
    checks.push({
      category: 'Project Roots',
      name: 'Project Roots configured',
      passed: false,
      detail: 'Configuration missing projectRoots mapping',
    });
    allGood = false;
  }

  // 3. Required Tools
  const gitPath = findExecutableInPath('git');
  checks.push({
    category: 'Required Tools',
    name: 'git in PATH',
    passed: Boolean(gitPath),
    detail: gitPath ? gitPath : 'Git was not found in PATH',
    path: gitPath ?? undefined,
  });
  if (!gitPath) allGood = false;

  // 4. Optional Tools
  const optionalTools = [
    { name: 'node', label: 'Node.js (for JavaScript/TypeScript projects)' },
    { name: 'cargo', label: 'Cargo / Rust (for Rust projects & rtbtui build)' },
    { name: 'python', label: 'Python (for Python projects)' },
    { name: 'tar', label: 'tar (for rtb archive / unarchive)' },
  ];
  for (const t of optionalTools) {
    const p = findExecutableInPath(t.name);
    checks.push({
      category: 'Optional Tools',
      name: t.label,
      passed: Boolean(p),
      optional: true,
      detail: p ? p : `${t.name} not found in PATH`,
      path: p ?? undefined,
    });
  }

  // 5. AI Agents
  const agents = getInstalledAgents();
  const installedAgents = agents.filter((a) => a.installed);
  checks.push({
    category: 'AI Agents',
    name: 'Installed AI Agents',
    passed: installedAgents.length > 0,
    optional: true,
    detail: installedAgents.length > 0
      ? installedAgents.map((a) => `${a.name} (${a.command})`).join(', ')
      : 'No supported AI agents found in PATH',
  });

  // 6. TUI Binary
  const tuiPath = findRtbtuiBinary();
  checks.push({
    category: 'TUI Binary',
    name: 'rtbtui binary',
    passed: Boolean(tuiPath),
    optional: true,
    detail: tuiPath ? `Installed at ${tuiPath}` : 'Build with: cargo build --release -p rtbtui in tui/',
    path: tuiPath ?? undefined,
  });

  return { allGood, checks };
}

export function registerDoctorCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('doctor')
    .description('Run comprehensive environment, tool, config, and diagnostic checks')
    .action(() => {
      const ctx = getContext();
      const { allGood, checks } = runDoctorChecks(ctx);

      if (ctx.isJson) {
        outputJson({
          healthy: allGood,
          checks,
        });
        if (!allGood) process.exitCode = 1;
        return;
      }

      console.log(`\n${chalk.cyan('══════════════════════════════════════════')}`);
      console.log(`  ${chalk.bold('rtb (رتّب) » System Doctor')}`);
      console.log(`${chalk.cyan('══════════════════════════════════════════')}\n`);

      const categories = [...new Set(checks.map((c) => c.category))];
      for (const cat of categories) {
        console.log(`  ${chalk.cyan(cat)}`);
        const catChecks = checks.filter((c) => c.category === cat);
        for (const c of catChecks) {
          if (c.passed) {
            console.log(`    ${chalk.green('✓')} ${c.name}`);
            if (c.detail && cat === 'AI Agents') {
              console.log(`       ${chalk.gray(c.detail)}`);
            }
          } else if (c.optional) {
            console.log(`    ${chalk.yellow('⚠')} ${chalk.yellow(c.name)} ${chalk.gray(`(${c.detail})`)}`);
          } else {
            console.log(`    ${chalk.red('✗')} ${chalk.red(c.name)}`);
            if (c.detail) {
              console.log(`       ${chalk.yellow(`→ ${c.detail}`)}`);
            }
          }
        }
        console.log('');
      }

      console.log(chalk.cyan('══════════════════════════════════════════'));
      if (allGood) {
        console.log(`  ${chalk.green('✓ All checks passed — RTB is healthy!')}`);
      } else {
        console.log(`  ${chalk.red('✗ Some checks failed — see above for details.')}`);
        process.exitCode = 1;
      }
      console.log(`${chalk.cyan('══════════════════════════════════════════')}\n`);
    });
}
