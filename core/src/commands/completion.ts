import type { Command } from 'commander';
import fs from 'node:fs';
import path from 'node:path';
import type { CliContext } from '../types/context.js';
import type { RtbConfig } from '../types/config.js';
import { outputError } from '../utils/output.js';
import { loadConfig } from '../config/loader.js';

export const ALL_SUBCOMMANDS = [
  'init',
  'config',
  'doctor',
  'upgrade',
  'uninstall',
  'run',
  'build',
  'test',
  'info',
  'deps',
  'workspace',
  'agent',
  'goto',
  'open',
  'new',
  'pause',
  'resume',
  'deploy',
  'archive',
  'unarchive',
  'list',
  'health',
  'clean',
  'index',
  'guard',
  'maintenance',
  'backup',
  'env',
  'ui',
  'menu',
  'help',
  'version',
  'status',
  'shell-init',
  'completion',
  'agy',
  'claude',
  'gemini',
  'codex',
  'cursor',
  'windsurf',
  'aider',
  'openhands',
];

export function getProjectNames(config: RtbConfig | null, category?: string): string[] {
  if (!config || !config.projectRoots) return [];
  const names = new Set<string>();
  const targetCategory = category ? category.trim().toLowerCase() : null;

  for (const [key, entry] of Object.entries(config.projectRoots)) {
    if (targetCategory && targetCategory !== 'all') {
      const normKey = key.toLowerCase();
      const normLabel = (entry && typeof entry === 'object' && entry.label ? entry.label : '').toLowerCase();
      if (
        normKey !== targetCategory &&
        !normKey.startsWith(targetCategory) &&
        !normKey.includes(targetCategory) &&
        !normLabel.includes(targetCategory)
      ) {
        continue;
      }
    }

    const p = typeof entry === 'string' ? entry : entry?.path;
    if (!p || !fs.existsSync(p)) continue;

    try {
      const items = fs.readdirSync(p, { withFileTypes: true });
      for (const item of items) {
        if (item.isDirectory() && !item.name.startsWith('.')) {
          names.add(item.name);
        }
      }
    } catch {}
  }

  return Array.from(names).sort();
}

export function getArchiveNames(config: RtbConfig | null): string[] {
  const archives = new Set<string>();
  const candidates: string[] = [];

  if (config?.backupRoot) {
    candidates.push(config.backupRoot);
    candidates.push(path.join(config.backupRoot, 'project-snapshots'));
  }
  candidates.push('D:\\08-Backup\\project-snapshots');
  candidates.push('D:\\08-Backups\\projects');

  for (const dir of candidates) {
    if (!fs.existsSync(dir)) continue;
    try {
      const items = fs.readdirSync(dir);
      for (const item of items) {
        if (item.endsWith('.tar.gz')) {
          archives.add(item.replace(/\.tar\.gz$/, ''));
        }
      }
    } catch {}
  }

  return Array.from(archives).sort();
}

