---
id: TASK-TEN-003
title: "Stripe billing integration — USD/EUR/SGD/GBP customer + subscription + per-period invoice + overage push for international tenants"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-05-17T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: TEN
priority: p0
status: draft
verify: T
phase: P2
milestone: P2 · billing-substrate
slice: 2
owner: Stephen Cheng (CFO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-TEN-001, TASK-TEN-002, TASK-TEN-004, TASK-TEN-101, TASK-TEN-102, TASK-TEN-103, TASK-TEN-104, TASK-INV-003, TASK-INV-006, TASK-AUTH-101, TASK-AI-003, TASK-MEMORY-111, TASK-OBS-007]
depends_on: [TASK-INV-003, TASK-TEN-002, TASK-TEN-004]
blocks: [TASK-TEN-102, TASK-TEN-101]

source_pages:
  - website/docs/modules/ten.html#billing
  - website/docs/modules/ten.html#international
  - https://stripe.com/docs/api/customers
  - https://stripe.com/docs/api/subscriptions
  - https://stripe.com/docs/api/subscription_items
  - https://stripe.com/docs/api/usage_records
  - https://stripe.com/docs/api/subscription_schedules
  - https://stripe.com/docs/billing/subscriptions/metered
  - https://stripe.com/docs/api/idempotent_requests
  - https://stripe.com/docs/billing/revenue-recognition

source_decisions:
  - DEC-784 2026-05-17 — Stripe is the international billing rail (USD/EUR/SGD/GBP); VND domestic via TASK-TEN-102 (VnPay/Momo/ZaloPay); a tenant is on exactly one rail
  - DEC-785 2026-05-17 — billing_currency = function of residency at provisioning: vn-1→VND (TASK-TEN-102 path), sg-1→SGD, eu-1→EUR, us-1→USD; GBP available only as explicit `--billing-currency GBP` override at provisioning when residency=eu-1
  - DEC-786 2026-05-17 — Stripe Customer created lazily on first non-VND billing event (subscription create, refund, or manual invoice); not at tenant provisioning — tenants may exist briefly before billing rail activation
  - DEC-787 2026-05-17 — Stripe Customer creation idempotent on `(tenant_id)` via Stripe `Idempotency-Key: ten.<tenant_id>.customer_create.v1` + `metadata.cyberos_tenant_id`
  - DEC-788 2026-05-17 — Subscription billing cycle anchored to `tenant.provisioned_at` calendar-day (e.g., provisioned 2026-05-17 → billing day 17 each month); leap-day edge: 29/30/31 falls back to last day of month
  - DEC-789 2026-05-17 — Overage charges modeled as Stripe Subscription Items with `usage_type=metered`; reported via Usage Records API at billing-period close (TASK-TEN-004 period_close hook)
  - DEC-790 2026-05-17 — Failed payment dunning — Stripe retries per `smart_retries` schedule (3 attempts over 14 days); after final failure, TEN sets `tenant.dunning_state=suspended` + TASK-TEN-104 status transition to `suspended`
  - DEC-791 2026-05-17 — Refunds via Stripe API only (no in-app refund button); requires `cfo` role per TASK-AUTH-101; sev-1 memory audit; max refund amount = original invoice (no over-refund)
  - DEC-792 2026-05-17 — Price catalog as compile-time Rust constants per (currency × tier) matrix; CI cardinality test asserts 3 tiers × 4 currencies = 12 entries; Stripe Price IDs synced at deploy time via `cyberos-ten stripe-sync-prices`
  - DEC-793 2026-05-17 — Stripe webhook events from TASK-INV-003 dispatch into TEN handlers via NATS subject `tenant.<slug>.ten.stripe.<event_type>`; INV layer is rail-agnostic, TEN consumes domain events
  - DEC-794 2026-05-17 — Idempotency_key on every Stripe write API call; format `ten.<tenant_id>.<operation>.<resource_ref_or_period_ts>`; max 255 chars (Stripe limit)
  - DEC-795 2026-05-17 — Stripe API errors with `Retry-After` honoured; 5xx → exponential backoff (1s, 2s, 4s, 8s, 16s) to 5 min cap; persistent 5xx → memory audit + alert sev-2
  - DEC-796 2026-05-17 — `tenants.stripe_customer_id` UNIQUE NOT NULL when populated; nullable until first Stripe interaction; partial unique index `WHERE stripe_customer_id IS NOT NULL`
  - DEC-797 2026-05-17 — Plan downgrade defers Stripe subscription update to next billing period via Stripe Subscription Schedule (consistent with TASK-TEN-002 DEC-773)
  - DEC-798 2026-05-17 — Multi-currency conversion not handled — tenant locked to billing_currency at provisioning; currency change = new tenant + manual migration (out-of-scope, deferred to task-TEN-2xx)
  - DEC-799 2026-05-17 — Tax handling deferred to task-TEN-1xx — Stripe Tax integration not in slice 2; invoices ship `tax_behavior=exclusive` + manual tax_rate=0 in P2; tax-inclusive pricing for EU VAT lands at P3
  - DEC-800 2026-05-17 — Plan upgrade prorates via Stripe `proration_behavior=create_prorations`; downgrade uses `proration_behavior=none` (deferred per DEC-797)
  - DEC-801 2026-05-17 — Stripe API key per residency (us-1 uses Stripe US account; eu-1 uses Stripe EU; sg-1 uses Stripe Singapore); TASK-TEN-103 wires the residency→API-key map; this task consumes that map and never short-circuits
  - DEC-802 2026-05-17 — Stripe Customer email = the tenant's `billing_contact_email`; PII-scrubbed in memory chain via TASK-MEMORY-111 (hash16 form retained for forensic match)
  - DEC-803 2026-05-17 — Subscription cancellation is non-destructive — set `cancel_at_period_end=true`; tenant retains access until period boundary; hard cancel via TASK-TEN-104 termination flow only
  - DEC-804 2026-05-17 — Stripe webhook `invoice.payment_succeeded` → emit `ten.stripe_invoice_paid` memory row + clear `dunning_state` + un-suspend tenant if previously suspended via dunning
  - DEC-805 2026-05-17 — Founder tenant (TASK-TEN-002 DEC-777) skips Stripe billing entirely — `tenant.is_founder_tenant=true` short-circuits all Stripe API calls; founder tenant carries `billing_rail='internal'` synthetic value
  - DEC-806 2026-05-17 — Per-tenant `stripe_subscription_id` UNIQUE NOT NULL when populated; partial unique index `WHERE stripe_subscription_id IS NOT NULL`
  - DEC-807 2026-05-17 — Idempotency cache for outbound Stripe API calls stored in `stripe_api_calls` table; entries pruned at 7 days (Stripe accepts idempotency keys for 24h, we keep 7d for forensic replay)
  - DEC-808 2026-05-17 — Stripe Webhook → TEN dispatcher MUST be at-least-once safe; every consumer is idempotent via `(tenant_id, stripe_event_id)` UNIQUE in `stripe_event_dispatch_log`
  - DEC-809 2026-05-17 — Invoice currency MUST match tenant.billing_currency; mismatch from webhook = sev-1 alert + reject + manual reconciliation
  - DEC-810 2026-05-17 — Overage push window: at most 1 hour after period_close completes; if push fails, retry up to 24 h then alert sev-1 (lost overage revenue = recoverable but visible)
  - PDPL Art. 13 (data minimisation — billing_contact_email PII-scrubbed in memory chain via TASK-MEMORY-111)
  - PCI DSS SAQ-A (Stripe-hosted card data — no PAN at our endpoint)
  - EU AI Act Art. 12 (audit trail — every Stripe state change emits a memory row with chain hash)
  - GDPR Art. 7 (consent — billing_contact_email collected at provisioning with explicit consent acknowledgement; not used for marketing without separate opt-in)

