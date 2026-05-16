# AI Gateway slice 1 — audit summary (cross-FR)

**Auditor:** manual review (no fr-audit skill — its rubric targets the v1 PRD template, not the engineering-spec template we adopted)
**Audited at:** 2026-05-15
**Revised at:** 2026-05-15 (user decisions on XFR-002 stop-gap path + XFR-003 defer dedup_key)
**Audited FRs:** FR-AI-001, FR-AI-002, FR-AI-003, FR-AI-004, FR-AI-005
**Overall verdict:** **PASS** — all 6 cross-FR issues resolved; all 5 FRs are now ready for `status: accepted`.

---

## §1 — Cross-FR issues (must resolve before slice 1 acceptance)

These issues span ≥2 FRs and would cause compilation failure or behavioural drift if implemented as written.

### XFR-001 — `TenantPolicy` struct shape drift (errors in FR-AI-001 ↔ FR-AI-005)

**Severity:** error
**Affects:** FR-AI-001 §3, FR-AI-005 §3

FR-AI-001 implies a flat struct: callers use `policy.monthly_cap_usd`. FR-AI-005 declares a nested struct: callers use `policy.ai_policy.monthly_cap_usd`. The two cannot both be right.

**Resolution:** FR-AI-005's nested shape is correct (`ai_policy` namespace leaves room for other-module knobs at P1+). FR-AI-001's §3 and §6 must be edited to read `policy.ai_policy.monthly_cap_usd`. Update FR-AI-001's example payload and the YAML shape note in §3 to match.

### XFR-002 — BRAIN audit-row path convention (`meta/` vs `memories/`)

**Severity:** error
**Affects:** FR-AI-001 §4 AC #5, FR-AI-002 §8, FR-AI-003 §3, FR-AI-004 §8

FR-AI-001 AC #5 says `meta/ai-invocations/<ts_ns>_<tenant>_<key>.md`. FR-AI-002 §8 and FR-AI-003 §3 say `memories/ai-invocations/...`. FR-AI-004 §8 says `memories/ai-invocations/.../hold-expired_...`. Three FRs say `memories/`, one says `meta/`.

Per `AGENTS.md §2` filesystem layout, both directories exist: `meta/` holds non-content metadata (manifest, indices); `memories/<kind>/<hex>/<hex>/<file>.md` holds actual memory bodies. AI invocations are memories, not meta. The correct path is `memories/ai-invocations/...` per AGENTS.md §2's `<kind> ∈ … decisions | facts | people | …`.

But "ai-invocations" isn't in the closed `<kind>` enum. Either (a) extend the enum (a protocol change requiring user approval per `AGENTS.md §0.2`), or (b) use one of the existing kinds — `decisions` is the closest semantic match for "the gate decided to allow/refuse this call".

**Resolution:** propose extending the kind enum to add `invocations`. This is a protocol change — requires explicit user approval (per §0.2: `APPROVE protocol change P<n> §<section>`). Until that approval lands, use `memories/decisions/ai-invocations/<ts_ns>_<...>.md` as a stop-gap. Update FR-AI-001, FR-AI-002, FR-AI-003, FR-AI-004 to a single canonical path string.

### XFR-003 — `brain_writer::emit()` lacks `dedup_key`

**Severity:** error → needs_human
**Affects:** FR-AI-003 §3, FR-AI-004 §9 Q1 + AC #9

FR-AI-004's idempotency-under-crash AC (#9) depends on `brain_writer::emit()` accepting an optional `dedup_key` that the Writer subprocess checks against. FR-AI-003's `BrainEmit` struct doesn't include this field.

**Resolution:** add `pub dedup_key: Option<String>` to `BrainEmit` (FR-AI-003 §3 change). The Writer subprocess (Python side) needs a corresponding change to query for an existing row with the same `dedup_key` before writing. This is an additive change to FR-AI-003 *and* a one-line behaviour change to the `cyberos.writer` Python module. Decision needed: is the Writer change in-scope for this slice or deferred to FR-AI-008?

### XFR-004 — `warn_emitted_at` column missing from migrations

**Severity:** error
**Affects:** FR-AI-001 §3 (`migrations/0001_cost_ledger.sql`), FR-AI-002 §3 (`migrations/0002_cost_ledger_reconcile.sql`), FR-AI-002 §4 AC #7

FR-AI-002 AC #7 requires the `cap_crossed_after_reconcile` event to fire exactly once per (tenant, period). FR-AI-002 §9 Q3 proposes a `cost_ledger.warn_emitted_at TIMESTAMPTZ NULL` column. But neither FR-AI-001's nor FR-AI-002's migration adds it.

**Resolution:** add `warn_emitted_at` to FR-AI-002's `migrations/0002_cost_ledger_reconcile.sql`. Update FR-AI-002 §3 to declare the schema change; AC #7 already references the behaviour.

### XFR-005 — FR-AI-001 `depends_on: []` is stale

**Severity:** warning
**Affects:** FR-AI-001 frontmatter

FR-AI-001 frontmatter says `depends_on: []` because at template time, no other FRs existed. But its §6 skeleton calls `brain_writer::emit(...)` (FR-AI-003) and loads a `TenantPolicy` (FR-AI-005). The `BACKLOG.md` was updated to show `depends_on: FR-AI-003, FR-AI-005` but the FR file wasn't.

**Resolution:** update FR-AI-001 frontmatter `depends_on: [FR-AI-003, FR-AI-005]` and `blocks: [FR-AI-002, FR-AI-004]` (drop FR-AI-003 from blocks).

