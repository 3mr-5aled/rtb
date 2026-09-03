#!/usr/bin/env node
import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(__dirname, '..');

const versionPath = resolve(repoRoot, 'VERSION');
const version = readFileSync(versionPath, 'utf-8').trim().replace(/^v/, '');

if (!version) {
  console.error('Error: VERSION file is empty');
  process.exit(1);
}

console.log(`Synchronizing project version to: ${version}`);

function updateFile(relPath, replacer) {
  const fullPath = resolve(repoRoot, relPath);
  if (!existsSync(fullPath)) return;
  try {
    const original = readFileSync(fullPath, 'utf-8');
    const updated = replacer(original);
    if (original !== updated) {
      writeFileSync(fullPath, updated, 'utf-8');
      console.log(`  ✓ ${relPath} -> ${version}`);
    } else {
      console.log(`  - ${relPath} (already ${version})`);
    }
  } catch (err) {
    console.warn(`  ⚠ Could not update ${relPath}: ${err.message}`);
  }
}

// 1. core/package.json
updateFile('core/package.json', (content) => {
  const pkg = JSON.parse(content);
  pkg.version = version;
  return JSON.stringify(pkg, null, 2) + '\n';
});

// 2. core/package-lock.json
updateFile('core/package-lock.json', (content) => {
  const lock = JSON.parse(content);
  lock.version = version;
  if (lock.packages && lock.packages['']) {
    lock.packages[''].version = version;
  }
  return JSON.stringify(lock, null, 2) + '\n';
});

// 3. core/src/commands/version.ts
updateFile('core/src/commands/version.ts', (content) =>
  content.replace(/return\s+['"][0-9]+\.[0-9]+\.[0-9]+['"];/, `return '${version}';`)
);

// 4. cli/rtb.psd1
updateFile('cli/rtb.psd1', (content) =>
  content.replace(/ModuleVersion\s*=\s*['"][^'"]+['"]/, `ModuleVersion     = '${version}'`)
);

// 5. cli/rtb.psm1
updateFile('cli/rtb.psm1', (content) =>
  content.replace(/\$ver\s*=\s*['"][0-9]+\.[0-9]+\.[0-9]+['"]/, `$ver = '${version}'`)
);

// 6. cli/src/commands/upgrade.ps1
updateFile('cli/src/commands/upgrade.ps1', (content) =>
  content.replace(/\$currentVersion\s*=\s*['"][0-9]+\.[0-9]+\.[0-9]+['"]/, `$currentVersion = '${version}'`)
);

// 7. tui/Cargo.toml
updateFile('tui/Cargo.toml', (content) =>
  content.replace(/^version\s*=\s*"[^"]+"/m, `version = "${version}"`)
);

// 8. README.md
updateFile('README.md', (content) =>
  content.replace(/version-v[0-9]+\.[0-9]+\.[0-9]+-blue/g, `version-v${version}-blue`)
);

// 9. install.ps1
updateFile('install.ps1', (content) =>
  content.replace(/(Get-RtbInstallerVersion[\s\S]*?return\s+['"])[0-9]+\.[0-9]+\.[0-9]+(['"])/, `$1${version}$2`)
);

// 10. install.sh
updateFile('install.sh', (content) =>
  content.replace(/echo\s+['"][0-9]+\.[0-9]+\.[0-9]+['"]/, `echo "${version}"`)
);

console.log('Version synchronization complete.');
