#!/usr/bin/env bash
# build.sh - assemble the vendorable CyberOS payload into dist/cyberos/.
# install.sh lays this out under a target repo's gitignored .cyberos/, by module:
#   .cyberos/cuo/     - the task workflow engine (ship-tasks + doctrine + status
#                       contract + author/audit skills + gates + templates)
#   .cyberos/memory/  - the memory module (Layer-1 protocol + schema + invariants)
#   .cyberos/plugin/  - the Claude/Cowork plugin
# Best-effort: the three normative docs always vendor (doc-driven mode). Author/audit skill
# bodies and caf tooling vendor only if present (full mode); otherwise the reduced-profile
# floor applies (the target repo's own build/lint/test + code review + the two human gates).
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"                 # tools/install
repo="$(cd "$here/../.." && pwd)"                      # cyberos repo root
out="${1:-$repo/dist/cyberos}"

# The single platform VERSION - validated UP FRONT so a missing/invalid VERSION can never
# stamp a payload, and failure writes (and deletes) nothing (TASK-IMP-068 §1 #3).
[ -f "$repo/VERSION" ] || { echo "cyberos: ERROR: $repo/VERSION missing - refusing to build an unstamped payload" >&2; exit 2; }
cyver="$(tr -d ' \n\r' < "$repo/VERSION")"
printf '%s' "$cyver" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$' || { echo "cyberos: ERROR: VERSION is not X.Y.Z semver (got '$cyver')" >&2; exit 2; }

echo "cyberos: assembling payload into $out"
rm -rf "$out"
mkdir -p "$out/cuo/skills" "$out/cuo/gates/caf" "$out/cuo/templates" "$out/memory"

# --- cuo module: the workflow engine ---
cp "$repo/modules/cuo/chief-technology-officer/workflows/ship-tasks.md" "$out/cuo/ship-tasks.md"
cp "$repo/modules/cuo/EXECUTION-DISCIPLINE.md"                                       "$out/cuo/EXECUTION-DISCIPLINE.md"
cp "$repo/modules/skill/contracts/task/STATUS-REFERENCE.md"              "$out/cuo/STATUS-REFERENCE.md"
cp "$here/gates/run-gates.sh"                                                        "$out/cuo/gates/run-gates.sh"
cp -R "$here/templates/." "$out/cuo/templates/"

# Per-type body templates. task-author dispatches on templates/{type}.md and HALTs when
# one is missing (task-author/SKILL.md §4 W2) — deliberately, so a missing template is
# loud rather than silently resolved to `feature`.
#
# These were NOT shipped until 2026-07-15. The type discriminator added them under
# modules/skill/contracts/task/templates/ and wired the skill to dispatch on them, but
# never added them to the payload — so every installed repo vendored a task-author that
# HALTed at W2 on its first task. Verified on a clean install: `find .cyberos -name
# feature.md` returned nothing.
#
# They flatten into cuo/templates/ next to TASK-TEMPLATE.md, matching how
# STATUS-REFERENCE.md (also from contracts/task/) flattens into cuo/ above — the
# installed tree has no skill/ or contracts/ root. No name collision: this dir holds
# BACKLOG.md + TASK-TEMPLATE.md.
#
# Gated by scripts/tests/test_template_schema.sh t07.
cp "$repo/modules/skill/contracts/task/templates/"*.md "$out/cuo/templates/"

# Per-type RULE families (rubrics) — the same incident class as the templates above,
# found on the 2026-07-16 sachviet consumer-repo audit: the vendored task-audit
# RUBRIC.md dispatches `type: bug` to the BUG-*/REGRESSION-* family and FM-114 cites
# `contracts/task/rubrics/bug.md`, but the payload shipped no rubrics/ at all — so a
# consumer repo could author a bug it could never audit. They flatten into cuo/rubrics/
# next to cuo/templates/, matching the STATUS-REFERENCE flattening convention.
#
# Gated by tools/install/tests/test_rubrics_vendored.sh (real-repo builds MUST carry them);
# best-effort here like the skill bodies, so a minimal/doc-driven source tree still builds —
# but loudly, because a silent skip is exactly how the templates went missing.
if ls "$repo/modules/skill/contracts/task/rubrics/"*.md >/dev/null 2>&1; then
  mkdir -p "$out/cuo/rubrics"
  cp "$repo/modules/skill/contracts/task/rubrics/"*.md "$out/cuo/rubrics/"
