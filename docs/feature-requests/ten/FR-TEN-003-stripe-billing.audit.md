---
fr_id: FR-TEN-003
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9/10
score_post_revision: 10/10
issues_resolved: 11
template: engineering-spec@1
---

## §1 — Verdict summary

The spec lands the international Stripe billing rail on top of FR-INV-003 (inbound webhook receiver) + FR-TEN-002 (plan tiers) + FR-TEN-004 (metering). Final form: 1,054 lines, 25 §1 normative clauses (covering schema additions, billing_rail derivation, price catalog, lazy Customer creation, subscription lifecycle, plan-change push, overage push, NATS dispatch, dunning state machine, refund flow, 11 memory audit kinds, per-residency Stripe API key routing, founder + VND short-circuit guards, PII scrubbing, OTel tracing), 20 acceptance criteria, 10 verification test snippets (`stripe_currency_lock_test` · `stripe_customer_idempotency_test` · `stripe_dunning_state_machine_test` · `stripe_founder_skip_test` · `stripe_api_retry_test` · `stripe_price_catalog_cardinality_test` · `stripe_refund_cfo_only_test` · `stripe_dispatcher_idempotency_test` · `stripe_overage_push_test` · `stripe_residency_apikey_routing_test`), 20+ failure-mode rows, 25 implementation notes. Spec depth justified by genuine surface complexity: 5 Postgres migrations, 11 audit-row kinds, 60 Stripe Price IDs per residency (12 base + 48 overage axes), per-residency Stripe API key routing, dunning state machine, refund flow, deploy-time price-sync CLI.

The audit identified 11 issues across self-correction prose, missing SQL constructs referenced in clauses, role grant lineage, deferred-DEC placeholders, overage axis Price ID enumeration, CLI per-residency auth flow, and overage_meter_price_id reference in §1 #7. All were resolved in revision before this 10/10 score.

## §2 — Findings (all resolved)

### ISS-001 — Self-correction prose in §1 #11 about dunning history

The first draft of §1 #11 contained a dangling self-correction ("State transitions persisted via a separate same-TX `tenant_dunning_history` table — actually no, dunning history is OBSERVED via…"). This is bad form — readers can't tell which version is the spec. Resolved: §1 #11 final paragraph now states cleanly that dunning state is observed via `stripe_event_dispatch_log` rows joined with `tenants.dunning_state` snapshots + memory chain rows; no separate history table introduced. Reference DEC-808.

### ISS-002 — Self-correction prose in §11.4 about cyberos_pruner column grants

The first draft of §11.4 invented a "column-level GRANT on dispatched_at and failure_reason" pattern that conflicted with the §3.1 `REVOKE UPDATE, DELETE` grant. The "Actually correction:" follow-up resolved it but left both versions in prose. Resolved: §11.4 rewrites as single-pass INSERT-only design with retry-via-correction_to deferred to slice 3.

### ISS-003 — Self-correction prose in §11.20 missing schema column

The first draft of §11.20 said the spec "missed" the `stripe_subscription_items_map JSONB` column with an in-prose "Wait — I missed this" correction. Reader couldn't tell if column was in migration or not. Resolved: §11.20 rewritten as authoritative statement; §3.1 migration `0006_stripe_billing.sql` now includes `ADD COLUMN stripe_subscription_items_map JSONB NOT NULL DEFAULT '{}'::jsonb`.

### ISS-004 — Trigger `billing_currency_immutable` referenced but not defined

AC #1 and the §5.1 test reference a trigger by name (`billing_currency_immutable`), and §1 #4 mandates it, but §3.1 SQL did not include the CREATE FUNCTION + CREATE TRIGGER for it. A reader implementing the spec would have to invent the trigger. Resolved: §3.1 now defines `trg_billing_currency_immutable()` + `tenants_billing_currency_immutable` trigger explicitly. Additionally added the companion `trg_founder_no_stripe()` trigger to enforce DEC-805 (founder) and DEC-784 (VND) at the schema level, closing two failure modes that previously relied on handler-only guards.

### ISS-005 — `cyberos_pruner` role grant lacked lineage reference

The §3.1 `GRANT DELETE ON stripe_api_calls TO cyberos_pruner;` referenced a role that this FR did not introduce. A reviewer would have to grep for where the role is defined. Resolved: §3.1 now carries an inline comment citing `cyberos_pruner` as the existing scheduled-job role from FR-AUTH-003 §3.4, scoped to TTL pruning per DEC-807.

### ISS-006 — §9 carried `DEC-XXX TBD` placeholder

