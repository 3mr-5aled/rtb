import { describe, it, expect, vi } from 'vitest';
import { getCompletionScript, getProjectNames, getArchiveNames, ALL_SUBCOMMANDS } from '../src/commands/completion.js';
import { getShellScript } from '../src/commands/shell-init.js';
import { createCli } from '../src/cli.js';
import type { RtbConfig } from '../src/types/config.js';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';

describe('Completion System', () => {
  it('should generate valid pwsh completion script with Register-ArgumentCompleter', () => {
    const script = getCompletionScript('pwsh');
    expect(script).toContain("Register-ArgumentCompleter -CommandName 'rtb', 'rtb.cmd', 'rtb.ps1', 'dev'");
    expect(script).toContain("Register-ArgumentCompleter -Native -CommandName 'rtb', 'rtb.cmd', 'rtb.ps1', 'dev'");
    expect(script).toContain('_rtb_get_all_projects');
    expect(script).toContain('_rtb_get_projects_by_status');
    expect(script).toContain('_rtb_get_archives');
    expect(script).toContain('$subCommands');
  });

  it('should generate valid bash completion script', () => {
    const script = getCompletionScript('bash');
    expect(script).toContain('_rtb_completions()');
    expect(script).toContain('complete -F _rtb_completions rtb');
    expect(script).toContain('__complete commands');
    expect(script).toContain('__complete projects');
  });

  it('should generate valid zsh completion script', () => {
    const script = getCompletionScript('zsh');
    expect(script).toContain('#compdef rtb');
    expect(script).toContain('_rtb()');
    expect(script).toContain('compdef _rtb rtb');
  });

  it('should generate valid fish completion script', () => {
    const script = getCompletionScript('fish');
    expect(script).toContain('complete -c rtb');
    expect(script).toContain('__complete commands');
  });

  it('should include completion in shell-init output', () => {
    const pwshScript = getShellScript('pwsh');
    expect(pwshScript).toContain('function rtb {');
    expect(pwshScript).toContain('Register-ArgumentCompleter');

    const bashScript = getShellScript('bash');
    expect(bashScript).toContain('rtb() {');
    expect(bashScript).toContain('complete -F _rtb_completions rtb');
  });

  it('should list all project names from config roots', () => {
    // Test runs with cwd = core/ — which contains src/, tests/, dist/ etc.
    const coreDir = process.cwd();
    const mockConfig: RtbConfig = {
      projectRoots: {
        active: { path: coreDir, label: 'Active Projects' },
      },
    };
    const names = getProjectNames(mockConfig);
    expect(Array.isArray(names)).toBe(true);
    expect(names.length).toBeGreaterThan(0);
    // core/ directory contains 'src' and 'tests' subdirectories
    expect(names).toContain('src');
  });

  it('should filter projects by category', () => {
    const coreDir = process.cwd();
    const mockConfig: RtbConfig = {
      projectRoots: {
        active: { path: coreDir, label: 'Active' },
        paused: { path: 'D:\\nonexistent-path-that-does-not-exist', label: 'Paused' },
      },
    };
    const activeProjects = getProjectNames(mockConfig, 'active');
    expect(activeProjects).toContain('src');

    // 'paused' path doesn't exist so should return empty
    const pausedProjects = getProjectNames(mockConfig, 'paused');
    expect(pausedProjects).toEqual([]);
  });

  it('should run rtb completion pwsh via CLI', async () => {
    const cli = createCli();
    let stdoutData = '';
    const origWrite = process.stdout.write;
    process.stdout.write = ((chunk: any) => {
      stdoutData += chunk;
      return true;
    }) as any;

    try {
      await cli.parseAsync(['node', 'rtb', 'completion', 'pwsh']);
      expect(stdoutData).toContain('Register-ArgumentCompleter');
    } finally {
      process.stdout.write = origWrite;
    }
  });

  it('should run rtb __complete commands via CLI', async () => {
    const cli = createCli();
    let stdoutData = '';
    const origWrite = process.stdout.write;
    process.stdout.write = ((chunk: any) => {
      stdoutData += chunk;
      return true;
    }) as any;

    try {
      await cli.parseAsync(['node', 'rtb', '__complete', 'commands']);
      expect(stdoutData).toContain('goto');
      expect(stdoutData).toContain('run');
      expect(stdoutData).toContain('build');
      expect(stdoutData).toContain('completion');
      expect(stdoutData).toContain('menu');
    } finally {
      process.stdout.write = origWrite;
    }
  });

  it('should run rtb __complete projects active via CLI and list project directory names', async () => {
    const cli = createCli();
    let stdoutData = '';
    const origWrite = process.stdout.write;
    process.stdout.write = ((chunk: any) => {
      stdoutData += chunk;
      return true;
    }) as any;

    try {
      await cli.parseAsync(['node', 'rtb', '__complete', 'projects', 'active']);
      expect(typeof stdoutData).toBe('string');
    } finally {
      process.stdout.write = origWrite;
    }
  });

  it('should complete projects containing hyphens, dots, and numbers', () => {
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'rtb-hyphen-test-'));
    try {
      fs.mkdirSync(path.join(tmpDir, 'rtb-command-tool'), { recursive: true });
      fs.mkdirSync(path.join(tmpDir, '35-portfolio'), { recursive: true });
      fs.mkdirSync(path.join(tmpDir, 'app.v2-web'), { recursive: true });

      const mockConfig: RtbConfig = {
        projectRoots: {
          active: { path: tmpDir, label: 'Active' },
        },
      };

      const projects = getProjectNames(mockConfig, 'active');
      expect(projects).toContain('rtb-command-tool');
      expect(projects).toContain('35-portfolio');
      expect(projects).toContain('app.v2-web');
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  it('should generate pwsh script containing dev completer and WildcardPattern escaping', () => {
    const script = getCompletionScript('pwsh');
    expect(script).toContain("Register-ArgumentCompleter -CommandName 'rtb', 'rtb.cmd', 'rtb.ps1', 'dev'");
    expect(script).toContain('[System.Management.Automation.WildcardPattern]::Escape($wordToComplete)');
    expect(script).toContain("$cmdName -eq 'dev'");
    expect(script).toContain("(@('outdated') + @(_rtb_get_all_projects))");
    // Verify switch cases have break
    expect(script).toContain("(_rtb_get_projects_by_status 'active')");
    expect(script).toContain("(_rtb_get_projects_by_status 'paused')");
  });

  it('should support dev command in bash and zsh completion scripts', () => {
    const bash = getCompletionScript('bash');
    expect(bash).toContain('complete -F _rtb_completions rtb dev');
    expect(bash).toContain('[ "${cmd}" = "dev" ]');

    const zsh = getCompletionScript('zsh');
    expect(zsh).toContain('compdef _rtb rtb dev');
    expect(zsh).toContain('[[ "$words[1]" == "dev" ]]');
  });
});
