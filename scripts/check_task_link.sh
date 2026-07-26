#!/usr/bin/env bash
# CyberOS traceability gate.
#
# THE RULE: every change introduced by a commit must be linked to one or more tasks
# and reflected on the Status page and in the Release Notes. No exceptions.
#
# This script is the single implementation both entry points share:
#   commit-msg hook  ->  check_task_link.sh --msg <msg-file>
#       Advisory by default (prints a loud warning). Blocks when
#       CYBEROS_STRICT_COMMITS=1 or CYBEROS_REQUIRE_TASK_LINK=1 or
#       .cyberos/config.yaml: traceability.strict: true.
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
# Cutoff resolution (TASK-DOCS-019): CYBEROS_TRACE_CUTOFF env →
# .cyberos/config.yaml: traceability.cutoff → repo-local default (mothership seed).
# Commits at or before CUTOFF are history — visible on the status page, never
# retro-failed here. Fixing a historical link happens in the reviewed ledger
# (docs/tasks/_state/commit-links.yaml), never by rewriting git history.
set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
TASK_RE='TASK-[A-Z][A-Z0-9]*-[0-9]+'

# Mothership seed (Phase 0); consumer installs overwrite via install.sh.
_DEFAULT_CUTOFF="a7e0e2121a3750e260a64e44828c0c798cceb045"

_read_yaml_trace_key() {
  # Tiny non-nested YAML reader for `traceability:` block keys (cutoff/strict/scaffold_ci).
  local key="$1" file="$2"
  [ -f "$file" ] || return 1
  local in=0 line val
  while IFS= read -r line || [ -n "$line" ]; do
    case "$line" in
      traceability:*) in=1; continue ;;
      [a-zA-Z]*:*) [ "$in" = 1 ] && in=0 ;;
    esac
    [ "$in" = 1 ] || continue
    case "$line" in
      *" ${key}:"*|*"${key}:"*)
        val="${line#*:}"
        val="${val#"${val%%[![:space:]]*}"}"
        val="${val%%#*}"
        val="${val%"${val##*[![:space:]]}"}"
        val="${val#\"}"; val="${val%\"}"
        val="${val#\'}"; val="${val%\'}"
        [ -n "$val" ] || return 1
        printf '%s' "$val"
        return 0
        ;;
    esac
  done < "$file"
  return 1
}

_resolve_cutoff() {
  if [ -n "${CYBEROS_TRACE_CUTOFF:-}" ]; then
    printf '%s' "$CYBEROS_TRACE_CUTOFF"
    return
  fi
  local from_cfg
  if from_cfg="$(_read_yaml_trace_key cutoff "$root/.cyberos/config.yaml" 2>/dev/null)"; then
    printf '%s' "$from_cfg"
    return
  fi
  printf '%s' "$_DEFAULT_CUTOFF"
}

_strict_enabled() {
  [ "${CYBEROS_STRICT_COMMITS:-0}" = "1" ] && return 0
  [ "${CYBEROS_REQUIRE_TASK_LINK:-0}" = "1" ] && return 0
  local s
  s="$(_read_yaml_trace_key strict "$root/.cyberos/config.yaml" 2>/dev/null || true)"
  case "$s" in true|TRUE|yes|YES|1) return 0 ;; esac
  return 1
}

CUTOFF="$(_resolve_cutoff)"

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
    if _strict_enabled; then
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
