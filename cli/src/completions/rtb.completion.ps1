# Tab completion registration for the RTB (rtb) CLI

Register-ArgumentCompleter -CommandName 'rtb' -ParameterName 'Command' -ScriptBlock {
    param($commandName, $parameterName, $wordToComplete, $commandAst, $fakeBoundParameters)
    
    $subCommands = @(
        'init', 'run', 'build', 'test', 'goto', 'new', 'pause', 'resume', 'deploy', 'archive',
        'unarchive', 'list', 'info', 'agent', 'health', 'clean', 'index',
        'backup', 'guard', 'env', 'maintenance', 'ui', 'help'
    )
    
    $subCommands |
        Where-Object { $_ -like "$wordToComplete*" } |
        ForEach-Object {
            [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', "rtb $_")
        }
}

Register-ArgumentCompleter -CommandName 'rtb' -ParameterName 'Arguments' -ScriptBlock {
    param($commandName, $parameterName, $wordToComplete, $commandAst, $fakeBoundParameters)
    
    $sub = $fakeBoundParameters['Command']
    if (-not $sub) {
        $elements = $commandAst.CommandElements
        if ($elements.Count -gt 1) {
            $sub = $elements[1].Extent.Text
        }
    }
    
    if (-not $sub) { return }

    switch ($sub.ToLower()) {
        { $_ -in 'goto', 'run', 'build', 'test', 'info', 'agent' } {
            $elements = $commandAst.CommandElements
            if ($elements.Count -gt 2 -and $sub.ToLower() -eq 'agent') {
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

        'clean' {
            @('--force', '--dry-run', '--days') |
                Where-Object { $_ -like "$wordToComplete*" } |
                ForEach-Object {
                    [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                }
        }
    }
}
