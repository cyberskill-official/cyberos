---
fr_id: FR-CHAT-269
audited: 2026-07-11
verdict: PASS (after revision)
score_pre_revision: 5/10
score_post_expansion: 8/10
score_post_revision: 10/10
issues_resolved: 8
template: engineering-spec@1
---

## §1 - Verdict summary

FR-CHAT-269 specifies the workspace moderation queue: 21 normative §1 clauses, one migration (retention),
three administrator routes, a fail-closed role gate that is chat's first consumer of the FR-AUTH-101 `roles`
claim, 20 acceptance criteria, 10 Rust integration tests plus 2 client tests, and a 15-row failure-mode
inventory. Eight findings were raised across three rounds; all resolved. The finding that changes the shape
of the feature is ISS-001: the natural implementation of "show the reviewer some context" would have turned a
safety feature into an employer surveillance tool.

## §2 - Findings (all resolved)

### ISS-001 - "Surrounding context" would have handed private DMs to the employer
The first draft returned the messages before and after the reported one, unconditionally, so the reviewer
could "see context". Applied to a DM report, that means: Alice reports one harassing message from Bob, and her
entire private correspondence with Bob becomes readable by her workspace administrator - who is, in a small
company, her boss. It is the easiest thing to build, the most natural thing to want, and a betrayal of the
person who came to us for help. It is also self-defeating: it is precisely why people do not report. Resolved:
§1 #9 forbids disclosing *any* part of a DM thread beyond the single reported message, with no override flag;
§1 #8 restricts group-channel context to channels the administrator is *already a member of*, so a report
cannot become a skeleton key into a private channel; §2 argues both. AC #7 asserts on the raw JSON body - not
just the `context` field - that no other DM message appears anywhere in the response. §10 rows 3 and 4.

### ISS-002 - The role gate treated a missing claim as permissive
The draft checked `claims.roles.contains("tenant-admin")` against a `#[serde(default)]` vector, and FR-AUTH-101
allows a grace window in which tokens may lack the RBAC claims entirely. So every token issued before
FR-AUTH-101 shipped - all of them still valid - would have deserialised to `roles: []`... and an early
draft's `if roles.is_empty() { /* legacy: allow */ }` branch would have made every one of them a moderator.
Resolved: §1 #2 mandates fail-closed with no legacy path (chat has never read `roles`, so there is nothing to
be gentle with); §3 shows `require_moderator` as a pure whitelist match with no else-allow branch; AC #1 tests
an empty-roles token *and* a token with no claim at all. §10 row 1.

### ISS-003 - A channel owner would have been able to resolve reports about themselves
Chat already has channel roles (`owner > manager > member`) and the draft reached for them, since they were
to hand. But a channel owner is not a workspace moderator, and the report most likely to be filed in a channel
someone owns is a report *about* them. Resolved: §1 #3 forbids channel roles from granting queue access; AC #2
asserts a channel owner with no workspace role receives `403`. §10 row 2.

### ISS-004 - Concurrent resolves would have written a false audit chain
Two admins opening the queue simultaneously is the normal case in a small workspace. Without a compare-and-swap
the second `delete_message` is a harmless no-op *at the data layer* - but it emits a second `chat.report_resolved`
row, so the hash-chained audit log ends up asserting that two people independently decided to delete the same
message. The audit chain is the artefact we ask customers to trust; writing a falsehood into it is worse than
the race. Resolved: §1 #12 makes `resolve` a CAS on `status = 'open'`, the loser receives `200` carrying the
winning resolution, re-applies nothing, and emits nothing. AC #10 runs the two calls under `tokio::join!` and
asserts exactly one of each audit row.

### ISS-005 - Grouping after pagination shows one message three times on page one
The draft folded duplicate reports in Rust, after the SQL `LIMIT`. With four rows in a test fixture this is
indistinguishable from correct. In production, twelve reports against one message fill page one with twelve
copies of it and the reviewer never reaches anything else - which is the actual failure mode of moderation
queues: not bad decisions, but a reviewer who stops opening the queue. Resolved: §3 groups by the target triple
in SQL, *before* the `LIMIT`; §11 names the trap; AC #3 asserts three reports fold to one entry with
`report_count: 3`.

### ISS-006 - Sibling reports were left open against a deleted message
Resolving one of five reports against a message deleted it and closed one row. The other four stayed `open`,
pointing at a message that no longer exists, and the queue accumulated ghosts. Resolved: §1 #13 closes every
sibling open report against the same target in the same transaction; the audit row lists their ids so the
decision remains traceable to all five. AC #11.

### ISS-007 - The sibling `UPDATE` would have matched nothing
Having written the sibling update, it silently matched zero rows: the target columns are nullable and two of the
three are `NULL` on any given row, so `target_message_id = $6` yields `NULL` against `NULL`, not `TRUE`. This is
the same three-valued-logic trap that killed the partial unique index in FR-CHAT-267 (its ISS-007), wearing a
different hat. Resolved: §3 uses `IS NOT DISTINCT FROM` for all three target columns; §11 records why, so the
next reader does not "simplify" it back to `=`.

### ISS-008 - The snapshot would have been retained forever
FR-CHAT-267's snapshot is justified while the report is open, because the sender can destroy the original. Once
the report is resolved that justification expires - and what is left is a durable copy of precisely the content
someone asked us to remove, sitting in a table their employer can read, indefinitely. Resolved: §1 #17 purges
resolved reports *including snapshots* 90 days after resolution; the window is not arbitrary, it matches the
number already published at `cyberskill.world/en/cyberos/delete-account`, so there is one number and one
behaviour. §3 adds `purge_after`; §6 implements the purge as an hourly job rather than a trigger (a trigger
would delete rows out from under an admin mid-read); AC #15.

## §3 - Resolution

Eight findings, all resolved. The three with teeth are pinned by assertions rather than prose: AC #7 reads the
raw response body for leaked DM content, AC #1 tests the claim-absent token that would have made everyone a
moderator, and AC #10 forces the concurrent path with `tokio::join!`. The DM carve-out (§1 #9) is the clause a
future contributor is most likely to "fix" for the sake of reviewer convenience; §2 exists to explain to them
why not.

**Score = 10/10.**

---

*End of FR-CHAT-269 audit.*
