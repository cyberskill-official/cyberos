---
id: NFR-CHAT-004
title: "CHAT Mattermost fork upstream-merge cadence — monthly minor rebase; quarterly major"
module: CHAT
category: maintainability
priority: SHOULD
verification: I
phase: P0
slo: "Fork rebased onto upstream minor monthly; major version quarterly; security CVEs within 14 days"
owner: CTO
created: 2026-05-18
related_tasks: [TASK-CHAT-001]
---

## §1 — Statement (BCP-14 normative)

1. The CyberOS Mattermost fork **MUST** be rebased onto upstream minor releases at least monthly (the first business day after the upstream minor ships).
2. Major version rebases **MUST** be performed at least quarterly. The platform commits to never being more than one major version behind upstream.
3. Security CVEs in upstream **MUST** be merged within 14 days of CVE publication, regardless of regular rebase cadence.
4. Each rebase **MUST** be verified by the full CHAT integration test suite passing; merge-conflict resolution requires CTO sign-off if it touches the memory bridge or AUTH integration plugins.
5. The fork **MUST** maintain a `MATTERMOST_UPSTREAM_VERSION.md` file at repo root noting current upstream alignment and next planned rebase date.

## §2 — Why this constraint

Forking Mattermost is a long-term maintenance commitment — without disciplined rebase cadence, the fork drifts into untenable merge conflicts and security holes pile up. Monthly minor rebases keep delta small; quarterly major rebases prevent the multi-major drift trap. The 14-day CVE rule overrides the calendar rhythm — security patches don't wait. The "never more than one major behind" rule sets the contractual maintenance bound the platform commits to its tenants.

## §3 — Measurement

- File inspection — `MATTERMOST_UPSTREAM_VERSION.md` must be current; quarterly audit.
- Repo metric — git log on the fork's main branch should show an "upstream rebase" commit at least monthly.
- CVE log — `docs/compliance/mattermost-cves.md` carries each upstream CVE and its merge date; sev-3 if any > 14 days unaddressed.

## §4 — Verification

- Inspection (I) — quarterly review by CTO of upstream alignment + CVE table.
- CI integration test (T) — runs on every rebase PR; asserts CHAT integration tests pass.

## §5 — Failure handling

- Rebase delayed > 6 weeks → sev-3 ticket to CTO; allocate engineering time.
- CVE unpatched > 14 days → sev-2; emergency rebase or selective backport.
- Quarterly major rebase skipped → sev-2; fork drift risk; CTO + CEO decide whether to deprecate fork.

---

*End of NFR-CHAT-004.*
