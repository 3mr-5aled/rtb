import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import path from 'node:path';
import fs from 'node:fs';
import os from 'node:os';
import { ProjectLifecycle, toKebabCase } from '../src/services/lifecycle.js';
import { DirtyGitError, ProjectNotFoundError, RtbError } from '../src/errors.js';
import type { RtbConfig } from '../src/types/config.js';

describe('ProjectLifecycle Domain Seam', () => {
  let tmpDir: string;
  let activeDir: string;
  let pausedDir: string;
  let config: RtbConfig;
  let lifecycle: ProjectLifecycle;

  beforeEach(() => {
    tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'rtb-lifecycle-seam-test-'));
    activeDir = path.join(tmpDir, '01-Active');
    pausedDir = path.join(tmpDir, '04-Paused');
    fs.mkdirSync(activeDir, { recursive: true });
    fs.mkdirSync(pausedDir, { recursive: true });

    config = {
      version: '1.0.0',
      projectRoots: {
        active: { path: activeDir, label: 'Active', emoji: '📁' },
        paused: { path: pausedDir, label: 'Paused', emoji: '⏸️' },
      },
      cleanDeps: {
        targets: ['node_modules', '.venv'],
      },
    };

    lifecycle = new ProjectLifecycle();
  });

  afterEach(() => {
    fs.rmSync(tmpDir, { recursive: true, force: true });
    vi.restoreAllMocks();
  });

  describe('toKebabCase', () => {
    it('converts mixed strings to kebab-case', () => {
      expect(toKebabCase('My Cool App')).toBe('my-cool-app');
      expect(toKebabCase('__TEST_Project 123--')).toBe('test-project-123');
    });
  });

  describe('create', () => {
    it('scaffolds project with README.md, PROJECT.md, and .gitignore in active root', () => {
      const result = lifecycle.create({
        name: 'test-project',
        stack: 'nextjs',
        activeRoot: activeDir,
      });

      expect(result.name).toBe('test-project');
      expect(result.path).toBe(path.join(activeDir, 'test-project'));
      expect(fs.existsSync(path.join(result.path, 'README.md'))).toBe(true);
      expect(fs.existsSync(path.join(result.path, 'PROJECT.md'))).toBe(true);
      expect(fs.existsSync(path.join(result.path, '.gitignore'))).toBe(true);
    });

    it('throws ALREADY_EXISTS error if target project directory already exists', () => {
      fs.mkdirSync(path.join(activeDir, 'existing-app'), { recursive: true });

      expect(() =>
        lifecycle.create({
          name: 'existing-app',
          activeRoot: activeDir,
        })
      ).toThrowError(/already exists/);
    });
  });

  describe('pause', () => {
    it('moves clean project from Active to Paused and optionally prunes dependencies', () => {
      const projDir = path.join(activeDir, 'clean-app');
      fs.mkdirSync(projDir, { recursive: true });
      fs.mkdirSync(path.join(projDir, 'node_modules'), { recursive: true });
      fs.writeFileSync(path.join(projDir, 'index.js'), 'console.log("hi");');

      const result = lifecycle.pause({
        name: 'clean-app',
        config,
        prune: true,
        force: true, // Bypass git check in fixture
      });

      expect(result.name).toBe('clean-app');
      expect(fs.existsSync(projDir)).toBe(false);
      const targetDir = path.join(pausedDir, 'clean-app');
      expect(fs.existsSync(targetDir)).toBe(true);
      expect(fs.existsSync(path.join(targetDir, 'node_modules'))).toBe(false);
      expect(fs.existsSync(path.join(targetDir, 'index.js'))).toBe(true);
    });

    it('throws ProjectNotFoundError if project does not exist in Active', () => {
      expect(() =>
        lifecycle.pause({
          name: 'non-existent',
          config,
          force: true,
        })
      ).toThrow(ProjectNotFoundError);
    });

    it('blocks pause if working tree is dirty unless force is passed', async () => {
      const gitModule = await import('../src/utils/git.js');
      vi.spyOn(gitModule, 'isGitClean').mockReturnValue(false);

      const projDir = path.join(activeDir, 'dirty-app');
      fs.mkdirSync(projDir, { recursive: true });

      expect(() =>
        lifecycle.pause({
          name: 'dirty-app',
          config,
          force: false,
        })
      ).toThrow(DirtyGitError);
    });
  });

  describe('resume', () => {
    it('moves project from Paused to Active', () => {
      const projDir = path.join(pausedDir, 'paused-app');
      fs.mkdirSync(projDir, { recursive: true });
      fs.writeFileSync(path.join(projDir, 'app.py'), 'print("hi")');

      const result = lifecycle.resume({
        name: 'paused-app',
        config,
        install: false,
      });

      expect(result.name).toBe('paused-app');
      expect(fs.existsSync(projDir)).toBe(false);
      const targetDir = path.join(activeDir, 'paused-app');
      expect(fs.existsSync(targetDir)).toBe(true);
      expect(fs.existsSync(path.join(targetDir, 'app.py'))).toBe(true);
    });

    it('throws ALREADY_EXISTS if destination project already exists in Active', () => {
      fs.mkdirSync(path.join(pausedDir, 'collision-app'), { recursive: true });
      fs.mkdirSync(path.join(activeDir, 'collision-app'), { recursive: true });

      expect(() =>
        lifecycle.resume({
          name: 'collision-app',
          config,
          install: false,
        })
      ).toThrowError(/already exists/);
    });
  });
});
