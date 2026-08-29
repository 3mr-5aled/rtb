function Dev-Guard {
    Write-RtbHeader 'Root Guardrail'
    Invoke-MaintenanceTask -Task 'guard' -ReportOnly
}


function Protect-RtbRoot { Dev-Guard @args }
