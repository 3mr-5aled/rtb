import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import path from 'node:path';
import fs from 'node:fs';
import os from 'node:os';
import { runDoctorChecks } from '../src/commands/doctor.js';
import { detectWorkspaceStatus } from '../src/commands/status.js';
import { createCli } from '../src/cli.js';
import type { CliContext } from '../types/context.js';

describe('Diagnostics & System Commands (doctor, status, info, ui)', () => {
  const tmpDir = path.join(os.tmpdir(), `rtb-diag-test-${Date.now()}`);
  const activeDir = path.join(tmpDir, '01-Active');
  const sampleProj = path.join(activeDir, 'sample-app');
  const configFile = path.join(tmpDir, 'rtb.config.json');

  beforeEach(() => {
    fs.mkdirSync(sampleProj, { recursive: true });
    fs.writeFileSync(
      configFile,
      JSON.stringify({
        version: '1.0.0',
        projectRoots: {
          active: { path: activeDir, label: 'Active', emoji: '📁' },
        },
      })
    );
    fs.writeFileSync(
      path.join(sampleProj, 'package.json'),
      JSON.stringify({ name: 'sample-app', dependencies: { react: '^19.0.0' } })
    );
  });

  afterEach(() => {
    try {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    } catch {}
  });

  it('runDoctorChecks should evaluate environment and return structured checks', () => {
    const mockContext: CliContext = {
      isJson: false,
      isInteractive: false,
      config: {
        version: '1.0.0',
        projectRoots: {
          active: { path: activeDir, label: 'Active', emoji: '📁' },
        },
      },
    };

    const { allGood, checks } = runDoctorChecks(mockContext);
    expect(checks.length).toBeGreaterThanOrEqual(5);

    const gitCheck = checks.find((c) => c.name.includes('git'));
    expect(gitCheck).toBeDefined();
    expect(gitCheck?.passed).toBe(true);

    const activeCheck = checks.find((c) => c.name.includes('Active'));
    expect(activeCheck).toBeDefined();
    expect(activeCheck?.passed).toBe(true);
  });

  it('detectWorkspaceStatus should detect current directory inside workspace', () => {
    const mockContext: CliContext = {
      isJson: false,
      isInteractive: false,
      config: {
        version: '1.0.0',
        projectRoots: {
          active: { path: activeDir, label: 'Active', emoji: '📁' },
        },
      },
    };

    const status = detectWorkspaceStatus(sampleProj, mockContext);
    expect(status.inWorkspace).toBe(true);
    expect(status.project?.name).toBe('sample-app');
    expect(status.project?.rootCategory).toBe('Active');
  });

  it('rtb doctor should run cleanly via CLI', async () => {
    const cli = createCli();
    let stdoutData = '';
    const origLog = console.log;
    console.log = (...args: any[]) => {
      stdoutData += args.join(' ') + '\n';
    };

    try {
      await cli.parseAsync(['node', 'rtb', '--config', configFile, 'doctor']);
      expect(stdoutData).toContain('System Doctor');
    } finally {
      console.log = origLog;
    }
  });

  it('rtb info should report project details in JSON mode', async () => {
    const cli = createCli();
    let stdoutData = '';
    const origLog = console.log;
    console.log = (...args: any[]) => {
      stdoutData += args.join(' ') + '\n';
    };

    try {
      await cli.parseAsync(['node', 'rtb', '--config', configFile, '--json', 'info', sampleProj]);
      const parsed = JSON.parse(stdoutData);
      expect(parsed.name).toBe('sample-app');
      expect(parsed.stack).toContain('React');
    } finally {
      console.log = origLog;
    }
  });
});
