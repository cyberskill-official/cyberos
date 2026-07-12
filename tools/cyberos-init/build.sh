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

skills="repo-context-map-author repo-context-map-audit architecture-decision-record-author architecture-decision-record-audit edge-case-matrix-author edge-case-matrix-audit mock-contract-test-author mock-contract-test-audit implementation-plan-author implementation-plan-audit observability-injection-author observability-injection-audit backlog-state-update-author backlog-state-update-audit code-review-author code-review-audit coverage-gate-author coverage-gate-audit feature-request-author feature-request-audit debugging-cycle-author debugging-cycle-audit"  # FR-SKILL-116: ship steps 25-26
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
# stamp the standalone mcp package version too (best-effort; ignore if sed is quirky)
[ -f "$out/mcp/package.json" ] && sed "s/\"version\": \"0.0.0\"/\"version\": \"$cyver\"/" "$out/mcp/package.json" > "$out/mcp/package.json.tmp" && mv "$out/mcp/package.json.tmp" "$out/mcp/package.json"
cat > "$out/manifest.yaml" <<EOF
# Generated by tools/cyberos-init/build.sh - do not edit by hand.
vendor: cyberos
cyberos_version: $cyver     # the single distribution version (module versions are internal)
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

echo "cyberos-init: done. profile=$profile skills=$vendored_skills caf=$caf_vendored"
echo "cyberos-init: payload at $out - init.sh lays it out under a target repo's .cyberos/ (gitignored)"
