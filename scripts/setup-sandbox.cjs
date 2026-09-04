const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const sandbox = path.resolve('.sandbox');
if (fs.existsSync(sandbox)) fs.rmSync(sandbox, { recursive: true, force: true });
fs.mkdirSync(sandbox, { recursive: true });

const active = path.join(sandbox, 'roots', '01-Active');
const paused = path.join(sandbox, 'roots', '04-Paused');
const prod = path.join(sandbox, 'roots', '01-Production');
const stage = path.join(sandbox, 'roots', '02-Staging');
const vibe = path.join(sandbox, 'roots', '03-Vibe');
const backup = path.join(sandbox, 'backup');

[active, paused, prod, stage, vibe, backup].forEach((d) => fs.mkdirSync(d, { recursive: true }));

// 1. web-nextjs
const nextDir = path.join(active, 'web-nextjs');
fs.mkdirSync(nextDir, { recursive: true });
fs.writeFileSync(
  path.join(nextDir, 'package.json'),
  JSON.stringify(
    {
      name: 'web-nextjs',
      version: '1.0.0',
      scripts: { dev: 'echo running-dev', build: 'echo running-build', test: 'echo running-test' },
      dependencies: { next: '^14.0.0', react: '^18.0.0', tailwindcss: '^3.0.0' },
      devDependencies: { typescript: '^5.0.0' },
    },
    null,
    2
  )
);

// 2. api-python
const pyDir = path.join(active, 'api-python');
fs.mkdirSync(pyDir, { recursive: true });
fs.writeFileSync(path.join(pyDir, 'pyproject.toml'), '[tool.poetry]\nname = "api-python"\nversion = "0.1.0"\n');
fs.writeFileSync(path.join(pyDir, 'requirements.txt'), 'fastapi==0.100.0\nuvicorn==0.23.0\n');
fs.writeFileSync(path.join(pyDir, 'main.py'), 'print("Hello Python")\n');

// 3. cli-rust
const rustDir = path.join(active, 'cli-rust');
fs.mkdirSync(rustDir, { recursive: true });
fs.writeFileSync(path.join(rustDir, 'Cargo.toml'), '[package]\nname = "cli-rust"\nversion = "0.1.0"\nedition = "2021"\n');
fs.mkdirSync(path.join(rustDir, 'src'), { recursive: true });
fs.writeFileSync(path.join(rustDir, 'src', 'main.rs'), 'fn main() { println!("Hello Rust"); }\n');

// 4. clean-git & dirty-git
const cleanGit = path.join(active, 'clean-git');
fs.mkdirSync(cleanGit, { recursive: true });
try {
  execSync('git init -b main', { cwd: cleanGit, stdio: 'ignore' });
  execSync('git config user.email test@test.com', { cwd: cleanGit, stdio: 'ignore' });
  execSync('git config user.name Test', { cwd: cleanGit, stdio: 'ignore' });
  fs.writeFileSync(path.join(cleanGit, 'README.md'), '# Clean Repo\n');
  execSync('git add . && git commit -m "initial commit"', { cwd: cleanGit, stdio: 'ignore' });
} catch (e) {}

const dirtyGit = path.join(active, 'dirty-git');
fs.mkdirSync(dirtyGit, { recursive: true });
try {
  execSync('git init -b main', { cwd: dirtyGit, stdio: 'ignore' });
  execSync('git config user.email test@test.com', { cwd: dirtyGit, stdio: 'ignore' });
  execSync('git config user.name Test', { cwd: dirtyGit, stdio: 'ignore' });
  fs.writeFileSync(path.join(dirtyGit, 'README.md'), '# Dirty Repo\n');
  execSync('git add . && git commit -m "initial commit"', { cwd: dirtyGit, stdio: 'ignore' });
  fs.writeFileSync(path.join(dirtyGit, 'uncommitted.txt'), 'uncommitted content\n');
} catch (e) {}

// 5. monorepo-app
const monoDir = path.join(active, 'monorepo-app');
fs.mkdirSync(monoDir, { recursive: true });
fs.writeFileSync(
  path.join(monoDir, 'package.json'),
  JSON.stringify(
    {
      name: 'monorepo-app',
      workspaces: ['packages/*'],
    },
    null,
    2
  )
);
fs.writeFileSync(path.join(monoDir, 'pnpm-workspace.yaml'), 'packages:\n  - packages/*\n');
fs.mkdirSync(path.join(monoDir, 'packages', 'pkg-a'), { recursive: true });
fs.writeFileSync(
  path.join(monoDir, 'packages', 'pkg-a', 'package.json'),
  JSON.stringify({ name: '@mono/pkg-a', version: '1.0.0' }, null, 2)
);

// 6. stale-deps project with old node_modules
const staleDir = path.join(active, 'stale-deps');
fs.mkdirSync(staleDir, { recursive: true });
fs.writeFileSync(path.join(staleDir, 'package.json'), JSON.stringify({ name: 'stale-deps' }, null, 2));
const oldModules = path.join(staleDir, 'node_modules');
fs.mkdirSync(oldModules, { recursive: true });
const oldTime = Date.now() - 90 * 24 * 60 * 60 * 1000;
fs.utimesSync(oldModules, new Date(oldTime), new Date(oldTime));

// 7. Paused / Prod / Stage / Vibe projects
const pausedApp = path.join(paused, 'paused-app');
fs.mkdirSync(pausedApp, { recursive: true });
fs.writeFileSync(path.join(pausedApp, 'package.json'), JSON.stringify({ name: 'paused-app' }, null, 2));

const prodApp = path.join(prod, 'prod-app');
fs.mkdirSync(prodApp, { recursive: true });
fs.writeFileSync(path.join(prodApp, 'package.json'), JSON.stringify({ name: 'prod-app' }, null, 2));

const vibeApp = path.join(vibe, 'vibe-app');
fs.mkdirSync(vibeApp, { recursive: true });
fs.writeFileSync(path.join(vibeApp, 'package.json'), JSON.stringify({ name: 'vibe-app' }, null, 2));

// Config file
const config = {
  version: '1.0.0',
  projectRoots: {
    active: { path: active, label: 'Active', emoji: '🟢' },
    paused: { path: paused, label: 'Paused', emoji: '⏸️' },
    production: { path: prod, label: 'Production', emoji: '🚀' },
    staging: { path: stage, label: 'Staging', emoji: '🚀' },
    vibe: { path: vibe, label: 'Vibe', emoji: '✨' },
  },
  backupRoot: backup,
  staleThresholdDays: 30,
  cleanDeps: {
    daysInactive: 60,
    targets: ['node_modules', '.venv', 'target', 'dist'],
  },
  gitHealth: {
    scanRoots: [active],
  },
};

fs.writeFileSync(path.join(sandbox, 'rtb.config.json'), JSON.stringify(config, null, 2));
console.log('Sandbox created successfully at ' + sandbox);
