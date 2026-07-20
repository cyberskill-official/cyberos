# TASK-CHAT-269 review packet - status-drift reconciliation (2026-07-12)

## Situation
Implementation shipped in commit f62b018 ("feat(chat): UGC controls - reporting, blocking, moderation queue") by a prior session, but the task never left ready_to_implement - discovered when the ship queue selected it. This packet is the clause-by-clause verification of the EXISTING code; nothing was re-implemented.

## Clause verification (spec §1 #1-21 vs services/chat/src/moderation.rs @ f62b018 lineage)
#1 three admin routes PASS (lib.rs:119+ /v1/chat/admin/reports[/:id][/resolve]); #2 tenant-admin/root-admin gate, fail-closed PASS (auth::require_moderator; module header cites the clause); #3 channel roles grant nothing PASS (same gate; no channel-role path); #4 grouped queue with report_count PASS (grouped query, report_count field); #5 severity rank as SQL CASE, self_harm first PASS (SEVERITY_CASE + severity_rank_matches_sql_case drift test); #6 status/reason filters + opaque keyset cursor, no unpaginated mode PASS (base64 keyset cursor); #7 immutable snapshot + original_present PASS; #8 group-context only when admin already a member PASS (module header: "must not become a skeleton key"); #9 DM carve-out - single reported message only PASS (dedicated header doctrine); #10 blocked-set not applied to admin routes PASS; #11 closed action enum + note <= 1000 PASS; #12 CAS on status='open', loser gets winner's outcome, no second audit row PASS (cas_lost flag, WHERE id=$1 AND status='open'); #13 sibling reports resolved in same transaction PASS (sibling_report_ids); #14 remove_member = channel-scoped PASS; #15 one chat.report_resolved row + effect rows separate PASS; #16 no snapshot/note on audit rows PASS; #17 90-day purge (purge_after + migration 0016) PASS - matches published retention; #18 client nav only for admins PASS (isModerator, fail-closed note); #19 content-policy links in Settings + report dialog PASS; #20 note/detail rendered as text PASS (React text nodes, no dangerouslySetInnerHTML - grep clean); #21 en+vi strings PASS (lib/i18n.ts "moderation (TASK-CHAT-269)" block).

## Recorded deviations (repo conventions, newest wins)
- apps/web/src/pages/Moderation.tsx (spec said routes/) - matches the app's actual pages/ convention.
- apps/web/src/lib/i18n.ts (spec said src/i18n.ts) - actual i18n home.
- Migration landed as 0015_chat_blocks.sql + 0016_chat_reports_retention.sql (spec reserved 0015 for retention; 268's blocks migration took it first). Content per spec.

## Test evidence
services/chat/tests/moderation.rs: 850 lines, 23 test fns incl. the severity-drift canary. NOT run in this session (no Rust toolchain in the sandbox) - evidence source is the CI Rust gate on the push that carried f62b018, plus an operator-run `cargo test -p chat` if desired. This gap is named, not papered.
