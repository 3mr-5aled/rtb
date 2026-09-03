import { defineConfig } from 'tsup';

export default defineConfig({
  entry: ['src/index.ts'],
  format: ['esm'],
  target: 'node18',
  platform: 'node',
  shims: true,
  clean: true,
  dts: false,
  sourcemap: true,
  minify: false,
  noExternal: [/(.*)/],
  banner: {
    js: `#!/usr/bin/env node\nimport { createRequire } from 'node:module';\nconst require = createRequire(import.meta.url);`,
  },
});
