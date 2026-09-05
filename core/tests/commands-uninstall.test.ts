import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { createCli } from '../src/cli.js';
import { performUninstall, cleanShellProfiles, cleanProfileContent } from '../src/commands/uninstall.js';

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
  }, 15000);

  it('cleanProfileContent strips modern shell-init and legacy module imports across shells', () => {
    const bashContent = [
      '# User config',
      'export PATH="$HOME/bin:$PATH"',
      '# RTB Shell Integration',
      'eval "$(rtb shell-init bash)"',
      '(& rtb shell-init pwsh | Out-String) | Invoke-Expression',
      'Import-Module "$env:USERPROFILE\\AppData\\Roaming\\rtb\\module\\rtb.psd1"',
      '# RTB PowerShell Module',
      '# End config',
    ].join('\n');

    const cleaned = cleanProfileContent(bashContent);
    expect(cleaned).toContain('export PATH="$HOME/bin:$PATH"');
    expect(cleaned).toContain('# End config');
    expect(cleaned).not.toContain('rtb shell-init');
    expect(cleaned).not.toContain('Import-Module');
    expect(cleaned).not.toContain('# RTB Shell Integration');
    expect(cleaned).not.toContain('# RTB PowerShell Module');
  });

  it('cleanShellProfiles removes lines from provided profile paths and returns modified files', () => {
    const p1 = path.join(tmpHome, 'profile1.ps1');
    const p2 = path.join(tmpHome, 'profile2.sh');
    fs.writeFileSync(p1, 'Write-Host "Hi"\r\n(& rtb shell-init pwsh | Out-String) | Invoke-Expression\r\n');
    fs.writeFileSync(p2, 'echo "Clean file"\n');

    const modified = cleanShellProfiles([p1, p2]);
    expect(modified).toEqual([p1]);
    expect(fs.readFileSync(p1, 'utf-8')).not.toContain('rtb shell-init');
  });
});
