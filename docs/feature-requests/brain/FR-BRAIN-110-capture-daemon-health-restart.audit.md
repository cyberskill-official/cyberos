---
fr_id: FR-BRAIN-110
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

## §1 — Verdict summary

FR-BRAIN-110 authored direct-to-10/10. ~680 lines. 15 §1 clauses (systemd + launchd units, exp-backoff, /healthz schema, 4 unhealthy conditions, 60s sweeper, signal handlers, crash-count file, metrics, OTel spans, supervisor_event audit, FR-OBS-007 integration, status + logs CLI, idempotent install). 9 §2 rationale paragraphs. Full systemd unit + launchd plist + healthz handler + sweeper + installer bash in §3. 24 ACs. 4 healthz tests + 2 sweeper tests + bash e2e. 18 failure modes. 9 implementation notes.

## §2 — Findings (all resolved during authoring)

### ISS-001 — Custom supervisor vs OS-native
A naive author might write a custom Rust supervisor. Resolved: §1 #1 + DEC-161 explicit prohibition; systemd/launchd units in §3.

### ISS-002 — Exp backoff on launchd (no native support)
launchd has no `RestartSec` equivalent. Without spec, would either flap or block. Resolved: §1 #2 + #7 + daemon-side crash-count file with `sleep(min(2^count, 300))`; AC #6 verifies daemon-side backoff.

### ISS-003 — /healthz 200/503 conditions not enumerated
Could return 503 for any failure → operators can't distinguish causes. Resolved: §1 #4 enumerates 4 specific conditions; response body includes `reasons[]`; AC #8 #9 #10 cover each.

### ISS-004 — Sweeper scope ambiguous
Which /tmp paths get pruned? With what TTL? Resolved: §1 #5 enumerates 4 categories with explicit TTL; §3 `prune_dir_older_than` + `maybe_reset_crash_count`.

### ISS-005 — Supervisor events not audited
Operators investigating "when was the daemon last down" had to grep journalctl. Resolved: §1 #10 first-class BRAIN audit row `brain.capture_supervisor_event` with kind=started|reloaded|exited; AC #17 + #18.

### ISS-006 — Health-state staleness
Doctor check runs at boot; without periodic re-check, `/healthz` would lie about stale invariant failures. Resolved: §1 #4 + §3 `last_doctor` cached behind RwLock with 60s refresh; AC #8 covers cached-fail path.

## §3 — Resolution

All 6 mechanical concerns addressed during authoring. **Score = 10/10.** Ready to ship + transition `draft → accepted`.

---

*End of FR-BRAIN-110 audit.*
