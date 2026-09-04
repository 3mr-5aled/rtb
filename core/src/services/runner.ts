import fs from 'node:fs';
import path from 'node:path';
import { spawn } from 'node:child_process';
import chalk from 'chalk';
import type { Command } from 'commander';
import type { CliContext } from '../types/context.js';
import { resolveProjectTarget } from '../navigation/fuzzy.js';
import { ProjectNotFoundError, RtbError } from '../errors.js';
import { outputError, outputJson } from '../utils/output.js';

export interface ResolvedCommand {
  executable: string;
  args: string[];
}

export type ProjectAction = 'run' | 'build' | 'test';

function detectNodePackageManager(projectDir: string): string {
  if (fs.existsSync(path.join(projectDir, 'pnpm-lock.yaml'))) return 'pnpm';
  if (fs.existsSync(path.join(projectDir, 'yarn.lock'))) return 'yarn';
  return 'npm';
}

function hasDotNetProject(projectDir: string): boolean {
  try {
    const files = fs.readdirSync(projectDir);
    return files.some((f) => f.endsWith('.csproj') || f.endsWith('.sln'));
  } catch {
    return false;
  }
}

export function resolveProjectAction(
  action: ProjectAction,
  projectDir: string,
  extraArgs: string[] = []
): ResolvedCommand | null {
  const pkgJsonPath = path.join(projectDir, 'package.json');
  const cargoPath = path.join(projectDir, 'Cargo.toml');
  const goModPath = path.join(projectDir, 'go.mod');
  const makefilePath = path.join(projectDir, 'Makefile');
  const mainPyPath = path.join(projectDir, 'main.py');
  const pytestIniPath = path.join(projectDir, 'pytest.ini');
  const pyprojectPath = path.join(projectDir, 'pyproject.toml');

  // 1. Node.js project (npm / pnpm / yarn)
  if (fs.existsSync(pkgJsonPath)) {
    try {
      const pkg = JSON.parse(fs.readFileSync(pkgJsonPath, 'utf8'));
      const pm = detectNodePackageManager(projectDir);

      if (action === 'run') {
        if (pkg.scripts?.dev) {
          const args = pm === 'npm' ? ['run', 'dev'] : ['dev'];
          if (extraArgs.length > 0) args.push('--', ...extraArgs);
          return { executable: pm, args };
        }
        if (pkg.scripts?.start) {
          const args = pm === 'npm' ? ['start'] : ['start'];
          if (extraArgs.length > 0) args.push('--', ...extraArgs);
          return { executable: pm, args };
        }
      } else if (action === 'build') {
        if (pkg.scripts?.build) {
          const args = pm === 'npm' ? ['run', 'build'] : ['build'];
          if (extraArgs.length > 0) args.push('--', ...extraArgs);
          return { executable: pm, args };
        }
      } else if (action === 'test') {
        if (pkg.scripts?.test) {
          const args = pm === 'npm' ? ['test'] : ['test'];
          if (extraArgs.length > 0) args.push('--', ...extraArgs);
          return { executable: pm, args };
        }
      }
    } catch {}
  }

  // 2. Rust (Cargo)
  if (fs.existsSync(cargoPath)) {
    if (action === 'run') {
      const args = ['run'];
      if (extraArgs.length > 0) args.push('--', ...extraArgs);
      return { executable: 'cargo', args };
    }
    if (action === 'build') {
      return { executable: 'cargo', args: ['build', '--release', ...extraArgs] };
    }
    if (action === 'test') {
      return { executable: 'cargo', args: ['test', ...extraArgs] };
    }
  }

  // 3. Go
  if (fs.existsSync(goModPath)) {
    if (action === 'run') {
      return { executable: 'go', args: ['run', '.', ...extraArgs] };
    }
    if (action === 'build') {
      return { executable: 'go', args: ['build', ...extraArgs] };
    }
    if (action === 'test') {
      return { executable: 'go', args: ['test', './...', ...extraArgs] };
    }
  }

  // 4. .NET
  if (hasDotNetProject(projectDir)) {
    if (action === 'run') {
      const args = ['run'];
      if (extraArgs.length > 0) args.push('--', ...extraArgs);
      return { executable: 'dotnet', args };
    }
    if (action === 'build') {
      return { executable: 'dotnet', args: ['build', ...extraArgs] };
    }
    if (action === 'test') {
      return { executable: 'dotnet', args: ['test', ...extraArgs] };
    }
  }

  // 5. Python
  if (action === 'run' && fs.existsSync(mainPyPath)) {
    return { executable: 'python', args: ['main.py', ...extraArgs] };
  }
  if (action === 'test' && (fs.existsSync(pytestIniPath) || fs.existsSync(pyprojectPath))) {
    return { executable: 'pytest', args: [...extraArgs] };
  }

  // 6. Makefile
  if (fs.existsSync(makefilePath)) {
    if (action === 'run') {
      return { executable: 'make', args: ['run', ...extraArgs] };
    }
    if (action === 'build') {
      return { executable: 'make', args: ['build', ...extraArgs] };
    }
    if (action === 'test') {
      return { executable: 'make', args: ['test', ...extraArgs] };
    }
  }

  return null;
}

