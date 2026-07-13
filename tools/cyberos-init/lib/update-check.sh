#!/usr/bin/env bash
# update-check.sh — sourced or exec'd whenever anything under .cyberos runs.
# Soft by default (warn); CYBEROS_UPDATE_CHECK=strict|always|0
# shellcheck shell=bash

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
  # Prefer check beside payload init (vendored) or sibling check-latest
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

  printf '%s\n' "$now" > "$cache" 2>/dev/null || true

  if [ "$verdict" != "up_to_date" ]; then
    echo "cyberos: UPDATE AVAILABLE — installed=$inst latest=${latest:-?} payload=${payload_ver:-?} ($verdict)" >&2
    echo "cyberos: next: bash \${CYBEROS_PAYLOAD:-/path/to/dist/cyberos}/init.sh $root" >&2
    echo "cyberos:   or: curl -fsSL https://github.com/cyberskill-official/cyberos/releases/latest/download/cyberos-payload.tar.gz | tar -xz -C /tmp && bash /tmp/cyberos/init.sh $root" >&2
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