else
  echo "cyberos: WARN no modules/skill/contracts/task/rubrics/ in source - payload ships without per-type rule families (task-audit cannot enforce BUG-*/REGRESSION-*)" >&2
fi

# TASK-IMP-111: the plan workflow's standalone rubric. plan-audit loads plan_rubric@1.0 from
# modules/skill/rubrics/plan_rubric.md and flattens it into cuo/rubrics/ next to bug.md/common.md
# (the installed home .cyberos/cuo/rubrics/). A rubric correct in modules/ and absent from dist/ is
# correct nowhere: the vendored plan-audit would name a rubric no installed repo carries. Best-effort
# like the block above, but loud — a silent skip is exactly how the per-type templates went missing.
if [ -f "$repo/modules/skill/rubrics/plan_rubric.md" ]; then
  mkdir -p "$out/cuo/rubrics"
  cp "$repo/modules/skill/rubrics/plan_rubric.md" "$out/cuo/rubrics/plan_rubric.md"
else
  echo "cyberos: WARN no modules/skill/rubrics/plan_rubric.md in source - payload ships without plan_rubric@1.0 (plan-audit cannot enforce PLAN-*)" >&2
fi

# The vendored skill set - one name per line with its SDP stage, in lifecycle order
# (TASK-CUO-209: reviewable data, not a drifting string; TASK-SKILL-116's chain-coverage
# check runs at the end of this build and fails on any under-coverage).
skills="$(sed 's/#.*//' <<'VENDORED_SKILLS' | xargs
statement-of-work-author                    # SDP 1  SOW
statement-of-work-audit                     # SDP 1
product-requirements-document-author        # SDP 2  PRD
product-requirements-document-audit         # SDP 2
software-requirements-specification-author  # SDP 3  SRS
software-requirements-specification-audit   # SDP 3
nfr-certification-author                    # SDP 4  NFR (allowlisted unpaired)
nfr-evaluator                               # SDP 4  NFR
nfr-test-runner                             # SDP 4  NFR
nfr-regression-handler                      # SDP 4  NFR
plan-author                      # SDP 5  plan (front door: idea -> plan@1 -> create-tasks, TASK-IMP-111)
plan-audit                       # SDP 5  plan
task-author                      # SDP 5  task
task-audit                       # SDP 5
task-reconcile                   # SDP 5  reconcile drifted entry states (TASK-IMP-100)
architectural-spike-author                  # SDP 6  spike (ADR input)
architectural-spike-audit                   # SDP 6
architecture-decision-record-author         # SDP 6  ADR
architecture-decision-record-audit          # SDP 6
threat-model-author                         # SDP 6  threat model
threat-model-audit                          # SDP 6
software-design-document-author             # SDP 7  SDD
software-design-document-audit              # SDP 7
repo-context-map-author                     # SDP 8  implementation
repo-context-map-audit                      # SDP 8
implementation-plan-author                  # SDP 8
implementation-plan-audit                   # SDP 8
edge-case-matrix-author                     # SDP 8
edge-case-matrix-audit                      # SDP 8
mock-contract-test-author                   # SDP 8
mock-contract-test-audit                    # SDP 8
observability-injection-author              # SDP 8
observability-injection-audit               # SDP 8
backlog-state-update-author                 # SDP 8
backlog-state-update-audit                  # SDP 8
code-review-author                          # SDP 9  review
code-review-audit                           # SDP 9
test-strategy-author                        # SDP 10 test
test-strategy-audit                         # SDP 10
coverage-gate-author                        # SDP 10
coverage-gate-audit                         # SDP 10
debugging-cycle-author                      # SDP 10 (TASK-SKILL-116)
debugging-cycle-audit                       # SDP 10
deployment-checklist-author                 # SDP 11 deploy
deployment-checklist-audit                  # SDP 11
release-notes-author                        # SDP 12 release
release-notes-audit                         # SDP 12
runbook-author                              # SDP 13 runbook
runbook-audit                               # SDP 13
retrospective-author                        # SDP 14 retro
retrospective-audit                         # SDP 14
postmortem-author                           # SDP 14
postmortem-audit                            # SDP 14
decommissioning-author                      # SDP 14
decommissioning-audit                       # SDP 14
workflow-improver                # outer loop: propose skill amendments from run evidence (TASK-IMP-110)
VENDORED_SKILLS
)"
vendored_skills=0
for s in $skills; do
  if [ -d "$repo/modules/skill/$s" ]; then
    cp -R "$repo/modules/skill/$s" "$out/cuo/skills/$s"
    vendored_skills=$((vendored_skills + 1))
  fi
