import { describe, it, expect } from 'vitest';
import fs from 'node:fs';
import path from 'node:path';
import { execSync } from 'node:child_process';

describe('npm registry packaging contract', () => {
  const coreDir = path.resolve(__dirname, '..');
  const pkgPath = path.join(coreDir, 'package.json');

  it('declares comprehensive registry distribution metadata in package.json', () => {
    const pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf8'));

    expect(pkg.name).toBe('@3mr5aled/rtb');
    expect(pkg.bin).toEqual({ rtb: './dist/index.js' });
    expect(pkg.files).toContain('dist');
    expect(pkg.engines?.node).toBe('>=18.0.0');
    expect(pkg.repository?.url).toContain('3mr-5aled/rtb');
    expect(pkg.homepage).toBe('https://github.com/3mr-5aled/rtb#readme');
    expect(pkg.bugs?.url).toBe('https://github.com/3mr-5aled/rtb/issues');
  });

  it('packs only dist/ and package metadata via npm pack --dry-run', () => {
    const rawOutput = execSync('npm pack --dry-run --json', {
      cwd: coreDir,
      encoding: 'utf8',
      stdio: ['pipe', 'pipe', 'ignore'],
    });

    const parsed = JSON.parse(rawOutput);
    expect(Array.isArray(parsed)).toBe(true);
    expect(parsed.length).toBeGreaterThan(0);

    const packDetails = parsed[0];
    const filenames: string[] = packDetails.files.map((f: { path: string }) => f.path);

    for (const file of filenames) {
      const isAllowed =
        file.startsWith('dist/') ||
        file === 'package.json' ||
        file.toLowerCase() === 'readme.md' ||
        file.toLowerCase() === 'license';
      expect(isAllowed, `Unexpected file included in npm pack: ${file}`).toBe(true);
    }

    expect(filenames.some((f) => f.startsWith('src/'))).toBe(false);
    expect(filenames.some((f) => f.startsWith('tests/'))).toBe(false);
  }, 20000);

  it('runs cleanly via node when invoked from declared binary entrypoint', () => {
    const binPath = path.join(coreDir, 'dist', 'index.js');
    if (!fs.existsSync(binPath)) {
      execSync('npm run build', { cwd: coreDir, stdio: 'ignore' });
    }

    const versionOutput = execSync(`node "${binPath}" --version`, {
      encoding: 'utf8',
    }).trim();

    expect(versionOutput).toMatch(/^\d+\.\d+\.\d+/);
  });
});
