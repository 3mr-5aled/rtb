import fs from 'node:fs';
import path from 'node:path';

export type DependencyType =
  | 'npm/pnpm/yarn'
  | 'npm/pnpm (dev)'
  | 'Cargo (Rust)'
  | 'Python (pyproject)'
  | 'Python (requirements)'
  | 'Go (go.mod)'
  | 'Other';

export type DependencyStatus = 'Declared' | 'Outdated' | 'Vulnerable';

export interface DeclaredDependency {
  package: string;
  spec: string;
  type: DependencyType;
  status: DependencyStatus;
}

export function inspectDependencies(projectPath: string): DeclaredDependency[] {
  const depsList: DeclaredDependency[] = [];

  // 1. Node.js (package.json)
  const pkgJsonPath = path.join(projectPath, 'package.json');
  if (fs.existsSync(pkgJsonPath)) {
    try {
      const pkg = JSON.parse(fs.readFileSync(pkgJsonPath, 'utf8'));
      if (pkg.dependencies && typeof pkg.dependencies === 'object') {
        for (const [name, spec] of Object.entries(pkg.dependencies)) {
          depsList.push({
            package: name,
            spec: String(spec),
            type: 'npm/pnpm/yarn',
            status: 'Declared',
          });
        }
      }
      if (pkg.devDependencies && typeof pkg.devDependencies === 'object') {
        for (const [name, spec] of Object.entries(pkg.devDependencies)) {
          depsList.push({
            package: name,
            spec: String(spec),
            type: 'npm/pnpm (dev)',
            status: 'Declared',
          });
        }
      }
    } catch {}
  }

  // 2. Rust (Cargo.toml)
  const cargoPath = path.join(projectPath, 'Cargo.toml');
  if (fs.existsSync(cargoPath)) {
    try {
      const content = fs.readFileSync(cargoPath, 'utf8');
      const lines = content.split('\n');
      let inDeps = false;

      for (const rawLine of lines) {
        const line = rawLine.trim();
        if (line.startsWith('[dependencies]') || line.startsWith('[dev-dependencies]')) {
          inDeps = true;
          continue;
        } else if (line.startsWith('[')) {
          inDeps = false;
          continue;
        }

        if (inDeps && line && !line.startsWith('#')) {
          const match = line.match(/^([a-zA-Z0-9_-]+)\s*=\s*(?:"([^"]+)"|\{.*version\s*=\s*"([^"]+)".*\})/);
          if (match) {
            const spec = match[2] || match[3] || '*';
            depsList.push({
              package: match[1],
              spec,
              type: 'Cargo (Rust)',
              status: 'Declared',
            });
          }
        }
      }
    } catch {}
  }

  // 3. Python (pyproject.toml or requirements.txt)
  const pyprojectPath = path.join(projectPath, 'pyproject.toml');
  let pyprojectFound = false;
  if (fs.existsSync(pyprojectPath)) {
    try {
      const content = fs.readFileSync(pyprojectPath, 'utf8');
      const depBlockMatch = content.match(/dependencies\s*=\s*\[([\s\S]*?)\]/);
      if (depBlockMatch) {
        pyprojectFound = true;
        const entries = depBlockMatch[1].match(/"([^"]+)"/g);
        if (entries) {
          for (const rawEntry of entries) {
            const entry = rawEntry.replace(/"/g, '').trim();
            const parts = entry.match(/^([a-zA-Z0-9_-]+)(.*)$/);
            if (parts) {
              depsList.push({
                package: parts[1],
                spec: parts[2].trim() || '*',
                type: 'Python (pyproject)',
                status: 'Declared',
              });
            }
          }
        }
      }
    } catch {}
  }

  const reqsPath = path.join(projectPath, 'requirements.txt');
  if (!pyprojectFound && fs.existsSync(reqsPath)) {
    try {
      const content = fs.readFileSync(reqsPath, 'utf8');
      const lines = content.split('\n');
      for (const rawLine of lines) {
        const line = rawLine.trim();
        if (!line || line.startsWith('#')) continue;
        const parts = line.match(/^([a-zA-Z0-9_-]+)([<>=!~]+.*)?$/);
        if (parts) {
          depsList.push({
            package: parts[1],
            spec: parts[2] ? parts[2].trim() : '*',
            type: 'Python (requirements)',
            status: 'Declared',
          });
        }
      }
    } catch {}
  }

  // 4. Go (go.mod)
  const goModPath = path.join(projectPath, 'go.mod');
  if (fs.existsSync(goModPath)) {
    try {
      const content = fs.readFileSync(goModPath, 'utf8');
      const lines = content.split('\n');
      let inRequireBlock = false;

      const recordGoDep = (rawDepLine: string) => {
        const parts = rawDepLine.split(/\s+/);
        if (parts.length >= 2) {
          depsList.push({
            package: parts[0],
            spec: parts[1],
            type: 'Go (go.mod)',
            status: 'Declared',
          });
        }
      };

      for (const rawLine of lines) {
        const line = rawLine.trim();
        if (line === 'require (') {
          inRequireBlock = true;
          continue;
        } else if (inRequireBlock && line === ')') {
          inRequireBlock = false;
          continue;
        }

        if (inRequireBlock && line && !line.startsWith('//')) {
          recordGoDep(line);
        } else if (line.startsWith('require ') && !line.includes('(')) {
          recordGoDep(line.substring('require '.length).trim());
        }
      }
    } catch {}
  }

  return depsList;
}
