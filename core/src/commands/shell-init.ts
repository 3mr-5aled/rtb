import type { Command } from 'commander';
import path from 'node:path';
import chalk from 'chalk';
import type { CliContext } from '../types/context.js';
import { outputError } from '../utils/output.js';

import { getCompletionScript } from './completion.js';

export type SupportedShell = 'bash' | 'zsh' | 'fish' | 'pwsh' | 'powershell';

export function getShellScript(shell: string): string {
  const norm = shell.trim().toLowerCase();

  let wrapper = '';
  switch (norm) {
    case 'bash':
      wrapper = `# rtb shell integration for bash
# Add to ~/.bashrc:
#   eval "$(rtb shell-init bash)"

rtb() {
    local cd_file
    cd_file="$(mktemp -t rtb_cd.XXXXXX 2>/dev/null || mktemp 2>/dev/null || echo "/tmp/rtb_cd_$$")"
    export RTB_CD_FILE="$cd_file"
    if [ "$1" = "goto" ]; then
        if [ $# -le 1 ] || [ "$2" = "--help" ] || [ "$2" = "-h" ]; then
            rm -f "$cd_file" 2>/dev/null
            unset RTB_CD_FILE
            command rtb "$@"
            return $?
        fi
        shift
        command rtb goto "$@"
        local ret=$?
        local target=""
        if [ -f "$cd_file" ]; then
            target="$(cat "$cd_file" 2>/dev/null)"
            rm -f "$cd_file" 2>/dev/null
        fi
        unset RTB_CD_FILE
        if [ -z "$target" ]; then
            target="$(command rtb goto "$@" --print 2>/dev/null)"
        fi
        if [ -n "$target" ] && [ -d "$target" ]; then
            cd "$target" || return $ret
        fi
        return $ret
    else
        command rtb "$@"
        local ret=$?
        if [ -f "$cd_file" ]; then
            local target
            target="$(cat "$cd_file" 2>/dev/null)"
            rm -f "$cd_file" 2>/dev/null
            unset RTB_CD_FILE
            if [ -n "$target" ] && [ -d "$target" ]; then
                cd "$target" || return $ret
            fi
        else
            unset RTB_CD_FILE
        fi
        return $ret
    fi
}

goto() {
    rtb goto "$@"
}
`;
      break;

    case 'zsh':
      wrapper = `# rtb shell integration for zsh
# Add to ~/.zshrc:
#   eval "$(rtb shell-init zsh)"

rtb() {
    local cd_file
    cd_file="$(mktemp -t rtb_cd.XXXXXX 2>/dev/null || mktemp 2>/dev/null || echo "/tmp/rtb_cd_$$")"
    export RTB_CD_FILE="$cd_file"
    if [ "$1" = "goto" ]; then
        if [ $# -le 1 ] || [ "$2" = "--help" ] || [ "$2" = "-h" ]; then
            rm -f "$cd_file" 2>/dev/null
            unset RTB_CD_FILE
            command rtb "$@"
            return $?
        fi
        shift
        command rtb goto "$@"
        local ret=$?
        local target=""
        if [ -f "$cd_file" ]; then
            target="$(cat "$cd_file" 2>/dev/null)"
            rm -f "$cd_file" 2>/dev/null
        fi
        unset RTB_CD_FILE
        if [ -z "$target" ]; then
            target="$(command rtb goto "$@" --print 2>/dev/null)"
        fi
        if [ -n "$target" ] && [ -d "$target" ]; then
            cd "$target" || return $ret
        fi
        return $ret
    else
        command rtb "$@"
        local ret=$?
        if [ -f "$cd_file" ]; then
            local target
            target="$(cat "$cd_file" 2>/dev/null)"
            rm -f "$cd_file" 2>/dev/null
            unset RTB_CD_FILE
            if [ -n "$target" ] && [ -d "$target" ]; then
                cd "$target" || return $ret
            fi
        else
            unset RTB_CD_FILE
        fi
        return $ret
    fi
}

goto() {
    rtb goto "$@"
}
`;
      break;

    case 'fish':
      wrapper = `# rtb shell integration for fish
# Add to ~/.config/fish/config.fish:
#   rtb shell-init fish | source

function rtb
    set -l cd_file (mktemp -t rtb_cd.XXXXXX 2>/dev/null; or mktemp 2>/dev/null; or echo "/tmp/rtb_cd_$fish_pid")
    set -gx RTB_CD_FILE $cd_file
    if test (count $argv) -gt 0; and test $argv[1] = "goto"
        if test (count $argv) -le 1; or test $argv[2] = "--help"; or test $argv[2] = "-h"
            rm -f $cd_file 2>/dev/null
            set -e RTB_CD_FILE
            command rtb $argv
            return $status
        end
        set -l goto_args $argv[2..-1]
        command rtb goto $goto_args
        set -l ret $status
        set -l target ""
        if test -f $cd_file
            set target (cat $cd_file 2>/dev/null)
            rm -f $cd_file 2>/dev/null
        end
        set -e RTB_CD_FILE
        if test -z "$target"
            set target (command rtb goto $goto_args --print 2>/dev/null)
        end
        if test -n "$target"; and test -d "$target"
            cd "$target"
        end
        return $ret
    else
        command rtb $argv
        set -l ret $status
        if test -f $cd_file
            set -l target (cat $cd_file 2>/dev/null)
            rm -f $cd_file 2>/dev/null
            set -e RTB_CD_FILE
            if test -n "$target"; and test -d "$target"
                cd "$target"
            end
        else
            set -e RTB_CD_FILE
        end
        return $ret
    end
end

function goto
    rtb goto $argv
end
`;
      break;

    case 'pwsh':
    case 'powershell':
    case 'posh':
      wrapper = `# rtb shell integration for PowerShell (pwsh / Windows PowerShell)
# Add to $PROFILE:
#   (& rtb shell-init pwsh | Out-String) | Invoke-Expression

function rtb {
    $rtbApp = (Get-Command -CommandType Application,ExternalScript -Name rtb.cmd, rtb.ps1, rtb.exe, rtb -ErrorAction SilentlyContinue | Where-Object { $_.Source -notmatch '\\.js$' } | Select-Object -First 1)
    $invokeTarget = if ($rtbApp) { $rtbApp.Source } else { 'rtb' }

    $cdFile = [System.IO.Path]::GetTempFileName()
    $env:RTB_CD_FILE = $cdFile
    try {
        if ($args.Count -gt 0 -and $args[0] -eq 'goto') {
            if ($args.Count -le 1 -or $args[1] -in @('--help', '-h')) {
                & $invokeTarget @args
                return
            }
            $gotoArgs = @($args | Select-Object -Skip 1)
            & $invokeTarget goto @gotoArgs
            $target = if (Test-Path -LiteralPath $cdFile) { (Get-Content -LiteralPath $cdFile -Raw -ErrorAction SilentlyContinue) } else { $null }
            if ($target) { $target = $target.Trim() }
            if (-not $target) {
                $target = (& $invokeTarget goto @gotoArgs --print 2>$null | Out-String).Trim()
            }
            if ($target -and (Test-Path -LiteralPath $target -PathType Container)) {
                Set-Location -LiteralPath $target
            }
        } else {
            & $invokeTarget @args
            $target = if (Test-Path -LiteralPath $cdFile) { (Get-Content -LiteralPath $cdFile -Raw -ErrorAction SilentlyContinue) } else { $null }
            if ($target) { $target = $target.Trim() }
            if ($target -and (Test-Path -LiteralPath $target -PathType Container)) {
                Set-Location -LiteralPath $target
            }
        }
    } finally {
        Remove-Item -LiteralPath $cdFile -Force -ErrorAction SilentlyContinue
        Remove-Item Env:RTB_CD_FILE -ErrorAction SilentlyContinue
    }
}

function goto {
    rtb goto @args
}
`;
      break;

    default:
      throw new Error(`Unsupported shell: '${shell}'. Supported shells: bash, zsh, fish, pwsh`);
  }

  const completion = getCompletionScript(norm);
  return `${wrapper}\n${completion}`;
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
