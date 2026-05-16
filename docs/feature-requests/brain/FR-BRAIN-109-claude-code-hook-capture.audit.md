---
fr_id: FR-BRAIN-109
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

## §1 — Verdict summary

FR-BRAIN-109 authored direct-to-10/10. ~720 lines. 14 §1 clauses (3 hook events + redaction ruleset + fail-closed + 50ms budget + socket-IPC + trace correlation + JSON schema + opt-in installer + uninstall/status + OTel + metrics). 10 §2 rationale paragraphs. Full Cargo.toml + hook dispatch + redactor + socket emit + bash installer in §3. 24 ACs. 5 e2e + 8 redactor unit tests. 17 failure modes. 8 implementation notes.

## §2 — Findings (all resolved during authoring)

### ISS-001 — Which Claude Code hooks to subscribe to
Could have subscribed to all 7 hook events (UserPromptSubmit, PreToolUse, PostToolUse, Notification, Stop, SubagentStop, SessionStart). Without scoping, chain pressure would dominate. Resolved: §1 #2 + #3 lock to 3 events; §2 rationale paragraph explains the cost/value tradeoff.

### ISS-002 — Tool args raw vs hashed
Operators want to know "what command ran" — but raw args often contain secrets. Resolved: §1 #2 hashed-only; §2 rationale + DEC-151 source; AC #2 asserts raw NOT present.

### ISS-003 — Redaction failure mode (open vs closed)
Fail-open = leak risk; fail-closed = capture loss. Resolved: §1 #5 fail-closed + `brain.claude_capture_redaction_failed` row; AC #12 covers the panic path.

### ISS-004 — Latency vs reliability tradeoff for chain write
Direct chain write from hook = correctness but 200ms latency; queued via socket = fast but eventual emit. Resolved: §1 #6 + #10 socket IPC to FR-BRAIN-107 daemon; AC #15 latency budget; §2 explains the architectural choice.

### ISS-005 — Trace correlation across hook invocations
Three hooks per session need shared trace_id. /tmp file cache approach. Resolved: §1 #7 + §3 `trace_id_for_session()`; AC #13 same-session; AC #14 different-session.

### ISS-006 — Installer idempotency + user-hook coexistence
Naively overwriting user's settings.json destroys their hooks. Resolved: §1 #1 + §3 bash installer uses `jq -s '.[0] * .[1]'` deep merge; AC #19 idempotent + AC #20 preserves user hooks + AC #21 clean uninstall.

## §3 — Resolution

All 6 mechanical concerns addressed during authoring. **Score = 10/10.** Ready to ship + transition `draft → accepted`.

---

*End of FR-BRAIN-109 audit.*
