#!/usr/bin/env bash
# audit-fleet.sh — deep audit: every .cyberos module/channel after init must be present and usable.
# Usage: bash tools/cyberos-init/audit-fleet.sh <expected-version> <root-dir> [...]
#
# env: CYBEROS_EXPECT_RULES_SHA  rules_sha every install must match. Defaults to the repo's own
#                                dist/ payload manifest. VERSION alone cannot see rule content —
#                                a rules-only change keeps the version and every filename identical,
#                                so without this the audit reports drifted repos green.
set -uo pipefail
WANT="${1:?usage: audit-fleet.sh <expected-version> <root> [...]}"; shift
FAILED=0

_rs() { [ -f "${1:-}" ] || return 1; grep -E '^rules_sha:' "$1" 2>/dev/null | head -1 | awk '{print $2}' | tr -d ' \n\r'; }
WANT_SHA="${CYBEROS_EXPECT_RULES_SHA:-}"
if [ -z "$WANT_SHA" ]; then
  _self="$(cd "$(dirname "$0")/../.." 2>/dev/null && pwd || true)"
  [ -n "$_self" ] && WANT_SHA="$(_rs "$_self/dist/cyberos/manifest.yaml" || true)"
fi
[ -n "$WANT_SHA" ] || echo "audit-fleet: WARNING — no expected rules_sha resolved; rule-drift check DISABLED" >&2

