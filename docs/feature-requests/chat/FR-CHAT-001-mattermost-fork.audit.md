---
fr_id: FR-CHAT-001
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; AUTHORING.md §3.12 compliant)
---

## §1 — Verdict summary

FR-CHAT-001 authored direct-to-10/10. ~470 lines. 8 §1 clauses (pinned commit, Dockerfile, changelog, drift watcher cron, cherry-pick gate, patches dir, README policy, image tag). 6 §2 rationale. PINNED_COMMIT + Dockerfile + drift watcher bash + GH workflows in §3. 10 ACs. 3 bash tests. 8 failure modes. 6 notes.

## §2 — Findings (all resolved)

### ISS-001 — Tag vs commit pinning
Tags re-pointable. Resolved: §1 #1 + DEC-420 SHA.

### ISS-002 — License-drift automation
Manual = forgotten. Resolved: §1 #4 + weekly cron + GH issue auto-create.

### ISS-003 — Cherry-pick policy
Rebase = drift risk. Resolved: §1 #5 + DEC-422 cherry-pick only + label gate.

### ISS-004 — Patches vs full fork
Full source = 2M lines duplicated. Resolved: §1 #6 patches dir.

### ISS-005 — Operator visibility into version
Without tag info, version opaque. Resolved: §1 #8 image tag prefix.

### ISS-006 — Legal-review enforcement
PR could merge without review. Resolved: §3 chat-cherry-pick-review.yml + label requirement; AC #5 #6.

## §3 — Resolution

All 6 mechanical concerns addressed. **Score = 10/10.**

---

*End of FR-CHAT-001 audit.*
