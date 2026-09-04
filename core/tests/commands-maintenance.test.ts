import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { createCli } from '../src/cli.js';

describe('Maintenance commands integration (maintenance, backup, env, guard)', () => {
  let tmpDir: string;
  let activeDir: string;
  let backupDir: string;
  let configFile: string;

  beforeEach(() => {
    tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'rtb-cli-maint-test-'));
    activeDir = path.join(tmpDir, '01-Active');
    backupDir = path.join(tmpDir, 'backups');
    fs.mkdirSync(activeDir, { recursive: true });
    fs.mkdirSync(backupDir, { recursive: true });

    configFile = path.join(tmpDir, 'rtb.config.json');
    fs.writeFileSync(
      configFile,
      JSON.stringify(
        {
          version: '0.5.3',
          projectRoots: {
            active: { path: activeDir, label: 'Active', emoji: '🚀' },
          },
          backupRoot: backupDir,
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

  it('rtb backup --json outputs backup result', async () => {
    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});

    const cli = createCli();
    await cli.parseAsync(['node', 'rtb', 'backup', '--config', configFile, '--json']);

    expect(logSpy).toHaveBeenCalled();
    const rawOutput = logSpy.mock.calls.map((c) => c.join(' ')).join('\n');
    const parsed = JSON.parse(rawOutput);

    expect(parsed.task).toBe('backup');
    expect(parsed.success).toBe(true);
  });

  it('rtb env --json backs up .env files and outputs json', async () => {
    const projDir = path.join(activeDir, 'my-proj');
    fs.mkdirSync(projDir, { recursive: true });
    fs.writeFileSync(path.join(projDir, '.env'), 'KEY=VALUE');

    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});

    const cli = createCli();
    await cli.parseAsync(['node', 'rtb', 'env', '--config', configFile, '--json']);

    expect(logSpy).toHaveBeenCalled();
    const rawOutput = logSpy.mock.calls.map((c) => c.join(' ')).join('\n');
    const parsed = JSON.parse(rawOutput);

    expect(parsed.task).toBe('env');
    expect(parsed.success).toBe(true);
  });

  it('rtb guard --json inspects root directory', async () => {
    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});

    const cli = createCli();
    await cli.parseAsync(['node', 'rtb', 'guard', '--config', configFile, '--json']);

    expect(logSpy).toHaveBeenCalled();
    const rawOutput = logSpy.mock.calls.map((c) => c.join(' ')).join('\n');
    const parsed = JSON.parse(rawOutput);

    expect(parsed.task).toBe('guard');
    expect(parsed.success).toBe(true);
  });

  it('rtb maintenance --json runs all tasks', async () => {
    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});

    const cli = createCli();
    await cli.parseAsync(['node', 'rtb', 'maintenance', '--config', configFile, '--json']);

    expect(logSpy).toHaveBeenCalled();
    const rawOutput = logSpy.mock.calls.map((c) => c.join(' ')).join('\n');
    const parsed = JSON.parse(rawOutput);

    expect(parsed.success).toBe(true);
    expect(parsed.results.length).toBeGreaterThanOrEqual(3);
  });

  it('rtb maintenance <task> --json runs single task', async () => {
    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});

    const cli = createCli();
    await cli.parseAsync(['node', 'rtb', 'maintenance', 'backup', '--config', configFile, '--json']);

    expect(logSpy).toHaveBeenCalled();
    const rawOutput = logSpy.mock.calls.map((c) => c.join(' ')).join('\n');
    const parsed = JSON.parse(rawOutput);

    expect(parsed.task).toBe('backup');
    expect(parsed.success).toBe(true);
  });
});