done

caf_vendored="no"
if [ -f "$repo/scripts/caf_gate.sh" ] && [ -d "$repo/tools/caf" ]; then
  cp "$repo/scripts/caf_gate.sh" "$out/cuo/gates/caf/caf_gate.sh"
  cp -R "$repo/tools/caf" "$out/cuo/gates/caf/caf"
  caf_vendored="yes"
fi

# --- memory module: Layer-1 protocol + schema + invariants ---
cp "$repo/AGENTS.md" "$out/memory/AGENTS.md"
memory_vendored="protocol"
[ -f "$repo/modules/memory/memory.schema.json" ]    && { cp "$repo/modules/memory/memory.schema.json"    "$out/memory/memory.schema.json";    memory_vendored="protocol+schema"; }
[ -f "$repo/modules/memory/memory.invariants.yaml" ] && cp "$repo/modules/memory/memory.invariants.yaml" "$out/memory/memory.invariants.yaml"

# --- plugin + runtime + docs ---
rm -rf "$repo/dist/task-pack"   # self-heal: purge the pre-rename payload if a stale copy lingers
cp -R "$here/plugin"    "$out/plugin"
# Marketplace manifest at the payload ROOT: lets Claude add dist/cyberos as a plugin
# marketplace (`/plugin marketplace add <path>` or the desktop Plugins > Add picker),
# then install the `cyberos` plugin from it. plugin/.claude-plugin/plugin.json is the
# plugin's own manifest; this file is the catalog pointing at it.
mkdir -p "$out/.claude-plugin"
cp "$here/marketplace/.claude-plugin/marketplace.json" "$out/.claude-plugin/marketplace.json"
cp "$here/install.sh"   "$out/install.sh"
cp "$here/uninstall.sh" "$out/uninstall.sh"
cp "$here/check-latest.sh"      "$out/check-latest.sh"
# Portable lib + docs-tools (status-page hooks, update soft-check). No migrate-tasks / install.sh.
mkdir -p "$out/lib"
# version-compare.sh is REQUIRED, not optional: install.sh's downgrade guard and update-check.sh
# both source it, and a payload without it would silently skip the guard (TASK-IMP-104).
cp "$here/lib/version-compare.sh" "$out/lib/version-compare.sh"
[ -f "$here/lib/task-migrate.sh" ] && cp "$here/lib/task-migrate.sh" "$out/lib/task-migrate.sh"
[ -f "$here/lib/update-check.sh" ] && cp "$here/lib/update-check.sh" "$out/lib/update-check.sh"
[ -f "$here/lib/status-page.sh" ] && cp "$here/lib/status-page.sh" "$out/lib/status-page.sh"
# (docs-tools sources may be absent in trimmed fixture builds — vendor what exists)
if [ -f "$here/../../scripts/migrate_task_layout.py" ]; then
  mkdir -p "$out/docs-tools/templates"
  cp "$here/../../scripts/migrate_task_layout.py" "$out/docs-tools/"
  [ -f "$here/../../scripts/repair_task_yaml.py" ] && cp "$here/../../scripts/repair_task_yaml.py" "$out/docs-tools/"
  # status page: render-status-hub.mjs + md.mjs + templates (all five or half-render fails loudly)
  [ -f "$here/../docs-site/render-status-hub.mjs" ] && cp "$here/../docs-site/render-status-hub.mjs" "$out/docs-tools/"
  [ -f "$here/../docs-site/md.mjs" ] && cp "$here/../docs-site/md.mjs" "$out/docs-tools/"
  # task-lint: deterministic machine floor under audit_rubric@2.0 (TASK-IMP-084)
  [ -f "$here/docs-tools/task-lint.mjs" ] && cp "$here/docs-tools/task-lint.mjs" "$out/docs-tools/"
  # batch-select: the maximal cone-independent batch, computed (v2.8.0). ship-tasks §11a runs it
  # before step 1, so a payload without it cannot obey the batch rule.
  [ -f "$here/docs-tools/batch-select.mjs" ] && cp "$here/docs-tools/batch-select.mjs" "$out/docs-tools/"
  # cone-audit: reports a task's writes that escape its DECLARED cone (TASK-IMP-119), run at the
  # implementing -> ready_to_review flip. It mirrors batch-select's containment + (none) filter
  # verbatim; a payload without it ships batch-select's promise with nothing checking it.
  [ -f "$here/docs-tools/cone-audit.mjs" ] && cp "$here/docs-tools/cone-audit.mjs" "$out/docs-tools/"
  # verify-goals: re-verifies what done claimed (TASK-IMP-109). ship-tasks §11c names the
  # vendored path, so a payload without it cannot obey the rule.
  [ -f "$here/docs-tools/verify-goals.mjs" ] && cp "$here/docs-tools/verify-goals.mjs" "$out/docs-tools/"
  # workflow helpers: ship-manifest@1 executor + backlog-state-update byte-discipline executor (TASK-IMP-085)
  [ -f "$here/docs-tools/ship-manifest.mjs" ] && cp "$here/docs-tools/ship-manifest.mjs" "$out/docs-tools/"
  [ -f "$here/docs-tools/backlog-mutate.mjs" ] && cp "$here/docs-tools/backlog-mutate.mjs" "$out/docs-tools/"
  # memory-append: doc-driven appender for the BRAIN audit chain (TASK-IMP-093)
  [ -f "$here/docs-tools/memory-append.mjs" ] && cp "$here/docs-tools/memory-append.mjs" "$out/docs-tools/"
  # coverage-scope: task diff -> per-file coverage skeleton (TASK-IMP-098)
  [ -f "$here/docs-tools/coverage-scope.mjs" ] && cp "$here/docs-tools/coverage-scope.mjs" "$out/docs-tools/"
  # TASK-IMP-100: reconcile the third state - work this workflow did not perform.
  [ -f "$here/docs-tools/task-reconcile.mjs" ] && cp "$here/docs-tools/task-reconcile.mjs" "$out/docs-tools/"
  # fm001-migrate: clean a repo's task corpus of FM-001 trailing frontmatter comments (TASK-IMP-117).
  # Vendored so any installed repo can run it against its OWN specs - the TASK-TEMPLATE.md that taught
  # the trailing-comment shape is itself vendored, so every consumer corpus inherited the violation.
  [ -f "$here/docs-tools/fm001-migrate.mjs" ] && cp "$here/docs-tools/fm001-migrate.mjs" "$out/docs-tools/"
  # workflow-improve: the outer loop's machine floor (TASK-IMP-110). The workflow-improver skill
  # and the /improve command both name `.cyberos/docs-tools/workflow-improve.mjs`, so a payload
  # without it ships a skill that cannot reach its own floor.
  [ -f "$here/docs-tools/workflow-improve.mjs" ] && cp "$here/docs-tools/workflow-improve.mjs" "$out/docs-tools/"
  [ -f "$here/../../modules/templates/html/status-hub.html" ] && cp "$here/../../modules/templates/html/status-hub.html" "$out/docs-tools/templates/"
  [ -f "$here/../../modules/templates/html/status-app.js" ] && cp "$here/../../modules/templates/html/status-app.js" "$out/docs-tools/templates/"
  [ -f "$here/../../modules/templates/cds/status.css" ] && cp "$here/../../modules/templates/cds/status.css" "$out/docs-tools/templates/"
  [ -f "$here/../../modules/templates/cds/tokens.css" ] && cp "$here/../../modules/templates/cds/tokens.css" "$out/docs-tools/templates/"
