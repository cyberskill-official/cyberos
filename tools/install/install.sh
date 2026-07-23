#!/usr/bin/env bash
# install.sh — one-time (or re-vendor) install of CyberOS into a project under gitignored .cyberos/.
# Organised by module (.cyberos/cuo, .cyberos/memory, .cyberos/plugin), scaffolds
# docs/tasks/ + CHANGELOG.md + the BRAIN, runs task migration + status page
# (skip with CYBEROS_NO_MIGRATE=1). Idempotent; never clobbers BACKLOG/CHANGELOG/BRAIN.
# Day-to-day: soft update checks run on any .cyberos use; manual check: version.sh.
# Remove: uninstall.sh. Open status page: status.sh.
set -euo pipefail

# -P / pwd -P: PHYSICAL paths, symlinks resolved.
#
# `root` below is `git rev-parse --show-toplevel`, which always returns a physical path.
# `src` used a logical `cd && pwd`. On macOS /tmp is a symlink to /private/tmp, so src came
# back /tmp/x/.cyberos-install while root came back /private/tmp/x — and the self-cleanup at
# the foot of this file is a STRING COMPARE, `[ "$src" = "$root/.cyberos-install" ]`. It never
# matched, so .cyberos-install was never removed after a bootstrap install on a Mac. Invisible
# on Linux, where /tmp is a real directory and logical == physical.
#
# Comparing paths from two resolvers is comparing two different questions.
src="$(cd -P "$(dirname "$0")" && pwd -P)"             # the payload dir this script lives in
avail_ver="$( [ -f "$src/VERSION" ] && tr -d ' \n\r' < "$src/VERSION" || echo unknown )"


# Internal page regen lives at lib/status-page.sh (hooks + run-gates). Not user-facing.
# Full task migrate runs automatically during install (unless CYBEROS_NO_MIGRATE=1).

# -P here too: when git is absent `root` falls back to `$target`, so target must already be
# physical or the self-cleanup compare inherits the same /tmp vs /private/tmp mismatch.
target="${1:-$(pwd)}"; target="$(cd -P "$target" && pwd -P)"
root="$(cd "$target" && git rev-parse --show-toplevel 2>/dev/null || echo "$target")"
CY="$root/.cyberos"
_cy_downgrade_note=""    # §1.4: set only when an operator overrides a refused downgrade

# guard: install.sh runs from an ASSEMBLED payload (build.sh output), where cuo/ + VERSION are
# siblings. Running it from the un-built source tree is a common mistake - fail with a clear hint.
if [ ! -d "$src/cuo" ]; then
  echo "cyberos install: '$src' is not an assembled payload (no cuo/). Build it first:" >&2
  echo "  bash tools/install/build.sh   # -> dist/cyberos/, then run dist/cyberos/install.sh <repo>" >&2
  exit 1
fi

echo "cyberos install: target repo = $root (CyberOS $avail_ver)"
mkdir -p "$CY" "$root/docs/tasks/_audits"
mkdir -p "$root/docs/tasks/.workflow"
# .workflow run state stays untracked: ship manifests (TASK-CUO-206) + task-author run
# manifests (TASK-IMP-090) + the skill-trust ledger (TASK-IMP-113, docs/tasks/.workflow/
# skill-trust.tsv — append-only pass/fail measurement, never a gate). Fresh seed carries all
# three patterns; an existing seed that predates a pattern gains exactly that one (append-once,
# idempotent across re-installs; operator lines and everything else in the file are never touched).
wf_ignore="$root/docs/tasks/.workflow/.gitignore"
if [ ! -f "$wf_ignore" ]; then
  printf '%s\n' '*.ship.json' '*.manifest.json' 'skill-trust.tsv' > "$wf_ignore"
else
  # Each pattern is append-once and independent: a seed predating any one gains only that one,
  # in the exact discipline TASK-IMP-090 established (grep -qxF guard + trailing-newline heal).
  if ! grep -qxF '*.manifest.json' "$wf_ignore"; then
    [ -z "$(tail -c 1 "$wf_ignore")" ] || printf '\n' >> "$wf_ignore"   # heal a missing trailing newline so the pattern lands as its own line
    printf '%s\n' '*.manifest.json' >> "$wf_ignore"
  fi
  if ! grep -qxF 'skill-trust.tsv' "$wf_ignore"; then
    [ -z "$(tail -c 1 "$wf_ignore")" ] || printf '\n' >> "$wf_ignore"   # heal a missing trailing newline (TASK-IMP-113 append-once)
    printf '%s\n' 'skill-trust.tsv' >> "$wf_ignore"
  fi
fi

# 0a. downgrade guard (TASK-IMP-104) -----------------------------------------
# install vendored whatever it was handed and never compared versions, so an older payload
# silently downgraded a consumer: skills vanish, doctrine reverts, and the only trace is a
# VERSION line nobody re-reads. The workflow-version pins cannot catch it - they live in the
# payload being installed. Runs BEFORE the lock (§1.1): a refused downgrade takes no lock.
_vc="$src/lib/version-compare.sh"; [ -f "$_vc" ] || _vc="$(dirname "$0")/lib/version-compare.sh"
if [ ! -f "$_vc" ]; then
  # FAIL CLOSED. A guard that skips itself when its comparator is missing is not a guard - it is
  # a comment. Caught by t01 on first run: build.sh had not vendored the lib, so the payload
  # silently downgraded with the check present in the source and absent from the artefact.
  echo "cyberos install: version-compare.sh is missing from this payload - the downgrade guard cannot run." >&2
  echo "  This payload is malformed. Rebuild it: bash tools/install/build.sh" >&2
  exit 1
fi
if true; then
  . "$_vc"
  _inst_ver=""
  [ -f "$CY/VERSION" ] && _inst_ver="$(tr -d ' \n\r' < "$CY/VERSION" 2>/dev/null || echo "")"
  if [ -z "$_inst_ver" ]; then
    :                                   # §1.6 first install or damaged machine - not a downgrade
  elif ! is_ver "$_inst_ver" || ! is_ver "$avail_ver"; then
    # §1.6 + §3: 'unknown' and pre-release strings are NOT comparable. Not comparable is not
    # older - refusing on an ordering we cannot defend would block legitimate installs.
    echo "cyberos install: version not comparable (installed '$_inst_ver', payload '$avail_ver') - proceeding." >&2
  elif ver_lt "$avail_ver" "$_inst_ver"; then
    if [ "${CYBEROS_ALLOW_DOWNGRADE:-0}" = "1" ]; then
      echo "cyberos install: DOWNGRADE $_inst_ver -> $avail_ver (CYBEROS_ALLOW_DOWNGRADE=1)." >&2
      _cy_downgrade_note="downgraded $_inst_ver -> $avail_ver (operator override)"
    else
      echo "cyberos install: payload $avail_ver is OLDER than the installed $_inst_ver. Refusing." >&2
      echo "  An older payload removes skills and reverts doctrine with no other trace." >&2
      echo "  Deliberate rollback: CYBEROS_ALLOW_DOWNGRADE=1 bash $0 $root" >&2
      exit 1
    fi
  fi
fi

# 0. concurrency lock (TASK-IMP-103) -----------------------------------------
# The vendor step below removes the machine and re-copies it. Between those two
# operations .cyberos/ does not exist. A second install - or an agent reading
# .cyberos/ mid-install - sees a half-vendored tree. `mkdir` is the atomic primitive
# everywhere we run (flock is absent on stock macOS; the payload's floor is POSIX).
#
# Refusal vs breakage: a lock is broken ONLY when it is BOTH older than the stale
# threshold AND its pid is provably dead on THIS host. Either alone refuses - a
# just-started install has not had time to look alive (§1.4), and a pid from another
# machine on a shared mount is unknowable, so it is treated as alive until the
# threshold expires (the same conservative default TASK-IMP-093's lease uses).
CY_LOCK="$CY/.install.lock"
CYBEROS_LOCK_STALE_SECS="${CYBEROS_LOCK_STALE_SECS:-900}"
_cy_lock_held=""                       # set only once WE own it; the refusal path never releases
_cy_lock_stamp=""                      # the exact owner bytes WE wrote; identity, not just intent

