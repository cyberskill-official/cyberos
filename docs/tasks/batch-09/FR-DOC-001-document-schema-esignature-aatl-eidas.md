---
title: "DOC — schema (Documents, Templates, Envelopes, Signatures), e-signature primitives, AATL adv-cert, eIDAS QTSP integration"
author: "@stephen-cheng"
department: legal
status: ready_for_review
priority: p3
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: infrastructure
eu_ai_act_risk_class: not_ai
target_release: "P3 / 2027-Q4"
client_visible: false
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Stand up the **DOC (Document signing) module** that PRD §9.20 + §14.4.1 schedule for P3-P4: schema for **Document** (the canonical artefact), **Template** (reusable per-tenant), **Envelope** (a multi-signer signing flow), **Signature** (per-signer cryptographic + audit-trail evidence), **DocumentVersion** (immutable post-sign); **e-signature primitives** with three security tiers — Simple Electronic Signature (SES) for low-risk internal docs, Advanced Electronic Signature (AdES) for binding business docs (uses AATL — Adobe Approved Trust List — certified certificates), Qualified Electronic Signature (QES) for high-stakes docs that need eIDAS QTSP (Qualified Trust Service Provider) integration; **document storage** in the per-tenant blob store with cryptographic chain-of-custody; **signing workflow** with parallel + sequential signing modes + reminder cadence + full audit trail per Vietnamese Decree 130/2018 + EU eIDAS Regulation 910/2014. Subsequent FR-DOC-002 ships the contract redline review (read-only AI; CLO sign-off required) + the frontend remote.

## Problem

CyberSkill's contracts today are signed by paper + scanned + emailed; PRD §1.1's Origin notes "paper held the formal employment paperwork." For P3 SaaS-readiness, the platform needs:

- **Member-facing signing.** Every employee must accept their employment contract (FR-HR-001), salary letters (FR-REW-001), grant-agreements (FR-ESOP-001), 360 acknowledgements — all currently scanned-paper. Scaling past 10 employees + adding external tenants makes this unsustainable.
- **Customer-facing signing.** External tenants in P3+ sign service agreements + DPAs + amendments through the platform — not via DocuSign with PDFs emailed back-and-forth.
- **Regulatory-grade signing for EU customers.** GDPR + eIDAS + Vietnamese Decree 130/2018 (e-signature law) require specific technical standards; the platform must support QES for the highest tier of cross-border legal validity.

## Proposed Solution

The shape of the answer is `doc.*` schema + per-tenant blob storage with chain-of-custody + the signing-workflow state machine + AATL + eIDAS QTSP integration.

**Schema.**

