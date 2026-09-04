import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { inspectDependencies } from '../src/inspector/dependencies.js';

describe('ProjectInspector - inspectDependencies', () => {
  let tmpDir: string;

  beforeEach(() => {
    tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'rtb-deps-test-'));
  });

  afterEach(() => {
    fs.rmSync(tmpDir, { recursive: true, force: true });
  });

  it('parses package.json dependencies and devDependencies', () => {
    fs.writeFileSync(
      path.join(tmpDir, 'package.json'),
      JSON.stringify({
        dependencies: {
          chalk: '^5.0.0',
          commander: '^12.0.0',
        },
        devDependencies: {
          vitest: '^3.0.0',
        },
      })
    );

    const deps = inspectDependencies(tmpDir);
    expect(deps).toHaveLength(3);
    expect(deps).toContainEqual({
      package: 'chalk',
      spec: '^5.0.0',
      type: 'npm/pnpm/yarn',
      status: 'Declared',
    });
    expect(deps).toContainEqual({
      package: 'vitest',
      spec: '^3.0.0',
      type: 'npm/pnpm (dev)',
      status: 'Declared',
    });
  });

  it('parses Cargo.toml dependencies', () => {
    const cargoContent = `
[package]
name = "my-rust-tool"
version = "0.1.0"

[dependencies]
serde = "1.0"
tokio = { version = "1.28", features = ["full"] }
`;
    fs.writeFileSync(path.join(tmpDir, 'Cargo.toml'), cargoContent);

    const deps = inspectDependencies(tmpDir);
    expect(deps).toContainEqual({
      package: 'serde',
      spec: '1.0',
      type: 'Cargo (Rust)',
      status: 'Declared',
    });
    expect(deps).toContainEqual({
      package: 'tokio',
      spec: '1.28',
      type: 'Cargo (Rust)',
      status: 'Declared',
    });
  });

  it('parses pyproject.toml dependencies', () => {
    const pyproject = `
[project]
name = "my-py-app"
dependencies = [
    "requests>=2.28.0",
    "pydantic==2.0"
]
`;
    fs.writeFileSync(path.join(tmpDir, 'pyproject.toml'), pyproject);

    const deps = inspectDependencies(tmpDir);
    expect(deps).toContainEqual({
      package: 'requests',
      spec: '>=2.28.0',
      type: 'Python (pyproject)',
      status: 'Declared',
    });
    expect(deps).toContainEqual({
      package: 'pydantic',
      spec: '==2.0',
      type: 'Python (pyproject)',
      status: 'Declared',
    });
  });

  it('parses requirements.txt when pyproject is absent', () => {
    const reqs = "flask>=2.0.0\npytest==7.0.1\n# comment line\n";
    fs.writeFileSync(path.join(tmpDir, 'requirements.txt'), reqs);

    const deps = inspectDependencies(tmpDir);
    expect(deps).toContainEqual({
      package: 'flask',
      spec: '>=2.0.0',
      type: 'Python (requirements)',
      status: 'Declared',
    });
  });

  it('parses go.mod dependencies', () => {
    const goMod = `
module example.com/myapp

go 1.22

require (
    github.com/gin-gonic/gin v1.9.1
    golang.org/x/crypto v0.21.0
)
`;
    fs.writeFileSync(path.join(tmpDir, 'go.mod'), goMod);

    const deps = inspectDependencies(tmpDir);
    expect(deps).toContainEqual({
      package: 'github.com/gin-gonic/gin',
      spec: 'v1.9.1',
      type: 'Go (go.mod)',
      status: 'Declared',
    });
  });

  it('returns empty array when no manifests exist', () => {
    const deps = inspectDependencies(tmpDir);
    expect(deps).toEqual([]);
  });
});
