import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { createCli } from '../src/cli.js';
import * as openerModule from '../src/utils/opener.js';

describe('rtb open command integration', () => {
  let tmpDir: string;
  let activeDir: string;
  let configFile: string;

  beforeEach(() => {
    tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'rtb-cli-open-test-'));
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

  it('resolves project by name and invokes openPath', async () => {
    const projDir = path.join(activeDir, 'my-website');
    fs.mkdirSync(projDir, { recursive: true });

    const openPathSpy = vi.spyOn(openerModule, 'openPath').mockImplementation(() => {});

    const cli = createCli();
    await cli.parseAsync(['node', 'rtb', 'open', 'my-website', '--config', configFile]);

    expect(openPathSpy).toHaveBeenCalledWith(projDir);
  });

  it('reports error when project is not found', async () => {
    const errorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

    const cli = createCli();
    await cli.parseAsync(['node', 'rtb', 'open', 'nonexistent-proj', '--config', configFile]);

    expect(errorSpy).toHaveBeenCalled();
    const rawError = errorSpy.mock.calls.map((c) => c.join(' ')).join('\n');
    expect(rawError).toContain("Project or path 'nonexistent-proj' not found.");
  });
});
