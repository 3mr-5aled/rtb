# Tab completion registration for the dev CLI

# 1. Complete the first argument: subcommand (goto, list, new, pause, etc.)
Register-ArgumentCompleter -CommandName 'dev' -ParameterName 'Command' -ScriptBlock {
    param($commandName, $parameterName, $wordToComplete, $commandAst, $fakeBoundParameters)
    
    $subCommands = @(
        'goto', 'new', 'pause', 'resume', 'deploy', 'archive',
        'unarchive', 'list', 'health', 'clean', 'index',
        'backup', 'guard', 'env', 'maintenance', 'ui', 'help'
    )
    
    $subCommands |
        Where-Object { $_ -like "$wordToComplete*" } |
        ForEach-Object {
            [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', "dev $_")
        }
}

# 2. Complete subsequent arguments based on the active subcommand
Register-ArgumentCompleter -CommandName 'dev' -ParameterName 'Arguments' -ScriptBlock {
    param($commandName, $parameterName, $wordToComplete, $commandAst, $fakeBoundParameters)
    
    # Determine the subcommand
    $sub = $fakeBoundParameters['Command']
    if (-not $sub) {
        $elements = $commandAst.CommandElements
        if ($elements.Count -gt 1) {
            $sub = $elements[1].Extent.Text
        }
    }
    
    if (-not $sub) { return }

    switch ($sub.ToLower()) {
        'goto' {
            # List all projects across all roots
            $projects = Get-AllProjectNames
            $projects |
                Where-Object { $_ -like "$wordToComplete*" } |
                ForEach-Object {
                    [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                }
        }

        { $_ -in 'pause', 'deploy' } {
            # Only active projects
            $projects = Get-ProjectsByStatus 'active'
            $projects |
                Where-Object { $_ -like "$wordToComplete*" } |
                ForEach-Object {
                    [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                }
        }

        'resume' {
            # Only paused projects
            $projects = Get-ProjectsByStatus 'paused'
            $projects |
                Where-Object { $_ -like "$wordToComplete*" } |
                ForEach-Object {
                    [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                }
        }

        'archive' {
            # Active + Paused projects
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
            # List .tar.gz files in 08-Backup/project-snapshots
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
            @('--active', '--paused', '--deployed', '--vibe', '--all') |
                Where-Object { $_ -like "$wordToComplete*" } |
                ForEach-Object {
                    [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                }
        }

        'new' {
            $elements = $commandAst.CommandElements
            $lastElement = if ($elements.Count -gt 2) { $elements[-1].Extent.Text } else { '' }
            $secondLast = if ($elements.Count -gt 3) { $elements[-2].Extent.Text } else { '' }
            
            if ($wordToComplete -like '--*' -or $lastElement -eq '--stack') {
                if ($lastElement -eq '--stack' -or $secondLast -eq '--stack') {
                    @('react', 'nextjs', 'node', 'python', 'generic') |
                        Where-Object { $_ -like "$wordToComplete*" } |
                        ForEach-Object {
                            [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                        }
                } else {
                    @('--stack') |
                        Where-Object { $_ -like "$wordToComplete*" } |
                        ForEach-Object {
                            [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                        }
                }
            }
        }

        'clean' {
            @('--force', '--days') |
                Where-Object { $_ -like "$wordToComplete*" } |
                ForEach-Object {
                    [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                }
        }

        'maintenance' {
            @('--full') |
                Where-Object { $_ -like "$wordToComplete*" } |
                ForEach-Object {
                    [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                }
        }
    }
}
