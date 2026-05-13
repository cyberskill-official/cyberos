# cyberos-weekly.ps1 — Windows weekly maintenance.
#
# Runs Sundays at 02:00 local via Task Scheduler.
#
# Steps:
#   1. cyberos backup     — incremental snapshot to %USERPROFILE%\cyberos-backups\<project>\
#   2. cyberos consolidate — Walk → Compact → Sign → Publish
#   3. determinism guard   — two exports must be byte-identical

param(
    [Parameter(Mandatory = $true)]
    [string]$Project
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path $Project)) {
    Write-Error "project not found: $Project"
    exit 2
}

$ProjectName = Split-Path -Leaf $Project
$BackupRoot  = if ($env:CYBEROS_BACKUP_ROOT) { $env:CYBEROS_BACKUP_ROOT }
               else { Join-Path $env:USERPROFILE "cyberos-backups" }
$BackupTarget = Join-Path $BackupRoot $ProjectName

$LogDir = Join-Path $env:LOCALAPPDATA "cyberos\logs"
New-Item -ItemType Directory -Force -Path $LogDir, $BackupTarget | Out-Null
$Log = Join-Path $LogDir "weekly.log"

function Notify {
    param([string]$Title, [string]$Message)
    if (Get-Module -ListAvailable -Name BurntToast) {
        Import-Module BurntToast
        New-BurntToastNotification -Text $Title, $Message | Out-Null
    }
}

function Timestamp { (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ") }

Set-Location $Project
"=== weekly $(Timestamp) ===" | Add-Content $Log

if (-not (Test-Path .cyberos-memory)) {
    "no .cyberos-memory in $Project — skipping" | Add-Content $Log
    exit 0
}

"→ cyberos backup → $BackupTarget" | Add-Content $Log
python -m cyberos --store .cyberos-memory backup --target $BackupTarget --label "weekly-$(Timestamp)" *>> $Log
if ($LASTEXITCODE -ne 0) {
    "backup: FAIL" | Add-Content $Log
    Notify "cyberos weekly: backup failed" "See $Log"
    exit 1
}

"" | Add-Content $Log
"→ cyberos consolidate" | Add-Content $Log
python -m cyberos --store .cyberos-memory consolidate *>> $Log
if ($LASTEXITCODE -ne 0) {
    "consolidate: FAIL" | Add-Content $Log
    Notify "cyberos weekly: consolidate failed" "See $Log"
    exit 1
}

"" | Add-Content $Log
"→ determinism guard (two exports → byte-identical?)" | Add-Content $Log
$Scratch = New-Item -ItemType Directory -Path (Join-Path $env:TEMP "cyberos-det-$(Get-Random)")
try {
    $A = Join-Path $Scratch "a.zip"
    $B = Join-Path $Scratch "b.zip"
    python -m cyberos --store .cyberos-memory export $A *>> $Log
    python -m cyberos --store .cyberos-memory export $B *>> $Log
    $shaA = (Get-FileHash $A -Algorithm SHA256).Hash
    $shaB = (Get-FileHash $B -Algorithm SHA256).Hash
    if ($shaA -eq $shaB) {
        "determinism: OK (sha256=$shaA)" | Add-Content $Log
    } else {
        "determinism: REGRESSION ($shaA vs $shaB)" | Add-Content $Log
        Notify "cyberos weekly: NON-DETERMINISTIC EXPORT" "Export round-trip diverged. See $Log"
        exit 1
    }
} finally {
    Remove-Item -Recurse -Force $Scratch
}
