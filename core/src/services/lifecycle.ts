import fs from 'node:fs';
import path from 'node:path';
import { execSync } from 'node:child_process';
import type { RtbConfig } from '../types/config.js';
import { isGitClean } from '../utils/git.js';
import {
  RtbError,
  ProjectNotFoundError,
  DirtyGitError,
  ConfigInvalidError,
} from '../errors.js';

export function toKebabCase(str: string): string {
  return str
    .toLowerCase()
    .replace(/[^a-z0-9\-]/g, '-')
    .replace(/-+/g, '-')
    .replace(/^-|-$/g, '');
}

export function pruneDirectory(
  dir: string,
  targets: string[] = ['node_modules', '.venv', '.next', '__pycache__', 'dist', 'build', 'target']
): string[] {
  const pruned: string[] = [];
  for (const target of targets) {
    const targetPath = path.join(dir, target);
    if (fs.existsSync(targetPath)) {
      try {
        fs.rmSync(targetPath, { recursive: true, force: true });
        pruned.push(target);
      } catch {}
    }
  }
  return pruned;
}

export interface CreateProjectOptions {
  name: string;
  stack?: string;
  activeRoot: string;
  templateDir?: string;
}

export interface CreateProjectResult {
  name: string;
  path: string;
  stack: string;
}

export interface PauseProjectOptions {
  name: string;
  config: RtbConfig;
  prune?: boolean;
  force?: boolean;
}

export interface PauseProjectResult {
  name: string;
  from: string;
  to: string;
  pruned: boolean;
}

export interface ResumeProjectOptions {
  name: string;
  config: RtbConfig;
  install?: boolean;
}

export interface ResumeProjectResult {
  name: string;
  from: string;
  to: string;
  installed: boolean;
}

export class ProjectLifecycle {
  public create(options: CreateProjectOptions): CreateProjectResult {
    const stack = options.stack || 'generic';
    const kebabName = toKebabCase(options.name);
    const targetDir = path.join(options.activeRoot, kebabName);

    if (fs.existsSync(targetDir)) {
      throw new RtbError(
        `Project '${kebabName}' already exists at: ${targetDir}`,
        'ALREADY_EXISTS'
      );
    }

    fs.mkdirSync(targetDir, { recursive: true });

    // 1. PROJECT.md
    let projectMdContent = `# ${options.name}\n\nCreated: ${new Date().toISOString().slice(0, 10)}\nStack: ${stack}\n`;
    if (options.templateDir) {
      const templatePath = path.join(options.templateDir, 'PROJECT.md');
      if (fs.existsSync(templatePath)) {
        try {
          const rawTemplate = fs.readFileSync(templatePath, 'utf-8');
          projectMdContent = rawTemplate
            .replace(/\[Project Name\]/g, options.name)
            .replace(/YYYY-MM-DD/g, new Date().toISOString().slice(0, 10))
            .replace(/\[e\.g\..*\]/g, stack);
        } catch {}
      }
    }
    fs.writeFileSync(path.join(targetDir, 'PROJECT.md'), projectMdContent, 'utf-8');

    // 2. .gitignore
    const gitignoreContent = [
      'node_modules/',
      '.next/',
      '.venv/',
      '__pycache__/',
      'dist/',
      'build/',
      'target/',
      '.env',
      '.env.local',
      '*.log',
    ].join('\n');
    fs.writeFileSync(path.join(targetDir, '.gitignore'), gitignoreContent, 'utf-8');

    // 3. README.md
    const monthYear = new Intl.DateTimeFormat('en-US', { month: 'long', year: 'numeric' }).format(new Date());
    const readmeContent = `# ${options.name}\n\nNew development project (${stack} stack).\n\nCreated: ${monthYear}\n`;
    fs.writeFileSync(path.join(targetDir, 'README.md'), readmeContent, 'utf-8');

    return {
      name: kebabName,
      path: targetDir,
      stack,
    };
  }

  public pause(options: PauseProjectOptions): PauseProjectResult {
    const activeRoot = options.config.projectRoots.active?.path;
    const pausedRoot = options.config.projectRoots.paused?.path;

    if (!activeRoot || !pausedRoot) {
      throw new ConfigInvalidError('Active or Paused roots not defined in config');
    }

    const kebab = toKebabCase(options.name);
    let sourcePath = path.join(activeRoot, kebab);

    if (!fs.existsSync(sourcePath)) {
      sourcePath = path.join(activeRoot, options.name);
      if (!fs.existsSync(sourcePath)) {
        throw new ProjectNotFoundError(`Project '${options.name}' not found in Active!`, 'NOT_FOUND');
      }
    }

    const projectName = path.basename(sourcePath);
    const targetPath = path.join(pausedRoot, projectName);

    if (!options.force && !isGitClean(sourcePath)) {
      throw new DirtyGitError(
        'Project has uncommitted git changes! Commit/stash first, or pass --force.'
      );
    }

    if (!fs.existsSync(pausedRoot)) {
      fs.mkdirSync(pausedRoot, { recursive: true });
    }

    if (options.prune) {
      pruneDirectory(sourcePath, options.config.cleanDeps?.targets);
    }

    fs.renameSync(sourcePath, targetPath);

    return {
      name: projectName,
      from: sourcePath,
      to: targetPath,
      pruned: Boolean(options.prune),
    };
  }

  public resume(options: ResumeProjectOptions): ResumeProjectResult {
    const activeRoot = options.config.projectRoots.active?.path;
    const pausedRoot = options.config.projectRoots.paused?.path;

    if (!activeRoot || !pausedRoot) {
      throw new ConfigInvalidError('Active or Paused roots not defined in config');
    }

    const kebab = toKebabCase(options.name);
    let sourcePath = path.join(pausedRoot, kebab);

    if (!fs.existsSync(sourcePath)) {
      sourcePath = path.join(pausedRoot, options.name);
      if (!fs.existsSync(sourcePath)) {
        throw new ProjectNotFoundError(`Project '${options.name}' not found in Paused!`, 'NOT_FOUND');
      }
    }

    const projectName = path.basename(sourcePath);
    const targetPath = path.join(activeRoot, projectName);

    if (fs.existsSync(targetPath)) {
      throw new RtbError(
        `A project named '${projectName}' already exists in Active!`,
        'ALREADY_EXISTS'
      );
    }

    fs.renameSync(sourcePath, targetPath);

    let installRan = false;
    if (options.install) {
      if (fs.existsSync(path.join(targetPath, 'package.json'))) {
        try {
          execSync('npm install', { cwd: targetPath, stdio: 'inherit' });
          installRan = true;
        } catch {}
      } else if (fs.existsSync(path.join(targetPath, 'requirements.txt'))) {
        try {
          execSync('pip install -r requirements.txt', { cwd: targetPath, stdio: 'inherit' });
          installRan = true;
        } catch {}
      }
    }

    return {
      name: projectName,
      from: sourcePath,
      to: targetPath,
      installed: installRan,
    };
  }
}
