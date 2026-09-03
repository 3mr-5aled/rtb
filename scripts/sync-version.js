#!/usr/bin/env node
import { readFileSync, writeFileSync } from 'node:fs';
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

// 1. core/package.json
const pkgPath = resolve(repoRoot, 'core', 'package.json');
try {
  const pkg = JSON.parse(readFileSync(pkgPath, 'utf-8'));
  pkg.version = version;
  writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + '\n', 'utf-8');
  console.log(`  ✓ core/package.json -> ${version}`);
} catch (err) {
  console.warn(`  ⚠ Could not update core/package.json: ${err.message}`);
}

// 2. cli/rtb.psd1
const psdPath = resolve(repoRoot, 'cli', 'rtb.psd1');
try {
  let psdContent = readFileSync(psdPath, 'utf-8');
  psdContent = psdContent.replace(/ModuleVersion\s*=\s*['"][^'"]+['"]/, `ModuleVersion     = '${version}'`);
  writeFileSync(psdPath, psdContent, 'utf-8');
  console.log(`  ✓ cli/rtb.psd1 -> ${version}`);
} catch (err) {
  console.warn(`  ⚠ Could not update cli/rtb.psd1: ${err.message}`);
}

// 3. tui/Cargo.toml
const cargoPath = resolve(repoRoot, 'tui', 'Cargo.toml');
try {
  let cargoContent = readFileSync(cargoPath, 'utf-8');
  cargoContent = cargoContent.replace(/version\s*=\s*"[^"]+"/, `version = "${version}"`);
  writeFileSync(cargoPath, cargoContent, 'utf-8');
  console.log(`  ✓ tui/Cargo.toml -> ${version}`);
} catch (err) {
  console.warn(`  ⚠ Could not update tui/Cargo.toml: ${err.message}`);
}

console.log('Version synchronization complete.');
