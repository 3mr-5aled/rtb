import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { resolveProjectAction, executeProjectAction } from '../src/services/runner.js';

describe('ProjectRunner - resolveProjectAction', () => {
  let tmpDir: string;

  beforeEach(() => {
    tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'rtb-runner-test-'));
  });

  afterEach(() => {
    fs.rmSync(tmpDir, { recursive: true, force: true });
  });

  describe('action: run', () => {
    it('detects package.json with dev script and npm by default', () => {
      fs.writeFileSync(
        path.join(tmpDir, 'package.json'),
        JSON.stringify({ scripts: { dev: 'vite', start: 'node index.js' } })
      );

      const res = resolveProjectAction('run', tmpDir, ['--port', '3000']);
      expect(res).toEqual({
        executable: 'npm',
        args: ['run', 'dev', '--', '--port', '3000'],
      });
    });

    it('detects pnpm when pnpm-lock.yaml is present', () => {
      fs.writeFileSync(
        path.join(tmpDir, 'package.json'),
        JSON.stringify({ scripts: { dev: 'vite' } })
      );
      fs.writeFileSync(path.join(tmpDir, 'pnpm-lock.yaml'), '');

      const res = resolveProjectAction('run', tmpDir, ['--port', '3000']);
      expect(res).toEqual({
        executable: 'pnpm',
        args: ['dev', '--', '--port', '3000'],
      });
    });

    it('detects yarn when yarn.lock is present', () => {
      fs.writeFileSync(
        path.join(tmpDir, 'package.json'),
        JSON.stringify({ scripts: { start: 'node index.js' } })
      );
      fs.writeFileSync(path.join(tmpDir, 'yarn.lock'), '');

      const res = resolveProjectAction('run', tmpDir, []);
      expect(res).toEqual({
        executable: 'yarn',
        args: ['start'],
      });
    });

    it('detects Cargo.toml for rust projects with double dash for args', () => {
      fs.writeFileSync(path.join(tmpDir, 'Cargo.toml'), '[package]\nname = "foo"');

      const res = resolveProjectAction('run', tmpDir, ['hello']);
      expect(res).toEqual({
        executable: 'cargo',
        args: ['run', '--', 'hello'],
      });
    });

    it('detects go.mod for Go projects', () => {
      fs.writeFileSync(path.join(tmpDir, 'go.mod'), 'module foo');

      const res = resolveProjectAction('run', tmpDir, []);
      expect(res).toEqual({
        executable: 'go',
        args: ['run', '.'],
      });
    });

    it('detects .NET csproj/sln projects', () => {
      fs.writeFileSync(path.join(tmpDir, 'App.csproj'), '<Project></Project>');

      const res = resolveProjectAction('run', tmpDir, ['arg1']);
      expect(res).toEqual({
        executable: 'dotnet',
        args: ['run', '--', 'arg1'],
      });
    });

    it('detects Makefile for run target', () => {
      fs.writeFileSync(path.join(tmpDir, 'Makefile'), 'run:\n\techo hi');

      const res = resolveProjectAction('run', tmpDir, []);
      expect(res).toEqual({
        executable: 'make',
        args: ['run'],
      });
    });

    it('detects main.py for Python projects', () => {
      fs.writeFileSync(path.join(tmpDir, 'main.py'), 'print("hi")');

      const res = resolveProjectAction('run', tmpDir, ['arg1']);
      expect(res).toEqual({
        executable: 'python',
        args: ['main.py', 'arg1'],
      });
    });

    it('returns null if no entrypoint is found', () => {
      const res = resolveProjectAction('run', tmpDir, []);
      expect(res).toBeNull();
    });
  });

  describe('action: build', () => {
    it('detects package.json with build script', () => {
      fs.writeFileSync(
        path.join(tmpDir, 'package.json'),
        JSON.stringify({ scripts: { build: 'tsc' } })
      );

      const res = resolveProjectAction('build', tmpDir, []);
      expect(res).toEqual({
        executable: 'npm',
        args: ['run', 'build'],
      });
    });

    it('detects Cargo.toml for cargo build --release', () => {
      fs.writeFileSync(path.join(tmpDir, 'Cargo.toml'), '[package]\nname = "foo"');

      const res = resolveProjectAction('build', tmpDir, []);
      expect(res).toEqual({
        executable: 'cargo',
        args: ['build', '--release'],
      });
    });

    it('detects .NET project for dotnet build', () => {
      fs.writeFileSync(path.join(tmpDir, 'App.sln'), '');

      const res = resolveProjectAction('build', tmpDir, []);
      expect(res).toEqual({
        executable: 'dotnet',
        args: ['build'],
      });
    });

    it('detects go.mod for go build', () => {
      fs.writeFileSync(path.join(tmpDir, 'go.mod'), 'module foo');

      const res = resolveProjectAction('build', tmpDir, []);
      expect(res).toEqual({
        executable: 'go',
        args: ['build'],
      });
    });

    it('returns null if no build target is found', () => {
      const res = resolveProjectAction('build', tmpDir, []);
      expect(res).toBeNull();
    });
  });

  describe('action: test', () => {
    it('detects package.json with test script', () => {
      fs.writeFileSync(
        path.join(tmpDir, 'package.json'),
        JSON.stringify({ scripts: { test: 'vitest run' } })
      );

      const res = resolveProjectAction('test', tmpDir, ['--watch']);
      expect(res).toEqual({
        executable: 'npm',
        args: ['test', '--', '--watch'],
      });
    });

    it('detects Cargo.toml for cargo test', () => {
      fs.writeFileSync(path.join(tmpDir, 'Cargo.toml'), '[package]\nname = "foo"');

      const res = resolveProjectAction('test', tmpDir, []);
      expect(res).toEqual({
        executable: 'cargo',
        args: ['test'],
      });
    });

    it('detects .NET project for dotnet test', () => {
      fs.writeFileSync(path.join(tmpDir, 'App.csproj'), '');

      const res = resolveProjectAction('test', tmpDir, []);
      expect(res).toEqual({
        executable: 'dotnet',
        args: ['test'],
      });
    });

    it('detects pytest.ini or pyproject.toml with pytest', () => {
      fs.writeFileSync(path.join(tmpDir, 'pytest.ini'), '[pytest]');

      const res = resolveProjectAction('test', tmpDir, ['-v']);
      expect(res).toEqual({
        executable: 'pytest',
        args: ['-v'],
      });
    });

    it('returns null if no test suite is found', () => {
      const res = resolveProjectAction('test', tmpDir, []);
      expect(res).toBeNull();
    });
  });

  describe('action: executeProjectAction', () => {
    it('respects dryRun option and skips process spawning', async () => {
      const code = await executeProjectAction('/test/path', { executable: 'npm', args: ['run', 'dev'] }, { dryRun: true });
      expect(code).toBe(0);
    });

    it('executes a command that exits with 0', async () => {
      const isWindows = process.platform === 'win32';
      const executable = isWindows ? 'cmd.exe' : 'true';
      const args = isWindows ? ['/c', 'exit', '0'] : [];
      const code = await executeProjectAction(process.cwd(), { executable, args });
      expect(code).toBe(0);
    });

    it('executes a command that exits with non-zero code', async () => {
      const isWindows = process.platform === 'win32';
      const executable = isWindows ? 'cmd.exe' : 'false';
      const args = isWindows ? ['/c', 'exit', '7'] : [];
      const code = await executeProjectAction(process.cwd(), { executable, args });
      expect(code).toBe(isWindows ? 7 : 1);
    });

    it('executes without triggering DEP0190 warning when needsShell is true', async () => {
      const isWindows = process.platform === 'win32';
      if (!isWindows) return;

      const warnings: any[] = [];
      const onWarning = (w: any) => warnings.push(w);
      process.on('warning', onWarning);

      try {
        const code = await executeProjectAction(process.cwd(), {
          executable: 'cmd',
          args: ['/c', 'exit', '0'],
        });
        expect(code).toBe(0);
        const depWarnings = warnings.filter((w) => w.name === 'DeprecationWarning' && w.code === 'DEP0190');
        expect(depWarnings).toHaveLength(0);
      } finally {
        process.off('warning', onWarning);
      }
    });
  });
});
