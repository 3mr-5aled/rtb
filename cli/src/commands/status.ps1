function Rtb-Status {
    [CmdletBinding()]
    param(
        [Parameter(Position = 0)]
        [object]$FirstArg,

        [Parameter(Position = 1, ValueFromRemainingArguments = $true)]
        [string[]]$RemainingArgs,

        [Alias('j')]
        [switch]$Json
    )

    $allArgs = @()
    if ($null -ne $FirstArg) { $allArgs += "$FirstArg" }
    if ($RemainingArgs) { $allArgs += $RemainingArgs }

    $isJson = $Json.IsPresent -or ($allArgs -contains '-Json') -or ($allArgs -contains '--json') -or ($allArgs -contains '-j') -or ($allArgs -contains '-J') -or ($allArgs -contains '-json')

    $cwd = (Get-Location).Path
    $config = try { Get-RtbConfig -ErrorAction SilentlyContinue } catch { $null }

    $projectName = $null
    $projectStatus = $null
    $projectRootPath = $null

    if ($config -and $config.projectRoots) {
        $rootMap = [ordered]@{
            'Active'     = (Get-RtbRootPath $config.projectRoots.active)
            'Paused'     = (Get-RtbRootPath $config.projectRoots.paused)
            'Production' = (Get-RtbRootPath $config.projectRoots.production)
            'Staging'    = (Get-RtbRootPath $config.projectRoots.staging)
            'Vibe'       = (Get-RtbRootPath $config.projectRoots.vibe)
            'Sandbox'    = (Get-RtbRootPath $config.projectRoots.sandbox)
            'Planning'   = (Get-RtbRootPath $config.projectRoots.planning)
            'Testing'    = (Get-RtbRootPath $config.projectRoots.testing)
            'Abandoned'  = (Get-RtbRootPath $config.projectRoots.abandoned)
        }
        foreach ($status in $rootMap.Keys) {
            $root = $rootMap[$status]
            if ($root -and $cwd.StartsWith($root, [System.StringComparison]::OrdinalIgnoreCase)) {
                $relative = $cwd.Substring($root.Length).TrimStart('\', '/')
                if (-not [string]::IsNullOrWhiteSpace($relative)) {
                    $projectName = ($relative -split '[\\/]+')[0]
                    $projectStatus = $status
                    $projectRootPath = Join-Path $root $projectName
                    break
                }
            }
        }
    }

    # Upward Git Discovery
    $branch = ''
    $uncommitted = 0
    $gitRoot = $null
    $check = $cwd
    while ($check) {
        if (Test-Path -LiteralPath (Join-Path $check '.git')) {
            $gitRoot = $check
            try {
                $branchRaw = git -C $check branch --show-current 2>$null
                if ($branchRaw -and $branchRaw.Trim()) {
                    $branch = $branchRaw.Trim()
                } else {
                    $headRef = git -C $check rev-parse --short HEAD 2>$null
                    if ($headRef -and $headRef.Trim()) {
                        $branch = "HEAD@$($headRef.Trim())"
                    }
                }
                $statusLines = git -C $check status --porcelain 2>$null
                $uncommitted = if ($statusLines) { (@($statusLines) | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }).Count } else { 0 }
            } catch {}
            break
        }
        try {
            $parent = Split-Path $check -Parent
        } catch {
            break
        }
        if (-not $parent -or $parent -eq $check) { break }
        $check = $parent
    }

    # Stack detection across CWD, project root, and git root
    $searchPaths = @($cwd)
    if ($projectRootPath -and (Test-Path -LiteralPath $projectRootPath) -and ($searchPaths -notcontains $projectRootPath)) {
        $searchPaths += $projectRootPath
    }
    if ($gitRoot -and (Test-Path -LiteralPath $gitRoot) -and ($searchPaths -notcontains $gitRoot)) {
        $searchPaths += $gitRoot
    }

    $stackList = [System.Collections.Generic.List[string]]::new()
    foreach ($p in $searchPaths) {
        if (-not (Test-Path -LiteralPath $p)) { continue }
        if ((Test-Path -LiteralPath (Join-Path $p 'package.json')) -and (-not $stackList.Contains('Node.js'))) {
            $stackList.Add('Node.js')
        }
        if (((Test-Path -LiteralPath (Join-Path $p 'Cargo.toml')) -or (Test-Path -LiteralPath (Join-Path $p 'tui\Cargo.toml'))) -and (-not $stackList.Contains('Rust'))) {
            $stackList.Add('Rust')
        }
        if ((Test-Path -LiteralPath (Join-Path $p 'go.mod')) -and (-not $stackList.Contains('Go'))) {
            $stackList.Add('Go')
        }
        if (((Test-Path -LiteralPath (Join-Path $p 'pyproject.toml')) -or (Test-Path -LiteralPath (Join-Path $p 'requirements.txt')) -or (Test-Path -LiteralPath (Join-Path $p 'uv.lock'))) -and (-not $stackList.Contains('Python'))) {
            $stackList.Add('Python')
        }
        if (((Test-Path -LiteralPath (Join-Path $p 'rtb.psm1')) -or (Test-Path -LiteralPath (Join-Path $p 'rtb.psd1')) -or (Test-Path -LiteralPath (Join-Path $p 'cli\rtb.psm1')) -or (Test-Path -LiteralPath (Join-Path $p 'dev.psm1'))) -and (-not $stackList.Contains('PowerShell'))) {
            $stackList.Add('PowerShell')
        }
    }
    $stack = $stackList.ToArray()

    $displayName = if ($projectName) { $projectName } else { Split-Path $cwd -Leaf }
    if (-not $displayName) { $displayName = $cwd }

    if ($isJson) {
        return [PSCustomObject]@{
            project     = $displayName
            status      = $projectStatus
            branch      = $branch
            uncommitted = [int]$uncommitted
            stack       = @($stack)
            cwd         = $cwd
        } | ConvertTo-Json -Compress
    }

    $gitPart = if ($branch) {
        $unStr = if ($uncommitted -gt 0) { " ±$uncommitted" } else { '' }
        " [$branch$unStr]"
    } else { '' }
    $stackPart  = if ($stack.Count -gt 0) { " $($stack -join ',')" } else { '' }
    $statusPart = if ($projectStatus) { " ($projectStatus)" } else { '' }

    return "rtb » $displayName$statusPart$gitPart$stackPart"
}

function Dev-Status { Rtb-Status @args }
function Get-RtbStatus { Rtb-Status @args }
