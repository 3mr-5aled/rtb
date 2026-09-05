import { describe, it, expect, vi } from 'vitest';
import { getRawLogo, renderLogo, getLogoLines, getLogo } from '../src/utils/logo.js';
import { getHeroBanner } from '../src/utils/banner.js';
import { getCustomHelpMenu } from '../src/commands/help.js';
import { createCli } from '../src/cli.js';
import { RTB_VERSION } from '../src/commands/version.js';
import type { CliContext } from '../types/context.js';

describe('Logo Loader & HeroBanner', () => {
  it('should load non-empty raw logo content', () => {
    const raw = getRawLogo();
    expect(raw).toBeDefined();
    expect(raw.length).toBeGreaterThan(50);
    expect(raw).not.toMatch(/^\uFEFF/); // Asserts BOM is stripped
    expect(getLogo()).toBe(renderLogo());
  });

  it('should render ANSI colored logo lines in default color mode', () => {
    const rendered = renderLogo();
    expect(rendered).toContain('\x1b[');
    const lines = getLogoLines();
    expect(lines.length).toBeGreaterThan(5);
  });

  it('should cleanly strip ANSI codes when color is disabled', () => {
    const plain = renderLogo({ color: false });
    expect(plain).not.toContain('\x1b[');
    expect(plain).toContain('⣿');
  });

  it('should suppress logo when quiet or json mode is requested', () => {
    expect(renderLogo({ quiet: true })).toBe('');
    expect(renderLogo({ json: true })).toBe('');
  });

  it('should generate rich context-aware HeroBanner for configured workspace', () => {
    const mockCtx: CliContext = {
      config: {
        version: '1.2.0',
        projectRoots: {
          active: {
            path: 'C:\\Users\\devamr\\mock-workspace\\01-Active',
            label: 'Active Projects',
            emoji: '🟢',
          },
        },
      },
      configPath: 'test.json',
      isConfigured: true,
      isJson: false,
      isQuiet: false,
      isInteractive: true,
    };

    // Pass mock cwd outside workspace
    const banner = getHeroBanner(mockCtx, 'C:\\Users\\devamr\\other-dir');
    expect(banner).toContain('rtb');
    expect(banner).toContain(`v${RTB_VERSION}`);
    expect(banner).toContain('QUICK ACTIONS');
    expect(banner).toContain('rtb menu');
    expect(banner).toContain('rtb run');
    expect(banner).toContain('Workspace:');
    expect(banner).toContain('mock-workspace');
  });

  it('should suppress HeroBanner in quiet or json mode', () => {
    const mockCtxQuiet: CliContext = {
      config: null,
      configPath: '',
      isConfigured: false,
      isJson: false,
      isQuiet: true,
      isInteractive: false,
    };
    expect(getHeroBanner(mockCtxQuiet)).toBe('');

    const mockCtxJson: CliContext = {
      config: null,
      configPath: '',
      isConfigured: false,
      isJson: true,
      isQuiet: false,
      isInteractive: false,
    };
    expect(getHeroBanner(mockCtxJson)).toBe('');
  });

  it('should include logo in custom help menu when not suppressed', () => {
    const menu = getCustomHelpMenu();
    expect(menu).toContain('rtb');
    expect(menu).toContain('SETUP & CONFIG');
    expect(menu).toContain('QUICK INTERACTION');
    expect(menu).toContain('rtb menu');
  });

  it('should execute bare rtb command and print HeroBanner', async () => {
    const cli = createCli();
    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});

    await cli.parseAsync(['node', 'rtb']);

    expect(logSpy).toHaveBeenCalled();
    const output = logSpy.mock.calls[0][0];
    expect(output).toContain('rtb');
    expect(output).toContain(`v${RTB_VERSION}`);
    expect(output).toContain('QUICK ACTIONS');
    logSpy.mockRestore();
  }, 15000);

  it('should execute bare rtb --json and return structured status JSON', async () => {
    const cli = createCli();
    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});

    await cli.parseAsync(['node', 'rtb', '--json']);

    expect(logSpy).toHaveBeenCalled();
    const parsed = JSON.parse(logSpy.mock.calls[0][0]);
    expect(parsed.name).toBe('rtb');
    expect(parsed.version).toBe(RTB_VERSION);
    expect(parsed.status).toBe('ready');
    logSpy.mockRestore();
  });
});
