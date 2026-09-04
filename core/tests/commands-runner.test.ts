import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { Command } from 'commander';
import { registerRunCommand } from '../src/commands/run.js';
import { registerBuildCommand } from '../src/commands/build.js';
import { registerTestCommand } from '../src/commands/test.js';
import type { CliContext } from '../src/types/context.js';

describe('Project Action Commands (rtb run, rtb build, rtb test)', () => {
  let tmpRoot: string;
  let mockCtx: CliContext;

  beforeEach(() => {
    tmpRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'rtb-cmd-test-'));
    const activeDir = path.join(tmpRoot, 'Active');
    fs.mkdirSync(activeDir, { recursive: true });

    mockCtx = {
      config: {
        version: '1.0.0',
        projectRoots: {
          active: { path: activeDir, label: 'Active', emoji: '📁' },
          paused: { path: path.join(tmpRoot, 'Paused'), label: 'Paused', emoji: '⏸️' },
        },
        cleanDeps: { targets: ['node_modules'] },
        backupRoot: tmpRoot,
      },
      configPath: path.join(tmpRoot, 'rtb.config.json'),
      isConfigured: true,
      isJson: false,
      isQuiet: false,
      isInteractive: false,
    };
  });

  afterEach(() => {
    fs.rmSync(tmpRoot, { recursive: true, force: true });
  });

  it('rtb run executes resolved project command in target project', async () => {
    const projDir = path.join(mockCtx.config!.projectRoots.active!.path, 'my-app');
    fs.mkdirSync(projDir, { recursive: true });
    fs.writeFileSync(
      path.join(projDir, 'package.json'),
      JSON.stringify({ scripts: { dev: 'echo running dev' } })
    );

    const program = new Command();
    registerRunCommand(program, () => mockCtx);

    process.exitCode = 0;
    await program.parseAsync(['node', 'rtb', 'run', 'my-app', '--dry-run']);
    expect(process.exitCode).toBe(0);
  });

  it('rtb build executes resolved build command in target project', async () => {
    const projDir = path.join(mockCtx.config!.projectRoots.active!.path, 'my-rust-app');
    fs.mkdirSync(projDir, { recursive: true });
    fs.writeFileSync(path.join(projDir, 'Cargo.toml'), '[package]\nname = "my-rust-app"');

    const program = new Command();
    registerBuildCommand(program, () => mockCtx);

    process.exitCode = 0;
    await program.parseAsync(['node', 'rtb', 'build', 'my-rust-app', '--dry-run']);
    expect(process.exitCode).toBe(0);
  });

  it('rtb test executes resolved test command in target project', async () => {
    const projDir = path.join(mockCtx.config!.projectRoots.active!.path, 'my-py-app');
    fs.mkdirSync(projDir, { recursive: true });
    fs.writeFileSync(path.join(projDir, 'pytest.ini'), '[pytest]');

    const program = new Command();
    registerTestCommand(program, () => mockCtx);

    process.exitCode = 0;
    await program.parseAsync(['node', 'rtb', 'test', 'my-py-app', '--dry-run']);
    expect(process.exitCode).toBe(0);
  });
});
