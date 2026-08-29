# Tab completion registration for the RTB (rtb / dev) CLI

$rtbCompleter = {
    param($wordToComplete, $commandAst, $cursorPosition)

    $elements = $commandAst.CommandElements
    $count = $elements.Count

    $subCommands = @(
        'init', 'run', 'build', 'test', 'commit', 'info', 'agent', 'deps', 'workspace', 'upgrade',
        'goto', 'new', 'pause', 'resume', 'deploy', 'archive',
        'unarchive', 'list', 'health', 'clean', 'index',
        'backup', 'guard', 'env', 'maintenance', 'ui', 'help'
    )

    # 1. Complete subcommand (first argument after binary/function name)
    if ($count -le 2) {
        $subCommands |
            Where-Object { $_ -like "$wordToComplete*" } |
            ForEach-Object {
                [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
            }
        return
    }

    # 2. Complete subsequent arguments based on active subcommand
    $sub = $elements[1].Extent.Text.ToLower()

    switch ($sub) {
        { $_ -in 'goto', 'run', 'build', 'test', 'info', 'agent' } {
            if ($sub -eq 'agent' -and $count -gt 2) {
                @('agy', 'claude', 'gemini', 'codex', '--list') |
                    Where-Object { $_ -like "$wordToComplete*" } |
                    ForEach-Object {
                        [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                    }
            } else {
                $projects = Get-AllProjectNames
                $projects |
                    Where-Object { $_ -like "$wordToComplete*" } |
                    ForEach-Object {
                        [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                    }
            }
        }

        { $_ -in 'pause', 'deploy' } {
            $projects = Get-ProjectsByStatus 'active'
            $projects |
                Where-Object { $_ -like "$wordToComplete*" } |
                ForEach-Object {
                    [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                }
        }

        'resume' {
            $projects = Get-ProjectsByStatus 'paused'
            $projects |
                Where-Object { $_ -like "$wordToComplete*" } |
                ForEach-Object {
                    [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                }
        }

        'archive' {
            $names = @()
            $names += Get-ProjectsByStatus 'active'
            $names += Get-ProjectsByStatus 'paused'
            $names | Sort-Object -Unique |
                Where-Object { $_ -like "$wordToComplete*" } |
                ForEach-Object {
                    [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                }
        }

        'unarchive' {
            $archiveDir = 'D:\08-Backup\project-snapshots'
            if (Test-Path $archiveDir) {
                Get-ChildItem -Path $archiveDir -Filter '*.tar.gz' -ErrorAction SilentlyContinue | ForEach-Object {
                    $base = $_.Name -replace '\.tar\.gz$', ''
                    if ($base -like "$wordToComplete*") {
                        [System.Management.Automation.CompletionResult]::new($base, $base, 'ParameterValue', $_.Name)
                    }
                }
            }
        }

        'list' {
            @('--active', '--paused', '--deployed', '--vibe', '--all', '--json') |
                Where-Object { $_ -like "$wordToComplete*" } |
                ForEach-Object {
                    [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                }
        }

        'new' {
            if ($wordToComplete -like '--*' -or $elements.Count -gt 2) {
                @('--stack', 'react', 'nextjs', 'node', 'python', 'generic') |
                    Where-Object { $_ -like "$wordToComplete*" } |
                    ForEach-Object {
                        [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                    }
            }
        }

        'clean' {
            @('--force', '--dry-run', '--days') |
                Where-Object { $_ -like "$wordToComplete*" } |
                ForEach-Object {
                    [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                }
        }

        'commit' {
            @('--amend', '--push') |
                Where-Object { $_ -like "$wordToComplete*" } |
                ForEach-Object {
                    [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                }
        }
    }
}

Register-ArgumentCompleter -CommandName 'rtb', 'dev' -ScriptBlock $rtbCompleter
Register-ArgumentCompleter -CommandName 'rtb', 'dev' -ParameterName 'Command' -ScriptBlock $rtbCompleter
Register-ArgumentCompleter -CommandName 'rtb', 'dev' -ParameterName 'Arguments' -ScriptBlock $rtbCompleter
