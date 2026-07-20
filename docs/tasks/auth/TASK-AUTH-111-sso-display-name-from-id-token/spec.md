---
id: TASK-AUTH-111
title: "SSO JIT provisioning must take the person's name from the ID token, not their email address"
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
eu_ai_act_risk_class: not_ai
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-07-11T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: AUTH
priority: p1
status: done
verify: T
phase: P0
milestone: P0 - store compliance (UGC controls)
slice: 1
owner: Stephen Cheng (CTO)
created: 2026-07-11
shipped: 2026-07-12
memory_chain_hash: null
related_tasks: [TASK-AUTH-110, TASK-CHAT-269]
depends_on: []
blocks: []
source_pages:
  - services/auth/src/oidc.rs
  - services/auth/src/saml.rs
  - https://cyberskill.world/en/cyberos/privacy
source_decisions:
  - "The published CyberOS privacy policy states we receive the person's name from Google at sign-in. Today we receive it and throw it away. The code and the policy must agree."
language: Rust (axum + sqlx)
service: cyberos/services/auth
new_files:
  - services/auth/migrations/0026_sso_display_name_backfill.sql
modified_files:
  - services/auth/src/oidc.rs
  - services/auth/src/saml.rs
  - services/auth/tests/oidc_jit.rs
allowed_tools:
  - sqlx migrations against the auth schema
disallowed_tools:
  - any change to the subjects table's constraints
  - any invention of a name for an existing subject
effort_hours: 6
subtasks:
  - "read name / given_name / family_name / preferred_username from the ID token (1h)"
  - "display_name resolution chain + bind it at JIT INSERT (1h)"
  - "ON CONFLICT: refresh a display_name that was never really set, never clobber one that was (2h)"
  - "mirror the fix in saml.rs (1h)"
  - "tests: each rung of the fallback chain, and the no-clobber guarantee (1h)"
risk_if_skipped: "Every person provisioned through Google SSO wears their email address as their display name, in every channel, on every message, forever. It looks broken, it is a small privacy leak (the address is rendered wherever the name should be, including in any screenshot or export), and it contradicts the privacy policy we have already published, which tells people we take their name from their Google account. Fixing it after a few hundred users have signed in is the same work plus a data-repair exercise."
---

## §1 - Description (BCP-14 normative)

1. The OIDC callback **MUST** parse the person's name from the verified ID token, using the standard OpenID Connect claims: `name`, and failing that `given_name` + `family_name`, and failing that `preferred_username`.

2. At JIT provisioning the service **MUST** bind `subjects.display_name` from that resolution chain, falling back in order to: the ID token's `name`; `given_name family_name` joined; `preferred_username`; the local-part of the email; and only then the full email address. It **MUST NOT** bind `display_name` directly from the email, which is what it does today (`oidc.rs` binds `idp_email.unwrap_or("")` into the `display_name` column - the bug this task exists to remove).

3. The service **MUST NOT** invent, prettify, title-case, or otherwise transform a name it was given. If the IdP says the person is called `nguyenvana`, that is their name. Guessing at capitalisation or splitting on separators is how a product mangles Vietnamese names.

4. On the `ON CONFLICT (tenant_id, handle) DO UPDATE` path - a returning SSO user - the service **MUST** refresh `display_name` **only** when the stored value was never really a name: that is, when it is `NULL`, empty, or byte-equal to the subject's email. It **MUST NOT** overwrite a `display_name` that differs from the email, because that value was set deliberately - by an administrator, by a future profile editor, or by a data repair - and silently reverting it on next login is worse than the original bug.

5. Consequently the fix **MUST** be self-healing: every existing subject whose `display_name` is currently their email address acquires their real name the next time they sign in, with no migration and no administrator action.

6. The same defect exists in `saml.rs`, which binds `display_name` the same way. It **MUST** be fixed in the same change, with the same resolution chain and the same no-clobber rule. Fixing one and not the other leaves a second door open and guarantees the bug is rediscovered.

7. The migration **MUST NOT** attempt to reconstruct names for existing rows. There is no source of truth for them in our database - the ID token was discarded - and inventing a name from an email local-part would produce `van-anh.vu` where the person is called `Vũ Vân Anh`. §1 #5's self-healing path is the correct repair.

