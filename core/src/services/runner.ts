import fs from 'node:fs';
import path from 'node:path';
import { spawn } from 'node:child_process';

export interface ResolvedCommand {
  executable: string;
  args: string[];
}

export type ProjectAction = 'run' | 'build' | 'test';

export function resolveProjectAction(
  action: ProjectAction,
  projectDir: string,
  extraArgs: string[] = []
): ResolvedCommand | null {
  const pkgJsonPath = path.join(projectDir, 'package.json');
  const cargoPath = path.join(projectDir, 'Cargo.toml');
  const goModPath = path.join(projectDir, 'go.mod');
  const mainPyPath = path.join(projectDir, 'main.py');
  const pytestIniPath = path.join(projectDir, 'pytest.ini');
  const pyprojectPath = path.join(projectDir, 'pyproject.toml');

  if (action === 'run') {
    // 1. package.json (dev -> start)
    if (fs.existsSync(pkgJsonPath)) {
      try {
        const pkg = JSON.parse(fs.readFileSync(pkgJsonPath, 'utf8'));
        if (pkg.scripts?.dev) {
          const args = ['run', 'dev'];
          if (extraArgs.length > 0) args.push('--', ...extraArgs);
          return { executable: 'npm', args };
        }
        if (pkg.scripts?.start) {
          const args = ['start'];
          if (extraArgs.length > 0) args.push(...extraArgs);
          return { executable: 'npm', args };
        }
      } catch {}
    }

    // 2. Cargo.toml
    if (fs.existsSync(cargoPath)) {
      return { executable: 'cargo', args: ['run', ...extraArgs] };
    }

    // 3. go.mod
    if (fs.existsSync(goModPath)) {
      return { executable: 'go', args: ['run', '.', ...extraArgs] };
    }

    // 4. main.py
    if (fs.existsSync(mainPyPath)) {
      return { executable: 'python', args: ['main.py', ...extraArgs] };
    }

    return null;
  }

  if (action === 'build') {
    // 1. package.json (build)
    if (fs.existsSync(pkgJsonPath)) {
      try {
        const pkg = JSON.parse(fs.readFileSync(pkgJsonPath, 'utf8'));
        if (pkg.scripts?.build) {
          const args = ['run', 'build'];
          if (extraArgs.length > 0) args.push('--', ...extraArgs);
          return { executable: 'npm', args };
        }
      } catch {}
    }

    // 2. Cargo.toml
    if (fs.existsSync(cargoPath)) {
      return { executable: 'cargo', args: ['build', '--release', ...extraArgs] };
    }

    // 3. go.mod
    if (fs.existsSync(goModPath)) {
      return { executable: 'go', args: ['build', ...extraArgs] };
    }

    return null;
  }

  if (action === 'test') {
    // 1. package.json (test)
    if (fs.existsSync(pkgJsonPath)) {
      try {
        const pkg = JSON.parse(fs.readFileSync(pkgJsonPath, 'utf8'));
        if (pkg.scripts?.test) {
          const args = ['test'];
          if (extraArgs.length > 0) args.push('--', ...extraArgs);
          return { executable: 'npm', args };
        }
      } catch {}
    }

    // 2. Cargo.toml
    if (fs.existsSync(cargoPath)) {
      return { executable: 'cargo', args: ['test', ...extraArgs] };
    }

    // 3. pytest / pyproject
    if (fs.existsSync(pytestIniPath) || fs.existsSync(pyprojectPath)) {
      return { executable: 'pytest', args: [...extraArgs] };
    }

    return null;
  }

  return null;
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
    // On Windows, if invoking a .cmd/.bat command (like npm), use shell: true, but for standard binaries directly spawn
    const needsShell = isWindows && !cmd.executable.toLowerCase().endsWith('.exe');
    const child = spawn(cmd.executable, cmd.args, {
      cwd: projectPath,
      stdio: 'inherit',
      shell: needsShell,
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
