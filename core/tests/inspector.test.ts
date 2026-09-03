import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import path from 'node:path';
import fs from 'node:fs';
import os from 'node:os';
import { inspectProject, scanAllProjects } from '../src/inspector/inspector.js';
import type { RtbConfig } from '../types/config.js';

describe('Project Inspector', () => {
  const tmpDir = path.join(os.tmpdir(), `rtb-inspector-test-${Date.now()}`);

  beforeEach(() => {
    fs.mkdirSync(tmpDir, { recursive: true });
  });

  afterEach(() => {
    try {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    } catch {}
  });

  it('should detect Node/Next.js/Tailwind/TypeScript stack from package.json', () => {
    const projDir = path.join(tmpDir, 'web-app');
    fs.mkdirSync(projDir, { recursive: true });

    fs.writeFileSync(
      path.join(projDir, 'package.json'),
      JSON.stringify({
        name: 'web-app',
        dependencies: {
          next: '^14.0.0',
          react: '^18.0.0',
        },
        devDependencies: {
          typescript: '^5.0.0',
          tailwindcss: '^3.0.0',
        },
        engines: {
          node: '>=18.0.0',
        },
      })
    );

    const details = inspectProject(projDir, 'Active');
    expect(details).not.toBeNull();
    expect(details?.name).toBe('web-app');
    expect(details?.stack).toContain('Next.js');
    expect(details?.stack).toContain('Tailwind');
    expect(details?.stack).toContain('TypeScript');
    expect(details?.runtime_version).toBe('>=18.0.0');
  });

  it('should detect Python and Rust toolchains', () => {
    const projDir = path.join(tmpDir, 'py-rust-tool');
    fs.mkdirSync(projDir, { recursive: true });

    fs.writeFileSync(path.join(projDir, 'Cargo.toml'), '[package]\nname = "my-tool"');
    fs.writeFileSync(path.join(projDir, 'pyproject.toml'), '[tool.poetry]');
    fs.writeFileSync(path.join(projDir, '.python-version'), '3.11.4\n');

    const details = inspectProject(projDir, 'Active');
    expect(details?.stack).toContain('Rust');
    expect(details?.stack).toContain('Python');
    expect(details?.runtime_version).toBe('3.11.4');
  });

  it('should scan projects across config roots', () => {
    const activeDir = path.join(tmpDir, 'Active');
    const pausedDir = path.join(tmpDir, 'Paused');
    fs.mkdirSync(path.join(activeDir, 'proj-1'), { recursive: true });
    fs.mkdirSync(path.join(pausedDir, 'proj-2'), { recursive: true });

    const config: RtbConfig = {
      version: '1.0.0',
      projectRoots: {
        active: { path: activeDir, label: 'Active', emoji: '📁' },
        paused: { path: pausedDir, label: 'Paused', emoji: '⏸️' },
      },
    };

    const all = scanAllProjects(config, 'all');
    expect(all.length).toBe(2);
    expect(all.map((p) => p.name)).toContain('proj-1');
    expect(all.map((p) => p.name)).toContain('proj-2');

    const activeOnly = scanAllProjects(config, 'active');
    expect(activeOnly.length).toBe(1);
    expect(activeOnly[0].name).toBe('proj-1');
  });
});
