---
title: "GTM — ISO/IEC 42001 AI management system + SOC 2 Type II close + final compliance graduation"
author: "@stephen-cheng"
department: legal
status: ready_for_review
priority: p1
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: compliance
eu_ai_act_risk_class: not_ai
target_release: "P4 / 2028-Q4"
client_visible: true
template: feature_request@1
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Close out the **final compliance graduation** of CyberOS at P4: (1) **SOC 2 Type II audit close** — the 12-month operational evidence collection that started at P3 entry (FR-CP-005's scaffolding + FR-OBS-004's evidence map) is consumed by the SOC 2 Type II auditor, the audit closes, the report is issued; (2) **ISO/IEC 42001 (AI Management System) Stage 1 + Stage 2 audit** — the international standard for AI governance, especially relevant since CyberOS is AI-native and operates EU-AI-Act-classified high-risk components (compensation modules, etc.); ISO 42001 was finalised in December 2023 and has matured by 2028 into the de-facto certification for AI-using SaaS; (3) **EU AI Act high-risk obligations** discharged at full compliance: Article 9 risk management system, Article 10 data governance, Article 11 technical documentation, Article 12 record-keeping, Article 13 transparency to deployers, Article 14 human oversight, Article 15 accuracy/robustness/cybersecurity — all evidenced via FR-CP-004's evidence map; (4) **Trust Center final publish** — SOC 2 Type II report (NDA-gated), ISO 27001 certificate (already issued at P3), ISO 42001 certificate, EU AI Act conformity statement, audit attestation letters; (5) **Public commitment** to ongoing compliance: yearly recertification cadence published, sub-processor change-notification flow live, vulnerability disclosure program live, status page transparent. This FR is the moment CyberOS stops being "compliant by intent" and becomes "compliant by audited record".

## Problem

PRD §14.5.2 P4 → GA exit-gate criterion (the final post-launch milestone): "SOC 2 Type II report issued; ISO/IEC 42001 certificate issued; EU AI Act compliance attestation complete; Trust Center fully populated with the final compliance posture; first 10 paying customers' security reviews completed using only the Trust Center artefacts (no email back-and-forth with CyberSkill DPO)."

Without this final close-out:
- Enterprise prospects (companies with formal vendor security review) cannot fast-track CyberOS adoption — every contract is a custom security questionnaire.
- The EU AI Act enforcement window (which begins August 2026 for Article 5 prohibited practices and February 2027 for high-risk obligations) catches CyberOS unprepared if compensation/HR modules are still rolling out without conformity attestation.
- The competitive position weakens: every well-funded SaaS competitor has SOC 2 Type II by Year 3; CyberOS without it would be the cheap-but-risky option.

Three failure modes:

- **Audit timeline slip.** SOC 2 Type II requires 12 months evidence; if collection started late, the audit window slips. Mitigation: FR-CP-005 + FR-OBS-004 already started collection at P3 entry.
- **ISO 42001 surprise.** ISO 42001 is new; many auditors don't have experience. Mitigation: pre-engage one of the early-adopter audit firms (Schellman, A-LIGN, or local equivalents in SG/EU/VN); 6-month lead time.
- **EU AI Act gap.** A platform that claims AI-native must be ready for the high-risk article-by-article record. Mitigation: FR-CP-004 already produces this evidence map; this FR validates + publishes it.

## Customer Quotes

<!-- Required when client_visible: true. Verbatim, attributed where possible. Paraphrasing here costs you the signal. -->

<untrusted_content source="other">
…paste verbatim customer quote here…
</untrusted_content>

<!-- TODO during implementation PR: capture real customer quotes from sales calls / NPS / support tickets. -->

## Proposed Solution

A coordinated 3-track compliance close.

### Track 1: SOC 2 Type II close

**Auditor engagement.** Already pre-engaged at P3 entry (FR-CP-005 selected a CPA firm with both SOC 2 + ISO 42001 capability). Audit firm options at vendor selection: Schellman, A-LIGN, Prescient Assurance, BDO. Selection criteria: experience with multi-tenant SaaS + AI-native + multi-jurisdiction.