8. The resolution **MUST** be logged at `debug` with the *rung of the chain that matched*, never with the name itself. `tracing::debug!(rung = "given_name+family_name")` is a diagnostic; `tracing::debug!(?name)` puts personal data in the log stream, which the privacy policy says we do not do.

9. No claim beyond those in §1 #1 **MAY** be persisted. In particular the `picture` claim **MUST NOT** be stored: the published privacy policy enumerates what we collect, and adding a field to `subjects` is a change to that enumeration, not an implementation detail. If we want avatars, that is a separate task *and* a privacy-policy revision, in that order.

## §2 - Why this design (rationale for humans)

**Why this is a defect and not a cosmetic issue.** `oidc.rs` line ~965 binds the email into the `display_name` column. Every human provisioned by Google SSO therefore renders as `van-anh.vu@cyberskill.world` wherever a name belongs: in the channel list, above every message they have ever sent, in mentions, in the member picker, and in any screenshot or export. It reads as broken software. It also means the address is displayed in contexts where only a name was intended - a small but real leak, and one that has already forced a manual `UPDATE` against production to make a store screenshot presentable.

**Why the code contradicts the published policy.** `cyberskill.world/en/cyberos/privacy` says, in terms: "When you sign in with Google we receive your name, your email address, and your Google account identifier." We do receive the name. We then drop it on the floor and store the email in its place. The policy is not wrong about what Google sends us; the code is wrong about what we do with it. Aligning them is the whole of this task.

