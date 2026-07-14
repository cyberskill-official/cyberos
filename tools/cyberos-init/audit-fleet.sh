#!/usr/bin/env bash
# audit-fleet.sh — deep audit: every .cyberos module/channel after init must be present and usable.
# Usage: bash tools/cyberos-init/audit-fleet.sh <expected-version> <root-dir> [...]
set -uo pipefail
WANT="${1:?usage: audit-fleet.sh <expected-version> <root> [...]}"; shift
FAILED=0

for base in "$@"; do
  for r in "$base"/*; do
    [ -d "$r" ] || continue
    name="$(basename "$base")/$(basename "$r")"
    bad=""
    cy="$r/.cyberos"

    # --- version ---
    inst="none"; [ -f "$cy/VERSION" ] && inst="$(tr -d ' \n\r' < "$cy/VERSION")"
    [ "$inst" = "$WANT" ] || bad="$bad version($inst)"

    # --- core modules (must exist after init) ---
    for p in \
      install.sh uninstall.sh update.sh status.sh help.sh VERSION manifest.yaml \
      lib/fr-migrate.sh lib/update-check.sh lib/status-page.sh \
      cuo/gates/run-gates.sh \
      cuo/ship-feature-requests.md \
      cuo/EXECUTION-DISCIPLINE.md \
      cuo/STATUS-REFERENCE.md \
      plugin/.claude-plugin/plugin.json \
      docs-tools/render-status-hub.mjs \
      docs-tools/md.mjs \
      docs-tools/templates/status-hub.html \
      docs-tools/templates/status-app.js \
      docs-tools/templates/status.css \
      docs-tools/templates/tokens.css \
      memory/AGENTS.md \
      memory/memory.schema.json \
      memory/memory.invariants.yaml \
      mcp/cyberos-mcp.mjs \
      check-latest.sh \
      gates.env
    do
      [ -f "$cy/$p" ] || bad="$bad missing:.cyberos/$p"
    done

    # --- directories that must exist ---
    for d in cuo plugin mcp memory lib docs-tools cuo/gates cuo/skills plugin/skills plugin/commands; do
      [ -d "$cy/$d" ] || bad="$bad missing-dir:.cyberos/$d"
    done

    # orphan / legacy must NOT remain
    [ ! -f "$cy/migrate-frs.sh" ] || bad="$bad orphan:migrate-frs.sh"
    [ ! -f "$cy/init.sh" ] || bad="$bad orphan:init.sh"
    [ ! -f "$cy/changelog.sh" ] || bad="$bad orphan:changelog.sh"
    [ ! -f "$cy/status.html" ] || bad="$bad orphan:status.html"
    [ ! -d "$cy/status-site" ] || bad="$bad orphan:status-site"

    # --- workflows / skills non-empty ---
    skill_n=0
    if [ -d "$cy/cuo/skills" ]; then
      skill_n="$(find "$cy/cuo/skills" -mindepth 1 -maxdepth 1 -type d 2>/dev/null | wc -l | tr -d ' ')"
    fi
    [ "${skill_n:-0}" -ge 5 ] || bad="$bad cuo-skills-thin($skill_n)"
    pskill_n=0
    if [ -d "$cy/plugin/skills" ]; then
      pskill_n="$(find "$cy/plugin/skills" -mindepth 1 -maxdepth 1 -type d 2>/dev/null | wc -l | tr -d ' ')"
    fi
    [ "${pskill_n:-0}" -ge 5 ] || bad="$bad plugin-skills-thin($pskill_n)"
    # ship-feature-requests skill must exist in plugin
    [ -f "$cy/plugin/skills/ship-feature-requests/SKILL.md" ] \
      || [ -f "$cy/cuo/skills/ship-feature-requests/SKILL.md" ] \
      || bad="$bad missing:ship-feature-requests-skill"
    # plugin commands
    for cmd in install update help status create-feature-requests; do
      [ -f "$cy/plugin/commands/${cmd}.md" ] || bad="$bad missing:plugin/commands/${cmd}.md"
    done

    # --- memory protocol substance ---
    if [ -f "$cy/memory/AGENTS.md" ]; then
      grep -qE 'BRAIN|memory-root|Layer-1|§1' "$cy/memory/AGENTS.md" || bad="$bad memory-agents-thin"
    fi
    if [ -f "$cy/memory/memory.schema.json" ]; then
      grep -q 'MemoryPath\|Frontmatter\|AuditRecord' "$cy/memory/memory.schema.json" \
        || bad="$bad memory-schema-thin"
    fi

    # --- gates.env has expected keys ---
    if [ -f "$cy/gates.env" ]; then
      grep -qE '^(BUILD_CMD|LINT_CMD|TEST_CMD|COVERAGE_CMD)=' "$cy/gates.env" \
        || bad="$bad gates.env-keys"
    fi

    # --- repo entry points ---
    [ -f "$r/AGENTS.md" ] || bad="$bad missing:AGENTS.md"
    # --- AGENTS.md must be thin pointer (not Layer-1 protocol dump) ---
    if [ -f "$r/AGENTS.md" ]; then
      if grep -q 'Layer-1 Memory Protocol' "$r/AGENTS.md" 2>/dev/null; then
        if [ ! -f "$r/modules/memory/memory.schema.json" ]; then
          bad="$bad agents-protocol-dump"
        fi
      else
        grep -q '\.cyberos/AGENT-ENTRY' "$r/AGENTS.md" 2>/dev/null \
          || bad="$bad agents-no-entry-pointer"
      fi
    fi

    [ -d "$r/docs/feature-requests" ] || bad="$bad missing:docs/feature-requests"
    if [ -d "$r/.git" ]; then
      grep -q '.cyberos/' "$r/.gitignore" 2>/dev/null || bad="$bad gitignore-block"
      hk="$r/.git/hooks/pre-commit"
      if [ "$(git -C "$r" config core.hooksPath 2>/dev/null)" = "" ]; then
        { [ -f "$hk" ] && grep -qE 'cyberos-status-hook|init\.sh --page|check-status-sync|docs/status' "$hk"; } \
          || bad="$bad status-hook"
        # hook must not reference retired migrate-frs
        if [ -f "$hk" ] && grep -qE 'migrate-frs|init\.sh --page' "$hk" 2>/dev/null; then
          bad="$bad hook-legacy-status-path"
        fi
      fi
    fi

    # --- FR layout ---
    specs=0; flat=0; nospec=0; chunks=0
    if [ -d "$r/docs/feature-requests" ]; then
      specs="$(find "$r/docs/feature-requests" -mindepth 3 -maxdepth 3 -name spec.md -not -path '*/_*' -not -path '*/.*' 2>/dev/null | wc -l | tr -d ' ')"
      flat="$(find "$r/docs/feature-requests" -mindepth 1 -maxdepth 2 -type f -name 'FR-*.md' -not -name '*.audit.md' -not -path '*/_*' -not -path '*/.*' 2>/dev/null | wc -l | tr -d ' ')"
      nospec="$(find "$r/docs/feature-requests" -mindepth 2 -maxdepth 2 -type d -name 'FR-*' -not -path '*/_*' -not -path '*/.*' 2>/dev/null \
                 | while IFS= read -r d; do [ -f "$d/spec.md" ] || echo x; done | wc -l | tr -d ' ')"
    fi
    [ "$flat" -eq 0 ]   || bad="$bad flat-frs($flat)"
    [ "$nospec" -eq 0 ] || bad="$bad fr-folders-without-spec($nospec)"

    # --- status page ---
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
        inpage="$(grep -o '"i":"' "$page" 2>/dev/null | wc -l | tr -d ' ')"
        [ "$inpage" -eq "$specs" ] || bad="$bad corpus-drift(page=$inpage disk=$specs)"
        chunks="$(find "$r/docs/status/data/fr" -name '*.js' 2>/dev/null | wc -l | tr -d ' ')"
        [ "$specs" -eq 0 ] || [ "$chunks" -gt 0 ] || bad="$bad no-spec-chunks"

        # freshness re-render
        if command -v node >/dev/null 2>&1 && [ -f "$cy/docs-tools/render-status-hub.mjs" ]; then
          t="$(mktemp -d)"
          pinned="$(grep -o '"commit":"[^"]*"' "$page" | head -1 | cut -d'"' -f4)"
          if CYBEROS_HUB_LENIENT=1 CYBEROS_PAGE_ASSETS=1 \
             CYBEROS_PROJECT="$(basename "$r")" CYBEROS_FR_BASE="../feature-requests/" \
             CYBEROS_COMMIT="${pinned:-unknown}" \
             CYBEROS_TEMPLATES="$cy/docs-tools/templates" \
             node "$cy/docs-tools/render-status-hub.mjs" "$r" "$t" >/dev/null 2>"$t/err"; then
            cmp -s "$t/reference/status.html" "$page" || bad="$bad page-stale"
          else
            bad="$bad renderer-crashes"
          fi
          rm -rf "$t"
        fi
      fi
    fi

    # --- functional smokes (must work) ---
    if [ -f "$cy/init.sh" ]; then
      if ! bash "$cy/update.sh" "$r" >/dev/null 2>&1; then
        bad="$bad update-check-fails"
      fi
      if [ -f "$cy/lib/status-page.sh" ] && command -v node >/dev/null 2>&1; then
        if ! bash "$cy/lib/status-page.sh" "$r" >/dev/null 2>&1; then
          bad="$bad status-page-fails"
        fi
      fi
    fi
    # update-check must be sourceable
    if [ -f "$cy/lib/update-check.sh" ]; then
      if ! bash -c "source '$cy/lib/update-check.sh'; CYBEROS_UPDATE_CHECK=0 _cyberos_update_check" 2>/dev/null; then
        bad="$bad update-check-broken"
      fi
    fi
    # run-gates / update / help must parse
    for scr in "$cy/cuo/gates/run-gates.sh" "$cy/update.sh" "$cy/help.sh" "$cy/status.sh" "$cy/install.sh" "$cy/uninstall.sh"; do
      [ -f "$scr" ] || continue
      if ! bash -n "$scr" 2>/dev/null; then
        bad="$bad syntax:$(basename "$scr")"
      fi
    done
    # mcp entry non-empty
    if [ -f "$cy/mcp/cyberos-mcp.mjs" ]; then
      head -1 "$cy/mcp/cyberos-mcp.mjs" | grep -q . || bad="$bad mcp-empty"
      grep -q 'fr_init\|fr_gates\|ship_fr' "$cy/mcp/cyberos-mcp.mjs" || bad="$bad mcp-tools-missing"
    fi
    # workflow doc mentions HITL
    if [ -f "$cy/cuo/ship-feature-requests.md" ]; then
      grep -qi 'HITL\|human-acceptance\|human acceptance' "$cy/cuo/ship-feature-requests.md" \
        || bad="$bad workflow-no-hitl"
      # no retired migrate-frs in workflow
      grep -q 'migrate-frs' "$cy/cuo/ship-feature-requests.md" 2>/dev/null \
        && bad="$bad workflow-still-migrate-frs"
    fi

    if [ -z "$bad" ]; then
      printf 'PASS  %-46s v%s  specs=%-4s chunks=%s skills=%s\n' \
        "$name" "$inst" "$specs" "${chunks:-0}" "${skill_n:-0}"
    else
      printf 'FAIL  %-46s v%s %s\n' "$name" "$inst" "$bad"
      FAILED=$((FAILED + 1))
    fi
  done
done

echo
if [ "$FAILED" -eq 0 ]; then echo "audit-fleet: all green"; exit 0; fi
echo "audit-fleet: $FAILED repo(s) failed deep audit"; exit 1
