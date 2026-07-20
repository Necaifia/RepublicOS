#!/usr/bin/env pwsh
# Pantheon Agent Wrapper
# Usage: ./pantheon.ps1
# Reads .pantheon/ state, runs OpenCode, checks SUCCESS.md, repeats if work remains.

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$StateFile = Join-Path $ProjectRoot ".pantheon" "PROJECT_STATE.json"
$SuccessFile = Join-Path $ProjectRoot "SUCCESS.md"
$ConstitutionFile = Join-Path $ProjectRoot ".pantheon" "CONSTITUTION.md"

$host.UI.RawUI.WindowTitle = "Pantheon Agent Runtime"

function Test-MissionComplete {
    $state = Get-Content $StateFile -Raw | ConvertFrom-Json
    return $state.done -eq $true
}

function Test-SuccessCriteria {
    $content = Get-Content $SuccessFile -Raw
    # Check if all checkboxes are checked
    $unchecked = [regex]::Matches($content, '- \[ \]').Count
    return $unchecked -eq 0
}

function Update-State {
    param([string]$Cycle, [string]$Summary)
    $state = Get-Content $StateFile -Raw | ConvertFrom-Json
    $state.last_cycle = $Cycle
    if ($Summary) { $state.summary = $Summary }
    $state | ConvertTo-Json | Set-Content $StateFile
}

function Invoke-PantheonCycle {
    Write-Host "`n╔══════════════════════════════════════════════════╗" -ForegroundColor Cyan
    Write-Host "║          Pantheon Cycle Starting               ║" -ForegroundColor Cyan
    Write-Host "╚══════════════════════════════════════════════════╝" -ForegroundColor Cyan
    Write-Host ""

    # Check if opencode is available
    $opencodeCmd = Get-Command "opencode" -ErrorAction SilentlyContinue
    if (-not $opencodeCmd) {
        # Try common locations
        $candidates = @(
            Join-Path $env:LOCALAPPDATA "Programs" "opencode" "opencode.exe",
            Join-Path $env:USERPROFILE ".opencode" "bin" "opencode.exe",
            "opencode.cmd"
        )
        foreach ($c in $candidates) {
            if (Test-Path $c) { $opencodeCmd = $c; break }
        }
    }

    if (-not $opencodeCmd) {
        Write-Host "  ⚠ opencode not found. Install it from https://opencode.ai" -ForegroundColor Yellow
        Write-Host "  For now, manually run this command in your terminal:" -ForegroundColor Yellow
        Write-Host ""
        Write-Host "  Read .pantheon/CONSTITUTION.md. You are Pantheon. Execute." -ForegroundColor Green
        Write-Host ""
        return $false
    }

    $prompt = @"
Read $ConstitutionFile. You are Pantheon. Execute the next cycle.
Do NOT restate your understanding. Do NOT ask questions. Just begin.
"@

    & $opencodeCmd --input $prompt
    return $true
}

# ── Main Loop ──────────────────────────────────────────────────────────

Write-Host "╔══════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║       Pantheon Agent Runtime v0.1.0              ║" -ForegroundColor Cyan
Write-Host "║       Autonomous Engineering Operating System     ║" -ForegroundColor Cyan
Write-Host "╚══════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

if (Test-MissionComplete) {
    Write-Host "  ✓ Mission already marked complete." -ForegroundColor Green
    Write-Host "  Read SUCCESS.md and MISSION.md to set a new mission." -ForegroundColor Yellow
    exit 0
}

$cycleCount = 0
while (-not (Test-MissionComplete)) {
    $cycleCount++
    Write-Host "  Cycle $cycleCount" -ForegroundColor Magenta

    $ran = Invoke-PantheonCycle
    if (-not $ran) { break }

    Write-Host ""
    Write-Host "  ─────────────────────────────────────────────" -ForegroundColor DarkGray

    if (Test-MissionComplete) {
        Write-Host "  ✓ Mission complete! Stopping." -ForegroundColor Green
        break
    }

    if (Test-SuccessCriteria) {
        Write-Host "  ✓ All success criteria met! Marking done." -ForegroundColor Green
        Update-State -Cycle $cycleCount -Summary "All success criteria met. Stopping."
        break
    }

    # Check if there's uncommitted work
    $gitStatus = git -C $ProjectRoot status --porcelain
    if ($gitStatus) {
        Write-Host "  ⚠ Uncommitted changes detected." -ForegroundColor Yellow
        Write-Host "  The agent should have committed. Review manually?" -ForegroundColor Yellow
    }

    $remaining = (Get-Content $SuccessFile | Select-String '- \[ \]').Count
    if ($remaining -eq 0) {
        Write-Host "  ✓ All checkboxes checked. Marking done." -ForegroundColor Green
        Update-State -Cycle $cycleCount -Summary "All SUCCESS.md criteria satisfied."
        break
    }

    Write-Host "  $remaining task(s) remaining. Next cycle in 3s... (Ctrl+C to stop)" -ForegroundColor DarkGray
    Start-Sleep -Seconds 3
}

Write-Host ""
Write-Host "╔══════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║       Pantheon Agent Runtime — Stopped           ║" -ForegroundColor Cyan
Write-Host "╚══════════════════════════════════════════════════╝" -ForegroundColor Cyan
