import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { createCli } from '../src/cli.js';
import { parseSemver, compareSemver, upgradeService } from '../src/commands/upgrade.js';
import { RTB_VERSION } from '../src/commands/version.js';

describe('rtb upgrade command', () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('parseSemver parses semantic version numbers correctly', () => {
    expect(parseSemver('0.6.3')).toEqual([0, 6, 3]);
    expect(parseSemver('v1.2.3-alpha.1')).toEqual([1, 2, 3]);
    expect(parseSemver('2.0')).toEqual([2, 0, 0]);
  });

  it('compareSemver compares version numbers accurately', () => {
    expect(compareSemver('0.6.4', '0.6.3')).toBe(1);
    expect(compareSemver('0.6.3', '0.6.4')).toBe(-1);
    expect(compareSemver('0.6.3', '0.6.3')).toBe(0);
    expect(compareSemver('1.0.0', '0.9.9')).toBe(1);
    expect(compareSemver('0.7.0', '0.6.9')).toBe(1);
  });

  it('rtb upgrade --check --json reports update available when newer version exists', async () => {
    vi.spyOn(upgradeService, 'fetchLatestVersion').mockResolvedValue('99.0.0');
    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});

    const cli = createCli();
    await cli.parseAsync(['node', 'rtb', 'upgrade', '--check', '--json']);

    expect(logSpy).toHaveBeenCalled();
    const parsed = JSON.parse(logSpy.mock.calls[0][0]);
    expect(parsed.currentVersion).toBe(RTB_VERSION);
    expect(parsed.latestVersion).toBe('99.0.0');
    expect(parsed.updateAvailable).toBe(true);
    expect(parsed.checkOnly).toBe(true);
  });

  it('rtb upgrade --check --json reports up to date when on latest version', async () => {
    vi.spyOn(upgradeService, 'fetchLatestVersion').mockResolvedValue(RTB_VERSION);
    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});

    const cli = createCli();
    await cli.parseAsync(['node', 'rtb', 'upgrade', '--check', '--json']);

    expect(logSpy).toHaveBeenCalled();
    const parsed = JSON.parse(logSpy.mock.calls[0][0]);
    expect(parsed.currentVersion).toBe(RTB_VERSION);
    expect(parsed.latestVersion).toBe(RTB_VERSION);
    expect(parsed.updateAvailable).toBe(false);
  });

  it('rtb upgrade --json does not upgrade when already up to date unless forced', async () => {
    vi.spyOn(upgradeService, 'fetchLatestVersion').mockResolvedValue(RTB_VERSION);
    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});

    const cli = createCli();
    await cli.parseAsync(['node', 'rtb', 'upgrade', '--json']);

    expect(logSpy).toHaveBeenCalled();
    const parsed = JSON.parse(logSpy.mock.calls[0][0]);
    expect(parsed.upgraded).toBe(false);
    expect(parsed.updateAvailable).toBe(false);
  });

  it('rtb upgrade --force --json triggers upgrade execution', async () => {
    vi.spyOn(upgradeService, 'fetchLatestVersion').mockResolvedValue(RTB_VERSION);
    vi.spyOn(upgradeService, 'executeUpgrade').mockReturnValue({
      success: true,
      method: 'mock',
      message: 'Mock upgrade succeeded',
    });
    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});

    const cli = createCli();
    await cli.parseAsync(['node', 'rtb', 'upgrade', '--force', '--json']);

    expect(logSpy).toHaveBeenCalled();
    const parsed = JSON.parse(logSpy.mock.calls[0][0]);
    expect(parsed.upgraded).toBe(true);
    expect(parsed.method).toBe('mock');
  });

  it('end-to-end upgrade cycle reports success when newer version is detected', async () => {
    vi.spyOn(upgradeService, 'fetchLatestVersion').mockResolvedValue('1.0.0');
    vi.spyOn(upgradeService, 'executeUpgrade').mockReturnValue({
      success: true,
      method: 'standalone',
      message: 'Successfully updated bundle',
    });
    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});

    const cli = createCli();
    await cli.parseAsync(['node', 'rtb', 'upgrade', '--json']);

    expect(logSpy).toHaveBeenCalled();
    const parsed = JSON.parse(logSpy.mock.calls[0][0]);
    expect(parsed.upgraded).toBe(true);
    expect(parsed.targetVersion).toBe('1.0.0');
    expect(parsed.method).toBe('standalone');
  });
});
