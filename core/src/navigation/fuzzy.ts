import fs from 'node:fs';
import path from 'node:path';
import type { RtbConfig } from '../types/config.js';
import type { FuzzyMatch } from '../types/project.js';

export function findProjectPathFuzzy(query: string, config: RtbConfig): FuzzyMatch[] {
  if (!config || !config.projectRoots) return [];

  const q = query.trim().toLowerCase();
  const results: FuzzyMatch[] = [];

  for (const [key, entry] of Object.entries(config.projectRoots)) {
    if (!entry.path || !fs.existsSync(entry.path)) continue;

    try {
      const items = fs.readdirSync(entry.path, { withFileTypes: true });
      for (const item of items) {
        if (!item.isDirectory()) continue;
        const name = item.name;
        const n = name.toLowerCase();
        const fullPath = path.join(entry.path, name);
        const fullPathLower = fullPath.toLowerCase();

        let score = 0;
        if (n === q) {
          score = 100;
        } else if (n.startsWith(q)) {
          score = 75;
        } else if (n.includes(q)) {
          score = 50;
        } else if (fullPathLower.includes(q)) {
          score = 25;
        }

        if (score > 0) {
          results.push({
            name,
            path: fullPath,
            status: entry.label || key,
            score,
          });
        }
      }
    } catch {}
  }

  // Sort descending by score, then alphabetically
  return results.sort((a, b) => {
    if (b.score !== a.score) return b.score - a.score;
    return a.name.localeCompare(b.name);
  });
}

export function resolveProjectTarget(
  projectName: string | undefined,
  config: RtbConfig | null
): { targetPath: string; targetName: string; status?: string } | null {
  if (!projectName) {
    const cwd = process.cwd();
    return { targetPath: cwd, targetName: path.basename(cwd) };
  }

  if (fs.existsSync(projectName)) {
    const resolved = path.resolve(projectName);
    return { targetPath: resolved, targetName: path.basename(resolved) };
  }

  if (config) {
    const matches = findProjectPathFuzzy(projectName, config);
    if (matches.length > 0) {
      return { targetPath: matches[0].path, targetName: matches[0].name, status: matches[0].status };
    }
  }

  return null;
}

