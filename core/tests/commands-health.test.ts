import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { createCli } from '../src/cli.js';

describe('rtb health command integration', () => {
  let tmpDir: string;
  let activeDir: string;
  let configFile: string;

  beforeEach(() => {
    tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'rtb-cli-health-test-'));
    activeDir = path.join(tmpDir, '01-Active');
    fs.mkdirSync(activeDir, { recursive: true });

    configFile = path.join(tmpDir, 'rtb.config.json');
    fs.writeFileSync(
      configFile,
      JSON.stringify(
        {
          version: '0.5.3',
          projectRoots: {
            active: { path: activeDir, label: 'Active', emoji: '🚀' },
          },
          gitHealth: {
            scanRoots: [activeDir],
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

  it('outputs JSON health scan report when --json flag is passed', async () => {
    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});

    const cli = createCli();
    await cli.parseAsync(['node', 'rtb', 'health', '--config', configFile, '--json']);

    expect(logSpy).toHaveBeenCalled();
    const rawOutput = logSpy.mock.calls.map((c) => c.join(' ')).join('\n');
    const parsed = JSON.parse(rawOutput);

    expect(parsed).toHaveProperty('scannedCount');
    expect(parsed).toHaveProperty('issuesCount');
    expect(parsed).toHaveProperty('repos');
    expect(Array.isArray(parsed.repos)).toBe(true);
  });
});
