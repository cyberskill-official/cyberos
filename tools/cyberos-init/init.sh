#!/usr/bin/env bash
# init.sh - vendor the CyberOS machine into the CURRENT project under a gitignored .cyberos/,
# organised by module (.cyberos/cuo, .cyberos/memory, .cyberos/plugin), scaffold
# docs/feature-requests/ + CHANGELOG.md + the BRAIN, auto-run the FR migration (folder-per-FR
# + status page; skip with CYBEROS_NO_MIGRATE=1), and print next steps. No CyberOS clone required.
# Idempotent: never clobbers your BACKLOG.md, CHANGELOG.md, gates.env, or existing BRAIN;
# .gitignore is managed through ONE marked block (everything outside it is never touched).
# A payload copied INSIDE the target repo (channel 1: .cyberos-init/) removes itself on
# success - keep it with CYBEROS_KEEP_PAYLOAD=1; submodules are never removed.
set -euo pipefail

src="$(cd "$(dirname "$0")" && pwd)"                   # the payload dir this script lives in
avail_ver="$( [ -f "$src/VERSION" ] && tr -d ' \n\r' < "$src/VERSION" || echo unknown )"

# --check: three-value report (FR-IMP-070) - installed (.cyberos/VERSION), payload (this
# payload's VERSION), latest (published release via check-latest.sh) - plus exactly one
# verdict line and the exact next command. Read-only; CYBEROS_OFFLINE=1 skips the remote hop.
if [ "${1:-}" = "--check" ]; then
  target="${2:-$(pwd)}"; root="$(cd "$target" && git rev-parse --show-toplevel 2>/dev/null || echo "$target")"
  # FR-IMP-071 leg: test-then-read - a failed `<` redirect prints to stderr BEFORE 2>/dev/null applies.
  if [ -f "$root/.cyberos/VERSION" ]; then inst="$(tr -d ' \n\r' < "$root/.cyberos/VERSION")"; else inst="none"; fi
  latest_line="latest=unknown source=offline"
  if [ "${CYBEROS_OFFLINE:-0}" != "1" ] && [ -f "$src/check-latest.sh" ]; then
    latest_line="$(bash "$src/check-latest.sh")"
  fi
  latest="${latest_line#latest=}"; latest="${latest%% *}"
  echo "installed=$inst"
  echo "payload=$avail_ver"
  echo "$latest_line"
  is_ver() { printf '%s' "$1" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$'; }
  ver_lt() { [ "$1" = "$2" ] && return 1; [ "$(printf '%s\n%s\n' "$1" "$2" | sort -t. -k1,1n -k2,2n -k3,3n | head -1)" = "$1" ]; }
  if is_ver "$latest" && is_ver "$avail_ver" && ver_lt "$avail_ver" "$latest"; then
    echo "verdict=payload_stale"
    echo "next: curl -fsSL https://github.com/cyberskill-official/cyberos/releases/latest/download/cyberos-payload.tar.gz -o /tmp/cyberos-payload.tar.gz   # or rebuild: bash tools/cyberos-init/build.sh in a current checkout, then re-run init"
  elif [ "$inst" = "none" ]; then
    echo "verdict=repo_stale"
    echo "next: bash $0 $root   # not initialised here"
  elif { is_ver "$latest" && is_ver "$inst" && ver_lt "$inst" "$latest"; } || { is_ver "$inst" && is_ver "$avail_ver" && ver_lt "$inst" "$avail_ver"; }; then
    echo "verdict=repo_stale"
    echo "next: bash $0 $root"
  else
    echo "verdict=up_to_date"
    case "$latest_line" in latest=unknown*) echo "  note: remote check skipped or unavailable - answer only as fresh as the local payload" ;; esac
  fi
  exit 0
fi

target="${1:-$(pwd)}"; target="$(cd "$target" && pwd)"
root="$(cd "$target" && git rev-parse --show-toplevel 2>/dev/null || echo "$target")"
CY="$root/.cyberos"

# guard: init.sh runs from an ASSEMBLED payload (build.sh output), where cuo/ + VERSION are
# siblings. Running it from the un-built source tree is a common mistake - fail with a clear hint.
if [ ! -d "$src/cuo" ]; then
  echo "cyberos init: '$src' is not an assembled payload (no cuo/). Build it first:" >&2
  echo "  bash tools/cyberos-init/build.sh   # -> dist/cyberos/, then run dist/cyberos/init.sh <repo>" >&2
  exit 1
fi

echo "cyberos init: target repo = $root (CyberOS $avail_ver)"
mkdir -p "$CY" "$root/docs/feature-requests/_audits"
mkdir -p "$root/docs/feature-requests/.workflow"
[ -f "$root/docs/feature-requests/.workflow/.gitignore" ] || printf '%s\n' '*.ship.json' > "$root/docs/feature-requests/.workflow/.gitignore"  # ship-manifest@1 run state stays untracked (FR-CUO-206)

# 1. vendor the machine by module (replace any prior copy) --------------------
rm -rf "$CY/cuo" "$CY/plugin" "$CY/mcp"
cp -R "$src/cuo"    "$CY/cuo"
cp -R "$src/plugin" "$CY/plugin"
[ -d "$src/mcp" ] && cp -R "$src/mcp" "$CY/mcp"          # MCP server channel (optional; needs node)
[ -f "$src/manifest.yaml" ] && cp "$src/manifest.yaml" "$CY/manifest.yaml"
[ -f "$src/VERSION" ] && cp "$src/VERSION" "$CY/VERSION"
# portable FR migration kit (FR 1.0.0): rides into .cyberos so any repo can adopt folder-per-FR + the status page
[ -f "$src/migrate-frs.sh" ] && cp "$src/migrate-frs.sh" "$CY/migrate-frs.sh" && chmod +x "$CY/migrate-frs.sh"
[ -d "$src/docs-tools" ] && rm -rf "$CY/docs-tools" && cp -R "$src/docs-tools" "$CY/docs-tools"
chmod +x "$CY/cuo/gates/run-gates.sh" 2>/dev/null || true
[ -f "$CY/mcp/cyberos-mcp.mjs" ] && chmod +x "$CY/mcp/cyberos-mcp.mjs" 2>/dev/null || true

