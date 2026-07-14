---
name: project-cleanup
description: |
  Generic project hygiene skill. Triggers on "clean up the project",
  "tidy the repo", "remove leftovers", "absorb fragments", "merge small
  docs", "check project state", "verify structure", "audit FRs / backlog
  / changelog", or any request to detect + remove orphan files,
  consolidate small markdown fragments into unified docs, and report
  project-state health. Auto-detects project structure (cyberos FR
  catalog if present; falls back to generic repo cleanup otherwise).
skill_version: 1.0.1
persona: shared
owner_role: _shared
allowed_memory_scopes:
  read: []
  write: []
allowed_mcp_tools:
  - file_read
  - file_write
  - bash
  - file_delete
escalation:
  to_persona_on_legal: null
  to_persona_on_security: null
  to_persona_on_compliance: null
  to_human_on_irreversible: true
expects:
  schema_ref: ./envelopes/input.json
  required_fields: [project_root]
  optional_fields: [scope, dry_run, absorb_threshold_lines]
produces:
  schema_ref: ./envelopes/output.json
  output_kind: report
audit:
  emit_to: memory.action_log
  row_kind: cleanup_report
  payload_hash_field: report_sha256
  explanation_pane: required
confidence_band:
  default: 0.85
  defer_below: 0.7
  cite_sources: required
untrusted_inputs:
  wrap_in_marker: "untrusted_content"
  injection_scan: required
  on_marker_hit: surface_to_human
determinism:
  reproducible: false
  fixity_notes: "Project state varies between runs (new files, new commits). Each invocation produces a fresh inventory + report."
emitted_source_freshness_tier: 90
gated_until_phase: null
---

# project-cleanup — repo hygiene + state audit

A generic project-cleanup skill. Run it when the repo accumulates fragment files, orphan docs, audit-trail debt, or you want a single command that says "is the project still in good shape?"

## What it does

Four phases, executed in order:

1. **Inventory** — scan the project for stale fragments, orphan files, structural drift.
2. **Absorb + merge** — consolidate small markdown fragments (<N lines) into the most-suitable parent doc.
3. **Delete leftovers** — remove anything safely flagged as trash + confirm with the operator before destructive action.
4. **Verify state** — run project-aware checks (cyberos FR DAG coherence, audit-score grep, BACKLOG/CHANGELOG sync) or generic checks (broken links, orphan refs).

## When to use

Trigger this skill when you say any of:

- "Clean up the project"
- "Tidy the repo"
- "Remove leftovers / fragments / orphan files"
- "Absorb / merge small docs"
- "Check project state"
- "Verify project structure"
- "Audit FRs / backlog / changelog"
- "Health check the repo"

## Behaviour (what the skill does)

### Phase 1 — Inventory (read-only, never destructive)

The skill MUST first build an inventory before touching anything. The inventory is a structured map of:

- **Top-level layout** — directories + file counts per dir.
- **Suspicious leftovers** — markdown files matching patterns:
  - `*_SUMMARY.md`, `*_PROGRESS.md`, `*_NOTES.md` outside an explicit notes/ folder
  - `*.md.bak`, `*.old.md`, `tmp_*.md`, `draft_*.md`
  - Files in known-archive paths (`archive/`, `_old/`, `deprecated/`)
- **Orphan audit pairs** — for repos with audit discipline: spec files missing audits, audit files missing specs
- **Stale references** — broken cross-doc links (file deleted but linked from elsewhere)
- **Small fragments** — markdown files under `absorb_threshold_lines` (default: 80) that could merge into a parent

The inventory is reported back as a structured Markdown table BEFORE any mutation. The operator approves what to act on.

### Phase 2 — Absorb + merge (proposes, then acts)

For each candidate small fragment:

1. **Read** the fragment fully.
2. **Auto-detect** the suitable target parent by:
   - Same directory's `README.md` if present
   - Cross-references in the fragment that name another doc
   - Closest-related larger doc in the same module
3. **Propose** a merge plan: `<fragment> → <parent>` with a one-line absorb summary.
4. **Operator approves** (or skips per-fragment) → write absorb section into parent → delete fragment.
5. **Audit row emitted** for each merge.

The absorb output goes under a clearly-marked section in the parent doc (e.g. `## Historical: <fragment topic>`) so reviewers can trace provenance.

### Phase 3 — Delete leftovers (operator confirms)

For each leftover not absorbed:

1. **Show the file** + age + last-modified date + any incoming references.
2. **Recommend** keep / archive / delete based on:
   - Has incoming refs from active docs → KEEP (flag for manual review)
   - In `archive/` already → KEEP (already archived)
   - Backup or temp file with `.bak / .old / tmp_` prefix → DELETE
   - Empty file or just whitespace → DELETE
   - Otherwise → manual review
3. **Operator confirms** the recommendation per file (or accepts all).
4. **Delete** approved files.

The skill MUST escalate to human (`to_human_on_irreversible: true`) before any delete.

### Phase 4 — Verify state (project-aware)

