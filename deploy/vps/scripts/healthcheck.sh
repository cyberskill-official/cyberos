#!/bin/sh
set -eu

ROOT="${1:-http://127.0.0.1:8080}"
RETRIES="${CYBEROS_HEALTHCHECK_RETRIES:-30}"
SLEEP_SECONDS="${CYBEROS_HEALTHCHECK_SLEEP:-1}"
BODY_FILE="${TMPDIR:-/tmp}/cyberos-healthcheck-body.$$"
trap 'rm -f "$BODY_FILE"' EXIT

check() {
  name="$1"
  url="$2"
  echo "checking $name: $url"

  attempt=1
  while [ "$attempt" -le "$RETRIES" ]; do
    if curl -fsS "$url" >"$BODY_FILE"; then
      cat "$BODY_FILE"
      echo
      return 0
    fi

    if [ "$attempt" -lt "$RETRIES" ]; then
      echo "waiting for $name ($attempt/$RETRIES)" >&2
      sleep "$SLEEP_SECONDS"
    fi
    attempt=$((attempt + 1))
  done

  echo "$name did not become healthy after $RETRIES attempts" >&2
  return 1
}

check gateway "$ROOT/healthz"
check auth "$ROOT/auth/healthz"
check memory "$ROOT/memory/healthz"

echo "CyberOS HTTP smoke passed"
