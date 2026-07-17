#!/usr/bin/env bash
# version-compare.sh — the ONE semver comparison for CyberOS. Source it; do not re-implement it.
#
# TASK-IMP-104 found this logic living in two places (version.sh and lib/update-check.sh),
# byte-identical and agreeing only because nobody had edited one of them yet. Two functions that
# can answer the same question is one question with two answers waiting to happen (§8.1a).
# install.sh's downgrade guard would have been the third copy.
#
#   is_ver <s>        -> 0 when <s> is a strict X.Y.Z of integers
#   ver_lt <a> <b>    -> 0 when a < b (numeric per component; equal is NOT less-than)
#
# Non-semver input (pre-release suffixes, "unknown", empty) is NOT comparable: is_ver rejects it
# and the CALLER decides what that means. This file does not invent an ordering it cannot defend.
is_ver() { printf '%s' "$1" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$'; }
ver_lt() {
  [ "$1" = "$2" ] && return 1
  [ "$(printf '%s\n%s\n' "$1" "$2" | sort -t. -k1,1n -k2,2n -k3,3n | head -1)" = "$1" ]
}
