---
id: TASK-PROJ-011
title: "Blocker detector from comment stream — `blocked by` parser + dwell-time monitor + CUO Notify on stale blockers"
eu_ai_act_risk_class: not_ai
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-16T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: proj
priority: p0
status: done
verify: T
phase: P1
milestone: P1 · slice 3
slice: 3
owner: Stephen Cheng
created: 2026-05-16
shipped: null
memory_chain_hash: null
related_tasks: [TASK-PROJ-003, TASK-PROJ-004, TASK-CUO-101, TASK-OBS-007, TASK-MEMORY-101]
depends_on: [TASK-PROJ-003, TASK-CUO-101]
blocks: []

source_pages:
  - website/docs/modules/proj.html#blocker-detection
source_decisions:
  - DEC-320 (blocker = comment matching `blocked by <ref>` regex; <ref> = issue mention OR free text)
  - DEC-321 (dwell = wall-clock time since blocker comment + issue still in_progress; ≥ 3 business days → stale)
  - DEC-322 (CUO Notify is the canonical notification surface; never email directly)

language: rust 1.81
service: cyberos/services/proj-sync/
new_files:
  - services/proj-sync/src/blocker/mod.rs
  - services/proj-sync/src/blocker/parser.rs
  - services/proj-sync/src/blocker/dwell.rs
  - services/proj/tests/link_types_test.rs
modified_files:
  # spawn hourly dwell scan
  - services/proj-sync/src/main.rs
  - services/proj-sync/migrations/0011_blocker_state.sql
