import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { execSync } from 'node:child_process';
import { scanGitHealth } from '../src/inspector/health.js';

describe('scanGitHealth', () => {
  let tmpDir: string;
  let root1: string;

  beforeEach(() => {
    tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'rtb-health-test-'));
    root1 = path.join(tmpDir, 'root1');
    fs.mkdirSync(root1, { recursive: true });
  });

  afterEach(() => {
    fs.rmSync(tmpDir, { recursive: true, force: true });
  });

  it('scans git repositories and detects clean repo', () => {
    const bareDir = path.join(tmpDir, 'bare.git');
    execSync(`git init --bare "${bareDir}"`, { stdio: 'ignore' });

    const repoDir = path.join(root1, 'clean-repo');
    fs.mkdirSync(repoDir, { recursive: true });
    execSync('git init -b main', { cwd: repoDir, stdio: 'ignore' });
    execSync('git config user.email "test@example.com"', { cwd: repoDir, stdio: 'ignore' });
    execSync('git config user.name "Test"', { cwd: repoDir, stdio: 'ignore' });
    fs.writeFileSync(path.join(repoDir, 'README.md'), '# Clean Repo');
    execSync('git add . && git commit -m "initial commit"', { cwd: repoDir, stdio: 'ignore' });
    execSync(`git remote add origin "${bareDir}"`, { cwd: repoDir, stdio: 'ignore' });
    execSync('git push -u origin main', { cwd: repoDir, stdio: 'ignore' });

    const report = scanGitHealth([root1], 30);
    expect(report.scannedCount).toBe(1);
    expect(report.issuesCount).toBe(0);
    expect(report.repos).toHaveLength(1);
    expect(report.repos[0].issues).toHaveLength(0);
  });

  it('detects UNCOMMITTED and NO REMOTE issues', () => {
    const repoDir = path.join(root1, 'dirty-repo');
    fs.mkdirSync(repoDir, { recursive: true });
    execSync('git init -b main', { cwd: repoDir, stdio: 'ignore' });
    execSync('git config user.email "test@example.com"', { cwd: repoDir, stdio: 'ignore' });
    execSync('git config user.name "Test"', { cwd: repoDir, stdio: 'ignore' });
    fs.writeFileSync(path.join(repoDir, 'file.txt'), 'hello');

    const report = scanGitHealth([root1], 30);
    expect(report.scannedCount).toBe(1);
    expect(report.issuesCount).toBe(1);
    const repo = report.repos[0];
    expect(repo.issues.some((i) => i.type === 'UNCOMMITTED')).toBe(true);
    expect(repo.issues.some((i) => i.type === 'NO REMOTE')).toBe(true);
  });
});

