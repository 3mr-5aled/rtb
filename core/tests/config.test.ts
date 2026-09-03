import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import path from 'node:path';
import fs from 'node:fs';
import os from 'node:os';
import { loadConfig, resolveConfigPath, getStandardConfigPath } from '../src/config/loader.js';

describe('Configuration Loader', () => {
  const tmpDir = path.join(os.tmpdir(), `rtb-test-${Date.now()}`);

  beforeEach(() => {
    fs.mkdirSync(tmpDir, { recursive: true });
  });

  afterEach(() => {
    try {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    } catch {}
  });

  it('should resolve standard path under ~/.config/rtb/rtb.config.json', () => {
    const stdPath = getStandardConfigPath();
    expect(stdPath).toContain('.config');
    expect(stdPath).toContain('rtb');
    expect(stdPath.endsWith('rtb.config.json')).toBe(true);
  });

  it('should load custom config path and report configured when active path is present', () => {
    const customConfig = path.join(tmpDir, 'custom.config.json');
    fs.writeFileSync(
      customConfig,
      JSON.stringify({
        version: '1.0.0',
        projectRoots: {
          active: {
            path: 'D:/Projects/Active',
            label: 'Active',
            emoji: '📁',
          },
        },
      })
    );

    const resolution = loadConfig(customConfig);
    expect(resolution.isConfigured).toBe(true);
    expect(resolution.source).toBe('custom');
    expect(resolution.config?.version).toBe('1.0.0');
    expect(resolution.config?.projectRoots.active.path).toBe('D:/Projects/Active');
  });

  it('should report unconfigured when active path is missing or empty', () => {
    const customConfig = path.join(tmpDir, 'empty.config.json');
    fs.writeFileSync(
      customConfig,
      JSON.stringify({
        version: '1.0.0',
        projectRoots: {
          active: {
            path: '',
            label: 'Active',
            emoji: '📁',
          },
        },
      })
    );

    const resolution = loadConfig(customConfig);
    expect(resolution.isConfigured).toBe(false);
  });

  it('should gracefully handle non-existent file path', () => {
    const nonExistent = path.join(tmpDir, 'missing.json');
    const resolution = loadConfig(nonExistent);
    expect(resolution.config).toBeNull();
    expect(resolution.isConfigured).toBe(false);
  });
});
