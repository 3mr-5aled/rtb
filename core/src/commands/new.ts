import type { Command } from 'commander';
import chalk from 'chalk';
import fs from 'node:fs';
import path from 'node:path';
import type { CliContext } from '../types/context.js';
import { outputError, outputJson } from '../utils/output.js';

export function toKebabCase(str: string): string {
  return str
    .toLowerCase()
    .replace(/[^a-z0-9\-]/g, '-')
    .replace(/-+/g, '-')
    .replace(/^-|-$/g, '');
}

export function registerNewCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('new [name]')
    .description('Scaffold a new project in the active workspace root')
    .option('-s, --stack <stack>', 'Project stack template (react, nextjs, node, python, generic)', 'generic')
    .action((name: string | undefined, options: { stack: string }) => {
      const ctx = getContext();

      if (!name) {
        console.log(`\n  ${chalk.yellow('Usage:')} rtb new <project-name> [--stack react|nextjs|node|python|generic]\n`);
        return;
      }

      if (!ctx.config || !ctx.config.projectRoots.active?.path) {
        outputError('Active project root not configured in rtb.config.json', 'CONFIG_MISSING', ctx.isJson);
        process.exitCode = 1;
        return;
      }

      const activeRoot = ctx.config.projectRoots.active.path;
      if (!fs.existsSync(activeRoot)) {
        fs.mkdirSync(activeRoot, { recursive: true });
      }

      const kebabName = toKebabCase(name);
      const targetDir = path.join(activeRoot, kebabName);

      if (fs.existsSync(targetDir)) {
        outputError(`Project '${kebabName}' already exists at: ${targetDir}`, 'ALREADY_EXISTS', ctx.isJson);
        process.exitCode = 1;
        return;
      }

      fs.mkdirSync(targetDir, { recursive: true });

      // 1. PROJECT.md
      let projectMdContent = `# ${name}\n\nCreated: ${new Date().toISOString().slice(0, 10)}\nStack: ${options.stack}\n`;
      if (ctx.config.templateDir) {
        const templatePath = path.join(ctx.config.templateDir, 'PROJECT.md');
        if (fs.existsSync(templatePath)) {
          try {
            const rawTemplate = fs.readFileSync(templatePath, 'utf-8');
            projectMdContent = rawTemplate
              .replace(/\[Project Name\]/g, name)
              .replace(/YYYY-MM-DD/g, new Date().toISOString().slice(0, 10))
              .replace(/\[e\.g\..*\]/g, options.stack);
          } catch {}
        }
      }
      fs.writeFileSync(path.join(targetDir, 'PROJECT.md'), projectMdContent, 'utf-8');

      // 2. .gitignore
      const gitignoreContent = [
        'node_modules/',
        '.next/',
        '.venv/',
        '__pycache__/',
        'dist/',
        'build/',
        'target/',
        '.env',
        '.env.local',
        '*.log',
      ].join('\n');
      fs.writeFileSync(path.join(targetDir, '.gitignore'), gitignoreContent, 'utf-8');

      // 3. README.md
      const monthYear = new Intl.DateTimeFormat('en-US', { month: 'long', year: 'numeric' }).format(new Date());
      const readmeContent = `# ${name}\n\nNew development project (${options.stack} stack).\n\nCreated: ${monthYear}\n`;
      fs.writeFileSync(path.join(targetDir, 'README.md'), readmeContent, 'utf-8');

      if (ctx.isJson) {
        outputJson({
          created: true,
          name: kebabName,
          path: targetDir,
          stack: options.stack,
        });
        return;
      }

      console.log(`\n  ${chalk.green('✓')} Project '${chalk.bold(kebabName)}' created in Active!`);
      console.log(`  Directory: ${chalk.gray(targetDir)}`);
      console.log(`  Run: ${chalk.cyan(`rtb goto ${kebabName}`)}\n`);
    });
}
