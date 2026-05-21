# scripts/install.ps1 — drop the cyberos protocol into a fresh project (Windows).
#
# Counterpart of scripts/install.sh. Same six-phase install; uses
# PowerShell idioms (no symlinks by default — symlinks require dev mode
# or admin on Windows, so we copy AGENTS.md instead).
#
# Usage (from a regular PowerShell window):
#     .\scripts\install.ps1 -Target "C:\Projects\my-project"
#     .\scripts\install.ps1 -Target "C:\Projects\my-project" -WithAutomation -WithPreCommit

[CmdletBinding()]
param(
    [string]$Target = (Get-Location).Path,
    [switch]$WithAutomation,
    [switch]$WithPreCommit,
    [switch]$Force
)

$ErrorActionPreference = "Stop"
$Repo = (Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot)))
$MemModule = Join-Path $Repo "modules\memory"
$Target = (Resolve-Path $Target).Path

Write-Host "=== cyberos install ==="
Write-Host "  source : $Repo"
Write-Host "  target : $Target"
Write-Host ""

# ---------------------------------------------------------------------- 1. python deps

Write-Host "→ step 1/6: python dependencies"
$pip = (Get-Command pip -ErrorAction SilentlyContinue).Path
if (-not $pip) {
    Write-Error "pip not on PATH — install Python ≥ 3.11 first (https://python.org)"
    exit 1
}
& pip install --quiet msgspec cryptography crc32c rfc8785 'pyyaml>=6' jsonschema zstandard
Write-Host "  ✓ msgspec, cryptography, crc32c, rfc8785, pyyaml, jsonschema, zstandard"
Write-Host ""

# ---------------------------------------------------------------------- 2. pandoc (optional)

Write-Host "→ step 2/6: pandoc (optional, for docx round-trip)"
if (Get-Command pandoc -ErrorAction SilentlyContinue) {
    Write-Host "  ✓ pandoc present"
} else {
    Write-Host "  – pandoc not found; install via 'winget install JohnMacFarlane.Pandoc' if needed"
}
Write-Host ""

# ---------------------------------------------------------------------- 3. protocol files

Write-Host "→ step 3/6: install protocol files"
$dst = Join-Path $Target "memory\docs"
New-Item -ItemType Directory -Force -Path $dst | Out-Null
# AGENTS.md lives at repo root; schema/invariants live in the memory module
Copy-Item -Path (Join-Path $Repo "AGENTS.md") -Destination (Join-Path $dst "AGENTS.md") -Force
Write-Host "  ✓ AGENTS.md"
foreach ($f in @("memory.schema.json", "memory.invariants.yaml")) {
    $src = Join-Path $MemModule $f
    $dst_file = Join-Path $dst $f
    if ((Test-Path $dst_file) -and -not $Force) {
        Write-Host "  – $f exists; use -Force to overwrite"
    } else {
        Copy-Item -Path $src -Destination $dst_file -Force
        Write-Host "  ✓ $dst_file"
    }
}
$cyberos_dst = Join-Path $Target "memory\cyberos"
if ((-not (Test-Path $cyberos_dst)) -or $Force) {
    New-Item -ItemType Directory -Force -Path (Join-Path $Target "memory") | Out-Null
    Copy-Item -Path (Join-Path $MemModule "cyberos") -Destination $cyberos_dst -Recurse -Force
    Write-Host "  ✓ $cyberos_dst"
}
Write-Host ""

# ---------------------------------------------------------------------- 4. .cyberos-memory skeleton

Write-Host "→ step 4/6: initialise .cyberos-memory"
$Brain = Join-Path $Target ".cyberos-memory"
if ((Test-Path $Brain) -and -not $Force) {
    Write-Host "  – $Brain exists; use -Force to re-init"
} else {
    $subs = @(
        "audit",
        "memories\decisions", "memories\facts", "memories\people",
        "memories\projects", "memories\preferences", "memories\drift", "memories\refinements",
        "meta", "company", "module", "member", "client", "project", "persona",
        "conflicts", "exports", "index"
    )
    foreach ($s in $subs) {
        New-Item -ItemType Directory -Force -Path (Join-Path $Brain $s) | Out-Null
    }
    $manifest = @{
        schema_version = 2
        project = @{ root_path = $Target }
        created_at_ns = [int64]((Get-Date) - (Get-Date "1970-01-01Z")).TotalSeconds * 1e9
    } | ConvertTo-Json -Depth 5
    Set-Content -Path (Join-Path $Brain "manifest.json") -Value $manifest -Encoding UTF8
    Write-Host "  ✓ $Brain\manifest.json (schema_version=2)"
}
Write-Host ""

# ---------------------------------------------------------------------- 5. wire AGENTS.md

Write-Host "→ step 5/6: wire AGENTS.md for your agent"
Set-Location $Target
foreach ($name in @("AGENTS.md", "CLAUDE.md")) {
    $path = Join-Path $Target $name
    if ((Test-Path $path) -and -not $Force) {
        Write-Host "  – $name exists; skipping"
    } else {
        Copy-Item -Path (Join-Path $dst "AGENTS.md") -Destination $path -Force
        Write-Host "  ✓ $name (copy; symlinks on Windows require dev mode)"
    }
}
Write-Host ""

# ---------------------------------------------------------------------- 6. verify

Write-Host "→ step 6/6: verify"
& python -m cyberos --store .cyberos-memory doctor
Write-Host ""

# ---------------------------------------------------------------------- extras

if ($WithAutomation) {
    Write-Host "→ extra: Windows Task Scheduler automation"
    & (Join-Path $Repo "memory\scripts\automation-install.ps1") -Target $Target
    Write-Host ""
}

if ($WithPreCommit) {
    Write-Host "→ extra: git pre-commit hook"
    $hook_dir = Join-Path $Target ".git\hooks"
    if (Test-Path $hook_dir) {
        Copy-Item -Path (Join-Path $Repo "memory\scripts\hooks\pre-commit") `
                  -Destination (Join-Path $hook_dir "pre-commit") -Force
        Write-Host "  ✓ .git\hooks\pre-commit installed (copy, not symlink)"
    } else {
        Write-Host "  – $Target has no .git\ — skipping"
    }
    Write-Host ""
}

Write-Host "=== done ==="
Write-Host ""
Write-Host "next steps:"
Write-Host "  1. open the project in your agent (Claude / Cursor / Cowork)"
Write-Host "  2. AGENTS.md is loaded automatically"
Write-Host "  3. verify anytime:"
Write-Host "       python -m cyberos --store .cyberos-memory state"
Write-Host "       python -m cyberos --store .cyberos-memory doctor"