Detect the project flavor and run the appropriate verifier:

**Cyberos flavor detection:** presence of `docs/tasks/BACKLOG.md` AND either `task-audit` skill (post-2026-05-18 layout) or the legacy `task-audit` skill.

If cyberos flavor:
- Run FR DAG coherence (depends_on ↔ blocks reciprocity)
- Audit-score grep (every FR has audit at 10/10)
- Per-module spec/audit count balance (sanity check)
- BACKLOG.md sync state (header version matches latest FR additions)
- CHANGELOG.md last-entry date freshness (warn if stale > 30 days)
- Compare IMPLEMENTATION_ORDER.md FR count vs actual

Generic flavor (no task-audit skill / task-audit skill):
- Broken markdown link check (relative `.md` links pointing to nonexistent files)
- Orphan file detection (markdown files not referenced anywhere)
- Top-level README presence check
- CHANGELOG.md presence check + last-entry date freshness

The verify phase emits a final summary report:
```
Project: <path>
Inventory: N files scanned, X potential fragments, Y suspicious leftovers
Absorbed: M fragments merged into parent docs
Deleted: K leftover files removed
Verify state: <PASS|FAIL> — <details>
Report written to: <path>/CLEANUP_REPORT_<timestamp>.md
```

## What this skill MUST NOT do

- **Never** delete a file without operator confirmation. `to_human_on_irreversible: true`.
- **Never** delete files referenced by active docs (incoming refs preserve = keep).
- **Never** delete files under `.git/`, `node_modules/`, `target/`, `dist/`, build artifacts — these are out of scope.
- **Never** absorb a doc into a parent without explicit operator approval per fragment.
- **Never** modify any file outside the project root (path traversal guard).
- **Never** modify files in tracked-but-not-this-project paths (per `allowed_memory_scopes` discipline — this skill writes only its report).

## Why this skill exists

Repos accumulate cruft. The same patterns keep recurring:

- One-off `*_SUMMARY.md` from earlier audit sessions, useful then, stale now.
- 17-line fragment doc that should be a section in the module README.
- Orphan audit files where the spec was renamed or removed.
- Stale references in BACKLOG.md that don't match the actual spec folder.
- CHANGELOG hasn't been updated in 60 days but the repo had 3 major releases.

This skill makes the cleanup pass repeatable + auditable. Run it monthly (or before any major handoff). Each run produces a `CLEANUP_REPORT_<timestamp>.md` so the operator can review what changed.

## Invocation example

```bash
# Run interactively in current dir
$ cleanup run --project-root .

# Or trigger via natural language to the agent:
> "Clean up the project, absorb small fragments, verify state."
```

## Helper scripts

The skill ships with portable helper scripts under `./scripts/`:

| Script | Purpose |
|---|---|
| `find_fragments.py` | Phase 1: scan for stale fragments + orphan files |
| `propose_absorbs.py` | Phase 2: auto-detect merge targets + propose plan |
| `coherence_check.py` | Phase 4 (cyberos): FR DAG reciprocity + audit-score check |
| `gen_module_readmes.py` | Phase 4 (cyberos): regenerate per-module READMEs |
| `gen_impl_artifacts.py` | Phase 4 (cyberos): regenerate IMPLEMENTATION_ORDER + SPRINT_PLAN |
| `generic_verify.py` | Phase 4 (generic): broken link + orphan ref check |

Scripts are invoked by the skill body, not directly. They're versioned with the skill (`skill_version` bumps when scripts change).

## Acceptance harness

See `./acceptance/` for golden input/output pairs:

- `golden-cyberos-cleanup-input.json` — cyberos repo state with 4 fragments + 2 orphans
- `golden-cyberos-cleanup-output.md` — expected cleanup report
- `golden-generic-cleanup-input.json` — vanilla repo with 1 stale README
- `golden-generic-cleanup-output.md` — expected generic cleanup report

Determinism: the skill is NOT byte-deterministic (project state varies). Acceptance uses structural diff: "did we identify the same fragments + propose the same absorbs?"

## Output envelope

```json
{
  "project_root": "/path/to/project",
  "scope": "cyberos | generic",
  "phase_1_inventory": {
    "files_scanned": 1234,
    "fragments_detected": 4,
    "suspicious_leftovers": 2,
    "orphan_audits": 0
  },
  "phase_2_absorbs": [
    {"fragment": "ai/SLICE_1_AUDIT_SUMMARY.md", "merged_into": "ai/README.md", "lines_absorbed": 132}
  ],
  "phase_3_deletes": [
    {"path": "ai/SLICE_1_AUDIT_SUMMARY.md", "reason": "absorbed-then-deleted"}
  ],
  "phase_4_verify": {
    "coherence_errors": 0,
    "missing_audits": 0,
    "stale_references": 0,
    "overall": "PASS"
  },
  "report_path": "<project_root>/CLEANUP_REPORT_2026-05-17T14-32-00Z.md",
  "report_sha256": "abc123..."
}
```
