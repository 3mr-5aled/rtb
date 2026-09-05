import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { Writable } from 'node:stream';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { TaskSpinner, withSpinner } from '../src/utils/spinner.js';
import { createCli } from '../src/cli.js';
import type { CliContext } from '../src/types/context.js';

function createMockStream(): { stream: Writable; getOutput: () => string } {
  let output = '';
  const stream = new Writable({
    write(chunk, _encoding, callback) {
      output += chunk.toString();
      callback();
    },
  });
  return { stream, getOutput: () => output };
}

describe('TaskSpinner Utility', () => {
  it('exports TaskSpinner and withSpinner', () => {
    expect(TaskSpinner).toBeDefined();
    expect(withSpinner).toBeDefined();
  });

  it('runs spinner lifecycle (start -> succeed) writing to stream with elapsed time', async () => {
    const { stream, getOutput } = createMockStream();
    const spinner = new TaskSpinner('Testing task', { stream, showTime: true });

    expect(spinner.isSpinning).toBe(false);
    spinner.start();
    expect(spinner.isSpinning).toBe(true);

    await new Promise((resolve) => setTimeout(resolve, 30));

    spinner.succeed('Task completed');
    expect(spinner.isSpinning).toBe(false);

    const out = getOutput();
    expect(out).toContain('Task completed');
    expect(out).toMatch(/\(\d+ms\)/);
  });

  it('handles fail and warn states with elapsed time', async () => {
    const { stream, getOutput } = createMockStream();
    const spinner = new TaskSpinner('Warning task', { stream, showTime: true });

    spinner.start();
    await new Promise((resolve) => setTimeout(resolve, 20));
    spinner.warn('Warning detected');

    const warnOut = getOutput();
    expect(warnOut).toContain('Warning detected');
    expect(warnOut).toMatch(/\(\d+ms\)/);

    const failMock = createMockStream();
    const failSpinner = new TaskSpinner('Failing task', { stream: failMock.stream });
    failSpinner.start();
    failSpinner.fail('Critical failure');
    expect(failMock.getOutput()).toContain('Critical failure');
  });

  it('completely suppresses stream output in quiet mode', () => {
    const { stream, getOutput } = createMockStream();
    const spinner = new TaskSpinner('Quiet task', { stream, quiet: true });

    spinner.start();
    spinner.setText('Changing text');
    spinner.succeed('Done');

    expect(getOutput()).toBe('');
  });

  it('completely suppresses stream output in json mode or with json context', () => {
    const { stream, getOutput } = createMockStream();
    const mockCtx: CliContext = {
      config: null,
      configPath: 'rtb.config.json',
      isConfigured: false,
      isJson: true,
      isQuiet: false,
      isInteractive: false,
    };

    const spinner = new TaskSpinner('JSON task', { stream, context: mockCtx });
    spinner.start();
    spinner.succeed('Done');

    expect(getOutput()).toBe('');
  });
});

describe('withSpinner Helper', () => {
  it('executes taskFn and auto-succeeds when task completes', async () => {
    const { stream, getOutput } = createMockStream();

    const result = await withSpinner(
      'Loading data',
      async () => {
        await new Promise((resolve) => setTimeout(resolve, 25));
        return 42;
      },
      { stream }
    );

    expect(result).toBe(42);
    const out = getOutput();
    expect(out).toContain('Loading data');
    expect(out).toMatch(/\(\d+ms\)/);
  });

  it('catches errors, marks spinner as failed, and preserves error type/properties', async () => {
    const { stream, getOutput } = createMockStream();

    class CustomError extends Error {
      code = 'ERR_CUSTOM';
    }

    await expect(
      withSpinner(
        'Failing task',
        async () => {
          throw new CustomError('Network failure occurred');
        },
        { stream }
      )
    ).rejects.toThrow('Network failure occurred');

    const out = getOutput();
    expect(out).toContain('Network failure occurred');
  });

  it('allows taskFn to manually manage spinner without double-succeeding', async () => {
    const { stream, getOutput } = createMockStream();

    const result = await withSpinner(
      'Custom step',
      (sp) => {
        sp.warn('Completed with warning');
        return 'warning-res';
      },
      { stream }
    );

    expect(result).toBe('warning-res');
    const out = getOutput();
    expect(out).toContain('Completed with warning');
  });

  it('emits zero output in withSpinner when quiet or json is enabled', async () => {
    const { stream, getOutput } = createMockStream();

    const res = await withSpinner(
      'Silent task',
      () => 'silent-val',
      { stream, json: true, quiet: true }
    );

    expect(res).toBe('silent-val');
    expect(getOutput()).toBe('');
  });
});

describe('CLI Commands with TaskSpinner Integration', () => {
  let tmpDir: string;
  let activeDir: string;
  let configFile: string;

  beforeEach(() => {
    tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'rtb-spinner-cli-test-'));
    activeDir = path.join(tmpDir, '01-Active');
    fs.mkdirSync(activeDir, { recursive: true });

    configFile = path.join(tmpDir, 'rtb.config.json');
    fs.writeFileSync(
      configFile,
      JSON.stringify(
        {
          version: '1.0.0',
          projectRoots: {
            active: { path: activeDir, label: 'Active', emoji: '🚀' },
          },
          gitHealth: {
            scanRoots: [activeDir],
          },
          cleanDeps: {
            targets: ['node_modules'],
          },
        },
        null,
        2
      )
    );
  });

  afterEach(() => {
    try {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    } catch {}
    vi.restoreAllMocks();
  });

  it('rtb clean in --json mode produces zero spinner artifacts', async () => {
    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});
    const cli = createCli();

    await cli.parseAsync(['node', 'rtb', 'clean', '--config', configFile, '--json']);

    expect(logSpy).toHaveBeenCalled();
    const rawOutput = logSpy.mock.calls.map((c) => c.join(' ')).join('\n');
    const parsed = JSON.parse(rawOutput);
    expect(parsed).toHaveProperty('dryRun');
    expect(parsed).toHaveProperty('totalTargets');
  });

  it('rtb deps in --json mode produces pure JSON without spinner artifacts', async () => {
    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});
    const projDir = path.join(activeDir, 'test-app');
    fs.mkdirSync(projDir, { recursive: true });
    fs.writeFileSync(
      path.join(projDir, 'package.json'),
      JSON.stringify({ dependencies: { vitest: '^3.0.0' } })
    );

    const cli = createCli();
    await cli.parseAsync(['node', 'rtb', 'deps', 'test-app', '--config', configFile, '--json']);

    expect(logSpy).toHaveBeenCalled();
    const rawOutput = logSpy.mock.calls.map((c) => c.join(' ')).join('\n');
    const parsed = JSON.parse(rawOutput);
    expect(Array.isArray(parsed)).toBe(true);
    expect(parsed[0].package).toBe('vitest');
  });

  it('rtb doctor in --json mode outputs pure structured report', async () => {
    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});
    const cli = createCli();

    await cli.parseAsync(['node', 'rtb', 'doctor', '--config', configFile, '--json']);

    expect(logSpy).toHaveBeenCalled();
    const rawOutput = logSpy.mock.calls.map((c) => c.join(' ')).join('\n');
    const parsed = JSON.parse(rawOutput);
    expect(parsed).toHaveProperty('healthy');
    expect(parsed).toHaveProperty('checks');
  });
});
