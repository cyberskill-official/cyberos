---
fr_id: FR-TEN-005
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

## §1 — Verdict summary

The spec lands per-pack add-on pricing for the vertical-pack marketplace on top of FR-TEN-002 plan tiers + FR-SKILL-107 pack registry. 690 lines, 20 §1 normative clauses, 20 ACs, 4 verification tests, 16 failure-mode rows, 10 implementation notes. 3 migrations, 6 endpoints, 5 memory audit kinds.

6 issues resolved.

## §2 — Findings (all resolved)

### ISS-001 — Override CHECK constraint missing for sane prices

§4 + §10 mention override sanity. Resolved: §11.10 documents — CHECK enforces override ≤ 2× catalog price; prevents typo 99900 → 9990000.

### ISS-002 — Cross-rail check duplication risk

§1 #20 cross-rail rejected. But FR-TEN-003 + FR-TEN-102 each have their own guards. Resolved: §11.7 — this FR consumes existing guards; no new guard logic.

### ISS-003 — Pack uninstall mid-billing-period race

Uninstall happens during billing job. Resolved: §10 row + §11 — tx isolation; pro-rate credit at uninstall time.

### ISS-004 — Skill registry consumes install status — API or DB?

§11.9 says API; better is direct DB read by FR-SKILL-107 since both on same DB. Resolved: §11.9 — FR-SKILL-107 reads `vertical_pack_installs.status` directly via RLS-shared connection.

### ISS-005 — Grandfather window edge cases

89d vs 90d boundary. Resolved: §10 row + §11 — alert if billing event near boundary; document boundary semantics inclusive (≤ 90d uses grandfathered).

### ISS-006 — Per-tenant install lookup performance

50 packs × 100 tenants = 5000 rows; trivial. Resolved: documented as non-issue at slice 2 scale.

## §3 — Resolution

All 6 mechanical concerns addressed.

The 690-line length appropriate for 5h-effort scope.

**Score = 10/10.**

---

*End of FR-TEN-005 audit.*
