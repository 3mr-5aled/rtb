import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { getStandardConfigDir } from '../config/loader.js';

export const EMBEDDED_LOGO = `\\e[38;2;255;215;0m⠀⠀⢸⣿⣿⣿⣿⣿⣿⣷⡄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
\\e[38;2;255;207;0m⠀⠀⢸⣿⣿⣿⣿⣿⣿⣿⣷⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⠀
\\e[38;2;255;199;0m⠀⠀⢸⣿⣿⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠀⠀⠀⠀⠀
\\e[38;2;255;191;0m⠀⠀⢸⣿⡏⢠⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡟
\\e[38;2;255;183;0m⠀ ⢸⣿⠃⣼⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿ ⣿ ⣿⣿⣿⣿⣿⣿⣿⣿⣿⠃⠀⠀
\\e[38;2;255;175;0m⠀⠀⢸⡟⢀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠃⠀⠀
\\e[38;2;255;167;0m⠀⠀⢸⡇⢸⣿⣿⣿⣿⣿⣿ ⣿⣿⣿⣿ ⣿⣿⣿ ⣿⣿⣿⣿ ⣿⣿⣿⡟⠀⠀⠀
\\e[38;2;255;159;0m⠀⠀⢸⠁⣿⣿⣿⣿⣿⣿⣿          ⣿⣿⣿ ⣿⣿⣿⡟⠀⠀⠀
\\e[38;2;255;151;0m⠀⠀⢸⢀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿ ⣿⣿⣿⡟⠀⠀⠀⠀
\\e[38;2;255;143;0m⠀⠀⠘⢸⣿⣿⣿⣿⣿⣿⣿⣿⣿ ⣿⣿⣿⣿⣿⣿⣿  ⣿⣿⣿⡿⠀⠀⠀⠀
\\e[38;2;255;135;0m⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠇⠀⠀⠀⠀
\\e[38;2;255;127;0m⠀ ⠈⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉   `;

/**
 * Multi-tier lookup for the RTB visual logo:
 * 1. Current working directory (logo.txt)
 * 2. Module dist/package root (../../logo.txt or ../logo.txt)
 * 3. User configuration directory (~/.config/rtb/logo.txt or %USERPROFILE%/.config/rtb/logo.txt)
 * 4. Embedded compile-time fallback constant
 */
export function getRawLogo(): string {
  // 1. Current working directory
  const localPath = path.resolve(process.cwd(), 'logo.txt');
  if (fs.existsSync(localPath)) {
    try {
      const content = fs.readFileSync(localPath, 'utf8');
      if (content.trim().length > 0) return content.replace(/^\uFEFF/, '');
    } catch {}
  }

  // 2. Module root
  try {
    const currentDir = path.dirname(fileURLToPath(import.meta.url));
    const moduleCandidates = [
      path.resolve(currentDir, '..', '..', 'logo.txt'),
      path.resolve(currentDir, '..', 'logo.txt'),
      path.resolve(currentDir, 'logo.txt'),
    ];
    for (const cand of moduleCandidates) {
      if (fs.existsSync(cand)) {
        const content = fs.readFileSync(cand, 'utf8');
        if (content.trim().length > 0) return content.replace(/^\uFEFF/, '');
      }
    }
  } catch {}

  // 3. User configuration directory
  try {
    const userLogoPath = path.join(getStandardConfigDir(), 'logo.txt');
    if (fs.existsSync(userLogoPath)) {
      const content = fs.readFileSync(userLogoPath, 'utf8');
      if (content.trim().length > 0) return content.replace(/^\uFEFF/, '');
    }
  } catch {}

  // 4. Embedded fallback
  return EMBEDDED_LOGO.replace(/^\uFEFF/, '');
}

export interface LogoRenderOptions {
  color?: boolean;
  quiet?: boolean;
  json?: boolean;
}

/**
 * Normalizes and formats the logo with ANSI truecolor escapes.
 * Cleanly strips escapes if color is disabled, or suppresses if quiet/json.
 */
export function renderLogo(options?: LogoRenderOptions): string {
  if (options?.quiet || options?.json) {
    return '';
  }

  const raw = getRawLogo();
  const normalized = raw
    .replace(/\\e/g, '\x1b')
    .replace(/\\033/g, '\x1b')
    .replace(/\\x1b/g, '\x1b');

  const useColor = options?.color ?? (!process.env.NO_COLOR && process.env.TERM !== 'dumb');

  const lines = normalized.split(/\r?\n/).map((line) => {
    if (!useColor) {
      // Strip ANSI escape codes
      return line.replace(/\x1b\[[0-9;]*[a-zA-Z]/g, '');
    }
    // Append reset code if line has colors
    return line.includes('\x1b') ? `${line}\x1b[0m` : line;
  });

  return lines.join('\n');
}

export function getLogoLines(options?: LogoRenderOptions): string[] {
  const rendered = renderLogo(options);
  if (!rendered) return [];
  return rendered.split('\n');
}

export const getLogo = renderLogo;
