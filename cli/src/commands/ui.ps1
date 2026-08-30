function Dev-Ui {
    $userBin = if ($env:RTB_BIN_DIR) {
        $env:RTB_BIN_DIR
    } elseif ($env:APPDATA) {
        Join-Path $env:APPDATA 'rtb\bin'
    } else {
        Join-Path ([Environment]::GetFolderPath('UserProfile')) '.config\rtb\bin'
    }

    $candidates = @(
        (Get-Command 'rtbtui' -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Source),
        (Join-Path $userBin 'rtbtui.exe'),
        (Join-Path $userBin 'devtui.exe'),
        (Join-Path $PSScriptRoot '..\..\..\tui\target\release\rtbtui.exe'),
        (Join-Path $PSScriptRoot '..\..\..\tui\target\debug\rtbtui.exe')
    )

    $binary = $null
    foreach ($cand in $candidates) {
        if ($cand -and (Test-Path $cand)) {
            $binary = $cand
            break
        }
    }

    if (-not $binary) {
        Write-Host '  rtbtui binary not found.' -ForegroundColor Red
        Write-Host '  Build it with: cargo build --release inside tui/, or run .\install.ps1' -ForegroundColor Gray
        return
    }
    & $binary
}

function Start-RtbUi { Dev-Ui @args }
