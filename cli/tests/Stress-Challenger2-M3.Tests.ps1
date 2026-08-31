# Stress & Edge Case Tests for Milestone M3 (Challenger 2)
# Tests: Navigation UX, Agent Switches, Picker Selection Bounds, Polyglot Dependency Detection
# Compatible with Pester 3.4.0 and Pester 5+

Import-Module (Join-Path $PSScriptRoot '..\rtb.psd1') -Force

InModuleScope rtb {
    Describe "Milestone M3 Stress: Navigation & Agent Switches (goto.ps1)" {
        BeforeAll {
            $script:stressTemp = Join-Path ([System.IO.Path]::GetTempPath()) "rtb_m3_stress_$([Guid]::NewGuid().ToString('N'))"
            $script:activeDir = Join-Path $script:stressTemp "active"
            $script:pausedDir = Join-Path $script:stressTemp "paused"
            New-Item -ItemType Directory -Path $script:activeDir -Force | Out-Null
            New-Item -ItemType Directory -Path $script:pausedDir -Force | Out-Null

            $script:proj1 = Join-Path $script:activeDir "alpha-service"
            $script:proj2 = Join-Path $script:activeDir "beta-frontend"
            $script:proj3 = Join-Path $script:activeDir "beta-backend"
            $script:proj4 = Join-Path $script:activeDir "beta-shared"
            New-Item -ItemType Directory -Path $script:proj1 -Force | Out-Null
            New-Item -ItemType Directory -Path $script:proj2 -Force | Out-Null
            New-Item -ItemType Directory -Path $script:proj3 -Force | Out-Null
            New-Item -ItemType Directory -Path $script:proj4 -Force | Out-Null

            $script:origLoc = (Get-Location).Path
        }

        AfterAll {
            Set-Location $script:origLoc
            if ($script:stressTemp -and (Test-Path $script:stressTemp)) {
                Remove-Item -Recurse -Force $script:stressTemp -ErrorAction SilentlyContinue
            }
        }

        Context "Agent Switch Resolution & Execution" {
            It "resolves each supported agent switch properly" {
                $switchMap = @{
                    '-Agy'       = 'agy'
                    '-Claude'    = 'claude'
                    '-Gemini'    = 'gemini'
                    '-Codex'     = 'codex'
                    '-Cursor'    = 'cursor'
                    '-Windsurf'  = 'windsurf'
                    '-Aider'     = 'aider'
                    '-OpenHands' = 'openhands'
                }

                Mock Find-ProjectPathFuzzy {
                    @([PSCustomObject]@{ Name = "alpha-service"; Path = $script:proj1; Status = "Active"; Score = 100 })
                }

                foreach ($sw in $switchMap.Keys) {
                    $expectedAgent = $switchMap[$sw]
                    $script:capturedAgent = $null
                    $script:capturedPath = $null

                    Mock Rtb-Agent {
                        param($ProjectName, $Agent)
                        $script:capturedAgent = $Agent
                        $script:capturedPath = $ProjectName
                    }

                    $paramHash = @{ Name = "alpha-service" }
                    $paramHash[$sw.TrimStart('-')] = $true
                    Dev-Goto @paramHash

                    $script:capturedAgent | Should -Be $expectedAgent
                    $script:capturedPath | Should -Be $script:proj1
                }
            }

            It "resolves positional Agent argument when supplied" {
                Mock Find-ProjectPathFuzzy {
                    @([PSCustomObject]@{ Name = "alpha-service"; Path = $script:proj1; Status = "Active"; Score = 100 })
                }
                $script:capturedAgent = $null
                Mock Rtb-Agent {
                    param($ProjectName, $Agent)
                    $script:capturedAgent = $Agent
                }

                Dev-Goto -Name "alpha-service" -Agent "gemini"
                $script:capturedAgent | Should -Be "gemini"
            }

            It "resolves positional syntax 'Dev-Goto alpha-service codex'" {
                Mock Find-ProjectPathFuzzy {
                    @([PSCustomObject]@{ Name = "alpha-service"; Path = $script:proj1; Status = "Active"; Score = 100 })
                }
                $script:capturedAgent = $null
                Mock Rtb-Agent {
                    param($ProjectName, $Agent)
                    $script:capturedAgent = $Agent
                }

                Dev-Goto "alpha-service" "codex"
                $script:capturedAgent | Should -Be "codex"
            }

            It "preserves agent execution after interactive disambiguation picker choice" {
                Set-Location $script:stressTemp
                Mock Find-ProjectPathFuzzy {
                    @(
                        [PSCustomObject]@{ Name = "beta-frontend"; Path = $script:proj2; Status = "Active"; Score = 75 },
                        [PSCustomObject]@{ Name = "beta-backend";  Path = $script:proj3; Status = "Active"; Score = 75 }
                    )
                }
                $script:capturedAgent = $null
                $script:capturedPath = $null
                Mock Rtb-Agent {
                    param($ProjectName, $Agent)
                    $script:capturedAgent = $Agent
                    $script:capturedPath = $ProjectName
                }

                # User chooses option 2 (beta-backend) with -Claude switch
                Dev-Goto -Name "beta" -Choice "2" -Claude
                (Get-Location).Path | Should -Be $script:proj3
                $script:capturedAgent | Should -Be "claude"
                $script:capturedPath | Should -Be $script:proj3
            }

            It "dispatches 'rtb goto alpha-service -Claude' via top-level rtb command" {
                Mock Find-ProjectPathFuzzy {
                    @([PSCustomObject]@{ Name = "alpha-service"; Path = $script:proj1; Status = "Active"; Score = 100 })
                }
                $script:capturedAgent = $null
                Mock Rtb-Agent {
                    param($ProjectName, $Agent)
                    $script:capturedAgent = $Agent
                }

                rtb goto alpha-service -Claude
                $script:capturedAgent | Should -Be "claude"
            }

            It "dispatches 'dev goto alpha-service -Agy' via dev alias" {
                Mock Find-ProjectPathFuzzy {
                    @([PSCustomObject]@{ Name = "alpha-service"; Path = $script:proj1; Status = "Active"; Score = 100 })
                }
                $script:capturedAgent = $null
                Mock Rtb-Agent {
                    param($ProjectName, $Agent)
                    $script:capturedAgent = $Agent
                }

                dev goto alpha-service -Agy
                $script:capturedAgent | Should -Be "agy"
            }
        }

        Context "Multi-Match Picker Selection Bounds & Edge Cases" {
            BeforeEach {
                Set-Location $script:stressTemp
                Mock Find-ProjectPathFuzzy {
                    @(
                        [PSCustomObject]@{ Name = "beta-frontend"; Path = $script:proj2; Status = "Active"; Score = 75 },
                        [PSCustomObject]@{ Name = "beta-backend";  Path = $script:proj3; Status = "Active"; Score = 75 },
                        [PSCustomObject]@{ Name = "beta-shared";   Path = $script:proj4; Status = "Active"; Score = 75 }
                    )
                }
            }

            It "selects first item with choice '1'" {
                Dev-Goto -Name "beta" -Choice "1"
                (Get-Location).Path | Should -Be $script:proj2
            }

            It "selects third item with choice '3'" {
                Dev-Goto -Name "beta" -Choice "3"
                (Get-Location).Path | Should -Be $script:proj4
            }

            It "cancels when choice is '0' (zero is out of 1-based bounds)" {
                $output = (Dev-Goto -Name "beta" -Choice "0" *>&1 | Out-String)
                $output | Should -Match "Cancelled\."
                (Get-Location).Path | Should -Be $script:stressTemp
            }

            It "cancels when choice is '4' (greater than match count 3)" {
                $output = (Dev-Goto -Name "beta" -Choice "4" *>&1 | Out-String)
                $output | Should -Match "Cancelled\."
                (Get-Location).Path | Should -Be $script:stressTemp
            }

            It "cancels when choice is '9' (upper digit bound but beyond match count)" {
                $output = (Dev-Goto -Name "beta" -Choice "9" *>&1 | Out-String)
                $output | Should -Match "Cancelled\."
                (Get-Location).Path | Should -Be $script:stressTemp
            }

            It "cancels when choice is negative '-1'" {
                $output = (Dev-Goto -Name "beta" -Choice "-1" *>&1 | Out-String)
                $output | Should -Match "Cancelled\."
                (Get-Location).Path | Should -Be $script:stressTemp
            }

            It "cancels when choice is arbitrary non-numeric text 'cancel'" {
                $output = (Dev-Goto -Name "beta" -Choice "cancel" *>&1 | Out-String)
                $output | Should -Match "Cancelled\."
                (Get-Location).Path | Should -Be $script:stressTemp
            }

            It "cancels when choice is special characters '!@#'" {
                $output = (Dev-Goto -Name "beta" -Choice "!@#" *>&1 | Out-String)
                $output | Should -Match "Cancelled\."
                (Get-Location).Path | Should -Be $script:stressTemp
            }

            It "cancels when choice is empty string ''" {
                Mock Read-Host { "" }
                $output = (Dev-Goto -Name "beta" *>&1 | Out-String)
                $output | Should -Match "Cancelled\."
                (Get-Location).Path | Should -Be $script:stressTemp
            }

            It "cancels when choice is whitespace '   '" {
                Mock Read-Host { "   " }
                $output = (Dev-Goto -Name "beta" *>&1 | Out-String)
                $output | Should -Match "Cancelled\."
                (Get-Location).Path | Should -Be $script:stressTemp
            }
        }

        Context "Picker with Large Result Sets (>9 matches)" {
            BeforeAll {
                $script:largeMatches = @()
                for ($i = 1; $i -le 15; $i++) {
                    $p = Join-Path $script:stressTemp "item-$i"
                    New-Item -ItemType Directory -Path $p -Force | Out-Null
                    $script:largeMatches += [PSCustomObject]@{
                        Name   = "item-$i"
                        Path   = $p
                        Status = "Active"
                        Score  = 50
                    }
                }
            }

            It "handles choice within displayed 1..9 range" {
                Set-Location $script:stressTemp
                Mock Find-ProjectPathFuzzy { $script:largeMatches }

                Dev-Goto -Name "item" -Choice "5"
                (Get-Location).Path | Should -Be (Join-Path $script:stressTemp "item-5")
            }

            It "cancels on out-of-range choice '99'" {
                Set-Location $script:stressTemp
                Mock Find-ProjectPathFuzzy { $script:largeMatches }

                $output = (Dev-Goto -Name "item" -Choice "99" *>&1 | Out-String)
                $output | Should -Match "Cancelled\."
                (Get-Location).Path | Should -Be $script:stressTemp
            }
        }
    }

    Describe "Milestone M3 Stress: Polyglot Dependency Detection & Context Generation" {
        BeforeAll {
            $script:polyglotRoot = Join-Path ([System.IO.Path]::GetTempPath()) "rtb_polyglot_$([Guid]::NewGuid().ToString('N'))"
            New-Item -ItemType Directory -Path $script:polyglotRoot -Force | Out-Null
        }

        AfterAll {
            if ($script:polyglotRoot -and (Test-Path $script:polyglotRoot)) {
                Remove-Item -Recurse -Force $script:polyglotRoot -ErrorAction SilentlyContinue
            }
        }

        Context "Polyglot Repo: Node.js + Rust (package.json + Cargo.toml)" {
            It "extracts both package.json deps and Cargo.toml crates into .rtb_context.md" {
                $dir = Join-Path $script:polyglotRoot "node_rust_combo"
                New-Item -ItemType Directory -Path $dir -Force | Out-Null

                $pkg = @'
{
  "name": "fullstack-hybrid",
  "dependencies": {
    "next": "^14.0.0",
    "react": "^18.2.0",
    "tailwindcss": "^3.3.0"
  },
  "devDependencies": {
    "typescript": "^5.3.0",
    "@types/react": "^18.2.0"
  }
}
'@
                Set-Content -Path (Join-Path $dir "package.json") -Value $pkg

                $cargo = @'
[package]
name = "hybrid_backend"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.7"
tokio = { version = "1.36", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tower-http = { version = "0.5", features = ["cors"] }
'@
                Set-Content -Path (Join-Path $dir "Cargo.toml") -Value $cargo

                $ctxPath = New-RtbAgentContextFile -ProjectPath $dir -ProjectName "fullstack-hybrid"
                $ctxPath | Should -Not -BeNullOrEmpty
                Test-Path $ctxPath | Should -Be $true

                $content = Get-Content -Path $ctxPath -Raw
                $content | Should -Match "\*\*package\.json deps:\*\* next, react, tailwindcss"
                $content | Should -Match "\*\*devDependencies:\*\* typescript, @types/react"
                $content | Should -Match "\*\*Cargo\.toml crates:\*\* name, version, edition, axum, tokio, serde, serde_json, tower-http"
            }
        }

        Context "Polyglot Repo: Python + Go (requirements.txt + go.mod)" {
            It "extracts both Python requirements and Go modules into .rtb_context.md" {
                $dir = Join-Path $script:polyglotRoot "py_go_combo"
                New-Item -ItemType Directory -Path $dir -Force | Out-Null

                $reqs = @'
# Machine Learning dependencies
torch>=2.1.0
transformers==4.35.0
fastapi>=0.104.0
uvicorn[standard]>=0.24.0
# Utilities
pydantic>=2.5.0
'@
                Set-Content -Path (Join-Path $dir "requirements.txt") -Value $reqs

                $gomod = @'
module github.com/myorg/microservice

go 1.22.0

require (
	github.com/gin-gonic/gin v1.9.1
	github.com/google/uuid v1.6.0
	github.com/redis/go-redis/v9 v9.5.1
	go.uber.org/zap v1.27.0
)
'@
                Set-Content -Path (Join-Path $dir "go.mod") -Value $gomod

                $ctxPath = New-RtbAgentContextFile -ProjectPath $dir -ProjectName "py_go_combo"
                $content = Get-Content -Path $ctxPath -Raw

                $content | Should -Match "\*\*requirements\.txt:\*\* torch>=2\.1\.0, transformers==4\.35\.0, fastapi>=0\.104\.0, uvicorn\[standard\]>=0\.24\.0, pydantic>=2\.5\.0"
                $content | Should -Match "\*\*go\.mod requires:\*\* github\.com/gin-gonic/gin v1\.9\.1, github\.com/google/uuid v1\.6\.0, github\.com/redis/go-redis/v9 v9\.5\.1, go\.uber\.org/zap v1\.27\.0"
            }
        }

        Context "Polyglot Quad Repo (All 4 Manifests Present)" {
            It "captures all 4 manifest summaries cleanly without cross-contamination" {
                $dir = Join-Path $script:polyglotRoot "quad_combo"
                New-Item -ItemType Directory -Path $dir -Force | Out-Null

                Set-Content -Path (Join-Path $dir "package.json") -Value '{"dependencies": {"vue": "^3.3.0"}}'
                Set-Content -Path (Join-Path $dir "Cargo.toml") -Value "ratatui = `"0.29`"`ncrossterm = `"0.28`""
                Set-Content -Path (Join-Path $dir "requirements.txt") -Value "numpy>=1.26.0`npandas>=2.1.0"
                Set-Content -Path (Join-Path $dir "go.mod") -Value "module app`n`nrequire (`n`tgithub.com/gorilla/mux v1.8.1`n)"

                $ctxPath = New-RtbAgentContextFile -ProjectPath $dir -ProjectName "quad_combo"
                $content = Get-Content -Path $ctxPath -Raw

                $content | Should -Match "\*\*package\.json deps:\*\* vue"
                $content | Should -Match "\*\*Cargo\.toml crates:\*\* ratatui, crossterm"
                $content | Should -Match "\*\*requirements\.txt:\*\* numpy>=1\.26\.0, pandas>=2\.1\.0"
                $content | Should -Match "\*\*go\.mod requires:\*\* github\.com/gorilla/mux v1\.8\.1"
            }
        }

        Context "Manifest Edge Cases & Malformed Files" {
            It "handles malformed package.json gracefully with error placeholder" {
                $dir = Join-Path $script:polyglotRoot "malformed_json"
                New-Item -ItemType Directory -Path $dir -Force | Out-Null
                Set-Content -Path (Join-Path $dir "package.json") -Value "{ invalid json content, missing quotes }"

                $ctxPath = New-RtbAgentContextFile -ProjectPath $dir -ProjectName "malformed_json"
                $content = Get-Content -Path $ctxPath -Raw

                $content | Should -Match "\(could not parse package\.json\)"
            }

            It "handles empty package.json with no dependencies" {
                $dir = Join-Path $script:polyglotRoot "empty_json"
                New-Item -ItemType Directory -Path $dir -Force | Out-Null
                Set-Content -Path (Join-Path $dir "package.json") -Value "{}"

                $ctxPath = New-RtbAgentContextFile -ProjectPath $dir -ProjectName "empty_json"
                $content = Get-Content -Path $ctxPath -Raw

                $content | Should -Match "\(no recognised dependency manifest found\)"
            }

            It "filters comments and blank lines in requirements.txt" {
                $dir = Join-Path $script:polyglotRoot "commented_reqs"
                New-Item -ItemType Directory -Path $dir -Force | Out-Null
                $reqs = @"
# Header comment
# Another comment

scikit-learn>=1.3.0

# Middle comment
scipy>=1.11.0

"@
                Set-Content -Path (Join-Path $dir "requirements.txt") -Value $reqs

                $ctxPath = New-RtbAgentContextFile -ProjectPath $dir -ProjectName "commented_reqs"
                $content = Get-Content -Path $ctxPath -Raw

                $content | Should -Match "\*\*requirements\.txt:\*\* scikit-learn>=1\.3\.0, scipy>=1\.11\.0"
                $content -notmatch "# Header comment" | Should -Be $true
            }

            It "handles go.mod with inline single require line" {
                $dir = Join-Path $script:polyglotRoot "single_gomod"
                New-Item -ItemType Directory -Path $dir -Force | Out-Null
                $gomod = "module mylib`n`ngo 1.21`n`nrequire github.com/stretchr/testify v1.8.4`n"
                Set-Content -Path (Join-Path $dir "go.mod") -Value $gomod

                # Note: go.mod requires block or single require line
                $ctxPath = New-RtbAgentContextFile -ProjectPath $dir -ProjectName "single_gomod"
                $content = Get-Content -Path $ctxPath -Raw
                # Should generate context file without error
                Test-Path $ctxPath | Should -Be $true
            }
        }
    }
}
