# Safety Guardrails Unit Tests (Milestone M2)
# Compatible with Pester 3.4.0 and Pester 5+

Describe "Safety Guardrails: Confirm-RtbAction" {
    BeforeAll {
        . (Join-Path $PSScriptRoot '..\src\utils\helpers.ps1')
    }

    It "returns `$false when user passes 'n' via pipeline" {
        $result = 'n' | Confirm-RtbAction -Message 'Delete this?'
        $result | Should Be $false
    }

    It "returns `$false when user passes 'N' via pipeline" {
        $result = 'N' | Confirm-RtbAction -Message 'Delete this?'
        $result | Should Be $false
    }

    It "returns `$false when user passes 'no' via pipeline" {
        $result = 'no' | Confirm-RtbAction -Message 'Delete this?'
        $result | Should Be $false
    }

    It "returns `$true when user passes 'y' via pipeline" {
        $result = 'y' | Confirm-RtbAction -Message 'Delete this?'
        $result | Should Be $true
    }

    It "returns `$true when user passes 'Y' via pipeline" {
        $result = 'Y' | Confirm-RtbAction -Message 'Delete this?'
        $result | Should Be $true
    }

    It "returns `$true when user passes 'yes' via pipeline" {
        $result = 'yes' | Confirm-RtbAction -Message 'Delete this?'
        $result | Should Be $true
    }

    It "returns `$true when user passes 'YES' via pipeline" {
        $result = 'YES' | Confirm-RtbAction -Message 'Delete this?'
        $result | Should Be $true
    }

    It "returns `$false for empty string via pipeline" {
        $result = '' | Confirm-RtbAction -Message 'Delete this?'
        $result | Should Be $false
    }

    It "returns `$false for whitespace string via pipeline" {
        $result = '   ' | Confirm-RtbAction -Message 'Delete this?'
        $result | Should Be $false
    }

    It "returns `$false for arbitrary string via pipeline" {
        $result = 'invalid-input' | Confirm-RtbAction -Message 'Delete this?'
        $result | Should Be $false
    }

    It "returns `$true when -Answer 'y' is passed as argument" {
        $result = Confirm-RtbAction -Message 'Delete this?' -Answer 'y'
        $result | Should Be $true
    }

    It "returns `$false when -Answer 'n' is passed as argument" {
        $result = Confirm-RtbAction -Message 'Delete this?' -Answer 'n'
        $result | Should Be $false
    }

    It "prompts Read-Host interactively and returns `$true when user inputs 'y'" {
        Mock Read-Host { 'y' }
        $result = Confirm-RtbAction -Message 'Delete this?'
        $result | Should Be $true
    }

    It "prompts Read-Host interactively and returns `$false when user inputs 'n'" {
        Mock Read-Host { 'n' }
        $result = Confirm-RtbAction -Message 'Delete this?'
        $result | Should Be $false
    }

    It "prompts Read-Host interactively and returns `$false on empty Enter" {
        Mock Read-Host { '' }
        $result = Confirm-RtbAction -Message 'Delete this?'
        $result | Should Be $false
    }
}