fi
# Never ship retired names (pre-1.0.0)
# (retired-orphan scrub removed at 1.0.0: no build step has emitted migrate-tasks.sh /
#  install.sh / changelog.sh / update.sh since bb0f2392e, so this deleted nothing.)
cp "$here/bootstrap.sh" "$out/bootstrap.sh"
cp "$here/create.sh"    "$out/create.sh"        # template / fresh-project scaffolder channel
# 1.0.0 CLI: install | uninstall | version | status | help
cp "$here/version.sh"   "$out/version.sh"
cp "$here/status.sh"    "$out/status.sh"
cp "$here/help.sh"      "$out/help.sh"
cp -R "$here/ci"        "$out/ci"
cp -R "$here/mcp"       "$out/mcp"              # MCP server channel (node stdio; every MCP agent)
cp -R "$here/cli"       "$out/cli"              # npx CLI channel (one bin: `cyberos <command>`)
cp -R "$here/template"  "$out/template"         # skeleton for create.sh + GitHub template repo
cp "$here/Dockerfile"   "$out/Dockerfile"
cp "$here/README.md"    "$out/README.md"
cp "$here/docs/index.md" "$out/GUIDE.md"   # the guide source lives in docs/ (site-rendered); ships as GUIDE.md
chmod +x "$out/install.sh" "$out/uninstall.sh" "$out/bootstrap.sh" "$out/create.sh" \
  "$out/version.sh" "$out/status.sh" "$out/help.sh" "$out/cuo/gates/run-gates.sh" 2>/dev/null || true
