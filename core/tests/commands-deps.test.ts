import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { Command } from 'commander';
import { registerDepsCommand } from '../src/commands/deps.js';
import type { CliContext } from '../src/types/context.js';

describe('Dependency Inspection Command (rtb deps)', () => {
  let tmpRoot: string;
  let mockCtx: CliContext;

  beforeEach(() => {
    tmpRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'rtb-deps-cmd-'));
    const activeDir = path.join(tmpRoot, 'Active');
    fs.mkdirSync(activeDir, { recursive: true });

    mockCtx = {
      config: {
        version: '1.0.0',
        projectRoots: {
          active: { path: activeDir, label: 'Active', emoji: '📁' },
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

  it('outputs tabular summary of dependencies in project', async () => {
    const projDir = path.join(mockCtx.config!.projectRoots.active!.path, 'my-proj');
    fs.mkdirSync(projDir, { recursive: true });
    fs.writeFileSync(
      path.join(projDir, 'package.json'),
      JSON.stringify({ dependencies: { express: '^4.18.2' } })
    );

    const program = new Command();
    registerDepsCommand(program, () => mockCtx);

    const consoleSpy = vi.spyOn(console, 'log').mockImplementation(() => {});
    await program.parseAsync(['node', 'rtb', 'deps', 'my-proj']);

    expect(consoleSpy).toHaveBeenCalled();
    const calls = consoleSpy.mock.calls.map((c) => c.join(' ')).join('\n');
    expect(calls).toContain('express');
    expect(calls).toContain('^4.18.2');

    consoleSpy.mockRestore();
  });

  it('supports --json flag returning structured dependencies', async () => {
    const projDir = path.join(mockCtx.config!.projectRoots.active!.path, 'my-rust');
    fs.mkdirSync(projDir, { recursive: true });
    fs.writeFileSync(path.join(projDir, 'Cargo.toml'), '[package]\nname="foo"\n[dependencies]\nserde="1.0"');

    const jsonCtx = { ...mockCtx, isJson: true };
    const program = new Command();
    registerDepsCommand(program, () => jsonCtx);

    const consoleSpy = vi.spyOn(console, 'log').mockImplementation(() => {});
    await program.parseAsync(['node', 'rtb', 'deps', 'my-rust']);

    expect(consoleSpy).toHaveBeenCalled();
    const output = consoleSpy.mock.calls[0][0];
    const parsed = JSON.parse(output);
    expect(parsed).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          package: 'serde',
          spec: '1.0',
          type: 'Cargo (Rust)',
        }),
      ])
    );

    consoleSpy.mockRestore();
  });
});
