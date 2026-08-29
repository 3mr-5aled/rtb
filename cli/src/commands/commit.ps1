<#
.SYNOPSIS
    rtb commit [-Message <string>] [-Amend] [-Push]
.DESCRIPTION
    Interactive CLI pop up or prompt to stage files and write a git commit message.
#>
function Rtb-Commit {
    [CmdletBinding()]
    param(
        [Parameter(Position = 0)]
        [string]$Message,

        [switch]$Amend,
        [switch]$Push
    )

    Write-RtbHeader 'Git Commit & Push'

    # Check if in git repository
    if (-not (Test-Path '.git')) {
        Write-Host "  Error: Current directory is not a Git repository." -ForegroundColor Red
        return
    }

    # Staged/Unstaged status check
    $status = git status --short
    if (-not $status) {
        Write-Host "  Working tree is clean. Nothing to commit." -ForegroundColor Green
        return
    }

    Write-Host "  Changed Files:" -ForegroundColor Yellow
    $status | ForEach-Object { Write-Host "    $_" -ForegroundColor Cyan }
    Write-Host ""

    # If message is not supplied, prompt via interactive pop up / Read-Host
    if (-not $Message) {
        $msgFromPopup = $null
        try {
            Add-Type -AssemblyName Microsoft.VisualBasic -ErrorAction SilentlyContinue
            $msgFromPopup = [Microsoft.VisualBasic.Interaction]::InputBox("Enter your Git commit message:", "RTB Git Commit", "update: sync workspace changes")
        } catch {}

        if ($msgFromPopup -and $msgFromPopup.Trim().Length -gt 0) {
            $Message = $msgFromPopup.Trim()
        } else {
            Write-Host -NoNewline "  Enter Commit Message: " -ForegroundColor Yellow
            $Message = Read-Host
        }
    }

    if (-not $Message -or $Message.Trim().Length -eq 0) {
        $Message = "update: sync workspace changes"
    }

    Write-Host "  Staging files (git add .)..." -ForegroundColor Gray
    git add .

    $commitArgs = @("commit", "-m", $Message)
    if ($Amend) { $commitArgs += "--amend" }

    Write-Host "  Running git commit..." -ForegroundColor Cyan
    git @commitArgs

    if ($LASTEXITCODE -eq 0) {
        Write-Host "  Successfully committed with message: '$Message'" -ForegroundColor Green
        if ($Push) {
            Write-Host "  Pushing to remote..." -ForegroundColor Cyan
            git push
        }
    } else {
        Write-Host "  Git commit failed." -ForegroundColor Red
    }
}

function Dev-Commit { Rtb-Commit @args }
