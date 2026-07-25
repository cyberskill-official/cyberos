#!/usr/bin/env bash
# rules-cone.sh — the ONE rules_sha cone + digest for CyberOS. Source it; do not re-implement it.
#
# TASK-IMP-122: rules_sha must be recomputed, not recalled. build.sh used to define `_rsha()`
# inline and hash `find cuo plugin mcp cli memory` — a cone that both missed vendored trees
# (lib, docs-tools, root scripts, ci) and included `cli/` which is never installed. That
# definition lived only in build.sh, which is never vendored, so installed comparators could
# not recompute at all and grepped the stored manifest token instead.
#
# This file is the shared declaration (§1.2): kinded cone + exclusions + `_rsha()`. install.sh
# vendors all of lib/, so version.sh / update-check.sh / audit-fleet.sh can source it from
# `.cyberos/lib/rules-cone.sh` after install. Precedence: lib/version-compare.sh (TASK-IMP-104).
#
# Kinds (§1.3):
#   dir:<path>     — every file beneath <path> (recursive)
#   file:<path>    — exactly that path
#   prune:<path>   — remove every file at-or-beneath <path> from the resolved set
#   exempt:<glob>  — classify a $CY path that no dir:/file: reaches (Direction-1 standing)
#
# Hash filter (§1.5): skip __pycache__/ and *.pyc (stale bytecode noise).
#
# shellcheck shell=bash

# Kinded entries: cone ∪ exclusions. One list; §1.6 classifies against it.
# Stored as a variable (not a heredoc) so callers under /tmp pressure do not exhaust mktemp.
_RULES_CONE_ENTRIES='dir:cuo
dir:plugin
dir:mcp
dir:lib
dir:docs-tools
dir:ci
dir:memory
file:install.sh
file:uninstall.sh
file:version.sh
file:status.sh
file:help.sh
file:check-latest.sh
prune:memory/store/
exempt:gates.env
exempt:config.yaml
exempt:.update-check-cache
exempt:AGENT-ENTRY.md
exempt:gates.env.bak.*
exempt:.install.lock
exempt:manifest.yaml
exempt:VERSION'

_rules_cone_entries() {
  printf '%s\n' "$_RULES_CONE_ENTRIES"
}

# sha256sum on Linux; shasum -a 256 on macOS. Same two-space text-mode output.
_rsha() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$@"
  else
    shasum -a 256 "$@"
  fi
}

# True when relative path should be skipped by the hash filter (§1.5).
_rules_cone_skip_noise() {
  local rel="$1" base="${1##*/}"
  case "$base" in
    *.pyc) return 0 ;;
  esac
  case "/$rel/" in
    */__pycache__/*) return 0 ;;
  esac
  return 1
}

# True when relative path matches an exempt: glob (basename or full relative).
_rules_cone_is_exempt() {
  local rel="$1" entry kind pat
  while IFS= read -r entry; do
    [ -n "$entry" ] || continue
    kind="${entry%%:*}"
    pat="${entry#*:}"
    [ "$kind" = "exempt" ] || continue
    # shellcheck disable=SC2254
    case "$rel" in
      $pat) return 0 ;;
    esac
    # shellcheck disable=SC2254
    case "${rel##*/}" in
      $pat) return 0 ;;
    esac
  done < <(_rules_cone_entries)
  return 1
}