# Release only what we STILL own. `_cy_lock_held` records that we once acquired the lock - it
# cannot record that we still hold it. The gap is real: we acquire, we hang past the stale
# threshold, a second installer breaks our lock (§1.3) and mkdirs a NEW one at the same path,
# then we wake and exit. A held-only check would `rm -rf` the new holder's lock and reopen the
# unguarded vendor window for a third process - the exact window the lock exists to close.
# So compare the owner bytes: ours, or nobody's business.
# (External review, 2026-07-17. The earlier review confirmed the REFUSAL path cannot delete a
# holder's lock, which is true and is a different path. This is the stale-break path.)
_cy_lock_release() {
  [ -n "$_cy_lock_held" ] || return 0
  # Unreadable or changed owner -> not provably ours -> leave it. A leaked lock self-heals at
  # the stale threshold; a wrongly-deleted lock corrupts a live install. Fail safe, not tidy.
  [ "$(cat "$CY_LOCK/owner" 2>/dev/null)" = "$_cy_lock_stamp" ] || return 0
  rm -rf "$CY_LOCK" 2>/dev/null
  return 0
}
_cy_now() { date +%s; }

_cy_lock_acquire() {
  if mkdir "$CY_LOCK" 2>/dev/null; then
    _cy_lock_held=1
    # Build the stamp BEFORE the trap arms: the trap must never fire against an empty stamp,
    # which would compare equal to an unreadable owner file and delete a lock we cannot verify.
    _cy_lock_stamp="$(printf 'pid=%s\nstarted_at=%s\nhost=%s\n' "$$" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$(hostname 2>/dev/null || echo unknown)")"
    printf '%s\n' "$_cy_lock_stamp" > "$CY_LOCK/owner" 2>/dev/null || true
    trap '_cy_lock_release' EXIT INT TERM
    return 0
  fi
  # mkdir failed. Contention, or something else entirely? Distinguishing the two is the
  # difference between an operator killing a process and an operator hunting a ghost.
  if [ ! -d "$CY_LOCK" ]; then
    echo "cyberos install: cannot create lock at $CY_LOCK (permissions or read-only filesystem) - this is NOT contention." >&2
    exit 1
  fi
  _lp=""; _ls=""; _lh=""
  if [ -r "$CY_LOCK/owner" ]; then
    _lp="$(sed -n 's/^pid=//p'        "$CY_LOCK/owner" 2>/dev/null | head -1)"
    _ls="$(sed -n 's/^started_at=//p' "$CY_LOCK/owner" 2>/dev/null | head -1)"
    _lh="$(sed -n 's/^host=//p'       "$CY_LOCK/owner" 2>/dev/null | head -1)"
  fi
  # Age from the directory's own mtime: survives an owner file that was never written
  # (killed between mkdir and the write) and cannot be forged by editing the file.
  # GNU stat -f is --file-system and SUCCEEDS with unrelated output ("File: ..."), which then
  # detonates in arithmetic under set -u. Try GNU -c first, BSD -f second, and hard-validate
  # numeric before any arithmetic touches it. Caught by t02 on Linux, 2026-07-17.
  _lock_mtime="$(stat -c %Y "$CY_LOCK" 2>/dev/null || stat -f %m "$CY_LOCK" 2>/dev/null || echo 0)"
  case "$_lock_mtime" in ""|*[!0-9]*) _lock_mtime=0 ;; esac
  _age=$(( $(_cy_now) - _lock_mtime )); [ "$_age" -lt 0 ] && _age=0
  # Liveness is TRI-STATE, not a boolean: alive | dead | unknown. Same-host pids are decidable
  # via kill -0. A pid from another host on a shared mount, or an owner file we cannot read, is
  # UNKNOWN - and unknown is NOT alive. Collapsing unknown into alive wedges the lock forever:
  # nothing would ever break a foreign lock at any age (caught by t07, 2026-07-17 - the same
  # reboot-wedge class the batch-4 review found in TASK-IMP-093's lease).
  #   alive   -> never break (a live install owns it)
  #   dead    -> break iff stale (§1.3); refuse while fresh (§1.4)
  #   unknown -> break iff stale. Past the threshold the holder is gone or hung, and both mean
  #              abandoned; before it, we defer - which is the §3 "alive until the threshold" rule.
  _liveness=unknown
  _this_host="$(hostname 2>/dev/null || echo unknown)"
  if [ -n "$_lp" ] && [ "$_lh" = "$_this_host" ]; then
    if kill -0 "$_lp" 2>/dev/null; then _liveness=alive; else _liveness=dead; fi
  fi
  if [ "$_liveness" != alive ] && [ "$_age" -ge "$CYBEROS_LOCK_STALE_SECS" ]; then
    echo "cyberos install: breaking stale lock (pid ${_lp:-unknown}${_lh:+ on $_lh}, age ${_age}s >= ${CYBEROS_LOCK_STALE_SECS}s, liveness=${_liveness})" >&2
    rm -rf "$CY_LOCK"
    _cy_lock_acquire; return $?
  fi
  echo "cyberos install: another install holds the lock (pid ${_lp:-unknown}${_lh:+ on $_lh}, age ${_age}s); this pid $$. Refusing." >&2
  echo "  If that process is gone, the lock is broken automatically after ${CYBEROS_LOCK_STALE_SECS}s (CYBEROS_LOCK_STALE_SECS)." >&2
  exit 1
}
_cy_lock_acquire

