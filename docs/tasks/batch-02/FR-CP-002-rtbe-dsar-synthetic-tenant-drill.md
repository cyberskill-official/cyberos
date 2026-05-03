---
title: "Compliance Plane — RTBE (right-to-be-erased) and DSAR scaffold; synthetic-tenant drill at P0 exit"
author: "@stephen-cheng"
department: operations
status: ready_for_review
priority: p0
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: internal_tooling
eu_ai_act_risk_class: not_ai
target_release: "P0 / 2026-Q3"
client_visible: false
template: feature_request@1
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Ship the **right-to-be-erased (RTBE)** and **data-subject-access-request (DSAR)** scaffolding for CyberOS, then exercise the RTBE flow end-to-end on a synthetic tenant during S0-6 as the P0 exit-gate proof. The scaffolding includes: tenant-deletion request + verification + execution pipeline, **cryptographic shredding** of per-tenant KMS keys (the deletion mechanism for tenant data — keys are destroyed; ciphertext is left to weather), Layer 1 + Layer 2 + Layer 3 + audit + outbound-archive coordinated deletion with **chain-of-custody preservation** (the audit chain remains valid and the deletion is itself audit-logged, so a regulator can verify "this tenant's data has been erased on this date" without anyone having to trust an operator's word), and a **DSAR export** that produces a portable bundle of every personal data record about a Member or contact within the regulator-mandated SLA. The S0-6 sprint exit demo (PRD §17.6) requires "tenant-deletion (RTBE) flow exercised end-to-end on a synthetic tenant" with audit chain integrity preserved.

## Problem

PDPL Decree 13/2023, GDPR Article 17 (P3+), and the platform's own "no lock-in" trust posture all require that:

- A tenant can be erased verifiably and irreversibly.
- A data subject (a Member, a CRM contact, an external email correspondent) can request access to their data and receive a structured export within 30 days.
- The deletion does not break the audit log's integrity (a regulator must still be able to verify "X happened on Y date" for every event up to the deletion).
- The deletion does not require trusting any single operator — the cryptographic shredding mechanism makes the deletion irreversible at the algorithm level, not at policy level.

S0-6 risk-gate (PRD §17.6): "Cross-tenant leakage rate ≠ 0 blocks phase exit. Persona scope contract enforcement < 100% blocks phase exit." The RTBE drill is the compliance-side analogue — proving the deletion path works without breaking other invariants.

## Proposed Solution

The shape of the answer is a multi-step CP-owned workflow, a per-tenant KMS-key model, and a synthetic-tenant drill harness.

**Per-tenant KMS-key model.** Every tenant has a dedicated KMS key in HashiCorp Vault (P0) / AWS KMS (P3 cross-region). The key is used to envelope-encrypt:

- BRAIN Layer 3 raw documents (the canonical raw text on disk in S3 cold archive).
- AUTH per-Member sensitive columns (WebAuthn challenge replays, refresh-token sessions).
- HR/REW/ESOP encrypted columns (when those modules ship in P2; their schema already references `tenant_kms_key_id`).
- Per-tenant signed-export private key (FR-BRAIN-001 §"Signed `.zip` export").

Hot-tier Postgres rows are *also* encrypted at rest by Postgres TDE plus a per-row column-level encryption for the most sensitive classes; the column-level key is the same per-tenant KMS key.

**RTBE flow.**

```
1. REQUEST     → tenant administrator (or DPO acting on their behalf)
                 submits an RTBE request via the /compliance UI
                 plus a separate signed email confirmation.

2. VERIFY      → CP module verifies:
                 - the requester's identity (passkey + TOTP)
                 - the request signature against the email confirmation
                 - the 14-day revocation window has elapsed (or is waived
                   by an explicit "I understand this is irreversible"
                   acknowledgement; default is the 14-day window)

3. SUSPEND     → tenant is moved to status "deletion-pending":
                 - all incoming writes rejected
                 - existing sessions invalidated
                 - module-federation host shell shows the tenant a
                   "deletion in progress" banner
                 - export bundle is offered for download one last time
                   (signed .zip per FR-BRAIN-001)

4. EXPORT      → if the tenant requested a final export, the bundle is
                 generated, signed, and held for 30 days behind a
                 download URL gated by passkey + TOTP. After 30 days
                 the bundle is destroyed.

5. CRYPTO-SHRED → the tenant's KMS key is destroyed in the KMS provider:
                 - Vault: vault delete sys/internal/tenant-kms/<tenant_id>
                 - AWS KMS: aws kms schedule-key-deletion --pending-window-in-days 7
                 The key destruction is the deletion mechanism — the
                 ciphertext rows in Postgres + S3 remain physically
                 present but become unreadable to anyone, including
                 the operator.

6. SOFT-DELETE → all per-tenant rows are marked deleted_at = now() and
                 their tenant_id is rotated to a synthetic post-deletion
                 UUID so they no longer match any extant tenant scope.
                 The audit log rows are preserved (chain-of-custody) but
                 their personal-data payloads are pseudonymised in place
                 with a one-way hash so the chain hashes still verify.

7. HARD-DELETE → after 90 days the soft-deleted rows are physically
                 removed from Postgres (DELETE) and from any cold
                 archive partitions (S3 deletion request scheduled
                 against the partition's Object Lock retention end date,
                 which for tenant-deletion is shortened from the default
                 7y to the 90-day RTBE floor).

8. CERTIFY     → CP produces a "Certificate of Erasure" PDF signed by
                 the DPO + Founder summarising:
                 - the request reference
                 - the verification chain
                 - the timestamp of crypto-shred
                 - the audit chain head hash before and after pseudonymisation
                 - the soft-delete and hard-delete dates
                 The certificate is filed in the platform's permanent
                 compliance archive and offered to the tenant.
```

The flow is implemented as a state machine in `cp.rtbe_request` with the eight states above; transitions are dual-approved (DPO + Founder); each transition writes an audit row.

**Audit-log preservation.** When a Member is pseudonymised, only the audit row's `payload` field is rewritten (one-way hash of the original PII content); the row's `prev_hash` and `this_hash` are preserved bitwise, so the Merkle chain still verifies. This is the structural reason the audit-row payload schema is designed to allow pseudonymisation without re-hashing (FR-AUTH-002 §"Right-to-erasure handling"). A regulator can verify "the chain is intact" and "this tenant's PII has been pseudonymised" simultaneously.

**DSAR scaffold.** A data subject (a Member, a CRM contact, an external email correspondent referenced in CHAT/EMAIL) can request access to their data:

1. The DSAR request is submitted via the `/compliance` UI (Members) or via the public `dsar@cyberos.world` email + passkey-verified portal (external subjects, P3+; in P0 only Member-self DSAR is exercised).
2. The CP module runs the DSAR enumerator: a per-tenant query that walks Layer 1 (files where the subject's authors[] contains the subject UUID), Layer 2 (facts whose subject_uri references the subject's URI), Layer 3 (raw documents where the subject is in `authors[]` or matched by name), audit log (rows where `actor_subject` or `payload.member_id` matches), AUTH sessions, CHAT messages, KB pages, etc.
3. The enumerator produces a structured JSON bundle plus a Markdown summary plus an HTML view; bundle is signed with the tenant export key.
4. SLA: DSAR responses must complete in ≤ 30 days (PDPL Decree 13 + GDPR Article 12 floor). The CP dashboard shows the open requests' timers.
5. Bundle download is gated by a passkey-verified one-time link valid for 7 days.

**Synthetic-tenant drill harness.** A small `cyberos-rtbe-drill` CLI runs the entire RTBE flow on a synthetic tenant called `tenant_rtbe_drill_{date}`:

1. Provision a synthetic tenant with seed data: 5 synthetic Members, 100 BRAIN facts, 1,000 Layer 3 raw documents, 50 audit rows, 10 CHAT channels, 200 messages.
2. Issue a synthetic RTBE request signed by a synthetic DPO key.
3. Walk the eight-state flow with audit-row capture at every transition.
4. Verify post-shred: (a) reading any synthetic-tenant ciphertext returns "key not found" — proving crypto-shred worked, (b) the audit chain still verifies on `cyberos-audit-verify`, (c) the synthetic Member sessions are invalidated, (d) the certificate is produced.
5. Generate a drill report attached to the P0 → P1 gate-readiness submission.

The drill runs in CI nightly during S0-6 and quarterly thereafter; quarterly drills become the standing evidence for SOC 2 + ISO 27001 audits.

**MCP tool surface.**

- `cyberos.cp.dsar_request(member_id, requester_proof)` — `destructive: false` (read-only operation; produces a request record; actual data export requires DPO approval).
- `cyberos.cp.dsar_status(request_id)` — read.
- `cyberos.cp.rtbe_request(tenant_id, requester_proof)` — `destructive: true; requires_confirmation: true; irreversible: true`. **Despite being annotated `irreversible: true`, this tool is registered.** The annotation marks it as the highest-stakes tool in the platform; the gateway requires both `client_confirmed: true` *and* a separate `irreversible_confirmation_token` produced by a 14-day-elapsed verification step. This is the only `irreversible: true` tool in P0; all others are non-MCP per FR-MCP-001 §"Tool annotations enforced at the proxy".
- `cyberos.cp.rtbe_status(request_id)` — read.
- `cyberos.cp.list_open_requests` — read; DPO-only.

The CUO persona's `tools_forbidden_explicit` list includes `cyberos.cp.rtbe_request` for every persona — only humans can initiate.

## Alternatives Considered

- **Logical deletion only (set deleted_at; never crypto-shred).** Rejected: a regulator-grade "verifiably erased" claim needs a mechanism that cannot be undone by an operator with database access; crypto-shred is the floor.
- **Hard-delete immediately at request acceptance.** Rejected: the 14-day revocation window is a structural protection against social-engineering attacks where an attacker submits an RTBE on behalf of a tenant they compromised.
- **Skip the synthetic drill; rely on production walkthrough at the first real RTBE.** Rejected: P0 exit-gate explicitly requires the drill (PRD §17.6); deferring removes the gate.
- **Delete audit rows for the deleted tenant.** Rejected: the audit chain is platform-wide; deleting rows breaks the chain hashes for the surviving tenants. Pseudonymisation of payloads + retention of row + chain is the architectural compromise.
- **Single-key cryptographic system (one master key for the whole platform).** Rejected: per-tenant keys are the *only* path to per-tenant cryptographic shredding.

## Success Metrics

- **Primary metric.** S0-6 demo passes: (1) the synthetic-tenant drill completes end-to-end with the certificate generated; (2) post-shred reads return "key not found" for ≥ 95% of attempted decryptions on synthetic ciphertext; (3) the audit-chain verifier passes on the post-pseudonymisation chain; (4) a synthetic DSAR request for `synthetic-member-3` returns a structured bundle in ≤ 60 minutes (well within the 30-day SLA at production scale).
- **Compliance metric.** Certificate of Erasure produced and filed; the compliance archive has 0 unfiled certificates.
- **Drill cadence.** Drill runs successfully every quarter from P1 onward; failure is sev-0.

## Scope

**In-scope (S0-6).**
- Per-tenant KMS-key model with Vault provisioning + lifecycle.
- The eight-state RTBE flow in `cp.rtbe_request`.
- Crypto-shred mechanics across BRAIN L3, AUTH columns, signed-export keys.
- Audit-row pseudonymisation pattern preserving chain hashes.
- DSAR scaffold for Member-self DSARs (Members only in P0; external subjects in P3).
- The DSAR enumerator that walks every per-tenant data store.
- Certificate of Erasure PDF generator + DPO + Founder dual-sign flow.
- Synthetic-tenant drill CLI + drill harness in CI for S0-6.
- The MCP tools listed above.
- Audit integration in scopes `cp.rtbe.{tenant}` and `cp.dsar.{tenant}`.

**Out-of-scope (deferred).**
- External-subject DSAR portal (P3).
- Cross-region RTBE for tenants spanning multiple residency regions (P3 — single-region in P0).
- Bulk DSAR for organisational requests (P4 PORTAL).
- ISO/IEC 27001 evidence-of-erasure structured logging (P2 onwards).
- Right-to-explanation as part of DSAR (P3).

## Dependencies

- FR-INFRA-001 (Postgres + S3 + Vault).
- FR-AUTH-001 / FR-AUTH-002 (DPO role + audit log + chain pseudonymisation).
- FR-MCP-001 (the only `irreversible: true` tool in P0 lives here).
- FR-BRAIN-001 / FR-BRAIN-002 / FR-BRAIN-003 (per-tenant data deleted on shred; signed export available pre-deletion).
- FR-CP-001 (DPIA references inform DSAR scope; certificate stored in CP archive).
- FR-OBS-001 / FR-OBS-002 (Compliance Cockpit shows RTBE backlog; alerts on drill failure).
- HashiCorp Vault provisioned + per-tenant key lifecycle automation.
- Compliance: PDPL Decree 13 Articles on data subject rights; GDPR Articles 12, 17, 22 (P3 surface); SOC 2 CC6 (logical access) + CC7 (system operations); the synthetic drill is the structural audit evidence.
- Locked decisions referenced: DEC-070 (per-tenant KMS keys), DEC-071 (crypto-shred is the deletion mechanism), DEC-072 (audit-row pseudonymisation preserves chain), DEC-073 (RTBE only `irreversible: true` MCP tool in P0).

## AI Risk Assessment

Not applicable. `eu_ai_act_risk_class: not_ai`. The RTBE / DSAR machinery is deterministic; no AI inference in the flow. (DSAR enumerator output may flow through CUO for natural-language presentation in P3; that surface is FR-CP-DSAR-NL-001 with appropriate AI risk classification at that point.)

## AI Authorship Disclosure

- **Tools used:** Claude Cowork (Anthropic).
- **Scope:** drafted the FR end-to-end against the PRD + SRS; founder reviews and edits before status changes from `ready_for_review`.
- **Human review:** founder (`@stephen-cheng`) — final wording is the founder's responsibility.