# Resolve kinded entries against <tree_root> → relative file paths (one per line).
# Applies dir/file/prune; skips noise; does NOT emit exempt paths (they are never hashed).
_rules_cone_list() {
  local root="${1:?}"
  [ -d "$root" ] || return 0
  local entry kind pat rel
  local tmp prunes
  tmp="$(mktemp)"
  prunes=""
  while IFS= read -r entry; do
    [ -n "$entry" ] || continue
    kind="${entry%%:*}"
    pat="${entry#*:}"
    pat="${pat%/}"
    case "$kind" in
      dir)
        if [ -d "$root/$pat" ]; then
          ( cd "$root" && find "$pat" -type f 2>/dev/null ) >> "$tmp" || true
        fi
        ;;
      file)
        if [ -f "$root/$pat" ]; then
          printf '%s\n' "$pat" >> "$tmp"
        fi
        ;;
      prune)
        prunes="${prunes}${prunes:+ }$pat"
        ;;
      exempt) ;;
      *) ;;
    esac
  done < <(_rules_cone_entries)

  while IFS= read -r rel; do
    [ -n "$rel" ] || continue
    _rules_cone_skip_noise "$rel" && continue
    for pat in $prunes; do
      case "$rel" in
        "$pat"|"$pat"/*) continue 2 ;;
      esac
    done
    printf '%s\n' "$rel"
  done < "$tmp"
  rm -f "$tmp"
}

# Digest of the cone over <tree_root>. Empty / nothing resolved → empty string.
_rules_sha_of() {
  local root="${1:?}"
  local list digest
  list="$(_rules_cone_list "$root" | LC_ALL=C sort -u)"
  [ -n "$list" ] || { printf ''; return 0; }
  digest="$(
    cd "$root" || exit 1
    printf '%s\n' "$list" | while IFS= read -r f; do
      [ -n "$f" ] || continue
      [ -f "$f" ] || continue
      _rsha "$f"
    done | _rsha | cut -d' ' -f1
  )"
  printf '%s' "$digest"
}

# List relative paths whose content differs between two trees. Optional helper.
_rules_sha_diff() {
  local a="${1:?}" b="${2:?}"
  local list
  list="$(
    { _rules_cone_list "$a"; _rules_cone_list "$b"; } | LC_ALL=C sort -u
  )"
  printf '%s\n' "$list" | while IFS= read -r f; do
    [ -n "$f" ] || continue
    if [ -f "$a/$f" ] && [ -f "$b/$f" ]; then
      if ! cmp -s "$a/$f" "$b/$f"; then
        printf '%s\n' "$f"
      fi
    elif [ -f "$a/$f" ] || [ -f "$b/$f" ]; then
      printf '%s\n' "$f"
    fi
  done
}

# Validate kind grammar + prune/exempt invariants against <tree_root>.
_rules_cone_validate_grammar() {
  local root="${1:?}"
  local entry kind pat removed would_remove rel parent_exists any_under parent
  local tmp

  while IFS= read -r entry; do
    [ -n "$entry" ] || continue
    case "$entry" in
      *:*) ;;
      *)
        echo "rules-cone: ERROR: entry has no kind prefix: '$entry'" >&2
        return 1
        ;;
    esac
    kind="${entry%%:*}"
    case "$kind" in
      dir|file|prune|exempt) ;;
      *)
        echo "rules-cone: ERROR: unrecognised kind '$kind' in '$entry'" >&2
        return 1
        ;;
    esac
  done < <(_rules_cone_entries)

  tmp="$(mktemp)"
  while IFS= read -r entry; do
    [ -n "$entry" ] || continue
    kind="${entry%%:*}"
    pat="${entry#*:}"
    pat="${pat%/}"
    case "$kind" in
      dir)
        if [ -d "$root/$pat" ]; then
          ( cd "$root" && find "$pat" -type f 2>/dev/null ) >> "$tmp" || true
        fi
        ;;
      file)
        [ -f "$root/$pat" ] && printf '%s\n' "$pat" >> "$tmp"
        ;;
    esac
  done < <(_rules_cone_entries)

  # prune invariant (NEW5-002): removing nothing fails UNLESS path absent from tree.
  while IFS= read -r entry; do
    [ -n "$entry" ] || continue
    kind="${entry%%:*}"
    [ "$kind" = "prune" ] || continue
    pat="${entry#*:}"
    pat="${pat%/}"
    removed=0
    while IFS= read -r rel; do
      [ -n "$rel" ] || continue
      case "$rel" in
        "$pat"|"$pat"/*) removed=1; break ;;
      esac
    done < "$tmp"
    if [ "$removed" -eq 0 ]; then
      if [ -e "$root/$pat" ] || [ -L "$root/$pat" ]; then
        # Exists but matched no files. Fail if misspelt under a parent that has files.
        parent_exists=0
        case "$pat" in
          */*)
            parent="${pat%/*}"
            if [ -d "$root/$parent" ]; then
              any_under=0
              while IFS= read -r rel; do
                case "$rel" in
                  "$parent"/*) any_under=1; break ;;
                esac
              done < "$tmp"
              [ "$any_under" -eq 1 ] && parent_exists=1
            fi
            ;;
          *)
            [ -d "$root" ] && parent_exists=1
            ;;
        esac
        if [ "$parent_exists" -eq 1 ]; then
          echo "rules-cone: ERROR: prune:$pat removes nothing under $root (misspelt or no longer beneath a dir:)" >&2
          rm -f "$tmp"
          return 1
        fi
      fi
      # path wholly absent → defensive prune OK
    fi
  done < <(_rules_cone_entries)

  # exempt invariant: must NOT overlap the resolved set.
  while IFS= read -r entry; do
    [ -n "$entry" ] || continue
    kind="${entry%%:*}"
    [ "$kind" = "exempt" ] || continue
    pat="${entry#*:}"
    would_remove=0
    while IFS= read -r rel; do
      [ -n "$rel" ] || continue
      _rules_cone_skip_noise "$rel" && continue
      # shellcheck disable=SC2254
      case "$rel" in
        $pat) would_remove=1; break ;;
      esac
      # shellcheck disable=SC2254
      case "${rel##*/}" in
        $pat) would_remove=1; break ;;
      esac
    done < "$tmp"
    if [ "$would_remove" -eq 1 ]; then
      echo "rules-cone: ERROR: exempt:$pat overlaps the cone (use prune: instead)" >&2
      rm -f "$tmp"
      return 1
    fi
  done < <(_rules_cone_entries)

  rm -f "$tmp"
  return 0
}

# Classify a relative path against cone∪exclusions. Echoes: cone|prune|exempt|unclassified
_rules_cone_classify() {
  local rel="$1"
  local entry kind pat
  while IFS= read -r entry; do
    [ -n "$entry" ] || continue
    kind="${entry%%:*}"
    pat="${entry#*:}"
    [ "$kind" = "prune" ] || continue
    pat="${pat%/}"
    case "$rel" in
      "$pat"|"$pat"/*) printf 'prune'; return 0 ;;
    esac
  done < <(_rules_cone_entries)
  if _rules_cone_is_exempt "$rel"; then
    printf 'exempt'
    return 0
  fi
  while IFS= read -r entry; do
    [ -n "$entry" ] || continue
    kind="${entry%%:*}"
    pat="${entry#*:}"
    case "$kind" in
      dir)
        pat="${pat%/}"
        case "$rel" in
          "$pat"|"$pat"/*)
            if _rules_cone_skip_noise "$rel"; then
              printf 'exempt'
              return 0
            fi
            printf 'cone'
            return 0
            ;;
        esac
        ;;
      file)
        [ "$rel" = "$pat" ] && { printf 'cone'; return 0; }
        ;;
    esac
  done < <(_rules_cone_entries)
  printf 'unclassified'
}

# True when <tree_root> looks like a payload (has cone dirs), not merely an install.
_rules_cone_is_payload_tree() {
  local root="${1:?}"
  [ -d "$root/cuo" ] && [ -d "$root/plugin" ] && [ -f "$root/install.sh" ]
}