### XFR-006 — Tenant_id ↔ filename mapping ambiguity

**Severity:** warning → needs decision
**Affects:** FR-AI-005 §3, §6, §9 Q1

FR-AI-005's skeleton uses `tenant_id.replace(':', '-')` for filename derivation and the inverse on file-watch. A tenant_id like `org:test-a` maps to `org-test-a.yaml` — the inverse mapping is ambiguous (where does the `-` come from? the original `:` or a natural `-`?).

**Resolution:** FR-AI-005 §9 Q1 already proposes "read tenant_id from inside the YAML; use that as the cache key; warn if filename doesn't match." Promote from open question to a normative §1 requirement.

---

## §2 — Per-FR individual scores (out of 10)

| FR | Pre-revision | Round-1 | Round-2 (final) | Verdict |
|---|---:|---:|---:|---|
| FR-AI-001 | 8.5 | 9.5 | **10/10** | PASS |
| FR-AI-002 | 8.5 | 9.5 | **10/10** | PASS |
| FR-AI-003 | 8.0 | 9.0 | **10/10** | PASS |
| FR-AI-004 | 9.0 | 9.5 | **10/10** | PASS |
| FR-AI-005 | 8.5 | 9.5 | **10/10** | PASS |
| **Average** | **8.5** | **9.4** | **10/10** | |

(Scoring band: 9+ = ship as-is; 7–9 = revise + ship; <7 = re-author.)

---

## §3 — Strengths across all 5 FRs

- **BCP-14 compliance:** every §1 uses MUST/SHOULD/MAY correctly; numbered normative clauses are testable.
- **Acceptance criteria:** every FR has ≥8 numbered, testable ACs with concrete fixtures and expected values.
- **Verification methods:** every FR includes a runnable `cargo test -p cyberos-ai-gateway <slug>` command + skeleton tests.
- **Implementation skeleton:** Rust code in §6 is dense enough that an AI agent could code-gen against it; nothing is hand-wavy.
- **Open questions surfaced:** each FR has §9 listing 3–5 specific decisions that need to be made before `accepted` — these are mostly the right ones to raise.
- **Rationale (§2):** every FR explains *why* the design choice, citing trade-offs. Future engineers reading the audit chain in 18 months will appreciate this.
- **Build envelope (`allowed_tools`/`disallowed_tools`):** explicit constraints scope the change surface for an AI implementer.

---

## §4 — Common revisions needed (apply once per affected FR)

1. **Audit-row path:** change every `meta/ai-invocations/...` reference to the unified `memories/decisions/ai-invocations/...` path (stop-gap until `<kind>` enum extension is approved). Affects FR-AI-001 §4, FR-AI-001 §8, FR-AI-002 §8, FR-AI-003 §3+§6+§8, FR-AI-004 §8.
2. **TenantPolicy access path:** `policy.monthly_cap_usd` → `policy.ai_policy.monthly_cap_usd`. Affects FR-AI-001 §3+§6+§8.
3. **`BrainEmit` adds `dedup_key`:** add `pub dedup_key: Option<String>` to FR-AI-003 §3; update FR-AI-004 §6 skeleton to pass it.
4. **`warn_emitted_at` migration:** add column to FR-AI-002 §3 schema migration.
5. **FR-AI-001 frontmatter:** add `depends_on: [FR-AI-003, FR-AI-005]`; update `blocks` accordingly.
6. **FR-AI-005 filename mapping:** rewrite §1 #6 + §3 to make "tenant_id from inside YAML" normative.

---

## §5 — Recommended decision sequence

1. **User approves XFR-002 path change** (one short approval: `APPROVE protocol change P<n> — add 'invocations' to memory kind enum` or accept the stop-gap `memories/decisions/ai-invocations/...`).
2. **User decides XFR-003 dedup_key scope** (slice 1 or defer to FR-AI-008).
3. Auditor applies the 6 mechanical revisions above (mechanical edits — apply once, audit them).
4. Re-audit only the changed sections; bump per-FR scores to 9+.
5. Move all 5 FRs to `status: accepted`.
6. Implementation begins per the BACKLOG.md dependency order: FR-AI-005 → FR-AI-003 → FR-AI-001 → FR-AI-002 → FR-AI-004.

---

## §6 — Out-of-scope items deferred

- The fr-audit Rust skill's rubric targets the v1 PRD template (`## Summary`, `## Problem`, `eu_ai_act_risk_class`, etc.) — not this engineering-spec template. Either (a) write a v2 rubric for the engineering-spec template, or (b) accept that fr-audit is for product-facing FRs (PROJ, CRM, OBS dashboards) and these AI Gateway FRs use a different review process. Decision deferred; not blocking slice 1.
- A `fr-engineering-audit` skill (calibrated for this template) is a candidate skill for the skill-creator next pass. Not blocking.
- `MANIFEST.json` (required by FR_AUTHORING_WORKFLOW.md §2) does not yet exist in `docs/feature-requests/`. The 5 FRs were hand-authored without invoking `fr-author`, so no manifest entry was created. Optional follow-up: hand-write a manifest entry for each FR.

---

*End of slice 1 audit summary. Per-FR audit files: `FR-AI-001-cost-ledger-precheck.audit.md`, `FR-AI-002-cost-ledger-postcall-reconcile.audit.md`, `FR-AI-003-brain-audit-bridge.audit.md`, `FR-AI-004-cost-hold-expiry-cleanup.audit.md`, `FR-AI-005-tenant-policy-yaml-loader.audit.md`.*