# 1. vendor the machine by module (stage + swap - TASK-IMP-137 §1.6) -----------
# Replacing a subtree used to be `rm -rf` then `cp -R`: between the two, any reader of
# .cyberos/ (agents read it constantly - it is their entry point) saw a missing or partial
# machine for the whole copy duration; the TASK-IMP-103 lock serializes INSTALLERS only.
# Now each subtree is fully staged BESIDE its destination first, then swapped into place:
#   - first install: one clean rename of the staged tree;
#   - identical re-vendor (the common reinstall): a content fingerprint short-circuits the
#     swap entirely - nothing moves, so nothing can be observed missing;
#   - changed payload: a per-path sync where every FILE is rename(2)'d over its
#     destination - a path that exists in both the old and the new tree is NEVER absent or
#     truncated (rename replaces atomically), and only payload-dropped paths are pruned.
# The measured alternative (whole-tree mv old->trash + mv staged->live) still left a
# multi-millisecond absence window - two separate mv processes - that a tight reader
# caught thousands of times across 20 reinstalls; per-path rename is the form that
# actually delivers "bounded by rename/move operations". A kill mid-install leaves the
# old tree intact plus a stray staging dir; the cleanup below reclaims strays at the
# next install start, and the re-vendor restores the machine.
rm -rf "$CY"/*.tmp.* "$CY"/*.old.* 2>/dev/null || true   # stray staging dirs from killed installs
_vnonce="$$.$(date +%s)"
_tree_fp() {   # content fingerprint of a tree (names + bytes; payload filenames are ours: no whitespace)
  ( cd "$1" 2>/dev/null && find . ! -type d | LC_ALL=C sort | xargs cksum 2>/dev/null | cksum )
}
vendor_swap() {   # $1 = subtree name under both $src and $CY
  local name="${1:?}" staged live rel p
  staged="$CY/$name.tmp.$_vnonce"; live="$CY/$name"
  cp -R "$src/$name" "$staged"
  if [ ! -e "$live" ] && [ ! -L "$live" ]; then
    mv "$staged" "$live"                                   # first install: one rename
    return 0
  fi
  if [ "$(_tree_fp "$staged")" = "$(_tree_fp "$live")" ]; then
    rm -rf "$staged"                                       # identical re-vendor: no-op swap
    return 0
  fi
  # changed payload: ensure the directory skeleton (clearing same-named non-dirs - shape
  # changes across versions are real), then rename every staged file over its destination
  while IFS= read -r p; do
    rel="${p#"$staged"/}"
    if [ -e "$live/$rel" ] && [ ! -d "$live/$rel" ]; then rm -f "$live/$rel"; fi
    mkdir -p "$live/$rel"
  done < <(find "$staged" -mindepth 1 -type d)
  while IFS= read -r p; do
    rel="${p#"$staged"/}"
    if [ -d "$live/$rel" ] && [ ! -L "$live/$rel" ]; then rm -rf "$live/$rel"; fi
    mv -f "$p" "$live/$rel"
  done < <(find "$staged" -mindepth 1 ! -type d)
  # prune paths the new payload no longer ships (files first, then now-empty dirs deepest-first)
  while IFS= read -r p; do
    rel="${p#"$live"/}"
    [ -e "$src/$name/$rel" ] || [ -L "$src/$name/$rel" ] || rm -f "$p"
  done < <(find "$live" -mindepth 1 ! -type d)
  while IFS= read -r p; do
    rel="${p#"$live"/}"
    [ -d "$src/$name/$rel" ] || rm -rf "$p"
  done < <(find "$live" -mindepth 1 -depth -type d)
  rm -rf "$staged"
}
vendor_swap cuo
vendor_swap plugin
[ -d "$src/mcp" ] && vendor_swap mcp          # MCP server channel (optional; needs node)
# TASK-IMP-137 §1.5: the GitHub Action channel is only real if install vendors it - the
# README documented .cyberos/ci/ while install never created it. Same ownership as the
# other machine trees (inside the wholly machine-owned .cyberos/; uninstall reclaims it).
[ -d "$src/ci" ] && vendor_swap ci
[ -f "$src/manifest.yaml" ] && cp "$src/manifest.yaml" "$CY/manifest.yaml"
[ -f "$src/VERSION" ] && cp "$src/VERSION" "$CY/VERSION"
[ -f "$src/install.sh" ] && cp "$src/install.sh" "$CY/install.sh" && chmod +x "$CY/install.sh"
[ -f "$src/uninstall.sh" ] && cp "$src/uninstall.sh" "$CY/uninstall.sh" && chmod +x "$CY/uninstall.sh"
[ -f "$src/version.sh" ] && cp "$src/version.sh" "$CY/version.sh" && chmod +x "$CY/version.sh"
[ -f "$src/status.sh" ] && cp "$src/status.sh" "$CY/status.sh" && chmod +x "$CY/status.sh"
[ -f "$src/help.sh" ] && cp "$src/help.sh" "$CY/help.sh" && chmod +x "$CY/help.sh"
[ -f "$src/check-latest.sh" ] && cp "$src/check-latest.sh" "$CY/check-latest.sh" && chmod +x "$CY/check-latest.sh"
# lib (task-migrate, update-check, status-page) + docs-tools
[ -d "$src/lib" ] && vendor_swap lib
[ -d "$src/docs-tools" ] && vendor_swap docs-tools
rm -f "$CY"/gates.env.bak.* 2>/dev/null || true   # not an orphan: our own backup churn
chmod +x "$CY/cuo/gates/run-gates.sh" 2>/dev/null || true
[ -f "$CY/mcp/cyberos-mcp.mjs" ] && chmod +x "$CY/mcp/cyberos-mcp.mjs" 2>/dev/null || true
# update check on every full install (soft)
if [ -f "$CY/lib/update-check.sh" ]; then
  # shellcheck source=/dev/null
  source "$CY/lib/update-check.sh"
  CYBEROS_UPDATE_CHECK="${CYBEROS_UPDATE_CHECK:-always}" _cyberos_update_check || true
fi

# 2. auto-detect gate commands (TASK-CUO-207: union across stacks, first claim per gate wins;
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
# monorepo fallback tier for the TEST gate (TASK-CUO-302 §1.4): an ordered, CLOSED list -
# the canonical suite entrypoint first, a Makefile test: target second - probed only when
# no language stack above claimed a test command. It sits BEFORE the Makefile stack block
# so the repo's own canonical suite outranks `make test` when both exist (the order is
# contractual, asserted by test_fail_closed_gates.sh t04). Existence checks only: the
# probed files are NEVER executed at install time. Provenance lands in SRC_TEST so
# run-gates.sh's provenance line shows where the command came from (fallback:*), and
# ECOSYSTEM is deliberately NOT touched - a fallback is not a detected ecosystem.
if [ -z "$TEST_CMD" ]; then
  if [ -f "$root/scripts/tests/run_all.sh" ]; then
    TEST_CMD="bash scripts/tests/run_all.sh"; SRC_TEST="fallback:run_all"
  elif [ -f "$root/Makefile" ] && grep -q '^test:' "$root/Makefile"; then
    TEST_CMD="make test"; SRC_TEST="fallback:make"
  fi
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
env_bak=""
if [ -f "$env_file" ]; then
  env_bak="$env_file.bak.$(date +%s)"
  cp "$env_file" "$env_bak"
fi
cat > "$env_file" <<EOF
# .cyberos/gates.env - machine-owned; regenerated on every install (the previous file is
# backed up beside it). Durable overrides belong in .cyberos/config.yaml
# (gates.build / gates.lint / gates.test / gates.coverage) - edits made HERE are wiped
# by the next install.
# Auto-detected ecosystem: $ECOSYSTEM. Empty command = that gate is skipped.
# The reduced-profile floor = build + lint + test + coverage. These always run.
BUILD_CMD="$BUILD_CMD"
LINT_CMD="$LINT_CMD"
TEST_CMD="$TEST_CMD"
COVERAGE_CMD="$COVERAGE_CMD"
COVERAGE_MIN="90"
# Per-gate autodetect provenance (TASK-CUO-207; consumed by run-gates.sh provenance lines).
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
EOF
# A silent clobber of an operator-edited file is a trust leak even when the backup exists
# (TASK-IMP-095): when regeneration CHANGED the file, say where the previous content went
# and where durable overrides belong. Identical regeneration and the fresh-create case
# stay silent - nothing was lost. gates.env stays machine-owned (TASK-CUO-207).
if [ -n "$env_bak" ] && ! cmp -s "$env_bak" "$env_file"; then
  echo "cyberos install: gates.env regenerated (previous kept at $env_bak); durable overrides belong in .cyberos/config.yaml"
fi

# 3b. scaffold .cyberos/config.yaml exactly once (TASK-CUO-207 §1 #3; never clobber) --
# is_platform_repo() is HOISTED here from the AGENTS.md handling further down (its other
# caller) because step 3b runs first and needs it: consumer installs scaffold a LIVE
# `task_template: task@1` line so a fresh repo's first authoring run resolves the profile
# its vendored materials assume; the platform repo keeps the commented corpus default so
# the heavy profile stays operator-chosen (TASK-IMP-088, recorded decision IMP-06).
is_platform_repo() {
  # CyberOS monorepo: root AGENTS.md is the normative Layer-1 protocol source.
  [ -f "$root/modules/memory/memory.schema.json" ]
}
cfg_file="$root/.cyberos/config.yaml"
if [ ! -f "$cfg_file" ]; then
  cfg_tmpl_line="task_template: task@1"
  is_platform_repo && cfg_tmpl_line="# task_template: engineering-spec@1"
  cat > "$cfg_file" <<EOF
# .cyberos/config.yaml - per-repo CyberOS overrides (TASK-CUO-207). Commented lines are
# inert; uncomment one to override ONLY that key. Detected defaults are shown as comments
# so this file documents what runs today. Live (uncommented) lines are in effect as
# written - on consumer installs, task_template is scaffolded live (TASK-IMP-088).
# gates:
#   build: "$BUILD_CMD"$([ -n "$SRC_BUILD" ] && printf '%s' "        # autodetected: $SRC_BUILD")
#   lint: "$LINT_CMD"$([ -n "$SRC_LINT" ] && printf '%s' "         # autodetected: $SRC_LINT")
#   test: "$TEST_CMD"$([ -n "$SRC_TEST" ] && printf '%s' "         # autodetected: $SRC_TEST")
#   coverage: "$COVERAGE_CMD"$([ -n "$SRC_COVERAGE" ] && printf '%s' "     # autodetected: $SRC_COVERAGE")
# coverage_threshold: 90
$cfg_tmpl_line
# profile: full
EOF
fi

# 4. scaffold the backlog -----------------------------------------------------
# A pre-existing docs/BACKLOG.md is ADOPTED into the canonical home first (content preserved);
# only a repo with neither gets the template.
bl="$root/docs/tasks/BACKLOG.md"; BACKLOG_SET="docs/tasks/BACKLOG.md"
if [ ! -f "$bl" ] && [ -f "$root/docs/BACKLOG.md" ]; then
  mv "$root/docs/BACKLOG.md" "$bl"
  BACKLOG_SET="adopted docs/BACKLOG.md -> docs/tasks/BACKLOG.md"
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
and every task id you name in an entry becomes a chip that opens that task).

## [0.1.0] - $(date +%Y-%m-%d)

- CyberOS initialised: task workflow vendored to .cyberos/, backlog at docs/tasks/BACKLOG.md, status page at docs/status/.
EOF
  CHANGELOG_SET="created CHANGELOG.md (seeds the status page's releases lens)"
fi

# 4c. Task migration + status page (auto; skip with CYBEROS_NO_MIGRATE=1) --------
# Brings pre-existing tasks to the folder-per-task rule (root-level flat tasks included) and
# (re)generates the status page at docs/status/ - ONE page, three lenses (board | table |
# releases) over the task corpus, with a drawer carrying each task's full spec.
# Idempotent and verified: cyberos-migrate ends with a machine-readable verify line and
# WARNs for anything it could not place. A failure here never aborts install.
MIGRATE_SET="skipped (CYBEROS_NO_MIGRATE=1)"
if [ "${CYBEROS_NO_MIGRATE:-0}" != "1" ]; then
  if [ -f "$src/lib/task-migrate.sh" ] || [ -f "$CY/lib/task-migrate.sh" ]; then
    # shellcheck source=/dev/null
    if [ -f "$src/lib/task-migrate.sh" ]; then source "$src/lib/task-migrate.sh"; kit="$src"
    else source "$CY/lib/task-migrate.sh"; kit="$CY"; fi
    if mig_out="$(PAGE_ONLY=0 _cyberos_task_migrate "$root" "$kit" 2>&1)"; then MIGRATE_SET="ok"; else MIGRATE_SET="FAILED (non-fatal; re-run: bash $0 $root)"; fi
    printf '%s\n' "$mig_out" | sed 's/^/  | /'
    mig_verify="$(printf '%s\n' "$mig_out" | grep '^cyberos-migrate verify: ' | tail -1 || true)"
    MIGRATE_SET="$MIGRATE_SET${mig_verify:+; ${mig_verify#cyberos-migrate }}"
  else
    MIGRATE_SET="unavailable (payload built without lib/task-migrate.sh)"
  fi
fi

# The summary must never claim a page (or a part of it) that was not rendered (migration is
# what renders it). data/ holds per-task spec chunks and only exists once tasks land, so a
# fresh 0-task install legitimately ships index.html + assets/ alone - list what IS there.
if [ -f "$root/docs/status/index.html" ]; then
  status_parts="index.html"
  [ -d "$root/docs/status/assets" ] && status_parts="$status_parts + assets/"
  [ -d "$root/docs/status/data" ]   && status_parts="$status_parts + data/"
  STATUS_SET="docs/status/ ($status_parts; ONE page, three lenses - board | table |
                                       releases - over THIS repo's tasks, with a drawer carrying each
                                       full spec. data/ spec chunks appear once tasks land. Replaces
                                       the old standalone docs; tracked)"
else
  # A successful install always renders the page (CHANGELOG.md seeds >=1 release even with
  # 0 tasks), so absence here means migration was skipped or failed - say that, not "no tasks".
  STATUS_SET="none (render skipped or failed - re-run: bash .cyberos/lib/status-page.sh .)"
fi

# 5. memory module + BRAIN (default on; skip with CYBEROS_NO_MEMORY=1) --------
MEMORY_SET="skipped"; MEM_AGENTS=""; MEM_BRAIN=""
if [ "${CYBEROS_NO_MEMORY:-0}" != "1" ] && [ -d "$src/memory" ]; then
  # vendor the protocol docs into .cyberos/memory/ WITHOUT touching the live
  # store at .cyberos/memory/store/ (an update refreshes docs, never the data).
  mkdir -p "$CY/memory"
  # INTEROP.md joins the vendored protocol docs (TASK-MEMORY-303 §1.3; guarded by -f so
  # payloads built before the doc exists still install cleanly).
  for f in AGENTS.md memory.schema.json memory.invariants.yaml INTEROP.md; do
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
  "actor": "cyberos",
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

1. Work = tasks. Read `.cyberos/cuo/ship-tasks.md` and drive
   the next eligible task in `docs/tasks/BACKLOG.md` (one backlog for both
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

# --- root AGENTS.md is a thin pointer (like CLAUDE.md / GEMINI.md), NOT the memory protocol ---
# Full workflow one-pager: .cyberos/AGENT-ENTRY.md
# Memory protocol (Layer-1):     .cyberos/memory/AGENTS.md
# Platform monorepo exception: root AGENTS.md remains the normative protocol source.
agents_spine() {
  cat <<'SPINE'
# AGENTS.md

This repository runs **CyberOS**. Canonical agent instructions: `.cyberos/AGENT-ENTRY.md`.

Work is tasks; HITL is required at the two human-acceptance gates; run gates with `bash .cyberos/cuo/gates/run-gates.sh`. Never push, deploy, or merge without an explicit operator instruction.

Memory (BRAIN): protocol at `.cyberos/memory/AGENTS.md`; store at `.cyberos/memory/store/`.
SPINE
}
SP_MARK="cyberos-agent-spine (managed by cyberos install; edit above/below this marker)"
write_agents_spine() {
  { agents_spine; printf '\n<!-- %s -->\n' "$SP_MARK"; } > "$root/AGENTS.md"
}
# is_platform_repo() is defined above step 3b (hoisted there for the config.yaml scaffold - TASK-IMP-088).
is_protocol_dump() {
  # Follows symlinks; true if content is the dense Layer-1 protocol.
  [ -e "$root/AGENTS.md" ] && grep -q 'Layer-1 Memory Protocol' "$root/AGENTS.md" 2>/dev/null
}

if [ -L "$root/AGENTS.md" ] && [ ! -e "$root/AGENTS.md" ]; then
  rm -f "$root/AGENTS.md"
  write_agents_spine
  AGENTS_SET="replaced DANGLING AGENTS.md symlink with thin pointer → .cyberos/AGENT-ENTRY.md"
elif is_platform_repo && { [ -L "$root/AGENTS.md" ] || is_protocol_dump; }; then
  AGENTS_SET="kept platform AGENTS.md (Layer-1 protocol source; entry at .cyberos/AGENT-ENTRY.md)"
elif [ -L "$root/AGENTS.md" ] && is_protocol_dump; then
  # Consumer accidentally symlinked root AGENTS.md → protocol source. Replace with thin pointer.
  rm -f "$root/AGENTS.md"
  write_agents_spine
  AGENTS_SET="replaced AGENTS.md protocol-symlink with thin pointer → .cyberos/AGENT-ENTRY.md"
elif [ -L "$root/AGENTS.md" ]; then
  AGENTS_SET="kept your AGENTS.md symlink (not protocol; entry at .cyberos/AGENT-ENTRY.md)"
elif [ ! -f "$root/AGENTS.md" ]; then
  write_agents_spine
  AGENTS_SET="created AGENTS.md (thin pointer → .cyberos/AGENT-ENTRY.md, like CLAUDE.md)"
elif is_protocol_dump; then
  # Mis-install: consumers must not host the dense protocol at root.
  write_agents_spine
  AGENTS_SET="replaced mis-installed memory protocol at root AGENTS.md with thin pointer → .cyberos/AGENT-ENTRY.md"
elif grep -q "$SP_MARK" "$root/AGENTS.md" 2>/dev/null \
  || grep -qE 'cyberos-agent-spine \(managed by cyberos' "$root/AGENTS.md" 2>/dev/null; then
  # Refresh managed pointer every install so it tracks AGENT-ENTRY wording.
  write_agents_spine
  AGENTS_SET="refreshed AGENTS.md thin pointer → .cyberos/AGENT-ENTRY.md"
elif grep -q '\.cyberos/AGENT-ENTRY\.md' "$root/AGENTS.md" 2>/dev/null; then
  AGENTS_SET="kept your AGENTS.md (already points at .cyberos/AGENT-ENTRY.md)"
else
  { printf '\n---\n\n'; agents_spine; printf '\n<!-- %s -->\n' "$SP_MARK"; } >> "$root/AGENTS.md"
  AGENTS_SET="appended CyberOS pointer to your AGENTS.md"
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
      { printf -- '---\ndescription: CyberOS task workflow (HITL-gated). Always apply.\nalwaysApply: true\n---\n'
        printf 'This repo runs CyberOS. Canonical instructions: AGENTS.md (root) and .cyberos/AGENT-ENTRY.md.\n'
        printf 'Work is tasks; HITL is required at the two human-acceptance gates; run gates with `bash .cyberos/cuo/gates/run-gates.sh`. Never push/deploy/merge without an operator instruction.\n'; } > "$abs" ;;
    plain)
      { printf 'This repo runs CyberOS. Canonical instructions: AGENTS.md (root) and .cyberos/AGENT-ENTRY.md.\n'
        printf 'Work is tasks; HITL is required at the two human-acceptance gates; gates: bash .cyberos/cuo/gates/run-gates.sh. Never push/deploy/merge without an operator instruction.\n'; } > "$abs" ;;
    *)
      { printf '# %s\n\n' "$(basename "$rel" .md)"
        printf 'This repo runs **CyberOS**. Canonical agent instructions: `AGENTS.md` (root) and `.cyberos/AGENT-ENTRY.md`.\n\n'
        printf 'Work is tasks; HITL is required at the two human-acceptance gates; run gates with `bash .cyberos/cuo/gates/run-gates.sh`. Never push, deploy, or merge without an explicit operator instruction.\n'; } > "$abs" ;;
  esac
  AGENT_FILES="$AGENT_FILES $rel"
}
# TASK-IMP-121 §1.1: capture whether the two .agents containers exist BEFORE any mkdir -p that could
# create them. The antigravity pointer below (.agents/rules/cyberos.md, whose dirname mkdir at ~575
# creates .agents) runs BEFORE the .agents/skills mkdir (~669), and `mkdir -p` is silently ok on an
# existing dir AND creates every chain level - so a test taken AFTER any mkdir would falsely read an
# operator's pre-existing dir as installer-created. We mark a parent (below, after the skills block)
# ONLY when install itself created it, so uninstall reclaims what it made and leaves what predated it.
# Two independent flags: .agents has a second creator (~669) and needs its own gate, not .agents/skills'.
_agents_pre=0
_agents_skills_pre=0
[ -d "$root/.agents" ]        && _agents_pre=1
[ -d "$root/.agents/skills" ] && _agents_skills_pre=1
pointer claude-code   CLAUDE.md                          md      # Claude Code CLI (Command Code also reads CLAUDE.md)
pointer gemini        GEMINI.md                          md      # Gemini CLI + Antigravity (GEMINI.md wins on conflict)
pointer cursor        .cursorrules                       plain   # Cursor (legacy rules file)
pointer cursor        .cursor/rules/cyberos.mdc          mdc     # Cursor (modern .cursor/rules/*.mdc)
pointer grok          .grok/GROK.md                      md      # Grok CLI (superagent-ai)
pointer copilot       .github/copilot-instructions.md    md      # GitHub Copilot
pointer antigravity   .agents/rules/cyberos.md           md      # Antigravity / zcode workspace rules (.agents/rules/)
pointer windsurf      .windsurfrules                     plain   # Windsurf (legacy file - still read post-rebrand, kept; TASK-IMP-094)
pointer devin         .devin/rules/cyberos.md            md      # Devin Desktop (Windsurf rebrand, June 2026; .devin/rules/ preferred)
pointer windsurf      .windsurf/rules/cyberos.md         md      # Windsurf rules dir (rebrand fallback beside the legacy file)
# Codex, zcode, Command Code, Aider, Zed, Jules, Warp, OpenCode read AGENTS.md -> covered by the spine.

# --- native skill install: drop ship-tasks into each skill-aware agent's dir ---
# so it is invocable natively (/ship-tasks, $ship-tasks) - not just prose.
# Default = relative symlink into the self-contained skill at .cyberos/plugin/skills (tracks
# updates on re-install; regenerable, so gitignored). CYBEROS_COPY_SKILLS=1 copies it instead.
SKILL_SRC="$CY/plugin/skills/ship-tasks"
relup() { local up="" seg; local IFS=/; for seg in $1; do [ -n "$seg" ] && up="../$up"; done; printf '%s' "$up"; }
install_skill() {           # $1 = agent skills dir (rel to root), $2 = agent key, $3 = skill (default ship-tasks; TASK-IMP-094)
  want_agent "$2" || return 0
  local skill="${3:-ship-tasks}" src="$CY/plugin/skills/${3:-ship-tasks}"
  [ -d "$src" ] || return 0
  local dir="$root/$1" dest="$root/$1/$skill"
  if [ -e "$dest" ] || [ -L "$dest" ]; then         # already there: only refresh OUR own link/copy
    case "$(readlink "$dest" 2>/dev/null)" in *".cyberos/plugin/skills/$skill") : ;; *) return 0;; esac
    rm -rf "$dest" 2>/dev/null || return 0
  fi
  mkdir -p "$dir"
  if [ "${CYBEROS_COPY_SKILLS:-0}" = "1" ]; then
    cp -R "$src" "$dest"
  else
    ln -s "$(relup "$1").cyberos/plugin/skills/$skill" "$dest" 2>/dev/null || cp -R "$src" "$dest"
  fi
  # One shape for every entry: the FULL skill path. TASK-IMP-094 made install_skill
  # multi-skill, and the ship-tasks arm kept appending its containing dir (.claude/skills)
  # while the new arms appended full paths - so the summary's "native skills:" line mixed
  # two shapes (PR-review, Devin 2026-07-17: cosmetic, no functional consequence, but the
  # line is read by operators). Full paths for all: it is the more informative shape, and
  # it is what the shared .agents/skills entries already report.
  SKILL_DIRS="$SKILL_DIRS $1/$skill"
}
install_skill .claude/skills      claude-code    # Claude Code
install_skill .grok/skills        grok           # Grok CLI
install_skill .commandcode/skills command-code   # Command Code
install_skill .codex/skills       codex          # Codex CLI (skills)
install_skill .opencode/skill     opencode       # OpenCode (singular 'skill')
# /create-tasks runs the task-author + task-audit pair; they land beside ship-tasks for the
# claude-code family so the shared .agents/skills entries below have in-repo counterparts
# to point at (TASK-IMP-094).
install_skill .claude/skills      claude-code task-author
install_skill .claude/skills      claude-code task-audit
# zcode + Hermes load skills from a global home ($HOME); opt in with CYBEROS_GLOBAL_SKILLS=1.
if [ "${CYBEROS_GLOBAL_SKILLS:-0}" = "1" ]; then
  for gp in "$HOME/.claude/skills" "$HOME/.grok/skills" "$HOME/.hermes/skills" "$HOME/.commandcode/skills"; do
    [ -e "$gp/ship-tasks" ] || { mkdir -p "$gp" && cp -R "$SKILL_SRC" "$gp/ship-tasks" 2>/dev/null && SKILL_DIRS="$SKILL_DIRS ~${gp#"$HOME"}"; }
  done
fi

# --- shared project skills dir: .agents/skills/ (Agent Skills open standard) ---
# ONE dir read natively by Codex, Copilot, Cursor, Gemini CLI and OpenCode (2026-07-16
# channel research; RELEASE-CHECKLIST.md matrix + line E3). Entries are the three workflow
# commands' skills - ship-tasks (/ship-tasks) plus task-author + task-audit (the pair
# behind /create-tasks) - each a RELATIVE symlink to its .claude/skills/<cmd> copy so the
# skill stays single-sourced; where a resolving symlink cannot exist (no symlink support,
# CYBEROS_COPY_SKILLS=1, or the claude-code family filtered off so the counterpart is
# absent) a plain copy of the payload skill lands instead. Create-if-absent: an entry an
# operator put there is never touched. (TASK-IMP-094)
SHARED_CMDS="ship-tasks task-author task-audit"
if want_agent agents; then
  for _sc in $SHARED_CMDS; do
    [ -d "$CY/plugin/skills/$_sc" ] || continue
    _sdest="$root/.agents/skills/$_sc"
    # Create-if-absent is deliberate idempotence: an entry that landed as a COPY (its
    # .claude/skills counterpart was filtered off at first install) stays a copy on later
    # installs even after the counterpart appears - no churn of existing trees. Re-vendor
    # from scratch (uninstall + install) if the symlink form is wanted. (PR-review note,
    # Devin 2026-07-17: behavioral nuance, not a defect.)
    { [ -e "$_sdest" ] || [ -L "$_sdest" ]; } && continue
    mkdir -p "$root/.agents/skills"
    if [ "${CYBEROS_COPY_SKILLS:-0}" != "1" ] && [ -e "$root/.claude/skills/$_sc" ] \
       && ln -s "$(relup ".agents/skills").claude/skills/$_sc" "$_sdest" 2>/dev/null && [ -e "$_sdest" ]; then
      :                                              # relative link resolves via .claude/skills
    else
      rm -f "$_sdest" 2>/dev/null || true            # never leave a dangling link behind
      cp -R "$CY/plugin/skills/$_sc" "$_sdest"
      # PR-review fix (Devin, 2026-07-17): mark the copy as OURS. The symlink arm proves
      # ownership by readlink target; a copy had no such proof, so uninstall's "dir with a
      # SKILL.md" heuristic would rm -rf an operator's own .agents/skills/<cmd>/ that happens
      # to carry one - exactly what spec §1.3 promises never to touch. A marker file is the
      # copy's readlink: uninstall removes only what carries it.
      printf '%s\n' "cyberos install marker (TASK-IMP-094) - this directory is an installer" \
        "copy-fallback of .cyberos/plugin/skills/$_sc. uninstall.sh removes ONLY directories" \
        "carrying this file. Delete it to adopt the directory as your own; uninstall will" \
        "then leave it alone." > "$_sdest/.cyberos-owned"
    fi
    SKILL_DIRS="$SKILL_DIRS .agents/skills/$_sc"
  done
fi

# --- TASK-IMP-121 §1.1: parent .cyberos-owned markers on the containers install CREATED ---
# uninstall.sh reclaimed .agents/skills then .agents on EMPTINESS alone (no ownership test), so an
# operator's pre-existing empty .agents/skills - and .agents when the antigravity pointer is filtered
# off - was destroyed on uninstall. Mirror the child copy-fallback marker (~684, TASK-IMP-094) one
# directory up: mark a container ONLY when install itself created it (decided by the pre-mkdir
# existence captured above), so uninstall removes only what we made. The marker text names the dir
# and carries the same adoption sentence the child marker gives, so deleting it re-adopts the dir.
_mark_parent() {            # $1 = container dir, relative to root
  printf '%s\n' \
    "cyberos install marker (TASK-IMP-121) - cyberos install CREATED this directory ($1)." \
    "uninstall.sh removes this container ONLY while it carries this file AND it is otherwise empty." \
    "Delete it to adopt the directory as your own; uninstall will then leave it alone." \
    > "$root/$1/.cyberos-owned"
}
if [ "$_agents_skills_pre" = 0 ] && [ -d "$root/.agents/skills" ]; then _mark_parent ".agents/skills"; fi
if [ "$_agents_pre" = 0 ]        && [ -d "$root/.agents" ];        then _mark_parent ".agents";        fi

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

# 6. gitignore - ONE managed block, regenerated in place on every install ------
# Policy (what install writes, tracked vs ignored):
#   TRACKED  - the agent surface (AGENTS.md, CLAUDE.md, GEMINI.md, .cursorrules, .cursor/rules/,
#              .grok/GROK.md, .github/copilot-instructions.md, .agents/rules/, .windsurfrules,
#              .devin/rules/cyberos.md, .windsurf/rules/cyberos.md (TASK-IMP-094 pointers),
#              .mcp.json, .cursor/mcp.json), docs/tasks/** (BACKLOG + specs/audits),
#              CHANGELOG.md, docs/status.html (the generated status page - the repo's published
#              Roadmap/Backlog/Changelog view), and skill COPIES (CYBEROS_COPY_SKILLS=1).
#   IGNORED  - .cyberos/ (vendored machine + BRAIN store + render intermediates: regenerable /
#              tenant data) and the skill SYMLINKS (they point INTO the ignored .cyberos/).
# Mechanics: everything lives between the two markers below; re-install REPLACES the block content
# (never appends duplicates), anything outside the markers is never touched, and the scattered
# entries older inits appended (pre-block) are lifted into the block on first contact.
gi="$root/.gitignore"
[ -f "$gi" ] || : > "$gi"
GI_BEGIN="# >>> cyberos (managed block - regenerated by cyberos install; edit outside the markers) >>>"
GI_END="# <<< cyberos <<<"
gi_block() {
  printf '%s\n' "$GI_BEGIN"
  printf '%s\n' "# vendored machine + local BRAIN store + render intermediates (regenerable via install; tenant data)"
  printf '%s\n' ".cyberos/"
  if [ "${CYBEROS_COPY_SKILLS:-0}" != "1" ]; then
    printf '%s\n' "# skill symlinks -> .cyberos/plugin/skills (regenerable via install)"
    for sp in .claude/skills/ship-tasks .claude/skills/task-author .claude/skills/task-audit .grok/skills/ship-tasks .commandcode/skills/ship-tasks .codex/skills/ship-tasks .opencode/skill/ship-tasks; do
      printf '%s\n' "$sp"
    done
    printf '%s\n' "# shared .agents/skills entries (Agent Skills open standard) chain via .claude/skills (TASK-IMP-094)"
    for sp in .agents/skills/ship-tasks .agents/skills/task-author .agents/skills/task-audit; do
      printf '%s\n' "$sp"
    done
  fi
  printf '%s\n' "$GI_END"
}
# Strip any prior managed block, then trim the trailing blank run and append the fresh block.
#
# The begin marker is matched by SHAPE (/^# >>> cyberos .*>>>$/), never by exact text. An
# exact match cannot survive its own wording changing: renaming `cyberos install` ->
# `cyberos install` inside the marker made every already-installed block unstrippable, so
# install appended a SECOND block beside the first and left the original's comment lines
# orphaned. That shipped to 21 of 23 repos. A marker that identifies a block must not depend
# on prose nobody thinks of as an interface.
awk -v e="$GI_END" '
  /^# >>> cyberos .*>>>$/ {inblk=1; next} $0==e {inblk=0; next} inblk {next}
  # orphans from a block whose marker changed before the shape-match fix landed
  /^# vendored machine \+ local BRAIN store \+ render intermediates \(regenerable via/ {next}
  /^# skill symlinks -> \.cyberos\/plugin\/skills \(regenerable via/ {next}
  $0==".cyberos/" {next}
  $0==".claude/skills/ship-tasks" {next}
  $0==".grok/skills/ship-tasks" {next}
  $0==".commandcode/skills/ship-tasks" {next}
  $0==".codex/skills/ship-tasks" {next}
  $0==".opencode/skill/ship-tasks" {next}
  {lines[++n]=$0}
  END { while (n>0 && lines[n] ~ /^[[:space:]]*$/) n--; for (i=1;i<=n;i++) print lines[i] }
' "$gi" > "$gi.cyberos.tmp"
{ cat "$gi.cyberos.tmp"; [ -s "$gi.cyberos.tmp" ] && printf '\n'; gi_block; } > "$gi"
rm -f "$gi.cyberos.tmp"

# 6b. status auto-sync hook (managed; CYBEROS_NO_HOOK=1 skips) -----------------
# docs/status/ must stay synced with the markdown it renders (task frontmatter, CHANGELOG.md,
# VERSION). Touchpoints: run-gates.sh after gates, and this pre-commit when inputs are staged.
# v2: blocking on regen failure + pipefail-safe staged list (never `git diff | grep -q`).
# An existing foreign pre-commit is never replaced - we append a marked block once.
HOOK_SET="skipped (CYBEROS_NO_HOOK=1)"
if [ "${CYBEROS_NO_HOOK:-0}" != "1" ]; then
  if [ ! -d "$root/.git" ]; then
    HOOK_SET="skipped (not a git checkout)"
  else
    # Resolve the EFFECTIVE hooks directory (TASK-IMP-083). git executes hooks from
    # core.hooksPath when that config is set - a relative value anchors at the repo root,
    # an absolute one is used as is - and falls back to .git/hooks only when it is unset.
    # Writing .git/hooks/pre-commit unconditionally on a hooksPath repo (the cyberos repo
    # itself is one) installs a hook git never runs: install prints success and status
    # sync silently dies - the exact "indistinguishable from success" class documented
    # elsewhere in this file. Empty output (unset OR set to "") takes the default branch,
    # and $hook_at stays empty there, so on no-hooksPath repos every written byte and
    # every summary word below expands exactly as before this resolver existed.
    hooks_path="$(git -C "$root" config core.hooksPath 2>/dev/null || true)"
    hook_at=""
    if [ -z "$hooks_path" ]; then
      hooks_dir="$root/.git/hooks"
    else
      case "$hooks_path" in
        /*) hooks_dir="$hooks_path" ;;
        *)  hooks_dir="$root/${hooks_path%/}" ;;
      esac
      hook_at=" at ${hooks_path%/}/pre-commit"
    fi
    hk="$hooks_dir/pre-commit"
    mkdir -p "$hooks_dir"

    # Do we own this file OUTRIGHT? Exact, not positional.
    #
    # This was `head -5 "$hk" | grep -q cyberos-status-hook` — a heuristic that asked
    # "is our marker near the top?" instead of "is this our file?". The two differ for a
    # foreign hook SHORTER than 5 lines: install #1 appends our marked block, the marker
    # lands at line 4, install #2 reads it inside head -5, concludes it owns the file, and
    # `cat >` DESTROYS the user's hook. Reproduced:
    #
    #   foreign hook 3 lines  -> marker inside head -5  -> foreign body DESTROYED on re-install
    #   foreign hook 10 lines -> marker outside head -5 -> foreign body survives
    #
    # Silent data loss whose trigger is the LENGTH of someone else's file. It matters now
    # because `rm -rf .cyberos && install` does not touch .git/hooks/, so every re-install
    # re-enters this branch against a hook the previous install already appended to.
    #
    # Our standalone form always carries the managed header on line 2. The appended form
    # is marked `>>>` and belongs to whoever owns the lines above it. Line 2 + the `>>>`
    # exclusion separates them exactly, at any file length.
    _cyberos_owns_hook() {
      [ -f "$1" ] || return 1
      local l2; l2="$(sed -n '2p' "$1" 2>/dev/null)"
      case "$l2" in
        *'>>>'*)                    return 1 ;;   # the APPENDED form — the file is theirs
        '# cyberos-status-hook'*)   return 0 ;;   # our managed standalone header
        *)                          return 1 ;;
      esac
    }

    if [ ! -f "$hk" ] || _cyberos_owns_hook "$hk"; then
      # absent, or a hook WE own outright (managed header on line 2): (re)write the standalone form
      cat > "$hk" <<'HOOK'
#!/usr/bin/env bash
# cyberos-status-hook v2 (managed by cyberos install)
# Regenerates docs/status/ when task sources change and STAGES it in the same commit.
# Blocks the commit if regeneration fails (so status never lags GitHub).
# Disable: delete this file, or re-install with CYBEROS_NO_HOOK=1.
set -euo pipefail
# Read staged list ONCE — never `git diff | grep -q` under pipefail (SIGPIPE skip bug).
staged="$(git diff --cached --name-only || true)"
if grep -Eq '^(docs/tasks/|CHANGELOG\.md$|VERSION$)' <<<"$staged"; then
  # An ORPHANED hook stands down; it does not veto the commit. .git/hooks/ is untracked
  # and machine-local, so this file outlives .cyberos/ by construction - deleting the
  # machine without running uninstall (or cloning a repo whose .cyberos/ is ignored)
  # leaves the hook behind with nothing to call. It guards a RENDERED STATUS PAGE, a
  # cosmetic concern, so exiting 1 here blocked every commit touching VERSION,
  # CHANGELOG.md or docs/tasks/ for no correctness gain. Fail-closed is for gates that
  # guard correctness; this is not one. uninstall.sh removes this hook on the tidy path
  # (:93-128) - this branch covers the untidy one, which is the common one.
  if [ ! -f .cyberos/lib/status-page.sh ] || [ ! -f .cyberos/lib/task-migrate.sh ]; then
    echo "cyberos: hook is orphaned (.cyberos/ missing) - skipping status regen." >&2
    echo "cyberos: restore with 'npx cs install' (or bash …/install.sh), or delete .git/hooks/pre-commit." >&2
    exit 0
  fi
  if ! command -v node >/dev/null 2>&1; then
    echo "cyberos: ERROR node required to regenerate docs/status" >&2
    exit 1
  fi
  echo "cyberos: regenerating docs/status/ …"
  bash .cyberos/lib/status-page.sh . || {
    echo "cyberos: ERROR status regen failed — run: bash .cyberos/lib/status-page.sh ." >&2
    exit 1
  }
  git add docs/status 2>/dev/null || true
  echo "cyberos: docs/status staged"
fi
exit 0
HOOK
      HOOK_SET="pre-commit hook v2 installed${hook_at} (blocks if docs/status regen fails; auto-stages status page)"
    elif grep -q "cyberos-status-hook v2" "$hk" 2>/dev/null; then
      HOOK_SET="kept your pre-commit hook${hook_at} (cyberos status-sync v2 already present)"
    elif grep -q "cyberos-status-hook" "$hk" 2>/dev/null; then
      # Upgrade v1 append block → v2
      if grep -q ">>> cyberos-status-hook v1" "$hk" 2>/dev/null; then
        tmp="$hk.cyberos.tmp"
        sed '/# >>> cyberos-status-hook v1/,/# <<< cyberos-status-hook <<</d' "$hk" > "$tmp" && mv "$tmp" "$hk"
      fi
      if ! grep -q "cyberos-status-hook v2" "$hk" 2>/dev/null; then
        cat >> "$hk" <<'HOOK'

# >>> cyberos-status-hook v2 (appended by cyberos install; edits above survive re-install) >>>
# POSIX sh ONLY. This block is appended to a hook we did not write, whose shebang is very
# often `#!/bin/sh`. It used `grep -Eq ... <<<"$staged"` — a bash herestring, a SYNTAX
# ERROR under dash — so wherever /bin/sh is dash the hook aborted and the foreign hook's
# exit code was lost. It looked correct on macOS, where /bin/sh is bash in sh-mode and
# herestrings work. (The standalone form above declares `#!/usr/bin/env bash` and may
# keep its bash-isms. This one may not: the shebang is the user's, not ours.)
#
# `case` and not `printf | grep -q`: the pipe is exactly what the standalone form's own
# comment warns about — grep -q exits early, the writer takes SIGPIPE, and under a foreign
# hook's `set -o pipefail` that becomes a spurious failure. A case loop has no pipe and no
# herestring, so it holds under dash, bash, and whatever options the user's hook set.
_cyberos_rc=$?
_cyberos_staged="$(git diff --cached --name-only || true)"
_cyberos_hit=0
for _cyberos_f in $_cyberos_staged; do
  case "$_cyberos_f" in
    docs/tasks/*|CHANGELOG.md|VERSION) _cyberos_hit=1; break ;;
  esac
done
if [ "$_cyberos_hit" = 1 ]; then
  if [ -f .cyberos/lib/status-page.sh ] && command -v node >/dev/null 2>&1; then
    if bash .cyberos/lib/status-page.sh .; then
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
        HOOK_SET="upgraded appended status-sync block to v2${hook_at} (blocking regen)"
      else
        HOOK_SET="kept your pre-commit hook${hook_at} (cyberos status-sync v2 already appended)"
      fi
    elif grep -q "status-page.sh" "$hk" 2>/dev/null; then
      # Reaching here means the hook regenerates docs/status but carries NO cyberos marker: a
      # hand-written or self-hosted hook (this repo's own is exactly this). Appending the managed
      # block would regenerate the page TWICE per commit and bolt an `exit 1` onto a hook whose
      # own comments state a non-blocking posture. What we did not write, we do not "fix".
      #
      # ORDER MATTERS, TWICE OVER:
      #  1. This generic CONTENT check must come AFTER both marker checks. Every marked block, v1
      #     and v2 alike, contains "status-page.sh" (`git log -S "cyberos-status-hook v1"` shows
      #     the v1 block runs `bash .cyberos/lib/status-page.sh .`), so placed first it swallows
      #     them and a v1 hook never upgrades.
      #  2. It belongs HERE, in the installer's if/elif chain - NOT inside the <<'HOOK' heredoc
      #     above. That heredoc is single-quoted: every line is copied verbatim into the USER's
      #     pre-commit hook. A branch placed there never executes at install time, and ships
      #     $hk / ${hook_at} / HOOK_SET into a foreign hook where they are unbound - under
      #     `set -u` that aborts the user's commit.
      # (External review 2026-07-17: pass 2 caught (1), a regression pass 1 introduced; pass 3
      # caught (2), a regression the fix for (1) introduced. Both are recorded because the next
      # editor of this chain will be tempted to make the same two moves.)
      HOOK_SET="kept your pre-commit hook${hook_at} (it already regenerates docs/status - no second block appended)"
    else
      # a FOREIGN hook exists: append a marked block that preserves the foreign exit code
      cat >> "$hk" <<'HOOK'

# >>> cyberos-status-hook v2 (appended by cyberos install; edits above survive re-install) >>>
# POSIX sh ONLY. This block is appended to a hook we did not write, whose shebang is very
# often `#!/bin/sh`. It used `grep -Eq ... <<<"$staged"` — a bash herestring, a SYNTAX
# ERROR under dash — so wherever /bin/sh is dash the hook aborted and the foreign hook's
# exit code was lost. It looked correct on macOS, where /bin/sh is bash in sh-mode and
# herestrings work. (The standalone form above declares `#!/usr/bin/env bash` and may
# keep its bash-isms. This one may not: the shebang is the user's, not ours.)
#
# `case` and not `printf | grep -q`: the pipe is exactly what the standalone form's own
# comment warns about — grep -q exits early, the writer takes SIGPIPE, and under a foreign
# hook's `set -o pipefail` that becomes a spurious failure. A case loop has no pipe and no
# herestring, so it holds under dash, bash, and whatever options the user's hook set.
_cyberos_rc=$?
_cyberos_staged="$(git diff --cached --name-only || true)"
_cyberos_hit=0
for _cyberos_f in $_cyberos_staged; do
  case "$_cyberos_f" in
    docs/tasks/*|CHANGELOG.md|VERSION) _cyberos_hit=1; break ;;
  esac
done
if [ "$_cyberos_hit" = 1 ]; then
  if [ -f .cyberos/lib/status-page.sh ] && command -v node >/dev/null 2>&1; then
    if bash .cyberos/lib/status-page.sh .; then
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
      HOOK_SET="appended status-sync v2 to your existing pre-commit hook${hook_at}"
    fi
    chmod +x "$hk"
  fi
fi

# 7. tell the operator what to do next ----------------------------------------
cat <<EOF

cyberos install: done.
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
              + the skill symlinks; agent files, docs/tasks/**, CHANGELOG.md and
              docs/status/ stay TRACKED (commit them). Everything outside the block is yours.
  version   -> CyberOS $avail_ver (.cyberos/VERSION); check: bash .cyberos/version.sh  (auto soft-check on any .cyberos use)${_cy_downgrade_note:+
  DOWNGRADE -> $_cy_downgrade_note}

Next:
  1. Write a task from the template:
       mkdir -p docs/tasks/<module>/TASK-001-<slug> && cp .cyberos/cuo/templates/TASK-TEMPLATE.md docs/tasks/<module>/TASK-001-<slug>/spec.md
       # fill in section 1, set status: ready_to_implement, add the row to BACKLOG.md
  2. Trigger the workflow in your agent (Claude Code / Cowork / Codex):
       "Follow .cyberos/cuo/ship-tasks.md and drive the next eligible task in
        docs/tasks/BACKLOG.md. HITL is required: halt at the two human-acceptance
        gates. repo_root is this repo."
  3. Run the machine gates any time:
       bash .cyberos/cuo/gates/run-gates.sh
  4. Clean your task corpus of FM-001 trailing frontmatter comments (older specs, and any born
     from a pre-2026-07 template, carry them). Preview first, then rewrite in place:
       node .cyberos/docs-tools/fm001-migrate.mjs --check docs/tasks/*/*/spec.md   # report only
       node .cyberos/docs-tools/fm001-migrate.mjs docs/tasks/*/*/spec.md           # rewrite
     It only moves trailing comments to their own line, never touches the body, is idempotent,
     and refuses any path not tracked at HEAD.

