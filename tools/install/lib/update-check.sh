#!/usr/bin/env bash
# update-check.sh — sourced or exec'd whenever anything under .cyberos runs.
# Soft by default (warn); CYBEROS_UPDATE_CHECK=strict|always|0
# shellcheck shell=bash

# TASK-IMP-122: installed side is ALWAYS recomputed via lib/rules-cone.sh.
# Reference is a build token or a reachable payload tree. Never claim tamper.

_cyberos_manifest_token() {
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
    if [ -n "${CYBEROS_PAYLOAD:-}" ] && [ -f "${CYBEROS_PAYLOAD}/VERSION" ]; then
      payload_ver="$(tr -d ' \n\r' < "${CYBEROS_PAYLOAD}/VERSION")"
    fi
    latest_line="latest=unknown source=offline"
  fi
  latest="${latest_line#latest=}"; latest="${latest%% *}"

  # ONE comparator, sourced (TASK-IMP-104).
  . "$(dirname "${BASH_SOURCE[0]}")/version-compare.sh"

  verdict="up_to_date"
  if is_ver "$latest" && is_ver "$inst" && ver_lt "$inst" "$latest"; then
    verdict="repo_stale"
  fi
  if [ -n "${CYBEROS_PAYLOAD:-}" ] && [ -f "${CYBEROS_PAYLOAD}/VERSION" ]; then
    payload_ver="$(tr -d ' \n\r' < "${CYBEROS_PAYLOAD}/VERSION")"
    if is_ver "$payload_ver" && is_ver "$inst" && ver_lt "$inst" "$payload_ver"; then
      verdict="repo_stale"
    fi
  fi

  # Rule-content drift (TASK-IMP-122). Recompute installed; reference = token or tree.
  local self_root="" inst_sha="" pay_sha="" can_name=0 pay_tree=""
  local _rc
  _rc="$(dirname "${BASH_SOURCE[0]:-$0}")/rules-cone.sh"
  if [ -f "$_rc" ]; then
    # shellcheck source=/dev/null
    . "$_rc"
    inst_sha="$(_rules_sha_of "$cy" || true)"
    self_root="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.." 2>/dev/null && pwd || true)"
    # CYBEROS_PAYLOAD takes PRECEDENCE over self_root (§1.9).
    if [ -n "${CYBEROS_PAYLOAD:-}" ] && [ -d "${CYBEROS_PAYLOAD}" ]; then
      if _rules_cone_is_payload_tree "${CYBEROS_PAYLOAD}" 2>/dev/null; then
        pay_sha="$(_rules_sha_of "${CYBEROS_PAYLOAD}" || true)"
        pay_tree="${CYBEROS_PAYLOAD}"
        can_name=1
      else
        pay_sha="$(_cyberos_manifest_token "${CYBEROS_PAYLOAD}/manifest.yaml" || true)"
      fi
    elif [ -n "$self_root" ]; then
      case "$self_root" in
        */.cyberos)
          # Sourced from an install — reference is the build token; cannot name.
          pay_sha="$(_cyberos_manifest_token "$self_root/manifest.yaml" || true)"
          can_name=0
          ;;
        *)
          if _rules_cone_is_payload_tree "$self_root" 2>/dev/null; then
            pay_sha="$(_rules_sha_of "$self_root" || true)"
            pay_tree="$self_root"
            can_name=1
          else
            pay_sha="$(_cyberos_manifest_token "$self_root/manifest.yaml" || true)"
          fi
          ;;
      esac
    fi

    if [ -z "$inst_sha" ] || [ -z "$pay_sha" ]; then
      verdict="unknown"
    elif [ "$inst_sha" != "$pay_sha" ] && [ "$verdict" = "up_to_date" ]; then
      verdict="rules_drift"
    elif [ "$inst_sha" != "$pay_sha" ] && [ "$verdict" != "repo_stale" ]; then
      verdict="rules_drift"
    fi
  else
    verdict="unknown"
  fi

  printf '%s\n' "$now" > "$cache" 2>/dev/null || true

  if [ "$verdict" = "unknown" ]; then
    echo "cyberos: RULES CHECK unknown — could not compute rules_sha (missing cone lib, tree, or reference)" >&2
    if [ "$mode" = "strict" ]; then
      return 1
    fi
    return 0
  fi

  if [ "$verdict" = "rules_drift" ]; then
    echo "cyberos: RULE DRIFT — installed=$inst payload=${payload_ver:-?} (same version, different rules)" >&2
    echo "cyberos:   installed rules_sha=${inst_sha:-<none>}" >&2
    echo "cyberos:   payload   rules_sha=${pay_sha:-<none>}" >&2
    if [ "$can_name" -eq 1 ] && [ -n "$pay_tree" ]; then
      echo "cyberos:   differing paths:" >&2
      _rules_sha_diff "$pay_tree" "$cy" | while IFS= read -r _p; do
        [ -n "$_p" ] && echo "cyberos:     $_p" >&2
      done
    else
      echo "cyberos:   (cannot name differing paths — no reachable payload tree)" >&2
    fi
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
