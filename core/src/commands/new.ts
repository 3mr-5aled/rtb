import type { Command } from 'commander';
import chalk from 'chalk';
import type { CliContext } from '../types/context.js';
import { ProjectLifecycle, toKebabCase } from '../services/lifecycle.js';
import { ConfigMissingError } from '../errors.js';
import { outputJson } from '../utils/output.js';

export { toKebabCase } from '../services/lifecycle.js';

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
        throw new ConfigMissingError('Active project root not configured in rtb.config.json');
      }

      const lifecycle = new ProjectLifecycle();
      const result = lifecycle.create({
        name,
        stack: options.stack,
        activeRoot: ctx.config.projectRoots.active.path,
        templateDir: ctx.config.templateDir,
      });

      if (ctx.isJson) {
        outputJson({
          created: true,
          name: result.name,
          path: result.path,
          stack: result.stack,
        });
        return;
      }

      console.log(`\n  ${chalk.green('✓')} Project '${chalk.bold(result.name)}' created in Active!`);
      console.log(`  Directory: ${chalk.gray(result.path)}`);
      console.log(`  Run: ${chalk.cyan(`rtb goto ${result.name}`)}\n`);
    });
}
