import chalk from 'chalk';
import type { CliContext } from '../types/context.js';
import { RTB_VERSION } from '../commands/version.js';
import { RTB_BRAND_ARABIC } from './output.js';
import { renderLogo } from './logo.js';
import { detectWorkspaceStatus } from '../commands/status.js';

/**
 * Renders the context-aware HeroBanner greeting when rtb is invoked
 * without arguments or when displaying the primary help interface.
 */
export function getHeroBanner(ctx: CliContext, cwd: string = process.cwd()): string {
  if (ctx.isJson || ctx.isQuiet) return '';

  const c = chalk.cyan;
  const y = chalk.yellow;
  const g = chalk.gray;
  const w = chalk.white;
  const gold = chalk.hex('#FFD700');

  const lines: string[] = [];
  lines.push('');

  // 1. Logo
  const logo = renderLogo({ quiet: ctx.isQuiet, json: ctx.isJson });
  if (logo) {
    lines.push(logo);
    lines.push('');
  }

  // 2. Title & Version Banner
  lines.push(`  ${gold.bold('rtb')} (${gold(RTB_BRAND_ARABIC)}) ${g(`v${RTB_VERSION}`)} — ${w.bold('Repository & Tooling Base Cockpit')}`);
  lines.push(`  ${g('Unified developer workspace manager, AI launcher & project telemetry')}`);
  lines.push('');

  // 3. Workspace Context
  const activeRoot = ctx.config?.projectRoots?.active?.path;
  if (activeRoot) {
    lines.push(`  ${g('Workspace:')} ${c(activeRoot)}`);
  } else {
    lines.push(`  ${g('Workspace:')} ${y('Not configured yet')} ${g("(run 'rtb init' to get started)")}`);
  }

  // 4. Local Project Intelligence
  try {
    const status = detectWorkspaceStatus(process.cwd(), ctx);
    if (status.project) {
      const proj = status.project;
      const stackStr = proj.stack.length > 0 ? proj.stack.join(', ') : 'generic';
      const gitStr = proj.git
        ? ` | ${g('Git:')} ${chalk.magenta(proj.git.branch)} (${proj.git.uncommitted > 0 ? y(`${proj.git.uncommitted} uncommitted`) : chalk.green('clean')})`
        : '';
      lines.push(`  ${g('Current Project:')} ${w.bold(proj.name)} ${g(`[${proj.rootCategory}]`)} | ${g('Stack:')} ${y(stackStr)}${gitStr}`);
    } else if (status.inWorkspace) {
      lines.push(`  ${g('Current Location:')} ${g('Inside workspace root (outside individual project)')}`);
    }
  } catch {}

  lines.push('');
  lines.push(`  ${y.bold('QUICK ACTIONS')}`);
  lines.push(`    ${c('rtb menu')}                       Launch interactive prompt launcher ${g('(recommended)')}`);
  lines.push(`    ${c('rtb run')} [project]              Auto-detect and run dev/start script`);
  lines.push(`    ${c('rtb goto')} <name>                Switch directory into project ${g('(TAB to search)')}`);
  lines.push(`    ${c('rtb list')}                       List all managed projects with status`);
  lines.push(`    ${c('rtb health')}                     Git repository health scan across roots`);
  lines.push(`    ${c('rtb init')}                       Interactive workspace setup wizard`);
  lines.push(`    ${c('rtb ui')}                         Launch Ratatui interactive operations center`);
  lines.push(`    ${c('rtb help')}                       Full directory of commands and options`);
  lines.push('');
  lines.push(`  ${g('Tip: Run ')}${c('rtb menu')}${g(' for arrow-key navigation, or press ')}${w.bold('TAB')}${g(' for autocompletion!')}`);
  lines.push(`  ${g('Run ')}${c('rtb <command> --help')}${g(' for detailed flags on any command.')}`);
  lines.push('');

  return lines.join('\n');
}
