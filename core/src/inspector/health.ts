import fs from 'node:fs';
import path from 'node:path';
import { execSync } from 'node:child_process';

export type HealthIssueType = 'UNCOMMITTED' | 'UNPUSHED' | 'STALE' | 'NO REMOTE';


export interface HealthIssue {
  type: HealthIssueType;
  message: string;
  isCritical: boolean;
}

export interface RepoHealthResult {
  repoPath: string;
  repoName: string;
  lastCommitRelative: string;
  lastCommitDate: string | null;
  issues: HealthIssue[];
}

export interface HealthReport {
  scannedCount: number;
  issuesCount: number;
  repos: RepoHealthResult[];
}

function findGitRepos(dir: string, repos: string[]): void {
  if (!fs.existsSync(dir)) return;

  try {
    const entries = fs.readdirSync(dir, { withFileTypes: true });
    const hasGit = entries.some((e) => e.isDirectory() && e.name === '.git');

    if (hasGit) {
      repos.push(dir);
      return; // Do not recurse into subdirectories of a git repo
    }

    for (const entry of entries) {
      if (entry.isDirectory()) {
        if (entry.name === 'node_modules' || entry.name === '.git' || entry.name === 'target' || entry.name === 'vendor') {
          continue;
        }
        findGitRepos(path.join(dir, entry.name), repos);
      }
    }
  } catch {}
}

export function scanGitHealth(scanRoots: string[], staleThresholdDays: number = 30): HealthReport {
  const repoPaths: string[] = [];

  for (const root of scanRoots) {
    findGitRepos(root, repoPaths);
  }

  const results: RepoHealthResult[] = [];
  let totalIssues = 0;

  for (const repoPath of repoPaths) {
    const issues: HealthIssue[] = [];
    const repoName = path.basename(repoPath);

    // 1. Check uncommitted changes
    try {
      const statusRaw = execSync('git status --porcelain', {
        cwd: repoPath,
        stdio: ['ignore', 'pipe', 'ignore'],
      })
        .toString()
        .trim();

      if (statusRaw) {
        const fileCount = statusRaw.split('\n').filter(Boolean).length;
        issues.push({
          type: 'UNCOMMITTED',
          message: `UNCOMMITTED (${fileCount} files)`,
          isCritical: true,
        });
      }
    } catch {}

    // 2. Remote check & unpushed commits
    let hasRemote = false;
    try {
      const remotes = execSync('git remote', {
        cwd: repoPath,
        stdio: ['ignore', 'pipe', 'ignore'],
      })
        .toString()
        .trim();

      if (remotes) {
        hasRemote = true;
      } else {
        issues.push({
          type: 'NO REMOTE',
          message: 'NO REMOTE',
          isCritical: false,
        });
      }
    } catch {
      issues.push({
        type: 'NO REMOTE',
        message: 'NO REMOTE',
        isCritical: false,
      });
    }

    // Check unpushed commits if remote exists
    if (hasRemote) {
      try {
        let unpushedRaw = '';
        try {
          unpushedRaw = execSync('git log @{u}.. --oneline', {
            cwd: repoPath,
            stdio: ['ignore', 'pipe', 'ignore'],
          })
            .toString()
            .trim();
        } catch {
          // If upstream is not configured, check if any branch commits are unpushed
          unpushedRaw = execSync('git log --branches --not --remotes --oneline', {
            cwd: repoPath,
            stdio: ['ignore', 'pipe', 'ignore'],
          })
            .toString()
            .trim();
        }

        if (unpushedRaw) {
          const unpushedCount = unpushedRaw.split('\n').filter(Boolean).length;
          issues.push({
            type: 'UNPUSHED',
            message: `UNPUSHED (${unpushedCount})`,
            isCritical: true,
          });
        }
      } catch {}
    }

    // 3. Last commit date & staleness
    let lastCommitRelative = 'Never';
    let lastCommitDate: string | null = null;
    try {
      lastCommitRelative = execSync('git log -1 --format="%cr"', {
        cwd: repoPath,
        stdio: ['ignore', 'pipe', 'ignore'],
      })
        .toString()
        .trim() || 'Never';

      const isoDate = execSync('git log -1 --format="%ai"', {
        cwd: repoPath,
        stdio: ['ignore', 'pipe', 'ignore'],
      })
        .toString()
        .trim();

      if (isoDate) {
        lastCommitDate = isoDate;
        const commitTime = new Date(isoDate).getTime();
        if (!Number.isNaN(commitTime)) {
          const diffDays = Math.floor((Date.now() - commitTime) / (1000 * 60 * 60 * 24));
          if (diffDays > staleThresholdDays) {
            issues.push({
              type: 'STALE',
              message: `STALE (${diffDays} days)`,
              isCritical: false,
            });
          }
        }
      }
    } catch {}

    if (issues.length > 0) {
      totalIssues++;
    }

    results.push({
      repoPath,
      repoName,
      lastCommitRelative,
      lastCommitDate,
      issues,
    });
  }

  return {
    scannedCount: results.length,
    issuesCount: totalIssues,
    repos: results,
  };
}
