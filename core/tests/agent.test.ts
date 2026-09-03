import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import path from 'node:path';
import fs from 'node:fs';
import os from 'node:os';
import { getInstalledAgents, findExecutableInPath } from '../src/agent/discovery.js';
import { generateAgentContextFile } from '../src/agent/context.js';
import { createCli } from '../src/cli.js';

describe('Agent Orchestrator & Context Generator', () => {
  const tmpDir = path.join(os.tmpdir(), `rtb-agent-test-${Date.now()}`);
  const configFile = path.join(tmpDir, 'rtb.config.json');

  beforeEach(() => {
    fs.mkdirSync(tmpDir, { recursive: true });
    fs.writeFileSync(
      configFile,
      JSON.stringify({
        version: '1.0.0',
        projectRoots: {
          active: { path: tmpDir, label: 'Active', emoji: '📁' },
        },
      })
    );
  });

  afterEach(() => {
    try {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    } catch {}
  });

  it('getInstalledAgents should return list of known agents with installed boolean', () => {
    const agents = getInstalledAgents();
    expect(agents.length).toBeGreaterThanOrEqual(8);
    const agy = agents.find((a) => a.command === 'agy');
    expect(agy).toBeDefined();
    expect(typeof agy?.installed).toBe('boolean');
  });

  it('findExecutableInPath should resolve common system tools', () => {
    // node or npm should always be in PATH
    const nodeExe = findExecutableInPath('node');
    expect(nodeExe).not.toBeNull();
  });

  it('generateAgentContextFile should generate .rtb_context.md with project information', () => {
    // Scaffold sample project
    fs.writeFileSync(
      path.join(tmpDir, 'package.json'),
      JSON.stringify({
        name: 'test-app',
        dependencies: { react: '^19.0.0', 'react-dom': '^19.0.0' },
        devDependencies: { typescript: '^5.0.0' },
      })
    );
    fs.writeFileSync(path.join(tmpDir, 'README.md'), '# Test App\nSample README content');

    const contextPath = generateAgentContextFile(tmpDir);
    expect(fs.existsSync(contextPath)).toBe(true);

    const content = fs.readFileSync(contextPath, 'utf-8');
    expect(content).toContain('# RTB Agent Workspace Context:');
    expect(content).toContain('Sample README content');
    expect(content).toContain('**package.json deps:** react, react-dom');
    expect(content).toContain('**devDependencies:** typescript');
  });

  it('rtb agent --list should execute and list agents without throwing', async () => {
    const cli = createCli();
    let stdoutData = '';
    const origLog = console.log;
    console.log = (...args: any[]) => {
      stdoutData += args.join(' ') + '\n';
    };

    try {
      await cli.parseAsync(['node', 'rtb', '--config', configFile, 'agent', '--list']);
      expect(stdoutData).toContain('Google Antigravity');
      expect(stdoutData).toContain('Claude Code');
    } finally {
      console.log = origLog;
    }
  });

  it('rtb agent --no-launch should create .rtb_context.md without spawning', async () => {
    const cli = createCli();
    await cli.parseAsync(['node', 'rtb', '--config', configFile, 'agent', tmpDir, '--no-launch']);

    const ctxFile = path.join(tmpDir, '.rtb_context.md');
    expect(fs.existsSync(ctxFile)).toBe(true);
  });
});
