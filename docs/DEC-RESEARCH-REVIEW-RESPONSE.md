---
id: DEC-RESEARCH-REVIEW-RESPONSE-001
title: "Response to the 2026-05-14 senior product/engineering audit"
kind: decision
actor: Stephen Cheng (CEO)
authored: 2026-05-15
chain_anchor: pending (lands when BRAIN audit chain ingests this file)
supersedes: []
related_decisions: [DEC-053, DEC-070, DEC-071, DEC-072, DEC-073, DEC-074]
research_review: docs/archive/2026-05-14/RESEARCH_REVIEW.md
status: locked
---

# Response to the senior product/engineering audit of 2026-05-14

This is the founder-level decision memo recording what CyberOS accepted, rejected, and deferred from the senior product/engineering audit (`docs/archive/2026-05-14/RESEARCH_REVIEW.md`). It is the single artefact that future contributors and auditors should read to understand why the strategy looks the way it does after the May 2026 inflection. Every section below points at a specific review finding by section number and records the decision in BCP-14 terms.

---

## §1 — Strategic coherence findings (review §1)

| Finding | Decision | Status |
|---|---|---|
| **§1.1** Ecosystem-as-a-Service thesis fragile at L4 → L5 leap | ACCEPTED. The marketplace is announced at level 4 but treated as recruiting/PR for two years; serious investment gated at 50 paying tenants. Documented in milestones.html · Trajectory section. | Locked |
| **§1.2** Hosted SaaS gated too late (P4); should ship "managed single-tenant" at P2 | ACCEPTED. **DEC-058 revised:** TEN-billing thin slice at P2 (not P4). BACKLOG.md §4 P2.4 reflects this. | Locked |
| **§1.3** P0 → P1 descope gate missing | ACCEPTED. P0 → P1 descope gate added to milestones.html with 4-question scorecard. LEARN/HR/EMAIL slice 3 are the deferral candidates in order. | Locked (2026-05-15) |
| **§1.3** P1 8-module batch is unrealistic with 12 Members | ACCEPTED with mitigation. Descope gate above is the structural mitigation; HR split (HR-roster P1, HR-full P2) is the leading expected outcome. | Locked |
| **§1.4** Vietnamese wedge compounds, but `RSK-EXT-09` (VN export-control shift) is missing | ACCEPTED. Added as `R-EXT-09` in risk-register.html along with R-EXT-10..15. | Locked (2026-05-15) |

---

## §2 — Architecture findings (review §2)

| Finding | Decision | Status |
|---|---|---|
| **§2.1** Layer 1 → Layer 2 source-of-truth boundary undocumented | ACCEPTED. `docs/BRAIN_LAYER_2_SOURCE_OF_TRUTH.md` written. DEC-070..074 lock the rules. | Locked (2026-05-15) |
| **§2.1** Layer 2 pgvector cost ceiling incompatible with managed RDS at 50 tenants | ACCEPTED. Plan for self-hosted pgvector dedicated VMs by P3. Captured in §9 of the Layer 2 one-pager. | Locked |
| **§2.2** Single-Genie persona-confusion at boundary (REW/HR/LEARN data leak risk) | ACCEPTED. Add `cuo.boundary_test` doctor invariant on a corpus of cross-boundary queries. Captured as a follow-up FR in CUO Phase 2 (BACKLOG.md P1.3). | Locked |
| **§2.3** HR split, PROJ+TIME merge, OKR defer, ESOP promote, DOC eIDAS defer | ACCEPTED. Updated module phasing in BACKLOG.md: ESOP at P2, OKR at P3, DOC eIDAS (FR-DOC-002) at P4. HR split formalised in the descope gate. PROJ+TIME stay separate (decision: low cost of separation, high cost of merge + future split). | Locked |
| **§2.3** TEN-billing at P2 (not P4) | ACCEPTED. See §1.2 above. | Locked |
| **§2.4** AUTH should not be P0 #1 | ACCEPTED. **DEC-032 revised:** AI Gateway ships before AUTH. P0 slice 1 = AI Gateway (FR-AI-001..005); P0 slice 2 = OBS; P0 slice 3 = AUTH stub. Documented in milestones.html and BACKLOG.md. | Locked (2026-05-14) |

---

## §3 — Compliance findings (review §3)

| Finding | Decision | Status |
|---|---|---|
| **§3.1** Decree 20/2026 is fictitious; should be PDPL Art. 38 grace period | ACCEPTED. **DEC-053 revised:** all Decree 20/2026 references swept to PDPL Art. 38. Affected files: compliance.html, milestones.html, index.html, glossary.html, skill.html. | Locked (2026-05-15) |
| **§3.1** A05 cross-border-transfer is 60-day post-audit (PDPL Art. 20), not 15-day pre-form | ACCEPTED. compliance.html §3.1 updated; "ngày-15" reference removed. | Locked (2026-05-15) |
| **§3.1** PDPL Art. 7 personal-data-sale ban not surfaced | ACCEPTED. Added one-line policy in CRM + PORTAL via compliance.html §3.1. Each module page will add a tenant-visible "no data sale" affirmation at P3. | Locked (2026-05-15) |
| **§3.1** PDPL took effect 1 Jan 2026; not P2 graduation | ACCEPTED. Recharacterised in compliance.html: PDPL is the operative regime; CyberSkill operates under Art. 38 grace until P2 graduation. | Locked (2026-05-15) |
| **§3.2** Breach-notification 72h + data-subject notification for biometric/financial | ACCEPTED. compliance.html §3.1 updated. | Locked |
| **§3.3** Formal DPO appointment at P0 not required by regulator | ACCEPTED. Founder serves as DPO through P1; formal DPO appointed at P2 entry. compliance.html updated. | Locked |

