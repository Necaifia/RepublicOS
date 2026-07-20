<#
.SYNOPSIS
    Installs the RepublicOS Skill Engine as a daily scheduled task.

.DESCRIPTION
    Creates a Windows Scheduled Task to run the skill evolution pipeline daily.
    Uses the same Python interpreter that ran this script.

.PARAMETER TaskName
    Name of the scheduled task (default: RepublicOS-SkillEngine)

.PARAMETER Time
    Daily run time in HH:mm format (default: 03:00)

.PARAMETER PythonPath
    Path to Python executable (default: auto-detected)

.PARAMETER Stages
    Comma-separated stages to run (default: all)

.PARAMETER Full
    Force full rediscovery on each run

.PARAMETER Remove
    Remove the scheduled task instead of installing

.EXAMPLE
    .\install-scheduler.ps1
    .\install-scheduler.ps1 -Time "05:30" -Full
    .\install-scheduler.ps1 -Remove
#>

param(
    [string]$TaskName = "RepublicOS-SkillEngine",
    [string]$Time = "03:00",
    [string]$PythonPath = "",
    [string]$Stages = "",
    [switch]$Full,
    [switch]$Remove
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$SchedulerScript = Join-Path $ScriptDir "scheduler.py"

if (-not (Test-Path $SchedulerScript)) {
    Write-Error "scheduler.py not found in $ScriptDir"
    exit 1
}

if ($Remove) {
    Write-Host "Removing scheduled task '$TaskName'..."
    Unregister-ScheduledTask -TaskName $TaskName -Confirm:$false -ErrorAction SilentlyContinue
    if ($?) {
        Write-Host "Task '$TaskName' removed successfully."
    } else {
        Write-Host "Task '$TaskName' not found or already removed."
    }
    exit 0
}

if (-not $PythonPath) {
    $PythonPath = (Get-Command python).Source
}

if (-not $PythonPath -or -not (Test-Path $PythonPath)) {
    Write-Error "Python not found. Specify -PythonPath or ensure python is in PATH."
    exit 1
}

$Arguments = "`"$SchedulerScript`" run"
if ($Stages) {
    $StageArgs = ($Stages -split ",").Trim() -join " "
    $Arguments += " --stages $StageArgs"
}
if ($Full) {
    $Arguments += " --full"
}

$Action = New-ScheduledTaskAction -Execute $PythonPath -Argument $Arguments
$Trigger = New-ScheduledTaskTrigger -Daily -At $Time
$Principal = New-ScheduledTaskPrincipal -UserId "SYSTEM" -LogonType ServiceAccount -RunLevel Highest
$Settings = New-ScheduledTaskSettingsSet -StartWhenAvailable -DontStopOnIdleEnd -AllowStartIfOnBatteries

try {
    Register-ScheduledTask -TaskName $TaskName `
        -Action $Action `
        -Trigger $Trigger `
        -Principal $Principal `
        -Settings $Settings `
        -Description "RepublicOS Skill Engine - Daily skill discovery and evolution" `
        -Force

    Write-Host "Task '$TaskName' installed successfully."
    Write-Host "  Python: $PythonPath"
    Write-Host "  Script: $SchedulerScript"
    Write-Host "  Time:   $Time daily"
    Write-Host "  Stages: $(if ($Stages) { $Stages } else { 'all' })"
    if ($Full) { Write-Host "  Mode:   Full rediscovery" }
    Write-Host ""
    Write-Host "You can also run manually:"
    Write-Host "  Start-ScheduledTask -TaskName '$TaskName'"
    Write-Host ""
    Write-Host "To remove:"
    Write-Host "  .\install-scheduler.ps1 -Remove"
} catch {
    Write-Error "Failed to install scheduled task: $_"
    exit 1
}