build_envelope:
  language: rust 1.81
  service: cyberos/services/ten/
  new_files:
    - services/ten/migrations/0006_stripe_billing.sql                  # stripe_customer_id + stripe_subscription_id columns + dunning_state + billing_currency
    - services/ten/migrations/0007_stripe_api_calls.sql                # idempotency cache + audit trail for outbound calls
    - services/ten/migrations/0008_stripe_event_dispatch_log.sql       # inbound webhook dispatch log (idempotency on tenant_id × stripe_event_id)
    - services/ten/migrations/0009_billing_currency_enum.sql           # closed enum (VND, USD, EUR, SGD, GBP)
    - services/ten/migrations/0010_stripe_price_map.sql                # rail-synced (currency × tier) → Stripe Price ID
    - services/ten/src/billing/mod.rs                                  # billing orchestrator + rail selector
    - services/ten/src/billing/stripe/mod.rs                           # Stripe rail entry
    - services/ten/src/billing/stripe/customer.rs                      # create_customer + idempotency
    - services/ten/src/billing/stripe/subscription.rs                  # create_subscription + plan_change_push + cancel_at_period_end
    - services/ten/src/billing/stripe/overage.rs                       # usage_record push at period close
    - services/ten/src/billing/stripe/refund.rs                        # refund handler (CFO-gated)
    - services/ten/src/billing/stripe/dispatch.rs                      # NATS consumer for ten.stripe.* events
    - services/ten/src/billing/stripe/idempotency.rs                   # idempotency-key builder + cache
    - services/ten/src/billing/stripe/api_client.rs                    # async client + retry + Retry-After honour
    - services/ten/src/billing/stripe/price_catalog.rs                 # compile-time (currency × tier) → Price ID + amount
    - services/ten/src/billing/dunning.rs                              # dunning state machine + suspend trigger
    - services/ten/src/repo/stripe_customers.rs                        # tenant.stripe_customer_id CRUD (partial-unique guarded)
    - services/ten/src/repo/stripe_api_calls.rs                        # idempotency cache repo
    - services/ten/src/repo/stripe_event_dispatch_log.rs               # inbound dispatch idempotency repo
    - services/ten/src/audit/stripe_events.rs                          # 11 memory row builders
    - services/ten/src/cli/stripe_sync_prices.rs                       # cyberos-ten stripe-sync-prices (deploy-time price sync)
    - services/ten/src/handlers/billing_refund.rs                      # POST /v1/admin/tenants/{id}/billing/refund (CFO-only)
    - services/ten/src/handlers/billing_show.rs                        # GET /v1/admin/tenants/{id}/billing (subscription state + dunning + history)
    - services/ten/tests/stripe_customer_create_test.rs
    - services/ten/tests/stripe_customer_idempotency_test.rs
    - services/ten/tests/stripe_subscription_create_test.rs
    - services/ten/tests/stripe_plan_upgrade_proration_test.rs
    - services/ten/tests/stripe_plan_downgrade_defer_test.rs
    - services/ten/tests/stripe_overage_push_test.rs
    - services/ten/tests/stripe_dunning_state_machine_test.rs
    - services/ten/tests/stripe_dispatcher_idempotency_test.rs
    - services/ten/tests/stripe_refund_cfo_only_test.rs
    - services/ten/tests/stripe_founder_skip_test.rs
    - services/ten/tests/stripe_currency_lock_test.rs
    - services/ten/tests/stripe_api_retry_test.rs
    - services/ten/tests/stripe_price_catalog_cardinality_test.rs
    - services/ten/tests/stripe_residency_apikey_routing_test.rs
    - services/ten/tests/stripe_audit_emission_test.rs

  modified_files:
    - services/ten/src/handlers/plan_change.rs                         # plan_change handler invokes billing/stripe::push_plan_change after history write
    - services/ten/src/handlers/tenant_create.rs                       # tenant_create accepts --billing-currency override (residency default applied)
    - services/ten/Cargo.toml                                          # +async-stripe = "0.39" (or rust-stripe equivalent), +backoff, +kms-aws
    - services/inv/src/webhook/stripe_event_dispatch.rs                # bridge: relevant kinds NATS-publish to `tenant.<slug>.ten.stripe.<kind>`
    - services/metering/src/handlers/period_close.rs                   # period_close hook invokes ten::billing::stripe::push_overage_for_period

  allowed_tools:
    - file_read: services/ten/**
    - file_read: services/inv/src/webhook/**
    - file_read: services/metering/src/handlers/**
    - file_write: services/ten/{src,tests,migrations}/**
    - file_write: services/inv/src/webhook/stripe_event_dispatch.rs
    - file_write: services/metering/src/handlers/period_close.rs
    - bash: cd services/ten && cargo test stripe
    - bash: cd services/ten && cargo run --bin cyberos-ten -- stripe-sync-prices --dry-run

  disallowed_tools:
    - call Stripe API without an Idempotency-Key (per DEC-794)
    - charge a VND tenant via Stripe (per DEC-784 — VND uses TASK-TEN-102 only)
    - charge the founder tenant via Stripe (per DEC-805)
    - mutate billing_currency on an existing tenant (per DEC-798)
    - store Stripe API key in plaintext (KMS-encrypted only; per DEC-801)
    - hardcode Stripe Price IDs in source (must round-trip through stripe_price_map per DEC-792)
    - allow over-refund (per DEC-791 — max = original invoice amount)
    - silently swallow Stripe 5xx after backoff exhausted (per DEC-795 — alert sev-2)

effort_hours: 8
subtasks:
  - "0.5h: 0006_stripe_billing.sql + 0009_billing_currency_enum.sql migrations + partial unique indexes"
  - "0.5h: 0007_stripe_api_calls.sql + 0008_stripe_event_dispatch_log.sql + 0010_stripe_price_map.sql"
  - "0.8h: price_catalog.rs (12-entry currency×tier const table) + cardinality CI test"
  - "0.8h: api_client.rs with Retry-After + exponential backoff + idempotency-key threading"
  - "0.8h: customer.rs + idempotent create + Stripe metadata.cyberos_tenant_id"
  - "1.0h: subscription.rs (create + plan upgrade with prorations + downgrade via SubscriptionSchedule)"
  - "0.6h: overage.rs (per-axis Subscription Item + Usage Records push at period_close)"
  - "0.6h: dunning.rs (state machine: ok→retry1→retry2→retry3→suspended; un-suspend on payment_succeeded)"
  - "0.5h: dispatch.rs (NATS consumer + per-event idempotency via stripe_event_dispatch_log)"
  - "0.4h: refund.rs + handlers/billing_refund.rs (CFO-gated)"
  - "0.4h: handlers/billing_show.rs + audit/stripe_events.rs (11 builders)"
  - "0.3h: cli/stripe_sync_prices.rs (one-shot deploy-time Price ID upsert with --dry-run + --apply)"
  - "1.0h: tests — 15 test files covering happy + 5xx retry + idempotency + currency lock + founder skip + cardinality + dispatcher"
  - "0.3h: wire-up — plan_change.rs push hook, period_close.rs overage hook, inv dispatch NATS-publish"
  - "0.5h: integration smoke against Stripe test mode + sandbox tenant"

risk_if_skipped: "Without Stripe billing, every international tenant payment becomes a manual Stripe Dashboard entry — non-scalable past ~10 tenants, error-prone, no audit linkage to TASK-TEN-002 plan changes or TASK-TEN-004 overages. Tenants on sg-1/eu-1/us-1 residencies (the entire international market) cannot be billed at all without this task. Without DEC-787's idempotent customer creation, a transient failure during plan_change spawns duplicate Stripe Customers (one tenant → many customers = billing chaos + revenue recognition errors). Without DEC-789's metered Subscription Items, overage charges from TASK-TEN-004 metering are stranded in memory chain with no downstream rail to bill them. Without DEC-790's dunning state machine, failed payments leak revenue indefinitely (Stripe retries 3x then gives up; we must respond). Without DEC-805's founder-skip, the founder tenant gets charged for its own product (accounting noise + circular invoicing). Without DEC-798's currency lock, a tenant could be in two Stripe Customers (one USD, one EUR), making revenue split unanswerable. Without DEC-794's idempotency keys, network retries double-charge. TASK-TEN-101 self-serve signup cannot ship without an automated billing rail. TASK-TEN-102 VND domestic rail is a parallel rail; without TEN-003 first, the rail abstraction doesn't exist. The 8h effort lands the international billing primitive that unlocks the entire P3 commercial rollout."
---

## §1 — Description (BCP-14 normative)

The TEN service **MUST** ship the Stripe billing rail at `services/ten/src/billing/stripe/` — Customer/Subscription/Usage-Record lifecycle for tenants whose `billing_currency ∈ {USD, EUR, SGD, GBP}`, plan-change push from TASK-TEN-002, overage push from TASK-TEN-004, dispatcher for TASK-INV-003 webhook events, dunning state machine, refund flow, and 11 memory audit row kinds.

1. **MUST** add columns to `tenants` via migration `0006_stripe_billing.sql`:
   - `billing_currency billing_currency_enum NOT NULL DEFAULT 'VND'` — closed enum from migration `0009`.
   - `billing_rail TEXT NOT NULL CHECK (billing_rail IN ('stripe','vietqr_momo_zalo','internal')) DEFAULT 'vietqr_momo_zalo'` — derived from `billing_currency` at provisioning; `internal` reserved for founder tenant per DEC-805.
   - `stripe_customer_id TEXT` — nullable; populated lazily per DEC-786.
   - `stripe_subscription_id TEXT` — nullable; populated on first plan_change_push.
   - `billing_contact_email TEXT NOT NULL` — collected at provisioning; used as Stripe Customer email.
   - `dunning_state TEXT NOT NULL CHECK (dunning_state IN ('ok','retry_1','retry_2','retry_3','suspended')) DEFAULT 'ok'`.
   - Partial unique index `CREATE UNIQUE INDEX uniq_stripe_customer ON tenants(stripe_customer_id) WHERE stripe_customer_id IS NOT NULL`.
   - Partial unique index `CREATE UNIQUE INDEX uniq_stripe_subscription ON tenants(stripe_subscription_id) WHERE stripe_subscription_id IS NOT NULL`.

2. **MUST** define the closed `billing_currency_enum` Postgres type at migration `0009` with exactly 5 values: `VND, USD, EUR, SGD, GBP`. CI cardinality test asserts 5 (DEC-792 + task-audit skill rule 6 namespace pattern adapted for enum). A 6th value requires a schema migration + DEC entry.

3. **MUST** derive `billing_rail` from `billing_currency` at tenant provisioning (TASK-TEN-001 handler):
   - `VND` → `vietqr_momo_zalo` (TASK-TEN-102 path).
   - `USD | EUR | SGD | GBP` → `stripe` (this task's path).
   - Founder tenant (`is_founder_tenant=true`) → `internal` (DEC-805) regardless of currency; Stripe API calls short-circuit with a no-op + sev-3 memory row `ten.stripe_founder_skip`.

4. **MUST** lock `billing_currency` at provisioning — `UPDATE tenants SET billing_currency = ...` is REJECTED by a row-level trigger (DEC-798). Changing currency = create new tenant + manual data migration (out-of-scope, task-TEN-2xx).

5. **MUST** define the compile-time price catalog in `services/ten/src/billing/stripe/price_catalog.rs` as `const PRICE_CATALOG: [PriceEntry; 12]`. The 12 entries are 3 plan tiers × 4 international currencies (Starter/Team/Enterprise × USD/EUR/SGD/GBP). Each `PriceEntry` carries `{ currency, tier, amount_minor: i64 }`. The Stripe Price IDs are NOT baked into the constant — they live in the `stripe_price_map` Postgres table (per residency × currency × tier × axis) populated by the deploy-time `cyberos-ten stripe-sync-prices` CLI per DEC-792. Overage axes (4 per tier per currency) are ALSO mapped via `stripe_price_map.axis ∈ {base, overage_api_calls, overage_ai_tokens, overage_storage, overage_seats}`, giving 5 metered prices per (currency, tier) — totalling 12 base + 48 overage = 60 distinct Stripe Price IDs per residency. The CI cardinality test asserts 12 base catalog entries (the overage prices are derived from base — same currency/tier, different axis). Reference monthly base amounts (slice 2 baseline; CFO may rev via a follow-up DEC entry):

   | currency | starter | team | enterprise |
   |---|---:|---:|---:|
   | USD | 1_900 (¢) | 9_900 (¢) | 99_900 (¢) |
   | EUR | 1_900 (¢) | 9_900 (¢) | 99_900 (¢) |
   | SGD | 2_500 (¢) | 13_000 (¢) | 130_000 (¢) |
   | GBP | 1_500 (p) | 7_900 (p) | 79_900 (p) |

   Stripe Price IDs are populated by `cyberos-ten stripe-sync-prices` against the Stripe API (per residency Stripe account per DEC-801) and persisted to migration `0010_stripe_price_map`. CI test `stripe_price_catalog_cardinality_test` asserts exactly 12 entries and every entry has a non-empty `stripe_price_id_prod` after deploy-time sync.

6. **MUST** create the Stripe Customer lazily on the first billing event for the tenant (subscription create, manual invoice, refund). The path is `services/ten/src/billing/stripe/customer.rs::ensure_customer(tenant_id)`:
   - Lookup `tenants.stripe_customer_id`. If non-NULL → return.
   - Else: call Stripe `POST /v1/customers` with `Idempotency-Key: ten.<tenant_id>.customer_create.v1` and body `{ email: <billing_contact_email>, name: <display_name>, metadata: { cyberos_tenant_id: <tenant_id>, cyberos_residency: <residency>, cyberos_billing_currency: <currency> } }`.
   - On success: `UPDATE tenants SET stripe_customer_id = $1 WHERE id = $2` (partial-unique-guarded; concurrent attempts on same tenant race-safe via SELECT FOR UPDATE).
   - Emit memory row `ten.stripe_customer_created` with `(tenant_id, stripe_customer_id, currency, residency)`.

7. **MUST** create the Stripe Subscription at the first non-Starter plan_change OR at provisioning when `--seed-subscription` flag is set (default: subscription created lazily on plan_change → Team/Enterprise; Starter tenants get subscriptions too since Starter is paid per TASK-TEN-002 DEC-778). Path: `services/ten/src/billing/stripe/subscription.rs::ensure_subscription(tenant_id, plan_tier)`:
   - Ensure customer exists (per §1 #6).
   - Resolve Stripe Price ID via `price_catalog.rs` lookup on `(billing_currency, plan_tier)`.
   - Call Stripe `POST /v1/subscriptions` with `Idempotency-Key: ten.<tenant_id>.subscription_create.<plan_tier_ordinal>` and body `{ customer: <stripe_customer_id>, items: [{ price: <price_id> }, { price: <overage_meter_price_id>, ... per axis }], billing_cycle_anchor: <tenant.provisioned_at unix ts (DEC-788)>, proration_behavior: "create_prorations", metadata: { cyberos_tenant_id, cyberos_plan_tier } }`.
   - Subscription Items for the 4 overage axes (DEC-789): one Stripe `subscription_item` per `(currency, axis)` with `recurring: { usage_type: "metered" }` — 4 metered prices per currency (api_calls overage, ai_tokens overage, storage overage, seats overage) stored in `stripe_price_map` alongside the base tier prices.
   - On success: `UPDATE tenants SET stripe_subscription_id = $1`.
   - Emit `ten.stripe_subscription_created`.

8. **MUST** push plan changes to Stripe (`services/ten/src/billing/stripe/subscription.rs::push_plan_change`) invoked synchronously from the TASK-TEN-002 `plan_change` handler AFTER the `tenant_plan_history` row commits, IN A SEPARATE TRANSACTION (Stripe API is external; no 2-phase commit). The handler:
   - If `from_tier < to_tier` (upgrade): Stripe `POST /v1/subscriptions/{id}` with `items: [{ id: <base_item_id>, price: <new_price_id> }]` and `proration_behavior: "create_prorations"` and `Idempotency-Key: ten.<tenant_id>.plan_upgrade.<history_id>`. Prorations land on the next invoice.
   - If `from_tier > to_tier` (downgrade, deferred per TASK-TEN-002 DEC-773): create a Stripe Subscription Schedule (`POST /v1/subscription_schedules`) anchored to `effective_at` from history, with `proration_behavior: "none"`. `Idempotency-Key: ten.<tenant_id>.plan_downgrade.<history_id>`.
   - Emit `ten.stripe_subscription_updated` with `(from_tier, to_tier, history_id, stripe_invoice_item_ids[])`.

9. **MUST** push overages at billing-period close (`services/ten/src/billing/stripe/overage.rs::push_overage_for_period`) invoked from the TASK-TEN-004 `period_close` hook AFTER the aggregation completes. For each axis where `actual > tier_cap`:
   - Compute `overage = actual - tier_cap`.
   - Call Stripe `POST /v1/subscription_items/{item_id}/usage_records` with `{ quantity: <overage>, timestamp: <period_end_unix>, action: "set" }` and `Idempotency-Key: ten.<tenant_id>.overage_push.<axis>.<period_end_unix>`.
   - Emit `ten.stripe_overage_pushed` with `(axis, overage_quantity, period_end, stripe_usage_record_id)`.
   - **Push window:** must complete within 1 hour of period_close (DEC-810). On failure → exponential backoff retry up to 24 h; persistent failure → sev-1 alert + memory row `ten.stripe_overage_push_failed`.

10. **MUST** dispatch TASK-INV-003 Stripe webhook events into TEN-side handlers via NATS subject `tenant.<slug>.ten.stripe.<event_type>` (DEC-793 + DEC-808). The consumer (`services/ten/src/billing/stripe/dispatch.rs`) is idempotent via `(tenant_id, stripe_event_id) UNIQUE` in `stripe_event_dispatch_log`. The relevant events:
    - `invoice.finalized` → audit only (`ten.stripe_invoice_finalized`).
    - `invoice.payment_succeeded` → clear dunning + emit `ten.stripe_invoice_paid` + un-suspend tenant if `dunning_state='suspended'` (DEC-804).
    - `invoice.payment_failed` → advance dunning state (per §1 #11) + emit `ten.stripe_invoice_payment_failed`.
    - `customer.subscription.updated` → log only if drift from our state (alert sev-2 if Stripe state ≠ tenants.stripe_subscription_id-derived state).
    - `customer.subscription.deleted` → emit `ten.stripe_subscription_cancelled` + DO NOT auto-terminate tenant (handled by TASK-TEN-104 termination flow which sets `cancel_at_period_end` first).

11. **MUST** advance the dunning state machine on `invoice.payment_failed` (`services/ten/src/billing/dunning.rs`):
    - `ok → retry_1` on first failure.
    - `retry_1 → retry_2` on second.
    - `retry_2 → retry_3` on third.
    - `retry_3 → suspended` on fourth → trigger TASK-TEN-104 `tenant_suspended` status transition + emit `ten.tenant_billing_suspended` + `ten.stripe_dunning_advanced` with `(prior_state, new_state)`.
    - Any state + `invoice.payment_succeeded` → `ok` + un-suspend if suspended.
    - State transitions are OBSERVED via `stripe_event_dispatch_log` rows joined with `tenants.dunning_state` point-in-time snapshots; the canonical history is the immutable inbound webhook log + the memory chain row emitted on each advance (DEC-808 derivative). No separate `tenant_dunning_history` table is created — the existing event log + memory chain row carry the same information without schema duplication.

12. **MUST** expose `POST /v1/admin/tenants/{id}/billing/refund` for refunds (`services/ten/src/handlers/billing_refund.rs`). Caller MUST have role `cfo` per TASK-AUTH-101 (DEC-791). Body: `{ stripe_charge_id, amount_minor, currency, reason }`. Validations:
    - `amount_minor ≤ original_charge_amount` (DEC-791 — no over-refund). Lookup original charge amount via Stripe `GET /v1/charges/{id}`.
    - `currency == tenant.billing_currency` (DEC-809).
    - Call Stripe `POST /v1/refunds` with `Idempotency-Key: ten.<tenant_id>.refund.<charge_id>.<amount_minor>` and body `{ charge: <id>, amount: <amount_minor>, reason: requested_by_customer | duplicate | fraudulent }`.
    - Emit `ten.stripe_refund_issued` at sev-1 with `(tenant_id, stripe_refund_id, charge_id, amount_minor, currency, cfo_subject_id, reason)`.
    - Returns `201 CREATED` with `{ refund_id, status, expected_settlement_at }`.

13. **MUST** expose `GET /v1/admin/tenants/{id}/billing` (`services/ten/src/handlers/billing_show.rs`). Returns:
    ```json
    { "tenant_id": "...", "billing_rail": "stripe", "billing_currency": "USD",
      "stripe_customer_id": "cus_...", "stripe_subscription_id": "sub_...",
      "current_plan_tier": "team", "dunning_state": "ok",
      "billing_cycle_anchor": "2026-05-17T00:00:00Z", "next_invoice_date": "2026-06-17T00:00:00Z",
      "recent_invoices": [ ... last 12 ... ], "recent_dunning_events": [ ... ] }
    ```
    Caller MUST have role `tenant_admin` (own tenant) or `cfo` (any tenant).

14. **MUST** thread `Idempotency-Key` on every outbound Stripe write API call (DEC-794 + task-audit skill rule 12 derivative for external systems). Key format `ten.<tenant_id>.<operation>.<resource_ref_or_period_ts>`. Max 255 chars. Persisted in `stripe_api_calls` table with columns `(idempotency_key TEXT PRIMARY KEY, tenant_id UUID, operation TEXT, request_sha256 CHAR(64), response_status INT, response_body_sha256 CHAR(64), created_at TIMESTAMPTZ, ttl_until TIMESTAMPTZ DEFAULT now() + INTERVAL '7 days')`. A scheduled job prunes entries past `ttl_until` daily.

15. **MUST** honour Stripe API `Retry-After` header on rate-limit (429) responses (DEC-795). For 5xx responses without `Retry-After`, apply exponential backoff: `1s, 2s, 4s, 8s, 16s` capped at 5 min (max 5 retries). After exhaustion: memory audit `ten.stripe_api_call_failed` at sev-2 + return `502 BAD_GATEWAY` to caller with `{ error: "stripe_unavailable", retry_after_seconds: <next_retry> }`.

16. **MUST** route Stripe API calls through the per-residency Stripe API key (DEC-801). The api_client constructor takes `residency: Residency` and resolves `STRIPE_API_KEY_<RESIDENCY>` from KMS-encrypted secrets store. Wrong residency → wrong Stripe account → forensically catastrophic; CI test `stripe_residency_apikey_routing_test` asserts correct routing per residency.

17. **MUST** enforce RLS with both `USING` and `WITH CHECK` on `stripe_api_calls` and `stripe_event_dispatch_log` tables (task-audit skill rule 13). Policy: `tenant_id = current_setting('auth.tenant_id')::uuid`.

18. **MUST** REVOKE UPDATE, DELETE on `stripe_api_calls`, `stripe_event_dispatch_log`, and `stripe_price_map` from `cyberos_app` role (task-audit skill rule 12). Pruning of `stripe_api_calls` past TTL uses a separate `cyberos_pruner` role with DELETE grant only on past-TTL rows.

19. **MUST** emit 11 memory audit row kinds (DEC-784 expansion + task-audit skill rule 6 namespace pattern `^[a-z][a-z0-9_]*\.[a-z][a-z0-9_]*$`):
    - `ten.stripe_customer_created` (sev-2)
    - `ten.stripe_subscription_created` (sev-2)
    - `ten.stripe_subscription_updated` (sev-2)
    - `ten.stripe_subscription_cancelled` (sev-1)
    - `ten.stripe_invoice_finalized` (sev-3)
    - `ten.stripe_invoice_paid` (sev-2)
    - `ten.stripe_invoice_payment_failed` (sev-2)
    - `ten.stripe_dunning_advanced` (sev-1)
    - `ten.tenant_billing_suspended` (sev-1)
    - `ten.stripe_refund_issued` (sev-1)
    - `ten.stripe_overage_pushed` (sev-3)

    Each row carries `trace_id` (32-char W3C hex per task-audit skill rule 23 + 24) and is PII-scrubbed via TASK-MEMORY-111 BEFORE chain commit (task-audit skill rule 18). `billing_contact_email` is hashed (`billing_contact_email_hash16`) in chain; full value retained only in tenant Postgres.

20. **MUST** ship the `cyberos-ten stripe-sync-prices` deploy-time CLI (`services/ten/src/cli/stripe_sync_prices.rs`). Behaviour:
    - `--dry-run` (default): list every `(residency, currency, tier, axis)` permutation in `PRICE_CATALOG × AXES` (12 base + 48 overage = 60 per residency) and the Stripe Price ID it would create/lookup.
    - `--apply`: for each entry without an existing Stripe Price ID (looked up via Stripe `GET /v1/prices?lookup_keys=...` with `lookup_key = "cyberos.<currency>.<tier>.<axis>"`), call Stripe `POST /v1/prices` with the catalog amount + `metadata: { cyberos_currency, cyberos_tier, cyberos_axis }`; persist returned Price ID to `stripe_price_map`.
    - `--residency <sg-1|eu-1|us-1>` REQUIRED on `--apply` (CLI loads only that residency's KMS-encrypted Stripe API key per DEC-801); `--dry-run` defaults to all-residencies for reporting. Cross-residency API misuse is impossible because the loaded API key only addresses one Stripe account.
    - Idempotent on `(residency, currency, tier, axis)`; re-run is a no-op when all entries already mapped.
    - Exit codes from `cyberos-cli-exit` shared crate (task-audit skill rule 9): 0 success, 1 nothing-to-do, 64 invalid-arg, 73 stripe-create-failed, 77 perm-denied.

21. **MUST** support concurrent plan_change events safely. The plan_change handler acquires `SELECT ... FOR UPDATE` on the tenant row before computing the Stripe push; second concurrent plan_change blocks until first commits. Combined with the TASK-TEN-002 24h rate limit, plan_change concurrency is bounded.

22. **MUST NOT** call any Stripe write API for a tenant with `is_founder_tenant=true` (DEC-805). Guard at `services/ten/src/billing/stripe/api_client.rs::call()` entry — if `tenant.billing_rail = 'internal'`, return synthetic success with `Stripe-API-Bypass: founder` header for trace correlation + emit sev-3 memory row `ten.stripe_founder_skip` (informational; not in the 11 main kinds because founder is rare).

23. **MUST NOT** call any Stripe write API for a tenant with `billing_currency = 'VND'` (DEC-784). Guard at api_client entry — VND tenant routes through TASK-TEN-102 only. Cross-rail attempts return `400 BAD_REQUEST` with `{ error: "wrong_billing_rail", expected: "vietqr_momo_zalo", got: "stripe" }`.

24. **MUST** PII-scrub `billing_contact_email` and any reason text in refund/dunning audit rows via TASK-MEMORY-111 BEFORE chain commit (task-audit skill rule 18). Raw values retained in tenant Postgres (RLS-scoped); memory chain holds `billing_contact_email_hash16` and scrubbed reason.

25. **SHOULD** observe Stripe API latency p95 via OTel span `stripe.api.<operation>` (task-audit skill rule 22 + 24). Alarm sev-3 if p95 > 2 s sustained 5 min; sev-2 if p95 > 5 s.

---

## §2 — Why this design (rationale for humans)

**Why a separate `billing/stripe/` subtree rather than putting Stripe code into `services/inv/` (§1 #1–§1 #5)?** TASK-INV-003 is the *webhook receiver* — it does signature verification, idempotency dedupe, and writes raw `payment_receipts` rows. The TEN billing rail is the *outbound subscription lifecycle*: customer create, subscription create/update, usage record push, dunning state. These are different concerns: INV is rail-agnostic ("a payment of $X arrived from Y"), TEN owns the subscription contract with the customer. Mixing them couples webhook idempotency to subscription state machines and makes it impossible to add the TASK-TEN-102 VND rail without forking INV. The clean cut is: INV records inbound money, TEN orchestrates outbound rail calls + dispatches relevant inbound events back to TEN handlers via NATS (DEC-793).

**Why lazy Stripe Customer creation rather than at tenant provisioning (§1 #6, DEC-786)?** Provisioning happens before we know whether a tenant will ever transact. If the operator provisions 100 tenants for a demo and 90 are abandoned, eagerly creating 90 Stripe Customers leaks data into Stripe (PII residency concerns) and inflates Stripe Customer count for billing/audit. Lazy creation defers the externalisation until there's a real billing event — and the idempotency-key strategy means concurrent first-events don't race.

**Why a 12-entry compile-time price catalog rather than runtime lookup from Stripe (§1 #5, DEC-792)?** Two reasons. First, Stripe Price IDs are environment-specific (test vs prod, per-residency Stripe account) — runtime lookup requires API calls on every plan_change, which is latency + Stripe rate-limit pressure. Second, the price catalog IS the commercial contract — keeping it in source means a PR review enforces price changes (visible to engineering + legal + CFO). The `stripe_price_map` table is the binding between our internal `(currency, tier)` keys and Stripe's IDs, written once at deploy time by `stripe-sync-prices`.

**Why per-residency Stripe API key (§1 #16, DEC-801)?** EU-residency tenants billing through a Stripe US account violates GDPR data residency. Stripe operates separate legal entities per region (Stripe US, Stripe Singapore, Stripe Ireland for EU) with separate Connect accounts and separate KYC. Tenants on `eu-1` MUST bill through Stripe Ireland; `us-1` through Stripe US; `sg-1` through Stripe Singapore. A single global Stripe key is forensically catastrophic for compliance.

**Why dunning state machine in TEN rather than relying on Stripe's smart retries alone (§1 #11)?** Stripe smart retries handle payment retry timing but don't trigger our `tenant.status=suspended` transition. We need our own state machine that mirrors Stripe's retry attempts (event-driven via `invoice.payment_failed`) AND lands a domain transition (TASK-TEN-104 suspend → tenant CHAT goes read-only, no new metering events bill). Stripe is the source of truth for payment retry; TEN is the source of truth for tenant access state.

**Why subscription cancellation is non-destructive (§1 #10, DEC-803)?** Hard-cancel removes Stripe subscription immediately, but the tenant has paid through end-of-period — yanking access mid-period creates support escalations and refund pressure. `cancel_at_period_end=true` is industry standard: tenant retains access until they've used what they paid for; full termination via TASK-TEN-104 only when the period ends OR operator explicitly invokes hard-cancel.

**Why founder-skip rather than just billing $0 (§1 #22, DEC-805)?** Billing $0 still creates a Stripe Customer + Subscription, which is accounting noise (every monthly close has to explain why there's a $0 Stripe invoice for the founder tenant), and circular invoicing creates auditor confusion ("CyberSkill invoicing itself?"). Founder tenant short-circuits the whole rail; settled in internal accounting (not Stripe).

**Why idempotency keys at our layer when Stripe already has them (§1 #14, DEC-794)?** Stripe's idempotency keys protect Stripe — they're the *receiver*. We need our own keys to protect *us*: if our handler crashes after the Stripe call but before persisting the response, the retry sees Stripe's "you already did this" and we recover the result without double-side-effecting our own DB. Both layers must agree on key shape, which is why DEC-794 mandates the format.

**Why overage push at period_close rather than per-event (§1 #9, DEC-789, DEC-810)?** Per-event push to Stripe is rate-limit-prohibitive (Stripe limits ~100 req/s; a busy tenant might emit 10k metering events/period) and noisy (per-event partial usage records bloat invoices). Period_close aggregation gives one usage_record per axis per period — clean invoices + reasonable Stripe API load. The 1-hour push window matches Stripe's billing-finalization grace period.

**Why per-residency Stripe price IDs (§1 #5, DEC-801 derivative)?** A Stripe Price ID belongs to one Stripe account. If we tried to share `price_team_usd` across the US and Singapore Stripe accounts, it wouldn't exist in the other account → API error. Each residency's `stripe-sync-prices` populates that residency's account; the `stripe_price_map` table is keyed `(residency, currency, tier)` → `stripe_price_id`. Mid-provisioning failure where some residencies have prices and others don't is detectable at deploy via `--dry-run` listing un-mapped entries.

**Why downgrade defers via Subscription Schedule rather than just setting a flag (§1 #8, DEC-797)?** Stripe Subscription Schedule is the native Stripe primitive for "switch to plan X at time T". Re-implementing the deferral in our code means we'd have to (a) run a scheduled job at period boundary, (b) call Stripe to swap items, (c) reconcile with our plan_history. All three steps are failure points. Subscription Schedule moves the deferral into Stripe — when the period boundary hits, Stripe auto-swaps and emits `customer.subscription.updated` which we observe through dispatch (§1 #10).

---

## §3 — API contract

### 3.1 Postgres schema (migrations)

```sql
-- 0006_stripe_billing.sql
ALTER TABLE tenants
  ADD COLUMN billing_currency billing_currency_enum NOT NULL DEFAULT 'VND',
  ADD COLUMN billing_rail TEXT NOT NULL DEFAULT 'vietqr_momo_zalo'
    CHECK (billing_rail IN ('stripe','vietqr_momo_zalo','internal')),
  ADD COLUMN stripe_customer_id TEXT,
  ADD COLUMN stripe_subscription_id TEXT,
  ADD COLUMN stripe_subscription_items_map JSONB NOT NULL DEFAULT '{}'::jsonb,
  ADD COLUMN billing_contact_email TEXT,  -- NOT NULL activated after 7-day backfill (§11.11)
  ADD COLUMN dunning_state TEXT NOT NULL DEFAULT 'ok'
    CHECK (dunning_state IN ('ok','retry_1','retry_2','retry_3','suspended'));

CREATE UNIQUE INDEX uniq_stripe_customer ON tenants(stripe_customer_id)
  WHERE stripe_customer_id IS NOT NULL;
CREATE UNIQUE INDEX uniq_stripe_subscription ON tenants(stripe_subscription_id)
  WHERE stripe_subscription_id IS NOT NULL;

-- Trigger: billing_currency is immutable post-provisioning (DEC-798)
CREATE OR REPLACE FUNCTION trg_billing_currency_immutable() RETURNS trigger AS $$
BEGIN
  IF OLD.billing_currency IS DISTINCT FROM NEW.billing_currency THEN
    RAISE EXCEPTION 'billing_currency_immutable: cannot change billing_currency on existing tenant (DEC-798); create new tenant + manual migration';
  END IF;
  RETURN NEW;
END $$ LANGUAGE plpgsql;
CREATE TRIGGER tenants_billing_currency_immutable
  BEFORE UPDATE ON tenants
  FOR EACH ROW EXECUTE FUNCTION trg_billing_currency_immutable();

-- Trigger: founder tenants cannot have stripe_customer_id populated (DEC-805)
CREATE OR REPLACE FUNCTION trg_founder_no_stripe() RETURNS trigger AS $$
BEGIN
  IF NEW.is_founder_tenant = true AND NEW.stripe_customer_id IS NOT NULL THEN
    RAISE EXCEPTION 'founder_cannot_stripe: is_founder_tenant=true tenants cannot have stripe_customer_id (DEC-805)';
  END IF;
  IF NEW.billing_currency = 'VND' AND NEW.stripe_customer_id IS NOT NULL THEN
    RAISE EXCEPTION 'wrong_billing_rail: VND tenants cannot have stripe_customer_id (DEC-784)';
  END IF;
  RETURN NEW;
END $$ LANGUAGE plpgsql;
CREATE TRIGGER tenants_founder_no_stripe
  BEFORE INSERT OR UPDATE ON tenants
  FOR EACH ROW EXECUTE FUNCTION trg_founder_no_stripe();

-- 0009_billing_currency_enum.sql (run BEFORE 0006)
CREATE TYPE billing_currency_enum AS ENUM ('VND','USD','EUR','SGD','GBP');

-- 0007_stripe_api_calls.sql
CREATE TABLE stripe_api_calls (
  idempotency_key TEXT PRIMARY KEY,
  tenant_id UUID NOT NULL,
  operation TEXT NOT NULL,
  request_sha256 CHAR(64) NOT NULL,
  response_status INT,
  response_body_sha256 CHAR(64),
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  ttl_until TIMESTAMPTZ NOT NULL DEFAULT now() + INTERVAL '7 days'
);
ALTER TABLE stripe_api_calls ENABLE ROW LEVEL SECURITY;
CREATE POLICY stripe_api_calls_rls ON stripe_api_calls
  USING (tenant_id = current_setting('auth.tenant_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
REVOKE UPDATE, DELETE ON stripe_api_calls FROM cyberos_app;
-- cyberos_pruner is the existing scheduled-job role defined in TASK-AUTH-003 §3.4;
-- its DELETE grant is scoped to this table for TTL pruning (DEC-807):
GRANT DELETE ON stripe_api_calls TO cyberos_pruner;

-- 0008_stripe_event_dispatch_log.sql
CREATE TABLE stripe_event_dispatch_log (
  id BIGSERIAL PRIMARY KEY,
  tenant_id UUID NOT NULL,
  stripe_event_id TEXT NOT NULL,
  event_type TEXT NOT NULL,
  dispatch_status TEXT NOT NULL CHECK (dispatch_status IN ('dispatched','duplicate','failed')),
  failure_reason TEXT,
  dispatched_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE(tenant_id, stripe_event_id)
);
ALTER TABLE stripe_event_dispatch_log ENABLE ROW LEVEL SECURITY;
CREATE POLICY sed_log_rls ON stripe_event_dispatch_log
  USING (tenant_id = current_setting('auth.tenant_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
REVOKE UPDATE, DELETE ON stripe_event_dispatch_log FROM cyberos_app;

-- 0010_stripe_price_map.sql
CREATE TABLE stripe_price_map (
  residency TEXT NOT NULL CHECK (residency IN ('sg-1','eu-1','us-1','vn-1')),
  currency billing_currency_enum NOT NULL,
  plan_tier plan_tier NOT NULL,
  axis TEXT NOT NULL CHECK (axis IN ('base','overage_api_calls','overage_ai_tokens','overage_storage','overage_seats')),
  stripe_price_id TEXT NOT NULL,
  synced_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (residency, currency, plan_tier, axis)
);
REVOKE UPDATE, DELETE ON stripe_price_map FROM cyberos_app;
-- Note: stripe_price_map is global config; no RLS (read-all-tenants).
```

### 3.2 Rust types

```rust
// services/ten/src/billing/stripe/price_catalog.rs
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Currency { Vnd, Usd, Eur, Sgd, Gbp }

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum PlanTier { Starter, Team, Enterprise }

#[derive(Copy, Clone, Debug)]
pub struct PriceEntry {
    pub currency: Currency,
    pub tier: PlanTier,
    pub amount_minor: i64,
}

pub const PRICE_CATALOG: [PriceEntry; 12] = [
    PriceEntry { currency: Currency::Usd, tier: PlanTier::Starter,    amount_minor: 1_900 },
    PriceEntry { currency: Currency::Usd, tier: PlanTier::Team,       amount_minor: 9_900 },
    PriceEntry { currency: Currency::Usd, tier: PlanTier::Enterprise, amount_minor: 99_900 },
    PriceEntry { currency: Currency::Eur, tier: PlanTier::Starter,    amount_minor: 1_900 },
    PriceEntry { currency: Currency::Eur, tier: PlanTier::Team,       amount_minor: 9_900 },
    PriceEntry { currency: Currency::Eur, tier: PlanTier::Enterprise, amount_minor: 99_900 },
    PriceEntry { currency: Currency::Sgd, tier: PlanTier::Starter,    amount_minor: 2_500 },
    PriceEntry { currency: Currency::Sgd, tier: PlanTier::Team,       amount_minor: 13_000 },
    PriceEntry { currency: Currency::Sgd, tier: PlanTier::Enterprise, amount_minor: 130_000 },
    PriceEntry { currency: Currency::Gbp, tier: PlanTier::Starter,    amount_minor: 1_500 },
    PriceEntry { currency: Currency::Gbp, tier: PlanTier::Team,       amount_minor: 7_900 },
    PriceEntry { currency: Currency::Gbp, tier: PlanTier::Enterprise, amount_minor: 79_900 },
];

pub fn price_for(currency: Currency, tier: PlanTier) -> Option<&'static PriceEntry> {
    PRICE_CATALOG.iter().find(|e| e.currency == currency && e.tier == tier)
}

// services/ten/src/billing/stripe/api_client.rs
pub struct StripeClient {
    residency: Residency,
    api_key: SecretString,  // KMS-loaded per DEC-801
    http: reqwest::Client,
    idempotency_repo: IdempotencyRepo,
}

impl StripeClient {
    pub async fn call<T: serde::de::DeserializeOwned>(
        &self,
        method: reqwest::Method,
        endpoint: &str,
        idempotency_key: &str,
        body: &impl serde::Serialize,
    ) -> Result<T, StripeError> {
        // Check idempotency cache; if hit, return cached response_body
        // Otherwise: POST with Idempotency-Key header, persist to stripe_api_calls
        // Honour Retry-After on 429; exponential backoff on 5xx (1,2,4,8,16s)
        // After 5 retries: memory audit `ten.stripe_api_call_failed` + return Err
    }
}

// services/ten/src/billing/dunning.rs
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum DunningState { Ok, Retry1, Retry2, Retry3, Suspended }

impl DunningState {
    pub fn advance_on_payment_failed(self) -> DunningState {
        match self {
            DunningState::Ok       => DunningState::Retry1,
            DunningState::Retry1   => DunningState::Retry2,
            DunningState::Retry2   => DunningState::Retry3,
            DunningState::Retry3   => DunningState::Suspended,
            DunningState::Suspended => DunningState::Suspended,  // terminal
        }
    }
    pub fn reset_on_payment_succeeded(self) -> DunningState {
        DunningState::Ok
    }
}
```

### 3.3 REST endpoints

```text
POST   /v1/admin/tenants/{id}/billing/refund    (CFO-only)
GET    /v1/admin/tenants/{id}/billing            (tenant_admin or CFO)
```

Refund request body:
```json
{ "stripe_charge_id": "ch_3OabcXYZ...", "amount_minor": 9900, "currency": "USD",
  "reason": "requested_by_customer" }
```

Refund response (201):
```json
{ "refund_id": "re_3PdefABC...", "status": "succeeded",
  "expected_settlement_at": "2026-05-22T00:00:00Z" }
```

### 3.4 CLI

```text
cyberos-ten stripe-sync-prices [--dry-run | --apply] [--residency sg-1|eu-1|us-1]
```

---

## §4 — Acceptance criteria

1. **Currency lock at provisioning** — provisioning a tenant with `--billing-currency USD` sets `tenants.billing_currency='USD'` and `billing_rail='stripe'`; subsequent `UPDATE tenants SET billing_currency='EUR'` raises trigger error `billing_currency_immutable`.
2. **Stripe Customer lazy + idempotent** — first plan_change on a stripe-rail tenant creates one Stripe Customer; concurrent first plan_change attempts produce exactly one Customer (race-safe); re-invoking ensure_customer is a no-op.
3. **Subscription create with metered overage items** — `ensure_subscription(tenant, Team)` creates a Stripe Subscription with one base item (Team price) and 4 metered items (one per overage axis); `tenants.stripe_subscription_id` populated.
4. **Plan upgrade prorates immediately** — plan_change Starter→Team triggers Stripe `POST /v1/subscriptions/{id}` with `proration_behavior=create_prorations`; next invoice line items include proration amount.
5. **Plan downgrade defers via Schedule** — plan_change Team→Starter triggers Stripe Subscription Schedule with `effective_at = next_period_boundary` and `proration_behavior=none`; current period unchanged.
6. **Overage push at period_close** — TASK-TEN-004 period_close with `api_calls actual=550_000, cap=500_000` triggers Stripe usage_record with `quantity=50_000` and idempotency_key `ten.<tid>.overage_push.api_calls.<period_end_unix>`.
7. **Dispatcher idempotent on duplicate webhook** — same `(tenant_id, stripe_event_id)` arriving twice via NATS produces one dispatch + one `dispatch_status='duplicate'` log row + no double-side-effect.
8. **Dunning state machine** — three `invoice.payment_failed` events advance `dunning_state` ok→retry_1→retry_2→retry_3; fourth advances to `suspended` + triggers TASK-TEN-104 suspend transition.
9. **Un-suspend on payment_succeeded** — tenant in `suspended` state + `invoice.payment_succeeded` event → `dunning_state='ok'` + TASK-TEN-104 resume transition.
10. **Refund CFO-only** — POST refund with non-cfo subject returns 403 `forbidden`; with cfo subject returns 201 + emits sev-1 memory row `ten.stripe_refund_issued`.
11. **Refund amount cap** — POST refund with `amount_minor > original_charge_amount` returns 400 `refund_exceeds_charge`.
12. **Founder skip** — founder tenant plan_change → no Stripe API call invoked; sev-3 memory row `ten.stripe_founder_skip` emitted.
13. **VND tenant cross-rail rejection** — VND tenant invoking Stripe rail returns 400 `wrong_billing_rail` + no Stripe API call.
14. **API client retry on 5xx** — Stripe 503 response triggers backoff 1s, 2s, 4s, 8s, 16s; sixth attempt fails permanently + emits `ten.stripe_api_call_failed` sev-2.
15. **API client honours Retry-After on 429** — Stripe 429 + `Retry-After: 30` causes client to sleep 30s before retry.
16. **Price catalog cardinality** — `stripe_price_catalog_cardinality_test` asserts `PRICE_CATALOG.len() == 12` and every entry has unique `(currency, tier)`.
17. **Residency API key routing** — sg-1 tenant Stripe call uses `STRIPE_API_KEY_SG`; eu-1 uses `STRIPE_API_KEY_EU`; us-1 uses `STRIPE_API_KEY_US`; mis-route detected by sentinel-test calling against per-residency Stripe test accounts.
18. **Idempotency cache hit returns cached response** — invoking same `Idempotency-Key` twice within 7d returns cached `response_body_sha256` without re-calling Stripe.
19. **PII scrubbing in memory chain** — refund audit row carries `billing_contact_email_hash16` not raw email; reason field scrubbed via TASK-MEMORY-111.
20. **W3C trace_id propagated** — every Stripe audit row carries 32-char hex `trace_id` matching the upstream caller's traceparent header.

---

## §5 — Verification

### 5.1 `stripe_currency_lock_test.rs`

```rust
#[tokio::test]
async fn billing_currency_immutable() {
    let ctx = TestContext::new().await;
    let tenant_id = provision_tenant(&ctx, "acme", "USD").await;
    let res = sqlx::query("UPDATE tenants SET billing_currency='EUR' WHERE id=$1")
        .bind(tenant_id).execute(&ctx.pool).await;
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("billing_currency_immutable"));
}
```

### 5.2 `stripe_customer_idempotency_test.rs`

```rust
#[tokio::test]
async fn ensure_customer_idempotent_under_concurrency() {
    let ctx = TestContext::new().await;
    let tenant_id = provision_tenant(&ctx, "acme", "USD").await;
    let client = StripeClient::test_mode(&ctx);

    let (r1, r2, r3) = tokio::join!(
        ensure_customer(&ctx, &client, tenant_id),
        ensure_customer(&ctx, &client, tenant_id),
        ensure_customer(&ctx, &client, tenant_id),
    );
    let id1 = r1.unwrap(); let id2 = r2.unwrap(); let id3 = r3.unwrap();
    assert_eq!(id1, id2); assert_eq!(id2, id3);

    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM tenants WHERE stripe_customer_id IS NOT NULL AND id=$1")
        .bind(tenant_id).fetch_one(&ctx.pool).await.unwrap();
    assert_eq!(count, 1);
}
```

### 5.3 `stripe_dunning_state_machine_test.rs`

```rust
#[tokio::test]
async fn dunning_advances_then_suspends() {
    let ctx = TestContext::new().await;
    let tenant_id = provision_tenant(&ctx, "acme", "USD").await;
    activate_subscription(&ctx, tenant_id, PlanTier::Team).await;

    for expected in &[DunningState::Retry1, DunningState::Retry2, DunningState::Retry3] {
        dispatch_invoice_payment_failed(&ctx, tenant_id).await;
        assert_eq!(load_dunning(&ctx, tenant_id).await, *expected);
    }
    dispatch_invoice_payment_failed(&ctx, tenant_id).await;
    assert_eq!(load_dunning(&ctx, tenant_id).await, DunningState::Suspended);

    let status: String = sqlx::query_scalar("SELECT status::text FROM tenants WHERE id=$1")
        .bind(tenant_id).fetch_one(&ctx.pool).await.unwrap();
    assert_eq!(status, "suspended");

    let audit_rows = load_memory_rows_for_tenant(&ctx, tenant_id).await;
    assert!(audit_rows.iter().any(|r| r.kind == "ten.tenant_billing_suspended"));
}
```

### 5.4 `stripe_founder_skip_test.rs`

```rust
#[tokio::test]
async fn founder_tenant_never_calls_stripe() {
    let ctx = TestContext::new().await;
    let founder = provision_founder_tenant(&ctx).await;
    let stripe_mock = ctx.stripe_mock();

    change_plan(&ctx, founder, PlanTier::Team).await.unwrap();
    assert_eq!(stripe_mock.calls(), 0);

    let audit = load_memory_rows_for_tenant(&ctx, founder).await;
    assert!(audit.iter().any(|r| r.kind == "ten.stripe_founder_skip"));
}
```

### 5.5 `stripe_api_retry_test.rs`

```rust
#[tokio::test]
async fn five_xx_triggers_exponential_backoff() {
    let ctx = TestContext::new().await;
    let mock = ctx.stripe_mock_returning(503).times(5).then(200);
    let client = StripeClient::with_mock(&ctx, mock);
    let start = Instant::now();

    let res = client.call_subscription_create(/* ... */).await;
    let elapsed = start.elapsed();

    assert!(res.is_ok());
    assert!(elapsed >= Duration::from_secs(1 + 2 + 4 + 8 + 16));
    assert!(elapsed < Duration::from_secs(60));
}

