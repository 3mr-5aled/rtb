import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { createCli } from '../src/cli.js';

describe('rtb deploy command', () => {
  let tmpDir: string;
  let activeDir: string;
  let prodDir: string;
  let stagingDir: string;
  let configFile: string;

  beforeEach(() => {
    tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'rtb-cli-deploy-test-'));
    activeDir = path.join(tmpDir, '01-Active');
    prodDir = path.join(tmpDir, '02-Production');
    stagingDir = path.join(tmpDir, '03-Staging');

    fs.mkdirSync(activeDir, { recursive: true });
    fs.mkdirSync(prodDir, { recursive: true });
    fs.mkdirSync(stagingDir, { recursive: true });

    configFile = path.join(tmpDir, 'rtb.config.json');
    fs.writeFileSync(
      configFile,
      JSON.stringify(
        {
          version: '0.5.3',
          projectRoots: {
            active: { path: activeDir, label: 'Active', emoji: '🚀' },
            production: { path: prodDir, label: 'Production', emoji: '🌟' },
            staging: { path: stagingDir, label: 'Staging', emoji: '🧪' },
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

  it('deploys project from Active to Production by default', async () => {
    const projDir = path.join(activeDir, 'my-app');
    fs.mkdirSync(projDir, { recursive: true });
    fs.writeFileSync(path.join(projDir, 'package.json'), '{}');

    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});

    const cli = createCli();
    await cli.parseAsync(['node', 'rtb', 'deploy', 'my-app', '--config', configFile, '--json']);

    expect(fs.existsSync(projDir)).toBe(false);
    const deployedDir = path.join(prodDir, 'my-app');
    expect(fs.existsSync(deployedDir)).toBe(true);

    const rawOutput = logSpy.mock.calls.map((c) => c.join(' ')).join('\n');
    const parsed = JSON.parse(rawOutput);
    expect(parsed.deployed).toBe(true);
    expect(parsed.target).toBe('production');
    expect(parsed.name).toBe('my-app');
    expect(parsed.to).toBe(deployedDir);
  });

  it('deploys project to Staging when --staging flag is provided', async () => {
    const projDir = path.join(activeDir, 'api-service');
    fs.mkdirSync(projDir, { recursive: true });
    fs.writeFileSync(path.join(projDir, 'Cargo.toml'), '');

    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});

    const cli = createCli();
    await cli.parseAsync(['node', 'rtb', 'deploy', 'api-service', '--staging', '--config', configFile, '--json']);

    expect(fs.existsSync(projDir)).toBe(false);
    const deployedDir = path.join(stagingDir, 'api-service');
    expect(fs.existsSync(deployedDir)).toBe(true);

    const rawOutput = logSpy.mock.calls.map((c) => c.join(' ')).join('\n');
    const parsed = JSON.parse(rawOutput);
    expect(parsed.deployed).toBe(true);
    expect(parsed.target).toBe('staging');
    expect(parsed.name).toBe('api-service');
  });

  it('promotes project from Staging to Production when not found in Active', async () => {
    const projDir = path.join(stagingDir, 'staged-app');
    fs.mkdirSync(projDir, { recursive: true });
    fs.writeFileSync(path.join(projDir, 'package.json'), '{}');

    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});

    const cli = createCli();
    await cli.parseAsync(['node', 'rtb', 'deploy', 'staged-app', '--prod', '--config', configFile, '--json']);

    expect(fs.existsSync(projDir)).toBe(false);
    const deployedDir = path.join(prodDir, 'staged-app');
    expect(fs.existsSync(deployedDir)).toBe(true);

    const rawOutput = logSpy.mock.calls.map((c) => c.join(' ')).join('\n');
    const parsed = JSON.parse(rawOutput);
    expect(parsed.deployed).toBe(true);
    expect(parsed.target).toBe('production');
    expect(parsed.from).toBe(projDir);
    expect(parsed.to).toBe(deployedDir);
  });

  it('supports explicit --from flag', async () => {
    const projDir = path.join(stagingDir, 'explicit-app');
    fs.mkdirSync(projDir, { recursive: true });
    fs.writeFileSync(path.join(projDir, 'package.json'), '{}');

    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});

    const cli = createCli();
    await cli.parseAsync(['node', 'rtb', 'deploy', 'explicit-app', '--from', 'staging', '--prod', '--config', configFile, '--json']);

    expect(fs.existsSync(projDir)).toBe(false);
    const deployedDir = path.join(prodDir, 'explicit-app');
    expect(fs.existsSync(deployedDir)).toBe(true);
  });

  it('errors gracefully when project is not found in Active or Staging', async () => {
    const cli = createCli();
    await cli.parseAsync(['node', 'rtb', 'deploy', 'non-existent', '--config', configFile, '--json']);

    expect(process.exitCode).toBe(1);
    process.exitCode = 0;
  });
});
