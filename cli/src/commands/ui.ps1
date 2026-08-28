function Dev-Ui {
    $binary = 'D:\06-Tools\scripts\devtui.exe'
    if (-not (Test-Path $binary)) {
        $localBuild = Join-Path $PSScriptRoot '..\..\..\tui\target\release\devtui.exe'
        if (Test-Path $localBuild) {
            $binary = $localBuild
        } else {
            Write-Host '  devtui binary not found at D:\06-Tools\scripts\devtui.exe' -ForegroundColor Red
            Write-Host '  Build it with: cargo build --release in tui/' -ForegroundColor Gray
            return
        }
    }
    & $binary
}
