import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { createCli } from '../src/cli.js';
import {
  prompts,
  LIFECYCLE_OPTIONS,
  getShellIntegrationSnippet,
  getShellProfilePath,
  configureShellIntegration,
  deployCliLauncher,
} from '../src/commands/init.js';
import type { RtbConfig } from '../src/types/config.js';

describe('rtb init onboarding wizard & configuration', () => {
  let tmpHome: string;
  let tmpConfigDir: string;
  let tmpWorkspace: string;
  let origConfigDirEnv: string | undefined;

  beforeEach(() => {
    tmpHome = fs.mkdtempSync(path.join(os.tmpdir(), 'rtb-init-test-'));
    tmpConfigDir = path.join(tmpHome, '.config', 'rtb');
    tmpWorkspace = path.join(tmpHome, 'Projects');

    origConfigDirEnv = process.env.RTB_CONFIG_DIR;
    process.env.RTB_CONFIG_DIR = tmpConfigDir;
  });

  afterEach(() => {
    if (origConfigDirEnv !== undefined) {
      process.env.RTB_CONFIG_DIR = origConfigDirEnv;
    } else {
      delete process.env.RTB_CONFIG_DIR;
    }
    fs.rmSync(tmpHome, { recursive: true, force: true });
    vi.restoreAllMocks();
  });

  describe('Lifecycle Scaffolding Options', () => {
    it('should define all 8 lifecycle folders', () => {
      const keys = LIFECYCLE_OPTIONS.map((o) => o.key);
      expect(keys).toEqual([
        'active',
        'planning',
        'testing',
        'paused',
        'abandoned',
        'production',
        'staging',
        'vibe',
      ]);
    });
  });

  describe('Shell Integration Helpers', () => {
    it('should generate correct integration snippets for all shells', () => {
      expect(getShellIntegrationSnippet('pwsh')).toContain('(& rtb shell-init pwsh | Out-String) | Invoke-Expression');
      expect(getShellIntegrationSnippet('bash')).toContain('eval "$(rtb shell-init bash)"');
      expect(getShellIntegrationSnippet('zsh')).toContain('eval "$(rtb shell-init zsh)"');
      expect(getShellIntegrationSnippet('fish')).toContain('rtb shell-init fish | source');
    });

    it('should configure shell profile file and handle idempotency', () => {
      const mockProfile = path.join(tmpHome, '.mock_profile');
      const res1 = configureShellIntegration('bash', mockProfile);
      expect(res1.success).toBe(true);
      expect(fs.existsSync(mockProfile)).toBe(true);

      const content = fs.readFileSync(mockProfile, 'utf8');
      expect(content).toContain('rtb shell-init bash');

      // Second invocation should be idempotent
      const res2 = configureShellIntegration('bash', mockProfile);
      expect(res2.success).toBe(true);
      expect(res2.message).toContain('already configured');

      const contentAfter = fs.readFileSync(mockProfile, 'utf8');
      const occurrences = (contentAfter.match(/rtb shell-init/g) || []).length;
      expect(occurrences).toBe(1);
    });

    it('should upgrade legacy bare shell integration in existing profile', () => {
      const mockProfile = path.join(tmpHome, '.legacy_profile');
      fs.writeFileSync(mockProfile, '# Old stuff\n(& rtb shell-init pwsh | Out-String) | Invoke-Expression\n', 'utf8');

      const res = configureShellIntegration('pwsh', mockProfile);
      expect(res.success).toBe(true);
      expect(res.message).toContain('Upgraded shell integration');

      const contentAfter = fs.readFileSync(mockProfile, 'utf8');
      expect(contentAfter).toContain('$rtbBin =');
      expect(contentAfter).toContain('(& rtb shell-init pwsh | Out-String) | Invoke-Expression');
    });
  });

  describe('Headless / Flag Invocations', () => {
    it('should initialize headless workspace when --force and --root are passed', async () => {
      const cli = createCli();
      await cli.parseAsync(['node', 'rtb', 'init', '--force', '--root', tmpWorkspace]);

      const configFile = path.join(tmpConfigDir, 'rtb.config.json');
      expect(fs.existsSync(configFile)).toBe(true);

      const config: RtbConfig = JSON.parse(fs.readFileSync(configFile, 'utf8'));
      expect(config.version).toBe('1.0');
      expect(config.projectRoots.active.path).toBe(path.join(tmpWorkspace, '01-Active'));
      expect(config.projectRoots.paused.path).toBe(path.join(tmpWorkspace, '04-Paused'));
      expect(fs.existsSync(path.join(tmpWorkspace, '01-Active'))).toBe(true);
      expect(fs.existsSync(path.join(tmpWorkspace, '04-Paused'))).toBe(true);
    });

    it('should support --flat option headlessly', async () => {
      const cli = createCli();
      await cli.parseAsync(['node', 'rtb', 'init', '--force', '--root', tmpWorkspace, '--flat']);

      const configFile = path.join(tmpConfigDir, 'rtb.config.json');
      const config: RtbConfig = JSON.parse(fs.readFileSync(configFile, 'utf8'));
      expect(config.projectRoots.projects.path).toBe(tmpWorkspace);
      expect(config.projectRoots.active.path).toBe(tmpWorkspace);
    });

    it('should return JSON format when --json is passed', async () => {
      let logged = '';
      const logSpy = vi.spyOn(console, 'log').mockImplementation((...args: any[]) => {
        logged += args.join(' ') + '\n';
      });

      try {
        const cli = createCli();
        await cli.parseAsync(['node', 'rtb', 'init', '--force', '--root', tmpWorkspace, '--json']);

        expect(logged).toContain('"status": "success"');
        const parsed = JSON.parse(logged.trim());
        expect(parsed.status).toBe('success');
        expect(parsed.config.projectRoots.active).toBeDefined();
      } finally {
        logSpy.mockRestore();
      }
    });

    it('should return status: already_configured when config exists without --force under --json', async () => {
      fs.mkdirSync(tmpConfigDir, { recursive: true });
      fs.writeFileSync(path.join(tmpConfigDir, 'rtb.config.json'), '{"version":"1.0"}');

      let logged = '';
      const logSpy = vi.spyOn(console, 'log').mockImplementation((...args: any[]) => {
        logged += args.join(' ') + '\n';
      });

      try {
        const cli = createCli();
        await cli.parseAsync(['node', 'rtb', 'init', '--json']);

        const parsed = JSON.parse(logged.trim());
        expect(parsed.status).toBe('already_configured');
      } finally {
        logSpy.mockRestore();
      }
    });

    it('should execute full installation setup via rtb install command', async () => {
      let logged = '';
      const logSpy = vi.spyOn(console, 'log').mockImplementation((...args: any[]) => {
        logged += args.join(' ') + '\n';
      });

      try {
        const cli = createCli();
        await cli.parseAsync(['node', 'rtb', 'install', '--force', '--root', tmpWorkspace, '--json']);

        const parsed = JSON.parse(logged.trim());
        expect(parsed.status).toBe('success');
        expect(parsed.configPath).toBe(path.join(tmpConfigDir, 'rtb.config.json'));
        expect(parsed.launcherPath).toBeDefined();
        expect(fs.existsSync(parsed.configPath)).toBe(true);
      } finally {
        logSpy.mockRestore();
      }
    });
  });

  describe('Interactive Wizard Flow (@clack/prompts)', () => {
    let origIsTTY: any;
    let origCI: string | undefined;
    let origNonInteractive: string | undefined;

    beforeEach(() => {
      origIsTTY = process.stdin.isTTY;
      origCI = process.env.CI;
      origNonInteractive = process.env.RTB_NON_INTERACTIVE;

      (process.stdin as any).isTTY = true;
      delete process.env.CI;
      delete process.env.RTB_NON_INTERACTIVE;
    });

    afterEach(() => {
      (process.stdin as any).isTTY = origIsTTY;
      if (origCI !== undefined) process.env.CI = origCI;
      if (origNonInteractive !== undefined) process.env.RTB_NON_INTERACTIVE = origNonInteractive;
    });

    it('should execute full 5-step interactive wizard with custom folder selection', async () => {
      const introSpy = vi.spyOn(prompts, 'intro').mockImplementation(() => {});
      const selectSpy = vi.spyOn(prompts, 'select').mockResolvedValue(tmpWorkspace);
      const multiselectSpy = vi.spyOn(prompts, 'multiselect').mockResolvedValue(['active', 'vibe', 'staging']);
      const confirmSpy = vi.spyOn(prompts, 'confirm').mockResolvedValue(false);
      const outroSpy = vi.spyOn(prompts, 'outro').mockImplementation(() => {});

      try {
        const cli = createCli();
        await cli.parseAsync(['node', 'rtb', 'init']);

        expect(introSpy).toHaveBeenCalled();
        expect(selectSpy).toHaveBeenCalled();
        expect(multiselectSpy).toHaveBeenCalled();
        expect(confirmSpy).toHaveBeenCalled();
        expect(outroSpy).toHaveBeenCalled();

        const configFile = path.join(tmpConfigDir, 'rtb.config.json');
        expect(fs.existsSync(configFile)).toBe(true);

        const config: RtbConfig = JSON.parse(fs.readFileSync(configFile, 'utf8'));
        expect(config.projectRoots.active).toBeDefined();
        expect(config.projectRoots.vibe).toBeDefined();
        expect(config.projectRoots.staging).toBeDefined();
        expect(fs.existsSync(path.join(tmpWorkspace, '01-Active'))).toBe(true);
        expect(fs.existsSync(path.join(tmpWorkspace, '08-Vibe'))).toBe(true);
        expect(fs.existsSync(path.join(tmpWorkspace, '07-Staging'))).toBe(true);
      } finally {
        introSpy.mockRestore();
        selectSpy.mockRestore();
        multiselectSpy.mockRestore();
        confirmSpy.mockRestore();
        outroSpy.mockRestore();
      }
    });

    it('should handle cancellation at prompt gracefully without writing config', async () => {
      const cancelSymbol = Symbol('clack:cancel');
      const introSpy = vi.spyOn(prompts, 'intro').mockImplementation(() => {});
      const selectSpy = vi.spyOn(prompts, 'select').mockResolvedValue(cancelSymbol as any);
      const isCancelSpy = vi.spyOn(prompts, 'isCancel').mockReturnValue(true);
      const cancelSpy = vi.spyOn(prompts, 'cancel').mockImplementation(() => {});

      try {
        const cli = createCli();
        await cli.parseAsync(['node', 'rtb', 'init']);

        expect(introSpy).toHaveBeenCalled();
        expect(selectSpy).toHaveBeenCalled();
        expect(cancelSpy).toHaveBeenCalledWith('Setup cancelled.');

        const configFile = path.join(tmpConfigDir, 'rtb.config.json');
        expect(fs.existsSync(configFile)).toBe(false);
      } finally {
        introSpy.mockRestore();
        selectSpy.mockRestore();
        isCancelSpy.mockRestore();
        cancelSpy.mockRestore();
      }
    });

    it('should prompt to download rtbtui now and invoke provisionRtbtuiBinary when chosen', async () => {
      const introSpy = vi.spyOn(prompts, 'intro').mockImplementation(() => {});
      const selectSpy = vi
        .spyOn(prompts, 'select')
        .mockResolvedValueOnce(tmpWorkspace) // workspace root
        .mockResolvedValueOnce('now'); // UI choice
      const multiselectSpy = vi.spyOn(prompts, 'multiselect').mockResolvedValue(['active']);
      const confirmSpy = vi.spyOn(prompts, 'confirm').mockResolvedValue(false);
      const outroSpy = vi.spyOn(prompts, 'outro').mockImplementation(() => {});
      const spinnerSpy = vi.spyOn(prompts, 'spinner').mockReturnValue({
        start: vi.fn(),
        stop: vi.fn(),
        message: vi.fn(),
      } as any);

      const doctorMod = await import('../src/commands/doctor.js');
      const tuiSpy = vi.spyOn(doctorMod, 'findRtbtuiBinary').mockReturnValue(null);

      const uiMod = await import('../src/commands/ui.js');
      const provisionSpy = vi.spyOn(uiMod, 'provisionRtbtuiBinary').mockResolvedValue('/mock/bin/rtbtui');

      try {
        const cli = createCli();
        await cli.parseAsync(['node', 'rtb', 'init']);

        expect(selectSpy).toHaveBeenCalledTimes(2);
        expect(provisionSpy).toHaveBeenCalled();
      } finally {
        introSpy.mockRestore();
        selectSpy.mockRestore();
        multiselectSpy.mockRestore();
        confirmSpy.mockRestore();
        outroSpy.mockRestore();
        spinnerSpy.mockRestore();
        tuiSpy.mockRestore();
        provisionSpy.mockRestore();
      }
    });

    it('should skip UI download prompt when --skip-ui is passed', async () => {
      const introSpy = vi.spyOn(prompts, 'intro').mockImplementation(() => {});
      const selectSpy = vi.spyOn(prompts, 'select').mockResolvedValue(tmpWorkspace);
      const multiselectSpy = vi.spyOn(prompts, 'multiselect').mockResolvedValue(['active']);
      const confirmSpy = vi.spyOn(prompts, 'confirm').mockResolvedValue(false);
      const outroSpy = vi.spyOn(prompts, 'outro').mockImplementation(() => {});

      const uiMod = await import('../src/commands/ui.js');
      const provisionSpy = vi.spyOn(uiMod, 'provisionRtbtuiBinary').mockResolvedValue('/mock/bin/rtbtui');

      try {
        const cli = createCli();
        await cli.parseAsync(['node', 'rtb', 'init', '--skip-ui']);

        expect(selectSpy).toHaveBeenCalledTimes(1); // Only root prompt
        expect(provisionSpy).not.toHaveBeenCalled();
      } finally {
        introSpy.mockRestore();
        selectSpy.mockRestore();
        multiselectSpy.mockRestore();
        confirmSpy.mockRestore();
        outroSpy.mockRestore();
        provisionSpy.mockRestore();
      }
    });

    it('should keep existing configuration and continue installation when overwrite is declined', async () => {
      fs.mkdirSync(tmpConfigDir, { recursive: true });
      const initialConfig = {
        version: '1.0',
        projectRoots: {
          active: { path: path.join(tmpWorkspace, '01-Active'), label: 'Custom Active', emoji: '🟢' },
        },
      };
      fs.writeFileSync(path.join(tmpConfigDir, 'rtb.config.json'), JSON.stringify(initialConfig, null, 2));

      const introSpy = vi.spyOn(prompts, 'intro').mockImplementation(() => {});
      const confirmSpy = vi.spyOn(prompts, 'confirm').mockImplementation(async (opts: any) => {
        if (opts.message.includes('Overwrite')) return false; // decline overwrite
        if (opts.message.includes('shell integration')) return false;
        return false;
      });
      const outroSpy = vi.spyOn(prompts, 'outro').mockImplementation(() => {});

      try {
        const cli = createCli();
        await cli.parseAsync(['node', 'rtb', 'install', '--skip-ui']);

        expect(confirmSpy).toHaveBeenCalled();
        expect(outroSpy).toHaveBeenCalled();

        // Config file should be preserved
        const configAfter = JSON.parse(fs.readFileSync(path.join(tmpConfigDir, 'rtb.config.json'), 'utf8'));
        expect(configAfter.projectRoots.active.label).toBe('Custom Active');

        // Launcher should be deployed
        const binDir = path.join(tmpConfigDir, 'bin');
        expect(fs.existsSync(path.join(binDir, 'rtb-cli.js'))).toBe(true);
      } finally {
        introSpy.mockRestore();
        confirmSpy.mockRestore();
        outroSpy.mockRestore();
      }
    });
  });

  describe('deployCliLauncher', () => {
    it('should deploy launcher files into specified bin directory', () => {
      const customBin = path.join(tmpHome, 'custom-bin');
      const res = deployCliLauncher(customBin);

      expect(res.success).toBe(true);
      expect(res.binDir).toBe(customBin);
      expect(fs.existsSync(customBin)).toBe(true);
      expect(fs.existsSync(path.join(customBin, 'rtb-cli.js'))).toBe(true);
      expect(fs.existsSync(path.join(customBin, 'VERSION'))).toBe(true);

      if (process.platform === 'win32') {
        expect(fs.existsSync(path.join(customBin, 'rtb.cmd'))).toBe(true);
        expect(fs.existsSync(path.join(customBin, 'rtb.ps1'))).toBe(true);
        expect(res.launcherPath).toBe(path.join(customBin, 'rtb.cmd'));
      } else {
        expect(fs.existsSync(path.join(customBin, 'rtb'))).toBe(true);
        expect(res.launcherPath).toBe(path.join(customBin, 'rtb'));
      }
    });
  });
});
