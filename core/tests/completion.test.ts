import { describe, it, expect, vi } from 'vitest';
import { getCompletionScript, getProjectNames, getArchiveNames, ALL_SUBCOMMANDS } from '../src/commands/completion.js';
import { getShellScript } from '../src/commands/shell-init.js';
import { createCli } from '../src/cli.js';
import type { RtbConfig } from '../src/types/config.js';
import fs from 'node:fs';

describe('Completion System', () => {
  it('should generate valid pwsh completion script with Register-ArgumentCompleter', () => {
    const script = getCompletionScript('pwsh');
    expect(script).toContain('Register-ArgumentCompleter -CommandName \'rtb\', \'dev\'');
    expect(script).toContain('Register-ArgumentCompleter -Native -CommandName \'rtb\', \'dev\'');
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
    } finally {
      process.stdout.write = origWrite;
    }
  });
});
