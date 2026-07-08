#!/usr/bin/env bash
# init.sh - drop the FR workflow into the CURRENT project. No CyberOS clone required.
#
# Run this from inside a target repo (or pass its path as arg 1). It:
#   1. auto-detects the repo's build / lint / test / coverage commands,
#   2. copies the portable machine into <repo>/.cyberos/fr-pack/,
#   3. writes <repo>/.cyberos/fr.gates.env (the gate commands - edit freely),
#   4. scaffolds docs/feature-requests/ (BACKLOG.md + _audits/),
#   5. prints the trigger instructions.
# Idempotent: it never clobbers an existing BACKLOG.md or fr.gates.env (backs them up).
set -euo pipefail

pack="$(cd "$(dirname "$0")" && pwd)"                  # the pack dir this script lives in
target="${1:-$(pwd)}"
target="$(cd "$target" && pwd)"
root="$(cd "$target" && git rev-parse --show-toplevel 2>/dev/null || echo "$target")"

echo "fr-pack init: target repo = $root"
mkdir -p "$root/.cyberos" "$root/docs/feature-requests/_audits"

# 1. copy the machine in ------------------------------------------------------
cp -R "$pack/machine"   "$root/.cyberos/fr-pack-machine.tmp" && rm -rf "$root/.cyberos/fr-pack/machine" 2>/dev/null || true
mkdir -p "$root/.cyberos/fr-pack"
rm -rf "$root/.cyberos/fr-pack/machine" "$root/.cyberos/fr-pack/gates" "$root/.cyberos/fr-pack/templates" "$root/.cyberos/fr-pack/plugin"
mv "$root/.cyberos/fr-pack-machine.tmp" "$root/.cyberos/fr-pack/machine"
cp -R "$pack/gates"     "$root/.cyberos/fr-pack/gates"
cp -R "$pack/templates" "$root/.cyberos/fr-pack/templates"
cp -R "$pack/plugin"    "$root/.cyberos/fr-pack/plugin"
[ -f "$pack/manifest.yaml" ] && cp "$pack/manifest.yaml" "$root/.cyberos/fr-pack/manifest.yaml"
chmod +x "$root/.cyberos/fr-pack/gates/run-gates.sh" 2>/dev/null || true

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

# 3. write the gate env (never clobber; back up) ------------------------------
env_file="$root/.cyberos/fr.gates.env"
[ -f "$env_file" ] && cp "$env_file" "$env_file.bak.$(date +%s)"
cat > "$env_file" <<EOF
# .cyberos/fr.gates.env - gate commands for the FR workflow (edit freely).
# Auto-detected ecosystem: $ECOSYSTEM. Empty command = that gate is skipped.
# The reduced-profile floor = build + lint + test + coverage. These always run.
BUILD_CMD="$BUILD_CMD"
LINT_CMD="$LINT_CMD"
TEST_CMD="$TEST_CMD"
COVERAGE_CMD="$COVERAGE_CMD"
COVERAGE_MIN="90"
# Optional full-profile upgrades. Set enabled=true only when the baseline exists.
CAF_ENABLED="false"
CAF_CMD="bash .cyberos/fr-pack/gates/caf/caf_gate.sh ."
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
  sed "s/{{PROJECT}}/$proj/g" "$pack/templates/BACKLOG.md" > "$bl"
fi

# 5. tell the operator what to do next ----------------------------------------
cat <<EOF

fr-pack init: done.
  machine   -> .cyberos/fr-pack/         (workflow + doctrine + status contract)
  gates     -> .cyberos/fr.gates.env     (detected: build='${BUILD_CMD:-none}' test='${TEST_CMD:-none}')
  backlog   -> docs/feature-requests/BACKLOG.md

Next:
  1. Write an FR from the template:
       cp .cyberos/fr-pack/templates/FR-TEMPLATE.md docs/feature-requests/FR-001-<slug>.md
       # fill in section 1, set status: ready_to_implement, add the row to BACKLOG.md
  2. Trigger the workflow in your agent (Claude Code / Cowork / Codex):
       "Follow .cyberos/fr-pack/machine/ship-feature-requests.md and drive the next
        eligible FR in docs/feature-requests/BACKLOG.md. HITL is required: halt at the
        two human-acceptance gates. repo_root is this repo."
  3. Run the machine gates any time:
       bash .cyberos/fr-pack/gates/run-gates.sh
EOF