#[tokio::test]
async fn six_consecutive_5xx_emits_sev2() {
    let ctx = TestContext::new().await;
    let mock = ctx.stripe_mock_returning(503).times(6);
    let client = StripeClient::with_mock(&ctx, mock);

    let res = client.call_subscription_create(/* ... */).await;
    assert!(matches!(res, Err(StripeError::Unavailable { .. })));
    let audit = load_memory_rows(&ctx).await;
    assert!(audit.iter().any(|r| r.kind == "ten.stripe_api_call_failed" && r.severity == 2));
}
```

### 5.6 `stripe_price_catalog_cardinality_test.rs`

```rust
#[test]
fn price_catalog_has_12_entries_one_per_currency_tier() {
    assert_eq!(PRICE_CATALOG.len(), 12);
    let mut seen = std::collections::HashSet::new();
    for e in &PRICE_CATALOG {
        assert!(seen.insert((e.currency, e.tier)), "duplicate ({:?},{:?})", e.currency, e.tier);
        assert!(e.amount_minor > 0);
    }
    let currencies = [Currency::Usd, Currency::Eur, Currency::Sgd, Currency::Gbp];
    for c in &currencies {
        for t in &[PlanTier::Starter, PlanTier::Team, PlanTier::Enterprise] {
            assert!(seen.contains(&(*c, *t)), "missing ({:?},{:?})", c, t);
        }
    }
}
```

### 5.7 `stripe_refund_cfo_only_test.rs`

```rust
#[tokio::test]
async fn refund_requires_cfo_role() {
    let ctx = TestContext::new().await;
    let tenant = provision_tenant(&ctx, "acme", "USD").await;
    let tenant_admin_token = mint_jwt(&ctx, tenant, "tenant_admin");

    let res = post_refund(&ctx, tenant, &tenant_admin_token, 1900).await;
    assert_eq!(res.status(), 403);

    let cfo_token = mint_jwt(&ctx, tenant, "chief-financial-officer");
    let res = post_refund(&ctx, tenant, &cfo_token, 1900).await;
    assert_eq!(res.status(), 201);
    let audit = load_memory_rows_for_tenant(&ctx, tenant).await;
    assert!(audit.iter().any(|r| r.kind == "ten.stripe_refund_issued" && r.severity == 1));
}

