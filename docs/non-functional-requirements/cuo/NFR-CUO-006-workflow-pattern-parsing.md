---
id: NFR-CUO-006
title: "CUO workflow-pattern parsing — YAML safe-load + closed enum + clean error reporting"
module: CUO
category: maintainability
priority: MUST
verification: T
phase: P0
slo: "100% of workflow frontmatter parses; failed parses produce {file,line,reason} error"
owner: CTO
created: 2026-05-18
related_frs: [FR-CUO-101, FR-CUO-106]
---

## §1 — Statement (BCP-14 normative)

1. Workflow YAML frontmatter **MUST** parse with `yaml.safe_load()` (no full YAML loader, no arbitrary tag execution).
2. The `pattern:` field value **MUST** belong to a closed enum: `{linear, time-critical, per-instance, multi-output, sequential-approval, persona-pair}`. Unknown values cause catalog scan to fail.
3. The `skill_chain:` field **MUST** be a list of strings; each string **MUST** match an existing skill in the SKILL catalog at scan time.
4. Parse failures **MUST** produce a structured error: `{file: <path>, line: <line_no>, reason: <human-readable>}`. The supervisor never silently swallows YAML errors.
5. Frontmatter fields that look like YAML lists/maps but are intended as opaque strings (e.g., `format: "list[adr@1]"`) **MUST** be quoted by the workflow author — the parser will refuse unquoted ambiguous strings (per the lesson from `cto/threat-model-refresh.md` smoke fix).

## §2 — Why this constraint

YAML is famously footgun-prone (the "Norway problem", arbitrary code execution via untrusted tags). `safe_load` + closed enums + skill-existence checks transform the workflow markdown set from "loosely structured prose" into "machine-validated catalog data." The structured error format makes debugging fast — operator sees the exact file+line, fixes it, retries. The quoting discipline lesson is concrete: it was learned during Phase 1 smoke testing; codifying it here prevents reintroduction.

## §3 — Measurement

- Counter `cuo_workflow_parse_error_total{file, reason}`.
- CI metric: `cuo_unknown_pattern_count` — must be 0.
- CI metric: `cuo_dangling_skill_chain_ref_count` — must be 0 (no skill in chain that's missing from catalog).

## §4 — Verification

- Unit test `modules/cuo/tests/test_workflow_parse.py` (T) — fixtures for valid + 8 invalid workflows; assert correct error reporting.
- CI gate — full-catalog scan must succeed on every commit.
- Property test (T) — random YAML inputs; assert no untrusted-code-execution path is reachable.

## §5 — Failure handling

- Parse error on a workflow → CI blocks merge until fixed.
- Unknown pattern enum value → CI blocks; author must add the pattern via PR through dispatch-table extension.
- Dangling skill_chain ref → CI blocks; skill must exist in catalog or be added.

---

*End of NFR-CUO-006.*
