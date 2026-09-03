import fs from 'node:fs';
import path from 'node:path';
import { execSync } from 'node:child_process';
import type { ProjectDetails } from '../types/project.js';
import { inspectProject } from '../inspector/inspector.js';

export function generateAgentContextFile(projectPath: string, details?: ProjectDetails | null): string {
  const info = details || inspectProject(projectPath) || {
    name: path.basename(projectPath),
    path: projectPath,
    status: 'Active',
    stack: ['-'],
    last_modified: null,
    total_size_bytes: 0,
    dep_size_bytes: 0,
    git: null,
    readme_preview: null,
    is_monorepo: false,
    ci_cd: null,
    runtime_version: null,
  };

  const name = info.name;
  const contextPath = path.join(projectPath, '.rtb_context.md');
  const stackStr = info.stack.filter((s) => s !== '-').join(', ') || 'Unknown';
  const branchStr = info.git?.branch || 'unknown';

  // Git Context
  let gitLogLines = '  (not a git repository)';
  let gitDiffStat = '  (not a git repository)';
  if (fs.existsSync(path.join(projectPath, '.git'))) {
    try {
      const logRaw = execSync('git log --oneline -10', {
        cwd: projectPath,
        stdio: ['ignore', 'pipe', 'ignore'],
      })
        .toString()
        .trim();
      gitLogLines = logRaw ? logRaw.split('\n').map((l) => `  ${l}`).join('\n') : '  (no commits)';
    } catch {
      gitLogLines = '  (no commits)';
    }

    try {
      const diffRaw = execSync('git diff --stat HEAD', {
        cwd: projectPath,
        stdio: ['ignore', 'pipe', 'ignore'],
      })
        .toString()
        .trim();
      gitDiffStat = diffRaw ? diffRaw.split('\n').map((l) => `  ${l}`).join('\n') : '  (working tree clean)';
    } catch {
      gitDiffStat = '  (working tree clean)';
    }
  }

  // Dependencies summary
  let depsSection = '';
  const pkgPath = path.join(projectPath, 'package.json');
  if (fs.existsSync(pkgPath)) {
    try {
      const pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf-8'));
      if (pkg.dependencies) {
        const deps = Object.keys(pkg.dependencies).slice(0, 20).join(', ');
        if (deps) depsSection += `**package.json deps:** ${deps}\n`;
      }
      if (pkg.devDependencies) {
        const devDeps = Object.keys(pkg.devDependencies).slice(0, 10).join(', ');
        if (devDeps) depsSection += `**devDependencies:** ${devDeps}\n`;
      }
    } catch {
      depsSection += `(could not parse package.json)\n`;
    }
  }

  const cargoPath = path.join(projectPath, 'Cargo.toml');
  if (fs.existsSync(cargoPath)) {
    try {
      const content = fs.readFileSync(cargoPath, 'utf-8');
      const matches = [...content.matchAll(/^\s*([a-zA-Z0-9_\-]+)\s*=/gm)];
      const crates = matches.slice(0, 20).map((m) => m[1]).join(', ');
      if (crates) depsSection += `**Cargo.toml crates:** ${crates}\n`;
    } catch {}
  }

  const reqsPath = path.join(projectPath, 'requirements.txt');
  if (fs.existsSync(reqsPath)) {
    try {
      const lines = fs.readFileSync(reqsPath, 'utf-8')
        .split(/\r?\n/)
        .map((l) => l.trim())
        .filter((l) => l && !l.startsWith('#'))
        .slice(0, 20);
      if (lines.length > 0) depsSection += `**requirements.txt:** ${lines.join(', ')}\n`;
    } catch {}
  }

  if (!depsSection.trim()) {
    depsSection = '(no recognised dependency manifest found)\n';
  }

  const readmeStr = info.readme_preview ? info.readme_preview.trim() : '(no README)';

  const content = `# RTB Agent Workspace Context: ${name}

## Project Info
- **Project Path**: ${projectPath}
- **Status**: ${info.status}
- **Detected Stack**: ${stackStr}
- **Git Branch**: ${branchStr}
- **Generated At**: ${new Date().toISOString()}

## README Preview
${readmeStr}

## Git Context

### Last 10 Commits
${gitLogLines}

### Current Diff (--stat HEAD)
${gitDiffStat}

## Dependencies
${depsSection}
`;

  fs.writeFileSync(contextPath, content, 'utf-8');
  return contextPath;
}
