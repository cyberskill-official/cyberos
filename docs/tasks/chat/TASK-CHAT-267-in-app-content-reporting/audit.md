---
task_id: TASK-CHAT-267
audited: 2026-07-11
verdict: PASS (after revision)
score_pre_revision: 6/10
score_post_expansion: 8/10
score_post_revision: 10/10
issues_resolved: 8
template: engineering-spec@1
---

## §1 - Verdict summary

TASK-CHAT-267 specifies in-app content reporting for CyberOS chat: 14 normative §1 clauses, one migration introducing `chat_reports` with four CHECK constraints and a partial unique index, one endpoint, 16 acceptance criteria, 9 Rust integration tests plus 2 client tests, and a 15-row failure-mode inventory. Eight findings were raised across three audit rounds; all eight are resolved in the spec as it stands. The load-bearing clause is §1 #4 (evidence snapshot) - without it the feature is decorative, because the sender can destroy the evidence after the report lands.

## §2 - Findings (all resolved)

### ISS-001 - The moderation queue would render "(deleted)" for every report that mattered
First draft simply stored `target_message_id` and let the reviewer read the message back at review time. But `chat_messages` exposes both edit (`edited_at`) and soft delete (`deleted_at`) **to the sender**. The obvious abuse is therefore: post abuse, wait for the report, delete. The report survives; the evidence does not. Resolved: §1 #4 mandates an immutable snapshot captured at report time (`snapshot_body`, `snapshot_sender_id`, and the attachment triple), written once at INSERT and never updated; §3 migration carries the columns; AC #2 and AC #3 assert the snapshot survives a subsequent delete and a subsequent edit respectively.

### ISS-002 - The snapshot could race the edit it exists to defeat
Adding a snapshot is not enough if it is read in one statement and inserted in another: a sender editing concurrently can land between the two, and the snapshot captures the *sanitised* text. Resolved: §6 `snapshot_target` takes the message row `FOR SHARE` inside the same transaction that performs the INSERT, holding it for the life of the transaction. §11 explains why `FOR SHARE` and not `FOR UPDATE` (we do not write `chat_messages`, and `FOR UPDATE` would needlessly serialise unrelated readers). §10 row 1.

### ISS-003 - A 409 on a duplicate report is an oracle
First draft returned `409 Conflict` when the partial unique index fired. That is a distinguishable response, which means *anyone* can probe whether a given message already carries an open report simply by reporting it and reading the status code. It also punishes the ordinary user who taps twice because they were not sure it registered. Resolved: §1 #6 mandates `200 OK` with the existing report's id, identical body shape to the `201` path; §2 gives both the usability and the oracle rationale; AC #7 asserts 200-not-409 and that only one row exists.

### ISS-004 - The rate limit was checked after target resolution, leaking id existence
The first ordering resolved the target (403 for non-member, 404 for unknown id) and *then* counted recent reports. A caller already over the limit could therefore still use the endpoint as an existence oracle for message ids, because they would receive a 404 rather than a 429. Resolved: §3 handler runs the rate-limit count as the first statement inside the transaction, before `snapshot_target`; §1 #7; AC #9 asserts that the 21st report against a *non-existent* message id returns `429`, not `404` - the test fails if the ordering regresses. §10 row 6.

### ISS-005 - Reported content was being copied into the hash-chained audit log
First draft's `chat.report_created` payload included the message body, on the reasoning that "the audit row should be self-contained". That is precisely backwards. The audit chain is hash-chained and replicated into the memory module, where it is designed to be durable and resistant to rewriting - the correct property for "who did what when", and the wrong property for a copy of content that someone has just asked us to consider removing. It also doubles the blast radius of that content. Resolved: §1 #8 forbids the snapshot and the free-text `detail` in the audit payload; §2 explains why; AC #10 asserts the payload carries neither, and asserts the *serialised row* does not contain the body text anywhere.

### ISS-006 - Requiring co-membership would have made the DM case unreportable
Draft §1 #3 required the reporter to share a channel with the target for every target kind. But `chat_channels.kind = 'direct'` lets any workspace member open a DM with any other, so the single most likely harassment vector - an unsolicited DM from someone you do not work with - would have been the one thing you could not report. Resolved: §1 #3 splits the rule: message and attachment targets require channel membership (you cannot report what you cannot see); *subject* targets require nothing. AC #5 asserts a subject report succeeds with no shared channel. §2 carries the rationale.

### ISS-007 - The partial unique index would never have fired
The index was first written over the three nullable target columns directly. Postgres treats `NULL` as distinct from `NULL` inside a unique index, so every row - each with two of the three columns null - would have compared as unique, and the dedup guarantee in §1 #6 would have been silently absent. This is the category of bug that passes every happy-path test. Resolved: §3 coalesces the unused target columns to the nil UUID (`00000000-...`), which is never a real subject id, matching the module-wide nil-UUID convention; §11 documents why. AC #7's "creates no second row" assertion is what catches a regression.

### ISS-008 - `ON CONFLICT DO UPDATE` would have re-opened the race that ISS-001 closed
An earlier revision used `ON CONFLICT ... DO UPDATE ... RETURNING id` to get the existing id back in one round trip. That is elegant and wrong: the update would overwrite the *original* snapshot with a fresh one taken at the time of the duplicate submission - by which point the sender may already have edited the message. The dedup path would have quietly destroyed the evidence the dedup path was protecting. Resolved: §3 uses `ON CONFLICT DO NOTHING` plus a follow-up SELECT for the existing id; §11 states the reason explicitly so a future reader does not "optimise" it back.

## §3 - Resolution

Eight findings, all resolved in the spec. The three that would have shipped a feature that *looked* correct and was not - ISS-001 (no snapshot), ISS-004 (leaky rate-limit ordering), ISS-007 (dead unique index) - are each pinned by an acceptance criterion that fails loudly on regression, not by a comment.

**Score = 10/10.**

---

*End of TASK-CHAT-267 audit.*
