function Dev-Ui {
    $userBin = if ($env:RTB_BIN_DIR) {
        $env:RTB_BIN_DIR
    } elseif ($env:APPDATA) {
        Join-Path $env:APPDATA 'rtb\bin'
    } else {
        Join-Path ([Environment]::GetFolderPath('UserProfile')) '.config\rtb\bin'
    }

    $candidates = @(
        (Get-Command 'rtb' -CommandType Application -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Source),
        (Join-Path $userBin 'rtb.exe'),
        (Join-Path $userBin 'dev.exe'),
        (Join-Path $PSScriptRoot '..\..\..\tui\target\release\rtb.exe'),
        (Join-Path $PSScriptRoot '..\..\..\tui\target\debug\rtb.exe')
    )

    $binary = $null
    foreach ($cand in $candidates) {
        if ($cand -and (Test-Path $cand)) {
            $binary = $cand
            break
        }
    }

    if (-not $binary) {
        Write-Host '  rtb binary not found.' -ForegroundColor Red
        Write-Host '  Build it with: cargo build --release inside tui/, or run .\install.ps1' -ForegroundColor Gray
        return
    }
    & $binary
}

function Start-RtbUi { Dev-Ui @args }
