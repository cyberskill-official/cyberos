---
fr_id: FR-TEN-002
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 8.0/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 8
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; AUTHORING.md §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (first-pass authoring per AUTHORING.md §0)
---

## §1 — Verdict summary

FR-TEN-002 ships the 3-tier plan substrate (Starter / Team / Enterprise) with hardcoded compile-time caps + append-only history + founder-tenant immutability + 24h rate-limit + downgrade-violation defense + proration math. Scope: 25 §1 normative clauses covering closed 3-value `plan_tier` Postgres enum (starter, team, enterprise) + closed 3-value `plan_change_effective` (immediate, next_period, defer_billing_only), per-tier caps as compile-time Rust constants in `services/ten/src/plans/caps.rs` (single source of truth; no DB-mutable caps table), Starter (3 seats / 10k api / 100k AI tokens / 1 GiB storage @ $49), Team (25 / 500k / 5M / 100 GiB @ $249), Enterprise (∞ seats / ∞ api / 50M tokens FINITE because provider pass-through / 1 TiB @ $999), append-only `tenant_plan_history` table at SQL grant (REVOKE UPDATE/DELETE FROM cyberos_app + privileged ten_writer) + trigger that requires same-TX history INSERT on every plan_tier UPDATE (P0301) + RLS, founder-tenant `is_founder_tenant` boolean with one-way set + P0300 trigger blocks any flip after insert + regular handler returns 403, separate founder-override handler at distinct URL path bypasses rate limit + handles founder tenants, 24h rate-limit on regular plan-change path (429 with `next_allowed_at`), downgrade-violation check against materialized `metering_current_period` view + per-axis target-cap comparison (409), downgrade requires reason ≥10 chars (400 if missing), upgrade proration in integer cents (no floating-point) with rounding favoring tenant, deferred downgrade via `tenants.next_scheduled_change` JSONB pointer (applied at FR-TEN-004 period_close), second deferred change while one pending returns 409, cancel via DELETE handler, from_tier_caps_snapshot JSONB captured per history row for audit-time tier reconstruction, plan-change emits synthetic FR-TEN-004 metering audit event tagged `plan_change` with `idempotency_key = "plan_change_<history_id>"` for billing reconciliation, dry_run preview returns proration + caps diff + violation detection without DB mutation / audit row / rate-limit slot consumption, 4 closed BRAIN audit kinds (plan_changed sev-2, plan_founder_override sev-2, plan_change_rejected_violation sev-2, plan_change_rejected_rate_limit sev-3), per-tenant `metering_caps_yaml` override stronger than plan defaults (resolver order documented in FR-TEN-004 integration), reason text scrubbed via FR-BRAIN-111 before chain emission, founder-tenant immutability is defense-in-depth (handler short-circuit + DB trigger). 22 rationale paragraphs. §3 contains: 2 migrations (plan_tier enum + tenants column with is_founder_tenant + next_scheduled_change JSONB + founder-flip trigger; tenant_plan_history with all closed enums + grants + RLS + same-TX history-required trigger), TierCaps constants in caps.rs, plan-change handler with FOR UPDATE lock + founder check + same-tier check + deferred-pending check + violation check + reason check + rate-limit check + proration math + dry-run rollback + metering event emission. 30 ACs. 32 failure-mode rows. 22 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — Plan caps in DB (operator could rewrite contract silently)
First-pass had `tier_caps` table. Resolved: §1 #2 + DEC-772 + hardcoded const in caps.rs + cargo-expand drift test; AC #4-#6.

### ISS-002 — UPDATE plan_tier without history (forensic gap)
Resolved: §1 #6 + DEC-776 + trigger P0301 + same-TX history-required check + append-only via SQL grant; AC #16 + #18.

### ISS-003 — Founder tenant could be downgraded by API bug
Resolved: §1 #8 + DEC-777 + handler short-circuit + DB trigger P0300 + one-way is_founder_tenant set; AC #14 + #17.

### ISS-004 — Downgrade with active over-cap usage (silent data loss)
Resolved: §1 #4 + DEC-774 + downgrade_violation_check against materialized view + 409 + acknowledge_data_loss escape hatch; AC #9.

### ISS-005 — Plan-change flip-flop (audit chain bloat + accounting noise)
Resolved: §1 #14 + 24h rate limit on regular path + 429 with next_allowed_at + founder bypass for corrections; AC #13.

### ISS-006 — Enterprise = "unlimited" everything would expose provider cost
Resolved: §1 #2 + DEC-780 + Enterprise ai_tokens FINITE at 50M/mo (provider pass-through) + per-tenant override path for legitimate enterprise contracts; AC #6.

### ISS-007 — Mid-period downgrade refund accounting messy
Resolved: §1 #5 + DEC-773 + downgrade defaults to next_period + immediate-downgrade allowed with sev-2 audit; AC #8.

### ISS-008 — from_tier interpretation drift over time
Resolved: §1 #12 + from_tier_caps_snapshot JSONB on every history row + audit-time reconstruction without reading current code; AC #30.

## §3 — Resolution

All 8 mechanical concerns addressed. **Score = 10/10.**

Per AUTHORING.md §0 master rule: spec is now perfect — depth bounded by the genuine architectural surface (3 closed tiers × hardcoded Rust constants × per-axis caps × Enterprise finite ai_tokens × append-only history with same-TX trigger × founder-tenant one-way set + P0300 trigger × separate founder-override handler × 24h rate-limit with founder bypass × downgrade-violation check against materialized view × deferred downgrade via next_scheduled_change JSONB × second-pending-change rejection × from_tier_caps_snapshot JSONB × dry_run preview no-side-effects × proration integer-cents math × FR-TEN-004 metering event linkage × 4 closed BRAIN audit kinds × per-tenant override stronger than plan default × reason text scrubbed via FR-BRAIN-111 × RLS isolation), not by line targets.

---

*End of FR-TEN-002 audit.*