The §9 deferred-items list had one item citing `DEC-XXX TBD` for real-time invoice preview, which violates the feature-request-audit skill guidance that deferred items SHOULD list `Deferred:` prefix + concrete slice/phase reference. Resolved: §9 rewritten — all 9 deferred items now cite slice 3 + concrete FR-TEN-1xx or FR-TEN-2xx target (marked as `placeholder — not yet specified` where appropriate per feature-request-audit skill rule 3).

### ISS-007 — Overage axis Price IDs under-specified in §1 #5

§1 #5 originally said "12-entry price catalog" but §1 #7 references `overage_meter_price_id` per axis (4 per tier per currency) without explaining how the additional 48 (= 4 axes × 12 base) overage Stripe Price IDs are tracked. Resolved: §1 #5 expanded to cover the full 60-entry-per-residency model (12 base + 48 overage); the constant `PRICE_CATALOG` remains 12 entries (base only), with overage entries derived from base via `stripe_price_map.axis` column. CI cardinality test still asserts 12 base; overage prices live exclusively in `stripe_price_map`.

### ISS-008 — `cyberos-ten stripe-sync-prices` CLI auth flow under-specified

§1 #20 said `--apply` invokes Stripe but didn't explain how per-residency auth works. Per DEC-801, the CLI must use the right Stripe account per residency, but a single CLI invocation can't reach all 3 (sg-1 / eu-1 / us-1) without holding all 3 keys simultaneously — a forensic concern. Resolved: §1 #20 now requires `--residency <sg-1|eu-1|us-1>` on `--apply`; CLI loads only that residency's KMS-encrypted Stripe API key; `--dry-run` permits all-residencies for reporting. Cross-residency misuse impossible because loaded key only addresses one Stripe account.

### ISS-009 — `Stripe-Version` pinning policy unclear at slice-2 vs future

§11.1 says "Pin to exact `Stripe-Version` baseline `2024-06-20`" but doesn't specify how the bump policy works. A reader implementing might bump silently. Resolved: §11.14 already states per-call versioning override is not supported in slice 2; §11.1 reinforced that bumping the date is an ADR with regression-test sweep — operationally clear path.

### ISS-010 — Cross-rail guard at trigger level, not just handler

§1 #22 + #23 mandate handler-level guards against Stripe API calls for founder / VND tenants, but a forensic-grade design should fail-closed at the schema level too. Without schema guards, a buggy handler could populate stripe_customer_id on a VND tenant (or worse, the founder), leaving inconsistent state that would only surface at next API call. Resolved: §3.1 now ships `trg_founder_no_stripe()` trigger preventing INSERT/UPDATE that populates stripe_customer_id on `is_founder_tenant=true` OR `billing_currency='VND'` rows. Two failure-mode rows (rows 14 + 15 in §10) note the trigger now enforces what was previously handler-only.

### ISS-011 — `stripe_price_map` table missing partial unique index

§3.1 defined `stripe_price_map` with PRIMARY KEY `(residency, currency, plan_tier, axis)` but no index aiding the common lookup `WHERE residency=$1 AND currency=$2 AND plan_tier=$3` (to fetch base + 4 overage Price IDs in one query for subscription create). Primary key index covers this prefix-wise, so no additional index needed. After re-examination this is NOT an issue — Postgres B-tree on `(residency, currency, plan_tier, axis)` answers the 3-column prefix lookup natively. Marked resolved as a no-op (verified pre-shipping rather than added an index).

## §3 — Resolution

All 11 mechanical concerns addressed. Spec is now coherent (no dangling self-corrections), self-contained (no unresolved references to constructs defined outside the spec), and forensically defensive (triggers enforce DEC-805 + DEC-784 + DEC-798 at the schema level, not just at handler entry).

The 1,054-line length sits just above the feature-request-audit skill §3.14 "above 1,000 lines suggests prose padding" soft cap. Justification: the FR introduces 5 migrations, 11 audit-row kinds, 60 Stripe Price IDs per residency, dunning state machine, refund flow, NATS dispatcher, deploy-time CLI, and 20+ failure modes. Genuine surface complexity, not padding. The substantive density (clauses per line, failure modes per line, test coverage per line) is comparable to FR-TEN-002 (peer FR with similar scope) at 740 lines — TEN-003's extra 300 lines come from the per-axis Stripe Item map, per-residency API routing, and 11 audit-row kinds vs TEN-002's 1.

**Score = 10/10.**

---

*End of FR-TEN-003 audit.*
