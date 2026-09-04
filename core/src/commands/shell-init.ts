import type { Command } from 'commander';
import path from 'node:path';
import chalk from 'chalk';
import type { CliContext } from '../types/context.js';
import { outputError } from '../utils/output.js';

export type SupportedShell = 'bash' | 'zsh' | 'fish' | 'pwsh' | 'powershell';

export function getShellScript(shell: string): string {
  const norm = shell.trim().toLowerCase();

  switch (norm) {
    case 'bash':
      return `# rtb shell integration for bash
# Add to ~/.bashrc:
#   eval "$(rtb shell-init bash)"

rtb() {
    if [ "$1" = "goto" ]; then
        if [ $# -le 1 ] || [ "$2" = "--help" ] || [ "$2" = "-h" ]; then
            command rtb "$@"
            return $?
        fi
        shift
        local target
        target="$(command rtb goto "$@" --print 2>/dev/null)"
        local ret=$?
        if [ $ret -eq 0 ] && [ -n "$target" ] && [ -d "$target" ]; then
            cd "$target" || return $ret
            command rtb goto "$@"
        else
            command rtb goto "$@"
        fi
    else
        command rtb "$@"
    fi
}
`;

    case 'zsh':
      return `# rtb shell integration for zsh
# Add to ~/.zshrc:
#   eval "$(rtb shell-init zsh)"

rtb() {
    if [ "$1" = "goto" ]; then
        if [ $# -le 1 ] || [ "$2" = "--help" ] || [ "$2" = "-h" ]; then
            command rtb "$@"
            return $?
        fi
        shift
        local target
        target="$(command rtb goto "$@" --print 2>/dev/null)"
        local ret=$?
        if [ $ret -eq 0 ] && [ -n "$target" ] && [ -d "$target" ]; then
            cd "$target" || return $ret
            command rtb goto "$@"
        else
            command rtb goto "$@"
        fi
    else
        command rtb "$@"
    fi
}
`;

    case 'fish':
      return `# rtb shell integration for fish
# Add to ~/.config/fish/config.fish:
#   rtb shell-init fish | source

function rtb
    if test (count $argv) -gt 0; and test $argv[1] = "goto"
        if test (count $argv) -le 1; or test $argv[2] = "--help"; or test $argv[2] = "-h"
            command rtb $argv
            return $status
        end
        set -l goto_args $argv[2..-1]
        set -l target (command rtb goto $goto_args --print 2>/dev/null)
        set -l ret $status
        if test $ret -eq 0; and test -n "$target"; and test -d "$target"
            cd "$target"
            command rtb goto $goto_args
        else
            command rtb goto $goto_args
        end
    else
        command rtb $argv
    end
end
`;

    case 'pwsh':
    case 'powershell':
    case 'posh':
      return `# rtb shell integration for PowerShell (pwsh / Windows PowerShell)
# Add to $PROFILE:
#   (& rtb shell-init pwsh | Out-String) | Invoke-Expression

function rtb {
    $rtbApp = (Get-Command -CommandType Application,ExternalScript -Name rtb -ErrorAction SilentlyContinue | Select-Object -First 1)
    $invokeTarget = if ($rtbApp) { $rtbApp.Source } else { 'rtb' }

    if ($args.Count -gt 0 -and $args[0] -eq 'goto') {
        if ($args.Count -le 1 -or $args[1] -in @('--help', '-h')) {
            & $invokeTarget @args
            return
        }
        $gotoArgs = $args | Select-Object -Skip 1
        $target = & $invokeTarget goto @gotoArgs --print 2>$null
        if ($LASTEXITCODE -eq 0 -and $target -and (Test-Path $target)) {
            Set-Location $target
            & $invokeTarget goto @gotoArgs
        } else {
            & $invokeTarget goto @gotoArgs
        }
    } else {
        & $invokeTarget @args
    }
}
`;

    default:
      throw new Error(`Unsupported shell: '${shell}'. Supported shells: bash, zsh, fish, pwsh`);
  }
}

export function detectCurrentShell(): string {
  const shellEnv = process.env.SHELL;
  if (shellEnv) {
    const base = path.basename(shellEnv).toLowerCase();
    if (base.includes('zsh')) return 'zsh';
    if (base.includes('bash')) return 'bash';
    if (base.includes('fish')) return 'fish';
  }

  if (process.env.PSModulePath || process.platform === 'win32') {
    return 'pwsh';
  }

  return 'bash';
}

export function registerShellInitCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('shell-init [shell]')
    .description('Emit shell wrapper function for directory switching (bash, zsh, fish, pwsh)')
    .action((shellName: string | undefined) => {
      const ctx = getContext();
      const targetShell = shellName || detectCurrentShell();

      try {
        const script = getShellScript(targetShell);
        process.stdout.write(script);
      } catch (err: any) {
        outputError(err.message, 'UNSUPPORTED_SHELL', ctx.isJson);
        process.exitCode = 1;
      }
    });
}
