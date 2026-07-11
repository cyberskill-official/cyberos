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

echo "cyberos-init: assembling payload into $out"
rm -rf "$out"
mkdir -p "$out/cuo/skills" "$out/cuo/gates/caf" "$out/cuo/templates" "$out/memory"

# --- cuo module: the workflow engine ---
cp "$repo/modules/cuo/chief-technology-officer/workflows/ship-feature-requests.md" "$out/cuo/ship-feature-requests.md"
cp "$repo/modules/cuo/EXECUTION-DISCIPLINE.md"                                       "$out/cuo/EXECUTION-DISCIPLINE.md"
cp "$repo/modules/skill/contracts/feature-request/STATUS-REFERENCE.md"              "$out/cuo/STATUS-REFERENCE.md"
cp "$here/gates/run-gates.sh"                                                        "$out/cuo/gates/run-gates.sh"
cp -R "$here/templates/." "$out/cuo/templates/"

skills="repo-context-map-author repo-context-map-audit architecture-decision-record-author architecture-decision-record-audit edge-case-matrix-author edge-case-matrix-audit mock-contract-test-author mock-contract-test-audit implementation-plan-author implementation-plan-audit observability-injection-author observability-injection-audit backlog-state-update-author backlog-state-update-audit code-review-author code-review-audit coverage-gate-author coverage-gate-audit feature-request-author feature-request-audit"
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

# The single platform VERSION. Computed HERE (not later) because the plugin manifests must be
# stamped with it BEFORE the one-file bundle is zipped - a stale version sealed inside
# cyberos.plugin is exactly the drift this fixes (installed plugin said 1.0.0 forever).
cyver="$(tr -d ' \n\r' < "$repo/VERSION" 2>/dev/null || echo 0.0.0)"

# make the plugin self-contained: carry the cuo docs so the bundled skill works standalone
mkdir -p "$out/plugin/skills/ship-feature-requests/cuo"
cp "$out/cuo/ship-feature-requests.md" "$out/cuo/EXECUTION-DISCIPLINE.md" "$out/cuo/STATUS-REFERENCE.md" "$out/plugin/skills/ship-feature-requests/cuo/"

# bundle the FR AUTHORING skills into the plugin so /new-fr works standalone: feature-request-author
# drafts the FRs, feature-request-audit drives draft -> ready_to_implement, which is what
# /ship-feature-requests then consumes. Without these the plugin can only ship, never author.
plugin_skills=0
for s in feature-request-author feature-request-audit; do
  if [ -d "$out/cuo/skills/$s" ]; then
    cp -R "$out/cuo/skills/$s" "$out/plugin/skills/$s"
    plugin_skills=$((plugin_skills + 1))
  fi
done

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

echo "cyberos-init: done. profile=$profile skills=$vendored_skills caf=$caf_vendored"
echo "cyberos-init: payload at $out - init.sh lays it out under a target repo's .cyberos/ (gitignored)"
