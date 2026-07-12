---
fr_id: FR-AUTH-111
audited: 2026-07-11
verdict: PASS (after revision)
score_pre_revision: 6/10
score_post_expansion: 8/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

## §1 - Verdict summary

FR-AUTH-111 removes a defect found live: `oidc.rs` binds the subject's **email** into the `display_name`
column at JIT provisioning, so every Google-SSO user wears their email address as their name. 9 normative
clauses, one migration (a visibility view, deliberately no backfill), 13 acceptance criteria, 7 tests, a
10-row failure-mode inventory. Six findings raised; all resolved. This is a small surface with two sharp
edges: the no-clobber rule (§1 #4) and the refusal to guess a name (§1 #7).

## §2 - Findings (all resolved)

### ISS-001 - The naive fix would silently revert an administrator's repair
The obvious implementation refreshes `display_name` from the ID token on every login. It works today, and it
breaks the moment anything else can set a name - which is *already* the case: a manual `UPDATE` was run
against production during the Play submission work to fix exactly one of these rows. Under the naive fix, that
repair would silently revert on the next sign-in, with no error and no trace. Resolved: §1 #4 refreshes only a
`display_name` that is `NULL`, empty, or byte-equal to the email - the sentinel value the bug itself writes,
and one no human would choose. §3 implements it as a `CASE` inside the `ON CONFLICT DO UPDATE`. AC #8 seeds a
deliberately-set name (`"Play Review"`) and asserts it survives a login whose token would resolve to something
else. §10 rows 2 and 9.

### ISS-002 - The migration wanted to backfill, and would have guessed wrong
The first draft derived a name from the email local-part for existing rows: `van-anh.vu@` becomes
`Van Anh Vu`. It looks like a repair and it is a guess, and it guesses wrong on precisely the names that
matter most to a Vietnamese company - the person is `Vũ Vân Anh`. Worse, it would overwrite the sentinel, so
the self-healing path in §1 #4 would never fire and the wrong name would become permanent. Resolved: §1 #7
forbids reconstructing names; §2 explains why; the migration ships a **view**
(`subjects_display_name_unset`) instead, so the damage is countable and can be watched draining to zero as
people sign in. AC #13. §10 row 4.

### ISS-003 - SAML has the same bug and was going to be left open
`saml.rs` binds `display_name` the same way as `oidc.rs`. Fixing one door and leaving the other means the bug
is rediscovered by whoever next onboards a SAML tenant, having read a changelog entry that says it was fixed.
Resolved: §1 #6 makes both call sites normative in this FR; AC #12 runs the same assertions against the SAML
path.

### ISS-004 - The debug line would have logged the person's name
Adding `tracing::debug!(?display_name)` is the reflex when debugging a name-resolution bug, and it puts
personal data into the log stream - contradicting the privacy policy this FR exists to align the code with.
Removing the log line entirely is the other reflex, and leaves the resolver unobservable. Resolved: §1 #8 logs
the **rung of the chain that matched**, never the value; §3's `resolve_display_name` returns
`(&'static str, String)` so the caller gets both properties for free; AC #10 captures the log stream and
asserts the name is absent while the rung is present. §11.

### ISS-005 - `picture` was quietly in scope
The `picture` claim sits in the same ID token, avatars would make the product nicer, and adding the column is
one line. It would also make the Data Safety declaration we are about to file with Google **false**, since
that form and the published privacy policy both enumerate what we collect. Resolved: §1 #9 forbids persisting
any claim beyond those in §1 #1, names `picture` explicitly, and states that widening the set is a
policy revision first and a code change second. `IdTokenProfile` has no `picture` field to deserialise into.
AC #11 greps the raw row. §9 keeps it as a deferred FR with its paperwork attached.

### ISS-006 - Whitespace-only and empty claims would have written a blank name
`name: ""` and `name: "   "` are both things IdPs emit. The draft's `Option` check treated them as present, so
the person would have rendered as nothing at all - a worse outcome than the email. Resolved: every rung of
§3's chain applies `trim()` and `filter(|s| !s.is_empty())` before accepting, so blanks fall through; AC #9
asserts a whitespace `name` falls through to the email local-part rather than writing an empty
`display_name`. §10 row 3.

## §3 - Resolution

Six findings, all resolved. The two that would have caused lasting damage - ISS-001 (reverting a human's
repair) and ISS-002 (permanently inventing a wrong Vietnamese name) - are each pinned by an acceptance
criterion that fails loudly, not by a comment. ISS-005 is the one worth re-reading before anyone "just adds
avatars": the code change is trivial and the paperwork is not.

**Score = 10/10.**

---

*End of FR-AUTH-111 audit.*

## Ship record (2026-07-12 - status-drift reconciliation)

- Implemented at bc7af7b (parallel session); 9/9 clause verification PASS
  (packet: docs/feature-requests/.workflow/FR-AUTH-111/review-packet.md).
- Test evidence: 6 unit tests incl. anti-prettify canary + picture-cannot-land proof; operator
  confirmed tests green (CI/cargo) - sandbox has no Rust toolchain, gap named.
- HITL: operator verdict 2026-07-12 in-chat "Tests green - approve + done" (both gates).
