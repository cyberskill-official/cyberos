---
task_id: TASK-PLUGIN-008
audited: 2026-05-19
verdict: PASS (after revision)
score_pre_revision: 8.0/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 7
template: engineering-spec@1
---

## §1 — Verdict summary

Marketplace distribution — cyberos-plugin publish surface pushing to plugins.cyberskill.world (OCI) with public-tier mirror to agentskills.io, 3 visibility modes, 70/30 revenue share, vetted-by-CyberSkill JWT badge, yank-not-delete model. CLI in this FR; full marketplace server scaffolded (FR-PLUGIN-008a covers UI/search/billing). 500 lines, 14 §1 clauses, 22 ACs, 4 test files, 15 failure modes, 10 implementation notes. 7 issues resolved (OCI compatibility unlocks Docker/GitHub/AWS ecosystem; agentskills.io mirror amplifies Strategy Level 1 reach; 3 visibility modes cover all 4 strategy levels; locked publish-time re-validation closes tampered-CLI threat; JWT-signed vetted badge is unforgeable client-side; version monotonicity makes 'latest' tag deterministic; yank-not-delete preserves install reproducibility). **Score = 10/10.**

## §2 — Findings (all resolved)

### ISS-001 — Publish-time validation can be bypassed
Modified `cyberos-plugin` binary could skip local validation. Resolved: §1 clause 2 + DEC-2475 — server-side re-runs doctor + manifest validation + Sigstore verify; AC #1-2.

### ISS-002 — Public plugins miss Anthropic ecosystem visibility
Without mirror, public plugins are CyberSkill-walled. Resolved: §1 clause 4 + DEC-2472 — agentskills.io mirror default-on for public; AC #4.

### ISS-003 — Private plugin leaks via mirror
Without strict visibility gating, private/enterprise content can leak. Resolved: §1 clause 12 + AC #5 — private/enterprise MUST NOT mirror.

### ISS-004 — Revenue split obscure
Author doesn't know what % they get. Resolved: §1 clause 7 + DEC-2473 — default 70%; <70% warns; >100% rejects; AC #15-16.

### ISS-005 — Vetted badge forgeable
Without cryptographic anchor, vetted claim is text. Resolved: §1 clause 5 + DEC-2474 — JWT signed by CyberSkill marketplace key; client-side verify; AC #11-12.

### ISS-006 — Version disorder confuses install
Latest tag ambiguous when publish order doesn't match SemVer order. Resolved: §1 clause 9 + monotonicity check at registry; AC #8.

### ISS-007 — Delete breaks reproducibility
Permanent delete invalidates SHA-256 references in existing installs. Resolved: §1 clause 10 + yank model; AC #9-10; §11.7 explicit rationale.

## §3 — Resolution

All 7 ISS findings resolved by extending §1 (clauses 2, 4, 5, 7, 9, 10, 12), defining the OCI media types + Postgres schema, designing JWT vetted-badge format, and writing 4 integration tests. Marketplace server scaffolded; full server in FR-PLUGIN-008a.

Final score: **10/10.**

*End of TASK-PLUGIN-008 audit.*
