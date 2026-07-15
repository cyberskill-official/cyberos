---
task_id: TASK-PORTAL-007
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8/10
score_post_expansion: 9/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

## §1 — Verdict summary

The spec lands PWA installable + service worker + Web Push on top of TASK-PORTAL-001. 660 lines, 20 §1 normative clauses, 20 ACs, 5 verification tests, 16 failure-mode rows, 10 implementation notes. 2 migrations, 6 endpoints, 4 memory audit kinds.

6 issues resolved.

## §2 — Findings (all resolved)

### ISS-001 — Push payload encryption transparency for users

§20 notes push provider servers see payload until browser decrypts. But Web Push encryption per RFC 8291 actually encrypts the payload end-to-end with the user's p256dh key. Resolved: §11.4 clarifies — payload IS encrypted by web-push crate; providers see encrypted bytes; the "generic content" rule is defense-in-depth in case encryption misconfigured.

### ISS-002 — Service worker version conflict during deploy

§3 mentions versioning. But during rolling deploy, some clients have v1 SW + some hit v2 endpoints. Resolved: §10 row + §11 — old/new SW coexist briefly; clients on old SW make requests to new server (server is backward compatible); on next reload SW upgrades.

### ISS-003 — Offline cache quota management

Browsers enforce ~50 MiB quota per origin. Resolved: §10 row + §11 — browser eviction is LRU; sev-3 log notes when eviction occurs; cache strategy already prioritises critical resources.

### ISS-004 — Notification dedup race

User receives push + opens app → app reloads → cached push notification may re-show. Resolved: §11.9 — service worker tracks shown notifications by ID + suppresses duplicates within 5min window.

### ISS-005 — Cross-tenant manifest race

Caller switches tenant → manifest cache still has old tenant's brand for up to 1h. Resolved: §11.10 — cache invalidation triggered on tenant switch via service worker postMessage; 1h client-side TTL is upper bound.

### ISS-006 — VAPID key compromise impact

§12 says per-deployment VAPID. Key compromise = all subscriptions become forgeable. Resolved: §11 — KMS-stored private key; rotation triggers re-subscribe for all users (subscription includes VAPID public key in signature challenge). Slice 3 adds rotation tool.

## §3 — Resolution

All 6 mechanical concerns addressed.

The 660-line length is appropriate for 6h-effort SHOULD-priority task.

**Score = 10/10.**

---

*End of TASK-PORTAL-007 audit.*