---

## §4 — UX findings (review §4)

16 UX defects flagged in review §4. Tracked as task #38 in the project todo list. Status: **deferred to a focused UX sprint.** This memo records the decision to NOT batch the UX fixes into the current strategy-revision pass; they are real but they don't change strategic decisions and can be tackled in a focused 1-2 day pass.

**Acceptance:** all 16 defects accepted as bugs to fix. None require strategic decisions.

---

## §5 — Risk register additions (review §5)

7 risks added to risk-register.html: R-EXT-09 (VN cross-border data export), R-EXT-10 (Anthropic Skills churn), R-EXT-11 (Bedrock SG capacity), R-EXT-12 (VN hire latency), R-EXT-13 (Mattermost license), R-EXT-14 (Stalwart bounce rate), R-EXT-15 (eIDAS QTSP partner). Status: **locked 2026-05-15.**

---

## §6 — Compliance regulator-language fixes (review §6.2)

Three Vietnamese PDPL citation errors corrected:
1. Decree 20/2026 → PDPL Art. 38 grace period (review §6.2 #1)
2. A05 15-day pre-form → 60-day post-audit per PDPL Art. 20 (review §6.2 #3)
3. PDPL took effect 1 Jan 2026 (review §6.2 #1)

Plus added: PDPL Art. 7 personal-data-sale ban surfaced in CRM + PORTAL.
Plus added: PDPL biometric/financial-incident data-subject notification clause.

Files touched: compliance.html, glossary.html, milestones.html, index.html, skill.html. Status: **locked 2026-05-15.**

---

## §7 — Sequencing fixes (review §7)

| Finding | Decision | Status |
|---|---|---|
| **§7.1** AI Gateway BEFORE AUTH | ACCEPTED. P0 slice 1 = AI Gateway. | Locked |
| **§7.2** Compliance Cockpit needs an FR | ACCEPTED. Added to backlog at P1 (CP module placement). | Locked |
| **§7.3** TEN-billing thin slice at P2 | ACCEPTED. BACKLOG.md §4 P2.4 reflects this. | Locked |

---

## §8 — What was NOT accepted

| Review proposal | Why rejected | Reconsideration trigger |
|---|---|---|
| Merge PROJ + TIME | Low cost of separation now vs. high cost of a future split if the project-accounting workflow diverges. Two thin modules > one thick one. | Reconsider at P3 if PROJ + TIME share &gt; 90% of their schema. |
| Make DPO a formal P0 role | Regulator does not require it under PDPL Art. 38; cost is $50–80k/year for no compliance benefit. Founder-as-DPO is the lawful path through P1. | Move to formal DPO at P2 entry (already planned). |
| Defer LEARN entirely to P2 | Premature; descope gate (§1.3) handles this contingency. If P0 ships clean and hires arrive on time, LEARN can ship in P1. | The descope gate triggers — automatic deferral if any scorecard Red. |
| OpenAI Apps SDK as a competitive risk needing immediate response | Already covered by R-EXT-S2; CyberOS's Vietnamese-market wedge + audit-chain depth are the defenses. No immediate strategic change. | Monitor quarterly via R-EXT-S2 review. |

---

## §9 — New decisions locked by this memo

| ID | Decision | Source |
|---|---|---|
| **DEC-053** (revised) | SME grace via PDPL Art. 38, not the fictitious Decree 20/2026 | §3 |
| **DEC-058** (revised) | TEN-billing thin slice at P2; full multi-tenant at P4 | §1.2 |
| **DEC-070** | Layer 1 is the only source of truth for BRAIN | §2.1 |
| **DEC-071** | The Merkle chain is anchored at Layer 1 only | §2.1 |
| **DEC-072** | Layer 2 rebuild-from-Layer-1 is a REQUIRED CI gate | §2.1 |
| **DEC-073** | Layer 2 readers fall back to Layer 1 for read-your-writes | §2.1 |
| **DEC-074** | No application code may write directly to Layer 2 | §2.1 |
| **DEC-075** | P0 → P1 descope gate runs at P1 · start; LEARN → P2, HR split as default deferrals | §1.3 |
| **DEC-076** | DPO is Founder through P1; formal DPO at P2 entry | §3 |
| **DEC-077** | Self-hosted pgvector on dedicated VMs by P3 (not managed RDS) | §2.1 |
| **DEC-078** | Marketplace investment gated at 50 paying tenants | §1.1 |

---

## §10 — Open questions parked

These are research-review findings that need more information before deciding:

1. **OQ-15:** Is the Vietnamese-market wedge sustainable past P3 if VN engineering hire latency forces foreign hiring? — re-evaluate at P2 entry.
2. **OQ-08:** PORTAL customer-agent MCP scope-of-consent boundary — needs legal counsel input before P3. Tracked as R-EXT-L5.
3. **OQ-06:** On-prem SKU at advanced tier — defer evaluation until first client demand surfaces.

---

## §11 — Acknowledgement to the reviewer

The 2026-05-14 audit is the highest-quality external technical critique CyberOS has received. It is preserved verbatim at `docs/archive/2026-05-14/RESEARCH_REVIEW.md` and will not be edited. This response memo and the audit are companion artefacts: read together, they form the canonical record of *why* the strategy looks the way it does after the May 2026 inflection.

The reviewer's framing — "tell me what's broken before I lock this" — is the operating posture CyberOS should maintain at every phase gate. Future audits will land at P1 · exit and P2 · exit (per the phase-exit gate sequence in milestones.html); this memo serves as the template for those responses.

---

*End of DEC-RESEARCH-REVIEW-RESPONSE-001. This file is immutable once landed; future revisions are new files that supersede this one.*
