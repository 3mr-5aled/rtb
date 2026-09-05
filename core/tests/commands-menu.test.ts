import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { createCli } from '../src/cli.js';
import { prompts, MENU_ACTIONS, runMenuAction } from '../src/commands/menu.js';
import type { CliContext } from '../types/context.js';
import type { RtbConfig } from '../types/config.js';

describe('rtb menu interactive command menu', () => {
  let tmpHome: string;
  let tmpConfigDir: string;
  let tmpWorkspace: string;
  let activeDir: string;
  let mockConfig: RtbConfig;
  let mockContext: CliContext;

  beforeEach(() => {
    tmpHome = fs.mkdtempSync(path.join(os.tmpdir(), 'rtb-menu-test-'));
    tmpConfigDir = path.join(tmpHome, '.config', 'rtb');
    tmpWorkspace = path.join(tmpHome, 'Projects');
    activeDir = path.join(tmpWorkspace, '01-Active');

    fs.mkdirSync(path.join(activeDir, 'alpha-service'), { recursive: true });
    fs.mkdirSync(path.join(activeDir, 'web-dashboard'), { recursive: true });

    mockConfig = {
      version: '1.0',
      projectRoots: {
        active: { path: activeDir, label: 'Active Projects', emoji: '⚡' },
      },
    };

    fs.mkdirSync(tmpConfigDir, { recursive: true });
    fs.writeFileSync(
      path.join(tmpConfigDir, 'rtb.config.json'),
      JSON.stringify(mockConfig, null, 2)
    );

    mockContext = {
      config: mockConfig,
      configPath: path.join(tmpConfigDir, 'rtb.config.json'),
      isConfigured: true,
      isJson: false,
      isQuiet: false,
      isInteractive: true,
    };
  });

  afterEach(() => {
    fs.rmSync(tmpHome, { recursive: true, force: true });
    vi.restoreAllMocks();
  });

  describe('Menu Structure', () => {
    it('should define all 7 required menu actions', () => {
      const keys = MENU_ACTIONS.map((a) => a.value);
      expect(keys).toEqual([
        'run_build_test',
        'goto',
        'ui',
        'health_doctor',
        'agent',
        'config',
        'exit',
      ]);
    });
  });

  describe('Headless & JSON handling', () => {
    it('should return cleanly when invoked in non-interactive/json mode', async () => {
      let logged = '';
      const logSpy = vi.spyOn(console, 'log').mockImplementation((...args: any[]) => {
        logged += args.join(' ') + '\n';
      });

      try {
        const cli = createCli();
        await cli.parseAsync(['node', 'rtb', 'menu', '--json']);

        const parsed = JSON.parse(logged.trim());
        expect(parsed.status).toBe('interactive_only');
      } finally {
        logSpy.mockRestore();
      }
    });
  });

  describe('Interactive Selection Routing', () => {
    it('should exit cleanly when exit is selected', async () => {
      const introSpy = vi.spyOn(prompts, 'intro').mockImplementation(() => {});
      const selectSpy = vi.spyOn(prompts, 'select').mockResolvedValue('exit');
      const outroSpy = vi.spyOn(prompts, 'outro').mockImplementation(() => {});

      try {
        await runMenuAction('exit', mockContext);
        expect(outroSpy).toHaveBeenCalledWith('Goodbye!');
      } finally {
        introSpy.mockRestore();
        selectSpy.mockRestore();
        outroSpy.mockRestore();
      }
    });

    it('should route to goto project and display project path', async () => {
      const selectSpy = vi.spyOn(prompts, 'select').mockResolvedValue('alpha-service');
      const outroSpy = vi.spyOn(prompts, 'outro').mockImplementation(() => {});
      let logged = '';
      const logSpy = vi.spyOn(console, 'log').mockImplementation((...args: any[]) => {
        logged += args.join(' ') + '\n';
      });

      try {
        await runMenuAction('goto', mockContext);

        expect(selectSpy).toHaveBeenCalled();
        expect(outroSpy).toHaveBeenCalled();
        expect(logged).toContain('alpha-service');
      } finally {
        selectSpy.mockRestore();
        outroSpy.mockRestore();
        logSpy.mockRestore();
      }
    });

    it('should route to config and view configuration summary', async () => {
      const selectSpy = vi.spyOn(prompts, 'select').mockResolvedValue('view');
      const outroSpy = vi.spyOn(prompts, 'outro').mockImplementation(() => {});
      let logged = '';
      const logSpy = vi.spyOn(console, 'log').mockImplementation((...args: any[]) => {
        logged += args.join(' ') + '\n';
      });

      try {
        await runMenuAction('config', mockContext);

        expect(selectSpy).toHaveBeenCalled();
        expect(outroSpy).toHaveBeenCalled();
        expect(logged).toContain('Configuration File:');
      } finally {
        selectSpy.mockRestore();
        outroSpy.mockRestore();
        logSpy.mockRestore();
      }
    });

    it('should route to health_doctor and execute doctor diagnostic checks', async () => {
      const selectSpy = vi.spyOn(prompts, 'select').mockResolvedValue('doctor');
      const outroSpy = vi.spyOn(prompts, 'outro').mockImplementation(() => {});
      const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});

      try {
        await runMenuAction('health_doctor', mockContext);

        expect(selectSpy).toHaveBeenCalled();
        expect(outroSpy).toHaveBeenCalled();
      } finally {
        selectSpy.mockRestore();
        outroSpy.mockRestore();
        logSpy.mockRestore();
      }
    });

    it('should route to run_build_test and select project action', async () => {
      // First select 'run', then select project 'alpha-service'
      const selectSpy = vi
        .spyOn(prompts, 'select')
        .mockResolvedValueOnce('run')
        .mockResolvedValueOnce('alpha-service');
      const outroSpy = vi.spyOn(prompts, 'outro').mockImplementation(() => {});
      const warnSpy = vi.spyOn(prompts.log, 'warn').mockImplementation(() => {});

      try {
        await runMenuAction('run_build_test', mockContext);

        expect(selectSpy).toHaveBeenCalledTimes(2);
        expect(outroSpy).toHaveBeenCalled();
      } finally {
        selectSpy.mockRestore();
        outroSpy.mockRestore();
        warnSpy.mockRestore();
      }
    });

    it('should handle menu cancellation gracefully', async () => {
      const cancelSymbol = Symbol('clack:cancel');
      const isCancelSpy = vi.spyOn(prompts, 'isCancel').mockReturnValue(true);
      const outroSpy = vi.spyOn(prompts, 'outro').mockImplementation(() => {});

      try {
        await runMenuAction(cancelSymbol as any, mockContext);
        expect(outroSpy).toHaveBeenCalledWith('Goodbye!');
      } finally {
        isCancelSpy.mockRestore();
        outroSpy.mockRestore();
      }
    });

    it('should run interactive menu CLI and invoke selected action', async () => {
      const introSpy = vi.spyOn(prompts, 'intro').mockImplementation(() => {});
      const selectSpy = vi.spyOn(prompts, 'select').mockResolvedValue('exit');
      const outroSpy = vi.spyOn(prompts, 'outro').mockImplementation(() => {});

      const origIsTTY = process.stdin.isTTY;
      const origCI = process.env.CI;
      const origNonInteractive = process.env.RTB_NON_INTERACTIVE;

      (process.stdin as any).isTTY = true;
      delete process.env.CI;
      delete process.env.RTB_NON_INTERACTIVE;

      try {
        const cli = createCli();
        await cli.parseAsync(['node', 'rtb', 'menu']);

        expect(introSpy).toHaveBeenCalled();
        expect(selectSpy).toHaveBeenCalled();
        expect(outroSpy).toHaveBeenCalledWith('Goodbye!');
      } finally {
        (process.stdin as any).isTTY = origIsTTY;
        if (origCI !== undefined) process.env.CI = origCI;
        if (origNonInteractive !== undefined) process.env.RTB_NON_INTERACTIVE = origNonInteractive;

        introSpy.mockRestore();
        selectSpy.mockRestore();
        outroSpy.mockRestore();
      }
    });
  });
});
