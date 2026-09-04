import fs from 'node:fs';
import path from 'node:path';

export interface DeclaredDependency {
  package: string;
  spec: string;
  type: string;
  status: string;
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
          // format: name = "version" or name = { version = "1.0", ... }
          const simpleMatch = line.match(/^([a-zA-Z0-9_-]+)\s*=\s*"([^"]+)"/);
          if (simpleMatch) {
            depsList.push({
              package: simpleMatch[1],
              spec: simpleMatch[2],
              type: 'Cargo (Rust)',
              status: 'Declared',
            });
            continue;
          }

          const inlineTableMatch = line.match(/^([a-zA-Z0-9_-]+)\s*=\s*\{.*version\s*=\s*"([^"]+)".*\}/);
          if (inlineTableMatch) {
            depsList.push({
              package: inlineTableMatch[1],
              spec: inlineTableMatch[2],
              type: 'Cargo (Rust)',
              status: 'Declared',
            });
            continue;
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
      // Look for dependencies = [ "req>=1.0", ... ]
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
          const parts = line.split(/\s+/);
          if (parts.length >= 2) {
            depsList.push({
              package: parts[0],
              spec: parts[1],
              type: 'Go (go.mod)',
              status: 'Declared',
            });
          }
        } else if (line.startsWith('require ') && !line.includes('(')) {
          const parts = line.substring('require '.length).trim().split(/\s+/);
          if (parts.length >= 2) {
            depsList.push({
              package: parts[0],
              spec: parts[1],
              type: 'Go (go.mod)',
              status: 'Declared',
            });
          }
        }
      }
    } catch {}
  }

  return depsList;
}