#[tokio::test]
async fn refund_amount_cannot_exceed_charge() {
    let ctx = TestContext::new().await;
    let tenant = provision_tenant(&ctx, "acme", "USD").await;
    let cfo_token = mint_jwt(&ctx, tenant, "chief-financial-officer");
    seed_charge(&ctx, tenant, 1900).await;

    let res = post_refund(&ctx, tenant, &cfo_token, 9999).await;
    assert_eq!(res.status(), 400);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["error"], "refund_exceeds_charge");
}
```

### 5.8 `stripe_dispatcher_idempotency_test.rs`

```rust
#[tokio::test]
async fn duplicate_webhook_dispatches_once() {
    let ctx = TestContext::new().await;
    let tenant = provision_tenant(&ctx, "acme", "USD").await;
    activate_subscription(&ctx, tenant, PlanTier::Team).await;

    let event_id = "evt_3OabcXYZ";
    publish_stripe_event(&ctx, tenant, event_id, "invoice.payment_succeeded").await;
    publish_stripe_event(&ctx, tenant, event_id, "invoice.payment_succeeded").await;

    let dispatched: Vec<(String, String)> = sqlx::query_as(
        "SELECT stripe_event_id, dispatch_status FROM stripe_event_dispatch_log WHERE tenant_id=$1 ORDER BY id"
    ).bind(tenant).fetch_all(&ctx.pool).await.unwrap();

    assert_eq!(dispatched.len(), 2);
    assert_eq!(dispatched[0].1, "dispatched");
    assert_eq!(dispatched[1].1, "duplicate");
}
```

### 5.9 `stripe_overage_push_test.rs`

```rust
#[tokio::test]
async fn period_close_pushes_overage_within_one_hour() {
    let ctx = TestContext::new().await;
    let tenant = provision_tenant(&ctx, "acme", "USD").await;
    activate_subscription(&ctx, tenant, PlanTier::Team).await;
    seed_metering(&ctx, tenant, "api_calls", 550_000).await;

    let period_end = chrono::Utc::now().timestamp();
    run_period_close(&ctx, tenant, period_end).await;

    let calls = ctx.stripe_mock().calls_to("usage_records");
    assert_eq!(calls.len(), 4); // one per axis; only api_calls is over cap
    let api_call = calls.iter().find(|c| c.body["quantity"] == 50_000).unwrap();
    let key = api_call.headers["Idempotency-Key"].as_str();
    assert_eq!(key, format!("ten.{tenant}.overage_push.api_calls.{period_end}"));
}
```

### 5.10 `stripe_residency_apikey_routing_test.rs`

```rust
#[tokio::test]
async fn sg_tenant_uses_sg_api_key() {
    let ctx = TestContext::new().await;
    let sg_tenant = provision_tenant_with_residency(&ctx, "acme", "SGD", "sg-1").await;
    let calls = ctx.stripe_mock().calls();
    ensure_customer(&ctx, &client_for(&ctx, "sg-1"), sg_tenant).await.unwrap();
    assert_eq!(calls.last().unwrap().auth, ctx.kms_decrypt("STRIPE_API_KEY_SG"));
}
```

---

## §6 — Implementation skeleton

(API contract in §3 is the skeleton. Additional orchestrator wiring below.)

### 6.1 plan_change handler wire-up (modified `services/ten/src/handlers/plan_change.rs`)

```rust
// AFTER the existing tenant_plan_history INSERT commits:
if tenant.billing_rail == BillingRail::Stripe {
    let client = StripeClient::for_residency(tenant.residency).await?;
    billing::stripe::subscription::push_plan_change(
        &ctx, &client, tenant_id, history_id, from_tier, to_tier
    ).await?;
}
// For internal (founder) and vietqr_momo_zalo rails, no-op or TASK-TEN-102 path.
```

### 6.2 period_close hook wire-up (modified `services/metering/src/handlers/period_close.rs`)

```rust
// AFTER aggregation completes, for stripe-rail tenants only:
if tenant.billing_rail == BillingRail::Stripe {
    spawn_with_deadline(Duration::from_secs(3600), async move {
        billing::stripe::overage::push_overage_for_period(
            &ctx, tenant_id, period_end_unix
        ).await
    });
}
```

### 6.3 NATS dispatcher (new `services/ten/src/billing/stripe/dispatch.rs`)

```rust
pub async fn run_dispatcher(ctx: AppCtx) {
    let sub = ctx.nats.subscribe("tenant.*.ten.stripe.*").await.unwrap();
    while let Some(msg) = sub.next().await {
        let event: StripeWebhookEvent = serde_json::from_slice(&msg.payload).unwrap();
        let dispatched = ctx.repo.stripe_event_dispatch_log
            .upsert_idempotent(event.tenant_id, event.stripe_event_id, &event.event_type).await;
        if dispatched.is_new {
            match event.event_type.as_str() {
                "invoice.payment_succeeded" => handle_payment_succeeded(&ctx, event).await,
                "invoice.payment_failed"    => handle_payment_failed(&ctx, event).await,
                "invoice.finalized"         => handle_invoice_finalized(&ctx, event).await,
                "customer.subscription.updated"   => handle_subscription_updated(&ctx, event).await,
                "customer.subscription.deleted"   => handle_subscription_deleted(&ctx, event).await,
                _ => {}  // unrelated events ignored
            }
        }
        // duplicate path: log only; no side effect
    }
}
```

---

## §7 — Dependencies

**Upstream (depends_on):**
- **TASK-INV-003** Stripe webhook handler — signature-verified inbound events; bridges to TEN via NATS publish on relevant kinds.
- **TASK-TEN-002** Plan tiers + plan_change handler — TIER_PRICE_CENTS_MONTHLY constants + plan_change trigger point.
- **TASK-TEN-004** 4-axis metering + period_close — overage push hook + per-axis usage quantities.

**Cross-module (related_tasks):**
- **TASK-TEN-001** Provisioning — billing_currency captured at provisioning; billing_rail derived.
- **TASK-TEN-101** Self-serve signup — depends on TEN-003 being able to bill (TEN-003 blocks TEN-101).
- **TASK-TEN-102** VND domestic rail — parallel rail; TEN-003 ships first to establish rail abstraction (TEN-003 blocks TEN-102).
- **TASK-TEN-103** 4-residency provisioning — per-residency Stripe API key map consumed by TEN-003.
- **TASK-TEN-104** Tenant lifecycle (suspended/terminating) — dunning advances trigger TASK-TEN-104 status transitions.
- **TASK-INV-006** Cash application — reconciles Stripe receipts against TEN subscription invoices.
- **TASK-AUTH-101** RBAC catalogue — `cfo` and `tenant_admin` role gates.
- **TASK-AI-003** memory audit-row bridge — 11 new kinds register here.
- **TASK-MEMORY-111** PII scrubbing ruleset — billing_contact_email + reason text.
- **TASK-OBS-007** Auto-runbook — sev-1/sev-2 dunning + API-failure alerts route to CHAT/PagerDuty.

**Downstream (blocks):**
- **TASK-TEN-102** — VND rail follows the abstraction this task establishes.
- **TASK-TEN-101** — self-serve signup requires automated billing rail.

---

## §8 — Example payloads

### 8.1 `ten.stripe_customer_created` memory row

```json
{
  "kind": "ten.stripe_customer_created",
  "severity": 2,
  "tenant_id": "8a2f...",
  "actor_id": "system.ten.billing",
  "trace_id": "0af7651916cd43dd8448eb211c80319c",
  "occurred_at": "2026-05-17T09:14:32.847Z",
  "payload": {
    "stripe_customer_id": "cus_3OabcXYZ",
    "currency": "USD",
    "residency": "us-1",
    "billing_contact_email_hash16": "f8a1b2c3d4e5f607",
    "stripe_api_call_idempotency_key": "ten.8a2f....customer_create.v1"
  }
}
```

### 8.2 Refund request

```json
{
  "stripe_charge_id": "ch_3OabcXYZ012345",
  "amount_minor": 9900,
  "currency": "USD",
  "reason": "requested_by_customer"
}
```

### 8.3 Refund response (201)

```json
{
  "refund_id": "re_3PdefABC987654",
  "status": "succeeded",
  "expected_settlement_at": "2026-05-22T00:00:00Z",
  "audit_row_chain_hash": "9c4e7a8b6d2f1e3a5b9c7d4e6f8a2b1c4d7e9f3a6b8c2d5e7f9a1b3c5d7e9f1a"
}
```

### 8.4 `ten.stripe_overage_pushed` memory row

```json
{
  "kind": "ten.stripe_overage_pushed",
  "severity": 3,
  "tenant_id": "8a2f...",
  "actor_id": "system.metering.period_close",
  "trace_id": "0af7651916cd43dd8448eb211c80319c",
  "occurred_at": "2026-06-01T00:03:12.118Z",
  "payload": {
    "axis": "api_calls",
    "overage_quantity": 50000,
    "period_end_unix": 1748736000,
    "stripe_usage_record_id": "ur_3Q1zxyABC",
    "stripe_subscription_item_id": "si_3Pabc456",
    "stripe_api_call_idempotency_key": "ten.8a2f....overage_push.api_calls.1748736000"
  }
}
```

### 8.5 `GET /v1/admin/tenants/{id}/billing` response

```json
{
  "tenant_id": "8a2f...",
  "billing_rail": "stripe",
  "billing_currency": "USD",
  "stripe_customer_id": "cus_3OabcXYZ",
  "stripe_subscription_id": "sub_3Pabc456",
  "current_plan_tier": "team",
  "dunning_state": "ok",
  "billing_cycle_anchor": "2026-05-17T00:00:00Z",
  "next_invoice_date": "2026-06-17T00:00:00Z",
  "recent_invoices": [
    { "stripe_invoice_id": "in_3PXY...", "amount_minor": 9900, "currency": "USD",
      "status": "paid", "period_start": "2026-04-17T00:00:00Z", "period_end": "2026-05-17T00:00:00Z" }
  ],
  "recent_dunning_events": []
}
```

### 8.6 NATS subject example (TASK-INV-003 → TEN dispatcher bridge)

```text
Subject: tenant.acme-sg.ten.stripe.invoice.payment_succeeded
Payload:
{
  "tenant_id": "8a2f...",
  "stripe_event_id": "evt_3OabcXYZ",
  "event_type": "invoice.payment_succeeded",
  "stripe_invoice_id": "in_3PXY...",
  "stripe_customer_id": "cus_3OabcXYZ",
  "amount_paid_minor": 9900,
  "currency": "USD",
  "received_at": "2026-05-17T09:14:33.001Z"
}
```

---

## §9 — Open questions

All resolved for slice 2. Deferred to later slices:

- **Deferred:** Stripe Tax integration for EU VAT — slice 3, task-TEN-1xx (placeholder — not yet specified).
- **Deferred:** Multi-currency support per tenant (currency change without new tenant) — out-of-scope per DEC-798; new tenant + manual migration is the documented path.
- **Deferred:** Stripe Connect / marketplace flows for task-PORTAL-XXX — out-of-scope; client tenants not yet billed-through their portal (P4 milestone).
- **Deferred:** Real-time invoice preview before plan_change commits — slice 3 enhancement; for now CFO uses Stripe Dashboard.
- **Deferred:** SEPA Direct Debit / BACS / ACH local-payment rails — slice 3+ (task-TEN-2xx); slice 2 is card-only via Stripe Payment Intents.
- **Deferred:** Annual billing cycle (slice 2 is monthly only) — slice 3, task-TEN-1xx.
- **Deferred:** Coupon / promo code support — slice 3, task-TEN-1xx.
- **Deferred:** Reconciliation job comparing TEN dunning_state vs Stripe subscription.status (slice 3 per §11.13).
- **Deferred:** Hourly overage push (vs once-per-period) — slice 3 per §11.10; reduces revenue-recognition lag.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Stripe API 5xx persistent (> 5 retries) | api_client error counter; OTel span error rate | Operation returns 502 to caller; sev-2 memory row `ten.stripe_api_call_failed` | Manual retry via `cyberos-ten stripe-retry-failed --since <ts>` (operator CLI in slice 3); for slice 2, CFO re-invokes the operation from Stripe Dashboard manually |
| Stripe webhook arrives before TEN customer record exists | dispatcher lookup `tenants WHERE stripe_customer_id=$1` returns 0 rows | sev-2 alert `ten.stripe_orphan_webhook`; dispatch_status='failed' with reason='no_matching_tenant'; webhook ack'd to Stripe (200) so no retry storm | Operator investigates via Stripe Dashboard; may indicate test-mode webhook hit prod or customer manually deleted |
| Duplicate Stripe Customer for one tenant (race lost) | partial unique index `uniq_stripe_customer` rejects second INSERT | Second concurrent call sees Postgres unique violation; api_client retries the SELECT path; returns existing customer_id | Inherent race-safety — partial unique index + SELECT FOR UPDATE pattern prevents persistent duplicates |
| billing_currency mismatch between webhook invoice and tenant record | dispatch handler checks `event.currency == tenant.billing_currency` | sev-1 alert `ten.stripe_currency_drift`; webhook ack'd; dispatch_status='failed' | Manual reconciliation; likely indicates wrong-residency Stripe key used at customer creation — operator audit |
| Idempotency-Key collision on retry within 7d | stripe_api_calls table lookup hits | Cached response returned; no Stripe call; OTel span attribute `idempotency.hit=true` | Inherent — idempotency is the desired behaviour |
| Idempotency cache expired (7d) but Stripe still has key (24h) | api_client makes new call; Stripe returns previous response | Stripe response cached fresh; behaviour identical to original | Inherent — Stripe's idempotency window > ours; safe |
| Subscription Schedule downgrade conflicts with later upgrade | plan_change handler check for existing Subscription Schedule | New plan_change cancels prior Subscription Schedule via Stripe API; emits sev-2 `ten.stripe_schedule_cancelled` | Operator informed; if upgrade after pending downgrade, the upgrade takes effect immediately + downgrade cancelled |
| Overage push failure after period_close (1h window exhausted) | overage push retry loop hits 24h deadline | sev-1 alert `ten.stripe_overage_push_failed`; period_close marked as `overage_push_pending` | CFO manually pushes via Stripe Dashboard usage_records API; subsequent period_close reconciles |
| NATS dispatcher fails to consume (crash, network) | NATS lag metric; OBS dashboard | Backlog grows; webhooks ack'd to Stripe but not actioned in TEN | NATS replay from durable subject (slice 2 uses NATS JetStream with 7d retention); dispatcher resumes from last acked offset |
| Stripe Price ID drift (rev'd in Stripe Dashboard without sync) | next subscription_create sees mismatch between catalog amount and Stripe price.unit_amount | sev-2 alert `ten.stripe_price_drift`; operation proceeds with Stripe's value (Stripe is source of truth at API call time); memory row notes drift | CFO re-runs `cyberos-ten stripe-sync-prices --apply` to align catalog → Stripe; never the other direction |
| KMS unavailable when api_client constructor runs | KMS client timeout 5s | Constructor returns Err; calling handler returns 503; sev-2 memory row `ten.stripe_kms_unavailable` | Inherent KMS retry on next request; if persistent, AWS KMS incident — page on-call |
| Tenant terminated mid-period (TASK-TEN-104) with active Stripe subscription | TASK-TEN-104 termination handler calls `services/ten/src/billing/stripe/subscription.rs::cancel(tenant_id)` | Stripe subscription cancelled immediately (NOT cancel_at_period_end — termination is hard); pro-rata credit calculated by Stripe; emits `ten.stripe_subscription_cancelled` sev-1 | TASK-TEN-104 termination flow handles refund decision (refund unused period vs. forfeit per ToS) |
| Webhook secret rotation mid-flight (TASK-INV-003 60s overlap) | Stripe sends event during overlap; TASK-INV-003 accepts both | Both old and new secret valid; no impact on TEN dispatcher (sees signed events either way) | Inherent overlap window |
| Founder tenant accidentally flipped to non-internal rail | guard at api_client checks `tenant.billing_rail` | api_client returns Err `founder_cannot_stripe`; sev-1 memory row | Manual SQL fix + DEC entry explaining how the flip happened; trigger added in slice 3 to prevent UPDATE on is_founder_tenant |
| VND tenant configured stripe_customer_id by mistake | API client guard checks `billing_currency != VND` | Returns 400 `wrong_billing_rail`; no Stripe call | Manual SQL cleanup + DEC entry; trigger added in slice 3 to prevent stripe_customer_id population on VND tenants |
| Dunning state machine drift (Stripe state ≠ our state) | reconciliation job (slice 3) compares Stripe subscription.status with tenants.dunning_state | sev-2 alert `ten.stripe_state_drift`; manual reconciliation required | CFO consults Stripe Dashboard + invokes `cyberos-ten dunning-reconcile <tenant>` (slice 3) |
| Refund attempted on already-refunded charge | Stripe returns 400 `charge_already_refunded` | api_client returns Err; refund handler returns 400 `refund_failed` with Stripe error | CFO consults Stripe Dashboard; possibly attempting partial refund on already-fully-refunded — Stripe is source of truth |
| Stripe customer email mismatch on update (PII drift) | dispatcher detects `customer.updated` with email change | sev-3 memory row `ten.stripe_customer_email_changed`; tenants.billing_contact_email NOT auto-updated | Operator review; manual UPDATE if intentional |
| Subscription items list drift (extra metered items in Stripe) | reconciliation comparing our PRICE_CATALOG against Stripe subscription.items | sev-2 alert `ten.stripe_items_drift` | Operator runs `cyberos-ten stripe-rebuild-subscription <tenant>` (slice 3) to align items |

---

## §11 — Implementation notes

**§11.1** `async-stripe` crate is well-maintained but pin to exact version; Stripe API versions are date-stamped — our `Stripe-Version` header sent on every call (currently `2024-06-20` baseline). Bumping the date is an ADR with regression-test sweep.

**§11.2** The `billing/stripe/api_client.rs` retry logic uses the `backoff` crate's `ExponentialBackoff` builder; we customise to honour `Retry-After` over the backoff schedule when set (Stripe's `Retry-After` is gospel).

**§11.3** Idempotency keys MAX 255 chars per Stripe; UUID v4 + colon-delimited operation tag fits comfortably (`ten.<36-char-uuid>.<operation>.<suffix>` ~80 chars).

**§11.4** `stripe_event_dispatch_log` is strictly INSERT-only at the `cyberos_app` role (REVOKE UPDATE, DELETE per §3.1 + task-audit skill rule 12). `dispatch_status` is set at INSERT time as a single-pass write — the handler computes status (`dispatched`/`duplicate`/`failed`) before INSERT based on idempotency lookup and dispatch result. Retry-and-update for transient failures lands in slice 3 (task-TEN-2xx) with a separate `stripe_event_dispatch_retry` table that follows the correction_to pattern (task-audit skill rule 11 derivative).

**§11.5** The `cancel_at_period_end=true` cancellation primitive (§1 #10 + DEC-803) is the *normal* deactivation path; hard cancellation comes only from TASK-TEN-104 termination, which explicitly invokes Stripe's `DELETE /v1/subscriptions/{id}?prorate=true`.

**§11.6** Founder tenant detection is `tenants.is_founder_tenant=true`; the guard is at api_client entry, not per-handler, so all Stripe code paths are protected uniformly.

**§11.7** The `stripe_price_map` table is intentionally GLOBAL (no RLS) because Stripe Price IDs are not tenant-scoped — they're per-residency-Stripe-account, shared across all tenants in that residency. Tenant isolation is enforced by the (tenant.residency → API key) routing.

**§11.8** Per-currency Stripe Price IDs vs per-residency: GBP exists at slice 2 even though no residency defaults to GBP — it's an explicit override for eu-1 tenants who want GBP billing (e.g., UK customers post-Brexit). The catalog ships GBP entries; `stripe-sync-prices` populates them in the eu-1 Stripe account.

**§11.9** SubscriptionSchedule (§1 #8 downgrade path) outlives the subscription itself — even if the subscription is hard-cancelled, the schedule remains until executed or cancelled. TASK-TEN-104 termination must cancel any active SubscriptionSchedule for the tenant before deleting the subscription (else Stripe errors on the schedule's target).

**§11.10** The 1-hour overage push window (DEC-810) aligns with Stripe's billing-cycle-close grace period; pushing later means the overage lands on the NEXT period's invoice, which is acceptable but adds a 30-day delay to revenue recognition. Slice 3 adds a finer-grained push (hourly) for low-latency reporting.

**§11.11** `billing_contact_email` is collected at provisioning (TASK-TEN-001 + this task's modification of `tenant_create.rs` to accept the field). For backward compatibility with already-provisioned tenants (TASK-TEN-001 shipped before this), migration `0006` allows NULL initially with a 7-day operator backfill window before the NOT NULL constraint activates via a follow-up migration in slice 3.

**§11.12** The dunning state machine deliberately treats `Suspended → Suspended` on additional payment failures as a no-op (no further state advancement). Once suspended, further failures don't escalate; recovery via successful payment is the only exit. This matches operational reality: a suspended tenant's billing is "in a state of repair" — further failure signals don't add information.

**§11.13** Reconciliation jobs (slice 3) will compare our `tenants.stripe_subscription_id`-derived state against Stripe's live subscription.status and emit drift alerts. Slice 2 relies on webhook-driven state being eventually consistent.

**§11.14** The `Stripe-Version` header pinning (§11.1) is at the api_client level; per-call versioning override is NOT supported in slice 2 (some Stripe APIs require older versions for backward-compat — we'll cross that bridge when we hit it).

**§11.15** Audit row severities use the task-audit skill severity convention: sev-1 = revenue/security incident requiring CFO/operator attention; sev-2 = operational issue requiring eng response; sev-3 = informational/forensic. The 11-kind list at §1 #19 maps every kind to its severity.

**§11.16** The price catalog amounts (USD $19/$99/$999; SGD higher reflecting Singapore PPP) are slice 2 defaults; CFO reviews + revs via DEC entries. The CI test asserts cardinality (12) and uniqueness, not specific amounts — amount changes don't break the test.

**§11.17** `cyberos-ten stripe-sync-prices --dry-run` outputs JSON listing every (currency, tier) entry with its catalog amount and current Stripe Price ID (or `<missing>` if not yet synced). CI runs this nightly with `--dry-run` and alerts if any entry shows `<missing>` in any residency.

**§11.18** The NATS dispatcher (§6.3) runs as a separate binary `cyberos-ten-stripe-dispatcher` deployed alongside the TEN service; it scales horizontally (each instance consumes a slice of the NATS subject). Idempotency at the dispatch log means duplicates across instances are safe.

**§11.19** The handler's plan_change path (§6.1) invokes Stripe synchronously rather than async-queueing. The CFO-visible operation must reflect Stripe's reality quickly; the 200ms p95 target for Stripe API calls makes sync acceptable. Async-queue alternative considered but rejected as over-engineering for slice 2.

**§11.20** When `ensure_subscription` creates a subscription with 5 items (1 base + 4 overage), the Stripe Subscription Item IDs are persisted in `tenants.stripe_subscription_items_map JSONB DEFAULT '{}'` (added via migration `0006` per §3.1). JSON shape: `{ "base": "si_3Pabc", "overage_api_calls": "si_3Pdef", "overage_ai_tokens": "si_3Pghi", "overage_storage": "si_3Pjkl", "overage_seats": "si_3Pmno" }`. The overage push handler reads this map to locate the correct subscription_item_id per axis before calling `POST /v1/subscription_items/{id}/usage_records`.

**§11.21** Tests use `wiremock` to simulate Stripe responses (`ctx.stripe_mock()`); integration tests against Stripe test-mode are SEPARATE (run nightly, not in PR-CI) because Stripe rate-limits test-mode harshly. The wiremock-based tests validate our client logic; the nightly Stripe-test-mode integration validates end-to-end.

**§11.22** The 24h Stripe idempotency window vs our 7d cache: if Stripe expires the key but we still have it cached, our cache returns the cached response without consulting Stripe. This is desired — if Stripe has GC'd the key, it would re-process the call as new; our cache prevents that double-process.

**§11.23** Cross-tenant cache leakage in stripe_api_calls is prevented by RLS (§1 #17); cache lookups always include `tenant_id = current_setting('auth.tenant_id')::uuid`. Property test: spawn two tenants concurrently issuing same operation; assert each gets their own response (no key collision).

**§11.24** The `is_downgrade` check in plan_change (§1 #8) compares tier ordinals: `enterprise > team > starter`. Same-tier plan_change is rejected upstream at TASK-TEN-002 #13 (`no_change` 409), so this task doesn't need to handle same-tier.

**§11.25** Reason text in refund + dunning audit rows is free-text; TASK-MEMORY-111 PII scrubbing applies the standard ruleset. CFO provides reason at refund time; system generates reason for dunning (`payment_failed_retry_<n>`).

---

*End of TASK-TEN-003 spec.*
