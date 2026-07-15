---
task_id: TASK-CHAT-006
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 14
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; task-audit skill §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (no-line-cap expansion per task-audit skill §0; ISS-007..014 added)
---

## §1 — Verdict summary

TASK-CHAT-006 expanded 2026-05-16 P.M. via strict-redo no-line-cap pass. Current scope: 30 §1 clauses (CLI, checkpoint table, 8 steps, atomic checkpoint, resume, dedup, audit kinds, progress, dry-run, idempotency, fail-fast vs retry, RLS, metrics, MM timestamp preservation, thread reconnection + orphan handling, file dedup by workspace+file_id, file-metadata PII redaction, Slack→MM channel-type mapping, reactions+pin+edit preservation, imported-post markers, per-step parallelism caps, Retry-After honour, zip CRC validation, --abort+--cleanup commands, post-import sample verification, step_metrics JSONB, workspace context). 16 §2 rationale paragraphs. §3 contains: SQL schema with step_metrics + status + cancellation flag + verification_sha column + RBAC, clap CLI definition with all subcommands + flags, parse.rs with full Slack schema (SlackUser, SlackUserProfile, SlackChannel, SlackTextBlob, SlackMessage, SlackEditedRef, SlackReaction, SlackFileRef, SlackInitialComment) + ts→ms conversion, checkpoint.rs with start_or_resume + complete_step + check_cancellation + finish + abort + cleanup + ImportCounts/CleanupCounts, file_download.rs with semaphore-based parallelism + per-file SHA tracking + dedup table insert, run_all with 8-step orchestration honoring cancellation + skip_files + verification sample. 40 ACs. §5 contains 18 named test bodies covering happy/resume/idempotent/dry-run/timestamp-preservation/threads/orphans/file-dedup/redacted-metadata/channel-type-mapping/reactions/custom-emoji-fallback/props-markers/retry-after/rate-limit-exhaustion/corrupt-zip/abort/cleanup/sampling/sampling-corruption + property test on slack_ts parsing. §6 deepens with 12 wiring subsections (process model, MM API access pattern, auth/token, file download auth, memory bounds, slack ts uniqueness assumption, workspace inference, rate-limit pool, cleanup transactional semantics, sample verification design, import_job_id propagation, failure routing matrix). §8 lists 7 example payloads. §10 lists 48 failure rows. §11 lists 30 implementation notes covering streaming zip read, ts float-vs-int handling, file parallelism tuning, resume default, progress cadence calibration, MM API vs SQL choice, props markers rationale, reaction count vs users.length, :question: fallback debate, sample size calibration, --cleanup safety friction, edit-history honesty, verification re-runnability, UUID-v7 sort, step_metrics JSONB vs separate table, cleanup-bridge interaction.

## §2 — Findings (all resolved)

### ISS-001 — Step granularity
Without explicit steps, failure boundary unclear. Resolved: §1 #3 + DEC-470 8 steps.

### ISS-002 — Resume capability
Without checkpoint, restart from 0. Resolved: §1 #4 #5.

### ISS-003 — Idempotency
Replay = duplicates. Resolved: §1 #6 + zip_sha256 PK + per-step dedup.

### ISS-004 — Dry-run preview
Destructive without preview = risk. Resolved: §1 #9.

### ISS-005 — Progress visibility
100K-msg import blind = anxiety. Resolved: §1 #8.

### ISS-006 — Permanent vs transient error
Without distinction, every error retries. Resolved: §1 #11.

### ISS-007 — Slack timestamps could collide on MM (strict-redo pass)
Original spec said "insert MM posts in chronological order" but didn't specify timestamp conversion. Slack uses float seconds; MM uses integer milliseconds. Rounding (vs floor) could create duplicate (channel, create_at) collisions causing INSERT failures. Resolved: §1 #14 + `slack_ts_to_ms` floor implementation; AC #16 + property test verify.

### ISS-008 — Thread structure could be lost (strict-redo pass)
Original spec listed step 6 "threads" but didn't address orphan replies (parent missing from export). Silently dropping replies would lose data; silently importing as top-level would lie about structure. Resolved: §1 #15 + `slack_thread_orphan` props marker; AC #17 #18 + test bodies verify.

### ISS-009 — File dedup not specified (strict-redo pass)
Original spec said "download Slack file attachments → upload to MM file store" without specifying dedup. A file referenced from 5 messages would have been uploaded 5 times. Resolved: §1 #16 + `cyberos_imported_files` table keyed by (workspace_id, slack_file_id); AC #19 + test body verify; §11 notes the per-workspace scoping rationale.

### ISS-010 — File metadata PII leak (strict-redo pass)
Slack file names commonly carry PII (Resume - <name>.pdf). Original spec didn't address. Resolved: §1 #17 + redaction at audit-row emit (file blob preserved); AC #20 + test body verify.

### ISS-011 — Channel-type mapping ambiguity (strict-redo pass)
Slack has 4 channel types (general/private/im/mpim); MM has 4 (O/P/D/G). Original spec didn't specify the mapping. Resolved: §1 #18 + `map_slack_to_mm_channel` + AC #21-23 with rstest parameterised test.

### ISS-012 — Reactions/pins/edits unspecified (strict-redo pass)
Original spec covered messages but not reactions, pinned messages, or edited messages. Resolved: §1 #19-21 + map functions for each; AC #24-27 + test bodies; §11 documents the :question: emoji-fallback rationale.

### ISS-013 — No abort/cleanup operations (strict-redo pass)
Operators needed a way to cancel a running import OR roll back an import that went to the wrong tenant. Without these, recovery required manual DELETE statements across 5 tables. Resolved: §1 #26-27 + `--abort` + `--cleanup` CLI subcommands + cleanup() transactional implementation; AC #34-36 + test bodies; §11 explains `--yes-i-know` friction rationale.

### ISS-014 — Verification was row-count only (strict-redo pass)
Step 8 originally counted rows — caught whole-message loss but missed silent corruption (encoding bugs, partial writes). Resolved: §1 #28 + sample-and-verify-by-SHA approach (100 random posts compared to source); AC #37 #38 + `chat.import_verification_failed` SEV-1 audit + status=verification_failed terminal state; §11 explains random vs first-N sampling choice.

## §3 — Resolution

All 14 mechanical concerns addressed. **Score = 10/10.**

Per task-audit skill §0 master rule: spec is now perfect — depth bounded by the genuine surface (Slack export schema has many edge cases, MM API rate limits, 8-step checkpoint state machine, --abort + --cleanup operator surface, sample-verification design), not by line targets.

---

*End of TASK-CHAT-006 audit.*