Describe "Safety Guardrails: Test-GitClean" {
    BeforeAll {
        . (Join-Path $PSScriptRoot '..\src\utils\helpers.ps1')
        $script:baseTemp = Join-Path ([System.IO.Path]::GetTempPath()) "rtb_git_guard_$([Guid]::NewGuid().ToString('N'))"
        New-Item -ItemType Directory -Path $script:baseTemp -Force | Out-Null
    }

    AfterAll {
        if (Test-Path $script:baseTemp) {
            Remove-Item -Recurse -Force $script:baseTemp -ErrorAction SilentlyContinue
        }
    }

    It "returns `$true for a non-existent path" {
        $nonExistent = Join-Path $script:baseTemp "non_existent_folder"
        $result = Test-GitClean -ProjectPath $nonExistent
        $result | Should Be $true
    }

    It "returns `$true for a directory without a .git directory" {
        $noGitDir = Join-Path $script:baseTemp "no_git_project"
        New-Item -ItemType Directory -Path $noGitDir -Force | Out-Null
        $result = Test-GitClean -ProjectPath $noGitDir
        $result | Should Be $true
    }

    It "returns `$true for a clean git repository" {
        $cleanRepo = Join-Path $script:baseTemp "clean_repo"
        New-Item -ItemType Directory -Path $cleanRepo -Force | Out-Null
        git -C $cleanRepo init --quiet 2>$null
        git -C $cleanRepo config user.name "RTB Test" 2>$null
        git -C $cleanRepo config user.email "test@rtb.local" 2>$null
        Set-Content -Path (Join-Path $cleanRepo "file.txt") -Value "Hello World"
        git -C $cleanRepo add file.txt 2>$null
        git -C $cleanRepo commit -m "initial commit" --quiet 2>$null

        $result = Test-GitClean -ProjectPath $cleanRepo
        $result | Should Be $true
    }

    It "returns `$false for a git repository with untracked files" {
        $dirtyUntracked = Join-Path $script:baseTemp "dirty_untracked"
        New-Item -ItemType Directory -Path $dirtyUntracked -Force | Out-Null
        git -C $dirtyUntracked init --quiet 2>$null
        git -C $dirtyUntracked config user.name "RTB Test" 2>$null
        git -C $dirtyUntracked config user.email "test@rtb.local" 2>$null
        Set-Content -Path (Join-Path $dirtyUntracked "file.txt") -Value "Hello World"
        git -C $dirtyUntracked add file.txt 2>$null
        git -C $dirtyUntracked commit -m "initial commit" --quiet 2>$null

        # Add untracked file
        Set-Content -Path (Join-Path $dirtyUntracked "new_untracked.txt") -Value "untracked"

        $result = Test-GitClean -ProjectPath $dirtyUntracked
        $result | Should Be $false
    }

    It "returns `$false for a git repository with modified tracked files" {
        $dirtyModified = Join-Path $script:baseTemp "dirty_modified"
        New-Item -ItemType Directory -Path $dirtyModified -Force | Out-Null
        git -C $dirtyModified init --quiet 2>$null
        git -C $dirtyModified config user.name "RTB Test" 2>$null
        git -C $dirtyModified config user.email "test@rtb.local" 2>$null
        Set-Content -Path (Join-Path $dirtyModified "file.txt") -Value "Hello World"
        git -C $dirtyModified add file.txt 2>$null
        git -C $dirtyModified commit -m "initial commit" --quiet 2>$null

        # Modify tracked file
        Add-Content -Path (Join-Path $dirtyModified "file.txt") -Value "`nModified line"

        $result = Test-GitClean -ProjectPath $dirtyModified
        $result | Should Be $false
    }

    It "returns `$false for a git repository with staged changes" {
        $dirtyStaged = Join-Path $script:baseTemp "dirty_staged"
        New-Item -ItemType Directory -Path $dirtyStaged -Force | Out-Null
        git -C $dirtyStaged init --quiet 2>$null
        git -C $dirtyStaged config user.name "RTB Test" 2>$null
        git -C $dirtyStaged config user.email "test@rtb.local" 2>$null
        Set-Content -Path (Join-Path $dirtyStaged "file.txt") -Value "Hello World"
        git -C $dirtyStaged add file.txt 2>$null
        git -C $dirtyStaged commit -m "initial commit" --quiet 2>$null

        # Stage a new file without committing
        Set-Content -Path (Join-Path $dirtyStaged "staged.txt") -Value "staged content"
        git -C $dirtyStaged add staged.txt 2>$null

        $result = Test-GitClean -ProjectPath $dirtyStaged
        $result | Should Be $false
    }
}