allowed_tools:
  - file_read: services/proj-sync/**
  - file_write: services/proj-sync/{src,tests,migrations}/**
  - bash: cd services/proj-sync && cargo test blocker
disallowed_tools:
  - send email directly from this task (per DEC-322 — only CUO Notify)
  - infer blocker semantics from non-comment data (per DEC-320 — comment stream only)

effort_hours: 6
subtasks:
  - "0.5h: 0011_blocker_state.sql migration"
  - "0.5h: blocker/mod.rs — Blocker struct, BlockerKind enum"
  - "1.5h: parser.rs — regex `(?i)\\bblock(?:ed|s|ing)\\s+by[:\\s]+([^\\n,;.]+)` + #issue-ref extractor"
  - "1.0h: dwell.rs — hourly scan; business-day computation (skip Sat/Sun + VN holidays)"
  - "0.5h: CUO Notify integration (placeholder webhook to TASK-CUO-101)"
  - "0.5h: memory audit row 'proj.blocker_detected', 'proj.blocker_resolved', 'proj.blocker_stale'"
  - "1.5h: blocker_test.rs — regex coverage + dwell + resolution"
risk_if_skipped: "Blocker tracking is the highest-leverage signal in PROJ: a stale blocker means someone is stuck. Without detection, issues sit indefinitely. Without dwell-time semantics, even fresh blockers get pinged repeatedly. Without CUO routing, notifications flood Slack channels. Manual blocker tracking via labels is friction users skip."
---

## §1 — Description (BCP-14 normative)

The blocker detector **MUST** parse Issue comments for "blocked by" patterns and monitor dwell time. The contract:

1. **MUST** parse comments on insert/edit (TASK-PROJ-003 CRDT events) for the regex `(?i)\b(block(?:ed|s|ing))\s+by[:\s]+([^\n,;.]+)`. Captures: keyword (blocked|blocks|blocking) + target reference.
2. **MUST** classify target as:
- `IssueMention`: matches `#<uuid>` or `#<readable-id>`; resolved via TASK-PROJ-001 issue table.
- `MemoryPath`: matches `memories/...`; resolved via TASK-MEMORY-101.
- `FreeText`: anything else; stored verbatim ("waiting on customer", "needs design review").
3. **MUST** record blockers in `blocker_state` table: `(issue_id, comment_id, blocker_kind, target_ref, detected_at, detected_by, resolved_at, resolved_by_comment_id, tenant_id)`. Active = `resolved_at IS NULL`.
4. **MUST** auto-resolve a blocker when:
- Target Issue transitions to `Done` (forward-direction unblock).
- A subsequent comment on the blocking issue matches `(?i)\bunblocked\b` or `(?i)\bresolved\b`.
- The issue itself transitions to Done | Cancelled (block becomes moot).
5. **MUST** compute `dwell_business_days` since `detected_at`, excluding Sat/Sun and configured VN public holidays (`CYBEROS_HOLIDAYS_VN` env: comma-separated YYYY-MM-DD). ≥ 3 business days → mark stale.
6. **MUST** emit memory audit rows:
- `proj.blocker_detected` on new blocker.
- `proj.blocker_resolved` on auto-resolution (carries reason).
- `proj.blocker_stale` once dwell ≥ 3 business days (idempotent — once per blocker).
7. **MUST** notify via TASK-CUO-101 webhook on stale events (placeholder URL `https://cuo-internal.cyberos.world/notify`; OPSEC: not exposed publicly). Payload: `{tenant_id, issue_id, blocker_id, target_ref, dwell_business_days, assignee_subject_id}`.
8. **MUST** run hourly dwell scan; on-demand via `cyberos blocker scan [--tenant <uuid>]` CLI.
9. **MUST** RLS-enforce.
10. **MUST** emit OTel metrics:
- `proj_blockers_active{tenant_id_bucket}` (gauge).
- `proj_blockers_detected_total{kind}` (counter).
- `proj_blockers_resolved_total{auto_reason}` (counter).
- `proj_blockers_stale_total` (counter).
11. **MUST** redact PII from `target_ref` (especially FreeText kind) before memory audit emit via TASK-MEMORY-111 ruleset.
12. **MUST** support manual operator override: `POST /api/proj/blockers/:id/resolve` with reason; emits `proj.blocker_resolved` with `auto_reason="manual"`.
13. **MUST** support manual creation: `POST /api/proj/issues/:id/blockers` for cases where parser missed (e.g. blocker discussed verbally). Tracks `detected_by_subject_id` as the operator.
14. **MUST** track escalation: after `stale_notified_at + 7 business days` without resolution, emit `proj.blocker_escalated` SEV-2 + notify tenant admin (in addition to assignee).
15. **MUST** support per-tenant dwell threshold override: `cyberos_proj_tenant_settings.blocker_stale_business_days` (default 3); SLA-heavy tenants may want stricter (1d).
16. **MUST** include `cuo_notify_attempts INT` + `last_cuo_notify_at` columns for tracking notification reliability; if 3 consecutive notification failures, SEV-1 alert on CUO health.
17. **MUST** support a "dependency cycle" detection: if Issue A is blocked by Issue B AND Issue B is blocked by Issue A, emit `proj.blocker_cycle_detected` SEV-2; do NOT auto-break.
18. **MUST** include `blocker_age_distribution` histogram in metrics — bucketed by business days (0-1, 1-3, 3-7, 7-14, 14+).
19. **MUST** support a "snooze" action: operator can snooze a blocker's stale notification for N days (max 14); `snoozed_until` column; scan skips snoozed blockers.
20. **MUST** include `mentioned_users` (parsed @-mentions from the blocker comment) in audit payload — useful for "who was tagged as the unblock owner."
21. **MUST** emit `proj.blocker_resolved_diff` audit row containing the comment text that triggered auto-resolution (e.g. the "unblocked" comment). Operators reviewing resolution can see the context.
22. **MUST** support cross-engagement blocker references: Issue X (engagement A) blocked by Issue Y (engagement B) — link is valid if caller has read scope on both engagements.

---

## §2 — Why this design (rationale for humans)

**Why comment-stream parsing (DEC-320)?** Operators already write "blocked by X" in comments; structured-label workflows (drag to "Blocked" column) are friction. Parsing existing prose = zero friction adoption.

**Why three target classes (§1 #2)?** IssueMention is the strongest signal (machine-resolvable). MemoryPath catches memory-anchored decisions. FreeText handles human context ("blocked by customer's response") — can't auto-resolve but still timed.

**Why business days not calendar (§1 #5)?** A blocker filed Friday afternoon shouldn't ping Monday morning as "3 days stale." Business-day math + VN holidays = culturally-correct.

**Why CUO Notify routing (§1 #7, DEC-322)?** Direct emails create email-noise that operators filter to a forgotten folder. CUO is the unified inbox; routing through it makes blockers visible alongside other surfaces.

**Why hourly scan (§1 #8)?** Dwell is wall-clock-driven; hourly granularity is the right resolution for "3 business days = 72 working hours." More-frequent burns CPU; less-frequent delays signal.

**Why audit per state transition (§1 #6)?** Operators investigating "did this issue ever get blocked" need first-class events. Generic comment-history doesn't separate "blocked" from "everything else."

**Why redact target_ref (§1 #11)?** FreeText blockers ("waiting on alice@x.com response") embed PII. Years of accumulated audit = PII bloat.

**Why manual resolve (§1 #12)?** Auto-resolve covers known patterns; real-world blockers resolve in ways the parser misses (operator agrees verbally to unblock). Manual override prevents indefinite-stale.

**Why manual create (§1 #13)?** Operators discussing blockers in meetings (not comments) want to record them in PROJ for tracking. Manual create closes that gap.

**Why escalation at 7d (§1 #14)?** 3-day stale = assignee should fix; 10-day total = needs management attention. Escalation forces visibility.

**Why per-tenant dwell threshold (§1 #15)?** Enterprise SLAs vary; some clients want next-day escalation, others week.

**Why CUO notification health tracking (§1 #16)?** Silent notification failures = operators don't know blockers exist. Tracking + SEV-1 alert ensures detection.

**Why cycle detection (§1 #17)?** A↔B blocker cycle = both stuck indefinitely; auto-break would lose data. Detection alerts operator to intervene.

**Why age distribution histogram (§1 #18)?** Operators tracking team health see "we have 5 blockers in 7-14 day range" — actionable.

**Why snooze (§1 #19)?** Some blockers are legitimately long-running (vendor contract negotiation); snooze prevents nagging.

**Why mentioned_users in payload (§1 #20)?** Comment "blocked by Bob (@bob): waiting for design review" tags Bob as the unblock owner. Capturing the @-mention provides actionability.

**Why blocker_resolved_diff (§1 #21)?** Operators reviewing the auto-resolution see "Alice replied 'unblocked because customer approved'" — full context.

**Why cross-engagement blockers (§1 #22)?** Real engagements have dependencies across project boundaries; restricting to same-engagement misses these.

---

## §3 — API contract

### Migration

```sql
-- services/proj-sync/migrations/0011_blocker_state.sql

CREATE TABLE blocker_state (
    id                   UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    issue_id             UUID NOT NULL,
    comment_id           TEXT NOT NULL,
    blocker_kind         TEXT NOT NULL CHECK (blocker_kind IN ('issue_mention','memory_path','free_text')),
    target_ref           TEXT NOT NULL,
    detected_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    detected_by_subject_id UUID NOT NULL,
    resolved_at          TIMESTAMPTZ,
    resolved_by_comment_id TEXT,
    stale_notified_at    TIMESTAMPTZ,
    tenant_id            UUID NOT NULL
);
CREATE INDEX idx_blocker_active ON blocker_state (issue_id) WHERE resolved_at IS NULL;
CREATE INDEX idx_blocker_dwell  ON blocker_state (detected_at) WHERE resolved_at IS NULL AND stale_notified_at IS NULL;

ALTER TABLE blocker_state ENABLE ROW LEVEL SECURITY;
CREATE POLICY blocker_tenant_iso ON blocker_state
    USING (tenant_id = current_setting('app.tenant_id')::uuid);
```

### Rust API

```rust
// services/proj-sync/src/blocker/mod.rs
use serde::Serialize;

#[derive(Clone, Copy, Debug, Serialize, sqlx::Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum BlockerKind { IssueMention, MemoryPath, FreeText }

#[derive(Clone, Debug, Serialize)]
pub struct Blocker {
    pub id:                     uuid::Uuid,
    pub issue_id:               uuid::Uuid,
    pub comment_id:             String,
    pub blocker_kind:           BlockerKind,
    pub target_ref:             String,
    pub detected_at:            chrono::DateTime<chrono::Utc>,
    pub resolved_at:            Option<chrono::DateTime<chrono::Utc>>,
    pub dwell_business_days:    Option<i32>,
}
```

### Parser

```rust
// services/proj-sync/src/blocker/parser.rs
use crate::blocker::BlockerKind;
use once_cell::sync::Lazy;
use regex::Regex;

static BLOCKER_RX: Lazy<Regex> = Lazy::new(||
    Regex::new(r"(?i)\b(?:block(?:ed|s|ing))\s+by[:\s]+([^\n,;.]+)").unwrap());
static ISSUE_REF_RX: Lazy<Regex> = Lazy::new(||
    Regex::new(r"^\s*#([a-zA-Z0-9-]+)\s*$").unwrap());
static MEMORY_REF_RX: Lazy<Regex> = Lazy::new(||
    Regex::new(r"^\s*(memories/[^\s]+)\s*$").unwrap());

#[derive(Clone, Debug)]
pub struct ParsedBlocker {
    pub kind:        BlockerKind,
    pub target_ref:  String,
}

pub fn parse(comment_body: &str) -> Vec<ParsedBlocker> {
    BLOCKER_RX.captures_iter(comment_body)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().trim().to_string()))
        .map(|raw| {
            let kind = if ISSUE_REF_RX.is_match(&raw) { BlockerKind::IssueMention }
                       else if MEMORY_REF_RX.is_match(&raw) { BlockerKind::MemoryPath }
                       else { BlockerKind::FreeText };
            ParsedBlocker { kind, target_ref: raw }
        })
        .collect()
}
```

### Dwell scan

```rust
// services/proj-sync/src/blocker/dwell.rs
pub async fn scan_stale(pool: &sqlx::PgPool, tenant_id: uuid::Uuid) -> anyhow::Result<i32> {
    sqlx::query("SELECT set_config('app.tenant_id', $1, true)")
        .bind(tenant_id.to_string()).execute(pool).await?;

    let active: Vec<(uuid::Uuid, uuid::Uuid, chrono::DateTime<chrono::Utc>, String)> = sqlx::query_as(
        "SELECT id, issue_id, detected_at, target_ref
         FROM blocker_state
         WHERE resolved_at IS NULL AND stale_notified_at IS NULL"
    ).fetch_all(pool).await?;

    let now = chrono::Utc::now();
    let mut notified = 0;
    for (id, issue_id, detected_at, target_ref) in active {
        let dwell = business_days_between(detected_at, now);
        if dwell >= 3 {
            sqlx::query("UPDATE blocker_state SET stale_notified_at = NOW() WHERE id = $1")
                .bind(id).execute(pool).await?;
            cuo_notify(tenant_id, issue_id, id, &target_ref, dwell).await?;
            emit_memory_row("proj.blocker_stale", serde_json::json!({
                "blocker_id": id, "issue_id": issue_id,
                "target_ref": target_ref, "dwell_business_days": dwell,
            })).await;
            metrics::counter!("proj_blockers_stale_total").increment(1);
            notified += 1;
        }
    }
    Ok(notified)
}

fn business_days_between(start: chrono::DateTime<chrono::Utc>, end: chrono::DateTime<chrono::Utc>) -> i32 {
    use chrono::Datelike;
    let mut days = 0;
    let mut d = start.date_naive();
    let end_d = end.date_naive();
    let holidays = parse_vn_holidays_env();
    while d < end_d {
        d = d + chrono::Duration::days(1);
        if matches!(d.weekday(), chrono::Weekday::Sat | chrono::Weekday::Sun) { continue; }
        if holidays.contains(&d) { continue; }
        days += 1;
    }
    days
}

fn parse_vn_holidays_env() -> std::collections::HashSet<chrono::NaiveDate> {
    std::env::var("CYBEROS_HOLIDAYS_VN").ok()
        .map(|s| s.split(',').filter_map(|d| d.trim().parse().ok()).collect())
        .unwrap_or_default()
}
```

---

## §4 — Acceptance criteria

1. **Parse "blocked by #abc123"** → 1 ParsedBlocker, kind=IssueMention, target_ref="#abc123".
2. **Parse "blocks: memories/x/y.md"** → kind=MemoryPath.
3. **Parse "blocked by customer feedback"** → kind=FreeText.
4. **Multiple blockers in one comment** — "blocked by #X, blocked by #Y" → 2 parsed.
5. **Comment without blocker prose** → 0 parsed.
6. **Insert detected blocker into blocker_state** — on comment insert → row appears; `proj.blocker_detected` audit row.
7. **Auto-resolve on target Done** — referenced issue transitions to Done → blocker row resolved_at set; `proj.blocker_resolved` row.
8. **Auto-resolve on `unblocked` comment** — followup comment "unblocked" → resolved.
9. **Auto-resolve on issue cancellation** — block-owning issue cancelled → all its blockers resolved (moot).
10. **Dwell scan ≥ 3 business days** — fixture: detected_at = Mon 09:00; current = Thu 11:00 → dwell = 3; stale_notified_at set; CUO webhook fired.
11. **Weekend doesn't count** — Fri detected → Mon = dwell 1 (not 3).
12. **VN holiday env honoured** — config holiday 2026-09-02; Tue–Wed straddles it → dwell skips that day.
13. **Stale notification idempotent** — second scan after notification → no duplicate webhook; metric unchanged.
14. **memory audit on detect/resolve/stale** — all 3 kinds emitted at correct events.
15. **CUO Notify webhook payload schema** — payload matches §1 #7 spec.
16. **RLS tenant isolation** — tenant A's blockers invisible to tenant B.
17. **Gauge `proj_blockers_active`** — accurate count of unresolved blockers per tenant.
18. **target_ref PII redacted in audit** — FreeText "waiting on alice@x.com" → audit row contains "<EMAIL>" (AC for §1 #11).
19. **Manual resolve emits proj.blocker_resolved with reason=manual** — POST /:id/resolve → audit row (AC for §1 #12).
20. **Manual create works** — POST /blockers with fields → row inserted; detected_by_subject_id = operator (AC for §1 #13).
21. **Escalation at 10 business days** — stale + 7 more business days → `proj.blocker_escalated` SEV-2 (AC for §1 #14).
22. **Per-tenant dwell override** — set blocker_stale_business_days=1; blocker after 1 day flagged stale (AC for §1 #15).
23. **CUO notification failure tracking** — 3 consecutive failures → SEV-1 on CUO health (AC for §1 #16).
24. **Cycle detected** — A blocked by B, B blocked by A → `proj.blocker_cycle_detected` SEV-2 (AC for §1 #17).
25. **Age distribution histogram populated** — distinct blocker ages across buckets (AC for §1 #18).
26. **Snooze skips stale scan** — operator snoozes for 7d; scan during snooze window → no notification (AC for §1 #19).
27. **mentioned_users captured** — comment "blocked by @bob waiting on review" → audit payload mentioned_users=["bob"] (AC for §1 #20).
28. **blocker_resolved_diff contains comment** — auto-resolve via "unblocked" comment → audit row contains comment text (AC for §1 #21).
29. **Cross-engagement blocker validated** — blocker referring to another engagement's issue allowed only with read scope (AC for §1 #22).

---

## §5 — Verification

```rust
#[test]
fn parses_issue_mention() {
    let p = parse("This is blocked by #ABC-123 right now.");
    assert_eq!(p.len(), 1);
    assert_eq!(p[0].kind, BlockerKind::IssueMention);
    assert_eq!(p[0].target_ref, "#ABC-123");
}

#[test]
fn parses_memory_path() {
    let p = parse("blocks: memories/projects/cyberos/decisions/DEC-300.md");
    assert_eq!(p[0].kind, BlockerKind::MemoryPath);
}

#[test]
fn parses_free_text() {
    let p = parse("blocked by customer response");
    assert_eq!(p[0].kind, BlockerKind::FreeText);
}

#[test]
fn parses_multiple_per_comment() {
    let p = parse("blocked by #X, blocked by #Y. blocking #Z");
    assert_eq!(p.len(), 3);
}

#[test]
fn weekend_skipped() {
    let mon = chrono::DateTime::parse_from_rfc3339("2026-05-11T09:00:00Z").unwrap().with_timezone(&chrono::Utc);
    let next_mon = chrono::DateTime::parse_from_rfc3339("2026-05-18T09:00:00Z").unwrap().with_timezone(&chrono::Utc);
    // Mon-Mon = 5 business days
    assert_eq!(business_days_between(mon, next_mon), 5);
}

#[tokio::test]
async fn auto_resolve_on_target_done() {
    let env = TestEnv::new().await;
    let target = env.create_issue().await;
    let blocker_issue = env.create_issue_with_blocker_comment(target).await;
    apply_transition(&env.pool, target, IssueStatus::Done, env.alice(), None).await.unwrap();
    let blk: Blocker = env.read_blocker(blocker_issue).await;
    assert!(blk.resolved_at.is_some());
}

#[tokio::test]
async fn stale_notifies_at_three_business_days() {
    let env = TestEnv::with_paused_time().await;
    let blocker_issue = env.create_issue_with_blocker_comment_at("monday 9am").await;
    env.advance_to("thursday 9am").await;
    let notified = scan_stale(&env.pool, env.tenant_id()).await.unwrap();
    assert_eq!(notified, 1);
    let alert = env.cuo.latest_notification().await;
    assert_eq!(alert["dwell_business_days"], 3);
}
```

---

## §6 — Implementation skeleton

(API + DB above.)

---

## §7 — Dependencies

- **TASK-PROJ-003** — comment events trigger parser.
- **TASK-PROJ-004** — status transition triggers auto-resolve.
- **TASK-CUO-101 (placeholder)** — notification webhook.
- **TASK-OBS-007** — sev-3 alert (informational).

---

## §8 — Example payloads

```json
{
  "kind": "proj.blocker_detected",
  "payload": {
    "blocker_id": "blk-...",
    "issue_id": "iss-...",
    "comment_id": "cmt-...",
    "blocker_kind": "issue_mention",
    "target_ref": "#ABC-123",
    "detected_at_ns": 1747407137483000000,
    "trace_id": "0af..."
  }
}
```

---

## §9 — Open questions

All resolved. Deferred:
- ML-based "soft blocker" detection (e.g. "I'm waiting on..." without explicit keyword) — slice 4+.
- Per-tenant configurable dwell threshold — slice 4+.
- Bidirectional blocker (issue X blocks issue Y AND issue Y is blocked by X auto-link) — slice 4+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Comment parse regex fails on edge case | unit test fixtures catch | False negative | Operator files bug |
| Target issue mentioned doesn't exist | resolution lookup Err | Blocker stored with kind=FreeText (degraded) | Operator inspects |
| CUO webhook timeout | retry once; fail | sev-3 alarm via TASK-OBS-007 | Operator restores CUO |
| Holiday env malformed | parse Err | Falls back to weekend-only skip | Operator fixes env |
| Stale_notified_at race (two scans concurrent) | UPDATE WHERE NULL → second sees row already notified | No duplicate | None |
| Issue transitions but blocker resolve fails | sqlx Err | Blocker stays active; next scan retries | Acceptable |
| Comment edited to remove "blocked by" | edit not handled in slice 3 | Original blocker stays | Slice 4+ |
| 10K active blockers in one tenant | scan slow | sev-2 latency alarm | Slice 4+ paginate |
| RLS bypass | RLS policy | 0 rows | None |
| Concurrent unblocked comment + Done transition | both resolve | Idempotent UPDATE | None |
| MemoryEmit fails | row created; audit lost | sev-2 | Operator restores |
| Free-text blocker with secrets | redacted via TASK-MEMORY-111 at audit emit | Safe | None |
| Manual resolve without reason | 400 | None | Caller |
| Manual create with invalid target | falls back to FreeText | None | None |
| Escalation fires before assignee fixes (race) | dedup at OBS | None | None |
| Per-tenant dwell = 0 | every blocker immediately stale | as-configured | Operator |
| CUO notify down 3+ days | SEV-1 alert; blockers accumulate | operator restores | None |
| Cycle detection misses indirect cycle (A→B→C→A) | DFS depth 100 covers | detected | None |
| Histogram bucket out of range (>14d) | overflow bucket counts | None | None |
| Snooze longer than allowed | rejected | 400 | Caller |
| Snoozed blocker resolves naturally | cancel-on-resolve cleans snooze | None | None |
| Mention contains non-existent user | stored verbatim; resolver returns null | None | None |
| Resolved_diff > 5KB | truncated to 5KB | None | None |
| Cross-engagement target without scope | 403 | None | Caller |
| Engagement archived mid-blocker | resolve_on_archive auto-resolves | proj.blocker_resolved | None |
| Concurrent manual + auto resolve | last wins; idempotent | None | None |
| Two operators snooze same blocker | latest wins | None | None |
| Snooze on already-resolved blocker | rejected | 409 | Caller |
| Mentioned_users with > 50 mentions | truncated to 50 + warning | None | None |

---

## §11 — Implementation notes

- The regex avoids `blockchain` / `roadblock` etc. via `\bblock` word-boundary; `blocked|blocks|blocking` covers tense.
- Issue mention syntax (`#ABC-123` or `#<uuid>`) needs resolution against issues table; mismatches fall to FreeText.
- VN holidays default to: Tết (Lunar New Year, variable), 30/4 + 1/5, 2/9 (National Day), Hùng King (variable). Env override allows custom.
- CUO Notify URL is internal; production deployment sets via `CYBEROS_CUO_NOTIFY_URL` env.
- The `dwell_business_days` field is computed at query time (not stored) for accuracy across server time vs. wall clock.
- Hourly cron scan handles dwell threshold crossings; doesn't re-emit for already-notified blockers.
- Free-text blockers can't auto-resolve via target Done; only via explicit unblocked/resolved comment or owner-issue Done|Cancelled.
- PII redaction of target_ref runs at audit emit, not at storage — operator UI shows the original.
- Manual create + manual resolve provide operator-controlled paths for cases the parser misses.
- Escalation at 10 business days is calibrated: 3 days assignee fix + 7 days unattended = needs management.
- Per-tenant dwell threshold is per-engagement override (slice 4+); current MVP is per-tenant only.
- CUO notification health uses exponential backoff (1s, 2s, 4s); 3 failures = retry exhausted.
- Cycle detection is O(N²) in worst case but bounded by blocker count per tenant (~hundreds typically); fast in practice.
- Age distribution histogram updates on scan; buckets fixed for cross-tenant comparability.
- Snooze uses `snoozed_until TIMESTAMPTZ`; expired snoozes are cleared at next scan.
- Mentioned_users uses regex `@([a-zA-Z0-9_]+)` to extract; resolves against users table (best-effort).
- Resolved_diff stores up to 5KB of the resolving comment text; longer truncated.
- Cross-engagement blockers require the caller to have read scope on both engagements; enforced via RLS at insert.
- We considered NLP-based blocker detection but rejected: false positives on metaphorical "blocked" mentions; regex is precise + auditable.
- The `proj.blocker_escalated` SEV-2 notification is routed to tenant admin via TASK-OBS-007 + the assignee via CUO.
- Snooze max 14 days because: anything longer = blocker isn't really tracked; operator should remove or escalate.
- The blocker comment that triggered detection is the canonical reference; comment edits don't re-trigger detection (slice 4+ feature).
- Manual create + manual resolve emit memory audit rows with `auto_reason="manual"` so analytics can distinguish from automated detection.
- The mentioned_users extraction handles Vietnamese username conventions (with diacritics stripped for resolution).

---

*End of TASK-PROJ-011.*