# 2. auto-detect gate commands (FR-CUO-207: union across stacks, first claim per gate wins;
#    documented order: rust, node, python, go, maven, gradle, dotnet, php, ruby, make.
#    Never invent a command whose tool marker file is absent. Root-only scanning.)
BUILD_CMD=""; LINT_CMD=""; TEST_CMD=""; COVERAGE_CMD=""; ECOSYSTEM=""
SRC_BUILD=""; SRC_LINT=""; SRC_TEST=""; SRC_COVERAGE=""
has() { command -v "$1" >/dev/null 2>&1; }
json_has_script() { grep -q "\"$1\"[[:space:]]*:" "$root/package.json" 2>/dev/null; }
claim() { # claim <stack> <gate> <cmd> - first stack to claim a gate wins (union rule)
  local stack="$1" g="$2" cmd="$3"
  case "$g" in
    build)    [ -z "$BUILD_CMD"    ] && { BUILD_CMD="$cmd";    SRC_BUILD="$stack"; } ;;
    lint)     [ -z "$LINT_CMD"     ] && { LINT_CMD="$cmd";     SRC_LINT="$stack"; } ;;
    test)     [ -z "$TEST_CMD"     ] && { TEST_CMD="$cmd";     SRC_TEST="$stack"; } ;;
    coverage) [ -z "$COVERAGE_CMD" ] && { COVERAGE_CMD="$cmd"; SRC_COVERAGE="$stack"; } ;;
  esac
  case ",$ECOSYSTEM," in *",$stack,"*) ;; *) ECOSYSTEM="${ECOSYSTEM:+$ECOSYSTEM,}$stack" ;; esac
}

if [ -f "$root/Cargo.toml" ]; then
  claim rust build "cargo build --workspace"
  claim rust lint  "cargo clippy --workspace --all-targets -- -D warnings"
  claim rust test  "cargo test --workspace"
  has cargo-llvm-cov && claim rust coverage "cargo llvm-cov --workspace --summary-only"
fi
if [ -f "$root/package.json" ]; then
  pm="npm"; { [ -f "$root/pnpm-lock.yaml" ] && pm="pnpm"; } || { [ -f "$root/yarn.lock" ] && pm="yarn"; }
  run="$pm run"; [ "$pm" = "npm" ] && run="npm run"
  json_has_script build    && claim node build "$run build"
  json_has_script lint     && claim node lint "$run lint"
  if json_has_script test; then claim node test "$run test"; else claim node test "$pm test"; fi
  json_has_script coverage && claim node coverage "$run coverage"
fi
if [ -f "$root/pyproject.toml" ] || [ -f "$root/setup.py" ] || [ -f "$root/setup.cfg" ]; then
  has ruff && claim python lint "ruff check ."
  if has pytest; then claim python test "pytest"; else claim python test "python -m pytest"; fi
  has coverage && claim python coverage "coverage run -m pytest && coverage report"
fi
if [ -f "$root/go.mod" ]; then
  claim go build "go build ./..."
  if has golangci-lint; then claim go lint "golangci-lint run"; else claim go lint "go vet ./..."; fi
  claim go test "go test ./..."
  claim go coverage "go test -coverprofile=coverage.out ./..."
fi
if [ -f "$root/pom.xml" ]; then
  claim maven build "mvn -q -DskipTests package"
  claim maven test  "mvn -q verify"
  # coverage deliberately undetected for JVM (jacoco wiring is repo-specific) - use .cyberos/config.yaml
fi
if [ -f "$root/build.gradle" ] || [ -f "$root/build.gradle.kts" ]; then
  gw="gradle"; [ -x "$root/gradlew" ] && gw="./gradlew"
  claim gradle build "$gw build"
  claim gradle test  "$gw test"
