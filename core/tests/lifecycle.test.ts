import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import path from 'node:path';
import fs from 'node:fs';
import os from 'node:os';
import { createCli } from '../src/cli.js';
import { toKebabCase } from '../src/commands/new.js';
import { scanCleanTargets } from '../src/commands/clean.js';

describe('Project Lifecycle Commands', () => {
  const tmpDir = path.join(os.tmpdir(), `rtb-lifecycle-test-${Date.now()}`);
  const activeDir = path.join(tmpDir, '01-Active');
  const pausedDir = path.join(tmpDir, '04-Paused');
  const backupDir = path.join(tmpDir, '08-Backup');
  const configFile = path.join(tmpDir, 'rtb.config.json');

  beforeEach(() => {
    fs.mkdirSync(activeDir, { recursive: true });
    fs.mkdirSync(pausedDir, { recursive: true });
    fs.mkdirSync(backupDir, { recursive: true });

    fs.writeFileSync(
      configFile,
      JSON.stringify({
        version: '1.0.0',
        projectRoots: {
          active: { path: activeDir, label: 'Active', emoji: '📁' },
          paused: { path: pausedDir, label: 'Paused', emoji: '⏸️' },
        },
        backupRoot: backupDir,
        cleanDeps: {
          daysInactive: 30,
          targets: ['node_modules', '.venv', 'dist'],
        },
      })
    );
  });

  afterEach(() => {
    try {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    } catch {}
  });

  it('toKebabCase should format strings cleanly', () => {
    expect(toKebabCase('My Awesome Project')).toBe('my-awesome-project');
    expect(toKebabCase('Project_Name 123')).toBe('project-name-123');
  });

  it('rtb new should scaffold a project with .gitignore, README.md, and PROJECT.md', async () => {
    const cli = createCli();
    await cli.parseAsync(['node', 'rtb', '--config', configFile, 'new', 'demo-app', '--stack', 'nextjs']);

    const appDir = path.join(activeDir, 'demo-app');
    expect(fs.existsSync(appDir)).toBe(true);
    expect(fs.existsSync(path.join(appDir, 'README.md'))).toBe(true);
    expect(fs.existsSync(path.join(appDir, '.gitignore'))).toBe(true);
    expect(fs.existsSync(path.join(appDir, 'PROJECT.md'))).toBe(true);

    const readme = fs.readFileSync(path.join(appDir, 'README.md'), 'utf-8');
    expect(readme).toContain('nextjs');
  });

  it('rtb pause and resume should move project folders between Active and Paused', async () => {
    const cli = createCli();
    await cli.parseAsync(['node', 'rtb', '--config', configFile, 'new', 'toggle-proj']);

    const activeApp = path.join(activeDir, 'toggle-proj');
    const pausedApp = path.join(pausedDir, 'toggle-proj');
    expect(fs.existsSync(activeApp)).toBe(true);

    // Pause
    await cli.parseAsync(['node', 'rtb', '--config', configFile, 'pause', 'toggle-proj', '--force']);
    expect(fs.existsSync(activeApp)).toBe(false);
    expect(fs.existsSync(pausedApp)).toBe(true);

    // Resume
    await cli.parseAsync(['node', 'rtb', '--config', configFile, 'resume', 'toggle-proj']);
    expect(fs.existsSync(activeApp)).toBe(true);
    expect(fs.existsSync(pausedApp)).toBe(false);
  });

  it('scanCleanTargets should identify inactive dependency directories', () => {
    const sampleProj = path.join(activeDir, 'old-proj');
    const sampleNm = path.join(sampleProj, 'node_modules');
    fs.mkdirSync(sampleNm, { recursive: true });

    // Set mtime to 60 days ago
    const sixtyDaysAgo = new Date(Date.now() - 60 * 24 * 60 * 60 * 1000);
    fs.utimesSync(sampleNm, sixtyDaysAgo, sixtyDaysAgo);

    const targets = scanCleanTargets([activeDir], ['node_modules'], 30);
    expect(targets.length).toBe(1);
    expect(targets[0].project).toBe('old-proj');
    expect(targets[0].targetName).toBe('node_modules');
    expect(targets[0].daysInactive).toBeGreaterThanOrEqual(59);
  });
});