function quoteShellArg(arg: string): string {
  if (arg === '') return '""';
  if (/[ \t\n\v"]/.test(arg)) {
    return `"${arg.replace(/"/g, '\\"')}"`;
  }
  return arg;
}

export function executeProjectAction(
  projectPath: string,
  cmd: ResolvedCommand,
  options: { dryRun?: boolean } = {}
): Promise<number> {
  if (options.dryRun) {
    return Promise.resolve(0);
  }

  return new Promise((resolve) => {
    const isWindows = process.platform === 'win32';
    const needsShell = isWindows && !cmd.executable.toLowerCase().endsWith('.exe');
    
    // On Node 22+, passing args array with shell: true triggers DEP0190.
    // Format the command and args as a single string when shell: true.
    const child = needsShell
      ? spawn(
          [
            cmd.executable.includes(' ') ? `"${cmd.executable}"` : cmd.executable,
            ...cmd.args.map(quoteShellArg),
          ].join(' '),
          {
            cwd: projectPath,
            stdio: 'inherit',
            shell: true,
          }
        )
      : spawn(cmd.executable, cmd.args, {
          cwd: projectPath,
          stdio: 'inherit',
          shell: false,
        });

    child.on('error', (err) => {
      console.error(`Failed to execute ${cmd.executable}: ${err.message}`);
      resolve(1);
    });

    child.on('close', (code) => {
      resolve(code ?? 0);
    });
  });
}

export async function runActionCommand(
  action: ProjectAction,
  projectName: string | undefined,
  extraArgs: string[] | undefined,
  options: { dryRun?: boolean },
  ctx: CliContext
): Promise<void> {
  let targetPath = process.cwd();
  const finalExtraArgs = Array.isArray(extraArgs) ? extraArgs : [];

  if (projectName) {
    const target = resolveProjectTarget(projectName, ctx.config);
    if (!target) {
      throw new ProjectNotFoundError(projectName);
    }
    targetPath = target.targetPath;
  }

  const resolved = resolveProjectAction(action, targetPath, finalExtraArgs);
  if (!resolved) {
    const msg = `No ${action} configuration or entrypoint detected in ${targetPath}`;
    throw new RtbError(msg, 'NO_ENTRYPOINT', 1);
  }

  if (ctx.isJson && options.dryRun) {
    outputJson({
      action,
      targetPath,
      executable: resolved.executable,
      args: resolved.args,
      dryRun: true,
    });
    process.exitCode = 0;
    return;
  }

  if (!ctx.isQuiet && !ctx.isJson) {
    const leaf = path.basename(targetPath);
    const title = action.charAt(0).toUpperCase() + action.slice(1);
    console.log(`\n${chalk.cyan('══════════════════════════════════════════')}`);
    console.log(`  ${chalk.bold(`rtb (ﺐﺗر) » ${title} (${leaf})`)}`);
    console.log(`${chalk.cyan('══════════════════════════════════════════')}`);
    console.log(chalk.green(`  Running: ${resolved.executable} ${resolved.args.join(' ')}\n`));
  }

  const exitCode = await executeProjectAction(targetPath, resolved, { dryRun: options.dryRun });
  process.exitCode = exitCode;
}

export function registerRunCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('run [project] [args...]')
    .description('Run project dev server or main entrypoint')
    .allowUnknownOption(true)
    .option('--dry-run', 'Inspect command resolution without executing', false)
    .action((projectName: string | undefined, extraArgs: string[] | undefined, options: { dryRun?: boolean }) => {
      return runActionCommand('run', projectName, extraArgs, options, getContext());
    });
}

export function registerBuildCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('build [project] [args...]')
    .description('Build project for release or compilation')
    .allowUnknownOption(true)
    .option('--dry-run', 'Inspect build command resolution without executing', false)
    .action((projectName: string | undefined, extraArgs: string[] | undefined, options: { dryRun?: boolean }) => {
      return runActionCommand('build', projectName, extraArgs, options, getContext());
    });
}

export function registerTestCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('test [project] [args...]')
    .description('Run test suite in project directory')
    .allowUnknownOption(true)
    .option('--dry-run', 'Inspect test command resolution without executing', false)
    .action((projectName: string | undefined, extraArgs: string[] | undefined, options: { dryRun?: boolean }) => {
      return runActionCommand('test', projectName, extraArgs, options, getContext());
    });
}

export function registerRunnerCommands(program: Command, getContext: () => CliContext): void {
  registerRunCommand(program, getContext);
  registerBuildCommand(program, getContext);
  registerTestCommand(program, getContext);
}

