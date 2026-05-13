# scripts/automation-install.ps1 — Windows Task Scheduler automation.
#
# Installs two scheduled tasks:
#   - CyberosNightly  → daily 01:09 local
#   - CyberosWeekly   → Sundays 02:07 local
#
# Logs land at %USERPROFILE%\AppData\Local\cyberos\logs\ .
#
# Usage (run in a regular PowerShell window):
#     .\scripts\automation-install.ps1 -Target "C:\Projects\my-project"
#     .\scripts\automation-install.ps1 -Uninstall

[CmdletBinding()]
param(
    [string]$Target = (Get-Location).Path,
    [switch]$Uninstall
)

$ErrorActionPreference = "Stop"
$Repo = (Split-Path -Parent $PSScriptRoot)
$NightlyTaskName = "CyberosNightly"
$WeeklyTaskName  = "CyberosWeekly"

function Test-Job($Name) {
    $null -ne (Get-ScheduledTask -TaskName $Name -ErrorAction SilentlyContinue)
}

if ($Uninstall) {
    Write-Host "=== uninstalling Windows Task Scheduler jobs ==="
    foreach ($name in @($NightlyTaskName, $WeeklyTaskName)) {
        if (Test-Job $name) {
            Unregister-ScheduledTask -TaskName $name -Confirm:$false
            Write-Host "  removed: $name"
        }
    }
    Write-Host "done"
    return
}

if (-not (Test-Path $Target)) {
    Write-Error "target not found: $Target"
    exit 2
}
$Target = (Resolve-Path $Target).Path
if (-not (Test-Path (Join-Path $Target ".cyberos-memory"))) {
    Write-Error "target has no .cyberos-memory/: $Target"
    exit 2
}

Write-Host "=== installing Windows Task Scheduler jobs ==="
Write-Host "  target  : $Target"
Write-Host "  scripts : $Repo\scripts\automation\"
Write-Host "  logs    : $env:LOCALAPPDATA\cyberos\logs"
Write-Host ""

$pwsh = (Get-Command pwsh -ErrorAction SilentlyContinue).Path
if (-not $pwsh) { $pwsh = (Get-Command powershell).Path }

function New-Job {
    param(
        [string]$Name,
        [string]$Script,
        [DateTime]$When,
        [string]$Cadence  # 'Daily' or 'Weekly'
    )
    if (Test-Job $Name) {
        Unregister-ScheduledTask -TaskName $Name -Confirm:$false
    }
    $action = New-ScheduledTaskAction `
        -Execute $pwsh `
        -Argument "-NoProfile -ExecutionPolicy Bypass -File `"$Script`" -Project `"$Target`""
    $trigger = if ($Cadence -eq 'Daily') {
        New-ScheduledTaskTrigger -Daily -At $When
    } else {
        New-ScheduledTaskTrigger -Weekly -DaysOfWeek Sunday -At $When
    }
    $settings = New-ScheduledTaskSettingsSet `
        -AllowStartIfOnBatteries `
        -DontStopIfGoingOnBatteries `
        -StartWhenAvailable
    Register-ScheduledTask `
        -TaskName $Name `
        -Action $action `
        -Trigger $trigger `
        -Settings $settings `
        -Description "CyberOS automation: $Name" | Out-Null
    Write-Host "  registered: $Name"
}

New-Job -Name $NightlyTaskName `
    -Script (Join-Path $Repo "scripts\automation\cyberos-nightly.ps1") `
    -When (Get-Date "01:09") -Cadence Daily

New-Job -Name $WeeklyTaskName `
    -Script (Join-Path $Repo "scripts\automation\cyberos-weekly.ps1") `
    -When (Get-Date "02:07") -Cadence Weekly

Write-Host ""
Write-Host "next runs:"
Get-ScheduledTask -TaskName $NightlyTaskName, $WeeklyTaskName | `
    Select-Object TaskName, @{N='NextRun';E={(Get-ScheduledTaskInfo $_).NextRunTime}} | `
    Format-Table -AutoSize

Write-Host ""
Write-Host "to run one immediately:"
Write-Host "  Start-ScheduledTask -TaskName $NightlyTaskName"
Write-Host "  Get-Content `"$env:LOCALAPPDATA\cyberos\logs\nightly.log`" -Tail 30"
Write-Host ""
Write-Host "to remove:"
Write-Host "  .\scripts\automation-install.ps1 -Uninstall"
