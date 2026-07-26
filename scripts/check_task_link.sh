#!/usr/bin/env bash
# CyberOS traceability gate.
#
# THE RULE: every change introduced by a commit must be linked to one or more tasks
# and reflected on the Status page and in the Release Notes. No exceptions.
#
# This script is the single implementation both entry points share:
#   commit-msg hook  ->  check_task_link.sh --msg <msg-file>
#       Advisory by default (prints a loud warning). Blocks when
#       CYBEROS_STRICT_COMMITS=1 or CYBEROS_REQUIRE_TASK_LINK=1.
#   CI gate          ->  check_task_link.sh --range <A..B>
#       Hard-fails on any non-exempt commit NEWER than the cutoff whose message
#       (subject + body) does not cite a canonical task id.
#
# Canonical citation: the full id, e.g. TASK-TEN-208 (a "Task: TASK-TEN-208" trailer
# also works — it contains the id). Shorthand like "TEN-208" satisfied the old habit
# and is still recovered for history on the status page, but does NOT satisfy the gate:
# one grep-able grammar from the cutoff forward.
#
# Exempt (plumbing, not change): chore(release) commits, served-bundle rebuilds,
# merges, reverts, fixups, and [skip ci] automation commits.
#
# Cutoff: commits at or before CUTOFF are history — visible on the status page,
# never retro-failed here. Fixing a historical link happens in the reviewed ledger
# (docs/status-v3-preview/commit-links.yaml; docs/tasks/_state/commit-links.yaml
# after integration), never by rewriting git history.
set -euo pipefail

# Enforcement starts at the commit AFTER this one (set 2026-07-26, status-v3 review).
CUTOFF="${CYBEROS_TRACE_CUTOFF:-a7e0e2121a3750e260a64e44828c0c798cceb045}"
TASK_RE='TASK-[A-Z][A-Z0-9]*-[0-9]+'

exempt_subject() {
  case "$1" in
    "chore(release):"*|"chore(web): rebuild"*|"Merge "*|"Revert "*|fixup!*|squash!*|amend!*) return 0 ;;
  esac
  printf '%s' "$1" | grep -qi '\[skip ci\]' && return 0
  return 1
}

usage() { echo "usage: check_task_link.sh --msg <file> | --range <A..B>" >&2; exit 2; }

mode="${1:-}"; arg="${2:-}"
case "$mode" in

  --msg)
    [ -n "$arg" ] && [ -f "$arg" ] || exit 0
    subject="$(sed -n '1p' "$arg")"
    exempt_subject "$subject" && exit 0
    if grep -Eq "$TASK_RE" "$arg"; then exit 0; fi
    echo "cyberos traceability: no task link in this commit message." >&2
    echo "  Rule: every commit is linked to one or more tasks (Status page + Release Notes)." >&2
    echo "  Fix:  cite the canonical id in the subject or body, e.g.:" >&2
    echo "          feat(ten): host-e overage admission (TASK-TEN-208)" >&2
    echo "        or add a trailer line:  Task: TASK-TEN-208" >&2
    if [ "${CYBEROS_STRICT_COMMITS:-0}" = "1" ] || [ "${CYBEROS_REQUIRE_TASK_LINK:-0}" = "1" ]; then
      echo "  CYBEROS strict mode -> rejecting." >&2
      exit 1
    fi
    echo "  (advisory locally - committing anyway; CI's traceability-gate WILL fail this push.)" >&2
    exit 0
    ;;

  --range)
    [ -n "$arg" ] || usage
    from="${arg%%..*}"
    if ! git rev-parse --verify --quiet "$from^{commit}" >/dev/null 2>&1; then
      echo "traceability-gate: range start '$from' unknown (new branch?) - checking ${CUTOFF:0:8}..HEAD" >&2
      arg="$CUTOFF..HEAD"
    fi
    bad=0
    while IFS= read -r sha; do
      [ -n "$sha" ] || continue
      # commits at/before the cutoff are history, not violations
      if git merge-base --is-ancestor "$sha" "$CUTOFF" 2>/dev/null; then continue; fi
      subject="$(git log -1 --format=%s "$sha")"
      exempt_subject "$subject" && continue
      if git log -1 --format='%s%n%b' "$sha" | grep -Eq "$TASK_RE"; then continue; fi
      echo "UNLINKED  ${sha:0:10}  $subject" >&2
      bad=$((bad + 1))
    done < <(git rev-list --no-merges "$arg" 2>/dev/null || true)
    if [ "$bad" -gt 0 ]; then
      echo "" >&2
      echo "traceability-gate: $bad commit(s) violate the rule (no TASK-* id in message)." >&2
      echo "Amend the messages (git commit --amend / git rebase -i) before merging," >&2
      echo "or split plumbing into an exempt commit type. The rule has no exceptions." >&2
      exit 1
    fi
    echo "traceability-gate: all non-exempt commits after ${CUTOFF:0:8} cite a task."
    ;;

  *) usage ;;
esac
