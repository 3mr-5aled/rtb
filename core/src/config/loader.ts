import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import type { RtbConfig } from '../types/config.js';

export interface ConfigResolution {
  config: RtbConfig | null;
  configPath: string;
  isConfigured: boolean;
  source: 'custom' | 'user' | 'local' | 'none';
}

/**
 * Resolves standard user config directory: ~/.config/rtb
 * On Windows: %USERPROFILE%/.config/rtb
 * On macOS / Linux: ~/.config/rtb (or $XDG_CONFIG_HOME/rtb)
 */
export function getStandardConfigDir(): string {
  const xdg = process.env.XDG_CONFIG_HOME;
  if (xdg && xdg.trim().length > 0) {
    return path.join(xdg, 'rtb');
  }
  return path.join(os.homedir(), '.config', 'rtb');
}

export function getStandardConfigPath(): string {
  return path.join(getStandardConfigDir(), 'rtb.config.json');
}

export function resolveConfigPath(customPath?: string): { path: string; source: 'custom' | 'user' | 'local' | 'none' } {
  // 1. Explicit argument or environment variable
  const explicit = customPath || process.env.RTB_CONFIG;
  if (explicit) {
    const resolved = path.resolve(explicit);
    if (fs.existsSync(resolved)) {
      return { path: resolved, source: 'custom' };
    }
    return { path: resolved, source: 'none' };
  }

  // 2. Standard ~/.config/rtb/rtb.config.json
  const standard = getStandardConfigPath();
  if (fs.existsSync(standard)) {
    return { path: standard, source: 'user' };
  }

  // 3. Local repository fallback (./config/rtb.config.json or ../config/rtb.config.json)
  const localCandidates = [
    path.resolve(process.cwd(), 'config', 'rtb.config.json'),
    path.resolve(process.cwd(), '..', 'config', 'rtb.config.json'),
  ];
  for (const candidate of localCandidates) {
    if (fs.existsSync(candidate)) {
      return { path: candidate, source: 'local' };
    }
  }

  return { path: standard, source: 'none' };
}

export function loadConfig(customPath?: string): ConfigResolution {
  const resolution = resolveConfigPath(customPath);
  if (resolution.source === 'none' || !fs.existsSync(resolution.path)) {
    return {
      config: null,
      configPath: resolution.path,
      isConfigured: false,
      source: 'none',
    };
  }

  try {
    const raw = fs.readFileSync(resolution.path, 'utf-8');
    const parsed = JSON.parse(raw) as RtbConfig;
    const isConfigured = Boolean(
      parsed &&
      parsed.projectRoots &&
      parsed.projectRoots.active &&
      typeof parsed.projectRoots.active.path === 'string' &&
      parsed.projectRoots.active.path.trim().length > 0
    );

    return {
      config: parsed,
      configPath: resolution.path,
      isConfigured,
      source: resolution.source,
    };
  } catch {
    return {
      config: null,
      configPath: resolution.path,
      isConfigured: false,
      source: resolution.source,
    };
  }
}
