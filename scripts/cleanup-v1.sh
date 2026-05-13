#!/usr/bin/env bash
#
# scripts/cleanup-v1.sh — remove every v1 artefact from the repo.
#
# Run AFTER you've confirmed v2 works for your workflow. The deletion is
# irreversible from this script's point of view; rely on git history for
# rollback (every removed file has been committed at least once).
#
# Defaults to --dry-run so the destructive command is always intentional.
#
# Usage:
#     ./scripts/cleanup-v1.sh              # dry-run (default)
#     ./scripts/cleanup-v1.sh --apply      # actually delete

set -euo pipefail

cd "$(dirname "$0")/.."

DRY_RUN=1
if [[ "${1:-}" == "--apply" ]]; then
    DRY_RUN=0
fi

say() {
    if [[ "$DRY_RUN" == "1" ]]; then
        echo "  [dry-run] would: $*"
    else
        echo "  $*"
        eval "$@"
    fi
}

# Remove a file: prefer `git rm` so the deletion is staged; fall back to
# plain `rm` for files not tracked by git (e.g. created in this session
# but not yet `git add`-ed). Either way the file ends up gone.
git_or_rm() {
    local path="$1"
    if [[ "$DRY_RUN" == "1" ]]; then
        echo "  [dry-run] would: git rm \"$path\" (or rm -f if untracked)"
        return 0
    fi
    if git ls-files --error-unmatch "$path" >/dev/null 2>&1; then
        echo "  git rm \"$path\""
        git rm -f -- "$path" >/dev/null
    else
        echo "  rm -f \"$path\"   (untracked)"
        rm -f -- "$path"
    fi
}

echo "=== v1 → v2 cleanup ==="
if [[ "$DRY_RUN" == "1" ]]; then
    echo "    (dry-run; re-run with --apply to actually delete)"
fi
echo

# ----------------------------------------------------------------------
# 1. Frozen v1 documents
# ----------------------------------------------------------------------
echo "v1 frozen documents:"
for f in docs/memory/AGENTS.v1.md docs/memory/README.v1.md; do
    if [[ -f "$f" ]]; then
        git_or_rm "$f"
    fi
done
echo

# ----------------------------------------------------------------------
# 2. The legacy v1 writer + its compatibility shim
# ----------------------------------------------------------------------
echo "v1 writer + compatibility shim:"
for f in runtime/lib/brain_writer.py runtime/lib/brain_writer_shim.py; do
    if [[ -f "$f" ]]; then
        git_or_rm "$f"
    fi
done
echo

# ----------------------------------------------------------------------
# 3. One-shot migration scripts (their job is done; v2 stores don't
#    need them).
# ----------------------------------------------------------------------
echo "migration scripts:"
for f in \
    runtime/tools/cyberos_migrate.py \
    runtime/tools/cyberos_migrate_v2.py \
    runtime/tools/cyberos_migrate_sidecar.py \
; do
    if [[ -f "$f" ]]; then
        git_or_rm "$f"
    fi
done
echo

# ----------------------------------------------------------------------
# 4. Legacy CLI wrapper + the Group A scripts that exist only to be
#    routed via it. Replaced by `python -m cyberos`.
# ----------------------------------------------------------------------
echo "legacy bash CLI wrapper + Group A scripts:"
for f in \
    runtime/tools/cyberos \
    runtime/tools/cyberos_doctor.py \
    runtime/tools/cyberos_export.py \
    runtime/tools/cyberos_validate.py \
    runtime/tools/cyberos_lock.py \
    runtime/tools/cyberos_compact_stats.py \
    runtime/tools/cyberos_index.py \
    runtime/tools/cyberos_show.py \
    runtime/tools/canonical_sha.py \
; do
    if [[ -f "$f" ]]; then
        git_or_rm "$f"
    fi
done
echo

# ----------------------------------------------------------------------
# 5. Stage-1 deprecation stubs from the previous cleanup pass.
# ----------------------------------------------------------------------
echo "deprecation stubs:"
for f in \
    runtime/tools/cyberos_lazy.py \
    runtime/tools/cyberos_index_hook.py \
; do
    if [[ -f "$f" ]]; then
        git_or_rm "$f"
    fi
done
echo

# ----------------------------------------------------------------------
# 6. v1 debris under .cyberos-memory/ (if present).
#    Use git rm -rf only on tracked content; ignore if untracked.
# ----------------------------------------------------------------------
echo "v1 store debris under .cyberos-memory/:"
for d in \
    .cyberos-memory/staging \
    .cyberos-memory/cache \
    .cyberos-memory/.branches \
    .cyberos-memory/__pycache__ \
    .cyberos-memory/refinements \
    .cyberos-memory/tours \
    .cyberos-memory/drafts \
    .cyberos-memory/imports \
    .cyberos-memory/tests \
; do
    if [[ -d "$d" ]]; then
        say "rm -rf \"$d\""
    fi
done
for f in \
    .cyberos-memory/.lock.exclusive \
    .cyberos-memory/.lock.shared \
    .cyberos-memory/.brain_writer.py \
    .cyberos-memory/.DS_Store \
; do
    if [[ -f "$f" ]]; then
        say "rm -f \"$f\""
    fi
done
echo

# ----------------------------------------------------------------------
# 7. After cleanup, regenerate schema (the v1 ops that disappeared
#    should drop from the enum — but P1's v2 op set keeps `view`/`create`/
#    `str_replace`/`insert`/`rename` because audit rows in the existing
#    chain still use those names).
# ----------------------------------------------------------------------
if [[ "$DRY_RUN" == "0" ]]; then
    echo "regenerating memory.schema.json (msgspec → JSON Schema):"
    python -m runtime.tools.cyberos_generate_schema \
        --out docs/memory/memory.schema.json
    echo
    echo "running cyberos doctor on real BRAIN:"
    python -m cyberos --store .cyberos-memory doctor || true
    echo
    echo "next step:"
    echo "    git status     # review the staged deletions"
    echo "    git commit -m 'Retire v1: writer, shim, migrations, legacy CLI'"
fi
