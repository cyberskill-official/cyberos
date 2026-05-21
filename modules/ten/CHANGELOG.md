# Changelog — TEN

## 2026-05-15 — TEN module page rewritten to Gold (P2 billing slice + residency enforcement + 90-day offboarding contract)

Rewrote `website/docs/modules/ten.html` to Gold. Encodes the research review §7.3 mandate (TEN-billing thin slice at P2/P2 · exit, not P4) — three strategic roles: (1) tenant lifecycle owner with state machine + audit propagation, (2) billing slice P2 thin (Stripe + 3 plans + cost cap) vs P4 full (+ VN-PSP + self-serve + in-app UI), (3) residency enforcement (data lives where law says; cross-leak CI gate = 0).

Key changes:
- Phase chip changed: "P4 long-term" → "P2 thin slice · P4 full"
- Title/meta + hero reframed; phase 0 strategic frame
- Fact-grid extended (8→13 cards: + Strategic role, P2 slice scope, P4 full scope, Residency options, Cross-leak target = 0)
- NEW §0 "The bigger picture" — 3-card layout + tenant lifecycle Mermaid (10 nodes: external customer → TEN → 3 billing rails + 5 modules + audit/CFO) + 9-row auto-vs-human matrix
- NEW §2.5 "P2 thin slice scope" — 12-row capability contrast (P2 thin vs P4 full) + plan-tier × usage budget table (Starter $49/seat · Team $39/seat · Enterprise custom; vertical pack add-on $99/$79/negotiated)
- NEW §2.6 "Residency × jurisdiction matrix" — 4-row infra mapping (sg-1 / eu-1 / us-1 / vn-1 each with Postgres shard, S3 region, AI providers, OBS retention, compliance regime) + cross-leak CI gate spec (200+ property-based test attempts per PR)
- NEW §2.7 "90-day offboarding contract" — 4-phase timeline (Active → Terminating-A 30d → Terminating-B 60d → Terminated day 91+) + signed bundle 6-component export + permanent-delete attestation JSON with Ed25519 signature
- Risks +8 (R-TEN-013..020): P2 slice slip → margin moat delayed (High) · residency change mid-engagement · hostile termination override · Stripe DPA EU residency · plan-downgrade overage surprise · cross-leak CI gap (Critical) · vertical-pack revenue attribution leak · Lumi-pushed pack pricing change
- KPIs +9: P2 slice ship date adherence (= P2 · exit) · vertical-pack revenue share (≥ 30% of ARR by P4 · mid — the moat) · cross-leak rate (= 0 hard floor) · residency drill MTTR (≤ 72h) · plan-downgrade overage handling (= 1.0) · hostile-termination cycle time · VN-PSP coverage (≥ 0.95 at P4) · PCI-SAQ-A scope (= 0; Stripe handles all) · tenant attestation completeness (= 1.0)
- References expanded: 4 in-page sections + 6 cross-module links + AUDIT_AND_PLAN §3.3 + RESEARCH_REVIEW §7.3 (explicit cite of the P2 · exit mandate) + MEMORY_AUTOSYNC_DESIGN.md §6 + feature-request-audit skill + EU AI Act Art. 26 + expanded PDPL article citations