Describe "Safety Guardrails: Command Integration" {
    BeforeAll {
        . (Join-Path $PSScriptRoot '..\src\utils\helpers.ps1')
        . (Join-Path $PSScriptRoot '..\src\commands\archive.ps1')
        . (Join-Path $PSScriptRoot '..\src\commands\pause.ps1')
        . (Join-Path $PSScriptRoot '..\src\commands\clean.ps1')
    }

    It "Dev-Archive aborts without deleting when git repository is dirty and -Force is not passed" {
        $testArchiveDir = Join-Path ([System.IO.Path]::GetTempPath()) "rtb_archive_test_$([Guid]::NewGuid().ToString('N'))"
        New-Item -ItemType Directory -Path $testArchiveDir -Force | Out-Null
        try {
            git -C $testArchiveDir init --quiet 2>$null
            git -C $testArchiveDir config user.name "RTB Test" 2>$null
            git -C $testArchiveDir config user.email "test@rtb.local" 2>$null
            Set-Content -Path (Join-Path $testArchiveDir "dirty.txt") -Value "dirty"
            Mock Find-ProjectPath { @{ Path = $testArchiveDir; Status = 'Active' } }

            Dev-Archive -Name "test-proj"
            Test-Path $testArchiveDir | Should Be $true
        } finally {
            Remove-Item -Recurse -Force $testArchiveDir -ErrorAction SilentlyContinue
        }
    }

    It "Dev-Archive aborts when confirmation prompt is answered 'n'" {
        $testArchiveDir = Join-Path ([System.IO.Path]::GetTempPath()) "rtb_archive_test_n_$([Guid]::NewGuid().ToString('N'))"
        New-Item -ItemType Directory -Path $testArchiveDir -Force | Out-Null
        try {
            Mock Find-ProjectPath { @{ Path = $testArchiveDir; Status = 'Active' } }
            Mock Read-Host { 'n' }

            Dev-Archive -Name "test-proj"
            Test-Path $testArchiveDir | Should Be $true
        } finally {
            Remove-Item -Recurse -Force $testArchiveDir -ErrorAction SilentlyContinue
        }
    }

    It "Dev-Pause aborts when git repository is dirty and -Force is not passed" {
        $activeRoot = Join-Path ([System.IO.Path]::GetTempPath()) "rtb_pause_active_$([Guid]::NewGuid().ToString('N'))"
        $pausedRoot = Join-Path ([System.IO.Path]::GetTempPath()) "rtb_pause_paused_$([Guid]::NewGuid().ToString('N'))"
        $projectDir = Join-Path $activeRoot "my-dirty-project"
        New-Item -ItemType Directory -Path $projectDir -Force | Out-Null
        New-Item -ItemType Directory -Path $pausedRoot -Force | Out-Null
        try {
            git -C $projectDir init --quiet 2>$null
            git -C $projectDir config user.name "RTB Test" 2>$null
            git -C $projectDir config user.email "test@rtb.local" 2>$null
            Set-Content -Path (Join-Path $projectDir "uncommitted.txt") -Value "dirty"

            Mock Get-DevConfig {
                [PSCustomObject]@{
                    projectRoots = [PSCustomObject]@{
                        active = $activeRoot
                        paused = $pausedRoot
                    }
                    cleanDeps = [PSCustomObject]@{
                        targets = @('node_modules')
                    }
                }
            }
            Mock Get-RtbConfig {
                [PSCustomObject]@{
                    projectRoots = [PSCustomObject]@{
                        active = $activeRoot
                        paused = $pausedRoot
                    }
                    cleanDeps = [PSCustomObject]@{
                        targets = @('node_modules')
                    }
                }
            }

            Dev-Pause -Name "my-dirty-project"
            Test-Path $projectDir | Should Be $true
            Test-Path (Join-Path $pausedRoot "my-dirty-project") | Should Be $false
        } finally {
            Remove-Item -Recurse -Force $activeRoot -ErrorAction SilentlyContinue
            Remove-Item -Recurse -Force $pausedRoot -ErrorAction SilentlyContinue
        }
    }

    It "Rtb-Clean in default dry-run mode does not delete flagged folders" {
        $cleanTestDir = Join-Path ([System.IO.Path]::GetTempPath()) "rtb_clean_test_$([Guid]::NewGuid().ToString('N'))"
        $nodeModulesDir = Join-Path $cleanTestDir "node_modules"
        New-Item -ItemType Directory -Path $nodeModulesDir -Force | Out-Null
        try {
            # Set LastWriteTime to 100 days ago
            (Get-Item $nodeModulesDir).LastWriteTime = (Get-Date).AddDays(-100)

            Mock Get-RtbConfig {
                [PSCustomObject]@{
                    projectRoots = [PSCustomObject]@{
                        active   = $cleanTestDir
                        paused   = $null
                        vibe     = $null
                        sandbox  = $null
                    }
                    cleanDeps = [PSCustomObject]@{
                        targets = @('node_modules')
                    }
                }
            }

            Rtb-Clean
            Test-Path $nodeModulesDir | Should Be $true
        } finally {
            Remove-Item -Recurse -Force $cleanTestDir -ErrorAction SilentlyContinue
        }
    }

    It "Rtb-Clean with -Commit aborts when confirmation is answered 'n'" {
        $cleanTestDir = Join-Path ([System.IO.Path]::GetTempPath()) "rtb_clean_commit_n_$([Guid]::NewGuid().ToString('N'))"
        $nodeModulesDir = Join-Path $cleanTestDir "node_modules"
        New-Item -ItemType Directory -Path $nodeModulesDir -Force | Out-Null
        try {
            (Get-Item $nodeModulesDir).LastWriteTime = (Get-Date).AddDays(-100)

            Mock Get-RtbConfig {
                [PSCustomObject]@{
                    projectRoots = [PSCustomObject]@{
                        active   = $cleanTestDir
                        paused   = $null
                        vibe     = $null
                        sandbox  = $null
                    }
                    cleanDeps = [PSCustomObject]@{
                        targets = @('node_modules')
                    }
                }
            }
            Mock Read-Host { 'n' }

            Rtb-Clean -Commit
            Test-Path $nodeModulesDir | Should Be $true
        } finally {
            Remove-Item -Recurse -Force $cleanTestDir -ErrorAction SilentlyContinue
        }
    }
}

