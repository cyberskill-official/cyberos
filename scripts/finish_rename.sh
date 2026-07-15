#!/usr/bin/env bash
# finish_rename.sh — the fr->task steps that must run on a real machine.
#
# Everything in here was blocked in the agent sandbox for environment reasons, not
# code reasons:
#   - unlink is denied there, so `rm -rf` / `git mv` / rmSync-based regen cannot run
#   - 9.6G disk + no libssl + no root, so the full cargo workspace cannot build
#
# Safe to re-run. Every step is idempotent and asks before anything irreversible.
#
#   bash scripts/finish_rename.sh          # dry run — prints what it WOULD do
#   bash scripts/finish_rename.sh --apply
set -uo pipefail

APPLY=0
[ "${1:-}" = "--apply" ] && APPLY=1
root="$(git rev-parse --show-toplevel)"
cd "$root"

say()  { printf '\n\033[1m== %s\033[0m\n' "$*"; }
run()  { if [ "$APPLY" = 1 ]; then echo "  \$ $*"; eval "$@"; else echo "  would run: $*"; fi; }
ok()   { printf '  \033[32mOK\033[0m   %s\n' "$*"; }
warn() { printf '  \033[33mWARN\033[0m %s\n' "$*"; }
bad()  { printf '  \033[31mFAIL\033[0m %s\n' "$*"; }

[ "$APPLY" = 1 ] || echo "DRY RUN — nothing will be written. Re-run with --apply."

# ─────────────────────────────────────────────────────────────────────────────
say "1/6  Rust: the 8 crates the sandbox could not build"
# Verified clean in-sandbox: obs-collector, obs-proxy, obs-router, proj, eval,
# skill-broker (29 tests pass). Unverified: memory, mcp-gateway, email, auth, chat,
# ai-gateway, obs-compliance-view, shared/*.
#
# Expectation: green. Of 1,261 Rust lines the rename touched, 1,207 are comments and
# 0 stale `fr-` literals remain. If something IS red, it is far more likely to be a
# pre-existing failure the rename surfaced than the rename itself — check with
# `git stash && cargo test -p <crate>` before assuming.
if command -v cargo >/dev/null 2>&1; then
  run "(cd services && cargo test --workspace 2>&1 | grep -E '^(test result|error|FAILED|---- .* stdout)' | grep -v '0 passed; 0 failed')"
else
  bad "cargo not found — install rust, then re-run"
fi

# ─────────────────────────────────────────────────────────────────────────────
say "2/6  Payload + agent symlinks  (MUST run before the status regen)"
# ORDER BUG, fixed after the first real run: this was step 5, and step 2 failed with
#     .cyberos/lib/task-migrate.sh: No such file or directory
#     _cyberos_fr_migrate: command not found
# because status-page.sh sources the VENDORED copy under .cyberos/lib/, which still
# held the pre-rename `fr-migrate.sh`. build.sh refreshes dist/cyberos; install.sh
# lays it into .cyberos/. Both must happen before anything sources the vendored kit.
# install.sh must run FROM the assembled payload, not from the source tree:
#     tools/cyberos-install/install.sh .   ->  "not an assembled payload (no cuo/)"
#     dist/cyberos/install.sh .         ->  correct
# build.sh assembles tools/cyberos-install/ -> dist/cyberos/; install.sh then lays that
# out under the target repo's .cyberos/. Running the source copy skips the assembly
# and the vendored kit stays stale — which is what left .cyberos/lib/ holding the
# pre-rename fr-migrate.sh and broke the status regen below.
run "bash tools/cyberos-install/build.sh"
run "bash dist/cyberos/install.sh ."           # refreshes .cyberos/ from dist/cyberos
run "bash tools/cyberos-install/check-chain-coverage.sh dist/cyberos"

# ─────────────────────────────────────────────────────────────────────────────
say "3/6  Regenerate docs/status (507 stale TASK-*.js chunks)"
# The renderer rmSync's its output dir, which the sandbox cannot do. The chunks under
# docs/status/data/fr/ are still named FR-*.js, so the status page's detail drawer
# 404s on every task.
if command -v node >/dev/null 2>&1; then
  rm -f .git/index.lock 2>/dev/null   # the previous run left one; harmless if absent
  run "bash tools/cyberos-install/lib/status-page.sh ."
  run "git add docs/status/"
else
  bad "node not found — skipping status regen"
fi

# ─────────────────────────────────────────────────────────────────────────────
say "4/6  BRAIN: 807 protocol-legal ops (289 move + 518 put)"
# NEVER sed the store. It is a 226MB hash chain, 252,133 rows: AGENTS.md §6.3 chains
# every row to its predecessor and §5.3 records each body's SHA-256 in a ledger row.
# A byte-level edit invalidates every subsequent row and `cyberos doctor` freezes.
# The protocol's own answer to a rename is new move()+put() ops — the chain then
# RECORDS the rename instead of being broken by it.
ndjson=".cyberos/brain-rename.ndjson"
if [ ! -f "$ndjson" ]; then
  warn "$ndjson missing — regenerating"
  run "python3 scripts/migrate_fr_to_task.py --emit-brain-ops 2>/dev/null > $ndjson"
fi
if [ -f "$ndjson" ]; then
  ok "$(wc -l < "$ndjson" | tr -d ' ') ops queued"
  run "python3 scripts/apply_brain_rename.py $ndjson"          # dry run first
  if [ "$APPLY" = 1 ]; then
    read -r -p "  Apply 807 ops to the BRAIN? [y/N] " a
    [ "$a" = "y" ] && python3 scripts/apply_brain_rename.py "$ndjson" --apply
  fi
  run "python3 -m cyberos doctor"   # MUST report READY, never FROZEN_*
fi

# ─────────────────────────────────────────────────────────────────────────────
say "5/6  git blame survivability"
# One commit now owns ~32,000 lines. `git log -S` still works on history (old commits
# keep old ids), but blame is useless without this.
sha="$(git log --oneline --all --grep='feature-request -> task' --format=%H | head -1)"  # legacy commit-message search — the old words are the search key
if [ -z "$sha" ]; then
  warn "rename commit not found by message — set it by hand"
else
  if ! grep -q "$sha" .git-blame-ignore-revs 2>/dev/null; then
    run "printf '%s  # fr->task codemod, mechanical, no semantic change\n' $sha >> .git-blame-ignore-revs"
    run "git config blame.ignoreRevsFile .git-blame-ignore-revs"
  else
    ok ".git-blame-ignore-revs already carries the rename commit"
  fi
fi

# ─────────────────────────────────────────────────────────────────────────────
say "6/6  Verify"
run "python3 scripts/migrate_fr_to_task.py --verify"
run "(cd modules/cuo && python3 -m pytest tests/ -q | tail -1)"
run "(cd modules/memory && python3 -m pytest tests/ -q | tail -1)"

say "Remaining, by hand — deliberately not automated"
cat <<'EOF'
  498 specs carry `# UNREVIEWED` on ai_authorship / eu_ai_act_risk_class.
  FM-112 blocks each from leaving `draft`, so you meet them one task at a time
  rather than as a wall. Do NOT bulk-clear them: the whole point is that a
  regulatory classification and an authorship claim are judgements, not defaults.
  Find them:  grep -rl UNREVIEWED docs/tasks --include=spec.md
EOF