chmod +x "$out/lib/"*.sh "$out/mcp/cyberos-mcp.mjs" "$out"/cli/bin/*.mjs 2>/dev/null || true

# $cyver was validated + computed at the TOP of this script (TASK-IMP-068: fail-fast, no 0.0.0 fallback).

# make the plugin self-contained: carry the cuo docs so the bundled skill works standalone
mkdir -p "$out/plugin/skills/ship-tasks/cuo"
cp "$out/cuo/ship-tasks.md" "$out/cuo/EXECUTION-DISCIPLINE.md" "$out/cuo/STATUS-REFERENCE.md" "$out/plugin/skills/ship-tasks/cuo/"

# Bundle EVERY vendored skill into the plugin so it is genuinely self-contained.
# Why all of them: ship-tasks CHAINS ~18 author/audit skills (repo-context-map,
# edge-case-matrix, implementation-plan, observability-injection, code-review, coverage-gate, ...).
# Those only existed under .cyberos/cuo/skills/ after /install, so the plugin's bundled workflow could
# not reach its own children standalone - the plugin shipped the conductor without the orchestra.
# Task-{author,audit} additionally back /create-tasks. ~860K total; the zip
# stays well under a megabyte, so there is no reason to ship a partial set.
plugin_skills=0
for d in "$out"/cuo/skills/*/; do
  [ -d "$d" ] || continue
  s="$(basename "$d")"
  [ -e "$out/plugin/skills/$s" ] && continue     # never clobber a skill the plugin ships itself
  cp -R "$d" "$out/plugin/skills/$s"
  plugin_skills=$((plugin_skills + 1))
done

# Fail-closed guard: the plugin host REJECTS any bundled skill whose frontmatter `description`
# exceeds 1024 chars ("Plugin validation failed: field 'description' in SKILL.md must be at most
# 1024 characters"). The vendored .cyberos/cuo/skills copies have no such limit, so this guards only
# what we bundle. We FAIL rather than silently truncate: a truncated description loses its trailing
# "Do NOT use ..." routing clause, which quietly degrades skill selection.
if command -v python3 >/dev/null 2>&1; then
  python3 - "$out/plugin/skills" <<'PYCAP' || { echo "cyberos: fix the skill error(s) above (structure: rename the dir or its frontmatter name; length: shorten the description in modules/skill/<name>/SKILL.md), then rebuild." >&2; exit 1; }