Describe "Safety Guardrails: CLI Dispatcher Integration (rtb / dev)" {
    BeforeAll {
        Import-Module (Join-Path $PSScriptRoot '..\rtb.psd1') -Force
        $global:rtbDispTemp = Join-Path ([System.IO.Path]::GetTempPath()) "rtb_disp_guard_$([Guid]::NewGuid().ToString('N'))"
        New-Item -ItemType Directory -Path $global:rtbDispTemp -Force | Out-Null
    }

    AfterAll {
        if ($global:rtbDispTemp -and (Test-Path $global:rtbDispTemp)) {
            Remove-Item -Recurse -Force $global:rtbDispTemp -ErrorAction SilentlyContinue
        }
    }

    Context "rtb clean and dev clean CLI Dispatch" {
        It "executes 'rtb clean' default dry-run without parameter binding exceptions" {
            $nodeModulesDir = Join-Path $global:rtbDispTemp "node_modules_clean_1"
            New-Item -ItemType Directory -Path $nodeModulesDir -Force | Out-Null
            (Get-Item $nodeModulesDir).LastWriteTime = (Get-Date).AddDays(-100)

            Mock -ModuleName rtb Get-RtbConfig {
                [PSCustomObject]@{
                    projectRoots = [PSCustomObject]@{ active = $global:rtbDispTemp; paused = $null; vibe = $null; sandbox = $null }
                    cleanDeps = [PSCustomObject]@{ targets = @('node_modules_clean_1') }
                }
            }

            { rtb clean } | Should Not Throw
            Test-Path $nodeModulesDir | Should Be $true
        }

        It "executes 'rtb clean -Commit' and prompts for confirmation" {
            $nodeModulesDir = Join-Path $global:rtbDispTemp "node_modules_clean_2"
            New-Item -ItemType Directory -Path $nodeModulesDir -Force | Out-Null
            (Get-Item $nodeModulesDir).LastWriteTime = (Get-Date).AddDays(-100)

            Mock -ModuleName rtb Get-RtbConfig {
                [PSCustomObject]@{
                    projectRoots = [PSCustomObject]@{ active = $global:rtbDispTemp; paused = $null; vibe = $null; sandbox = $null }
                    cleanDeps = [PSCustomObject]@{ targets = @('node_modules_clean_2') }
                }
            }
            Mock -ModuleName rtb Read-Host { 'n' }

            { rtb clean -Commit } | Should Not Throw
            Test-Path $nodeModulesDir | Should Be $true
        }

        It "executes 'rtb clean --commit' (GNU style) without parameter binding error" {
            $nodeModulesDir = Join-Path $global:rtbDispTemp "node_modules_clean_3"
            New-Item -ItemType Directory -Path $nodeModulesDir -Force | Out-Null
            (Get-Item $nodeModulesDir).LastWriteTime = (Get-Date).AddDays(-100)

            Mock -ModuleName rtb Get-RtbConfig {
                [PSCustomObject]@{
                    projectRoots = [PSCustomObject]@{ active = $global:rtbDispTemp; paused = $null; vibe = $null; sandbox = $null }
                    cleanDeps = [PSCustomObject]@{ targets = @('node_modules_clean_3') }
                }
            }
            Mock -ModuleName rtb Read-Host { 'n' }

            { rtb clean --commit } | Should Not Throw
            Test-Path $nodeModulesDir | Should Be $true
        }

        It "executes 'rtb clean 30' with positional day count without type error" {
            $nodeModulesDir = Join-Path $global:rtbDispTemp "node_modules_clean_4"
            New-Item -ItemType Directory -Path $nodeModulesDir -Force | Out-Null
            (Get-Item $nodeModulesDir).LastWriteTime = (Get-Date).AddDays(-40)

            Mock -ModuleName rtb Get-RtbConfig {
                [PSCustomObject]@{
                    projectRoots = [PSCustomObject]@{ active = $global:rtbDispTemp; paused = $null; vibe = $null; sandbox = $null }
                    cleanDeps = [PSCustomObject]@{ targets = @('node_modules_clean_4') }
                }
            }

            { rtb clean 30 } | Should Not Throw
            Test-Path $nodeModulesDir | Should Be $true
        }

        It "executes 'rtb clean 30 -Commit' with positional days and commit flag" {
            $nodeModulesDir = Join-Path $global:rtbDispTemp "node_modules_clean_5"
            New-Item -ItemType Directory -Path $nodeModulesDir -Force | Out-Null
            (Get-Item $nodeModulesDir).LastWriteTime = (Get-Date).AddDays(-40)

            Mock -ModuleName rtb Get-RtbConfig {
                [PSCustomObject]@{
                    projectRoots = [PSCustomObject]@{ active = $global:rtbDispTemp; paused = $null; vibe = $null; sandbox = $null }
                    cleanDeps = [PSCustomObject]@{ targets = @('node_modules_clean_5') }
                }
            }
            Mock -ModuleName rtb Read-Host { 'n' }

            { rtb clean 30 -Commit } | Should Not Throw
            Test-Path $nodeModulesDir | Should Be $true
        }

        It "executes 'dev clean -Commit' through dev alias" {
            $nodeModulesDir = Join-Path $global:rtbDispTemp "node_modules_clean_6"
            New-Item -ItemType Directory -Path $nodeModulesDir -Force | Out-Null
            (Get-Item $nodeModulesDir).LastWriteTime = (Get-Date).AddDays(-100)

            Mock -ModuleName rtb Get-RtbConfig {
                [PSCustomObject]@{
                    projectRoots = [PSCustomObject]@{ active = $global:rtbDispTemp; paused = $null; vibe = $null; sandbox = $null }
                    cleanDeps = [PSCustomObject]@{ targets = @('node_modules_clean_6') }
                }
            }
            Mock -ModuleName rtb Read-Host { 'n' }

            { dev clean -Commit } | Should Not Throw
            Test-Path $nodeModulesDir | Should Be $true
        }
    }

    Context "rtb archive and dev archive CLI Dispatch" {
        It "executes 'rtb archive myproj -Force' without positional parameter error" {
            $testRepo = Join-Path $global:rtbDispTemp "archive_proj_1"
            New-Item -ItemType Directory -Path $testRepo -Force | Out-Null
            git -C $testRepo init --quiet 2>$null
            git -C $testRepo config user.name "RTB Test" 2>$null
            git -C $testRepo config user.email "test@rtb.local" 2>$null
            Set-Content -Path (Join-Path $testRepo "dirty.txt") -Value "uncommitted changes"

            Mock Find-ProjectPath { @{ Path = $testRepo; Status = 'Active' } }
            Mock -ModuleName rtb Find-ProjectPath { @{ Path = $testRepo; Status = 'Active' } }
            Mock Get-RtbConfig { [PSCustomObject]@{ backupRoot = $global:rtbDispTemp; cleanDeps = [PSCustomObject]@{ targets = @() } } }
            Mock -ModuleName rtb Get-RtbConfig { [PSCustomObject]@{ backupRoot = $global:rtbDispTemp; cleanDeps = [PSCustomObject]@{ targets = @() } } }
            Mock Get-DevConfig { [PSCustomObject]@{ backupRoot = $global:rtbDispTemp; cleanDeps = [PSCustomObject]@{ targets = @() } } }
            Mock -ModuleName rtb Get-DevConfig { [PSCustomObject]@{ backupRoot = $global:rtbDispTemp; cleanDeps = [PSCustomObject]@{ targets = @() } } }

            { rtb archive "archive_proj_1" -Force } | Should Not Throw
        }

        It "executes 'rtb archive myproj --force' (GNU style) without error" {
            $testRepo = Join-Path $global:rtbDispTemp "archive_proj_2"
            New-Item -ItemType Directory -Path $testRepo -Force | Out-Null
            git -C $testRepo init --quiet 2>$null
            git -C $testRepo config user.name "RTB Test" 2>$null
            git -C $testRepo config user.email "test@rtb.local" 2>$null
            Set-Content -Path (Join-Path $testRepo "dirty.txt") -Value "uncommitted changes"

            Mock Find-ProjectPath { @{ Path = $testRepo; Status = 'Active' } }
            Mock -ModuleName rtb Find-ProjectPath { @{ Path = $testRepo; Status = 'Active' } }
            Mock Get-RtbConfig { [PSCustomObject]@{ backupRoot = $global:rtbDispTemp; cleanDeps = [PSCustomObject]@{ targets = @() } } }
            Mock -ModuleName rtb Get-RtbConfig { [PSCustomObject]@{ backupRoot = $global:rtbDispTemp; cleanDeps = [PSCustomObject]@{ targets = @() } } }
            Mock Get-DevConfig { [PSCustomObject]@{ backupRoot = $global:rtbDispTemp; cleanDeps = [PSCustomObject]@{ targets = @() } } }
            Mock -ModuleName rtb Get-DevConfig { [PSCustomObject]@{ backupRoot = $global:rtbDispTemp; cleanDeps = [PSCustomObject]@{ targets = @() } } }

            { rtb archive "archive_proj_2" --force } | Should Not Throw
        }

        It "executes 'dev archive myproj -Force' through dev dispatcher" {
            $testRepo = Join-Path $global:rtbDispTemp "archive_proj_3"
            New-Item -ItemType Directory -Path $testRepo -Force | Out-Null
            git -C $testRepo init --quiet 2>$null
            git -C $testRepo config user.name "RTB Test" 2>$null
            git -C $testRepo config user.email "test@rtb.local" 2>$null
            Set-Content -Path (Join-Path $testRepo "dirty.txt") -Value "uncommitted changes"

            Mock Find-ProjectPath { @{ Path = $testRepo; Status = 'Active' } }
            Mock -ModuleName rtb Find-ProjectPath { @{ Path = $testRepo; Status = 'Active' } }
            Mock Get-RtbConfig { [PSCustomObject]@{ backupRoot = $global:rtbDispTemp; cleanDeps = [PSCustomObject]@{ targets = @() } } }
            Mock -ModuleName rtb Get-RtbConfig { [PSCustomObject]@{ backupRoot = $global:rtbDispTemp; cleanDeps = [PSCustomObject]@{ targets = @() } } }
            Mock Get-DevConfig { [PSCustomObject]@{ backupRoot = $global:rtbDispTemp; cleanDeps = [PSCustomObject]@{ targets = @() } } }
            Mock -ModuleName rtb Get-DevConfig { [PSCustomObject]@{ backupRoot = $global:rtbDispTemp; cleanDeps = [PSCustomObject]@{ targets = @() } } }

            { dev archive "archive_proj_3" -Force } | Should Not Throw
        }
    }

    Context "rtb pause and dev pause CLI Dispatch" {
        It "executes 'rtb pause myproj -Force' without positional parameter error" {
            $activeRoot = Join-Path $global:rtbDispTemp "active_pause_1"
            $pausedRoot = Join-Path $global:rtbDispTemp "paused_pause_1"
            $projDir = Join-Path $activeRoot "pause-proj-1"
            New-Item -ItemType Directory -Path $projDir -Force | Out-Null
            New-Item -ItemType Directory -Path $pausedRoot -Force | Out-Null
            git -C $projDir init --quiet 2>$null
            git -C $projDir config user.name "RTB Test" 2>$null
            git -C $projDir config user.email "test@rtb.local" 2>$null
            Set-Content -Path (Join-Path $projDir "dirty.txt") -Value "uncommitted changes"

            Mock Get-RtbConfig {
                [PSCustomObject]@{
                    projectRoots = [PSCustomObject]@{
                        active = (Join-Path $global:rtbDispTemp "active_pause_1")
                        paused = (Join-Path $global:rtbDispTemp "paused_pause_1")
                    }
                    cleanDeps = [PSCustomObject]@{ targets = @() }
                }
            }
            Mock -ModuleName rtb Get-RtbConfig {
                [PSCustomObject]@{
                    projectRoots = [PSCustomObject]@{
                        active = (Join-Path $global:rtbDispTemp "active_pause_1")
                        paused = (Join-Path $global:rtbDispTemp "paused_pause_1")
                    }
                    cleanDeps = [PSCustomObject]@{ targets = @() }
                }
            }
            Mock Get-DevConfig {
                [PSCustomObject]@{
                    projectRoots = [PSCustomObject]@{
                        active = (Join-Path $global:rtbDispTemp "active_pause_1")
                        paused = (Join-Path $global:rtbDispTemp "paused_pause_1")
                    }
                    cleanDeps = [PSCustomObject]@{ targets = @() }
                }
            }
            Mock -ModuleName rtb Get-DevConfig {
                [PSCustomObject]@{
                    projectRoots = [PSCustomObject]@{
                        active = (Join-Path $global:rtbDispTemp "active_pause_1")
                        paused = (Join-Path $global:rtbDispTemp "paused_pause_1")
                    }
                    cleanDeps = [PSCustomObject]@{ targets = @() }
                }
            }

            { rtb pause "pause-proj-1" -Force } | Should Not Throw
            Test-Path (Join-Path $pausedRoot "pause-proj-1") | Should Be $true
        }

        It "executes 'rtb pause myproj -Prune -Force' without parameter error" {
            $activeRoot = Join-Path $global:rtbDispTemp "active_pause_2"
            $pausedRoot = Join-Path $global:rtbDispTemp "paused_pause_2"
            $projDir = Join-Path $activeRoot "pause-proj-2"
            $nodeMod = Join-Path $projDir "node_modules"
            New-Item -ItemType Directory -Path $nodeMod -Force | Out-Null
            New-Item -ItemType Directory -Path $pausedRoot -Force | Out-Null
            git -C $projDir init --quiet 2>$null
            git -C $projDir config user.name "RTB Test" 2>$null
            git -C $projDir config user.email "test@rtb.local" 2>$null
            Set-Content -Path (Join-Path $projDir "dirty.txt") -Value "uncommitted changes"

            Mock Get-RtbConfig {
                [PSCustomObject]@{
                    projectRoots = [PSCustomObject]@{
                        active = (Join-Path $global:rtbDispTemp "active_pause_2")
                        paused = (Join-Path $global:rtbDispTemp "paused_pause_2")
                    }
                    cleanDeps = [PSCustomObject]@{ targets = @('node_modules') }
                }
            }
            Mock -ModuleName rtb Get-RtbConfig {
                [PSCustomObject]@{
                    projectRoots = [PSCustomObject]@{
                        active = (Join-Path $global:rtbDispTemp "active_pause_2")
                        paused = (Join-Path $global:rtbDispTemp "paused_pause_2")
                    }
                    cleanDeps = [PSCustomObject]@{ targets = @('node_modules') }
                }
            }
            Mock Get-DevConfig {
                [PSCustomObject]@{
                    projectRoots = [PSCustomObject]@{
                        active = (Join-Path $global:rtbDispTemp "active_pause_2")
                        paused = (Join-Path $global:rtbDispTemp "paused_pause_2")
                    }
                    cleanDeps = [PSCustomObject]@{ targets = @('node_modules') }
                }
            }
            Mock -ModuleName rtb Get-DevConfig {
                [PSCustomObject]@{
                    projectRoots = [PSCustomObject]@{
                        active = (Join-Path $global:rtbDispTemp "active_pause_2")
                        paused = (Join-Path $global:rtbDispTemp "paused_pause_2")
                    }
                    cleanDeps = [PSCustomObject]@{ targets = @('node_modules') }
                }
            }

            { rtb pause "pause-proj-2" -Prune -Force } | Should Not Throw
            Test-Path (Join-Path $pausedRoot "pause-proj-2") | Should Be $true
            Test-Path (Join-Path $pausedRoot "pause-proj-2\node_modules") | Should Be $false
        }

        It "executes 'dev pause myproj --prune --force' (GNU style) via dev dispatcher" {
            $activeRoot = Join-Path $global:rtbDispTemp "active_pause_3"
            $pausedRoot = Join-Path $global:rtbDispTemp "paused_pause_3"
            $projDir = Join-Path $activeRoot "pause-proj-3"
            $nodeMod = Join-Path $projDir "node_modules"
            New-Item -ItemType Directory -Path $nodeMod -Force | Out-Null
            New-Item -ItemType Directory -Path $pausedRoot -Force | Out-Null
            git -C $projDir init --quiet 2>$null
            git -C $projDir config user.name "RTB Test" 2>$null
            git -C $projDir config user.email "test@rtb.local" 2>$null
            Set-Content -Path (Join-Path $projDir "dirty.txt") -Value "uncommitted changes"

            Mock Get-RtbConfig {
                [PSCustomObject]@{
                    projectRoots = [PSCustomObject]@{
                        active = (Join-Path $global:rtbDispTemp "active_pause_3")
                        paused = (Join-Path $global:rtbDispTemp "paused_pause_3")
                    }
                    cleanDeps = [PSCustomObject]@{ targets = @('node_modules') }
                }
            }
            Mock -ModuleName rtb Get-RtbConfig {
                [PSCustomObject]@{
                    projectRoots = [PSCustomObject]@{
                        active = (Join-Path $global:rtbDispTemp "active_pause_3")
                        paused = (Join-Path $global:rtbDispTemp "paused_pause_3")
                    }
                    cleanDeps = [PSCustomObject]@{ targets = @('node_modules') }
                }
            }
            Mock Get-DevConfig {
                [PSCustomObject]@{
                    projectRoots = [PSCustomObject]@{
                        active = (Join-Path $global:rtbDispTemp "active_pause_3")
                        paused = (Join-Path $global:rtbDispTemp "paused_pause_3")
                    }
                    cleanDeps = [PSCustomObject]@{ targets = @('node_modules') }
                }
            }
            Mock -ModuleName rtb Get-DevConfig {
                [PSCustomObject]@{
                    projectRoots = [PSCustomObject]@{
                        active = (Join-Path $global:rtbDispTemp "active_pause_3")
                        paused = (Join-Path $global:rtbDispTemp "paused_pause_3")
                    }
                    cleanDeps = [PSCustomObject]@{ targets = @('node_modules') }
                }
            }

            { dev pause "pause-proj-3" --prune --force } | Should Not Throw
            Test-Path (Join-Path $pausedRoot "pause-proj-3") | Should Be $true
            Test-Path (Join-Path $pausedRoot "pause-proj-3\node_modules") | Should Be $false
        }
    }
}



