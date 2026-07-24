---
batch: batch/10-imp-draft-groom
recorded: 2026-07-25
actor: Stephen Cheng (session operator)
---

# batch/10 — groom ~71 older draft IMP-* tasks

**Operator session override (continuum):** auto-approve/accept to `done` with recorded evidence; pause parent only for real decisions.

## Inventory (post IMP-143/144 on main)

| Bucket | Count | IDs |
|---|---|---|
| draft stubs (001–067 minus 028) | 66 | TASK-IMP-001..027, 029..067 |
| authored drafts | 5 | TASK-IMP-124, 125, 127, 128, 129 |
| on_hold (untouched) | 1 | TASK-IMP-122 |
| already closed | 1 | TASK-IMP-028 → duplicate_of TASK-IMP-110 |

## Classification

### ship now (authored ACs, implement this wave)

| ID | Reason |
|---|---|
| TASK-IMP-125 | Small docs/test fix: `mechanical` = deterministic behaviour |
| TASK-IMP-127 | Payload build must read git (rules_sha reproducibility) |
| TASK-IMP-128 | Run `scripts/tests/run_all.sh` on ubuntu CI |
| TASK-IMP-129 | Uninstall preserves `.cyberos/config.yaml` (+ shell autodetect already landed) |
| TASK-IMP-124 | TRACE-007 authorship derivation — ship if throughput allows; else next PR |

### close / duplicate (cleared this batch)

| ID | Action | Reason |
|---|---|---|
| TASK-IMP-014 | `duplicate` → TASK-MEMORY-243 | Same intent: external chain anchoring |
| TASK-IMP-015 | `duplicate` → TASK-MEMORY-243 | Same intent: nightly chain integrity (folded into 243) |
| TASK-IMP-024 | `duplicate` → TASK-MEMORY-247 | Dream proposal ranking → BRAIN dream loop |
| TASK-IMP-025 | `duplicate` → TASK-MEMORY-247 | Dream budget/drift gates → same dream loop |
| TASK-IMP-034 | `duplicate` → TASK-CHAT-238 | Chat realtime fanout seam |
| TASK-IMP-060 | `closed` | CONTINUE-HERE retired; status hub / BACKLOG replace it |
| TASK-IMP-023 | `closed` | Meta-groom task; this batch *is* the groom |

### defer — operator decision required (left `draft`; do not park without call)

These still need product/infra decisions or full AC authoring before implement. **Pause list:**

| Theme | IDs | Ask |
|---|---|---|
| Supply-chain / CI security | 001, 003, 043, 044 | Want cargo-audit/deny + secret scan + SBOM now, or after 1.6? |
| Prod boot hardening | 002, 042, 045 | CORS refuse / rate limits / session validation priority? |
| Ops / observability | 004, 005, 006, 016, 017, 018, 019, 049, 050 | P0 deploy of obs stack + staging + SLOs — schedule? |
| Eval / LLM quality | 008, 009, 010, 020, 021 | Goldenset-as-gate vs awh-as-built; ledger completeness; outcome scoring |
| Gate tooling | 011, 012, 013, 022, 026, 040 | Taxonomy / coverage ratchet / contract tests / assert ban / auto-revert / mutation |
| Dream / auto-evolution | 027, 029, 030 | Auto mode envelope; paired-trajectory; QLoRA — research or won't-do? |
| Platform refactors | 031, 032, 035, 036, 037 | error crate / service-kit / unwrap burn-down / audit-chain / OpenAPI |
| Cloud AI | 033 | Explicitly deferred in 2026-07-02 plan — confirm still won't-do for 1.x |
| Security / data | 038, 041, 046, 047, 051, 052, 053, 054 | RLS probe / secrets inventory / backup / rebuild-60 / DB roles / migrations / pgvector / retention |
| Product / frontend | 007, 057 | apps/web test spine; fetch consolidation |
| Docs / governance | 055, 056, 058, 059, 061, 062 | module manifest / API versioning / ADR backfill / wiki links / BRAIN consent / quarterly ritual |
| Go-live tracks | 063, 064, 065, 066, 067 | Track A/B/C + readiness gate — product call |
| Perf / soak | 039, 048 | load/soak; cargo-chef caching |

### leave on_hold

| ID | Reason |
|---|---|
| TASK-IMP-122 | Operator: leave on_hold unless clear unblock; no unblock found (complementary to 127, not subsumed) |

## Decision pauses for operator

1. **Defer list above** — which themes to author next vs close as won't-do for 1.x?
2. **IMP-124** — include in this wave or hold for a dedicated rubric PR?
3. **Mass `on_hold` for remaining stubs?** — would clear draft count without killing intent; needs explicit OK.

## What to merge next

1. This groom PR (closes/duplicates + report)
2. IMP-125 PR
3. IMP-127/128/129 PR (install/CI trio from same PLAN gate)
4. IMP-124 when ready
