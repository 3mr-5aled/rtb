import type { Command } from 'commander';
import chalk from 'chalk';
import type { CliContext } from '../types/context.js';
import { RTB_BRAND_ARABIC } from '../utils/output.js';
import { renderLogo } from '../utils/logo.js';

export function getCustomHelpMenu(options?: { quiet?: boolean; json?: boolean; isQuiet?: boolean; isJson?: boolean }): string {
  const c = chalk.cyan;
  const y = chalk.yellow;
  const w = chalk.white;
  const g = chalk.gray;
  const gold = chalk.hex('#FFD700');

  const isQuiet = Boolean(options?.quiet || options?.isQuiet);
  const isJson = Boolean(options?.json || options?.isJson);
  const logo = renderLogo({ quiet: isQuiet, json: isJson });

  const lines = [
    ...(logo ? ['', logo] : []),
    '',
    `  ${gold.bold('rtb')} (${gold(RTB_BRAND_ARABIC)}) - ${w('Repository & Tooling Base Developer Project Operations CLI')}`,
    '',
    `  ${y('SETUP & CONFIG')}`,
    `    rtb install [--force]                Interactive installation setup & CLI launcher`,
    `    rtb init [--force]                   Interactive workspace setup wizard`,
    `    rtb config                           Open rtb.config.json in default editor`,
    `    rtb doctor                           System health & dependency diagnostic`,
    `    rtb upgrade [--check] [--force]      Check for updates and self-upgrade`,
    `    rtb uninstall [--force]              Cleanly uninstall RTB from system`,
    `    rtb --version                        Display RTB version`,
    `    rtb --help                           Display this help menu`,
    '',
    `  ${y('QUICK INTERACTION')}`,
    `    rtb menu                             Interactive prompt menu (select & run)`,
    '',
    `  ${y('PROJECT OPERATIONS')}`,
    `    rtb run [project]                    Auto-detect and run dev/start script`,
    `    rtb build [project]                  Auto-detect and run build script`,
    `    rtb test [project]                   Auto-detect and run test suite`,
    `    rtb info [project] [--json]          Display deep project intelligence`,
    `    rtb deps [outdated] [project]        Audit declared project dependencies`,
    `    rtb workspace [project]              Inspect monorepo workspace packages`,
    '',
    `  ${y('AI AGENT LAUNCHERS')}`,
    `    rtb agy [project]                    Launch Google Antigravity CLI`,
    `    rtb claude|gemini|codex [project]    Launch Claude / Gemini / Codex CLI`,
    `    rtb cursor|windsurf|aider [project]  Launch Cursor / Windsurf / Aider`,
    `    rtb agent [project] [--agy|--claude] Launch targeted agent (or rtb agent --list)`,
    `    rtb goto <name> [--agy|--claude]     cd into project and launch agent CLI`,
    '',
    `  ${y('LIFECYCLE')}`,
    `    rtb new <name> [--stack nextjs]     Create new project in 01-Active`,
    `    rtb pause <name> [--prune]          Pause project (move to 04-Paused)`,
    `    rtb resume <name> [--install]        Resume paused project`,
    `    rtb deploy <name> [--prod|--staging] Deploy to production/staging`,
    `    rtb archive <name>                   Compress to .tar.gz backup`,
    `    rtb unarchive <name>                 Extract archive to 01-Active`,
    '',
    `  ${y('NAVIGATION')}`,
    `    rtb goto <name>                      cd into any project (TAB to search)`,
    `    rtb status [--json]                  Single-line project & git prompt segment`,
    `    rtb open <name>                      Open project folder in File Explorer`,
    `    rtb list [--active|--paused|--all]   List projects with status`,
    `    rtb shell-init [shell]               Emit shell wrapper & completion (bash, zsh, fish, pwsh)`,
    `    rtb completion [shell]               Emit standalone shell completion script`,
    '',
    `  ${y('MAINTENANCE & SAFETY')}`,
    `    rtb health                           Git repo health scan`,
    `    rtb clean [--commit] [--dry-run]     Prune inactive dependencies (dry-run default)`,
    `    rtb index                            Generate PROJECT-INDEX.md`,
    `    rtb guard                            D drive root guardrail`,
    `    rtb maintenance [--full]             Run all maintenance tasks`,
    '',
    `  ${y('BACKUP')}`,
    `    rtb backup                           Full config backup`,
    `    rtb env                              Backup all .env files`,
    '',
    `  ${y('UI')}`,
    `    rtb ui (or rtbtui)                   Launch interactive TUI operations center`,
    '',
    `  ${g('Tip: Run rtb menu for interactive launcher, or press TAB for auto-completion!')}`,
    `  ${g("Run 'rtb <command> --help' for details on a specific command.")}`,
    '',
  ];

  return lines.join('\n');
}

export function registerHelpCommand(program: Command, getContext?: () => CliContext): void {
  // Override the root helpInformation so rtb --help, rtb -h, and rtb help use the rich menu
  program.helpInformation = function () {
    const ctx = getContext ? getContext() : undefined;
    return getCustomHelpMenu(ctx);
  };

  program
    .command('help [command]')
    .description('Display this help menu')
    .action((subCmd?: string) => {
      const ctx = getContext ? getContext() : undefined;
      if (!subCmd) {
        console.log(getCustomHelpMenu(ctx));
        return;
      }
      const target = program.commands.find((c) => c.name() === subCmd || c.aliases().includes(subCmd));
      if (target) {
        target.outputHelp();
      } else {
        console.log(chalk.red(`Unknown command: ${subCmd}`));
        console.log(getCustomHelpMenu(ctx));
      }
    });
}