import os, re, sys
root, LIMIT, bad = sys.argv[1], 1024, []
# A bundled skill dir with no SKILL.md does not load, and a dir whose name != frontmatter
# `name:` loads under the wrong id. Both are silent: the description scan below used to
broken = []
for name in sorted(os.listdir(root)):
    d = os.path.join(root, name)
    if not os.path.isdir(d):
        continue
    f = os.path.join(d, "SKILL.md")
    if not os.path.isfile(f):
        broken.append(f"skill '{name}' has no SKILL.md (dir will not load as a skill)")
        continue
    m0 = re.match(r"^---\n(.*?)\n---\n", open(f).read(), re.S)
    got = re.search(r"^name:\s*(.+)$", m0.group(1), re.M) if m0 else None
    got = got.group(1).strip() if got else ""
    if got != name:
        broken.append(f"skill '{name}' frontmatter name is '{got or '<missing>'}' (must equal the directory name)")
for msg in broken:
    print(f"cyberos: ERROR {msg}", file=sys.stderr)
if broken:
    sys.exit(1)
for name in sorted(os.listdir(root)):
    f = os.path.join(root, name, "SKILL.md")
    if not os.path.isfile(f):
        continue
    m = re.match(r"^---\n(.*?)\n---\n", open(f).read(), re.S)
    if not m:
        continue
    fm = m.group(1)
    b = re.search(r"^description:\s*(?:>-|>|\|)?\s*\n((?:[ \t]+.*\n?)+)", fm, re.M)
    if b:
        desc = " ".join(l.strip() for l in b.group(1).splitlines()).strip()
    else:
        s = re.search(r"^description:\s*(.+)$", fm, re.M)
        desc = s.group(1).strip() if s else ""
    if len(desc) > LIMIT:
        bad.append((len(desc), name))
for n, name in sorted(bad, reverse=True):
    print(f"cyberos: ERROR skill '{name}' description is {n} chars (plugin limit {LIMIT})", file=sys.stderr)
sys.exit(1 if bad else 0)
PYCAP
fi

# stamp BOTH manifests with the platform VERSION so the plugin never drifts from CyberOS again.
# Sources carry "0.0.0" as the placeholder; this is the single point that sets the real number.
for m in "$out/plugin/.claude-plugin/plugin.json" "$out/.claude-plugin/marketplace.json"; do
  [ -f "$m" ] || continue
  sed "s/\"version\": \"0.0.0\"/\"version\": \"$cyver\"/" "$m" > "$m.tmp" && mv "$m.tmp" "$m"
done

# One-file plugin bundle for pickers that want a FILE, not a folder (Claude desktop's
# Add dialog greys "Open" on directories): zip the plugin dir into cyberos.plugin.
(cd "$out/plugin" && rm -f ../cyberos.plugin && zip -qr ../cyberos.plugin .claude-plugin commands skills)

# --- profile + manifest ---
profile="reduced"
[ "$vendored_skills" -gt 0 ] && [ "$caf_vendored" = "yes" ] && profile="full"
ver="$(cd "$repo" && git rev-parse --short HEAD 2>/dev/null || echo unknown)"
# $cyver was computed above (before the plugin manifests were stamped + zipped).
echo "$cyver" > "$out/VERSION"     # plain file so version.sh can compare fast

