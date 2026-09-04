import { describe, it, expect, vi } from 'vitest';
import { openPath, resolveOpenCommand } from '../src/utils/opener.js';

describe('openPath & resolveOpenCommand', () => {
  it('resolves correct platform open command', () => {
    expect(resolveOpenCommand('win32')).toBe('explorer.exe');
    expect(resolveOpenCommand('darwin')).toBe('open');
    expect(resolveOpenCommand('linux')).toBe('xdg-open');
  });

  it('invokes launcher with resolved platform command and target path', () => {
    const launcher = vi.fn();
    openPath('/some/path', launcher, 'darwin');
    expect(launcher).toHaveBeenCalledWith('open', ['/some/path']);
  });
});
