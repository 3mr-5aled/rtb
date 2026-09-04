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
import { findProjectPathFuzzy } from '../navigation/fuzzy.js';

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

export interface ArchiveProjectOptions {
  name: string;
  config: RtbConfig;
  force?: boolean;
}

export interface ArchiveProjectResult {
  archived: boolean;
  project: string;
  archivePath: string;
}

export interface UnarchiveProjectOptions {
  archiveName: string;
  config: RtbConfig;
}

export interface UnarchiveProjectResult {
  unarchived: boolean;
  archive: string;
  destination: string;
}

export interface DeployProjectOptions {
  name: string;
  config: RtbConfig;
  targetEnvironment?: 'production' | 'staging' | string;
  from?: string;
}

export interface DeployProjectResult {
  deployed: boolean;
  name: string;
  target: string;
  from: string;
  to: string;
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

  public archive(options: ArchiveProjectOptions): ArchiveProjectResult {
    const matches = findProjectPathFuzzy(options.name, options.config);
    if (matches.length === 0) {
      throw new ProjectNotFoundError(`Project '${options.name}' not found!`);
    }

    const target = matches[0];
    const projectPath = target.path;
    const projectName = path.basename(projectPath);

    if (!options.force && !isGitClean(projectPath)) {
      throw new DirtyGitError(
        'Project has uncommitted git changes! Commit or stash first, or pass --force.'
      );
    }

    const backupRoot = options.config.backupRoot || path.join(path.dirname(projectPath), 'backup');
    const snapshotDir = path.join(backupRoot, 'project-snapshots');
    if (!fs.existsSync(snapshotDir)) {
      fs.mkdirSync(snapshotDir, { recursive: true });
    }

    const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19);
    const archiveFilename = `${projectName}-${timestamp}.tar.gz`;
    const archivePath = path.join(snapshotDir, archiveFilename);

    pruneDirectory(projectPath, options.config.cleanDeps?.targets);

    try {
      const parentDir = path.dirname(projectPath);
      execSync(`tar -czf "${archivePath}" "${projectName}"`, {
        cwd: parentDir,
        stdio: ['ignore', 'pipe', 'ignore'],
      });
    } catch (err: any) {
      throw new RtbError(`Failed to create archive: ${err.message}`, 'TAR_FAILED');
    }

    fs.rmSync(projectPath, { recursive: true, force: true });

    return {
      archived: true,
      project: projectName,
      archivePath,
    };
  }

  public unarchive(options: UnarchiveProjectOptions): UnarchiveProjectResult {
    const activeDir = options.config.projectRoots?.active?.path;
    if (!activeDir) {
      throw new ConfigInvalidError('Active project root not configured');
    }

    const backupRoot = options.config.backupRoot || path.join(path.dirname(activeDir), 'backup');
    const snapshotDir = path.join(backupRoot, 'project-snapshots');

    let targetArchive = options.archiveName;
    let archivePath = path.join(snapshotDir, targetArchive);

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
      throw new RtbError(`Archive '${options.archiveName}' not found in: ${snapshotDir}`, 'ARCHIVE_NOT_FOUND');
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
      throw new RtbError(`Failed to extract archive: ${err.message}`, 'TAR_FAILED');
    }

    return {
      unarchived: true,
      archive: archivePath,
      destination: activeDir,
    };
  }

  public deploy(options: DeployProjectOptions): DeployProjectResult {
    const projectRoots = options.config.projectRoots || {};
    const targetEnvironment = options.targetEnvironment || 'production';
    const targetRootEntry = projectRoots[targetEnvironment];

    if (!targetRootEntry?.path) {
      throw new ConfigInvalidError(`Target root '${targetEnvironment}' not configured in rtb.config.json`);
    }

    const kebabName = toKebabCase(options.name);

    const findProjectInRoot = (rootPath: string): string | null => {
      if (!fs.existsSync(rootPath)) return null;
      const candidateKebab = path.join(rootPath, kebabName);
      if (fs.existsSync(candidateKebab)) return candidateKebab;
      const candidateExact = path.join(rootPath, options.name);
      if (fs.existsSync(candidateExact)) return candidateExact;
      return null;
    };

    let sourcePath: string | null = null;
    let sourceLabel = 'Active';

    if (options.from) {
      const fromKey = options.from.toLowerCase();
      const fromEntry = projectRoots[fromKey];
      if (!fromEntry?.path || !fs.existsSync(fromEntry.path)) {
        throw new ConfigInvalidError(`Source root '${options.from}' not configured or does not exist`);
      }
      sourceLabel = fromEntry.label || options.from;
      sourcePath = findProjectInRoot(fromEntry.path);
      if (!sourcePath) {
        throw new ProjectNotFoundError(`Project '${kebabName}' not found in ${sourceLabel}!`);
      }
    } else {
      const activeRootEntry = projectRoots.active;
      if (activeRootEntry?.path) {
        sourcePath = findProjectInRoot(activeRootEntry.path);
      }

      if (!sourcePath && targetEnvironment === 'production') {
        const stagingEntry = projectRoots.staging;
        if (stagingEntry?.path) {
          const stagingCandidate = findProjectInRoot(stagingEntry.path);
          if (stagingCandidate) {
            sourcePath = stagingCandidate;
            sourceLabel = stagingEntry.label || 'Staging';
          }
        }
      }

      if (!sourcePath) {
        const searchRoots = targetEnvironment === 'production' && projectRoots.staging ? 'Active or Staging' : 'Active';
        throw new ProjectNotFoundError(`Project '${kebabName}' not found in ${searchRoots}!`);
      }
    }

    const projectName = path.basename(sourcePath);
    const deployRoot = targetRootEntry.path;
    if (!fs.existsSync(deployRoot)) {
      fs.mkdirSync(deployRoot, { recursive: true });
    }

    const destinationPath = path.join(deployRoot, projectName);

    if (fs.existsSync(destinationPath)) {
      throw new RtbError(`Destination path already exists: ${destinationPath}`, 'ALREADY_EXISTS');
    }

    try {
      fs.renameSync(sourcePath, destinationPath);
    } catch (err: any) {
      if (err.code === 'EXDEV') {
        fs.cpSync(sourcePath, destinationPath, { recursive: true });
        fs.rmSync(sourcePath, { recursive: true, force: true });
      } else {
        throw err;
      }
    }

    return {
      deployed: true,
      name: projectName,
      target: targetEnvironment,
      from: sourcePath,
      to: destinationPath,
    };
  }
}
