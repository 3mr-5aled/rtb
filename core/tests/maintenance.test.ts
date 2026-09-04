import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { MaintenanceTaskRegistry } from '../src/services/maintenance.js';
import type { RtbConfig } from '../src/types/config.js';

describe('MaintenanceTaskRegistry', () => {
  let tmpDir: string;
  let activeDir: string;
  let backupDir: string;
  let config: RtbConfig;

  beforeEach(() => {
    tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'rtb-maint-test-'));
    activeDir = path.join(tmpDir, '01-Active');
    backupDir = path.join(tmpDir, 'backups');
    fs.mkdirSync(activeDir, { recursive: true });
    fs.mkdirSync(backupDir, { recursive: true });

    config = {
      version: '0.5.3',
      projectRoots: {
        active: { path: activeDir, label: 'Active', emoji: '🚀' },
      },
      backupRoot: backupDir,
    };
  });

  afterEach(() => {
    fs.rmSync(tmpDir, { recursive: true, force: true });
  });

  it('runs custom registered tasks', async () => {
    const registry = new MaintenanceTaskRegistry();
    registry.registerTask('custom', async () => {
      return { task: 'custom', success: true, message: 'Custom done' };
    });

    const res = await registry.runTask('custom', { config });
    expect(res.success).toBe(true);
    expect(res.message).toBe('Custom done');
  });

  it('backup task backs up config to backupRoot/configs', async () => {
    const cfgPath = path.join(tmpDir, 'rtb.config.json');
    fs.writeFileSync(cfgPath, JSON.stringify(config, null, 2));

    const registry = new MaintenanceTaskRegistry();
    const res = await registry.runTask('backup', { config, configPath: cfgPath });

    expect(res.success).toBe(true);
    const cfgBackupDir = path.join(backupDir, 'configs');
    expect(fs.existsSync(cfgBackupDir)).toBe(true);
    const files = fs.readdirSync(cfgBackupDir);
    expect(files.some((f) => f.startsWith('rtb.config-') && f.endsWith('.json'))).toBe(true);
  });

  it('env task backs up .env files from active project roots', async () => {
    const projDir = path.join(activeDir, 'my-app');
    fs.mkdirSync(projDir, { recursive: true });
    fs.writeFileSync(path.join(projDir, '.env'), 'PORT=3000\nSECRET=xyz');
    fs.writeFileSync(path.join(projDir, '.env.local'), 'DEBUG=true');

    const registry = new MaintenanceTaskRegistry();
    const res = await registry.runTask('env', { config });

    expect(res.success).toBe(true);
    const envBackupDir = path.join(backupDir, 'env-backups');
    expect(fs.existsSync(envBackupDir)).toBe(true);
    const files = fs.readdirSync(envBackupDir);
    expect(files.length).toBeGreaterThan(0);
  });

  it('guard task checks root directory for unorganized files', async () => {
    const registry = new MaintenanceTaskRegistry();
    const res = await registry.runTask('guard', { config, rootDir: tmpDir });

    expect(res.success).toBe(true);
    expect(res.task).toBe('guard');
  });
});
