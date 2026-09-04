import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { createCli } from '../src/cli.js';

describe('rtb list command integration', () => {
  let tmpDir: string;
  let activeDir: string;
  let pausedDir: string;
  let configFile: string;

  beforeEach(() => {
    tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'rtb-cli-list-test-'));
    activeDir = path.join(tmpDir, '01-Active');
    pausedDir = path.join(tmpDir, '04-Paused');
    fs.mkdirSync(activeDir, { recursive: true });
    fs.mkdirSync(pausedDir, { recursive: true });

    configFile = path.join(tmpDir, 'rtb.config.json');
    fs.writeFileSync(
      configFile,
      JSON.stringify(
        {
          version: '0.6.0',
          projectRoots: {
            active: { path: activeDir, label: 'Active', emoji: '📁' },
            paused: { path: pausedDir, label: 'Paused', emoji: '⏸️' },
          },
        },
        null,
        2
      )
    );
  });

  afterEach(() => {
    fs.rmSync(tmpDir, { recursive: true, force: true });
    vi.restoreAllMocks();
  });

  it('outputs JSON project list when --json flag is passed', async () => {
    const projDir = path.join(activeDir, 'json-proj');
    fs.mkdirSync(projDir, { recursive: true });

    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});

    const cli = createCli();
    await cli.parseAsync(['node', 'rtb', 'list', '--config', configFile, '--json']);

    expect(logSpy).toHaveBeenCalled();
    const rawOutput = logSpy.mock.calls.map((c) => c.join(' ')).join('\n');
    const parsed = JSON.parse(rawOutput);

    expect(Array.isArray(parsed)).toBe(true);
    expect(parsed.length).toBe(1);
    expect(parsed[0].name).toBe('json-proj');
  });

  it('streams project discoveries one by one with banner and category headers in human-readable mode', async () => {
    const proj1 = path.join(activeDir, 'project-alpha');
    fs.mkdirSync(proj1, { recursive: true });
    fs.writeFileSync(
      path.join(proj1, 'package.json'),
      JSON.stringify({ name: 'project-alpha', dependencies: { react: '^18.0.0' } })
    );

    const proj2 = path.join(pausedDir, 'project-beta');
    fs.mkdirSync(proj2, { recursive: true });

    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});

    const cli = createCli();
    await cli.parseAsync(['node', 'rtb', 'list', '--config', configFile]);

    const calls = logSpy.mock.calls.map((c) => c.join(' ')).join('\n');
    expect(calls).toContain('Project List');
    expect(calls).toContain('📁 Active (1)');
    expect(calls).toContain('project-alpha');
    expect(calls).toContain('React');
    expect(calls).toContain('⏸️ Paused (1)');
    expect(calls).toContain('project-beta');
    expect(calls).toContain('Total: 2 projects');
  });

  it('outputs verbose project info when --verbose is passed', async () => {
    const proj = path.join(activeDir, 'verbose-app');
    fs.mkdirSync(proj, { recursive: true });
    fs.writeFileSync(
      path.join(proj, 'package.json'),
      JSON.stringify({ name: 'verbose-app', engines: { node: '>=20.0.0' } })
    );

    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});

    const cli = createCli();
    await cli.parseAsync(['node', 'rtb', 'list', '--config', configFile, '--verbose']);

    const calls = logSpy.mock.calls.map((c) => c.join(' ')).join('\n');
    expect(calls).toContain('verbose-app');
    expect(calls).toContain('Path:');
    expect(calls).toContain('Runtime:');
  });

  it('filters active projects only when --active is passed', async () => {
    fs.mkdirSync(path.join(activeDir, 'active-only'), { recursive: true });
    fs.mkdirSync(path.join(pausedDir, 'paused-only'), { recursive: true });

    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});

    const cli = createCli();
    await cli.parseAsync(['node', 'rtb', 'list', '--config', configFile, '--active']);

    const calls = logSpy.mock.calls.map((c) => c.join(' ')).join('\n');
    expect(calls).toContain('active-only');
    expect(calls).not.toContain('paused-only');
    expect(calls).toContain('Total: 1 projects');
  });
});