Every popular agent is wired: AGENTS.md is the cross-agent spine, and Claude Code, Codex,
Cursor, Gemini, Antigravity, Grok CLI, zcode, Command Code, Copilot, Windsurf & Devin each get the
right pointer file / native skill / MCP registration (all create-if-absent; your files are
never clobbered). Restrict with CYBEROS_AGENTS=..., copy skills with CYBEROS_COPY_SKILLS=1,
skip MCP with CYBEROS_NO_MCP=1. MCP server + per-agent registration snippets: .cyberos/mcp/README.md.

BRAIN memory protocol: .cyberos/memory/store/ is your local memory store (gitignored, tenant
data). The rules are in .cyberos/memory/AGENTS.md (root AGENTS.md is a thin pointer to
.cyberos/AGENT-ENTRY.md, like CLAUDE.md). An agent working here records decisions, audits,
and plans into the BRAIN per that protocol. Skip with CYBEROS_NO_MEMORY=1 on install.
EOF

# Say the quiet part where it is cheap to learn (TASK-IMP-096): ship-tasks needs commits,
# diff-scoped coverage, and route-back restores - none exist without git. Same truth the
# root resolution at the top of this file uses (git rev-parse), NOT a -d .git probe: a
# worktree or submodule checkout where .git is a FILE counts as a checkout and stays silent.
if ! git -C "$root" rev-parse --show-toplevel >/dev/null 2>&1; then
  echo "cyberos install: this repo is not a git checkout - ship-tasks needs one; run: git init -b main && git add -A && git commit -m init"
