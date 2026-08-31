function Rtb-Help {
    Write-Host ''
    Write-Host '  rtb' -ForegroundColor Cyan -NoNewline
    Write-Host ' - Repository & Tooling Base Developer Project Operations CLI' -ForegroundColor White
    Write-Host ''
    Write-Host '  SETUP & CONFIG' -ForegroundColor Yellow
    Write-Host '    rtb init [--force]                   Interactive workspace setup wizard'
    Write-Host '    rtb doctor                           System health & dependency diagnostic'
    Write-Host '    rtb upgrade [--check] [--force]      Check for updates and self-upgrade'
    Write-Host '    rtb uninstall [--force]              Cleanly uninstall RTB from system'
    Write-Host '    rtb --version                        Display RTB version'
    Write-Host '    rtb --help                           Display this help menu'
    Write-Host ''
    Write-Host '  PROJECT OPERATIONS' -ForegroundColor Yellow
    Write-Host '    rtb run [project]                    Auto-detect and run dev/start script'
    Write-Host '    rtb build [project]                  Auto-detect and run build script'
    Write-Host '    rtb test [project]                   Auto-detect and run test suite'
    Write-Host '    rtb info [project] [--json]          Display deep project intelligence'
    Write-Host '    rtb deps [outdated] [project]        Audit declared project dependencies'
    Write-Host '    rtb workspace [project]              Inspect monorepo workspace packages'
    Write-Host ''
    Write-Host '  AI AGENT LAUNCHERS' -ForegroundColor Yellow
    Write-Host '    rtb agy [project]                    Launch Google Antigravity CLI'
    Write-Host '    rtb claude|gemini|codex [project]    Launch Claude / Gemini / Codex CLI'
    Write-Host '    rtb cursor|windsurf|aider [project]  Launch Cursor / Windsurf / Aider'
    Write-Host '    rtb agent [project] [--agy|--claude] Launch targeted agent (or rtb agent -List)'
    Write-Host '    rtb goto <name> [--agy|--claude]     cd into project and launch agent CLI'
    Write-Host ''
    Write-Host '  LIFECYCLE' -ForegroundColor Yellow
    Write-Host '    rtb new <name> [--stack nextjs]     Create new project in 01-Active'
    Write-Host '    rtb pause <name> [--prune]          Pause project (move to 04-Paused)'
    Write-Host '    rtb resume <name> [--install]        Resume paused project'
    Write-Host '    rtb deploy <name> [--prod|--staging] Deploy to production/staging'
    Write-Host '    rtb archive <name>                   Compress to .tar.gz backup'
    Write-Host '    rtb unarchive <name>                 Extract archive to 01-Active'
    Write-Host ''
    Write-Host '  NAVIGATION' -ForegroundColor Yellow
    Write-Host '    rtb goto <name>                      cd into any project (TAB to search)'
    Write-Host '    rtb status [--json]                  Single-line project & git prompt segment'
    Write-Host '    rtb open <name>                      Open project folder in File Explorer'
    Write-Host '    rtb list [--active|--paused|--all]   List projects with status'
    Write-Host ''
    Write-Host '  MAINTENANCE & SAFETY' -ForegroundColor Yellow
    Write-Host '    rtb health                           Git repo health scan'
    Write-Host '    rtb clean [--commit] [--dry-run]     Prune inactive dependencies (dry-run default)'
    Write-Host '    rtb index                            Generate PROJECT-INDEX.md'
    Write-Host '    rtb guard                            D drive root guardrail'
    Write-Host '    rtb maintenance [--full]             Run all maintenance tasks'
    Write-Host ''
    Write-Host '  BACKUP' -ForegroundColor Yellow
    Write-Host '    rtb backup                           Full config backup'
    Write-Host '    rtb env                              Backup all .env files'
    Write-Host ''
    Write-Host '  UI' -ForegroundColor Yellow
    Write-Host '    rtb ui (or rtbtui)                   Launch interactive TUI operations center'
    Write-Host ''
    Write-Host '  Tip: Press TAB after any command for auto-completion!' -ForegroundColor Gray
    Write-Host ''
}

function Dev-Help {
    Rtb-Help
}


function Get-RtbHelp { Rtb-Help @args }