```sql
CREATE SCHEMA doc;

-- A document (any artefact: contract, NDA, addendum, acknowledgement form).
CREATE TABLE doc.document (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  document_kind TEXT NOT NULL,                                          -- "employment_contract" | "salary_letter"
                                                                       -- | "esop_grant_agreement" | "service_agreement"
                                                                       -- | "dpa" | "nda" | "amendment" | "acknowledgement"
                                                                       -- | "policy_handbook" | "ad_hoc"
  template_id UUID REFERENCES doc.template(id),
  current_version INT NOT NULL DEFAULT 1,
  current_version_blob_id UUID NOT NULL,                                -- the rendered PDF in the blob store
  source_md_blob_id UUID,                                               -- the Markdown source if templated
  parent_envelope_id UUID,                                              -- the signing envelope this doc rides in (when applicable)
  related_entity_kind TEXT,                                             -- "hr_contract" | "rew_salary" | "esop_grant" | "tenant_provisioning"
  related_entity_id UUID,
  status TEXT NOT NULL DEFAULT 'draft',                                  -- "draft" | "in_review" | "ready_for_signature"
                                                                      -- | "in_signing" | "fully_signed" | "voided" | "superseded"
  signed_xml_blob_id UUID,                                               -- the signed XAdES/XML when fully signed
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  superseded_by UUID REFERENCES doc.document(id),
  archived_at TIMESTAMPTZ
);

-- Template: reusable doc skeleton with placeholders.
CREATE TABLE doc.template (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  document_kind TEXT NOT NULL,
  name TEXT NOT NULL,
  description_md TEXT,
  body_md TEXT NOT NULL,                                                -- Markdown with {{placeholders}}
  placeholders_schema JSONB NOT NULL,                                   -- structured: list of placeholder names + types + required
  default_signing_tier TEXT NOT NULL DEFAULT 'AdES',                     -- "SES" | "AdES" | "QES"
  default_signers_pattern JSONB NOT NULL,                                -- e.g. [{role: "subject_employee"}, {role: "founder"}]
  signed_by_legal_counsel_at TIMESTAMPTZ,                                -- legal-counsel sign-off on the template
  legal_counsel_ref TEXT,
  is_active BOOLEAN NOT NULL DEFAULT true,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Envelope: the signing flow wrapping one or more documents.
CREATE TABLE doc.envelope (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  envelope_number TEXT NOT NULL,                                         -- "ENV-2026-001"
  initiator_member_id UUID NOT NULL,
  document_ids UUID[] NOT NULL,                                          -- one envelope can carry multiple related docs
  signing_tier TEXT NOT NULL,                                            -- "SES" | "AdES" | "QES"
  signing_mode TEXT NOT NULL DEFAULT 'sequential',                        -- "sequential" | "parallel"
  signers JSONB NOT NULL,                                                -- [{member_id, external_email, role, order}, ...]
  status TEXT NOT NULL DEFAULT 'draft',                                   -- "draft" | "ready_to_send" | "sent"
                                                                       -- | "in_progress" | "completed" | "voided" | "expired"
  sent_at TIMESTAMPTZ,
  expires_at TIMESTAMPTZ,                                                -- typically sent_at + 30 days
  completed_at TIMESTAMPTZ,
  completed_signed_envelope_blob_id UUID,                                -- the final fully-signed bundle PDF/XML
  reminder_cadence_md TEXT,                                              -- "every 7 days until signed or expired"
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (tenant_id, envelope_number)
);

-- Per-signer signature evidence.
CREATE TABLE doc.signature (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  envelope_id UUID NOT NULL REFERENCES doc.envelope(id) ON DELETE RESTRICT,
  document_id UUID NOT NULL REFERENCES doc.document(id) ON DELETE RESTRICT,
  signer_member_id UUID,                                                  -- when signer is internal Member
  signer_external_email TEXT,                                             -- when signer is external (client, vendor)
  signer_external_name TEXT,
  signer_external_id_proof_blob_id UUID,                                  -- ID-doc upload for external high-tier signers
  signer_role TEXT NOT NULL,                                              -- "subject_employee" | "founder" | "client_authorised"
                                                                       -- | "legal_counsel" | "witness"
  signing_tier TEXT NOT NULL,                                              -- "SES" | "AdES" | "QES"
  signing_method TEXT NOT NULL,                                            -- "click_to_sign" | "drawn" | "typed"
                                                                       -- | "aatl_certificate" | "qtsp_qes"
  signed_at TIMESTAMPTZ NOT NULL,
  ip_address INET,
  user_agent TEXT,
  certificate_subject TEXT,                                                 -- for AdES/QES: signer's cert subject DN
  certificate_serial TEXT,
  certificate_issuer TEXT,
  signed_value_blob_id UUID NOT NULL,                                       -- the cryptographic signature blob
  ots_timestamp_blob_id UUID,                                                -- OpenTimestamps proof (additional tamper-evidence)
  qtsp_provider TEXT,                                                       -- when QES: the QTSP that issued the cert
  qtsp_validation_blob_id UUID,                                             -- the QTSP's signed validation report
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);

CREATE INDEX signature_envelope_idx ON doc.signature (tenant_id, envelope_id);
CREATE INDEX signature_signer_idx   ON doc.signature (tenant_id, signer_member_id);

-- Document version history (immutable post-sign).
CREATE TABLE doc.document_version (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  document_id UUID NOT NULL REFERENCES doc.document(id) ON DELETE RESTRICT,
  version INT NOT NULL,
  body_md TEXT NOT NULL,
  rendered_pdf_blob_id UUID NOT NULL,
  signed_xml_blob_id UUID,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  finalised_at TIMESTAMPTZ,                                                  -- when status moved to fully_signed
  UNIQUE (tenant_id, document_id, version)
);
```

**Three signing tiers + when to use each.**

| Tier | When | Technical Implementation | Legal Effect |
|---|---|---|---|
| **SES** (Simple) | Internal acknowledgements, ad-hoc forms, low-risk docs | Click-to-sign with audit trail (IP, timestamp, account) | Acceptable for internal records; questionable in cross-border disputes |
| **AdES** (Advanced) | Employment contracts, salary letters, NDAs, ESOP grants, T1/T2 service agreements, internal Vietnamese contracts | AATL-trusted certificate (Adobe-trusted CA) signs the doc cryptographically; OpenTimestamps proof attached | Legally binding under Vietnamese Decree 130/2018 + most jurisdictions' e-sig laws |
| **QES** (Qualified) | T3 enterprise service agreements, cross-border contracts subject to EU jurisdiction, regulator filings | eIDAS QTSP (e.g. SwissSign, eMudhra, FNMT-RCM) issues a per-signer Qualified Certificate; the cert is bound to a verified national-ID; signing is via QTSP's portal or interoperable API | eIDAS Article 25 — equivalent to handwritten signature across all EU member states; same effect in Vietnam under Decree 130 + 2014 Civil Code |