export function getCompletionScript(shell: string): string {
  const norm = shell.trim().toLowerCase();

  switch (norm) {
    case 'bash':
      return `# Bash completion for rtb
_rtb_completions() {
    local cur prev words cword
    if declare -F _init_completion >/dev/null 2>&1; then
        _init_completion || return
    else
        cur="\${COMP_WORDS[COMP_CWORD]}"
        prev="\${COMP_WORDS[COMP_CWORD-1]}"
        words=("\${COMP_WORDS[@]}")
        cword=$COMP_CWORD
    fi

    local cmd="\${words[0]}"
    if [ "\${cmd}" = "dev" ]; then
        if [[ "\${cur}" == --* ]]; then
            COMPREPLY=( $(compgen -W "--agy --claude --gemini --cursor --windsurf --aider --openhands --print" -- "\${cur}") )
        else
            local projs
            projs="$(command rtb __complete projects 2>/dev/null)"
            COMPREPLY=( $(compgen -W "\${projs}" -- "\${cur}") )
        fi
        return 0
    fi

    local sub=""
    if [ $cword -le 1 ]; then
        local cmds
        cmds="$(command rtb __complete commands 2>/dev/null)"
        COMPREPLY=( $(compgen -W "\${cmds}" -- "\${cur}") )
        return 0
    fi

    sub="\${words[1]}"
    case "\${sub}" in
        goto|open|run|build|test|info|workspace|deps|agent|agy|claude|gemini|codex|cursor|windsurf|aider|openhands)
            if [[ "\${cur}" == --* ]]; then
                case "\${sub}" in
                    goto) COMPREPLY=( $(compgen -W "--agy --claude --gemini --cursor --windsurf --aider --openhands --print" -- "\${cur}") ) ;;
                    agent) COMPREPLY=( $(compgen -W "--list --agy --claude --gemini --codex --cursor --windsurf --aider --openhands --no-launch" -- "\${cur}") ) ;;
                    info) COMPREPLY=( $(compgen -W "--json" -- "\${cur}") ) ;;
                    *) COMPREPLY=( $(compgen -W "--help" -- "\${cur}") ) ;;
                esac
            else
                local projs
                projs="$(command rtb __complete projects 2>/dev/null)"
                COMPREPLY=( $(compgen -W "\${projs}" -- "\${cur}") )
            fi
            ;;
        pause)
            if [[ "\${cur}" == --* ]]; then
                COMPREPLY=( $(compgen -W "--prune" -- "\${cur}") )
            else
                local projs
                projs="$(command rtb __complete projects active 2>/dev/null)"
                COMPREPLY=( $(compgen -W "\${projs}" -- "\${cur}") )
            fi
            ;;
        resume)
            if [[ "\${cur}" == --* ]]; then
                COMPREPLY=( $(compgen -W "--install" -- "\${cur}") )
            else
                local projs
                projs="$(command rtb __complete projects paused 2>/dev/null)"
                COMPREPLY=( $(compgen -W "\${projs}" -- "\${cur}") )
            fi
            ;;
        archive)
            local projs
            projs="$(command rtb __complete projects 2>/dev/null)"
            COMPREPLY=( $(compgen -W "\${projs}" -- "\${cur}") )
            ;;
        unarchive)
            local archives
            archives="$(command rtb __complete archives 2>/dev/null)"
            COMPREPLY=( $(compgen -W "\${archives}" -- "\${cur}") )
            ;;
        list)
            COMPREPLY=( $(compgen -W "--active --paused --deployed --vibe --all --verbose --json" -- "\${cur}") )
            ;;
        clean)
            COMPREPLY=( $(compgen -W "--commit --dry-run --force --days --json" -- "\${cur}") )
            ;;
        maintenance)
            COMPREPLY=( $(compgen -W "--full --json" -- "\${cur}") )
            ;;
        status)
            COMPREPLY=( $(compgen -W "--json" -- "\${cur}") )
            ;;
        doctor)
            COMPREPLY=( $(compgen -W "--json" -- "\${cur}") )
            ;;
        upgrade)
            COMPREPLY=( $(compgen -W "--check --force" -- "\${cur}") )
            ;;
        uninstall)
            COMPREPLY=( $(compgen -W "--force" -- "\${cur}") )
            ;;
        new)
            if [[ "\${cur}" == --* ]]; then
                COMPREPLY=( $(compgen -W "--stack" -- "\${cur}") )
            elif [ "\${prev}" = "--stack" ]; then
                COMPREPLY=( $(compgen -W "react nextjs node python generic" -- "\${cur}") )
            fi
            ;;
        deploy)
            if [[ "\${cur}" == --* ]]; then
                COMPREPLY=( $(compgen -W "--prod --staging" -- "\${cur}") )
            else
                local projs
                projs="$(command rtb __complete projects active 2>/dev/null)"
                COMPREPLY=( $(compgen -W "\${projs}" -- "\${cur}") )
            fi
            ;;
        shell-init|completion)
            COMPREPLY=( $(compgen -W "bash zsh fish pwsh" -- "\${cur}") )
            ;;
    esac
}
complete -F _rtb_completions rtb dev
`;

    case 'zsh': {
      // Use a helper to build this string to avoid ${(f) being misinterpreted
      // by esbuild/TypeScript as a template literal expression
      const zshOpenBrace = '${';
      return '#compdef rtb dev\n\n_rtb() {\n'
        + '    local -a commands\n'
        + '    local curcontext="$curcontext" state line\n'
        + '    typeset -A opt_args\n\n'
        + '    if [[ "$words[1]" == "dev" ]]; then\n'
        + '        local -a projs\n'
        + '        projs=(' + zshOpenBrace + '(f)"$(command rtb __complete projects 2>/dev/null)"})\n'
        + "        _describe 'project' projs\n"
        + '        return 0\n'
        + '    fi\n\n'
        + '    _arguments -C \\\\\n'
        + "        '1: :->command' \\\\\n"
        + "        '*:: :->args'\n\n"
        + '    case $state in\n'
        + '        command)\n'
        + '            commands=(' + zshOpenBrace + '(f)"$(command rtb __complete commands 2>/dev/null)"})\n'
        + "            _describe 'command' commands\n"
        + '            ;;\n'
        + '        args)\n'
        + '            case $words[1] in\n'
        + '                goto|open|run|build|test|info|workspace|deps|agent|agy|claude|gemini|codex|cursor|windsurf|aider|openhands|archive)\n'
        + '                    local -a projs\n'
        + '                    projs=(' + zshOpenBrace + '(f)"$(command rtb __complete projects 2>/dev/null)"})\n'
        + "                    _describe 'project' projs\n"
        + '                    ;;\n'
        + '                pause|deploy)\n'
        + '                    local -a projs\n'
        + '                    projs=(' + zshOpenBrace + '(f)"$(command rtb __complete projects active 2>/dev/null)"})\n'
        + "                    _describe 'active project' projs\n"
        + '                    ;;\n'
        + '                resume)\n'
        + '                    local -a projs\n'
        + '                    projs=(' + zshOpenBrace + '(f)"$(command rtb __complete projects paused 2>/dev/null)"})\n'
        + "                    _describe 'paused project' projs\n"
        + '                    ;;\n'
        + '                unarchive)\n'
        + '                    local -a archives\n'
        + '                    archives=(' + zshOpenBrace + '(f)"$(command rtb __complete archives 2>/dev/null)"})\n'
        + "                    _describe 'archive' archives\n"
        + '                    ;;\n'
        + '                shell-init|completion)\n'
        + '                    _values \'shell\' bash zsh fish pwsh\n'
        + '                    ;;\n'
        + '            esac\n'
        + '            ;;\n'
        + '    esac\n'
        + '}\n'
        + 'compdef _rtb rtb dev\n';
    }

    case 'fish':
      return `# Fish completion for rtb
complete -c rtb -n '__fish_use_subcommand' -f -a '(command rtb __complete commands 2>/dev/null)'
complete -c rtb -n '__fish_seen_subcommand_from goto open run build test info workspace agent agy claude gemini codex cursor windsurf aider openhands archive' -f -a '(command rtb __complete projects 2>/dev/null)'
complete -c rtb -n '__fish_seen_subcommand_from pause deploy' -f -a '(command rtb __complete projects active 2>/dev/null)'
complete -c rtb -n '__fish_seen_subcommand_from resume' -f -a '(command rtb __complete projects paused 2>/dev/null)'
complete -c rtb -n '__fish_seen_subcommand_from unarchive' -f -a '(command rtb __complete archives 2>/dev/null)'
complete -c rtb -n '__fish_seen_subcommand_from shell-init completion' -f -a 'bash zsh fish pwsh'
complete -c rtb -n '__fish_seen_subcommand_from list' -l active -l paused -l deployed -l vibe -l all -l verbose -l json
complete -c rtb -n '__fish_seen_subcommand_from clean' -l commit -l dry-run -l force -l days -l json
complete -c rtb -n '__fish_seen_subcommand_from maintenance' -l full -l json
complete -c rtb -n '__fish_seen_subcommand_from doctor' -l json
complete -c rtb -n '__fish_seen_subcommand_from status' -l json
complete -c rtb -n '__fish_seen_subcommand_from upgrade' -l check -l force
complete -c rtb -n '__fish_seen_subcommand_from uninstall' -l force
complete -c rtb -n '__fish_seen_subcommand_from new' -l stack -a 'react nextjs node python generic'
complete -c dev -f -a '(command rtb __complete projects 2>/dev/null)'
`;

    case 'pwsh':
    case 'powershell':
    case 'posh':
      return `# Tab completion for RTB CLI (PowerShell / pwsh)
function _rtb_get_config {
    $cfgPath = if ($env:RTB_CONFIG) { $env:RTB_CONFIG } else {
        Join-Path ([System.Environment]::GetFolderPath('UserProfile')) '.config\\rtb\\rtb.config.json'
    }
    if (Test-Path $cfgPath) {
        try { return Get-Content $cfgPath -Raw | ConvertFrom-Json } catch {}
    }
    return $null
}

function _rtb_get_all_projects {
    $cfg = _rtb_get_config
    if (-not $cfg -or -not $cfg.projectRoots) {
        try {
            $rtbApp = (Get-Command -CommandType Application,ExternalScript -Name rtb -ErrorAction SilentlyContinue | Select-Object -First 1)
            $invokeTarget = if ($rtbApp) { $rtbApp.Source } else { 'rtb' }
            return & $invokeTarget __complete projects 2>$null
        } catch { return @() }
    }
    $names = [System.Collections.Generic.HashSet[string]]::new()
    foreach ($prop in $cfg.projectRoots.PSObject.Properties) {
        $p = if ($prop.Value.path) { $prop.Value.path } else { $prop.Value }
        if ($p -and (Test-Path $p)) {
            foreach ($d in (Get-ChildItem -Path $p -Directory -ErrorAction SilentlyContinue)) {
                if (-not $d.Name.StartsWith('.')) {
                    [void]$names.Add($d.Name)
                }
            }
        }
    }
    return @($names)
}

function _rtb_get_projects_by_status($status) {
    $cfg = _rtb_get_config
    if (-not $cfg -or -not $cfg.projectRoots) {
        try {
            $rtbApp = (Get-Command -CommandType Application,ExternalScript -Name rtb -ErrorAction SilentlyContinue | Select-Object -First 1)
            $invokeTarget = if ($rtbApp) { $rtbApp.Source } else { 'rtb' }
            return & $invokeTarget __complete projects $status 2>$null
        } catch { return @() }
    }
    $names = [System.Collections.Generic.HashSet[string]]::new()
    $s = $status.ToLower()
    foreach ($prop in $cfg.projectRoots.PSObject.Properties) {
        $key = $prop.Name.ToLower()
        $label = if ($prop.Value.label) { $prop.Value.label.ToLower() } else { '' }
        if ($key -eq $s -or $key.StartsWith($s) -or $key.Contains($s) -or $label.Contains($s)) {
            $p = if ($prop.Value.path) { $prop.Value.path } else { $prop.Value }
            if ($p -and (Test-Path $p)) {
                foreach ($d in (Get-ChildItem -Path $p -Directory -ErrorAction SilentlyContinue)) {
                    if (-not $d.Name.StartsWith('.')) {
                        [void]$names.Add($d.Name)
                    }
                }
            }
        }
    }
    return @($names)
}

function _rtb_get_archives {
    $cfg = _rtb_get_config
    $backupDirs = @()
    if ($cfg -and $cfg.backupRoot) {
        $backupDirs += $cfg.backupRoot
        $backupDirs += (Join-Path $cfg.backupRoot 'project-snapshots')
    }
    $backupDirs += 'D:\\08-Backup\\project-snapshots'
    $backupDirs += 'D:\\08-Backups\\projects'
    $archives = [System.Collections.Generic.HashSet[string]]::new()
    foreach ($dir in $backupDirs) {
        if ($dir -and (Test-Path $dir)) {
            Get-ChildItem -Path $dir -Filter '*.tar.gz' -ErrorAction SilentlyContinue | ForEach-Object {
                [void]$archives.Add(($_.Name -replace '\\.tar\\.gz$', ''))
            }
        }
    }
    return @($archives)
}

$rtbCompleter = {
    param($wordToComplete, $commandAst, $cursorPosition)

    $subCommands = @(
        'init', 'config', 'doctor', 'upgrade', 'uninstall',
        'run', 'build', 'test', 'info', 'deps', 'workspace',
        'agent', 'goto', 'open', 'new', 'pause', 'resume', 'deploy',
        'archive', 'unarchive', 'list', 'health', 'clean', 'index',
        'guard', 'maintenance', 'backup', 'env', 'ui', 'menu', 'help',
        'version', 'status', 'shell-init', 'completion',
        'agy', 'claude', 'gemini', 'codex', 'cursor', 'windsurf', 'aider', 'openhands'
    )

    $elements = $commandAst.CommandElements
    $count = $elements.Count

    # Determine the invoked binary/alias command name
    $cmdName = if ($count -ge 1) {
        [System.IO.Path]::GetFileNameWithoutExtension($elements[0].Extent.Text).ToLower()
    } else { 'rtb' }

    # Safely escape $wordToComplete so special characters (hyphens, brackets, dots) match literally
    $escapedWord = if ([string]::IsNullOrEmpty($wordToComplete)) { '' } else {
        [System.Management.Automation.WildcardPattern]::Escape($wordToComplete)
    }

    # Special case: 'dev' alias is a direct shortcut for 'rtb goto'
    if ($cmdName -eq 'dev') {
        if ($wordToComplete -like '--*') {
            @('--agy', '--claude', '--gemini', '--cursor', '--windsurf', '--aider', '--openhands', '--print') |
                Where-Object { $_ -like "$escapedWord*" } |
                ForEach-Object {
                    [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                }
            return
        }
        (_rtb_get_all_projects) |
            Where-Object { $_ -like "$escapedWord*" } |
            ForEach-Object {
                [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
            }
        return
    }

    # If completing the subcommand position on 'rtb'
    $isSubcommand = $false
    if ($count -le 1) {
        $isSubcommand = $true
    } elseif ($count -eq 2) {
        $secondElem = $elements[1].Extent.Text
        if ($wordToComplete -eq $secondElem -or ($cursorPosition -and $cursorPosition -le $elements[1].Extent.EndOffset)) {
            $isSubcommand = $true
        }
    }

    if ($isSubcommand) {
        $subCommands |
            Where-Object { $_ -like "$escapedWord*" } |
            ForEach-Object {
                [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
            }
        return
    }

    $sub = $elements[1].Extent.Text.ToLower()

    $agentFlags = @('--agy', '--claude', '--gemini', '--codex', '--cursor', '--windsurf', '--aider', '--openhands', '--list', '--no-launch', '--json')
    $agentNames = @('agy', 'claude', 'gemini', 'codex', 'cursor', 'windsurf', 'aider', 'openhands')

    switch ($sub) {
        { $_ -in 'goto', 'open', 'run', 'build', 'test', 'info', 'workspace', 'deps', 'agent',
                 'agy', 'claude', 'gemini', 'codex', 'cursor', 'windsurf', 'aider', 'openhands' } {
            if ($sub -eq 'agent') {
                if ($wordToComplete -like '--*') {
                    $agentFlags |
                        Where-Object { $_ -like "$escapedWord*" } |
                        ForEach-Object {
                            [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                        }
                    return
                }
                ($agentNames + @('--list') + @(_rtb_get_all_projects)) | Sort-Object -Unique |
                    Where-Object { $_ -like "$escapedWord*" } |
                    ForEach-Object {
                        [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                    }
                return
            }

            if ($sub -eq 'goto' -and $wordToComplete -like '--*') {
                @('--agy', '--claude', '--gemini', '--cursor', '--windsurf', '--aider', '--openhands', '--print') |
                    Where-Object { $_ -like "$escapedWord*" } |
                    ForEach-Object {
                        [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                    }
                return
            }

            if ($sub -eq 'info' -and $wordToComplete -like '--*') {
                @('--json') |
                    Where-Object { $_ -like "$escapedWord*" } |
                    ForEach-Object {
                        [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                    }
                return
            }

            if ($sub -eq 'deps' -and $wordToComplete -notlike '--*') {
                (@('outdated') + @(_rtb_get_all_projects)) | Sort-Object -Unique |
                    Where-Object { $_ -like "$escapedWord*" } |
                    ForEach-Object {
                        [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                    }
                return
            }

            if ($wordToComplete -like '--*') {
                @('--help') |
                    Where-Object { $_ -like "$escapedWord*" } |
                    ForEach-Object {
                        [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                    }
                return
            }

            (_rtb_get_all_projects) |
                Where-Object { $_ -like "$escapedWord*" } |
                ForEach-Object {
                    [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                }
            break
        }

        { $_ -in 'pause', 'deploy' } {
            if ($wordToComplete -like '--*') {
                $flags = if ($sub -eq 'pause') { @('--prune') } else { @('--prod', '--staging') }
                $flags |
                    Where-Object { $_ -like "$escapedWord*" } |
                    ForEach-Object {
                        [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                    }
                return
            }
            (_rtb_get_projects_by_status 'active') |
                Where-Object { $_ -like "$escapedWord*" } |
                ForEach-Object {
                    [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                }
            break
        }

        'resume' {
            if ($wordToComplete -like '--*') {
                @('--install') |
                    Where-Object { $_ -like "$escapedWord*" } |
                    ForEach-Object {
                        [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                    }
                return
            }
            (_rtb_get_projects_by_status 'paused') |
                Where-Object { $_ -like "$escapedWord*" } |
                ForEach-Object {
                    [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                }
            break
        }

        'archive' {
            @((_rtb_get_projects_by_status 'active') + (_rtb_get_projects_by_status 'paused')) | Sort-Object -Unique |
                Where-Object { $_ -like "$escapedWord*" } |
                ForEach-Object {
                    [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                }
            break
        }

        'unarchive' {
            (_rtb_get_archives) |
                Where-Object { $_ -like "$escapedWord*" } |
                ForEach-Object {
                    [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                }
            break
        }

        'list' {
            @('--active', '--paused', '--deployed', '--vibe', '--all', '--verbose', '--json') |
                Where-Object { $_ -like "$escapedWord*" } |
                ForEach-Object {
                    [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                }
            break
        }

        'new' {
            $lastElement = if ($elements.Count -gt 2) { $elements[-1].Extent.Text } else { '' }
            if ($wordToComplete -like '--*' -or $lastElement -eq '--stack') {
                if ($lastElement -eq '--stack') {
                    @('react', 'nextjs', 'node', 'python', 'generic') |
                        Where-Object { $_ -like "$escapedWord*" } |
                        ForEach-Object {
                            [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                        }
                } else {
                    @('--stack') |
                        Where-Object { $_ -like "$escapedWord*" } |
                        ForEach-Object {
                            [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                        }
                }
            }
            break
        }

        'clean' {
            @('--commit', '--dry-run', '--force', '--days', '--json') |
                Where-Object { $_ -like "$escapedWord*" } |
                ForEach-Object {
                    [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                }
            break
        }

        'maintenance' {
            @('--full', '--json') |
                Where-Object { $_ -like "$escapedWord*" } |
                ForEach-Object {
                    [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                }
            break
        }

        'status' {
            @('--json') |
                Where-Object { $_ -like "$escapedWord*" } |
                ForEach-Object {
                    [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                }
            break
        }

        'doctor' {
            @('--json') |
                Where-Object { $_ -like "$escapedWord*" } |
                ForEach-Object {
                    [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                }
            break
        }

        'upgrade' {
            @('--check', '--force') |
                Where-Object { $_ -like "$escapedWord*" } |
                ForEach-Object {
                    [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                }
            break
        }

        'uninstall' {
            @('--force') |
                Where-Object { $_ -like "$escapedWord*" } |
                ForEach-Object {
                    [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                }
            break
        }

        'shell-init' {
            @('bash', 'zsh', 'fish', 'pwsh') |
                Where-Object { $_ -like "$escapedWord*" } |
                ForEach-Object {
                    [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                }
            break
        }

        'completion' {
            @('bash', 'zsh', 'fish', 'pwsh') |
                Where-Object { $_ -like "$escapedWord*" } |
                ForEach-Object {
                    [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                }
            break
        }
    }
}

Register-ArgumentCompleter -CommandName 'rtb', 'rtb.cmd', 'rtb.ps1', 'dev' -ScriptBlock $rtbCompleter
Register-ArgumentCompleter -Native -CommandName 'rtb', 'rtb.cmd', 'rtb.ps1', 'dev' -ScriptBlock $rtbCompleter
`;

    default:
      throw new Error(`Unsupported shell for completion: '\${shell}'. Supported: bash, zsh, fish, pwsh`);
  }
}