# root package.json makes the payload npx/npm-installable. ONE bin: `cyberos` is the trigger
# keyword and everything after it is a command (`npx cyberos install [dir]`, `npx cyberos gates`,
# `npx cyberos mcp`, `npx cyberos --help`). Three verb-named bins meant `cyberos-install --help`
# read as nonsense and the bin name had to change every time the verb did — which is precisely
# how `init` outlived the init->install rename. cli/bin/cli.mjs dispatches; the command set
# mirrors help.sh and the plugin's slash commands 1:1, so the channels cannot drift.
cat > "$out/package.json" <<PKG
{
  "name": "@cyberskill/cyberos",
  "version": "$cyver",
  "description": "Run the CyberOS ship-tasks workflow in any repo - install, gates, and an MCP server, wired for every popular coding agent (Claude Code, Codex, Cursor, Gemini, Antigravity, Grok CLI, zcode, Command Code, Copilot, Windsurf).",
  "type": "module",
  "bin": {
    "cyberos": "cli/bin/cli.mjs"
  },
  "engines": { "node": ">=18" },
  "repository": { "type": "git", "url": "git+https://github.com/cyberskill-official/cyberos.git" },
  "homepage": "https://github.com/cyberskill-official/cyberos#readme",
  "bugs": { "url": "https://github.com/cyberskill-official/cyberos/issues" },
  "files": ["cuo", "memory", "plugin", "mcp", "cli", "ci", "template", "lib", "install.sh", "uninstall.sh", "version.sh", "status.sh", "help.sh", "bootstrap.sh", "create.sh", "Dockerfile", "VERSION", "manifest.yaml", "GUIDE.md", "README.md", ".claude-plugin"],
  "keywords": ["cyberos", "tasks", "agents", "mcp", "workflow", "hitl", "gates"],
  "license": "MIT"
}
PKG
# repository.url is NOT decoration. Trusted publishing (OIDC) rejects the publish unless it
# exactly matches the GitHub repo the workflow runs in -- npm's own docs call this out as a
# common failure for "misconfigured packages", and it surfaces as an auth error rather than
# as "your metadata is wrong". release.yml asserts it before calling publish so the message
# names the cause.
# stamp the standalone mcp package version too (best-effort; ignore if sed is quirky).
# Match ANY current version, not just the "0.0.0" placeholder: since the repo source at
# tools/install/mcp/package.json is itself stamped at bump time (stamp-release-version.mjs),
# a payload built between bumps must still land at exactly $cyver regardless of what the source says.
[ -f "$out/mcp/package.json" ] && sed -E "s/\"version\": \"[^\"]*\"/\"version\": \"$cyver\"/" "$out/mcp/package.json" > "$out/mcp/package.json.tmp" && mv "$out/mcp/package.json.tmp" "$out/mcp/package.json"
# TASK-IMP-074 group C: deterministic fingerprint over the DISTRIBUTED rule trees, so every
# channel (self-hosted .cyberos, claude plugin, mcp server, npx cli) can detect rule drift
# even when cyberos_version is unchanged. LC_ALL=C pins sort order across platforms;
# sha256sum on linux CI, shasum -a 256 on operator macs (same two-space text-mode output).
_rsha() { if command -v sha256sum >/dev/null 2>&1; then sha256sum "$@"; else shasum -a 256 "$@"; fi; }
rules_sha="$(cd "$out" && find cuo plugin mcp cli memory -type f 2>/dev/null | LC_ALL=C sort \
  | while IFS= read -r f; do _rsha "$f"; done | _rsha | cut -d' ' -f1)"
[ -n "$rules_sha" ] || { echo "cyberos: ERROR: rules_sha computation produced nothing" >&2; exit 2; }

cat > "$out/manifest.yaml" <<EOF
# Generated by tools/install/build.sh - do not edit by hand.
vendor: cyberos
cyberos_version: $cyver     # the single distribution version (module versions are internal)
rules_sha: $rules_sha     # TASK-IMP-074: content fingerprint of the distributed rule trees (cuo/plugin/mcp/cli/memory)
built_from_commit: $ver
built_at: $(date -u +%Y-%m-%dT%H:%M:%SZ)
profile: $profile          # full = skills + caf vendored; reduced = doc-driven + repo's own gates
modules:
  cuo:    { workflow: ship-tasks, normative_docs: 3, author_audit_skills: $vendored_skills, caf_tooling: $caf_vendored }
  memory: { protocol: $memory_vendored }
channels: [copy-folder, git-submodule, curl-bootstrap, claude-plugin, mcp-server, mcp-connector, npx-cli, root-cli, template-repo, create-scaffold, github-action, docker, makefile]
agents:   [claude-code, codex, cursor, gemini, antigravity, grok-cli, zcode, command-code, copilot, windsurf, aider, zed, jules, warp, opencode]
agent_surface:
  spine: AGENTS.md                       # canonical cross-agent instruction file
  pointer_files: [CLAUDE.md, GEMINI.md, .cursorrules, .cursor/rules/cyberos.mdc, .grok/GROK.md, .github/copilot-instructions.md, .agents/rules/cyberos.md, .windsurfrules]
  native_skill_dirs: [.claude/skills, .grok/skills, .commandcode/skills, .codex/skills, .opencode/skill]
  mcp: { server: mcp/cyberos-mcp.mjs, tools: [task_install, task_gates, task_status, ship_task] }