The platform routes the right tier per document_kind by default per the template; the envelope initiator can upgrade (never downgrade) at envelope-creation time.

**AATL integration.**

The platform maintains an AATL-trusted certificate per tenant per signing-key-purpose:
- **Tenant signing certificate.** Issued by an AATL-listed CA (e.g. SSL.com, Sectigo) to the tenant's legal entity; used for AdES platform-side signatures (e.g. counter-sign on tenant-side legal docs).
- **Member signing certificate.** Issued by the tenant's CA-of-record per Member when they enrol; used for AdES per-signer signatures.

Certificate lifecycle:
- Issued at provisioning + when new Member enrols.
- Stored in HashiCorp Vault under `tenant/{tenant}/doc-signing-cert/{member-id}`.
- Used for signing operations only; never exported.
- Rotated annually with audit trail.

Signing operation (AdES):
1. Member clicks "Sign" on a document.
2. Step-up auth (passkey).
3. Server-side: HashiCorp Vault signs the document hash with the Member's cert; the signed value is stored in `doc.signature.signed_value_blob_id`.
4. OpenTimestamps proof is attached (anchors the signature to a public Bitcoin block, providing tamper-evident timestamping that survives even if the AATL CA is compromised).
5. The PDF is updated with the visible signature block + the embedded XAdES envelope.

**eIDAS QTSP integration.**

For QES tier:
1. The platform integrates with QTSP APIs (initially: **eMudhra**, **SwissSign**, **FNMT-RCM** — the major QTSPs for VN+EU+US-bridge customers).
2. The Member or external signer authenticates through the QTSP's identity-verification flow (national ID upload + biometric check; one-time per signer per certificate term).
3. The QTSP issues a Qualified Certificate bound to the verified identity.
4. Signing happens through the QTSP's signature service; the platform receives the signed-document + the QTSP's validation report.
5. Audit trail records the QTSP provider + the validation report blob.

The QTSP integration is per-tenant per-shard:
- vn-shard tenants typically use eMudhra or VietCert (Vietnamese QTSPs).
- eu-shard tenants use a Trust List-listed EU QTSP per their legal jurisdiction.
- us-shard + sg-shard tenants use AATL + occasionally region-specific QES providers.

**Signing-workflow state machine.**

Envelope states + transitions:
```
draft → ready_to_send         (envelope owner reviews + confirms)
ready_to_send → sent          (initiator clicks send; signers receive notifications)
sent → in_progress             (first signer signs)
in_progress → completed        (all signers signed)
in_progress → expired           (auto-expire at expires_at if not all signers signed)
sent / in_progress → voided    (initiator voids; reason required; envelope cannot be revived)
```

Sequential signing: signers sign in declared order; signer N+1 receives notification only after signer N signs.
Parallel signing: all signers receive notification simultaneously; envelope completes when all have signed.

**Reminder cadence.**

Default: every 7 days after sent_at, a Notify card + email reminder to the next-required signer. Configurable per envelope. Three reminders max; final reminder includes the expiry date.

**Document storage + chain-of-custody.**

Per-tenant blob store; documents are content-addressed (SHA-256). Per-version PDFs are immutable; once finalised, the version's blob is never overwritten. The platform's signing certificate also signs the blob's hash, anchoring the version to the platform's identity. Cold archive at S3 Glacier with Object Lock per the doc's retention class (employment contracts: 7y post-termination; service agreements: 10y; tax-relevant docs: 10y).

**MCP tool surface (read + non-destructive create).**

- `cyberos.doc.list_my_envelopes(status?)` — read; calling Member.
- `cyberos.doc.get_envelope(id)` — read.
- `cyberos.doc.list_my_signatures(since?)` — read.
- `cyberos.doc.list_templates(document_kind?)` — read; HR/Ops + Founder + Legal.
- `cyberos.doc.create_envelope_from_template(template_id, placeholders, signers)` — `destructive: true; requires_confirmation: true; sensitivity: medium`.
- `cyberos.doc.send_envelope(id)` — `destructive: true; requires_confirmation: true; sensitivity: high; step_up_required: true`.
- `cyberos.doc.void_envelope(id, reason)` — `destructive: true; requires_confirmation: true; sensitivity: high; step_up_required: true`.
- `cyberos.doc.sign_envelope(id, document_ids, signing_tier)` — `destructive: true; requires_confirmation: true; sensitivity: high; step_up_required: true`. Each signing tier has progressively-stronger auth requirements (SES: passkey only; AdES: passkey + AATL cert from Vault; QES: passkey + redirect to QTSP signing flow + return).

