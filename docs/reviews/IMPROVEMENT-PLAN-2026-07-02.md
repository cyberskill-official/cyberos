# Improvement plan - 2026-07-02 (companion to MODULE-REVIEW-2026-07-02.md)

Awaiting operator approval before execution. Phases are ordered by risk; each item names its deliverable
and gate. Estimates are focused build-time on the existing Mac-gate loop.

## P0 - production truth (do first, ~half a day)

1. Eval 502: DOWNGRADED after reading deploy.sh - eval is already intentionally stopped (DEPLOY_EVAL
   gate; Supabase pooler headroom) and status.html handles it as "not deployed". Action reduced to
   documenting the gate in the review. [done in review]
2. Caddy reload on deploy: root cause found - the existing reload swallowed its errors, so a failing
   reload left stale config live (/status/ai 404). FIXED: reload surfaces errors + falls back to a caddy
   restart; next deploy self-heals. Deliverable: /status/ai routed after the next push. [done]
3. Attachment volume backups: nightly tar of chat-attachments to a dated file (VPS cron) + a documented
   restore path; note Supabase PITR coverage for the DBs in the same doc. Deliverable:
   docs/deploy/backups.md + cron in deploy/vps/. [1-2h]
4. Dependabot triage: list the 11 advisories, patch the safe upgrades (likely npm), ledger any accepted
   risks. Deliverable: 0 unreviewed advisories. [1-2h]

## P1 - status integrity + grooming (the approval covers doing this immediately after P0, ~a day)

5. Task status corrections (both directions), one commit:
   - flip to done: TASK-CHAT-101, TASK-AUTH-110, TASK-AI-003, TASK-AI-005 (+ TASK-CUO-204 per its gate);
   - close superseded (move to _archive/): TASK-CHAT-001..013 (native chat replaced the fork; keep as
     history), TASK-APP-001..007 (superseded by the React console; add pointer);
   - TASK-EVAL-001 -> built_blocked_on_legal (or agreed equivalent) + prod-container note;
   - TASK-MCP-004: decide flip-with-ledgered-deferral vs keep implementing (operator call, one line);
   - fix double-status frontmatter (TASK-SKILL-111..115, TASK-PROJ-012) to one canonical value;
   - normalize odd statuses (needs_human/completed/delivered/fixed/ready) into the fixed vocabulary:
     draft | ready_to_implement | implementing | done | superseded | blocked.
6. Re-home still-wanted intents from the superseded pile as NEW native-chat tasks (unbuilt, draft):
   Slack import, Zalo import, mobile push (exists as intent in push.rs), DSAR export, Lumi @-mention.
7. As-built notes on drifted done tasks (memory AGE/paths, skill broker consolidation, proj paths): a
   3-5 line "As built (2026-07-02)" block per affected task - no spec rewrites. [~15 tasks]
8. Orphan cleanup: delete services/eval_writetest; archive or delete services/chat-legacy-mattermost
   (operator call - it is the retired fork); confirm or remove services/business-suite; move all
   *.audit.md to docs/tasks/_audits/.
9. Regenerate BACKLOG.md from the corrected frontmatter (script or by hand) so the backlog lists ONLY
   not-yet-implemented + new tasks, grouped by module, with the deployed/built/draft distinction; retire
   remaining-build-plan.md into it. Update CONTINUE-HERE.md + let roadmap.html re-render from the fixed
   frontmatter.

## P2 - hardening follow-ups (separate confirmations, not part of this pass)

10. OBS-007 runbook-URL fix: allowlist runbook URLs against the KB index; reject/blank fabricated ones;
    add the failing case to the smoke. Precondition for deploying obs-router. [half day]
11. Review pass on the parallel-session chat commits (i18n catalog completeness VN/EN, prefs edge cases,
    drawer a11y). [1-2h]
12. AI activation when ready: VPS resize + COMPOSE_PROFILES=llm per the existing runbook - flips
    translate/summarize/replies live with no code change. [operator infra action]
13. Theme eyeball pass (both themes) - token tweaks as needed. [minutes per tweak]
14. Next feature candidates from the groomed backlog, in rough leverage order: chat i18n polish round 2,
    mobile push (FCM/APNS or web-push), DSAR export, TURN + group calls, moderation console, memory
    recall service container, MCP DB slice.

## Explicit non-actions (ledgered, deliberate)

- Eval stays counsel-gated regardless of container fix (governance-first).
- MCP TASK-005..008 stay implementing until the DB slice lands.
- Single-VPS topology accepted at team scale (RTO = redeploy; backups make it survivable).
- Cloud AI provider keys remain deferred; local inference path only.
