import fs from 'node:fs';
import path from 'node:path';
import { execSync } from 'node:child_process';

export function isGitClean(projectPath: string): boolean {
  const gitDir = path.join(projectPath, '.git');
  if (!fs.existsSync(gitDir)) {
    return true; // Not a git repo, consider clean
  }

  try {
    const status = execSync('git status --porcelain', {
      cwd: projectPath,
      stdio: ['ignore', 'pipe', 'ignore'],
    })
      .toString()
      .trim();

    return status.length === 0;
  } catch {
    return true;
  }
}