notes: >
  install.sh lays this out under a target repo's gitignored .cyberos/, by module. Doc-driven
  mode always works; missing skills/caf degrade to the reduced-profile floor (the target
  repo's own build/lint/test + coverage + code review + the two human gates).
EOF

# TASK-SKILL-116 §1 #5: a payload that under-covers its own workflow cannot be produced.
bash "$here/check-chain-coverage.sh" "$out"
bash "$here/check-pair-parity.sh" "$out/cuo/skills" || exit $?   # TASK-SKILL-118: pair contract parity over the vendored set

# TASK-CUO-209: report sizes on every build; the plugin zip carries a hard 2 MB budget.
payload_bytes=$(du -sk "$out" | awk '{print $1*1024}')   # KB granularity, portable (GNU + BSD du)
# tr -d ' ': BSD `wc -c` PADS its output to a fixed width, GNU does not. Unstripped, macOS
# emitted `plugin_zip=       1103748` and the size line stopped matching `plugin_zip=[0-9]`
# — so the budget report was malformed on the platform this repo is developed on. The
# payload_bytes line above already says "portable (GNU + BSD du)"; the thought stopped one
# line short.
plugin_bytes=$(wc -c < "$out/cyberos.plugin" | tr -d ' ')
[ "$plugin_bytes" -le 2097152 ] || { echo "cyberos: ERROR: cyberos.plugin ${plugin_bytes}B exceeds the 2 MB budget" >&2; exit 2; }

echo "cyberos: done. profile=$profile skills=$vendored_skills caf=$caf_vendored payload=${payload_bytes} plugin_zip=${plugin_bytes}"
echo "cyberos: payload at $out - install.sh lays it out under a target repo's .cyberos/ (gitignored)"

# Host-plugin refresh (developer machine only). Claude + Grok keep their own install caches;
# a rebuild of dist/cyberos does not update them. Soft /version checks cover the REPO machine
# (.cyberos/), not the host plugin — so without this step the host can stay on an old cache
# while the payload is already new. Best-effort: never fails the build.
#
#   default (auto): run only when $out is the canonical $repo/dist/cyberos (scratch/CI
#                   payload paths used by tests skip automatically)
#   CYBEROS_SYNC_HOST_PLUGINS=1  force for any $out
#   CYBEROS_SYNC_HOST_PLUGINS=0  skip even the default path
#   CYBEROS_OFFLINE=1            skip (honoured inside sync-host-plugins.sh)
_sync_mode="${CYBEROS_SYNC_HOST_PLUGINS:-auto}"
_do_host_sync=0
if [ "$_sync_mode" = "1" ]; then
  _do_host_sync=1
elif [ "$_sync_mode" = "auto" ]; then
  # Resolve both sides to physical paths so /var vs /private/var (macOS) does not skip
  # a real developer build. Do NOT mkdir the default path here — scratch builds must not
  # create dist/cyberos as a side effect of the comparison.
  _out_phys="$(cd "$out" 2>/dev/null && pwd -P || echo "$out")"
  if [ -d "$repo/dist/cyberos" ]; then
    _def_phys="$(cd "$repo/dist/cyberos" && pwd -P)"
  else
    _def_phys="$repo/dist/cyberos"
  fi
  [ "$_out_phys" = "$_def_phys" ] && _do_host_sync=1
fi
if [ "$_do_host_sync" -eq 1 ] && [ -f "$here/sync-host-plugins.sh" ]; then
  if ! bash "$here/sync-host-plugins.sh" "$out"; then
    echo "cyberos: WARN host plugin sync failed — payload at $out is still good; re-run: bash tools/install/sync-host-plugins.sh" >&2
  fi
fi
