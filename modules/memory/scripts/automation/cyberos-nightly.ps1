# cyberos-nightly.ps1 — Windows nightly soak.
#
# Wired by scripts/automation-install.ps1 as a Windows Task Scheduler
# job. Runs on the host (not in any agent sandbox), against the real
# BRAIN.
#
# Steps:
#   1. cyberos doctor — must return overall OK
#   2. cyberos consolidate --dry-run — confirms a real consolidate
#      would proceed cleanly
#
# Output goes to %USERPROFILE%\AppData\Local\cyberos\logs\nightly.log .
# On regression a balloon notification posts via BurntToast if
# available; otherwise silent (the log is the source of truth).

param(
    [Parameter(Mandatory = $true)]
    [string]$Project
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path $Project)) {
    Write-Error "project not found: $Project"
    exit 2
}

$LogDir = Join-Path $env:LOCALAPPDATA "cyberos\logs"
New-Item -ItemType Directory -Force -Path $LogDir | Out-Null
$Log = Join-Path $LogDir "nightly.log"

function Notify {
    param([string]$Title, [string]$Message)
    if (Get-Module -ListAvailable -Name BurntToast) {
        Import-Module BurntToast
        New-BurntToastNotification -Text $Title, $Message | Out-Null
    }
}

function Timestamp { (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ") }

Set-Location $Project
"=== nightly $(Timestamp) ===" | Add-Content $Log

if (-not (Test-Path .cyberos-memory)) {
    "no .cyberos-memory in $Project — skipping" | Add-Content $Log
    exit 0
}

"→ cyberos doctor" | Add-Content $Log
python -m cyberos --store .cyberos-memory doctor *>> $Log
if ($LASTEXITCODE -ne 0) {
    "doctor: FAIL (exit $LASTEXITCODE)" | Add-Content $Log
    Notify "cyberos nightly: FAIL" "cyberos doctor reported errors. See $Log"
    exit 1
}
"doctor: OK" | Add-Content $Log

"" | Add-Content $Log
"→ cyberos consolidate --dry-run" | Add-Content $Log
python -m cyberos --store .cyberos-memory consolidate --dry-run *>> $Log
if ($LASTEXITCODE -ne 0) {
    "consolidate dry-run: FAIL" | Add-Content $Log
    Notify "cyberos nightly: consolidate dry-run failed" "See $Log"
    exit 1
}
"consolidate dry-run: OK" | Add-Content $Log
