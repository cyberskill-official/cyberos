#!/usr/bin/env bash
# build.sh - assemble the vendorable CyberOS payload into dist/cyberos/.
# init.sh lays this out under a target repo's gitignored .cyberos/, by module:
#   .cyberos/cuo/     - the FR workflow engine (ship-feature-requests + doctrine + status
#                       contract + author/audit skills + gates + templates)
#   .cyberos/memory/  - the memory module (Layer-1 protocol + schema + invariants)
#   .cyberos/plugin/  - the Claude/Cowork plugin
# Best-effort: the three normative docs always vendor (doc-driven mode). Author/audit skill
# bodies and caf tooling vendor only if present (full mode); otherwise the reduced-profile
# floor applies (the target repo's own build/lint/test + code review + the two human gates).
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"                 # tools/cyberos-init
repo="$(cd "$here/../.." && pwd)"                      # cyberos repo root
out="${1:-$repo/dist/cyberos}"

# The single platform VERSION - validated UP FRONT so a missing/invalid VERSION can never
# stamp a payload, and failure writes (and deletes) nothing (FR-IMP-068 §1 #3).
[ -f "$repo/VERSION" ] || { echo "cyberos-init: ERROR: $repo/VERSION missing - refusing to build an unstamped payload" >&2; exit 2; }
cyver="$(tr -d ' \n\r' < "$repo/VERSION")"
printf '%s' "$cyver" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$' || { echo "cyberos-init: ERROR: VERSION is not X.Y.Z semver (got '$cyver')" >&2; exit 2; }

echo "cyberos-init: assembling payload into $out"
rm -rf "$out"
mkdir -p "$out/cuo/skills" "$out/cuo/gates/caf" "$out/cuo/templates" "$out/memory"

# --- cuo module: the workflow engine ---
cp "$repo/modules/cuo/chief-technology-officer/workflows/ship-feature-requests.md" "$out/cuo/ship-feature-requests.md"
cp "$repo/modules/cuo/EXECUTION-DISCIPLINE.md"                                       "$out/cuo/EXECUTION-DISCIPLINE.md"
cp "$repo/modules/skill/contracts/feature-request/STATUS-REFERENCE.md"              "$out/cuo/STATUS-REFERENCE.md"
cp "$here/gates/run-gates.sh"                                                        "$out/cuo/gates/run-gates.sh"
cp -R "$here/templates/." "$out/cuo/templates/"

# The vendored skill set - one name per line with its SDP stage, in lifecycle order
# (FR-CUO-209: reviewable data, not a drifting string; FR-SKILL-116's chain-coverage
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
feature-request-author                      # SDP 5  FR
feature-request-audit                       # SDP 5
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
debugging-cycle-author                      # SDP 10 (FR-SKILL-116)
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
rm -rf "$repo/dist/fr-pack"   # self-heal: purge the pre-rename payload if a stale copy lingers
cp -R "$here/plugin"    "$out/plugin"
# Marketplace manifest at the payload ROOT: lets Claude add dist/cyberos as a plugin
# marketplace (`/plugin marketplace add <path>` or the desktop Plugins > Add picker),
# then install the `cyberos` plugin from it. plugin/.claude-plugin/plugin.json is the
# plugin's own manifest; this file is the catalog pointing at it.
mkdir -p "$out/.claude-plugin"
cp "$here/marketplace/.claude-plugin/marketplace.json" "$out/.claude-plugin/marketplace.json"
cp "$here/init.sh"      "$out/init.sh"
cp "$here/check-latest.sh"      "$out/check-latest.sh"
# FR 1.0.0 portable migration kit (migrate-frs.sh + docs-tools) - target repos adopt folder-per-FR
# + the CDS status page without cloning the monorepo.
# (kit sources may be absent in trimmed/reduced fixture builds - vendor what exists, skip the rest)
if [ -f "$here/migrate-frs.sh" ] && [ -f "$here/../../scripts/migrate_fr_layout.py" ]; then
  cp "$here/migrate-frs.sh" "$out/migrate-frs.sh"
  mkdir -p "$out/docs-tools/templates"
  cp "$here/../../scripts/migrate_fr_layout.py" "$out/docs-tools/"
  [ -f "$here/../../scripts/repair_fr_yaml.py" ] && cp "$here/../../scripts/repair_fr_yaml.py" "$out/docs-tools/"
  [ -f "$here/../docs-site/render-status-hub.mjs" ] && cp "$here/../docs-site/render-status-hub.mjs" "$out/docs-tools/"
  [ -f "$here/../../modules/templates/html/status-hub.html" ] && cp "$here/../../modules/templates/html/status-hub.html" "$out/docs-tools/templates/"
  [ -f "$here/../../modules/templates/cds/tokens.css" ] && cp "$here/../../modules/templates/cds/tokens.css" "$out/docs-tools/templates/"
