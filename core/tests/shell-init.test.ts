import { describe, it, expect } from 'vitest';
import { getShellScript } from '../src/commands/shell-init.js';
import { createCli } from '../src/cli.js';

describe('Shell Integration (shell-init)', () => {
  it('should generate valid bash wrapper function', () => {
    const script = getShellScript('bash');
    expect(script).toContain('rtb() {');
    expect(script).toContain('goto');
    expect(script).toContain('cd "$target"');
    expect(script).toContain('command rtb goto "$@" --print');
    expect(script).toContain('goto() {');
  });

  it('should generate valid zsh wrapper function', () => {
    const script = getShellScript('zsh');
    expect(script).toContain('rtb() {');
    expect(script).toContain('cd "$target"');
    expect(script).toContain('goto() {');
  });

  it('should generate valid fish wrapper function', () => {
    const script = getShellScript('fish');
    expect(script).toContain('function rtb');
    expect(script).toContain('cd "$target"');
    expect(script).toContain('command rtb goto $goto_args --print');
    expect(script).toContain('function goto');
  });

  it('should generate valid pwsh wrapper function', () => {
    const script = getShellScript('pwsh');
    expect(script).toContain('function rtb {');
    expect(script).toContain('Set-Location -LiteralPath $target');
    expect(script).toContain('@($args | Select-Object -Skip 1)');
    expect(script).toContain('function goto {');
  });

  it('should throw for unsupported shells', () => {
    expect(() => getShellScript('tcsh')).toThrow(/Unsupported shell/);
  });

  it('should run shell-init via CLI without failing Config Gate', async () => {
    const cli = createCli();
    let stdoutData = '';
    const origWrite = process.stdout.write;
    process.stdout.write = ((chunk: any) => {
      stdoutData += chunk;
      return true;
    }) as any;

    try {
      await cli.parseAsync(['node', 'rtb', 'shell-init', 'bash']);
      expect(stdoutData).toContain('rtb() {');
      expect(stdoutData).toContain('eval "$(rtb shell-init bash)"');
    } finally {
      process.stdout.write = origWrite;
    }
  });
});
