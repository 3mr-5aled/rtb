import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { createCli } from '../src/cli.js';

describe('rtb workspace command integration', () => {
  let tmpDir: string;
  let activeDir: string;
  let configFile: string;

  beforeEach(() => {
    tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'rtb-cli-ws-test-'));
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

  it('outputs JSON workspace info when --json flag is passed', async () => {
    const projDir = path.join(activeDir, 'my-monorepo');
    fs.mkdirSync(projDir, { recursive: true });
    fs.writeFileSync(
      path.join(projDir, 'package.json'),
      JSON.stringify({ workspaces: ['apps/*', 'packages/*'] }, null, 2)
    );

    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});

    const cli = createCli();
    await cli.parseAsync(['node', 'rtb', 'workspace', 'my-monorepo', '--config', configFile, '--json']);

    expect(logSpy).toHaveBeenCalled();
    const rawOutput = logSpy.mock.calls.map((c) => c.join(' ')).join('\n');
    const parsed = JSON.parse(rawOutput);

    expect(parsed.isMonorepo).toBe(true);
    expect(parsed.workspaceType).toBe('npm/yarn Workspaces');
    expect(parsed.packages).toHaveLength(2);
  });
});