for base in "$@"; do
  for r in "$base"/*; do
    [ -d "$r" ] || continue
    name="$(basename "$base")/$(basename "$r")"
    bad=""
    cy="$r/.cyberos"

    # --- version ---
    inst="none"; [ -f "$cy/VERSION" ] && inst="$(tr -d ' \n\r' < "$cy/VERSION")"
    [ "$inst" = "$WANT" ] || bad="$bad version($inst)"

    # --- rule content (TASK-IMP-074 §10) ---
    # The version above is a promise; this is the evidence. Both were 1.0.0 across the fleet
    # while 23/24 repos ran the pre-rename ruleset.
    if [ -n "$WANT_SHA" ]; then
      inst_sha="$(_rs "$cy/manifest.yaml" || true)"
      [ "$inst_sha" = "$WANT_SHA" ] || bad="$bad rules_sha(${inst_sha:-none})"
    fi

    # --- core modules (must exist after init) ---
    for p in \
      install.sh uninstall.sh version.sh status.sh help.sh VERSION manifest.yaml \
      lib/task-migrate.sh lib/update-check.sh lib/status-page.sh \
      cuo/gates/run-gates.sh \
      cuo/ship-tasks.md \
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
    [ ! -f "$cy/migrate-tasks.sh" ] || bad="$bad orphan:migrate-tasks.sh"
    [ ! -f "$cy/init.sh" ] || bad="$bad orphan:init.sh"
    [ ! -f "$cy/changelog.sh" ] || bad="$bad orphan:changelog.sh"
    [ ! -f "$cy/update.sh" ] || bad="$bad orphan:update.sh"
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
    # ship-tasks skill must exist in plugin
    [ -f "$cy/plugin/skills/ship-tasks/SKILL.md" ] \
      || [ -f "$cy/cuo/skills/ship-tasks/SKILL.md" ] \
      || bad="$bad missing:ship-tasks-skill"
    # plugin commands
    for cmd in install uninstall version status help create-tasks; do
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

    [ -d "$r/docs/tasks" ] || bad="$bad missing:docs/tasks"
    if [ -d "$r/.git" ]; then
      grep -q '.cyberos/' "$r/.gitignore" 2>/dev/null || bad="$bad gitignore-block"
      hk="$r/.git/hooks/pre-commit"
      if [ "$(git -C "$r" config core.hooksPath 2>/dev/null)" = "" ]; then
        { [ -f "$hk" ] && grep -qE 'cyberos-status-hook|init\.sh --page|check-status-sync|docs/status' "$hk"; } \
          || bad="$bad status-hook"
        # hook must not reference retired migrate-tasks
        if [ -f "$hk" ] && grep -qE 'migrate-tasks|init\.sh --page' "$hk" 2>/dev/null; then
          bad="$bad hook-legacy-status-path"
        fi
      fi
    fi

    # --- task layout ---
    specs=0; flat=0; nospec=0; chunks=0
    if [ -d "$r/docs/tasks" ]; then
      specs="$(find "$r/docs/tasks" -mindepth 3 -maxdepth 3 -name spec.md -not -path '*/_*' -not -path '*/.*' 2>/dev/null | wc -l | tr -d ' ')"
      flat="$(find "$r/docs/tasks" -mindepth 1 -maxdepth 2 -type f -name 'TASK-*.md' -not -name '*.audit.md' -not -path '*/_*' -not -path '*/.*' 2>/dev/null | wc -l | tr -d ' ')"
      nospec="$(find "$r/docs/tasks" -mindepth 2 -maxdepth 2 -type d -name 'TASK-*' -not -path '*/_*' -not -path '*/.*' 2>/dev/null \
                 | while IFS= read -r d; do [ -f "$d/spec.md" ] || echo x; done | wc -l | tr -d ' ')"
    fi
    [ "$flat" -eq 0 ]   || bad="$bad flat-tasks($flat)"
    [ "$nospec" -eq 0 ] || bad="$bad task-folders-without-spec($nospec)"

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
        # data/fr -> data/task in the 2026-07-15 rename. `find` on a missing dir with
        # stderr dropped yields 0, so this silently counted 0 chunks against 509 specs
        # and flagged every healthy repo `no-spec-chunks`.
        chunks="$(find "$r/docs/status/data/task" -name '*.js' 2>/dev/null | wc -l | tr -d ' ')"
        [ "$specs" -eq 0 ] || [ "$chunks" -gt 0 ] || bad="$bad no-spec-chunks"

        # freshness re-render
        if command -v node >/dev/null 2>&1 && [ -f "$cy/docs-tools/render-status-hub.mjs" ]; then
          t="$(mktemp -d)"
          pinned="$(grep -o '"commit":"[^"]*"' "$page" | head -1 | cut -d'"' -f4)"
          if CYBEROS_HUB_LENIENT=1 CYBEROS_PAGE_ASSETS=1 \
             CYBEROS_PROJECT="$(basename "$r")" CYBEROS_TASK_BASE="../tasks/" \
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
    # These were gated on `[ -f "$cy/init.sh" ]` and so had never run once: the payload ships no
    # init.sh, and the orphan check above asserts .cyberos/init.sh must NOT exist — so the gate
    # was false on every correct install and true only on a broken one. Gate each smoke on the
    # script it actually invokes.
    if [ -f "$cy/version.sh" ]; then
      if ! CYBEROS_NONINTERACTIVE=1 CYBEROS_OFFLINE=1 bash "$cy/version.sh" "$r" >/dev/null 2>&1; then
        bad="$bad version-check-fails"
      fi
    fi
    if [ -f "$cy/lib/status-page.sh" ] && command -v node >/dev/null 2>&1; then
      if ! bash "$cy/lib/status-page.sh" "$r" >/dev/null 2>&1; then
        bad="$bad status-page-fails"
      fi
    fi
    # update-check must be sourceable
    if [ -f "$cy/lib/update-check.sh" ]; then
      if ! bash -c "source '$cy/lib/update-check.sh'; CYBEROS_UPDATE_CHECK=0 _cyberos_update_check" 2>/dev/null; then
        bad="$bad update-check-broken"
      fi
    fi
    # run-gates / update / help must parse
    for scr in "$cy/cuo/gates/run-gates.sh" "$cy/version.sh" "$cy/help.sh" "$cy/status.sh" "$cy/install.sh" "$cy/uninstall.sh"; do
      [ -f "$scr" ] || continue
      if ! bash -n "$scr" 2>/dev/null; then
        bad="$bad syntax:$(basename "$scr")"
      fi
    done
    # mcp entry non-empty
    if [ -f "$cy/mcp/cyberos-mcp.mjs" ]; then
      head -1 "$cy/mcp/cyberos-mcp.mjs" | grep -q . || bad="$bad mcp-empty"
      grep -q 'task_init\|task_gates\|ship_task' "$cy/mcp/cyberos-mcp.mjs" || bad="$bad mcp-tools-missing"
    fi
    # workflow doc mentions HITL
    if [ -f "$cy/cuo/ship-tasks.md" ]; then
      grep -qi 'HITL\|human-acceptance\|human acceptance' "$cy/cuo/ship-tasks.md" \
        || bad="$bad workflow-no-hitl"
      # no retired migrate-tasks in workflow
      grep -q 'migrate-tasks' "$cy/cuo/ship-tasks.md" 2>/dev/null \
        && bad="$bad workflow-still-migrate-tasks"
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