**Evidence intake.**
- 12 months of `audit.soc2_evidence_log` entries (per FR-OBS-004's scaffolding).
- Per-control walkthrough sessions: auditor interviews CyberSkill team for each control.
- Sample testing: auditor selects samples from each control (access reviews, change-management, incident response, etc.); CyberSkill provides evidence per sample.
- Management responses to any exceptions.

**Audit close timeline.**
- Month 1-2 of P4: evidence packaging + initial interviews.
- Month 3: fieldwork + sample testing.
- Month 4: report drafting + management responses.
- Month 5: report issuance.

**Report consumption.**
- NDA-gated download via FR-GTM-001's Trust Center.
- Auditor's report covers Common Criteria (CC1-9) + selected Trust Service Criteria (Security mandatory; Availability, Confidentiality, Privacy, Processing Integrity selected based on platform scope).

### Track 2: ISO/IEC 42001 audit

**Standard scope.** ISO/IEC 42001:2023 Information technology — Artificial intelligence — Management system. Lifecycle of AI management.

**Pre-engagement.** Same audit firm as ISO 27001 (already engaged via FR-CP-005) extended for ISO 42001 capability. Stage 1 (documentation review) at Month 2 of P4; Stage 2 (audit + sample testing) at Month 4 of P4.

**Required artefacts (auto-generated where possible).**
- AI Management System (AIMS) policy — signed by Founder.
- AI use-case inventory — every use case classified by risk; auto-generated from FR-AI-001's persona-version + skill-version registry.
- AI risk assessment per use-case — covering bias, robustness, transparency, accountability; auto-generated from per-FR `Risk Assessment` sections + supplemented with the platform-level risk register.
- AI data governance — covering training-data provenance, fine-tuning records, retention, deletion. CyberOS doesn't fine-tune frontier models, so this section primarily covers RAG context (BRAIN), persona Skills, and prompt templates.
- AI development lifecycle — covering review, testing, deployment, monitoring, retirement. Maps to FR-GENIE-001 dual-sign + FR-OBS-002 monitoring.
- AI third-party management — Bedrock + Anthropic ZDR + OpenAI ZDR contracts + sub-processor list.
- AI human oversight — for high-risk uses, the human-in-the-loop architecture from FR-REW-001/002 (compensation), FR-DOC-001 (signing), FR-HR-001 (hiring).
- Continual improvement plan — quarterly AIMS review.

**Stage 2 sample testing focuses on:**
- High-risk AI uses (compensation, hiring, contract review).
- Persona-version + skill-version chain integrity.
- Article 14 oversight mechanisms.
- Prompt-injection defence (CaMeL pattern from FR-EMAIL-003 + FR-CHAT-001).

### Track 3: EU AI Act high-risk attestation

**Conformity assessment.** For high-risk AI systems, the EU AI Act requires either internal control conformity assessment (Article 43 with Annex VI) or third-party assessment. CyberOS modules:
- Compensation (REW): high-risk per Annex III §4(b) "AI systems used to make decisions affecting employment relationships including pay…". CyberOS architecture: AI describes, humans decide → Article 43 internal-control conformity is appropriate.
- Hiring/promotion (LEARN, HR): same Annex III §4(a). Same architecture, same conformity path.
- All other AI surfaces: limited risk (Article 50 transparency).

**EU AI Act technical file.**
- Article 11 technical documentation pack.
- Article 12 logging records (samples).
- Article 13 instructions for use (deployers).
- Article 14 human oversight design.
- Article 15 accuracy + robustness metrics (hallucination rates, prompt-injection-defence rates, persona-acceptance rates).
- CE marking + EU declaration of conformity (Article 47).

The compliance plane (FR-CP-004 + this FR) generates this technical file from existing platform evidence; it is signed by the Founder and the DPO.

**Notified body engagement.** Article 43 internal-control conformity does not require a notified body unless harmonised standards are not yet published. CyberOS targets internal-control + harmonised standards (ISO 42001 acts as the harmonised standard once formally listed). Notified body engagement reserved for if standards drift.

**EU representative.** Per Article 25 + Decree 53/2022, CyberOS appoints an EU representative (a Singapore-based or EU-based partner — to be chosen via FR-CORP-001 partner list). The representative's contact is published in the Trust Center.

### Trust Center final publish

Trust Center extends with:
- SOC 2 Type II report (NDA-gated).
- ISO/IEC 42001 certificate.
- EU AI Act conformity declaration (publicly downloadable).
- AI use-case inventory (public-safe summary).
- AI risk assessment summary (public-safe; full version internal).
- AI human oversight overview (Article 14 mechanism descriptions).
- ISO/IEC 27001 SoA (Statement of Applicability) summary.

### Post-audit operational hardening

- **Yearly recertification cadence**: ISO 27001 + 42001 surveillance audits annually; full re-audit every 3 years. SOC 2 Type II annually.
- **Sub-processor change notification**: 30-day notice via Trust Center subscription + email.
- **Vulnerability disclosure program**: HackerOne-style; 90-day disclosure window; bounty (eventual).
- **Status page**: cyberos.statuspage.io (or self-hosted) live; incidents auto-tracked from FR-OBS-002.
- **Annual penetration test**: external firm; Trust Center publishes attestation letter.

### Compliance plane reconciliation

`compliance_cockpit` panel surfaces all 9 regimes green:
- PDPL Decree 13/2023 + 53/2022 + 20/2026 (FR-CP-001/002/003).
- GDPR (FR-CP-004).
- EU AI Act high-risk + limited (this FR).
- ISO/IEC 27001 (FR-CP-005).
- SOC 2 Type II (this FR).
- ISO/IEC 42001 (this FR).
- PCI-DSS SAQ A (FR-PORTAL-002).
- eIDAS QTSP usage (FR-DOC-001).
- VN Decree 130/2018 e-signature (FR-DOC-001).

## Alternatives Considered

The shape of the answer has been deliberately constrained by the architectural rules in §2 of `README.md` and the locked decisions cited in *Dependencies*. Notable rejected approaches:

- Approaches that would have allowed AI to make compensation, equity, or document-signing decisions — rejected per the "AI describes, humans decide" rule.
- Approaches that would have created cross-tenant read or write paths — rejected per the cross-tenant invariant (FR-TEN-001 invariant test harness).
- Where there are FR-specific alternatives, they're discussed inline in *Proposed Solution* and *Constraints*.

<!-- TODO during implementation PR: replace with FR-specific rejected alternatives. -->

## Out of Scope

- ISO/IEC 27017 (cloud-specific) + 27018 (cloud personal-data) — defer; ISO 27001 covers most; revisit Year 4.
- HIPAA (US healthcare) — defer; CyberOS not currently targeting healthcare verticals.
- FedRAMP (US Federal) — defer; not in 24-month roadmap.
- Cyber Essentials (UK) — defer; ISO 27001 supersedes.
- Vertical-specific certifications (PCI-DSS Level 1, HITRUST, etc.) — defer.
- Customer-specific audit requests beyond Trust Center artefacts — handled per-deal at Year 4+.

## Dependencies

- FR-CP-001/002/003/004/005 (compliance plane fully built across P0-P3).
- FR-OBS-004 (P3 → P4 gate evidence map; SOC 2 Type II 12-month scaffolding).
- FR-AUTH-002 (audit chain — primary evidence source).
- FR-AI-001 (AI Gateway persona-version + skill-version registry).
- FR-GENIE-001/004 (persona Skills + dual-sign).
- FR-REW-001/002 (compensation human-decision architecture for Article 14 evidence).
- FR-LEARN-002 (Hội đồng human-decision for hiring/promotion).
- FR-DOC-001 (e-signature + eIDAS).
- FR-EMAIL-003 + FR-CHAT-001 (CaMeL prompt-injection defence — Article 15 evidence).
- FR-GTM-001 (Trust Center surface for publication).
- FR-CORP-001 (EU representative appointment).
- DEC-024..DEC-030 (AI Gateway architecture).

## Constraints

- **AI describes, humans decide.** This architectural rule is the central evidence for EU AI Act Article 14 conformity. Cannot be relaxed for any high-risk module.
- **No ungrounded AI claims.** All AI outputs grounded in retrieval + cited; FR-AI-001 enforces.
- **No covert AI use.** Article 50 transparency for limited risk; Article 13 deployer-instruction transparency for high-risk; cannot be silently disabled.
- **Persona-version + skill-version + LangSmith trace** are immutable audit anchors.
- **Audit observations + management responses** preserved verbatim in audit-chain.
- **Annual recertification** is non-optional; budget reserved for ongoing audit fees.
- **EU representative** must be in place before first EU-shard tenant (already required by P3 → P4 gate via FR-CP-004).

## Compliance / Privacy

This FR *is* the compliance close-out. All regimes evidenced + audited.

- PDPL Decree 13/2023 + 53/2022 + 20/2026: FR-CP-001/002/003 closed at P2.
- GDPR: FR-CP-004 closed at P3; EU AI Act conformity adds depth at P4.
- EU AI Act: high-risk modules (REW, HR/LEARN) attested via Article 43 internal-control conformity; limited-risk surfaces (CUO, CXO, AI Gateway-emitted summaries) covered by Article 50 transparency throughout.
- ISO 27001: certificate issued at P3 (FR-CP-005); surveillance audit at P4 + 12 months.
- SOC 2 Type I: issued at P3 (FR-CP-005); Type II close at P4.
- ISO 42001: Stage 1 + Stage 2 in P4 Months 2 + 4; certificate at Month 6.
- eIDAS + Decree 130/2018: ongoing via FR-DOC-001 QTSP integrations.
- PCI-DSS SAQ A: scoping confirmed at P3; ongoing self-attestation annually.

## Risk Assessment (AI-emitting features)

This FR *audits* the AI surfaces; no new AI is introduced. `eu_ai_act_risk_class: not_ai` for the FR itself (the artefact is compliance documentation), though the artefact attests to the platform's AI risk classification.

## Vietnamese-locale considerations

- Vietnamese-language ISO 42001 + SOC 2 Type II report summaries published in vi-VN (full report English; summary bilingual).
- Vietnamese DPO communication channels active.
- vi-VN customers see vi-VN compliance posture summary first; en-US at language switch.
- Vietnamese-locale VN MIC (Ministry of Information and Communications) cross-border-transfer registration filed (Decree 53/2022) where applicable.

## Scope (acceptance criteria — auditable)

- [ ] SOC 2 Type II auditor fieldwork complete; report drafted; management responses delivered; report issued.
- [ ] SOC 2 Type II report uploaded to Trust Center with NDA-gated download flow.
- [ ] ISO/IEC 42001 Stage 1 + Stage 2 audit complete; certificate issued; uploaded to Trust Center.
- [ ] EU AI Act technical file complete (Articles 9-15); signed by Founder + DPO; declaration of conformity (Article 47) signed.
- [ ] EU representative appointed per Article 25; contact in Trust Center.
- [ ] AI use-case inventory auto-generated + reviewed; published in Trust Center (public-safe summary).
- [ ] AI risk assessment per use-case completed.
- [ ] Article 14 human oversight design documented for every high-risk AI use; cited in technical file.
- [ ] Article 15 robustness metrics measured + reported (hallucination rate, prompt-injection-defence success, persona acceptance rate).
- [ ] Yearly recertification cadence published.
- [ ] Sub-processor change-notification flow live + tested.
- [ ] Vulnerability disclosure program live; PGP key + email + 90-day window published.
- [ ] Annual pentest commissioned + attestation letter in Trust Center.
- [ ] Status page live; incidents auto-tracked from FR-OBS-002.
- [ ] Compliance Cockpit shows green on all 9 regimes for ≥ 30 consecutive days at gate.
- [ ] First 10 paying customers' security reviews completed using only Trust Center artefacts (no DPO email back-and-forth) — measured via DPO inbox audit.

**Gherkin (PRD §19.18).**

```gherkin
Feature: Trust Center artefacts complete enterprise security review

  Scenario: Enterprise prospect completes security review
    Given prospect Acme Inc has a 200-question vendor security questionnaire
    And the answers map to Trust Center artefacts (SOC 2 + ISO 27001 + ISO 42001 + EU AI Act conformity + sub-processors + DPIAs + audit-log architecture + pentest)
    When Acme's security team reviews the Trust Center
    Then ≥ 95% of questions are answered without any back-and-forth with CyberSkill DPO
    And the remaining ≤ 5% are tenant-specific (e.g. "what residency will our tenant use?")
    And those tenant-specific questions are routed to a self-service flow inside the trial-tenant admin console
    And no email exchange with CyberSkill DPO is required for the standard review
    And the security review completes in ≤ 5 business days

Feature: AI use-case inventory auto-stays current

  Scenario: A new persona Skill is dual-signed and shipped
    Given a new C-skill (e.g. CSO-Sales) is dual-signed and deployed via FR-GENIE-001
    When the AI use-case inventory regenerates (nightly)
    Then the new use-case appears with risk classification + Article 14 oversight description + persona-version + skill-version
    And the AIMS policy is reviewed by the DPO within 7 days
    And the Trust Center's public-safe summary updates within 24 hours
```

## Success Metrics

- Zero EU-AI-Act enforcement actions against CyberOS in first 3 years post-launch.
- ≥ 95% of enterprise prospects' security reviews completed using only Trust Center artefacts.
- Zero exceptions in SOC 2 Type II audit (target — small number of "minor" findings is acceptable).
- ISO 42001 certificate issued within 6 months of P4 entry.
- Sub-processor change subscriber list grows to ≥ 50 subscribers within 90 days.
- Pentest attestation: zero critical/high findings unremediated.

## Sales/CS Summary

<!-- Required when client_visible: true. One paragraph written so a non-engineer can pitch the feature. Plain English. No internal jargon, no module codes, no speculation about future scope. -->

<!-- TODO during implementation PR: write the customer-facing pitch. -->

## Open Questions

- **OQ-GTM-002-01.** Should we offer a paid bug-bounty program at launch or defer? Default: defer 6 months; vulnerability disclosure with public credit only at MVP.
- **OQ-GTM-002-02.** Should we pursue ISO/IEC 27017 (cloud-specific) + 27018 (cloud personal-data) at P4 + 12 months? Default: yes; budget for it.
- **OQ-GTM-002-03.** Should the SOC 2 Type II report cover Privacy Trust Service Criteria? Default: yes (signals strong PII posture; aligns with PDPL + GDPR claims).

## AI Authorship Disclosure

- **Tools used:** Claude Cowork (Anthropic).
- **Scope:** drafted the FR end-to-end against the PRD + SRS; founder reviews and edits before status changes from `ready_for_review`.
- **Human review:** founder (`@stephen-cheng`) — final wording is the founder's responsibility.

## References

- PRD §14.5.2 P4 → GA exit-gate.
- PRD §11.5 compliance regime catalogue.
- ISO/IEC 42001:2023.
- EU AI Act Articles 9-15, 43, 47.
- SRS Decisions Log: DEC-024..DEC-030, DEC-052.
- FR-CP-001/002/003/004/005, FR-OBS-004, FR-AUTH-002, FR-AI-001, FR-GENIE-001/004, FR-REW-001/002, FR-LEARN-002, FR-DOC-001, FR-EMAIL-003, FR-CHAT-001, FR-GTM-001, FR-CORP-001.

---

*ai_authorship: co_authored — drafted by Claude Cowork on 2026-05-03.*