export function registerCompletionCommand(program: Command, getContext: () => CliContext): void {
  // Public rtb completion [shell] command
  program
    .command('completion [shell]')
    .description('Emit shell completion script (bash, zsh, fish, pwsh)')
    .action((shellName?: string) => {
      const targetShell = shellName || (process.platform === 'win32' ? 'pwsh' : 'bash');
      try {
        const script = getCompletionScript(targetShell);
        process.stdout.write(script);
      } catch (err: any) {
        outputError(err.message, 'UNSUPPORTED_SHELL', false);
        process.exitCode = 1;
      }
    });

  // Internal __complete helper for dynamic completion queries
  program
    .command('__complete <action> [target]')
    .description('Internal helper for shell autocompletion')
    .action((action: string, target?: string) => {
      const act = action.toLowerCase();
      if (act === 'commands') {
        process.stdout.write(ALL_SUBCOMMANDS.join('\n') + '\n');
        return;
      }

      // For projects or archives, resolve config
      const resolution = loadConfig();
      const config = resolution.config;

      if (act === 'projects') {
        const names = getProjectNames(config, target);
        if (names.length > 0) {
          process.stdout.write(names.join('\n') + '\n');
        }
        return;
      }

      if (act === 'archives') {
        const archives = getArchiveNames(config);
        if (archives.length > 0) {
          process.stdout.write(archives.join('\n') + '\n');
        }
        return;
      }
    });
}
