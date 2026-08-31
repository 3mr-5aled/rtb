function Rtb-Init {
    [CmdletBinding()]
    param(
        [Parameter(Position = 0, ValueFromRemainingArguments = $true)]
        [string[]]$RemainingArgs,

        [switch]$Force
    )
    
    $allArgs = @()
    if ($RemainingArgs) { $allArgs += $RemainingArgs }
    $isForce = $Force.IsPresent -or ($allArgs -contains '-Force') -or ($allArgs -contains '--force') -or ($allArgs -contains '-f')

    Write-RtbHeader -Title "Interactive Setup Wizard"
    
    $userHomeDir = if ($env:USERPROFILE) { $env:USERPROFILE } else { $env:HOME }
    $userConfigDir = if ($env:APPDATA) { Join-Path $env:APPDATA 'rtb' } else { Join-Path $userHomeDir '.config/rtb' }
    $userConfigFile = Join-Path $userConfigDir 'rtb.config.json'
    
    if (-not (Test-Path $userConfigDir)) {
        New-Item -ItemType Directory -Path $userConfigDir -Force | Out-Null
    }
    
    if ((Test-Path $userConfigFile) -and -not $isForce) {
        Write-Host ""
        Write-Host "  Configuration already exists at:" -ForegroundColor Yellow
        Write-Host "    $userConfigFile" -ForegroundColor White
        Write-Host "  Use 'rtb init -Force' to overwrite and re-run the setup wizard." -ForegroundColor Gray
        Write-Host ""
        return
    }

    # ── Step 1: Detect or select projects root ─────────────────────────────
    Write-Host "`n  Step 1: Workspace Root Location" -ForegroundColor Cyan
    Write-Host "  Where do you want to keep and manage your projects?" -ForegroundColor Gray

    $defaultRoot = Join-Path $userHomeDir 'Projects'
    $candidateList = @(
        $defaultRoot,
        (Join-Path $userHomeDir 'dev'),
        (Join-Path $userHomeDir 'code'),
        (Join-Path $userHomeDir 'repos'),
        (Join-Path $userHomeDir 'workspace'),
        (Join-Path $userHomeDir 'src'),
        'D:\02-Projects',
        'D:\Projects',
        'D:\dev'
    ) | Select-Object -Unique

    $existingCandidates = @($candidateList | Where-Object { Test-Path $_ })
    $chosenRoot = $null

    if ($existingCandidates.Count -gt 0) {
        Write-Host "`n  Detected existing project directories:" -ForegroundColor Yellow
        for ($i = 0; $i -lt $existingCandidates.Count; $i++) {
            Write-Host "    [$($i + 1)] $($existingCandidates[$i])" -ForegroundColor White
        }
        Write-Host "    [C] Enter custom path" -ForegroundColor DarkGray
        Write-Host "`n  Select an option [1-$($existingCandidates.Count) or C] (Default: 1): " -ForegroundColor Yellow -NoNewline
        $selection = Read-Host

        if ([string]::IsNullOrWhiteSpace($selection)) {
            $chosenRoot = $existingCandidates[0]
        } elseif ($selection -match '^\d+$' -and [int]$selection -ge 1 -and [int]$selection -le $existingCandidates.Count) {
            $chosenRoot = $existingCandidates[[int]$selection - 1]
        }
    }

    if (-not $chosenRoot) {
        Write-Host "  Enter your projects root path (Default: $defaultRoot): " -ForegroundColor Yellow -NoNewline
        $customInput = Read-Host
        $chosenRoot = if ([string]::IsNullOrWhiteSpace($customInput)) { $defaultRoot } else { $customInput.Trim().Trim('"').Trim("'") }
    }

    # Expand environment variables or relative references
    $chosenRoot = [System.IO.Path]::GetFullPath($chosenRoot)
    Write-Host "  Selected root: $chosenRoot" -ForegroundColor Green

    if (-not (Test-Path $chosenRoot)) {
        New-Item -ItemType Directory -Path $chosenRoot -Force | Out-Null
        Write-Host "  Created root directory at: $chosenRoot" -ForegroundColor Gray
    }

    # ── Step 2: Select folders to scaffold ──────────────────────────────────
    Write-Host "`n  Step 2: Workspace Organization Scaffold" -ForegroundColor Cyan
    Write-Host "  RTB can organize projects into lifecycle subfolders." -ForegroundColor Gray
    Write-Host "  Would you like to scaffold the standard folder structure? [Y/n]: " -ForegroundColor Yellow -NoNewline
    $scaffoldAns = Read-Host
    $shouldScaffold = ([string]::IsNullOrWhiteSpace($scaffoldAns) -or $scaffoldAns.Trim() -match '^(y|yes)$')

    $folderDefs = @(
        @{ Key = 'active';     Dir = '01-Active';    Rel = '02-Projects\01-Development\01-Active';    Label = 'Active';     Emoji = '📁'; Selected = $true },
        @{ Key = 'paused';     Dir = '04-Paused';    Rel = '02-Projects\01-Development\04-Paused';    Label = 'Paused';     Emoji = '⏸️';  Selected = $true },
        @{ Key = 'production'; Dir = '02-Deployed';  Rel = '02-Projects\02-Deployed\01-Production';   Label = 'Production'; Emoji = '🚀'; Selected = $true },
        @{ Key = 'planning';   Dir = '02-Planning';  Rel = '02-Projects\01-Development\02-Planning';  Label = 'Planning';   Emoji = '📋'; Selected = $false },
        @{ Key = 'testing';    Dir = '03-Testing';   Rel = '02-Projects\01-Development\03-Testing';   Label = 'Testing';    Emoji = '🧪'; Selected = $false },
        @{ Key = 'abandoned';  Dir = '05-Abandoned'; Rel = '02-Projects\01-Development\05-Abandoned'; Label = 'Abandoned';  Emoji = '🪦'; Selected = $false },
        @{ Key = 'sandbox';    Dir = '01-SandBox';   Rel = '01-SandBox';                              Label = 'Sandbox';    Emoji = '📦'; Selected = $false }
    )

    if ($shouldScaffold) {
        $selecting = $true
        while ($selecting) {
            Write-Host "`n  Select folders to create (Type numbers to toggle, press ENTER to confirm):" -ForegroundColor Yellow
            for ($i = 0; $i -lt $folderDefs.Count; $i++) {
                $f = $folderDefs[$i]
                $check = if ($f.Selected) { "[✓]" } else { "[ ]" }
                $color = if ($f.Selected) { "Green" } else { "DarkGray" }
                Write-Host "    $check $($i + 1). $($f.Emoji) $($f.Label) → $($f.Dir)" -ForegroundColor $color
            }
            Write-Host "  Toggle numbers (e.g. '4 5') or ENTER to proceed: " -ForegroundColor Yellow -NoNewline
            $inputToggles = Read-Host

            if ([string]::IsNullOrWhiteSpace($inputToggles)) {
                $selecting = $false
                break
            }

            $tokens = $inputToggles -split '[\s,]+'
            foreach ($t in $tokens) {
                if ($t -match '^\d+$') {
                    $idx = [int]$t - 1
                    if ($idx -ge 0 -and $idx -lt $folderDefs.Count) {
                        $folderDefs[$idx].Selected = -not $folderDefs[$idx].Selected
                    }
                }
            }
        }
    }

    # ── Step 3: Customize Emoji / Name for Selected Folders ─────────────────
    Write-Host "`n  Step 3: Custom Display Names & Emojis" -ForegroundColor Cyan
    Write-Host "  (Press ENTER to accept suggested defaults)" -ForegroundColor Gray

    for ($i = 0; $i -lt $folderDefs.Count; $i++) {
        $f = $folderDefs[$i]
        if (-not $f.Selected) { continue }

        Write-Host "  Customize '$($f.Label)'? (Enter to keep '$($f.Emoji) $($f.Label)'): " -ForegroundColor Yellow -NoNewline
        $customLabelInput = Read-Host

        if (-not [string]::IsNullOrWhiteSpace($customLabelInput)) {
            $trimmed = $customLabelInput.Trim()
            # If user provided emoji + label, or just label
            if ($trimmed -match '^(\p{So}|\p{Cs}|\p{Sk}|[\u2600-\u27BF]|\uD83C[\uDF00-\uDFFF]|\uD83D[\uDC00-\uDE4F]|\uD83E[\uDD00-\uDDFF])\s*(.*)$') {
                $f.Emoji = $Matches[1]
                if (-not [string]::IsNullOrWhiteSpace($Matches[2])) {
                    $f.Label = $Matches[2].Trim()
                }
            } else {
                $f.Label = $trimmed
            }
        }
    }

    # ── Step 4: Create Directories & Build Config ───────────────────────────
    $projectRoots = [ordered]@{}

    foreach ($f in $folderDefs) {
        $folderPath = Join-Path $chosenRoot $f.Rel
        if ($f.Selected) {
            if (-not (Test-Path $folderPath)) {
                New-Item -ItemType Directory -Path $folderPath -Force | Out-Null
            }
        }
        $projectRoots[$f.Key] = [ordered]@{
            path  = $folderPath
            label = $f.Label
            emoji = $f.Emoji
        }
    }

    # Extra non-scaffolded optional roots preserved for consistency
    $stagingPath = Join-Path $chosenRoot '02-Projects\02-Deployed\02-Staging'
    $projectRoots['staging'] = [ordered]@{
        path  = $stagingPath
        label = 'Staging'
        emoji = '🚀'
    }

    $vibePath = Join-Path $chosenRoot '02-Projects\03-Vibe-Coding'
    $projectRoots['vibe'] = [ordered]@{
        path  = $vibePath
        label = 'Vibe Coding'
        emoji = '✨'
    }

    $backupRoot = Join-Path $chosenRoot '08-Backup'
    $configRoot = Join-Path $chosenRoot '05-Config'
    $templateDir = Join-Path $chosenRoot '05-Config\templates'

    $newConfig = [ordered]@{
        version            = "1.0.0"
        projectRoots       = $projectRoots
        backupRoot         = $backupRoot
        configRoot         = $configRoot
        templateDir        = $templateDir
        cleanDeps          = [ordered]@{
            daysInactive = 60
            targets      = @("node_modules", ".venv", ".next", "__pycache__", "dist", "build", "target")
        }
        staleThresholdDays = 90
        gitHealth          = [ordered]@{
            scanRoots = @((Join-Path $chosenRoot '02-Projects'), (Join-Path $chosenRoot '01-SandBox'))
        }
    }

    $json = $newConfig | ConvertTo-Json -Depth 6
    Set-Content -Path $userConfigFile -Value $json -Encoding UTF8

    # Also sync to dot-config if applicable
    $dotConfigDir = Join-Path $userHomeDir '.config\rtb'
    if (-not (Test-Path $dotConfigDir)) { New-Item -ItemType Directory -Path $dotConfigDir -Force | Out-Null }
    Set-Content -Path (Join-Path $dotConfigDir 'rtb.config.json') -Value $json -Encoding UTF8 -ErrorAction SilentlyContinue

    Write-Host "`n  ✓ RTB configuration successfully initialized!" -ForegroundColor Green
    Write-Host "    Configuration file: $userConfigFile" -ForegroundColor White
    Write-Host "    Workspace root    : $chosenRoot" -ForegroundColor White
    Write-Host "`n  Ready to build! Run 'rtb help' or launch the TUI with 'rtb ui'." -ForegroundColor Cyan
}

function Initialize-RtbConfig { Rtb-Init @args }