CUO scope contract: read all + create_envelope_from_template (as a draft) allowed; send + sign + void forbidden — all signing actions are human-only with step-up.

**Audit integration.** `doc.{tenant}` audit scope. Every envelope state transition + every signature event audit-logged. Signature audit rows include the cryptographic evidence references for forensic reconstruction.

## Alternatives Considered

- **Use DocuSign / Adobe Sign / Dropbox Sign as embedded SaaS.** Rejected: residency + the integration with HR + REW + ESOP modules + the per-tenant cert storage + the audit-chain integration cannot be enforced through embedded hosted providers.
- **Skip QES; only SES + AdES.** Rejected: cross-border EU customers in P3+ require QES for highest legal validity; without it, the deal-flow ceiling is hit.
- **Single signing tier (AdES) for everything.** Rejected: SES is sufficient + lighter-friction for low-risk acknowledgements; QES is required for highest-stakes; tier-mapping is the right design.
- **AI-assisted document drafting.** Deferred to FR-DOC-002 (read-only redline review only; AI never auto-signs or commits).

## Success Metrics

- **Primary metric.** P3 sprint demo passes: (1) the schema deploys; (2) a synthetic SES envelope is signed end-to-end (employee acknowledgement form); (3) a synthetic AdES envelope (employment contract) is signed with AATL certificate + OpenTimestamps proof; (4) a synthetic QES envelope (international service agreement) is signed via eMudhra QTSP integration with the validation report stored.
- **Compliance metric.** 100% of completed envelopes have valid signature evidence verifiable by an independent auditor.
- **Latency metric.** SES signing ≤ 5 s; AdES ≤ 30 s; QES ≤ 5 minutes (QTSP redirect adds latency).

## Scope

**In-scope.**
- The 5 schema additions (`document`, `template`, `envelope`, `signature`, `document_version`).
- AATL certificate provisioning per-tenant per-Member.
- AATL signing operation via HashiCorp Vault.
- eIDAS QTSP integration with eMudhra + SwissSign + FNMT-RCM (3 initial providers).
- Three-tier signing (SES + AdES + QES).
- Sequential + parallel signing modes.
- Reminder cadence.
- OpenTimestamps anchoring for AdES.
- Per-tenant per-doc-kind retention rules.
- The 8 MCP tools.
- Audit integration in scope `doc.{tenant}`.

**Out-of-scope (deferred to FR-DOC-002).**
- Contract redline AI review (FR-DOC-002).
- Frontend remote at /doc (FR-DOC-002).
- Custom QTSP integrations beyond the initial 3 (P4 — per customer demand).
- Bulk-signing for HR contract refresh (P4).
- Cross-tenant signing (one tenant signs another tenant's doc) — out of scope by design.

## Dependencies

- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-002 / FR-AUTH-003 / FR-MCP-001.
- FR-TEN-001 / FR-TEN-002 (tenant lifecycle + per-tenant blob store).
- FR-HR-001 / FR-REW-001 / FR-ESOP-001 (related-entity references).
- FR-CP-003 (DPIA library).
- HashiCorp Vault per-tenant for cert storage.
- AATL CA contracts (SSL.com or Sectigo).
- eIDAS QTSP API contracts (eMudhra + SwissSign + FNMT-RCM).
- OpenTimestamps client library.
- Compliance: Vietnamese Decree 130/2018 (e-signature law); EU eIDAS Regulation 910/2014; Adobe AATL program requirements; PDPL Decree 13 (signature evidence is personal data); SOC 2 CC6 + CC8.
- Locked decisions referenced: DEC-269 (3 signing tiers SES/AdES/QES), DEC-270 (AATL for AdES), DEC-271 (3 initial QTSP providers), DEC-272 (OpenTimestamps anchoring for AdES tamper-evidence beyond CA).

## AI Risk Assessment

Not applicable. `eu_ai_act_risk_class: not_ai`. The signing path is deterministic cryptography. AI surfaces (contract redline review) ship in FR-DOC-002 with the appropriate `limited` classification.
