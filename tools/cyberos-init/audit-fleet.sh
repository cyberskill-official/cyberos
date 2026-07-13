#!/usr/bin/env bash
# audit-fleet.sh - prove that init + migration landed as designed in every repo under the given
# roots. Read-only. One line per repo plus a machine-readable verdict; exits non-zero if any
# repo fails, so it can be looped until green.
# Usage: bash tools/cyberos-init/audit-fleet.sh <expected-version> <root-dir> [<root-dir> ...]
set -uo pipefail
WANT="${1:?usage: audit-fleet.sh <expected-version> <root> [...]}"; shift
FAILED=0

first() { find "$1" -name "$2" ${3:+$3} -print -quit 2>/dev/null; }   # never `find | grep -q`: pipefail + SIGPIPE lies

for base in "$@"; do
  for r in "$base"/*; do
    [ -d "$r" ] || continue
    name="$(basename "$base")/$(basename "$r")"
    bad=""

    # --- 1. the machine is vendored, at the expected version -----------------
    inst="none"; [ -f "$r/.cyberos/VERSION" ] && inst="$(tr -d ' \n\r' < "$r/.cyberos/VERSION")"
    [ "$inst" = "$WANT" ] || bad="$bad version($inst)"
    for p in cuo/gates/run-gates.sh plugin/.claude-plugin/plugin.json \
             docs-tools/render-status-hub.mjs docs-tools/md.mjs \
             docs-tools/templates/status-hub.html docs-tools/templates/status-app.js \
             docs-tools/templates/status.css docs-tools/templates/tokens.css; do
      [ -f "$r/.cyberos/$p" ] || bad="$bad missing:.cyberos/$p"
    done
    # migrate kit: lib/fr-migrate.sh (combined) OR legacy migrate-frs.sh shim
    if [ ! -f "$r/.cyberos/lib/fr-migrate.sh" ] && [ ! -f "$r/.cyberos/migrate-frs.sh" ]; then
      bad="$bad missing:migrate-kit"
    fi
    # status hook v2 preferred
    if [ -f "$r/.git/hooks/pre-commit" ]; then
      grep -qE 'cyberos-status-hook v2|init.sh --page|migrate-frs' "$r/.git/hooks/pre-commit" \
        || bad="$bad status-hook-stale"
    fi

    # --- 2. the repo's own entry points -------------------------------------
    [ -f "$r/AGENTS.md" ] || bad="$bad missing:AGENTS.md"
    [ -d "$r/docs/feature-requests" ] || bad="$bad missing:docs/feature-requests"
    if [ -d "$r/.git" ]; then
      grep -q '.cyberos/' "$r/.gitignore" 2>/dev/null || bad="$bad gitignore-block"
      hk="$r/.git/hooks/pre-commit"
      if [ "$(git -C "$r" config core.hooksPath 2>/dev/null)" = "" ]; then
        # Accept official status-hook OR combined consumer hooks (init --page / migrate-frs / local-ci)
        { [ -f "$hk" ] && grep -qE 'cyberos-status-hook|init\.sh --page|migrate-frs|check-status-sync|docs/status' "$hk"; } \
          || bad="$bad status-hook"
      fi
    fi

    # --- 3. FR layout: nothing flat, no folder without a spec ----------------
    # _* and .* trees are out of protocol scope (archives, audits, ship-manifest run state) -
    # exactly the exclusion migrate-frs.sh uses. Counting them was an audit bug, not a repo bug.
    specs=0; flat=0; nospec=0
    if [ -d "$r/docs/feature-requests" ]; then
      specs="$(find "$r/docs/feature-requests" -mindepth 3 -maxdepth 3 -name spec.md -not -path '*/_*' -not -path '*/.*' 2>/dev/null | wc -l | tr -d ' ')"
      flat="$(find "$r/docs/feature-requests" -mindepth 1 -maxdepth 2 -type f -name 'FR-*.md' -not -name '*.audit.md' -not -path '*/_*' -not -path '*/.*' 2>/dev/null | wc -l | tr -d ' ')"
      nospec="$(find "$r/docs/feature-requests" -mindepth 2 -maxdepth 2 -type d -name 'FR-*' -not -path '*/_*' -not -path '*/.*' 2>/dev/null \
                 | while IFS= read -r d; do [ -f "$d/spec.md" ] || echo x; done | wc -l | tr -d ' ')"
    fi
    [ "$flat" -eq 0 ]   || bad="$bad flat-frs($flat)"
    [ "$nospec" -eq 0 ] || bad="$bad fr-folders-without-spec($nospec)"

    # --- 4. the status page: present iff there is something to show ----------
    page="$r/docs/status/index.html"
    if [ "$specs" -gt 0 ] || [ -d "$r/docs/status" ]; then
      if [ ! -f "$page" ]; then
        bad="$bad missing:docs/status/index.html"
      else
        grep -q 'data-template-id="status-hub@2"' "$page" || bad="$bad page-not-v2"
        for lens in board table timeline; do
          grep -q "data-lens=\"$lens\"" "$page" || bad="$bad lens:$lens"
        done
        grep -q 'role="tabpanel"' "$page" && bad="$bad old-tabs-present"
        [ -f "$r/docs/status/assets/status.css" ] || bad="$bad missing:status.css"
        [ -f "$r/docs/status/assets/status.js" ]  || bad="$bad missing:status.js"
        # the corpus in the page must equal the corpus on disk. Probe on "i":" - NOT on "i":"FR-:
        # a repo's ids are not always FR-shaped (strategem carries COV-001, API-READY) and the
        # narrower probe under-counted them, reporting drift that did not exist.
        inpage="$(grep -o '"i":"' "$page" 2>/dev/null | wc -l | tr -d ' ')"
        [ "$inpage" -eq "$specs" ] || bad="$bad corpus-drift(page=$inpage disk=$specs)"
        chunks="$(find "$r/docs/status/data/fr" -name '*.js' 2>/dev/null | wc -l | tr -d ' ')"
        [ "$specs" -eq 0 ] || [ "$chunks" -gt 0 ] || bad="$bad no-spec-chunks"

        # FRESHNESS. Shape checks alone let a page from an EARLIER run pass while the renderer
        # is crashing on every run (exactly what happened once - the audit reported GREEN on
        # stale output). Re-render from this repo's own vendored kit and byte-compare: the only
        # honest proof that the committed page IS what today's CyberOS produces.
        if command -v node >/dev/null 2>&1 && [ -f "$r/.cyberos/docs-tools/render-status-hub.mjs" ]; then
          t="$(mktemp -d)"
          # Pin the provenance stamp to the one the page already carries: a page staged by the
          # pre-commit hook holds the PARENT commit's sha (the commit being made does not exist
          # yet), so comparing stamps would flag every freshly-committed page as stale. We are
          # proving the CONTENT is current, not re-deriving the sha.
          pinned="$(grep -o '"commit":"[^"]*"' "$page" | head -1 | cut -d'"' -f4)"
          if CYBEROS_HUB_LENIENT=1 CYBEROS_PAGE_ASSETS=1 \
             CYBEROS_PROJECT="$(basename "$r")" CYBEROS_FR_BASE="../feature-requests/" \
             CYBEROS_COMMIT="${pinned:-unknown}" \
             CYBEROS_TEMPLATES="$r/.cyberos/docs-tools/templates" \
             node "$r/.cyberos/docs-tools/render-status-hub.mjs" "$r" "$t" >/dev/null 2>"$t/err"; then
            cmp -s "$t/reference/status.html" "$page" || bad="$bad page-stale(differs from a fresh render)"
          else
            bad="$bad renderer-crashes($(head -c 60 "$t/err" | tr '\n' ' '))"
          fi
          rm -rf "$t"
        fi
      fi
    fi

    if [ -z "$bad" ]; then
      printf 'PASS  %-46s v%s  specs=%-4s chunks=%s\n' "$name" "$inst" "$specs" "${chunks:-0}"
    else
      FAILED=$((FAILED + 1))
      printf 'FAIL  %-46s v%s  specs=%-4s ->%s\n' "$name" "$inst" "$specs" "$bad"
    fi
    unset chunks
  done
done

echo "----"
if [ "$FAILED" -eq 0 ]; then echo "audit-fleet: GREEN (every repo matches CyberOS $WANT)"; exit 0; fi
echo "audit-fleet: RED ($FAILED repo(s) off-spec)"; exit 1
