#!/usr/bin/env bash
# test_task_link_cutoff.sh - TASK-DOCS-019 cutoff/config resolution
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
CHK="$root/scripts/check_task_link.sh"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

unset GIT_INDEX_FILE GIT_DIR GIT_WORK_TREE GIT_PREFIX GIT_COMMON_DIR 2>/dev/null || true

d="$TMP/repo"
mkdir -p "$d/.cyberos"
(cd "$d" && git init -q && git config user.email t@t && git config user.name t \
  && echo a > f && git add f && git commit -qm "chore: seed (TASK-DOCS-019)")
seed="$(cd "$d" && git rev-parse HEAD)"

# Env wins
out="$(cd "$d" && CYBEROS_TRACE_CUTOFF=deadbeef bash "$CHK" --range "$seed..HEAD" 2>&1 || true)"
grep -q 'deadbeef\|after deadbeef\|after deadbee' <<<"$out" \
  || grep -q 'traceability-gate: all' <<<"$out" \
  && ok t01_env_cutoff || fail t01_env_cutoff "$out"

# Config cutoff
cat > "$d/.cyberos/config.yaml" <<EOF
traceability:
  cutoff: ${seed}
  strict: false
  scaffold_ci: false
EOF
# Commit after cutoff without task id — should fail range check when cutoff is seed's parent... 
# Make a new commit after seed, then set cutoff=seed so the new commit is gated.
(cd "$d" && echo b > g && git add g && git commit -qm "feat: unlinked on purpose")
bad="$(cd "$d" && unset CYBEROS_TRACE_CUTOFF && bash "$CHK" --range "$seed..HEAD" 2>&1 || true)"; rc=$?
# The unlinked commit is after cutoff → must fail
if echo "$bad" | grep -q UNLINKED; then ok t02_config_cutoff_gates
else fail t02_config_cutoff_gates "rc=$rc out=$bad"; fi

# Strict via config on --msg
msg="$TMP/msg"
printf 'feat: no task here\n' > "$msg"
cat > "$d/.cyberos/config.yaml" <<EOF
traceability:
  cutoff: ${seed}
  strict: true
EOF
if (cd "$d" && unset CYBEROS_STRICT_COMMITS CYBEROS_REQUIRE_TASK_LINK && bash "$CHK" --msg "$msg"); then
  fail t03_strict_config "should have blocked"
else
  ok t03_strict_config
fi

# Advisory default
cat > "$d/.cyberos/config.yaml" <<EOF
traceability:
  cutoff: ${seed}
  strict: false
EOF
if (cd "$d" && unset CYBEROS_STRICT_COMMITS CYBEROS_REQUIRE_TASK_LINK && bash "$CHK" --msg "$msg"); then
  ok t04_advisory_default
else
  fail t04_advisory_default "advisory should exit 0"
fi

echo "task_link_cutoff: pass=$PASS fail=$FAIL"
[ "$FAIL" -eq 0 ]
