#!/usr/bin/env bash
# apply-resolutions.sh
#
# Resolves the 5 content conflicts identified after the restore-and-merge:
#
#   1. CP naming clash:
#        - Rename prior modules.yaml `CP` (Client Portal) → `PORTAL`.
#        - Add new `CP` entry for Compliance Plane (cross-cutting).
#
#   2. Phase mapping (align modules.yaml to the 100-FR backlog):
#        - HR P1 → P2     - REW P1 → P2     - LEARN P1 → P2
#        - RES P3 → P2    - OKR P3 → P2     - DOC P4 → P3
#
#   3. Modules in FRs missing from modules.yaml:
#        - Add: PORTAL (renamed from CP), TEN, BILL, API, CP (Compliance Plane).
#        - Skip: CORP, GTM (not runtime modules — tracked in docs/tasks/ only).
#
#   4. Template adherence:
#        - Re-add `template: feature_request@1` to every FR in docs/tasks/.
#
#   5. FR section list canonical-isation:
#        - Adopt union of prior + new sections.
#        - Insert stub "Alternatives Considered" + "AI Authorship Disclosure"
#          into every FR.
#        - Insert stub "Customer Quotes" + "Sales/CS Summary" into FRs whose
#          frontmatter has client_visible: true.
#        - Existing FR content is preserved verbatim; stubs are added at the
#          right location with TODO markers.
#
# Plus: delete docs/feature-requests/ (the 318 prior skeleton FRs the founder
# wants retired), and run a final audit pass that reports any FR still missing
# a required section.
#
# Run from repo root.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

echo "==> CyberOS — apply 5 content-conflict resolutions"
echo "    repo: $REPO_ROOT"
echo

# ─── 1. Delete prior 318 skeleton FRs ──────────────────────────────────────
echo "==> 1/6: Delete docs/feature-requests/ (318 prior skeleton FRs)"
if [ -d docs/feature-requests ]; then
  prior_count=$(find docs/feature-requests -name 'FR-*.md' 2>/dev/null | wc -l | tr -d ' ')
  rm -rf docs/feature-requests
  echo "    ✓ removed (had $prior_count prior FRs)"
else
  echo "    (already removed)"
fi
echo

# ─── 2. Update modules.yaml ────────────────────────────────────────────────
echo "==> 2/6: Update modules.yaml (CP rename, phase mapping, new modules)"
node "$REPO_ROOT/scripts/apply-resolutions.mjs" modules
echo

# ─── 3. Re-add template field + add missing sections to FRs ────────────────
echo "==> 3/6: Re-add 'template: feature_request@1' frontmatter to all FRs"
node "$REPO_ROOT/scripts/apply-resolutions.mjs" frontmatter
echo

echo "==> 4/6: Add stub sections (Alternatives Considered, AI Authorship Disclosure, Customer Quotes, Sales/CS Summary)"
node "$REPO_ROOT/scripts/apply-resolutions.mjs" sections
echo

# ─── 4. Update master index docs/tasks/README.md ───────────────────────────
echo "==> 5/6: Update docs/tasks/README.md to document canonical section list"
node "$REPO_ROOT/scripts/apply-resolutions.mjs" readme
echo

# ─── 5. Final audit pass ───────────────────────────────────────────────────
echo "==> 6/6: Audit all FRs for required sections"
node "$REPO_ROOT/scripts/apply-resolutions.mjs" audit
echo

# ─── Summary ───────────────────────────────────────────────────────────────
fr_count=$(find docs/tasks -name 'FR-*.md' -type f 2>/dev/null | wc -l | tr -d ' ')
echo "==> Done."
echo "    FRs in docs/tasks/: $fr_count"
echo "    docs/feature-requests/: removed"
echo "    modules.yaml: updated to 25 modules"
echo
echo "Next:"
echo "    git status"
echo "    git diff modules.yaml"
echo "    git diff docs/tasks/batch-01/FR-AUTH-001-oauth21-webauthn-rbac-rls.md   # spot-check one FR diff"
echo "    git add -A"
echo "    git commit -m 'feat: resolve 5 content conflicts (modules.yaml + FR section canonical-isation)'"
