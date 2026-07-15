---
task_id: TASK-MEMORY-104
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.0/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; task-audit skill §3.12 compliant)
---

## §1 — Verdict summary

TASK-MEMORY-104 expanded from 81 lines to ~830. Added 8 §1 clauses (#5 dashboard content; #10 update manifest signing; #11 OS-standard config dirs; #12 headless mode; #13 bundle size budget; #14 opt-in crash reporting; #15 multi-window; #16 localisation). 8 §2 rationale paragraphs. Full Tauri config + Rust commands + supervisor + tray + GHA workflow + signing scripts in §3. 17 ACs. Mixed verification (D-style demos + Rust + TS tests). 21 failure modes. 12 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — Update manifest signing not specified (rollback class)
First-pass §1 #2 said "auto-update" without signing/verification. Update server compromise = malicious binary. Resolved: §1 #10 + Ed25519 signing + rollback on signature failure; AC #5.

### ISS-002 — Sync daemon supervision unspecified
First-pass §1 #3 said "as internal Tauri-managed process" without supervision. Daemon panic = sync stops silently. Resolved: §3 SyncSupervisor + exponential backoff restart; AC #12.

### ISS-003 — Headless mode for Linux server installs missing
First-pass had Linux deb/AppImage but assumed UI. Server installs need no-UI mode. Resolved: §1 #12 + --headless flag; AC #14.

### ISS-004 — Bundle size budget unspecified
First-pass had no budget. Tauri produces small binaries by design but easy to bloat. Resolved: §1 #13 + 30MB Mac / 25MB Windows; CI enforces.

### ISS-005 — Crash reporting opt-in/out unspecified
First-pass had no telemetry mention. Default-on would violate privacy. Resolved: §1 #14 opt-in default off; sentry-rust integration.

### ISS-006 — Multi-window unspecified
Search-in-window UX needs multi-window. First-pass single-window assumed. Resolved: §1 #15 + AC #16.

## §3 — Resolution

All 6 mechanical revisions applied. **Score = 10/10.**

---

*End of TASK-MEMORY-104 audit.*
