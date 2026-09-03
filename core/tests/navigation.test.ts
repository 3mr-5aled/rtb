import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import path from 'node:path';
import fs from 'node:fs';
import os from 'node:os';
import { findProjectPathFuzzy } from '../src/navigation/fuzzy.js';
import type { RtbConfig } from '../types/config.js';

describe('Fuzzy Navigation Engine', () => {
  const tmpDir = path.join(os.tmpdir(), `rtb-fuzzy-test-${Date.now()}`);
  const activeDir = path.join(tmpDir, 'Active');

  beforeEach(() => {
    fs.mkdirSync(path.join(activeDir, 'rtb-command-tool'), { recursive: true });
    fs.mkdirSync(path.join(activeDir, 'rtb-extension'), { recursive: true });
    fs.mkdirSync(path.join(activeDir, 'another-project'), { recursive: true });
  });

  afterEach(() => {
    try {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    } catch {}
  });

  const config: RtbConfig = {
    version: '1.0.0',
    projectRoots: {
      active: { path: activeDir, label: 'Active', emoji: '📁' },
    },
  };

  it('should return score 100 on exact match', () => {
    const matches = findProjectPathFuzzy('rtb-command-tool', config);
    expect(matches.length).toBeGreaterThanOrEqual(1);
    expect(matches[0].name).toBe('rtb-command-tool');
    expect(matches[0].score).toBe(100);
  });

  it('should return score 75 on prefix match', () => {
    const matches = findProjectPathFuzzy('rtb', config);
    expect(matches.length).toBeGreaterThanOrEqual(2);
    expect(matches[0].score).toBe(75);
    expect(matches.map((m) => m.name)).toContain('rtb-command-tool');
    expect(matches.map((m) => m.name)).toContain('rtb-extension');
  });

  it('should return score 50 on substring match', () => {
    const matches = findProjectPathFuzzy('command', config);
    expect(matches.length).toBe(1);
    expect(matches[0].name).toBe('rtb-command-tool');
    expect(matches[0].score).toBe(50);
  });

  it('should return 0 matches for non-existent query', () => {
    const matches = findProjectPathFuzzy('completely_unknown_xyz', config);
    expect(matches.length).toBe(0);
  });
});
