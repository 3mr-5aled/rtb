import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { createCli } from '../src/cli.js';
import { performUninstall, cleanShellProfiles } from '../src/commands/uninstall.js';

describe('rtb uninstall command', () => {
  let tmpHome: string;
  let tmpConfigDir: string;
  let tmpBinDir: string;
  let sampleProfile: string;

  beforeEach(() => {
    tmpHome = fs.mkdtempSync(path.join(os.tmpdir(), 'rtb-uninstall-test-'));
    tmpConfigDir = path.join(tmpHome, '.config', 'rtb');
    tmpBinDir = path.join(tmpConfigDir, 'bin');

    fs.mkdirSync(tmpBinDir, { recursive: true });
    fs.writeFileSync(path.join(tmpConfigDir, 'rtb.config.json'), '{"version":"0.6.3"}');
    fs.writeFileSync(path.join(tmpBinDir, 'rtb.js'), '// test');
    fs.writeFileSync(path.join(tmpBinDir, 'rtb.cmd'), '@echo off');

    sampleProfile = path.join(tmpHome, '.bashrc');
    fs.writeFileSync(
      sampleProfile,
      'export FOO=1\n# RTB Shell Integration\neval "$(rtb shell-init bash)"\nexport BAR=2\n'
    );
  });

  afterEach(() => {
    fs.rmSync(tmpHome, { recursive: true, force: true });
    vi.restoreAllMocks();
  });

  it('fails gracefully in non-interactive mode without --force', async () => {
    const cli = createCli();
    await cli.parseAsync(['node', 'rtb', 'uninstall', '--json']);

    expect(process.exitCode).toBe(1);
    process.exitCode = 0;
  });

  it('uninstalls cleanly with --force and removes config by default', async () => {
    process.env.RTB_BIN_DIR = tmpBinDir;

    const res = performUninstall({
      keepConfig: false,
      customConfigDir: tmpConfigDir,
    });

    expect(fs.existsSync(tmpBinDir)).toBe(false);
    expect(fs.existsSync(tmpConfigDir)).toBe(false);
    expect(res.removedPaths).toContain(tmpBinDir);
    expect(res.removedPaths).toContain(tmpConfigDir);

    delete process.env.RTB_BIN_DIR;
  }, 15000);

  it('preserves configuration when --keep-config is passed', async () => {
    process.env.RTB_BIN_DIR = tmpBinDir;

    const res = performUninstall({
      keepConfig: true,
      customConfigDir: tmpConfigDir,
    });

    expect(fs.existsSync(tmpBinDir)).toBe(false);
    expect(fs.existsSync(tmpConfigDir)).toBe(true);
    expect(fs.existsSync(path.join(tmpConfigDir, 'rtb.config.json'))).toBe(true);
    expect(res.removedPaths).toContain(tmpBinDir);
    expect(res.removedPaths).not.toContain(tmpConfigDir);

    delete process.env.RTB_BIN_DIR;
  }, 15000);

  it('executes via CLI with --force --json', async () => {
    process.env.RTB_BIN_DIR = tmpBinDir;
    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});

    const cli = createCli();
    await cli.parseAsync([
      'node',
      'rtb',
      '--config',
      path.join(tmpConfigDir, 'rtb.config.json'),
      'uninstall',
      '--force',
      '--keep-config',
      '--json',
    ]);

    expect(logSpy).toHaveBeenCalled();
    const parsed = JSON.parse(logSpy.mock.calls[0][0]);
    expect(parsed.uninstalled).toBe(true);
    expect(parsed.keptConfig).toBe(true);

    delete process.env.RTB_BIN_DIR;
  });
});
