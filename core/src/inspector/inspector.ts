import fs from 'node:fs';
import path from 'node:path';
import { execSync } from 'node:child_process';
import type { RtbConfig } from '../types/config.js';
import type { ProjectDetails, GitInfo } from '../types/project.js';

export function inspectProject(projectPath: string, status: string = 'Active'): ProjectDetails | null {
  if (!fs.existsSync(projectPath)) return null;

  const name = path.basename(projectPath);
  const stack: string[] = [];

  // 1. Package.json analysis
  const pkgPath = path.join(projectPath, 'package.json');
  let hasNode = false;
  let isMonorepo = false;
  let runtimeVersion: string | null = null;

  if (fs.existsSync(pkgPath)) {
    hasNode = true;
    try {
      const rawPkg = fs.readFileSync(pkgPath, 'utf-8');
      const pkg = JSON.parse(rawPkg);
      const allKeys = [
        ...Object.keys(pkg.dependencies || {}),
        ...Object.keys(pkg.devDependencies || {}),
      ];

      if (allKeys.includes('next')) stack.push('Next.js');
      else if (allKeys.includes('react')) stack.push('React');
      else if (allKeys.includes('vue')) stack.push('Vue');
      else if (allKeys.includes('vite')) stack.push('Vite');

      if (allKeys.includes('tailwindcss')) stack.push('Tailwind');
      if (allKeys.includes('prisma') || allKeys.includes('@prisma/client')) stack.push('Prisma');
      if (allKeys.includes('typescript')) stack.push('TypeScript');
      if (allKeys.includes('express')) stack.push('Express');
      else if (allKeys.includes('fastify')) stack.push('Fastify');

      if (pkg.workspaces) isMonorepo = true;
      if (pkg.engines && pkg.engines.node) runtimeVersion = String(pkg.engines.node);
    } catch {}

    if (!stack.some((s) => ['Next.js', 'React', 'Vue', 'Vite', 'Node.js'].includes(s))) {
      stack.push('Node.js');
    }
  }

  // 2. Python, Rust, Go, Java, Docker, PowerShell
  if (
    fs.existsSync(path.join(projectPath, 'uv.lock')) ||
    fs.existsSync(path.join(projectPath, 'poetry.lock')) ||
    fs.existsSync(path.join(projectPath, 'requirements.txt')) ||
    fs.existsSync(path.join(projectPath, 'pyproject.toml'))
  ) {
    stack.push('Python');
  }

  if (fs.existsSync(path.join(projectPath, 'Cargo.toml'))) stack.push('Rust');
  if (fs.existsSync(path.join(projectPath, 'go.mod'))) stack.push('Go');
  if (fs.existsSync(path.join(projectPath, 'pom.xml')) || fs.existsSync(path.join(projectPath, 'build.gradle'))) {
    stack.push('Java');
  }
  if (fs.existsSync(path.join(projectPath, 'Dockerfile'))) stack.push('Docker');
  if (
    fs.existsSync(path.join(projectPath, 'docker-compose.yml')) ||
    fs.existsSync(path.join(projectPath, 'docker-compose.yaml'))
  ) {
    stack.push('Compose');
  }
  if (
    fs.existsSync(path.join(projectPath, 'rtb.psm1')) ||
    fs.existsSync(path.join(projectPath, 'rtb.psd1')) ||
    fs.existsSync(path.join(projectPath, 'dev.psm1'))
  ) {
    stack.push('PowerShell');
  }

  // .NET check
  try {
    const files = fs.readdirSync(projectPath);
    if (files.some((f) => f.endsWith('.csproj') || f.endsWith('.sln'))) {
      stack.push('.NET');
    }
  } catch {}

  if (stack.length === 0) stack.push('-');

  // 3. Monorepo checks
  if (!isMonorepo) {
    isMonorepo =
      fs.existsSync(path.join(projectPath, 'pnpm-workspace.yaml')) ||
      fs.existsSync(path.join(projectPath, 'lerna.json')) ||
      fs.existsSync(path.join(projectPath, 'nx.json')) ||
      fs.existsSync(path.join(projectPath, 'turbo.json'));
  }

  // 4. CI/CD checks
  let ciCd: string | null = null;
  if (fs.existsSync(path.join(projectPath, '.github', 'workflows'))) {
    ciCd = 'GitHub Actions';
  } else if (fs.existsSync(path.join(projectPath, '.gitlab-ci.yml'))) {
    ciCd = 'GitLab CI';
  } else if (fs.existsSync(path.join(projectPath, 'azure-pipelines.yml'))) {
    ciCd = 'Azure Pipelines';
  } else if (fs.existsSync(path.join(projectPath, '.circleci'))) {
    ciCd = 'CircleCI';
  }

  // 5. Runtime version checks (.nvmrc, .python-version, rust-toolchain.toml)
  const nvmrcPath = path.join(projectPath, '.nvmrc');
  const pyverPath = path.join(projectPath, '.python-version');
  const rusttcPath = path.join(projectPath, 'rust-toolchain.toml');

  if (fs.existsSync(nvmrcPath)) {
    try {
      runtimeVersion = fs.readFileSync(nvmrcPath, 'utf-8').split(/\r?\n/)[0].trim();
    } catch {}
  } else if (fs.existsSync(pyverPath)) {
    try {
      runtimeVersion = fs.readFileSync(pyverPath, 'utf-8').split(/\r?\n/)[0].trim();
    } catch {}
  } else if (fs.existsSync(rusttcPath)) {
    try {
      const content = fs.readFileSync(rusttcPath, 'utf-8');
      for (const line of content.split(/\r?\n/)) {
        if (line.trim().startsWith('channel')) {
          const parts = line.split('=');
          if (parts.length > 1) {
            runtimeVersion = parts[1].trim().replace(/['"]/g, '');
            break;
          }
        }
      }
    } catch {}
  }

  // 6. Git telemetry
  let gitInfo: GitInfo | null = null;
  const gitDir = path.join(projectPath, '.git');
  if (fs.existsSync(gitDir)) {
    try {
      const branch = execSync('git branch --show-current', { cwd: projectPath, stdio: ['ignore', 'pipe', 'ignore'] })
        .toString()
        .trim() || 'unknown';

      const statusOutput = execSync('git status --porcelain', { cwd: projectPath, stdio: ['ignore', 'pipe', 'ignore'] })
        .toString()
        .trim();
      const uncommitted = statusOutput ? statusOutput.split('\n').filter(Boolean).length : 0;

      let unpushed = 0;
      try {
        const unpushedOutput = execSync('git log @{u}.. --oneline', { cwd: projectPath, stdio: ['ignore', 'pipe', 'ignore'] })
          .toString()
          .trim();
        unpushed = unpushedOutput ? unpushedOutput.split('\n').filter(Boolean).length : 0;
      } catch {}

      const lastCommitMsg = execSync('git log -1 --format="%s"', { cwd: projectPath, stdio: ['ignore', 'pipe', 'ignore'] })
        .toString()
        .trim();
      const lastCommitRelative = execSync('git log -1 --format="%cr"', { cwd: projectPath, stdio: ['ignore', 'pipe', 'ignore'] })
        .toString()
        .trim();

      const remotes = execSync('git remote', { cwd: projectPath, stdio: ['ignore', 'pipe', 'ignore'] })
        .toString()
        .trim();
      const hasRemote = remotes.length > 0;

      gitInfo = {
        branch,
        uncommitted,
        unpushed,
        last_commit_msg: lastCommitMsg,
        last_commit_relative: lastCommitRelative,
        has_remote: hasRemote,
      };
    } catch {}
  }

  // 7. Readme preview (first 6 lines)
  let readmePreview: string | null = null;
  for (const rName of ['README.md', 'readme.md', 'README.txt']) {
    const rPath = path.join(projectPath, rName);
    if (fs.existsSync(rPath)) {
      try {
        const lines = fs.readFileSync(rPath, 'utf-8').split(/\r?\n/).slice(0, 6);
        readmePreview = lines.join('\n').trim();
        break;
      } catch {}
    }
  }

  // 8. Last modified timestamp (fast stat from top directory or files)
  let lastModified: string | null = null;
  try {
    const stat = fs.statSync(projectPath);
    lastModified = stat.mtime.toISOString().replace(/\.\d{3}Z$/, '');
  } catch {}

  return {
    name,
    path: projectPath,
    status,
    stack,
    last_modified: lastModified,
    total_size_bytes: 0,
    dep_size_bytes: 0,
    git: gitInfo,
    readme_preview: readmePreview,
    is_monorepo: isMonorepo,
    ci_cd: ciCd,
    runtime_version: runtimeVersion,
  };
}

export interface ScanCategoryInfo {
  key: string;
  label: string;
  emoji: string;
  count: number;
}

export interface ScanAllProjectsCallbacks {
  onCategory?: (category: ScanCategoryInfo) => void;
  onProject?: (project: ProjectDetails, category: ScanCategoryInfo) => void;
  onCategoryEnd?: (category: ScanCategoryInfo) => void;
}

export function scanAllProjects(
  config: RtbConfig,
  filter: string = 'all',
  callbacks?: ScanAllProjectsCallbacks | ((project: ProjectDetails) => void)
): ProjectDetails[] {
  const normFilter = filter.toLowerCase();
  const results: ProjectDetails[] = [];

  const callbacksObj: ScanAllProjectsCallbacks =
    typeof callbacks === 'function' ? { onProject: callbacks } : callbacks || {};

  const shouldInclude = (categoryKey: string): boolean => {
    if (normFilter === 'all') return true;
    if (normFilter === 'active') return categoryKey === 'active';
    if (normFilter === 'paused') return categoryKey === 'paused';
    if (normFilter === 'deployed') return categoryKey === 'production' || categoryKey === 'staging';
    if (normFilter === 'vibe') return categoryKey === 'vibe';
    return categoryKey === normFilter;
  };

  for (const [key, entry] of Object.entries(config.projectRoots)) {
    if (!shouldInclude(key)) continue;
    if (!entry.path || !fs.existsSync(entry.path)) continue;

    try {
      const items = fs.readdirSync(entry.path, { withFileTypes: true });
      const dirItems = items.filter((item) => item.isDirectory() && !item.name.startsWith('.'));
      if (dirItems.length === 0) continue;

      const catInfo: ScanCategoryInfo = {
        key,
        label: entry.label || key,
        emoji: entry.emoji || '📁',
        count: dirItems.length,
      };

      callbacksObj.onCategory?.(catInfo);

      for (const item of dirItems) {
        const projectPath = path.join(entry.path, item.name);
        const details = inspectProject(projectPath, entry.label || key);
        if (details) {
          results.push(details);
          callbacksObj.onProject?.(details, catInfo);
        }
      }

      callbacksObj.onCategoryEnd?.(catInfo);
    } catch {}
  }

  return results;
}
