#compdef rtb

_rtb() {
    local -a commands
    commands=(
        'init:Initialize RTB configuration'
        'run:Run project script'
        'build:Build project'
        'test:Run project tests'
        'goto:Navigate to project directory'
        'new:Create a new project'
        'pause:Pause an active project'
        'resume:Resume a paused project'
        'deploy:Deploy project to production'
        'archive:Archive project'
        'list:List projects'
        'health:Show repository health'
        'clean:Prune unused dependency folders'
        'ui:Launch interactive TUI'
        'help:Show help overview'
    )
    _describe 'command' commands
}

_rtb "$@"