fi
if ls "$root"/*.sln >/dev/null 2>&1 || ls "$root"/*.csproj >/dev/null 2>&1; then
  claim dotnet build "dotnet build"
  claim dotnet test  "dotnet test"
fi
if [ -f "$root/composer.json" ]; then
  claim php lint "composer validate --strict"
  [ -f "$root/vendor/bin/phpunit" ] && claim php test "vendor/bin/phpunit"
fi
if [ -f "$root/Gemfile" ]; then
  if [ -d "$root/spec" ]; then claim ruby test "bundle exec rspec"
  elif [ -f "$root/Rakefile" ]; then claim ruby test "bundle exec rake test"; fi
fi
if [ -f "$root/Makefile" ]; then
  grep -q '^build:'    "$root/Makefile" && claim make build "make build"
  grep -q '^lint:'     "$root/Makefile" && claim make lint "make lint"
  grep -q '^test:'     "$root/Makefile" && claim make test "make test"
  grep -q '^coverage:' "$root/Makefile" && claim make coverage "make coverage"
fi
[ -z "$ECOSYSTEM" ] && ECOSYSTEM="unknown"

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
# Per-gate autodetect provenance (FR-CUO-207; consumed by run-gates.sh provenance lines).
SRC_BUILD="$SRC_BUILD"
SRC_LINT="$SRC_LINT"
SRC_TEST="$SRC_TEST"
SRC_COVERAGE="$SRC_COVERAGE"
# Optional full-profile upgrades. Set enabled=true only when the baseline exists.
CAF_ENABLED="false"
CAF_CMD="bash .cyberos/cuo/gates/caf/caf_gate.sh ."
AWH_ENABLED="false"
AWH_CMD=""
# HITL is required: the two human-acceptance gates (review acceptance, final
# acceptance) are never automated. The agent halts; a human records each verdict.
HITL_REQUIRED="true"
EOF

# 3b. scaffold .cyberos/config.yaml exactly once (FR-CUO-207 §1 #3; never clobber) --
cfg_file="$root/.cyberos/config.yaml"
if [ ! -f "$cfg_file" ]; then
  cat > "$cfg_file" <<EOF
# .cyberos/config.yaml - per-repo CyberOS overrides (FR-CUO-207). Everything below is
# commented out = inert; uncomment a line to override ONLY that key. Detected defaults
# are shown as comments so this file documents what runs today.
# gates:
#   build: "$BUILD_CMD"$([ -n "$SRC_BUILD" ] && printf '%s' "        # autodetected: $SRC_BUILD")
#   lint: "$LINT_CMD"$([ -n "$SRC_LINT" ] && printf '%s' "         # autodetected: $SRC_LINT")
#   test: "$TEST_CMD"$([ -n "$SRC_TEST" ] && printf '%s' "         # autodetected: $SRC_TEST")
#   coverage: "$COVERAGE_CMD"$([ -n "$SRC_COVERAGE" ] && printf '%s' "     # autodetected: $SRC_COVERAGE")
# coverage_threshold: 90
# fr_template: engineering-spec@1
# profile: full
EOF
fi

# 4. scaffold the backlog -----------------------------------------------------
# A pre-existing docs/BACKLOG.md is ADOPTED into the canonical home first (content preserved);
# only a repo with neither gets the template.
bl="$root/docs/feature-requests/BACKLOG.md"; BACKLOG_SET="docs/feature-requests/BACKLOG.md"
if [ ! -f "$bl" ] && [ -f "$root/docs/BACKLOG.md" ]; then
  mv "$root/docs/BACKLOG.md" "$bl"
  BACKLOG_SET="adopted docs/BACKLOG.md -> docs/feature-requests/BACKLOG.md"
fi
if [ ! -f "$bl" ]; then
  proj="$(basename "$root")"
  sed "s/{{PROJECT}}/$proj/g" "$src/cuo/templates/BACKLOG.md" > "$bl"
fi

# 4b. scaffold CHANGELOG.md exactly once (never clobber) -----------------------
# The status page's Changelog tab reads root CHANGELOG.md `## [X.Y.Z] - date` sections.
# A pre-existing docs/CHANGELOG.md is ADOPTED to the root first (content preserved);
# only a repo with neither gets the Keep-a-Changelog seed with release 0.1.0.
cl="$root/CHANGELOG.md"; CHANGELOG_SET="kept your CHANGELOG.md"
if [ ! -f "$cl" ] && [ -f "$root/docs/CHANGELOG.md" ]; then
  mv "$root/docs/CHANGELOG.md" "$cl"
  CHANGELOG_SET="adopted docs/CHANGELOG.md -> CHANGELOG.md (root; status page input)"
fi
if [ ! -f "$cl" ]; then
  cat > "$cl" <<EOF
# Changelog

All notable changes to this project live here - one \`## [X.Y.Z] - YYYY-MM-DD\` section per
release (Keep-a-Changelog style; the CyberOS status page's releases lens reads these sections,
and every FR id you name in an entry becomes a chip that opens that FR).

## [0.1.0] - $(date +%Y-%m-%d)

- CyberOS initialised: FR workflow vendored to .cyberos/, backlog at docs/feature-requests/BACKLOG.md, status page at docs/status/.
EOF
  CHANGELOG_SET="created CHANGELOG.md (seeds the status page's releases lens)"
fi

# 4c. FR migration + status page (auto; skip with CYBEROS_NO_MIGRATE=1) --------
# Brings pre-existing FRs to the folder-per-FR rule (root-level flat FRs included) and
# (re)generates the status page at docs/status/ - ONE page, three lenses (board | table |
# releases) over the FR corpus, with a drawer carrying each FR's full spec.
# Idempotent and verified: migrate-frs.sh ends with a machine-readable verify line and
# WARNs for anything it could not place. A failure here never aborts init.
MIGRATE_SET="skipped (CYBEROS_NO_MIGRATE=1)"
if [ "${CYBEROS_NO_MIGRATE:-0}" != "1" ]; then
  if [ -f "$CY/migrate-frs.sh" ]; then
    if mig_out="$(bash "$CY/migrate-frs.sh" "$root" 2>&1)"; then MIGRATE_SET="ok"; else MIGRATE_SET="FAILED (non-fatal; re-run: bash .cyberos/migrate-frs.sh)"; fi
    printf '%s\n' "$mig_out" | sed 's/^/  | /'
    mig_layout="$(printf '%s\n' "$mig_out" | grep '^fr-layout: ' | tail -1 || true)"
    mig_verify="$(printf '%s\n' "$mig_out" | grep '^migrate-frs verify: ' | tail -1 || true)"
    MIGRATE_SET="$MIGRATE_SET${mig_layout:+; ${mig_layout#fr-layout: }}${mig_verify:+; ${mig_verify#migrate-frs }}"
  else
    MIGRATE_SET="unavailable (payload built without the migration kit)"
  fi
fi

# The summary must never claim a page that was not rendered (migration is what renders it).
if [ -f "$root/docs/status/index.html" ]; then
  STATUS_SET="docs/status/ (index.html + assets/ + data/; ONE page, three lenses - board | table |
                                       releases - over THIS repo's FRs, with a drawer carrying each
                                       full spec. Replaces the old standalone docs; tracked)"
else
  STATUS_SET="none (no FRs to render - the page appears the moment this repo has its first FR)"
fi

# 5. memory module + BRAIN (default on; skip with CYBEROS_NO_MEMORY=1) --------
MEMORY_SET="skipped"; MEM_AGENTS=""; MEM_BRAIN=""
if [ "${CYBEROS_NO_MEMORY:-0}" != "1" ] && [ -d "$src/memory" ]; then
  # vendor the protocol docs into .cyberos/memory/ WITHOUT touching the live
  # store at .cyberos/memory/store/ (an update refreshes docs, never the data).
  mkdir -p "$CY/memory"
  for f in AGENTS.md memory.schema.json memory.invariants.yaml; do
    [ -f "$src/memory/$f" ] && cp "$src/memory/$f" "$CY/memory/$f"
  done

  # The memory PROTOCOL lives at .cyberos/memory/AGENTS.md. The repo-root AGENTS.md is the
  # cross-agent SPINE (step 5b) and references this protocol - so we do NOT overwrite root
  # AGENTS.md with the dense protocol here (that would bury the workflow every agent needs).
  MEM_AGENTS="memory protocol -> .cyberos/memory/AGENTS.md"

  # scaffold the BRAIN store at .cyberos/memory/store/ (canonical, section 0.4).
  brain="$CY/memory/store"
  if [ ! -d "$brain" ]; then
    # canonical v2 top-level dirs only (memory.invariants.yaml layout-root-canonical);
    # CUO artifact kinds (adrs, audits, impl-plans, ...) are NOT top-level - they
    # nest under their memory kind and are created on demand.
    for d in memories meta company module member client project persona conflicts exports index audit; do
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
    MEM_BRAIN="created .cyberos/memory/store/ (fresh BRAIN, HEAD=0)"
  else
    MEM_BRAIN="kept existing .cyberos/memory/store/"
  fi
  MEMORY_SET="yes"
fi

# 5b. AGENT SURFACE - make the workflow discoverable to EVERY popular coding agent.
# AGENTS.md (root) is the canonical cross-agent spine (read natively by Codex, Cursor,
# Gemini, Antigravity, Grok CLI, zcode, Command Code, Aider, Zed, Copilot, Warp, Jules...).
# Per-agent pointer files + native skill installs layer on top. Everything is create-if-
# absent: an existing operator file is NEVER clobbered. Controls:
#   CYBEROS_AGENTS=all|"claude-code,codex,..."   restrict which agents get files (default all)
#   CYBEROS_COPY_SKILLS=1                          copy skills instead of symlinking (committable)
#   CYBEROS_GLOBAL_SKILLS=1                        also install skills into $HOME agent dirs
#   CYBEROS_NO_MCP=1                               skip the MCP .mcp.json registration
AGENT_FILES=""; SKILL_DIRS=""; MCP_SET="skipped"

want_agent() { case ",${CYBEROS_AGENTS:-all}," in *,all,*) return 0;; *",$1,"*) return 0;; *) return 1;; esac; }

# canonical one-page entry - the source of truth every pointer file defers to.
cat > "$CY/AGENT-ENTRY.md" <<'ENTRY'
# CyberOS agent entry

This repository runs CyberOS. Any coding agent operating here follows these rules:

1. Work = feature requests. Read `.cyberos/cuo/ship-feature-requests.md` and drive
   the next eligible FR in `docs/feature-requests/BACKLOG.md` (one backlog for both
   `class: product` and `class: improvement`; frontmatter `status` is the truth).
2. HITL is required: halt at review acceptance (`reviewing -> ready_to_test`) and at
   final acceptance (`testing -> done`) for a recorded human verdict. Never set
   `done` yourself. Doctrine: `.cyberos/cuo/EXECUTION-DISCIPLINE.md`; lifecycle:
   `.cyberos/cuo/STATUS-REFERENCE.md`.
3. Machine gates: `bash .cyberos/cuo/gates/run-gates.sh` (reads `.cyberos/gates.env`).
   Green is necessary, never sufficient.
4. Memory: record decisions, audits, and plans into the BRAIN at
   `.cyberos/memory/store/` per the protocol in `.cyberos/memory/AGENTS.md`.
5. Never push, deploy, merge, or delete without an explicit operator instruction.
ENTRY

# --- the universal AGENTS.md spine (the one file the most agents read natively) ---
agents_spine() {
  cat <<'SPINE'
# AGENTS.md

This repository runs **CyberOS**. Any coding agent working here follows these rules.
This file is the cross-tool spine; the full one-pager is `.cyberos/AGENT-ENTRY.md`.

1. Work = feature requests. Read `.cyberos/cuo/ship-feature-requests.md` and drive the
   next eligible FR in `docs/feature-requests/BACKLOG.md` (frontmatter `status` is the
   record of truth; one backlog for `class: product` and `class: improvement`).
2. HITL is required. Halt at review acceptance (`reviewing -> ready_to_test`) and final
   acceptance (`testing -> done`) for a recorded human verdict. Never set `done` yourself.
   Doctrine: `.cyberos/cuo/EXECUTION-DISCIPLINE.md`; lifecycle: `.cyberos/cuo/STATUS-REFERENCE.md`.
3. Gates: `bash .cyberos/cuo/gates/run-gates.sh` (reads `.cyberos/gates.env`). Green is
   necessary, never sufficient.
4. Memory (BRAIN): record decisions, audits, and plans into `.cyberos/memory/store/`
   per the protocol in `.cyberos/memory/AGENTS.md`.
5. Never push, deploy, merge, or delete without an explicit operator instruction.
SPINE
}
SP_MARK="cyberos-agent-spine (managed by cyberos init; edit above/below this marker)"
if [ -L "$root/AGENTS.md" ] && [ ! -e "$root/AGENTS.md" ]; then
  # DANGLING symlink (a spine that pointed at a file its target repo no longer has). Leaving it
  # is worse than having nothing: every agent that reads AGENTS.md natively gets ENOENT. Replace.
  rm -f "$root/AGENTS.md"
  { agents_spine; printf '\n<!-- %s -->\n' "$SP_MARK"; } > "$root/AGENTS.md"
  AGENTS_SET="replaced a DANGLING AGENTS.md symlink with the spine (it pointed at a file that no longer exists)"
elif [ -L "$root/AGENTS.md" ]; then
  # Symlinked spine (e.g. CyberOS itself links AGENTS.md -> the memory-protocol source): creating
  # or appending would write THROUGH the link into whatever it points at - and a DANGLING link
  # (target checkout absent on this machine) would crash the write outright. The spine lives in
  # .cyberos/AGENT-ENTRY.md regardless; leave the operator's link untouched. Checked FIRST because
  # `-f` follows links, so a dangling symlink would otherwise fall into the create branch.
  AGENTS_SET="kept your AGENTS.md symlink untouched (spine at .cyberos/AGENT-ENTRY.md)"
elif [ ! -f "$root/AGENTS.md" ]; then
  { agents_spine; printf '\n<!-- %s -->\n' "$SP_MARK"; } > "$root/AGENTS.md"
  AGENTS_SET="created AGENTS.md (canonical cross-agent spine)"
elif grep -q "$SP_MARK" "$root/AGENTS.md" 2>/dev/null || grep -q '\.cyberos/AGENT-ENTRY\.md' "$root/AGENTS.md" 2>/dev/null; then
  AGENTS_SET="kept your AGENTS.md (already CyberOS-aware)"
else
  { printf '\n---\n\n'; agents_spine; printf '\n<!-- %s -->\n' "$SP_MARK"; } >> "$root/AGENTS.md"
  AGENTS_SET="appended a CyberOS section to your AGENTS.md"
fi

# --- per-agent pointer files (create only when absent; agent prefers its own file) ---
# pointer <agent-key> <path-rel-to-root> <style: md|plain|mdc>
pointer() {
  want_agent "$1" || return 0
  local rel="$2" style="$3" abs="$root/$2"
  [ -e "$abs" ] && return 0
  mkdir -p "$(dirname "$abs")"
  case "$style" in
    mdc)
      { printf -- '---\ndescription: CyberOS feature-request workflow (HITL-gated). Always apply.\nalwaysApply: true\n---\n'
        printf 'This repo runs CyberOS. Canonical instructions: AGENTS.md (root) and .cyberos/AGENT-ENTRY.md.\n'
        printf 'Work is feature-requests; HITL is required at the two human-acceptance gates; run gates with `bash .cyberos/cuo/gates/run-gates.sh`. Never push/deploy/merge without an operator instruction.\n'; } > "$abs" ;;
    plain)
      { printf 'This repo runs CyberOS. Canonical instructions: AGENTS.md (root) and .cyberos/AGENT-ENTRY.md.\n'
        printf 'Work is feature-requests; HITL is required at the two human-acceptance gates; gates: bash .cyberos/cuo/gates/run-gates.sh. Never push/deploy/merge without an operator instruction.\n'; } > "$abs" ;;
    *)
      { printf '# %s\n\n' "$(basename "$rel" .md)"
        printf 'This repo runs **CyberOS**. Canonical agent instructions: `AGENTS.md` (root) and `.cyberos/AGENT-ENTRY.md`.\n\n'
        printf 'Work is feature-requests; HITL is required at the two human-acceptance gates; run gates with `bash .cyberos/cuo/gates/run-gates.sh`. Never push, deploy, or merge without an explicit operator instruction.\n'; } > "$abs" ;;
  esac
  AGENT_FILES="$AGENT_FILES $rel"
}
pointer claude-code   CLAUDE.md                          md      # Claude Code CLI (Command Code also reads CLAUDE.md)
pointer gemini        GEMINI.md                          md      # Gemini CLI + Antigravity (GEMINI.md wins on conflict)
pointer cursor        .cursorrules                       plain   # Cursor (legacy rules file)
pointer cursor        .cursor/rules/cyberos.mdc          mdc     # Cursor (modern .cursor/rules/*.mdc)
pointer grok          .grok/GROK.md                      md      # Grok CLI (superagent-ai)
pointer copilot       .github/copilot-instructions.md    md      # GitHub Copilot
pointer antigravity   .agents/rules/cyberos.md           md      # Antigravity / zcode workspace rules (.agents/rules/)
pointer windsurf      .windsurfrules                     plain   # Windsurf
# Codex, zcode, Command Code, Aider, Zed, Jules, Warp, OpenCode read AGENTS.md -> covered by the spine.

# --- native skill install: drop ship-feature-requests into each skill-aware agent's dir ---
# so it is invocable natively (/ship-feature-requests, $ship-feature-requests) - not just prose.
# Default = relative symlink into the self-contained skill at .cyberos/plugin/skills (tracks
# updates on re-init; regenerable, so gitignored). CYBEROS_COPY_SKILLS=1 copies it instead.
SKILL_SRC="$CY/plugin/skills/ship-feature-requests"
relup() { local up="" seg; local IFS=/; for seg in $1; do [ -n "$seg" ] && up="../$up"; done; printf '%s' "$up"; }
install_skill() {                                  # $1 = agent skills dir (rel to root)
  want_agent "$2" || return 0
  [ -d "$SKILL_SRC" ] || return 0
  local dir="$root/$1" dest="$root/$1/ship-feature-requests"
  if [ -e "$dest" ] || [ -L "$dest" ]; then         # already there: only refresh OUR own link/copy
    case "$(readlink "$dest" 2>/dev/null)" in *".cyberos/plugin/skills/ship-feature-requests") : ;; *) return 0;; esac
    rm -rf "$dest" 2>/dev/null || return 0
  fi
  mkdir -p "$dir"
  if [ "${CYBEROS_COPY_SKILLS:-0}" = "1" ]; then
    cp -R "$SKILL_SRC" "$dest"
  else
    ln -s "$(relup "$1").cyberos/plugin/skills/ship-feature-requests" "$dest" 2>/dev/null || cp -R "$SKILL_SRC" "$dest"
  fi
  SKILL_DIRS="$SKILL_DIRS $1"
}
install_skill .claude/skills      claude-code    # Claude Code
install_skill .grok/skills        grok           # Grok CLI
install_skill .commandcode/skills command-code   # Command Code
install_skill .codex/skills       codex          # Codex CLI (skills)
install_skill .opencode/skill     opencode       # OpenCode (singular 'skill')
# zcode + Hermes load skills from a global home ($HOME); opt in with CYBEROS_GLOBAL_SKILLS=1.
if [ "${CYBEROS_GLOBAL_SKILLS:-0}" = "1" ]; then
  for gp in "$HOME/.claude/skills" "$HOME/.grok/skills" "$HOME/.hermes/skills" "$HOME/.commandcode/skills"; do
    [ -e "$gp/ship-feature-requests" ] || { mkdir -p "$gp" && cp -R "$SKILL_SRC" "$gp/ship-feature-requests" 2>/dev/null && SKILL_DIRS="$SKILL_DIRS ~${gp#"$HOME"}"; }
  done
fi

# --- MCP server registration (any MCP-capable agent triggers the workflow tool-natively) ---
# Writes a project .mcp.json (Claude Code, Cursor via .cursor/mcp.json, Windsurf, etc. read it)
# only when absent. Needs node at run time. Snippets for Codex/others are in .cyberos/mcp/README.md.
if [ "${CYBEROS_NO_MCP:-0}" != "1" ] && [ -f "$CY/mcp/cyberos-mcp.mjs" ]; then
  mcp_json() { printf '{\n  "mcpServers": {\n    "cyberos": { "command": "node", "args": [".cyberos/mcp/cyberos-mcp.mjs"] }\n  }\n}\n'; }
  wrote=""
  [ -e "$root/.mcp.json" ]        || { mcp_json > "$root/.mcp.json";        wrote="$wrote .mcp.json"; }
  if want_agent cursor; then mkdir -p "$root/.cursor"; [ -e "$root/.cursor/mcp.json" ] || { mcp_json > "$root/.cursor/mcp.json"; wrote="$wrote .cursor/mcp.json"; }; fi
  MCP_SET="server -> .cyberos/mcp/ ; registered:${wrote:- (none new; see .cyberos/mcp/README.md)}"
fi

# 6. gitignore - ONE managed block, regenerated in place on every init ---------
# Policy (what init writes, tracked vs ignored):
#   TRACKED  - the agent surface (AGENTS.md, CLAUDE.md, GEMINI.md, .cursorrules, .cursor/rules/,
#              .grok/GROK.md, .github/copilot-instructions.md, .agents/rules/, .windsurfrules,
#              .mcp.json, .cursor/mcp.json), docs/feature-requests/** (BACKLOG + specs/audits),
#              CHANGELOG.md, docs/status.html (the generated status page - the repo's published
#              Roadmap/Backlog/Changelog view), and skill COPIES (CYBEROS_COPY_SKILLS=1).
#   IGNORED  - .cyberos/ (vendored machine + BRAIN store + render intermediates: regenerable /
#              tenant data) and the skill SYMLINKS (they point INTO the ignored .cyberos/).
# Mechanics: everything lives between the two markers below; re-init REPLACES the block content
# (never appends duplicates), anything outside the markers is never touched, and the scattered
# entries older inits appended (pre-block) are lifted into the block on first contact.
gi="$root/.gitignore"
[ -f "$gi" ] || : > "$gi"
GI_BEGIN="# >>> cyberos (managed block - regenerated by cyberos init; edit outside the markers) >>>"
GI_END="# <<< cyberos <<<"
gi_block() {
  printf '%s\n' "$GI_BEGIN"
  printf '%s\n' "# vendored machine + local BRAIN store + render intermediates (regenerable via init; tenant data)"
  printf '%s\n' ".cyberos/"
  if [ "${CYBEROS_COPY_SKILLS:-0}" != "1" ]; then
    printf '%s\n' "# skill symlinks -> .cyberos/plugin/skills (regenerable via init)"
    for sp in .claude/skills/ship-feature-requests .grok/skills/ship-feature-requests .commandcode/skills/ship-feature-requests .codex/skills/ship-feature-requests .opencode/skill/ship-feature-requests; do
      printf '%s\n' "$sp"
    done
  fi
  printf '%s\n' "$GI_END"
}
# strip any prior managed block + the exact legacy lines pre-block inits appended, then trim
# the trailing blank run and append the fresh block (one blank line of separation).
awk -v b="$GI_BEGIN" -v e="$GI_END" '
  $0==b {inblk=1; next} $0==e {inblk=0; next} inblk {next}
  $0=="# CyberOS vendored machine + local BRAIN at .cyberos/memory/store (regenerable via init; tenant data). Do not commit." {next}
  $0=="# CyberOS skill symlinks -> .cyberos/plugin/skills (regenerable via init)." {next}
  $0==".cyberos/" {next}
  $0==".claude/skills/ship-feature-requests" {next}
  $0==".grok/skills/ship-feature-requests" {next}
  $0==".commandcode/skills/ship-feature-requests" {next}
  $0==".codex/skills/ship-feature-requests" {next}
  $0==".opencode/skill/ship-feature-requests" {next}
  {lines[++n]=$0}
  END { while (n>0 && lines[n] ~ /^[[:space:]]*$/) n--; for (i=1;i<=n;i++) print lines[i] }
' "$gi" > "$gi.cyberos.tmp"
{ cat "$gi.cyberos.tmp"; [ -s "$gi.cyberos.tmp" ] && printf '\n'; gi_block; } > "$gi"
rm -f "$gi.cyberos.tmp"

# 6b. status auto-sync hook (managed; CYBEROS_NO_HOOK=1 skips) -----------------
# docs/status/ must stay synced with the markdown it renders (FR frontmatter, CHANGELOG.md,
# VERSION). Touchpoints: run-gates.sh after gates, and this pre-commit when inputs are staged.
# v2: blocking on regen failure + pipefail-safe staged list (never `git diff | grep -q`).
# An existing foreign pre-commit is never replaced - we append a marked block once.
HOOK_SET="skipped (CYBEROS_NO_HOOK=1)"
if [ "${CYBEROS_NO_HOOK:-0}" != "1" ]; then
  if [ ! -d "$root/.git" ]; then
    HOOK_SET="skipped (not a git checkout)"
  else
    hk="$root/.git/hooks/pre-commit"
    mkdir -p "$root/.git/hooks"
    if [ ! -f "$hk" ] || head -5 "$hk" 2>/dev/null | grep -q "cyberos-status-hook"; then
      # absent, or a hook WE own outright (marker in the header): (re)write the standalone form
      cat > "$hk" <<'HOOK'
#!/usr/bin/env bash
# cyberos-status-hook v2 (managed by cyberos init)
# Regenerates docs/status/ when FR sources change and STAGES it in the same commit.
# Blocks the commit if regeneration fails (so status never lags GitHub).
# Disable: delete this file, or re-init with CYBEROS_NO_HOOK=1.
set -euo pipefail
# Read staged list ONCE — never `git diff | grep -q` under pipefail (SIGPIPE skip bug).
staged="$(git diff --cached --name-only || true)"
if grep -Eq '^(docs/feature-requests/|CHANGELOG\.md$|VERSION$)' <<<"$staged"; then
  if [ ! -f .cyberos/migrate-frs.sh ]; then
    echo "cyberos: ERROR .cyberos/migrate-frs.sh missing (run cyberos init)" >&2
    exit 1
  fi
  if ! command -v node >/dev/null 2>&1; then
    echo "cyberos: ERROR node required to regenerate docs/status" >&2
    exit 1
  fi
  echo "cyberos: regenerating docs/status/ …"
  bash .cyberos/migrate-frs.sh --page . || {
    echo "cyberos: ERROR status regen failed — run: bash .cyberos/migrate-frs.sh --page ." >&2
    exit 1
  }
  git add docs/status 2>/dev/null || true
  echo "cyberos: docs/status staged"
fi
exit 0
HOOK
      HOOK_SET="pre-commit hook v2 installed (blocks if docs/status regen fails; auto-stages status page)"
    elif grep -q "cyberos-status-hook v2" "$hk" 2>/dev/null || grep -q "cyberos-status-hook v2" "$hk" 2>/dev/null; then
      HOOK_SET="kept your pre-commit hook (cyberos status-sync v2 already present)"
    elif grep -q "cyberos-status-hook" "$hk" 2>/dev/null; then
      # Upgrade v1 append block → v2 if we own only the marker (leave foreign body)
      if grep -q "cyberos-status-hook v2" "$hk" 2>/dev/null; then
        HOOK_SET="kept your pre-commit hook (cyberos status-sync v2 already appended)"
      else
        # Replace v1 marker block with v2 blocking form (best-effort)
        if grep -q ">>> cyberos-status-hook v1" "$hk" 2>/dev/null; then
          # strip old append block then re-append v2
          tmp="$hk.cyberos.tmp"
          sed '/# >>> cyberos-status-hook v1/,/# <<< cyberos-status-hook <<</d' "$hk" > "$tmp" && mv "$tmp" "$hk"
        fi
        cat >> "$hk" <<'HOOK'

# >>> cyberos-status-hook v2 (appended by cyberos init; edits above survive re-init) >>>
_cyberos_rc=$?
staged="$(git diff --cached --name-only || true)"
if grep -Eq '^(docs/feature-requests/|CHANGELOG\.md$|VERSION$)' <<<"$staged"; then
  if [ -f .cyberos/migrate-frs.sh ] && command -v node >/dev/null 2>&1; then
    if bash .cyberos/migrate-frs.sh --page .; then
      git add docs/status 2>/dev/null || true
      echo "cyberos: docs/status regenerated + staged"
    else
      echo "cyberos: ERROR docs/status regen failed" >&2
      exit 1
    fi
  fi
fi
exit $_cyberos_rc
# <<< cyberos-status-hook <<<
HOOK
        HOOK_SET="upgraded appended status-sync block to v2 (blocking regen)"
      fi
    else
      # a FOREIGN hook exists: append a marked block that preserves the foreign exit code
      cat >> "$hk" <<'HOOK'

# >>> cyberos-status-hook v2 (appended by cyberos init; edits above survive re-init) >>>
_cyberos_rc=$?
staged="$(git diff --cached --name-only || true)"
if grep -Eq '^(docs/feature-requests/|CHANGELOG\.md$|VERSION$)' <<<"$staged"; then
  if [ -f .cyberos/migrate-frs.sh ] && command -v node >/dev/null 2>&1; then
    if bash .cyberos/migrate-frs.sh --page .; then
      git add docs/status 2>/dev/null || true
      echo "cyberos: docs/status regenerated + staged"
    else
      echo "cyberos: ERROR docs/status regen failed" >&2
      exit 1
    fi
  fi
fi
exit $_cyberos_rc
# <<< cyberos-status-hook <<<
HOOK
      HOOK_SET="appended status-sync v2 to your existing pre-commit hook"
    fi
    chmod +x "$hk"
  fi
fi

# 7. tell the operator what to do next ----------------------------------------
cat <<EOF

cyberos init: done.
  cuo       -> .cyberos/cuo/          (workflow + doctrine + status contract + skills + gates)
  memory    -> .cyberos/memory/       (Layer-1 protocol + schema)
  gates     -> .cyberos/gates.env     (detected: build='${BUILD_CMD:-none}' test='${TEST_CMD:-none}')
  backlog   -> ${BACKLOG_SET}
  changelog -> ${CHANGELOG_SET}
  migrate   -> ${MIGRATE_SET}
  status    -> ${STATUS_SET}
  auto-sync -> ${HOOK_SET}; run-gates.sh also regenerates the page after every gates run
  agents    -> ${AGENTS_SET}
              pointer files:${AGENT_FILES:- (none new)}
              native skills:${SKILL_DIRS:- (none new)}
              MCP: ${MCP_SET}
  BRAIN     -> ${MEMORY_SET}${MEM_BRAIN:+ (${MEM_BRAIN})}${MEM_AGENTS:+; ${MEM_AGENTS}}
  gitignored: one managed block in .gitignore covers .cyberos/ (vendored machine + BRAIN store)
              + the skill symlinks; agent files, docs/feature-requests/**, CHANGELOG.md and
              docs/status/ stay TRACKED (commit them). Everything outside the block is yours.
  version   -> CyberOS $avail_ver (.cyberos/VERSION); check for updates: <payload>/init.sh --check $root

Next:
  1. Write an FR from the template:
       mkdir -p docs/feature-requests/<module>/FR-001-<slug> && cp .cyberos/cuo/templates/FR-TEMPLATE.md docs/feature-requests/<module>/FR-001-<slug>/spec.md
       # fill in section 1, set status: ready_to_implement, add the row to BACKLOG.md
  2. Trigger the workflow in your agent (Claude Code / Cowork / Codex):
       "Follow .cyberos/cuo/ship-feature-requests.md and drive the next eligible FR in
        docs/feature-requests/BACKLOG.md. HITL is required: halt at the two human-acceptance
        gates. repo_root is this repo."
  3. Run the machine gates any time:
       bash .cyberos/cuo/gates/run-gates.sh

Every popular agent is wired: AGENTS.md is the cross-agent spine, and Claude Code, Codex,
Cursor, Gemini, Antigravity, Grok CLI, zcode, Command Code, Copilot & Windsurf each get the
right pointer file / native skill / MCP registration (all create-if-absent; your files are
never clobbered). Restrict with CYBEROS_AGENTS=..., copy skills with CYBEROS_COPY_SKILLS=1,
skip MCP with CYBEROS_NO_MCP=1. MCP server + per-agent registration snippets: .cyberos/mcp/README.md.

BRAIN memory protocol: .cyberos/memory/store/ is your local memory store (gitignored, tenant
data). The rules are in .cyberos/memory/AGENTS.md (root AGENTS.md is the workflow spine and
points to it). An agent working here records decisions, audits, and plans into the BRAIN per
that protocol. Skip memory setup by re-running init with CYBEROS_NO_MEMORY=1.
EOF

# 8. payload self-cleanup ------------------------------------------------------
# A payload COPIED INSIDE the target repo at the canonical channel-1 location
# (cp -R dist/cyberos <repo>/.cyberos-init) is redundant once init succeeds: everything the
# repo needs onward lives under .cyberos/ (machine, gates, migration kit, MCP server).
# Remove it so it is never committed by accident. ONLY <root>/.cyberos-init is ever removed -
# never payloads outside the repo (dev checkouts / dist), never other in-repo paths, never a
# git submodule/checkout (channel 2 - carries a .git entry), and never with CYBEROS_KEEP_PAYLOAD=1.
if [ "$src" = "$root/.cyberos-init" ]; then
  if [ "${CYBEROS_KEEP_PAYLOAD:-0}" = "1" ]; then
    echo "payload: kept .cyberos-init/ (CYBEROS_KEEP_PAYLOAD=1 - remember it is untracked; .cyberos/ is what the repo runs on)"
  elif [ -e "$src/.git" ] || { [ -f "$root/.gitmodules" ] && grep -q "path = .cyberos-init" "$root/.gitmodules"; }; then
    echo "payload: kept .cyberos-init/ (git submodule/checkout - never auto-removed)"
  else
    # two attempts with a settle pause (network/virtual mounts defer the final rmdir);
    # never abort init over cleanup
    rm -rf "$src" 2>/dev/null || true
    if [ -e "$src" ]; then sleep 1; rm -rf "$src" 2>/dev/null || true; fi
    if [ ! -e "$src" ]; then
      echo "payload: removed .cyberos-init/ (self-cleanup - everything now lives in .cyberos/; keep it next time with CYBEROS_KEEP_PAYLOAD=1)"
    elif [ -d "$src" ] && [ -z "$(ls -A "$src" 2>/dev/null)" ]; then
      # network/virtual mounts defer the last unlink while this script's own file handle is open
      echo "payload: emptied .cyberos-init/; the folder handle is held by this run (network-mount quirk) - finish with: rmdir .cyberos-init"
    else
      echo "payload: WARN could not fully remove .cyberos-init/ - delete it manually; everything the repo needs now lives in .cyberos/"
    fi
  fi
else
  case "$src/" in
    "$root"/*) if [ "$src" != "$root" ]; then echo "payload: kept ${src#"$root"/}/ (non-canonical in-repo location - only <repo>/.cyberos-init self-cleans; it is redundant after init and safe to delete)"; fi ;;
    *) : ;;   # payload lives outside the target repo - not ours to touch
  esac
fi
