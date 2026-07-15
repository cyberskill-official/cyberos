#!/usr/bin/env bash
# update-check.sh — sourced or exec'd whenever anything under .cyberos runs.
# Soft by default (warn); CYBEROS_UPDATE_CHECK=strict|always|0
# shellcheck shell=bash

# Read rules_sha out of a payload/installed manifest. Echoes the 64-hex value, or nothing.
# TASK-IMP-074 §10: VERSION cannot see rule content — this fingerprint can.
_cyberos_rules_sha() {
  [ -f "${1:-}" ] || return 1
  local v
  v="$(grep -E '^rules_sha:' "$1" 2>/dev/null | head -1 | awk '{print $2}' | tr -d ' \n\r')"
  printf '%s' "$v"
  [ -n "$v" ]
}

_cyberos_update_check() {
  # 0 = skip, strict = exit 1 if stale, always = ignore throttle, soft = default warn
  local mode="${CYBEROS_UPDATE_CHECK:-soft}"
  [ "$mode" = "0" ] || [ "$mode" = "off" ] || [ "$mode" = "false" ] && return 0

  local root=""
  if git rev-parse --show-toplevel >/dev/null 2>&1; then
    root="$(git rev-parse --show-toplevel)"
  else
    root="$(pwd)"
  fi
  local cy="${root}/.cyberos"
  [ -f "$cy/VERSION" ] || return 0

  local cache="$cy/.update-check-cache"
  local now
  now="$(date +%s 2>/dev/null || echo 0)"
  if [ "$mode" != "always" ] && [ "$mode" != "strict" ] && [ -f "$cache" ]; then
    local last
    last="$(tr -d ' \n\r' < "$cache" 2>/dev/null || echo 0)"
    # throttle: once per 12h unless always/strict
    if [ "$now" -gt 0 ] && [ "$last" -gt 0 ] 2>/dev/null; then
      if [ $((now - last)) -lt 43200 ]; then
        return 0
      fi
    fi
  fi

  local inst payload_ver latest_line latest verdict
  inst="$(tr -d ' \n\r' < "$cy/VERSION")"
  payload_ver="$inst"
  # Prefer check beside payload install (vendored) or sibling check-latest
  if [ -f "$cy/check-latest.sh" ] && [ "${CYBEROS_OFFLINE:-0}" != "1" ]; then
    latest_line="$(bash "$cy/check-latest.sh" 2>/dev/null || echo "latest=unknown source=offline")"
  elif [ -f "$cy/../check-latest.sh" ] && [ "${CYBEROS_OFFLINE:-0}" != "1" ]; then
    latest_line="$(bash "$cy/../check-latest.sh" 2>/dev/null || echo "latest=unknown source=offline")"
  else
    # Try networkless compare only if we have a payload path in env
    if [ -n "${CYBEROS_PAYLOAD:-}" ] && [ -f "${CYBEROS_PAYLOAD}/VERSION" ]; then
      payload_ver="$(tr -d ' \n\r' < "${CYBEROS_PAYLOAD}/VERSION")"
    fi
    latest_line="latest=unknown source=offline"
  fi
  latest="${latest_line#latest=}"; latest="${latest%% *}"

  is_ver() { printf '%s' "$1" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$'; }
  ver_lt() {
    [ "$1" = "$2" ] && return 1
    [ "$(printf '%s\n%s\n' "$1" "$2" | sort -t. -k1,1n -k2,2n -k3,3n | head -1)" = "$1" ]
  }

  verdict="up_to_date"
  if is_ver "$latest" && is_ver "$inst" && ver_lt "$inst" "$latest"; then
    verdict="repo_stale"
  fi
  # payload newer than installed (local dist ahead of this repo)
  if [ -n "${CYBEROS_PAYLOAD:-}" ] && [ -f "${CYBEROS_PAYLOAD}/VERSION" ]; then
    payload_ver="$(tr -d ' \n\r' < "${CYBEROS_PAYLOAD}/VERSION")"
    if is_ver "$payload_ver" && is_ver "$inst" && ver_lt "$inst" "$payload_ver"; then
      verdict="repo_stale"
    fi
  fi

  # Rule-content drift (TASK-IMP-074 §10). VERSION is a promise; rules_sha is the evidence.
  # An unchanged VERSION shipping changed rules is the exact case the version compare above
  # cannot see, and is how 23/24 repos silently kept running the pre-rename ruleset.
  # Compare against a reachable payload: explicit CYBEROS_PAYLOAD first, else the tree this
  # script itself was sourced from (a payload copy; when it IS the install, it self-compares
  # equal and correctly stays quiet).
  local self_root="" pay_manifest="" inst_sha="" pay_sha=""
  self_root="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.." 2>/dev/null && pwd || true)"
  if [ -n "${CYBEROS_PAYLOAD:-}" ] && [ -f "${CYBEROS_PAYLOAD}/manifest.yaml" ]; then
    pay_manifest="${CYBEROS_PAYLOAD}/manifest.yaml"
  elif [ -n "$self_root" ] && [ -f "$self_root/manifest.yaml" ]; then
    pay_manifest="$self_root/manifest.yaml"
  fi
  if [ -n "$pay_manifest" ]; then
    pay_sha="$(_cyberos_rules_sha "$pay_manifest" || true)"
    inst_sha="$(_cyberos_rules_sha "$cy/manifest.yaml" || true)"
    # Mismatch OR an installed manifest with no fingerprint at all (payload predates the
    # field) both mean: the vendored rules are not the payload's rules.
    if [ -n "$pay_sha" ] && [ "$inst_sha" != "$pay_sha" ]; then
      verdict="rules_drift"
    fi
  fi

  printf '%s\n' "$now" > "$cache" 2>/dev/null || true

  if [ "$verdict" = "rules_drift" ]; then
    echo "cyberos: RULE DRIFT — installed=$inst payload=${payload_ver:-?} (same version, different rules)" >&2
    echo "cyberos:   installed rules_sha=${inst_sha:-<none>}" >&2
    echo "cyberos:   payload   rules_sha=${pay_sha:-<none>}" >&2
    echo "cyberos: next: bash ${CYBEROS_PAYLOAD:-$self_root}/install.sh $root   # re-vendor to match" >&2
    if [ "$mode" = "strict" ]; then
      return 1
    fi
    return 0
  fi

  if [ "$verdict" != "up_to_date" ]; then
    echo "cyberos: UPDATE AVAILABLE — installed=$inst latest=${latest:-?} payload=${payload_ver:-?} ($verdict)" >&2
    echo "cyberos: next: bash .cyberos/version.sh   # or: bash \${CYBEROS_PAYLOAD:-/path/to/dist/cyberos}/install.sh $root" >&2
    echo "cyberos:   or: curl -fsSL https://github.com/cyberskill-official/cyberos/releases/latest/download/cyberos-payload.tar.gz | tar -xz -C /tmp && bash /tmp/cyberos/install.sh $root" >&2
    if [ "$mode" = "strict" ]; then
      return 1
    fi
  fi
  return 0
}

# Allow exec as script
if [ "${BASH_SOURCE[0]:-}" = "$0" ]; then
  _cyberos_update_check "$@"
fi