fi

# 8. payload self-cleanup ------------------------------------------------------
# A payload COPIED INSIDE the target repo at the canonical channel-1 location
# (cp -R dist/cyberos <repo>/.cyberos-install) is redundant once install succeeds: everything the
# repo needs onward lives under .cyberos/ (machine, gates, migration kit, MCP server).
# Remove it so it is never committed by accident. ONLY <root>/.cyberos-install is ever removed -
# never payloads outside the repo (dev checkouts / dist), never other in-repo paths, never a
# git submodule/checkout (channel 2 - carries a .git entry), and never with CYBEROS_KEEP_PAYLOAD=1.
if [ "$src" = "$root/.cyberos-install" ]; then
  if [ "${CYBEROS_KEEP_PAYLOAD:-0}" = "1" ]; then
    echo "payload: kept .cyberos-install/ (CYBEROS_KEEP_PAYLOAD=1 - remember it is untracked; .cyberos/ is what the repo runs on)"
  elif [ -e "$src/.git" ] || { [ -f "$root/.gitmodules" ] && grep -q "path = .cyberos-install" "$root/.gitmodules"; }; then
    echo "payload: kept .cyberos-install/ (git submodule/checkout - never auto-removed)"
  else
    # two attempts with a settle pause (network/virtual mounts defer the final rmdir);
    # never abort install over cleanup
    rm -rf "$src" 2>/dev/null || true
    if [ -e "$src" ]; then sleep 1; rm -rf "$src" 2>/dev/null || true; fi
    if [ ! -e "$src" ]; then
      echo "payload: removed .cyberos-install/ (self-cleanup - everything now lives in .cyberos/; keep it next time with CYBEROS_KEEP_PAYLOAD=1)"
    elif [ -d "$src" ] && [ -z "$(ls -A "$src" 2>/dev/null)" ]; then
      # network/virtual mounts defer the last unlink while this script's own file handle is open
      echo "payload: emptied .cyberos-install/; the folder handle is held by this run (network-mount quirk) - finish with: rmdir .cyberos-install"
    else
      echo "payload: WARN could not fully remove .cyberos-install/ - delete it manually; everything the repo needs now lives in .cyberos/"
    fi
  fi
else
  case "$src/" in
    "$root"/*) if [ "$src" != "$root" ]; then echo "payload: kept ${src#"$root"/}/ (non-canonical in-repo location - only <repo>/.cyberos-install self-cleans; it is redundant after init and safe to delete)"; fi ;;
    *) : ;;   # payload lives outside the target repo - not ours to touch
  esac
fi
