import { describe, it, expect, vi } from 'vitest';
import path from 'node:path';
import { createCli } from '../src/cli.js';

describe('CLI Framework & Registry', () => {
  it('should support version command in human mode', async () => {
    const cli = createCli();
    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});

    await cli.parseAsync(['node', 'rtb', 'version']);

    expect(logSpy).toHaveBeenCalled();
    const output = logSpy.mock.calls[0][0];
    expect(output).toContain('RTB');
    expect(output).toContain('v0.5.0');
    logSpy.mockRestore();
  });

  it('should support version command in JSON mode', async () => {
    const cli = createCli();
    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});

    await cli.parseAsync(['node', 'rtb', '--json', 'version']);

    expect(logSpy).toHaveBeenCalled();
    const parsed = JSON.parse(logSpy.mock.calls[0][0]);
    expect(parsed.name).toBe('rtb');
    expect(parsed.version).toBe('0.5.0');
    expect(parsed.engine).toBe('node');
    logSpy.mockRestore();
  });

  it('should support config --path option', async () => {
    const cli = createCli();
    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});

    await cli.parseAsync(['node', 'rtb', 'config', '--path']);

    expect(logSpy).toHaveBeenCalled();
    const output = logSpy.mock.calls[0][0];
    expect(output).toContain('rtb.config.json');
    logSpy.mockRestore();
  });

  it('should support config --json output when valid config exists', async () => {
    const sampleConfig = path.resolve(__dirname, '../../config/rtb.config.json');
    const cli = createCli();
    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});

    await cli.parseAsync(['node', 'rtb', '--config', sampleConfig, '--json', 'config']);

    expect(logSpy).toHaveBeenCalled();
    const parsed = JSON.parse(logSpy.mock.calls[0][0]);
    expect(parsed.version).toBeDefined();
    expect(parsed.projectRoots).toBeDefined();
    logSpy.mockRestore();
  });
});
