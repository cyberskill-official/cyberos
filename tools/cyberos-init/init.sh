#!/usr/bin/env bash
# init.sh - vendor the CyberOS machine into the CURRENT project under a gitignored .cyberos/,
# organised by module (.cyberos/cuo, .cyberos/memory, .cyberos/plugin), scaffold
# docs/feature-requests/ + the BRAIN, and print next steps. No CyberOS clone required.
# Idempotent: never clobbers your BACKLOG.md, gates.env, or existing BRAIN.
set -euo pipefail

src="$(cd "$(dirname "$0")" && pwd)"                   # the payload dir this script lives in
avail_ver="$(tr -d ' \n\r' < "$src/VERSION" 2>/dev/null || echo unknown)"

# --check: report installed vs available CyberOS version for the target, then exit.
if [ "${1:-}" = "--check" ]; then
  target="${2:-$(pwd)}"; root="$(cd "$target" && git rev-parse --show-toplevel 2>/dev/null || echo "$target")"
  inst="$(tr -d ' \n\r' < "$root/.cyberos/VERSION" 2>/dev/null || echo none)"
  echo "CyberOS: installed=$inst  available=$avail_ver"
  if [ "$inst" = "none" ]; then echo "  not initialised here - run: bash $0 $root"
  elif [ "$inst" = "$avail_ver" ]; then echo "  up to date."
  else echo "  UPDATE available ($inst -> $avail_ver). Update with: bash $0 $root"; fi
  exit 0
fi

target="${1:-$(pwd)}"; target="$(cd "$target" && pwd)"
root="$(cd "$target" && git rev-parse --show-toplevel 2>/dev/null || echo "$target")"
CY="$root/.cyberos"

echo "cyberos init: target repo = $root (CyberOS $avail_ver)"
mkdir -p "$CY" "$root/docs/feature-requests/_audits"

# 1. vendor the machine by module (replace any prior copy) --------------------
rm -rf "$CY/cuo" "$CY/plugin"
cp -R "$src/cuo"    "$CY/cuo"
cp -R "$src/plugin" "$CY/plugin"
[ -f "$src/manifest.yaml" ] && cp "$src/manifest.yaml" "$CY/manifest.yaml"
[ -f "$src/VERSION" ] && cp "$src/VERSION" "$CY/VERSION"
chmod +x "$CY/cuo/gates/run-gates.sh" 2>/dev/null || true

# 2. auto-detect gate commands ------------------------------------------------
BUILD_CMD=""; LINT_CMD=""; TEST_CMD=""; COVERAGE_CMD=""; ECOSYSTEM="unknown"
has() { command -v "$1" >/dev/null 2>&1; }
json_has_script() { grep -q "\"$1\"[[:space:]]*:" "$root/package.json" 2>/dev/null; }

if [ -f "$root/Cargo.toml" ]; then
  ECOSYSTEM="rust"
  BUILD_CMD="cargo build --workspace"
  LINT_CMD="cargo clippy --workspace --all-targets -- -D warnings"
  TEST_CMD="cargo test --workspace"
  has cargo-llvm-cov && COVERAGE_CMD="cargo llvm-cov --workspace --summary-only"
elif [ -f "$root/package.json" ]; then
  ECOSYSTEM="node"
  pm="npm"; { [ -f "$root/pnpm-lock.yaml" ] && pm="pnpm"; } || { [ -f "$root/yarn.lock" ] && pm="yarn"; }
  run="$pm run"; [ "$pm" = "npm" ] && run="npm run"
  json_has_script build    && BUILD_CMD="$run build"
  json_has_script lint     && LINT_CMD="$run lint"
  if json_has_script test; then TEST_CMD="$run test"; else TEST_CMD="$pm test"; fi
  json_has_script coverage && COVERAGE_CMD="$run coverage"
elif [ -f "$root/pyproject.toml" ] || [ -f "$root/setup.py" ] || [ -f "$root/setup.cfg" ]; then
  ECOSYSTEM="python"
  has ruff && LINT_CMD="ruff check ."
  has pytest && TEST_CMD="pytest" || TEST_CMD="python -m pytest"
  has coverage && COVERAGE_CMD="coverage run -m pytest && coverage report"
elif [ -f "$root/go.mod" ]; then
  ECOSYSTEM="go"
  BUILD_CMD="go build ./..."
  has golangci-lint && LINT_CMD="golangci-lint run" || LINT_CMD="go vet ./..."
  TEST_CMD="go test ./..."
  COVERAGE_CMD="go test -cover ./..."
elif [ -f "$root/Makefile" ]; then
  ECOSYSTEM="make"
  grep -q '^build:'    "$root/Makefile" && BUILD_CMD="make build"
  grep -q '^lint:'     "$root/Makefile" && LINT_CMD="make lint"
  grep -q '^test:'     "$root/Makefile" && TEST_CMD="make test"
  grep -q '^coverage:' "$root/Makefile" && COVERAGE_CMD="make coverage"
fi

