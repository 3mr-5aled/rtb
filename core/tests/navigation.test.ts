import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import path from 'node:path';
import fs from 'node:fs';
import os from 'node:os';
import { findProjectPathFuzzy } from '../src/navigation/fuzzy.js';
import type { RtbConfig } from '../types/config.js';

describe('Fuzzy Navigation Engine', () => {
  const tmpDir = path.join(os.tmpdir(), `rtb-fuzzy-test-${Date.now()}`);
  const activeDir = path.join(tmpDir, 'Active');

  beforeEach(() => {
    fs.mkdirSync(path.join(activeDir, 'rtb-command-tool'), { recursive: true });
    fs.mkdirSync(path.join(activeDir, 'rtb-extension'), { recursive: true });
    fs.mkdirSync(path.join(activeDir, 'another-project'), { recursive: true });
  });

  afterEach(() => {
    try {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    } catch {}
  });

  const config: RtbConfig = {
    version: '1.0.0',
    projectRoots: {
      active: { path: activeDir, label: 'Active', emoji: '📁' },
    },
  };

  it('should return score 100 on exact match', () => {
    const matches = findProjectPathFuzzy('rtb-command-tool', config);
    expect(matches.length).toBeGreaterThanOrEqual(1);
    expect(matches[0].name).toBe('rtb-command-tool');
    expect(matches[0].score).toBe(100);
  });

  it('should return score 75 on prefix match', () => {
    const matches = findProjectPathFuzzy('rtb', config);
    expect(matches.length).toBeGreaterThanOrEqual(2);
    expect(matches[0].score).toBe(75);
    expect(matches.map((m) => m.name)).toContain('rtb-command-tool');
    expect(matches.map((m) => m.name)).toContain('rtb-extension');
  });

  it('should return score 50 on substring match', () => {
    const matches = findProjectPathFuzzy('command', config);
    expect(matches.length).toBe(1);
    expect(matches[0].name).toBe('rtb-command-tool');
    expect(matches[0].score).toBe(50);
  });

  it('should return 0 matches for non-existent query', () => {
    const matches = findProjectPathFuzzy('completely_unknown_xyz', config);
    expect(matches.length).toBe(0);
  });
});

describe('rtb goto CLI integration', () => {
  let tmpDir: string;
  let activeDir: string;
  let configFile: string;

  beforeEach(() => {
    tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'rtb-cli-goto-test-'));
    activeDir = path.join(tmpDir, '01-Active');
    fs.mkdirSync(activeDir, { recursive: true });

    configFile = path.join(tmpDir, 'rtb.config.json');
    fs.writeFileSync(
      configFile,
      JSON.stringify(
        {
          version: '0.12.3',
          projectRoots: {
            active: { path: activeDir, label: 'Active', emoji: '🚀' },
          },
        },
        null,
        2
      )
    );
  });

  afterEach(() => {
    fs.rmSync(tmpDir, { recursive: true, force: true });
  });

  it('prints exact target path with --print flag', async () => {
    const { createCli } = await import('../src/cli.js');
    const projDir = path.join(activeDir, 'my-target-proj');
    fs.mkdirSync(projDir, { recursive: true });

    let stdoutData = '';
    const origWrite = process.stdout.write;
    process.stdout.write = ((chunk: any) => {
      stdoutData += chunk;
      return true;
    }) as any;

    try {
      const cli = createCli();
      await cli.parseAsync(['node', 'rtb', 'goto', 'my-target-proj', '--print', '--config', configFile]);
      expect(stdoutData.trim()).toBe(projDir);
    } finally {
      process.stdout.write = origWrite;
    }
  });

  it('prints target path when using fuzzy matching with --print', async () => {
    const { createCli } = await import('../src/cli.js');
    const projDir = path.join(activeDir, 'frontend-dashboard');
    fs.mkdirSync(projDir, { recursive: true });

    let stdoutData = '';
    const origWrite = process.stdout.write;
    process.stdout.write = ((chunk: any) => {
      stdoutData += chunk;
      return true;
    }) as any;

    try {
      const cli = createCli();
      await cli.parseAsync(['node', 'rtb', 'goto', 'dashboard', '--print', '--config', configFile]);
      expect(stdoutData.trim()).toBe(projDir);
    } finally {
      process.stdout.write = origWrite;
    }
  });

  it('sets process.exitCode = 1 on non-matching project with --print', async () => {
    const { createCli } = await import('../src/cli.js');
    const origCode = process.exitCode;
    let stdoutData = '';
    const origWrite = process.stdout.write;
    process.stdout.write = ((chunk: any) => {
      stdoutData += chunk;
      return true;
    }) as any;

    try {
      const cli = createCli();
      await cli.parseAsync(['node', 'rtb', 'goto', 'unknown-project', '--print', '--config', configFile]);
      expect(process.exitCode).toBe(1);
      expect(stdoutData).toBe('');
    } finally {
      process.stdout.write = origWrite;
      process.exitCode = origCode;
    }
  });

  it('supports --choice selection with --print flag', async () => {
    const { createCli } = await import('../src/cli.js');
    const projA = path.join(activeDir, 'proj-alpha');
    const projB = path.join(activeDir, 'proj-beta');
    fs.mkdirSync(projA, { recursive: true });
    fs.mkdirSync(projB, { recursive: true });

    let stdoutData = '';
    const origWrite = process.stdout.write;
    process.stdout.write = ((chunk: any) => {
      stdoutData += chunk;
      return true;
    }) as any;

    try {
      const cli = createCli();
      await cli.parseAsync(['node', 'rtb', 'goto', 'proj', '--choice', '2', '--print', '--config', configFile]);
      expect(stdoutData.trim()).toBe(projB);
    } finally {
      process.stdout.write = origWrite;
    }
  });
});