fi
cp "$here/bootstrap.sh" "$out/bootstrap.sh"
cp "$here/create.sh"    "$out/create.sh"        # template / fresh-project scaffolder channel
cp -R "$here/ci"        "$out/ci"
cp -R "$here/mcp"       "$out/mcp"              # MCP server channel (node stdio; every MCP agent)
cp -R "$here/cli"       "$out/cli"              # npx CLI channel (cyberos-init / -gates / -mcp)
cp -R "$here/template"  "$out/template"         # skeleton for create.sh + GitHub template repo
cp "$here/Dockerfile"   "$out/Dockerfile"
cp "$here/README.md"    "$out/README.md"
cp "$here/docs/index.md" "$out/GUIDE.md"   # the guide source lives in docs/ (site-rendered); ships as GUIDE.md
chmod +x "$out/init.sh" "$out/bootstrap.sh" "$out/create.sh" "$out/cuo/gates/run-gates.sh" 2>/dev/null || true
chmod +x "$out/mcp/cyberos-mcp.mjs" "$out"/cli/bin/*.mjs 2>/dev/null || true

# $cyver was validated + computed at the TOP of this script (FR-IMP-068: fail-fast, no 0.0.0 fallback).

# make the plugin self-contained: carry the cuo docs so the bundled skill works standalone
mkdir -p "$out/plugin/skills/ship-feature-requests/cuo"
cp "$out/cuo/ship-feature-requests.md" "$out/cuo/EXECUTION-DISCIPLINE.md" "$out/cuo/STATUS-REFERENCE.md" "$out/plugin/skills/ship-feature-requests/cuo/"

# Bundle EVERY vendored skill into the plugin so it is genuinely self-contained.
# Why all of them: ship-feature-requests CHAINS ~18 author/audit skills (repo-context-map,
# edge-case-matrix, implementation-plan, observability-injection, code-review, coverage-gate, ...).
# Those only existed under .cyberos/cuo/skills/ after /init, so the plugin's bundled workflow could
# not reach its own children standalone - the plugin shipped the conductor without the orchestra.
# feature-request-{author,audit} additionally back /create-feature-requests. ~860K total; the zip
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
  python3 - "$out/plugin/skills" <<'PYCAP' || { echo "cyberos-init: shorten the description(s) above in modules/skill/<name>/SKILL.md, then rebuild." >&2; exit 1; }
import os, re, sys
root, LIMIT, bad = sys.argv[1], 1024, []
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
    print(f"cyberos-init: ERROR skill '{name}' description is {n} chars (plugin limit {LIMIT})", file=sys.stderr)
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
echo "$cyver" > "$out/VERSION"     # plain file so `init --check`/update can compare fast

# root package.json makes the payload npx/npm-installable: `npx cyberos-init [dir]`,
# `npx cyberos-gates`, `npx cyberos-mcp` (bins live in cli/bin, resolve sibling scripts).
cat > "$out/package.json" <<PKG
{
  "name": "cyberos-init",
  "version": "$cyver",
  "description": "Run the CyberOS ship-feature-requests workflow in any repo - init, gates, and an MCP server, wired for every popular coding agent (Claude Code, Codex, Cursor, Gemini, Antigravity, Grok CLI, zcode, Command Code, Copilot, Windsurf).",
  "type": "module",
  "bin": {
    "cyberos-init": "cli/bin/cyberos-init.mjs",
    "cyberos-gates": "cli/bin/cyberos-gates.mjs",
    "cyberos-mcp": "cli/bin/cyberos-mcp.mjs"
  },
  "engines": { "node": ">=18" },
  "files": ["cuo", "memory", "plugin", "mcp", "cli", "ci", "template", "init.sh", "bootstrap.sh", "create.sh", "Dockerfile", "VERSION", "manifest.yaml", "GUIDE.md", "README.md", ".claude-plugin"],
  "keywords": ["cyberos", "feature-requests", "agents", "mcp", "workflow", "hitl", "gates"],
  "license": "MIT"
}
PKG
# stamp the standalone mcp package version too (best-effort; ignore if sed is quirky).
# Match ANY current version, not just the "0.0.0" placeholder: since the repo source at
# tools/cyberos-init/mcp/package.json is itself stamped at bump time (stamp-release-version.mjs),
# a payload built between bumps must still land at exactly $cyver regardless of what the source says.
[ -f "$out/mcp/package.json" ] && sed -E "s/\"version\": \"[^\"]*\"/\"version\": \"$cyver\"/" "$out/mcp/package.json" > "$out/mcp/package.json.tmp" && mv "$out/mcp/package.json.tmp" "$out/mcp/package.json"
# FR-IMP-074 group C: deterministic fingerprint over the DISTRIBUTED rule trees, so every
# channel (self-hosted .cyberos, claude plugin, mcp server, npx cli) can detect rule drift
# even when cyberos_version is unchanged. LC_ALL=C pins sort order across platforms;
# sha256sum on linux CI, shasum -a 256 on operator macs (same two-space text-mode output).
_rsha() { if command -v sha256sum >/dev/null 2>&1; then sha256sum "$@"; else shasum -a 256 "$@"; fi; }
rules_sha="$(cd "$out" && find cuo plugin mcp cli memory -type f 2>/dev/null | LC_ALL=C sort \
  | while IFS= read -r f; do _rsha "$f"; done | _rsha | cut -d' ' -f1)"
[ -n "$rules_sha" ] || { echo "cyberos-init: ERROR: rules_sha computation produced nothing" >&2; exit 2; }

cat > "$out/manifest.yaml" <<EOF
# Generated by tools/cyberos-init/build.sh - do not edit by hand.
vendor: cyberos
cyberos_version: $cyver     # the single distribution version (module versions are internal)
rules_sha: $rules_sha     # FR-IMP-074: content fingerprint of the distributed rule trees (cuo/plugin/mcp/cli/memory)
built_from_commit: $ver
built_at: $(date -u +%Y-%m-%dT%H:%M:%SZ)
profile: $profile          # full = skills + caf vendored; reduced = doc-driven + repo's own gates
modules:
  cuo:    { workflow: ship-feature-requests, normative_docs: 3, author_audit_skills: $vendored_skills, caf_tooling: $caf_vendored }
  memory: { protocol: $memory_vendored }
channels: [copy-folder, git-submodule, curl-bootstrap, claude-plugin, mcp-server, npx-cli, template-repo, create-scaffold, github-action, docker, makefile]
agents:   [claude-code, codex, cursor, gemini, antigravity, grok-cli, zcode, command-code, copilot, windsurf, aider, zed, jules, warp, opencode]
agent_surface:
  spine: AGENTS.md                       # canonical cross-agent instruction file
  pointer_files: [CLAUDE.md, GEMINI.md, .cursorrules, .cursor/rules/cyberos.mdc, .grok/GROK.md, .github/copilot-instructions.md, .agents/rules/cyberos.md, .windsurfrules]
  native_skill_dirs: [.claude/skills, .grok/skills, .commandcode/skills, .codex/skills, .opencode/skill]
  mcp: { server: mcp/cyberos-mcp.mjs, tools: [fr_init, fr_gates, fr_status, ship_fr] }
notes: >
  init.sh lays this out under a target repo's gitignored .cyberos/, by module. Doc-driven
  mode always works; missing skills/caf degrade to the reduced-profile floor (the target
  repo's own build/lint/test + coverage + code review + the two human gates).
EOF

# FR-SKILL-116 §1 #5: a payload that under-covers its own workflow cannot be produced.
bash "$here/check-chain-coverage.sh" "$out"
bash "$here/check-pair-parity.sh" "$out/cuo/skills" || exit $?   # FR-SKILL-118: pair contract parity over the vendored set

# FR-CUO-209: report sizes on every build; the plugin zip carries a hard 2 MB budget.
payload_bytes=$(du -sk "$out" | awk '{print $1*1024}')   # KB granularity, portable (GNU + BSD du)
plugin_bytes=$(wc -c < "$out/cyberos.plugin")
[ "$plugin_bytes" -le 2097152 ] || { echo "cyberos-init: ERROR: cyberos.plugin ${plugin_bytes}B exceeds the 2 MB budget" >&2; exit 2; }

echo "cyberos-init: done. profile=$profile skills=$vendored_skills caf=$caf_vendored payload=${payload_bytes} plugin_zip=${plugin_bytes}"
echo "cyberos-init: payload at $out - init.sh lays it out under a target repo's .cyberos/ (gitignored)"