# 3. write the gate env at .cyberos/gates.env (never clobber; back up) --------
env_file="$CY/gates.env"
[ -f "$env_file" ] && cp "$env_file" "$env_file.bak.$(date +%s)"
cat > "$env_file" <<EOF
# .cyberos/gates.env - gate commands for the FR workflow (edit freely).
# Auto-detected ecosystem: $ECOSYSTEM. Empty command = that gate is skipped.
# The reduced-profile floor = build + lint + test + coverage. These always run.
BUILD_CMD="$BUILD_CMD"
LINT_CMD="$LINT_CMD"
TEST_CMD="$TEST_CMD"
COVERAGE_CMD="$COVERAGE_CMD"
COVERAGE_MIN="90"
# Optional full-profile upgrades. Set enabled=true only when the baseline exists.
CAF_ENABLED="false"
CAF_CMD="bash .cyberos/cuo/gates/caf/caf_gate.sh ."
AWH_ENABLED="false"
AWH_CMD=""
# HITL is required: the two human-acceptance gates (review acceptance, final
# acceptance) are never automated. The agent halts; a human records each verdict.
HITL_REQUIRED="true"
EOF

# 4. scaffold the backlog -----------------------------------------------------
bl="$root/docs/feature-requests/BACKLOG.md"
if [ ! -f "$bl" ]; then
  proj="$(basename "$root")"
  sed "s/{{PROJECT}}/$proj/g" "$src/cuo/templates/BACKLOG.md" > "$bl"
fi

# 5. memory module + BRAIN (default on; skip with CYBEROS_NO_MEMORY=1) --------
MEMORY_SET="skipped"; MEM_AGENTS=""; MEM_BRAIN=""
if [ "${CYBEROS_NO_MEMORY:-0}" != "1" ] && [ -d "$src/memory" ]; then
  rm -rf "$CY/memory"; cp -R "$src/memory" "$CY/memory"

  # make the protocol discoverable at the repo root (never clobber an existing AGENTS.md)
  if [ ! -f "$root/AGENTS.md" ]; then
    cp "$src/memory/AGENTS.md" "$root/AGENTS.md"
    MEM_AGENTS="created root AGENTS.md (memory protocol)"
  else
    MEM_AGENTS="kept your AGENTS.md; protocol copy at .cyberos/memory/AGENTS.md"
  fi

  # scaffold the BRAIN store (.cyberos-memory, per protocol section 0.4) if absent
  brain="$root/.cyberos-memory"
  if [ ! -d "$brain" ]; then
    for d in audit memories adrs audits impl-plans code-reviews obs-injections index exports meta module member company client project; do
      mkdir -p "$brain/$d"
    done
    : > "$brain/.lock"
    head -c 8 /dev/zero > "$brain/HEAD"                       # 8-byte LE u64 seq counter = 0
    fp="$( (head -c 8 /dev/urandom | od -An -tx1 | tr -d ' \n') 2>/dev/null || echo 0000000000000000 )"
    ns="$(( $(date +%s) * 1000000000 ))"
    cat > "$brain/manifest.json" <<JSON
{
  "actor": "cyberos-init",
  "created_at_ns": $ns,
  "crypto_mode": "chained",
  "fingerprint": "$fp",
  "imports": {},
  "version": 2
}
JSON
    MEM_BRAIN="created .cyberos-memory/ (fresh BRAIN, HEAD=0)"
  else
    MEM_BRAIN="kept existing .cyberos-memory/"
  fi
  MEMORY_SET="yes"
fi

# 6. gitignore the vendored machine + the BRAIN (regenerable / tenant data) ---
gi="$root/.gitignore"
[ -f "$gi" ] || : > "$gi"
grep -q "CyberOS vendored machine" "$gi" || printf '\n# CyberOS vendored machine + local BRAIN (regenerable via init; tenant data). Do not commit.\n' >> "$gi"
grep -qx ".cyberos/" "$gi"        || echo ".cyberos/"        >> "$gi"
grep -qx ".cyberos-memory/" "$gi" || echo ".cyberos-memory/" >> "$gi"

# 7. tell the operator what to do next ----------------------------------------
cat <<EOF

cyberos init: done.
  cuo       -> .cyberos/cuo/          (workflow + doctrine + status contract + skills + gates)
  memory    -> .cyberos/memory/       (Layer-1 protocol + schema)
  gates     -> .cyberos/gates.env     (detected: build='${BUILD_CMD:-none}' test='${TEST_CMD:-none}')
  backlog   -> docs/feature-requests/BACKLOG.md
  BRAIN     -> ${MEMORY_SET}${MEM_BRAIN:+ (${MEM_BRAIN})}${MEM_AGENTS:+; ${MEM_AGENTS}}
  gitignored: .cyberos/ and .cyberos-memory/ (vendored machine + local state)
  version   -> CyberOS $avail_ver (.cyberos/VERSION); check for updates: <payload>/init.sh --check $root

Next:
  1. Write an FR from the template:
       cp .cyberos/cuo/templates/FR-TEMPLATE.md docs/feature-requests/FR-001-<slug>.md
       # fill in section 1, set status: ready_to_implement, add the row to BACKLOG.md
  2. Trigger the workflow in your agent (Claude Code / Cowork / Codex):
       "Follow .cyberos/cuo/ship-feature-requests.md and drive the next eligible FR in
        docs/feature-requests/BACKLOG.md. HITL is required: halt at the two human-acceptance
        gates. repo_root is this repo."
  3. Run the machine gates any time:
       bash .cyberos/cuo/gates/run-gates.sh

BRAIN memory protocol: .cyberos-memory/ is your local memory store (gitignored, tenant data).
The rules are in AGENTS.md (or .cyberos/memory/AGENTS.md). An agent working in this repo
records decisions, audits, and plans into the BRAIN per that protocol.
Skip memory setup by re-running init with CYBEROS_NO_MEMORY=1.
EOF
