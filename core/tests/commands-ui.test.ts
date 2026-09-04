import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { getPlatformBinaryAsset, getDefaultUserBinDir } from '../src/commands/doctor.js';
import { provisionRtbtuiBinary } from '../src/commands/ui.js';

describe('UI binary resolution and self-provisioning', () => {
  let tmpDir: string;

  beforeEach(() => {
    tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'rtb-ui-test-'));
  });

  afterEach(() => {
    if (fs.existsSync(tmpDir)) {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  it('maps platform and architecture to canonical release binary assets', () => {
    expect(getPlatformBinaryAsset('win32', 'x64')).toBe('rtbtui-windows-amd64.exe');
    expect(getPlatformBinaryAsset('linux', 'x64')).toBe('rtbtui-linux-amd64');
    expect(getPlatformBinaryAsset('linux', 'arm64')).toBe('rtbtui-linux-arm64');
    expect(getPlatformBinaryAsset('darwin', 'x64')).toBe('rtbtui-macos-amd64');
    expect(getPlatformBinaryAsset('darwin', 'arm64')).toBe('rtbtui-macos-arm64');
    expect(getPlatformBinaryAsset('sunos', 'x64')).toBe(null);
  });

  it('resolves default user binary directory based on platform or override', () => {
    const origEnv = process.env.RTB_BIN_DIR;
    try {
      process.env.RTB_BIN_DIR = tmpDir;
      expect(getDefaultUserBinDir()).toBe(tmpDir);
    } finally {
      process.env.RTB_BIN_DIR = origEnv;
    }
  });

  it('provisions prebuilt binary from mocked release stream into target directory', async () => {
    const fakeContent = Buffer.from('FAKE_BINARY_CONTENT');
    const mockFetch = vi.fn().mockResolvedValue({
      ok: true,
      status: 200,
      statusText: 'OK',
      arrayBuffer: async () => Uint8Array.from(fakeContent).buffer,
    });

    const result = await provisionRtbtuiBinary({
      destDir: tmpDir,
      fetchFn: mockFetch as any,
      platform: 'linux',
      arch: 'x64',
    });

    expect(result).not.toBeNull();
    expect(fs.existsSync(result!)).toBe(true);

    const saved = fs.readFileSync(result!);
    expect(saved.toString()).toBe('FAKE_BINARY_CONTENT');
  });

  it('gracefully returns null if download fails', async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      ok: false,
      status: 404,
      statusText: 'Not Found',
    });

    const result = await provisionRtbtuiBinary({
      destDir: tmpDir,
      fetchFn: mockFetch as any,
      platform: 'linux',
      arch: 'x64',
    });

    expect(result).toBeNull();
  });
});
