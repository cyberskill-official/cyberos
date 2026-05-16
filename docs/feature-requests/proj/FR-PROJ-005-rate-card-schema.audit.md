---
fr_id: FR-PROJ-005
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 16
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; AUTHORING.md §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (no-line-cap expansion per AUTHORING.md §0; ISS-007..016 added)
---

## §1 — Verdict summary

FR-PROJ-005 expanded 2026-05-16 P.M. via strict-redo no-line-cap pass. Current scope: 22 §1 clauses (schema, role enum, currency enum, append-only, partial-unique, lookup_at, REST contract, audit kinds, RLS, non-negative, gauge metric, CSV, preview endpoint, retroactive corrections, default rate-card pack, role aliasing, 365-day-future cap, archived_at separate from supersede, rate-cards lock, currency-mismatch warning, include_archived flag, IP/UA in audit). 16 §2 rationale paragraphs. §3 contains: migration + Currency::decimals + RateCard struct + handlers including create_or_supersede with FOR UPDATE locking + lookup_at half-open interval query. 32 ACs. §5 contains 5 e2e tests + parameterised case coverage. §10 lists 38 failure rows. §11 lists 26 implementation notes covering BIGINT vs pg_numeric choice, partial-unique pattern, closed-open interval rationale, Currency::decimals usage in FR-PROJ-007, idempotency reuse pattern, FOR UPDATE scope, preview-as-handler-branch consistency, deletion-vs-archive policy, gauge computation.

## §2 — Findings (all resolved during authoring)

### ISS-001 — Versioning via UPDATE vs supersession
UPDATE retroactively reprices old work. Resolved: §1 #4 + DEC-260 append-only; supersession via INSERT + close prior; AC #2.

### ISS-002 — Currency scope (per-engagement vs per-row)
Per-engagement collapses multi-currency cases. Resolved: §1 #3 + DEC-261 per-row; AC #14 multi-currency same engagement.

### ISS-003 — Money type (float vs decimal vs minor units)
Floats unsafe. Resolved: §1 #1 + §11 BIGINT minor units; `Currency::decimals()` helper.

### ISS-004 — Partial mutability
Without spec, all fields look mutable. Resolved: §1 #7 only `billable_default` patchable; AC #7 #8.

### ISS-005 — Concurrent supersede race
Two callers each read "no prior" and both insert. Resolved: §3 `FOR UPDATE` lock + AC #1 implicit.

### ISS-006 — Effective interval semantics (open vs closed)
Half-open vs closed-open at boundary date is ambiguous. Resolved: §11 closed-open (`effective_to > at`); §1 #6 lookup query.

### ISS-007 — No preview before commit (strict-redo pass)
Rate changes affect downstream billing for hundreds of timesheets; operators want preview before commit. Resolved: §1 #13 + /preview endpoint + AC #18 + §11 note that preview = commit-with-commit=false branch (guaranteed consistency).

### ISS-008 — No retroactive correction path (strict-redo pass)
Real-world: rate misconfigured at engagement start, discovered weeks later after billing. Mutation forbidden by §1 #4; correction needs a separate audited path. Resolved: §1 #14 + admin-only + correction_reason + `proj.rate_card_corrected` audit + AC #19 + #20.

### ISS-009 — Default pack at engagement creation missing (strict-redo pass)
Tenants creating engagements with standard role/currency rates were re-entering data per engagement. Resolved: §1 #15 + tenant default_rate_card_pack JSONB + copy-at-creation + AC #21.

### ISS-010 — Role aliasing for migration absent (strict-redo pass)
Tenants migrating from other tools have legacy role names; rejecting them blocks adoption. Resolved: §1 #16 + role-aliases table + handler-layer resolution + AC #22.

### ISS-011 — No sanity-check on far-future effective_from (strict-redo pass)
A typo like `effective_from=2030-01-01` (operator meant 2026) would silently install a future rate. Resolved: §1 #17 + 365-day cap + AC #23.

### ISS-012 — Archive vs supersede conflation (strict-redo pass)
Engagement archival should distinguish from rate supersession; without separate field, queries couldn't tell "this rate was replaced" vs "engagement done." Resolved: §1 #18 + archived_at + AC #24 + #25.

### ISS-013 — No lock for compliance-controlled rate changes (strict-redo pass)
Enterprise contracts often require legal sign-off for rate changes; without a lock, ops could change unilaterally. Resolved: §1 #19 + rate_cards_locked flag + AC #26.

### ISS-014 — Currency mismatch invisibility (strict-redo pass)
USD rate on VND-default engagement is probably intentional but worth surfacing. Resolved: §1 #20 + SEV-3 warning + AC #27.

### ISS-015 — History always excludes archived (strict-redo pass)
Forensic queries need to include archived cards for full reconstruction. Resolved: §1 #21 + ?include_archived=true + AC #28.

### ISS-016 — No forensic trail on rate-change actor (strict-redo pass)
created_by_subject_id was insufficient for fraud investigation; need IP + UA. Resolved: §1 #22 + audit-only IP/UA + AC #29 + §11 hot-path rationale.

## §3 — Resolution

All 16 mechanical concerns addressed. **Score = 10/10.** Ready to ship + transition `draft → accepted`.

Per AUTHORING.md §0 master rule: spec is now perfect — depth bounded by the genuine surface (versioning × multi-currency × supersede vs archive × correction workflow × preview/lock/alias/default-pack policy × forensic trail), not by line targets.

---

*End of FR-PROJ-005 audit.*
