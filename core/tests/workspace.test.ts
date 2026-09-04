import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { inspectWorkspace } from '../src/inspector/workspace.js';

describe('inspectWorkspace', () => {
  let tmpDir: string;

  beforeEach(() => {
    tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'rtb-workspace-test-'));
  });

  afterEach(() => {
    fs.rmSync(tmpDir, { recursive: true, force: true });
  });

  it('detects standard single-package repository when no workspace files exist', () => {
    const res = inspectWorkspace(tmpDir);
    expect(res.isMonorepo).toBe(false);
    expect(res.workspaceType).toBe('Single Package / Standard Repository');
    expect(res.packages).toHaveLength(0);
    expect(res.projectPath).toBe(tmpDir);
  });

  it('detects pnpm workspace packages from pnpm-workspace.yaml', () => {
    const content = `packages:\n  - 'packages/*'\n  - "apps/*"\n  - shared\n`;
    fs.writeFileSync(path.join(tmpDir, 'pnpm-workspace.yaml'), content);

    const res = inspectWorkspace(tmpDir);
    expect(res.isMonorepo).toBe(true);
    expect(res.workspaceType).toBe('pnpm Workspaces');
    expect(res.packages).toEqual([
      { packagePattern: 'packages/*', type: 'pnpm' },
      { packagePattern: 'apps/*', type: 'pnpm' },
      { packagePattern: 'shared', type: 'pnpm' },
    ]);
  });

  it('detects npm/yarn workspaces from package.json array and object format', () => {
    const pkgJson = {
      name: 'root-monorepo',
      workspaces: ['packages/*', 'core'],
    };
    fs.writeFileSync(path.join(tmpDir, 'package.json'), JSON.stringify(pkgJson, null, 2));

    const res = inspectWorkspace(tmpDir);
    expect(res.isMonorepo).toBe(true);
    expect(res.workspaceType).toBe('npm/yarn Workspaces');
    expect(res.packages).toEqual([
      { packagePattern: 'packages/*', type: 'npm/yarn' },
      { packagePattern: 'core', type: 'npm/yarn' },
    ]);
  });

  it('detects Cargo workspace members from Cargo.toml', () => {
    const cargoToml = `
[workspace]
members = [
    "crates/*",
    "cli"
]
`;
    fs.writeFileSync(path.join(tmpDir, 'Cargo.toml'), cargoToml);

    const res = inspectWorkspace(tmpDir);
    expect(res.isMonorepo).toBe(true);
    expect(res.workspaceType).toBe('Cargo Workspace (Rust)');
    expect(res.packages).toEqual([
      { packagePattern: 'crates/*', type: 'Cargo' },
      { packagePattern: 'cli', type: 'Cargo' },
    ]);
  });
});
