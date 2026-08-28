function Dev-Maintenance {
    $full = $args -contains '--full'
    Write-RtbHeader 'Weekly Maintenance'
    if ($full) {
        & 'D:\06-Tools\scripts\weekly-maintenance.ps1' -FullRun
    } else {
        & 'D:\06-Tools\scripts\weekly-maintenance.ps1'
    }
}
