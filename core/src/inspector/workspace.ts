import fs from 'node:fs';
import path from 'node:path';

export type WorkspacePackageManagerType = 'pnpm' | 'npm/yarn' | 'Cargo';

export interface WorkspacePackage {
  packagePattern: string;
  type: WorkspacePackageManagerType;
}

export interface WorkspaceInfo {
  projectPath: string;
  workspaceType: string;
  isMonorepo: boolean;
  packages: WorkspacePackage[];
}

export function inspectWorkspace(projectPath: string): WorkspaceInfo {
  const workspacePackages: WorkspacePackage[] = [];
  let workspaceType = 'Single Package / Standard Repository';

  // 1. Check pnpm-workspace.yaml
  const pnpmWsPath = path.join(projectPath, 'pnpm-workspace.yaml');
  if (fs.existsSync(pnpmWsPath)) {
    workspaceType = 'pnpm Workspaces';
    try {
      const content = fs.readFileSync(pnpmWsPath, 'utf-8');
      const lines = content.split(/\r?\n/);
      let inPackagesSection = false;

      for (const rawLine of lines) {
        const line = rawLine.trim();
        if (line.startsWith('packages:')) {
          inPackagesSection = true;
          continue;
        }
        if (inPackagesSection) {
          if (line.startsWith('-')) {
            const pattern = line.replace(/^-\s*/, '').replace(/['"]/g, '').trim();
            if (pattern) {
              workspacePackages.push({
                packagePattern: pattern,
                type: 'pnpm',
              });
            }
          } else if (line && !line.startsWith('#')) {
            inPackagesSection = false;
          }
        }
      }
    } catch {}
  }

  // 2. Check package.json workspaces
  const pkgJsonPath = path.join(projectPath, 'package.json');
  if (fs.existsSync(pkgJsonPath)) {
    try {
      const rawPkg = fs.readFileSync(pkgJsonPath, 'utf-8');
      const pkg = JSON.parse(rawPkg);

      let workspacesList: string[] = [];
      if (Array.isArray(pkg.workspaces)) {
        workspacesList = pkg.workspaces;
      } else if (pkg.workspaces && Array.isArray(pkg.workspaces.packages)) {
        workspacesList = pkg.workspaces.packages;
      }

      if (workspacesList.length > 0) {
        workspaceType = 'npm/yarn Workspaces';
        for (const pattern of workspacesList) {
          if (typeof pattern === 'string' && pattern.trim()) {
            workspacePackages.push({
              packagePattern: pattern.trim(),
              type: 'npm/yarn',
            });
          }
        }
      }
    } catch {}
  }

  // 3. Check Cargo.toml workspace members
  const cargoPath = path.join(projectPath, 'Cargo.toml');
  if (fs.existsSync(cargoPath)) {
    try {
      const content = fs.readFileSync(cargoPath, 'utf-8');
      const lines = content.split(/\r?\n/);
      let inWorkspace = false;
      let inMembers = false;

      for (const rawLine of lines) {
        const line = rawLine.trim();
        if (line.startsWith('[workspace]')) {
          inWorkspace = true;
          continue;
        } else if (line.startsWith('[')) {
          inWorkspace = false;
          inMembers = false;
          continue;
        }

        if (inWorkspace) {
          if (line.startsWith('members')) {
            inMembers = true;
            workspaceType = 'Cargo Workspace (Rust)';
          }

          if (inMembers) {
            const memberMatches = line.match(/"([^"]+)"|'([^']+)'/g);
            if (memberMatches) {
              for (const match of memberMatches) {
                const cleaned = match.replace(/['"]/g, '').trim();
                if (cleaned) {
                  workspacePackages.push({
                    packagePattern: cleaned,
                    type: 'Cargo',
                  });
                }
              }
            }
            if (line.includes(']')) {
              inMembers = false;
            }
          }
        }
      }
    } catch {}
  }

  return {
    projectPath,
    workspaceType,
    isMonorepo: workspacePackages.length > 0,
    packages: workspacePackages,
  };
}
