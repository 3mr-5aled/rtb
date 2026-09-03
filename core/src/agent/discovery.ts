import fs from 'node:fs';
import path from 'node:path';

export interface AgentDefinition {
  name: string;
  command: string;
  installed: boolean;
  path?: string;
}

export const KNOWN_AGENTS = [
  { name: 'Google Antigravity', command: 'agy' },
  { name: 'Claude Code', command: 'claude' },
  { name: 'Gemini CLI', command: 'gemini' },
  { name: 'Codex CLI', command: 'codex' },
  { name: 'Cursor', command: 'cursor' },
  { name: 'Windsurf', command: 'windsurf' },
  { name: 'Aider', command: 'aider' },
  { name: 'OpenHands', command: 'openhands' },
];

export function findExecutableInPath(command: string): string | null {
  const envPath = process.env.PATH || '';
  const pathDirs = envPath.split(path.delimiter).filter(Boolean);
  const isWindows = process.platform === 'win32';

  const extensions = isWindows
    ? (process.env.PATHEXT || '.COM;.EXE;.BAT;.CMD;.PS1').split(';').map((e) => e.toLowerCase())
    : [''];

  for (const dir of pathDirs) {
    // Exact match (on unix or if command already has extension)
    const exact = path.join(dir, command);
    if (fs.existsSync(exact)) {
      try {
        if (!fs.statSync(exact).isDirectory()) return exact;
      } catch {}
    }

    if (isWindows) {
      for (const ext of extensions) {
        const full = path.join(dir, `${command}${ext}`);
        if (fs.existsSync(full)) {
          try {
            if (!fs.statSync(full).isDirectory()) return full;
          } catch {}
        }
      }
    }
  }

  return null;
}

export function getInstalledAgents(): AgentDefinition[] {
  return KNOWN_AGENTS.map((agent) => {
    const exePath = findExecutableInPath(agent.command);
    return {
      name: agent.name,
      command: agent.command,
      installed: exePath !== null,
      path: exePath ?? undefined,
    };
  });
}
