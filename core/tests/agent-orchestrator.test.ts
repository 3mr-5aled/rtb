import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import path from 'node:path';
import fs from 'node:fs';
import os from 'node:os';
import { AgentOrchestrator } from '../src/services/agent.js';
import { createCli } from '../src/cli.js';
import type { RtbConfig } from '../types/config.js';

describe('AgentOrchestrator domain service and goto integration', () => {
  let tmpDir: string;
  let activeDir: string;
  let sampleProj: string;
  let configFile: string;
  let config: RtbConfig;

  beforeEach(() => {
    tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'rtb-agent-orch-test-'));
    activeDir = path.join(tmpDir, '01-Active');
    sampleProj = path.join(activeDir, 'my-orch-app');

    fs.mkdirSync(sampleProj, { recursive: true });
    fs.writeFileSync(
      path.join(sampleProj, 'package.json'),
      JSON.stringify({ name: 'my-orch-app', scripts: { dev: 'vite' } })
    );

    configFile = path.join(tmpDir, 'rtb.config.json');
    config = {
      version: '1.0.0',
      projectRoots: {
        active: { path: activeDir, label: 'Active', emoji: '🚀' },
      },
    };
    fs.writeFileSync(configFile, JSON.stringify(config, null, 2));
  });

  afterEach(() => {
    try {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    } catch {}
    vi.restoreAllMocks();
  });

  it('orchestrator generates .rtb_context.md without spawning when launch is false', async () => {
    const orchestrator = new AgentOrchestrator();
    const result = await orchestrator.orchestrate({
      projectPath: sampleProj,
      agent: 'agy',
      launch: false,
    });

    expect(result.launched).toBe(false);
    expect(result.projectName).toBe('my-orch-app');
    expect(fs.existsSync(result.contextPath)).toBe(true);

    const contextContent = fs.readFileSync(result.contextPath, 'utf-8');
    expect(contextContent).toContain('my-orch-app');
  });

  it('rtb goto with --agy and --no-launch triggers AgentOrchestrator and creates .rtb_context.md', async () => {
    const cli = createCli();
    await cli.parseAsync([
      'node',
      'rtb',
      'goto',
      'my-orch-app',
      '--agy',
      '--no-launch',
      '--config',
      configFile,
    ]);

    const expectedCtx = path.join(sampleProj, '.rtb_context.md');
    expect(fs.existsSync(expectedCtx)).toBe(true);
  });
});