**Why the no-clobber rule (§1 #4) is a MUST and not a nicety.** The naive fix - always refresh `display_name` from the ID token on login - would work today and break the moment anything else can set a name. An administrator who fixes a colleague's name by hand, or a future profile editor, would watch their change silently revert on that person's next sign-in, with no error and no trace. Refreshing *only* the sentinel value (null, empty, or exactly the email) is the narrow rule that repairs the damage without acquiring the authority to undo a human's decision.

**Why not backfill (§1 #7).** Because the information is gone. We can see that `display_name = email`, so we can see *which* rows are wrong, but we cannot see what they should say - the ID token that carried the name was never persisted. The only honest sources are the IdP (which means a login) or a human (which means typing). Deriving `Van Anh Vu` from `van-anh.vu@` looks like a fix and is a guess, and it guesses wrong on exactly the names that matter most to a Vietnamese company. Self-healing on next login is slower and correct.

**Why `picture` is explicitly excluded (§1 #9).** It is right there in the ID token, it would make the product nicer, and adding it would quietly make the published Data Safety declaration false. The store form and the policy both enumerate what we collect. Widening that set is a decision with paperwork attached, not a line of code.

## §3 - API contract

### The claims we read

```rust
// services/auth/src/oidc.rs

/// Standard OIDC claims. We read only what §1 #1 names. `picture` is deliberately absent:
/// adding it here would silently widen the Data Safety declaration (§1 #9).
#[derive(Debug, Deserialize)]
struct IdTokenProfile {
    #[serde(default)] name: Option<String>,
    #[serde(default)] given_name: Option<String>,
    #[serde(default)] family_name: Option<String>,
    #[serde(default)] preferred_username: Option<String>,
}
```

### The resolution chain

```rust
/// §1 #2. Ordered, total, and deliberately dumb: no transformation of any value it is handed (§1 #3).
/// Returns the rung that matched alongside the value, so the caller can log the rung without logging
/// the name (§1 #8).
fn resolve_display_name<'a>(
    p: &'a IdTokenProfile,
    email: Option<&'a str>,
) -> (&'static str, String) {
    if let Some(n) = p.name.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        return ("name", n.to_string());
    }
    let given  = p.given_name.as_deref().map(str::trim).filter(|s| !s.is_empty());
    let family = p.family_name.as_deref().map(str::trim).filter(|s| !s.is_empty());
    match (given, family) {
        (Some(g), Some(f)) => return ("given_name+family_name", format!("{g} {f}")),
        (Some(g), None)    => return ("given_name", g.to_string()),
        (None, Some(f))    => return ("family_name", f.to_string()),
        (None, None)       => {}
    }
    if let Some(u) = p.preferred_username.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        return ("preferred_username", u.to_string());
    }
    match email {
        Some(e) if e.contains('@') => ("email_local_part", e.split('@').next().unwrap().to_string()),
        Some(e)                    => ("email", e.to_string()),
        None                       => ("handle", String::new()),   // caller substitutes the handle
    }
}
```

### The JIT insert

```rust
let (rung, display_name) = resolve_display_name(&profile, idp_email);
tracing::debug!(target: "cyberos_auth::oidc", rung, "resolved display_name");   // §1 #8 - rung, not name

let row: (Uuid,) = sqlx::query_as(
    "INSERT INTO subjects (tenant_id, handle, display_name, email, kind, status, password_hash, roles)
          VALUES ($1, $2, $3, $4, 'human', 'active', $5, $6)
     ON CONFLICT (tenant_id, handle) DO UPDATE
        SET email = COALESCE(EXCLUDED.email, subjects.email),
            -- §1 #4: refresh ONLY a display_name that was never really set. A value that differs
            -- from the email was put there on purpose; do not silently revert someone's decision.
            display_name = CASE
                WHEN subjects.display_name IS NULL
                  OR subjects.display_name = ''
                  OR subjects.display_name = subjects.email
                THEN EXCLUDED.display_name
                ELSE subjects.display_name
            END,
            updated_at = NOW()
   RETURNING id",
)
.bind(idp.tenant_id)
.bind(&handle)
.bind(&display_name)          // <- was: idp_email.unwrap_or("")   THE BUG
.bind(idp_email)
.bind(&sso_password_hash)
.bind(&idp.default_roles)
.fetch_one(&mut *tx)
.await
.map_err(internal)?;
```

### Migration

```sql
-- services/auth/migrations/0026_sso_display_name_backfill.sql
-- TASK-AUTH-111. There is deliberately NO backfill of display_name here (§1 #7): the ID token that
-- carried the person's name was never persisted, so we can see WHICH rows are wrong but not what
-- they should say. Deriving "Van Anh Vu" from "van-anh.vu@" is a guess, and it guesses wrong on
-- exactly the names that matter most to a Vietnamese company. Every affected subject self-heals on
-- their next sign-in via the ON CONFLICT rule.
--
-- What this migration DOES is make the damage visible and measurable, so we can watch it drain.

CREATE OR REPLACE VIEW subjects_display_name_unset AS
    SELECT id, tenant_id, handle, email, created_at
      FROM subjects
     WHERE kind = 'human'
       AND email IS NOT NULL
       AND (display_name IS NULL OR display_name = '' OR display_name = email);

COMMENT ON VIEW subjects_display_name_unset IS
    'TASK-AUTH-111: humans still wearing their email as a display name. Drains to zero as people sign in.';

GRANT SELECT ON subjects_display_name_unset TO cyberos_ro;
```

## §4 - Acceptance criteria

1. **`name` wins** - an ID token carrying `name: "Trịnh Thái Anh"` provisions a subject whose `display_name` is exactly `Trịnh Thái Anh`, diacritics intact, untransformed.
2. **`given_name` + `family_name` is the second rung** - a token with no `name` but both parts provisions `"<given> <family>"`.
3. **A single part is used alone** - a token with only `given_name` provisions that value, not an empty-padded join.
4. **`preferred_username` is the third rung** - used when no name claims are present.
5. **The email local-part is the fourth rung** - a token with no name claims at all and `email: a.b@c.com` provisions `a.b`, never `a.b@c.com`.
6. **`display_name` is never the full email** - across every rung, no provisioned subject has `display_name = email`.
7. **A returning user with an email as their name is repaired** - a subject stored with `display_name = email` acquires their real name on the next sign-in, with no migration.
8. **A deliberately set name is never clobbered** - a subject whose `display_name` was set to `"Play Review"` by hand keeps it across sign-ins, even though the ID token would resolve to something else.
9. **A blank ID-token name does not blank the stored name** - a token whose `name` is `""` or whitespace falls through the chain and never writes an empty `display_name`.
10. **The name is never logged** - no log line at any level contains the resolved name; the `debug` line contains only the rung that matched.
11. **`picture` is not persisted** - no column receives it, and the `subjects` row after a Google sign-in has no avatar data.
12. **SAML is fixed identically** - the same eight assertions hold against `saml.rs`'s provisioning path.
13. **The visibility view drains** - `subjects_display_name_unset` returns a row for an affected subject before their next sign-in and no row after it.

## §5 - Verification

```rust
// services/auth/tests/oidc_jit.rs

#[tokio::test]
async fn the_resolution_chain_walks_in_order() {                     // AC 1-5, 9
    assert_eq!(resolve(tok().name("Trịnh Thái Anh")).1, "Trịnh Thái Anh");
    assert_eq!(resolve(tok().given("Thái Anh").family("Trịnh")).1, "Thái Anh Trịnh");
    assert_eq!(resolve(tok().given("Thái Anh")).1, "Thái Anh");
    assert_eq!(resolve(tok().preferred("stephen")).1, "stephen");
    assert_eq!(resolve(tok().email("van-anh.vu@cyberskill.world")).1, "van-anh.vu");
    // whitespace-only claims fall through rather than writing a blank
    assert_eq!(resolve(tok().name("   ").email("a.b@c.com")).1, "a.b");
}

#[tokio::test]
async fn display_name_is_never_the_full_email() {                    // AC 6 - the bug, pinned
    let app = harness().await;
    for tok in [tok().name("A B"), tok().preferred("ab"), tok().email("a.b@c.com")] {
        let s = app.sso_login(tok.email("a.b@c.com")).await;
        assert_ne!(s.display_name, s.email.unwrap(),
                   "display_name must never be bound from the email column");
    }
}

#[tokio::test]
async fn a_returning_user_self_heals_but_a_set_name_survives() {     // AC 7, 8 - the no-clobber rule
    let app = harness().await;

    // Damaged row, exactly as production has them today.
    let s = app.seed_subject(display_name = "van-anh.vu@cyberskill.world",
                             email        = "van-anh.vu@cyberskill.world").await;
    app.sso_login(tok().sub(s.idp_sub).name("Vũ Vân Anh")).await;
    assert_eq!(app.subject(s.id).await.display_name, "Vũ Vân Anh");   // repaired

    // Deliberately set row: an admin fixed this by hand. It must survive.
    let p = app.seed_subject(display_name = "Play Review",
                             email        = "play-review@cyberskill.world").await;
    app.sso_login(tok().sub(p.idp_sub).name("play-review")).await;
    assert_eq!(app.subject(p.id).await.display_name, "Play Review");  // NOT clobbered
}

#[tokio::test]
async fn the_name_never_reaches_the_log_stream() {                   // AC 10
    let logs = capture_logs(Level::DEBUG, || async {
        harness().await.sso_login(tok().name("Trịnh Thái Anh")).await;
    }).await;
    assert!(logs.contains(r#"rung="name""#));
    assert!(!logs.contains("Trịnh Thái Anh"), "personal data must not reach the log stream");
}

#[tokio::test]
async fn picture_is_not_persisted() {                                // AC 11
    let app = harness().await;
    let s = app.sso_login(tok().name("A B").picture("https://…/photo.jpg")).await;
    let raw = app.raw_subject_row(s.id).await;
    assert!(!raw.to_string().contains("photo.jpg"));
    assert!(!raw.to_string().contains("picture"));
}

#[tokio::test]
async fn saml_is_fixed_too() {                                       // AC 12
    let app = harness().await;
    let s = app.saml_login(assertion().display_name("Trịnh Thái Anh")
                                      .email("thai-anh.trinh@cyberskill.world")).await;
    assert_eq!(s.display_name, "Trịnh Thái Anh");
}

#[tokio::test]
async fn the_visibility_view_drains() {                              // AC 13
    let app = harness().await;
    let s = app.seed_subject(display_name = "a.b@c.com", email = "a.b@c.com").await;
    assert_eq!(app.view_rows("subjects_display_name_unset").await.len(), 1);
    app.sso_login(tok().sub(s.idp_sub).name("A B")).await;
    assert_eq!(app.view_rows("subjects_display_name_unset").await.len(), 0);
}
```

## §6 - Implementation skeleton

(§3 is the skeleton. The change is three lines of binding, one `CASE` in the upsert, one helper, and the same again in `saml.rs`.)

## §7 - Dependencies

- **Upstream:** none. `oidc.rs` already verifies the ID token before this code runs; we are reading claims from an already-verified token, not adding a trust boundary.
- **Downstream:** TASK-CHAT-269's moderation queue renders reporter and reported names; it inherits the fix for free.
- **Policy:** `cyberskill.world/en/cyberos/privacy` already declares that we receive the person's name. This task makes the code match the declaration. Any *widening* of what we persist - `picture`, `locale`, `hd` - is a policy change first and a code change second (§1 #9).

## §8 - Example payloads

The Google ID token, as verified today:

```json
{
  "iss": "https://accounts.google.com",
  "sub": "104…",
  "email": "thai-anh.trinh@cyberskill.world",
  "email_verified": true,
  "name": "Trịnh Thái Anh",
  "given_name": "Thái Anh",
  "family_name": "Trịnh",
  "picture": "https://lh3.googleusercontent.com/…"
}
```

The subject row today (the bug):

```json
{ "handle": "@thai-anh.trinh",
  "display_name": "thai-anh.trinh@cyberskill.world",
  "email": "thai-anh.trinh@cyberskill.world" }
```

The subject row after this task:

```json
{ "handle": "@thai-anh.trinh",
  "display_name": "Trịnh Thái Anh",
  "email": "thai-anh.trinh@cyberskill.world" }
```

## §9 - Open questions

**Deferred:**

- *Avatars.* The `picture` claim is right there and the product would look better for it. It is deliberately out of scope (§1 #9): persisting it widens the published Data Safety declaration, so it needs a privacy-policy revision and a Play Data Safety update before a line of code. Its own task.
- *A profile editor.* Once people can set their own display name, §1 #4's no-clobber rule is what stops SSO from reverting it. The rule is written now precisely so the editor is additive later.
- *Handle derivation.* `@oidc-<sub prefix>` for a subject with no email is unpleasant but out of scope here; this task touches `display_name` only.

## §10 - Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| `display_name` bound from the email column | AC 6 asserts `display_name != email` on every rung | The defect in production today | The bind is changed; the AC fails if anyone reverts it |
| SSO overwrites a hand-set name on next login | AC 8 seeds a deliberately set name and signs in | An admin's repair silently reverts, with no error | `CASE` refreshes only null / empty / equal-to-email |
| A blank `name` claim writes an empty display name | AC 9; every rung trims and filters empty | A person renders as nothing at all | The chain falls through on whitespace |
| Backfill invents a wrong name | §1 #7; no backfill exists | `Van Anh Vu` where the person is `Vũ Vân Anh` | The migration creates a *view*, not an `UPDATE` |
| Name written to the log stream | AC 10 greps captured logs for the name | Personal data in logs, contradicting the privacy policy | The `debug` line carries the rung only |
| `picture` quietly persisted | AC 11 greps the raw row | The published Data Safety declaration becomes false | `IdTokenProfile` has no `picture` field to deserialise into |
| SAML left broken | AC 12 runs the same assertions against `saml.rs` | The bug is rediscovered through the other door | Both call sites are changed in one commit |
| Diacritics mangled by a "helpful" transformation | AC 1 asserts byte equality on `Trịnh Thái Anh` | Vietnamese names corrupted - the worst possible failure for this company | §1 #3 forbids transformation; the resolver copies |
| A future contributor "simplifies" the CASE to an unconditional SET | AC 8 fails | The no-clobber guarantee is lost | Pinned by an assertion, not a comment |
| Subject has no email and no name claims | Chain's final rung returns empty; caller substitutes the handle | The person renders as their handle, not as blank | Explicit final arm in `resolve_display_name` |

## §11 - Implementation notes

- **Why the resolver returns the rung as well as the value.** So the caller can log *how* it resolved without logging *what* it resolved. Observability into a name-resolution bug is close to worthless without knowing which rung fired, and a `?name` in a debug line is a privacy incident. Returning `(&'static str, String)` gets both properties for free.

- **Why `subjects.display_name = subjects.email` is the sentinel for "never set".** Because it is exactly the value the bug wrote, and no human would ever choose it. It is a slightly ugly heuristic and it is the right one: the alternative - a nullable `display_name_source` column - is a schema change to fix a three-line bug, and it would need its own backfill.

- **Why the migration ships a view rather than doing nothing.** The self-healing repair is invisible: there is no way to know whether it worked, or how many people are still affected, without a query. `subjects_display_name_unset` makes the damage countable, so the repair can be watched draining to zero rather than assumed.

- **`format!("{g} {f}")` and not a locale-aware ordering.** Vietnamese convention puts the family name first, and `given_name`/`family_name` from Google are already ordered per the account's own locale in the `name` claim - which is why `name` is the first rung and the join is only a fallback. Building a locale-aware name formatter to serve the fallback path would be a large amount of code guarding an edge, and it would still be wrong for someone whose IdP disagrees with their culture. The `name` claim is the person's own answer; prefer it.

*End of TASK-AUTH-111.*
