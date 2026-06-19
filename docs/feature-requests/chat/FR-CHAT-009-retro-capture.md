---
id: FR-CHAT-009
title: "Retro-capture flow — `@lumi remember the last N messages` with per-message opt-in checkboxes and aggregated memory memory"
module: CHAT
priority: SHOULD
status: done
verify: T
phase: P1
milestone: P1 · slice 2
slice: 2
owner: Stephen Cheng
created: 2026-05-16
shipped: null
memory_chain_hash: null
related_frs: [FR-CHAT-005, FR-CHAT-008, FR-MEMORY-111, FR-MEMORY-101]
depends_on: [FR-CHAT-008]
blocks: []

source_pages:
  - website/docs/modules/chat.html#retro-capture
source_decisions:
  - DEC-500 (retro-capture surfaces last N messages as ephemeral picker; user opts in per-message)
  - DEC-501 (aggregated capture saved as ONE memory memory; sync_class follows channel)
  - DEC-502 (N cap = 100 messages; protects against noisy-capture)

language: rust 1.81 + go
service: cyberos/services/chat-lumi/
new_files:
  - services/chat-lumi/src/retro/mod.rs
  - services/chat-lumi/src/retro/picker.rs
  - services/chat-lumi/src/retro/aggregate.rs
  - services/chat-lumi/tests/retro_test.rs
modified_files:
  - services/chat-lumi/src/main.rs                   # add retro command dispatch
  - services/chat/plugins/cyberos-lumi-router/main.go # ephemeral message support
allowed_tools:
  - file_read: services/chat-lumi/**
  - file_write: services/chat-lumi/{src,tests}/**, services/chat/plugins/cyberos-lumi-router/**
  - bash: cd services/chat-lumi && cargo test retro
disallowed_tools:
  - capture more than 100 messages (per DEC-502)
  - auto-capture without explicit user opt-in (per DEC-500)

effort_hours: 6
sub_tasks:
  - "0.5h: retro/mod.rs — RetroRequest struct + Outcome enum"
  - "1.0h: parser: match `(?i)@lumi\\s+(remember|capture)\\s+(?:the\\s+)?last\\s+(\\d+)\\s+messages?`"
  - "1.5h: picker.rs — generate Mattermost ephemeral message with checkboxes per-message"
  - "1.0h: aggregate.rs — collect opt-in messages → compose markdown body → emit single memory memory at memories/chat/<channel>/captures/<ts>.md"
  - "0.5h: memory audit 'chat.retro_capture_completed'"
  - "1.5h: retro_test.rs — parse + cap-100 + aggregate + opt-in subset"
risk_if_skipped: "Without retro-capture, conversations only become memories via @lumi mention in real-time. Past insights stay buried in chat. With it, retroactively elevating 'we decided X yesterday' to a memory memory takes 30s instead of copying messages by hand."
---

## §1 — Description (BCP-14 normative)

The retro-capture flow **MUST** offer per-message opt-in capture of the last N chat messages into a single memory memory. The contract:

1. **MUST** trigger via @lumi command matching regex `(?i)@lumi\s+(remember|capture)\s+(?:the\s+)?last\s+(\d+)\s+messages?`.
2. **MUST** validate N ≤ 100; > 100 → reply "capture max is 100 messages"; do not proceed.
3. **MUST** fetch the last N messages from the same channel (excluding the @lumi command itself and system messages).
4. **MUST** generate an ephemeral Mattermost message (visible only to invoker) with a checkbox per fetched message + "Capture selected" button.
5. **MUST** on submit:
   - Aggregate selected messages into markdown body: `[2026-05-16T14:32 @alice] ...\n[2026-05-16T14:35 @bob] ...`.
   - PII-redact aggregated body via FR-MEMORY-111.
   - Save as memory memory at `memories/chat/<channel_id>/captures/<unix_ts>.md` with frontmatter `{kind: chat_capture, sync_class: <channel_sync_class>, captured_by: <subject_id>, captured_at: <iso>, source_channel_id: <id>, source_message_ids: [...], message_count: <n>}`.
6. **MUST** emit memory audit `chat.retro_capture_completed` with payload `{channel_id, captured_by, message_count_requested, message_count_selected, memory_path, trace_id}`.
7. **MUST** reply to the user (threaded to original command): "✓ Captured N messages → [memory link]".
8. **MUST** support cancellation: ephemeral picker has "Cancel" button → no capture, no memory write, audit `chat.retro_capture_cancelled`.
9. **MUST** RLS-enforce: channel access required (FR-AUTH-003).
10. **MUST** emit OTel metrics:
    - `chat_retro_captures_total{outcome}` (outcome ∈ completed | cancelled | over_limit).
    - `chat_retro_messages_per_capture` (histogram).
11. **MUST NOT** include the `@lumi remember ...` command post itself in the fetched messages (it would self-include and inflate the count).
12. **MUST NOT** include system / bot messages (post.type starting with `system_`, `system_join_channel`, etc.) in the picker — they're noise.
13. **MUST** preview each message in the picker with the first 200 characters of body + author display name + relative timestamp ("3 minutes ago"); full body stored in memory if selected.
14. **MUST** support "Select all" + "Select none" + "Invert selection" picker actions to speed up the common case (operator wants most-but-not-all).
15. **MUST** emit `chat.retro_capture_started` memory audit when the picker is posted, before the user submits. Payload includes `{channel_id, requested_by, n_offered, command_post_id, trace_id}`. Pairs with `_completed` or `_cancelled` for full lifecycle visibility.
16. **MUST** time-bound picker submission: 1h TTL from picker post. After TTL, submit returns "Picker expired; please re-issue the command." + audit `chat.retro_capture_expired`.
17. **MUST** support per-message "include surrounding context" toggle on each picker row: if checked, the memory will also include the message immediately preceding and following the selected one (with `[context]` marker). Useful when a single message references "this" or "that" without context.
18. **MUST** dedup against prior captures: if a memory at `memories/chat/<channel>/captures/<ts>.md` already exists for the same selected_ids set in the last 24h, refuse with "These messages were already captured at <link>. Re-capture anyway?" with confirmation button. Prevents double-capture during operator hesitation.
19. **MUST** support `@lumi capture from <date> to <date>` (slice-extension form) in addition to "last N" — a date-range fetcher with N≤100 hard cap still enforced.
20. **MUST** propagate W3C `trace_id` from the @lumi command post through to the picker, submit, and memory frontmatter; downstream consumers can correlate the full retro lifecycle.
21. **MUST** include the original command author (`captured_by`) AND any other users whose messages were selected in `meta.acl` of the memory frontmatter (per AGENTS.md §15). Even if `sync_class=shareable`, the ACL ensures opaque references.
22. **MUST** track aggregated body size: if redacted body exceeds 1MB, the memory is split into multiple `.md` files (`<ts>-part-1.md`, `<ts>-part-2.md`, ...) with a parent index `<ts>-index.md` linking all parts.
23. **MUST** support `--dry-run` flag in the @lumi command (`@lumi remember last 10 messages --dry-run`) that posts the picker but disables the submit button; shows preview of what WOULD be captured without writing.

---

## §2 — Why this design

**Why per-message opt-in (DEC-500)?** Bulk-capture noisy / off-topic messages dilutes signal. Checkbox UX = user selects exactly what's relevant.

**Why N ≤ 100 (DEC-502)?** Picker UI degrades past 100 (scroll fatigue + slow render). Hard cap forces narrower retro windows.

**Why single memory (DEC-501)?** Operator browsing memory sees one capture event, not 30 noisy rows. Aggregation = narrative.

**Why ephemeral picker (§1 #4)?** Other channel users don't need to see the picker UI; ephemeral keeps channel clean.

**Why redaction (§1 #5)?** Pre-existing channel messages may contain PII; capture should not bypass FR-MEMORY-111.

**Why exclude the command post (§1 #11)?** Self-inclusion means the count is `N` but the picker shows `N-1` user messages + 1 command. Confusing and noisy. Skipping the command keeps the count honest.

**Why exclude system messages (§1 #12)?** Mattermost emits "Alice joined the channel" auto-messages; capturing those produces ledger spam without insight. Always filtered.

**Why 200-char preview (§1 #13)?** Long messages would explode the picker UI; truncation keeps it scannable. Full body still captured if selected.

**Why bulk-select actions (§1 #14)?** The common case is "capture most of these, skip 1-2 noise messages." Three actions cover all selection patterns with one click.

**Why _started audit (§1 #15)?** Pairing with _completed/_cancelled gives operators a full lifecycle audit. A picker that's posted but never submitted is operationally relevant ("the user changed their mind" vs "the user got distracted").

**Why 1h picker TTL (§1 #16)?** Pickers older than 1h likely refer to a stale context (channel has new messages, selected IDs may be stale). Forcing re-issue keeps captures fresh.

**Why per-message context toggle (§1 #17)?** Captured messages often reference earlier/later messages obliquely. Letting the user opt-in to adjacent context per message gives precision without forcing everyone to capture full surrounding windows.

**Why dedup against prior (§1 #18)?** Operators sometimes capture, get interrupted, return, and capture the same set again. Loud confirmation avoids silent duplicates in memory.

**Why date-range form (§1 #19)?** Some retros happen days later ("capture last Friday's discussion"). Last-N is awkward when N is unknown; date-range is the natural query.

**Why ACL with all participants (§1 #21)?** A `shareable` memory containing messages by Bob means Bob is implicitly a participant. Listing all participants in the ACL respects their identity-as-contributor without changing the sync_class.

**Why split memory at 1MB (§1 #22)?** memory memory files >1MB have indexer + render performance issues. Splitting at the boundary preserves performance while keeping the full capture available via the index file.

**Why --dry-run (§1 #23)?** Operators want preview-before-commit for the high-stakes case (sensitive channel capture). Dry-run shows the picker without enabling commit.

---

## §3 — API contract

```rust
// services/chat-lumi/src/retro/mod.rs
use once_cell::sync::Lazy;
use regex::Regex;

static RETRO_RX: Lazy<Regex> = Lazy::new(||
    Regex::new(r"(?i)@lumi\s+(remember|capture)\s+(?:the\s+)?last\s+(\d+)\s+messages?").unwrap());

pub fn parse_retro_command(body: &str) -> Option<i32> {
    RETRO_RX.captures(body)
        .and_then(|c| c.get(2))
        .and_then(|m| m.as_str().parse().ok())
}

pub async fn handle_retro(req: RetroRequest) -> Result<(), RetroError> {
    if req.n > 100 {
        reply::post_to_user(&req.channel_id, &req.user_id,
            "Capture max is 100 messages. Try a smaller window.").await?;
        metrics::counter!("chat_retro_captures_total", "outcome" => "over_limit").increment(1);
        return Err(RetroError::OverLimit);
    }

    let messages = mm_api::fetch_last_n(&req.channel_id, req.n, &req.command_post_id).await?;
    picker::post_ephemeral(&req.channel_id, &req.user_id, &messages).await?;
    // Submit handled in separate webhook handler post_picker_submit()
    Ok(())
}

pub async fn post_picker_submit(submit: PickerSubmit) -> Result<(), RetroError> {
    if submit.selected_ids.is_empty() {
        reply::post_to_user(&submit.channel_id, &submit.user_id,
            "Nothing selected; capture cancelled.").await?;
        emit_memory_row("chat.retro_capture_cancelled", json!({
            "channel_id": submit.channel_id, "by": submit.user_id,
        })).await;
        return Ok(());
    }
    let messages = mm_api::fetch_by_ids(&submit.selected_ids).await?;
    let body = aggregate::compose_markdown(&messages);
    let redacted = pii::scan_and_redact(&body, &[]).await?.redacted_body;

    let channel = mm_api::get_channel(&submit.channel_id).await?;
    let sync_class = match channel.privacy_type {
        "P" | "D" => "private",
        _ => "shareable",
    };

    let ts = chrono::Utc::now().timestamp();
    let memory_path = format!("memories/chat/{}/captures/{}.md", submit.channel_id, ts);
    let frontmatter = format!(
        "---\nkind: chat_capture\nsync_class: {}\ncaptured_by: {}\ncaptured_at: {}\nsource_channel_id: {}\nsource_message_ids: {}\nmessage_count: {}\n---\n\n",
        sync_class, submit.user_id, chrono::Utc::now().to_rfc3339(),
        submit.channel_id,
        serde_json::to_string(&submit.selected_ids).unwrap(),
        submit.selected_ids.len()
    );
    memory_writer::put_memory(&memory_path, format!("{frontmatter}{redacted}").as_bytes()).await?;

    emit_memory_row("chat.retro_capture_completed", json!({
        "channel_id": submit.channel_id,
        "captured_by": submit.user_id,
        "message_count_requested": submit.n_requested,
        "message_count_selected": submit.selected_ids.len(),
        "memory_path": memory_path,
        "trace_id": current_trace_id(),
    })).await;
    metrics::counter!("chat_retro_captures_total", "outcome" => "completed").increment(1);
    metrics::histogram!("chat_retro_messages_per_capture")
        .record(submit.selected_ids.len() as f64);

    reply::post_to_channel(&submit.channel_id,
        &format!("✓ Captured {} messages → {}", submit.selected_ids.len(), memory_path)).await?;
    Ok(())
}
```

```rust
// services/chat-lumi/src/retro/aggregate.rs
pub fn compose_markdown(messages: &[CapturedMessage]) -> String {
    let mut out = String::new();
    for m in messages {
        if m.is_context_for_other.is_some() {
            out.push_str("[context] ");
        }
        out.push_str(&format!("[{} @{}] {}\n",
            m.timestamp.format("%Y-%m-%dT%H:%M"),
            m.username,
            m.body));
    }
    out
}

pub struct CapturedMessage {
    pub id:        String,
    pub username:  String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub body:      String,
    pub is_context_for_other: Option<String>,
}

/// Splits a redacted body into ≤ 1MB chunks at line boundaries; returns
/// (parts, index_body) where parts is each part's body and index_body lists them.
pub fn split_if_oversized(body: &str) -> (Vec<String>, Option<String>) {
    const MAX_BYTES: usize = 1_048_576;
    if body.len() <= MAX_BYTES {
        return (vec![body.to_owned()], None);
    }
    let mut parts = Vec::new();
    let mut cur = String::with_capacity(MAX_BYTES);
    for line in body.lines() {
        if cur.len() + line.len() + 1 > MAX_BYTES {
            parts.push(std::mem::take(&mut cur));
        }
        cur.push_str(line); cur.push('\n');
    }
    if !cur.is_empty() { parts.push(cur); }
    let index = format!(
        "# Capture index\n\nThis capture was split into {} parts due to size.\n\n{}\n",
        parts.len(),
        (0..parts.len()).map(|i| format!("- [Part {}](./part-{}.md)", i+1, i+1)).collect::<Vec<_>>().join("\n")
    );
    (parts, Some(index))
}
```

### picker.rs — ephemeral message + interactive actions

```rust
// services/chat-lumi/src/retro/picker.rs
use serde_json::json;

pub async fn post_ephemeral(
    channel_id: &str,
    user_id: &str,
    messages: &[Message],
    trace_id: &str,
    dry_run: bool,
) -> Result<String, MmError> {
    let actions = messages.iter().map(|m| {
        json!({
            "id":   format!("toggle-{}", m.id),
            "name": format!("{} · {} · {}",
                m.username,
                relative_time(m.create_at),
                truncate_200(&m.message)),
            "type": "button",
            "style": "default",
            "integration": {
                "url": format!("{}/retro/toggle", lumi_service_base()),
                "context": {"post_id": m.id, "action": "toggle"},
            },
        })
    }).collect::<Vec<_>>();

    let mut all_actions = actions;
    all_actions.push(json!({
        "id":   "select-all",  "name": "Select all",
        "type": "button", "style": "secondary",
        "integration": {"url": format!("{}/retro/select-all", lumi_service_base())},
    }));
    all_actions.push(json!({
        "id":   "select-none", "name": "Select none",
        "type": "button", "style": "secondary",
        "integration": {"url": format!("{}/retro/select-none", lumi_service_base())},
    }));
    all_actions.push(json!({
        "id":   "invert",      "name": "Invert selection",
        "type": "button", "style": "secondary",
        "integration": {"url": format!("{}/retro/invert", lumi_service_base())},
    }));
    if !dry_run {
        all_actions.push(json!({
            "id":   "commit",  "name": "Capture selected",
            "type": "button", "style": "primary",
            "integration": {"url": format!("{}/retro/commit", lumi_service_base())},
        }));
    } else {
        all_actions.push(json!({
            "id":   "commit-disabled",  "name": "(dry-run — commit disabled)",
            "type": "button", "style": "default",
        }));
    }
    all_actions.push(json!({
        "id":   "cancel",  "name": "Cancel",
        "type": "button", "style": "danger",
        "integration": {"url": format!("{}/retro/cancel", lumi_service_base())},
    }));

    let body = json!({
        "channel_id": channel_id,
        "user_id":    user_id,
        "props": {
            "attachments": [{
                "title": "Select messages to capture",
                "text":  format!("Showing the last {} messages. Toggle each to include in the memory capture.", messages.len()),
                "actions": all_actions,
            }],
            "trace_id":  trace_id,
        },
    });
    let resp = mm_client().post("/api/v4/posts/ephemeral", body).await?;
    Ok(resp["id"].as_str().unwrap().to_owned())
}

fn relative_time(ts: i64) -> String {
    let now = chrono::Utc::now().timestamp() * 1000;
    let delta_secs = (now - ts) / 1000;
    match delta_secs {
        n if n < 60        => format!("{}s ago", n),
        n if n < 3600      => format!("{}m ago", n / 60),
        n if n < 86400     => format!("{}h ago", n / 3600),
        n                  => format!("{}d ago", n / 86400),
    }
}
```

### Picker state — in-process + Redis

```rust
// services/chat-lumi/src/retro/state.rs
pub struct PickerState {
    pub picker_post_id:    String,
    pub channel_id:        String,
    pub user_id:           String,
    pub command_post_id:   String,
    pub n_requested:       i32,
    pub message_ids:       Vec<String>,
    pub selected_ids:      std::collections::HashSet<String>,
    pub include_context:   std::collections::HashSet<String>,
    pub trace_id:          String,
    pub dry_run:           bool,
    pub created_at:        chrono::DateTime<chrono::Utc>,
}

pub async fn store_state(state: &PickerState) -> RedisResult<()> {
    let key = format!("retro:picker:{}", state.picker_post_id);
    let bytes = serde_json::to_vec(state).unwrap();
    let mut conn = REDIS.get_async_connection().await?;
    let _: () = redis::cmd("SET").arg(&key).arg(&bytes)
        .arg("EX").arg(3600).query_async(&mut conn).await?;  // 1h TTL
    Ok(())
}

pub async fn fetch_state(picker_post_id: &str) -> RedisResult<Option<PickerState>> {
    let key = format!("retro:picker:{}", picker_post_id);
    let mut conn = REDIS.get_async_connection().await?;
    let bytes: Option<Vec<u8>> = redis::cmd("GET").arg(&key).query_async(&mut conn).await?;
    Ok(bytes.map(|b| serde_json::from_slice(&b).unwrap()))
}
```

### Dedup against prior captures

```rust
// services/chat-lumi/src/retro/dedup.rs
pub async fn find_recent_capture(
    channel_id: &str,
    message_ids: &[String],
    window_hours: u32,
) -> Option<String> {
    let cutoff_ts = chrono::Utc::now().timestamp() - (window_hours as i64 * 3600);
    let glob = format!("memories/chat/{}/captures/*.md", channel_id);
    for path in memory_writer::glob(&glob).await.ok()? {
        let basename_ts = path.file_stem().and_then(|s| s.to_str()?.parse::<i64>().ok())?;
        if basename_ts < cutoff_ts { continue; }
        let memory = memory_writer::read(&path).await.ok()?;
        let prior_ids = parse_frontmatter_array(&memory, "source_message_ids")?;
        if same_set(&prior_ids, message_ids) {
            return Some(path.display().to_string());
        }
    }
    None
}
```

---

## §4 — Acceptance criteria

1. **Parse `@lumi remember the last 10 messages`** → N=10.
2. **Parse `@lumi capture last 5 messages`** → N=5.
3. **Case-insensitive command** — `@LUMI REMEMBER LAST 3 MESSAGES` → N=3.
4. **N > 100 rejected with reply**.
5. **Picker has checkboxes per message** — ephemeral msg with N checkboxes.
6. **Aggregated memory created** — single file at canonical path.
7. **Frontmatter populated** — kind, sync_class, captured_by, source_message_ids.
8. **PII redacted in aggregated body** — email in source → <EMAIL> in memory.
9. **sync_class follows channel** — private channel → private memory.
10. **memory audit chat.retro_capture_completed** — row emitted.
11. **Cancel emits chat.retro_capture_cancelled** — picker cancel button → no memory; cancellation row.
12. **Reply to channel with memory link**.
13. **OTel counter increments per outcome**.
14. **RLS isolates** — user without channel access cannot trigger.
15. **Command post excluded from fetched messages** — fixture: command post in last-10 → picker shows 10 user messages, not 9 + command (AC for §1 #11).
16. **System messages excluded** — fixture channel with "Alice joined" auto-message in last-10 → picker shows 10 user messages, not 9 + system (AC for §1 #12).
17. **Picker shows 200-char preview + relative time** — observe each picker action label has username + relative-time + truncated body (AC for §1 #13).
18. **Select all / Select none / Invert work** — fixture: click each; state in Redis reflects (AC for §1 #14).
19. **chat.retro_capture_started fires on picker post** — observe memory row before submit (AC for §1 #15).
20. **Picker TTL = 1h** — fixture: store state with `created_at = now() - 65min`; submit returns `Expired` + audit `chat.retro_capture_expired` (AC for §1 #16).
21. **Per-message context toggle adds adjacent messages** — fixture: select msg5 with context; observe msg4 + msg5 + msg6 in memory with `[context]` markers (AC for §1 #17).
22. **Dedup against prior 24h capture** — fixture: capture set {m1,m2}; immediate re-capture same set → "Already captured" + confirmation (AC for §1 #18).
23. **Date-range form parses** — `@lumi capture from 2026-05-15T00:00 to 2026-05-15T23:59` → fetches that range capped at 100 (AC for §1 #19).
24. **Trace_id propagated through lifecycle** — observe same trace_id in command post props, picker post props, memory frontmatter, all 3 audit rows (AC for §1 #20).
25. **Memory ACL includes all participants** — fixture: capture {msg by Alice, msg by Bob, msg by Carol}; observe `meta.acl: ["Alice", "Bob", "Carol"]` (AC for §1 #21).
26. **Memory split at 1MB** — fixture: 5MB aggregated body → 5 part files + 1 index file; index links all parts (AC for §1 #22).
27. **--dry-run posts picker without commit** — `@lumi remember last 10 --dry-run` → picker has no "Capture selected" button; "(dry-run — commit disabled)" label present (AC for §1 #23).
28. **Picker is ephemeral** — verify post visible only to invoker via MM API check `is_ephemeral=true` (AC for §1 #4).
29. **Cancel button → chat.retro_capture_cancelled + no memory** — fixture: click cancel; observe audit + no memory file created (AC for §1 #8).
30. **OTel histogram populated** — observe `chat_retro_messages_per_capture` has observation matching selected_ids.len() (AC for §1 #10).

---

## §5 — Verification

### Parser coverage

```rust
#[rstest]
#[case("@lumi remember the last 10 messages",  Some(RetroCmd::LastN(10)))]
#[case("@LUMI CAPTURE LAST 5 MESSAGES",        Some(RetroCmd::LastN(5)))]
#[case("@lumi remember last 100 messages",     Some(RetroCmd::LastN(100)))]
#[case("@lumi capture last 1 message",         Some(RetroCmd::LastN(1)))]
#[case("@lumi remember last 10 messages --dry-run", Some(RetroCmd::LastN(10).with_dry_run()))]
#[case("@lumi capture from 2026-05-15T00:00 to 2026-05-15T23:59",
       Some(RetroCmd::DateRange("2026-05-15T00:00".into(), "2026-05-15T23:59".into())))]
#[case("@lumi help",                            None)]
#[case("@lumi remember",                        None)]
#[case("@lumi remember the last messages",      None)] // missing N
#[case("just normal chat",                      None)]
fn ac1_parser_coverage(#[case] input: &str, #[case] expected: Option<RetroCmd>) {
    assert_eq!(parse_retro_command(input), expected);
}
```

### AC #4 — over-limit rejected

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac4_over_limit_rejected() {
    let env = TestEnv::new().await;
    let req = test_request_n(200);
    let err = handle_retro(req).await.unwrap_err();
    assert!(matches!(err, RetroError::OverLimit));
    let last_msg = env.mm.last_reply().await;
    assert!(last_msg.message.contains("Capture max is 100"));
    let m = metric_value("chat_retro_captures_total", &[("outcome", "over_limit")]);
    assert_eq!(m, 1);
}
```

### AC #15 — command post excluded

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac15_command_post_excluded() {
    let env = TestEnv::new().await;
    let mut ids = env.seed_channel_with_messages(10).await;
    let cmd_post = env.post_command("@lumi remember last 10 messages").await;
    handle_retro_request(cmd_post.clone()).await.unwrap();
    let picker = env.mm.last_ephemeral().await;
    let actions: Vec<String> = picker.props["attachments"][0]["actions"].as_array().unwrap()
        .iter().map(|a| a["integration"]["context"]["post_id"].as_str().unwrap().to_owned()).collect();
    assert!(!actions.contains(&cmd_post.id));
    assert_eq!(actions.len(), 10);
}
```

### AC #16 — system messages excluded

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac16_system_messages_excluded() {
    let env = TestEnv::new().await;
    env.seed_channel_with_mixed(8, 2).await; // 8 user + 2 system
    let cmd = env.post_command("@lumi remember last 10 messages").await;
    handle_retro_request(cmd).await.unwrap();
    let picker = env.mm.last_ephemeral().await;
    let actions = picker.props["attachments"][0]["actions"].as_array().unwrap();
    assert_eq!(actions.iter().filter(|a| a["id"].as_str().unwrap().starts_with("toggle-")).count(), 8);
}
```

### AC #17 — picker preview + relative time

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac17_picker_preview_and_relative_time() {
    let env = TestEnv::new().await;
    let msg = env.seed_message_with_body("Lorem ipsum dolor sit amet ".repeat(50)).await;
    env.post_command("@lumi remember last 1 messages").await;
    handle_retro_request_current(env.tenant_id()).await.unwrap();
    let picker = env.mm.last_ephemeral().await;
    let action_name = picker.props["attachments"][0]["actions"][0]["name"].as_str().unwrap();
    assert!(action_name.contains("Lorem ipsum"));
    assert!(action_name.len() < 250); // 200 body + ~50 prefix
    assert!(action_name.contains("ago"));
}
```

### AC #18 — bulk actions

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac18_select_all_action() {
    let env = TestEnv::new().await;
    let picker_id = env.start_picker_with_n(10).await;
    apply_action(&picker_id, "select-all").await.unwrap();
    let state = retro::state::fetch_state(&picker_id).await.unwrap().unwrap();
    assert_eq!(state.selected_ids.len(), 10);

    apply_action(&picker_id, "invert").await.unwrap();
    let state = retro::state::fetch_state(&picker_id).await.unwrap().unwrap();
    assert_eq!(state.selected_ids.len(), 0);

    apply_action(&picker_id, "select-none").await.unwrap();
    let state = retro::state::fetch_state(&picker_id).await.unwrap().unwrap();
    assert_eq!(state.selected_ids.len(), 0);
}
```

### AC #19 — _started audit

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac19_retro_started_audit() {
    let env = TestEnv::new().await;
    env.post_command("@lumi remember last 5 messages").await;
    handle_retro_request_current(env.tenant_id()).await.unwrap();
    let row = env.memory.last_of_kind("chat.retro_capture_started").await.unwrap();
    assert_eq!(row["payload"]["n_offered"], 5);
}
```

### AC #20 — picker TTL

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac20_picker_ttl_1h() {
    let env = TestEnv::new().await;
    let picker_id = env.start_picker_with_n(5).await;
    env.expire_picker_state(&picker_id, chrono::Duration::hours(2)).await;
    let result = post_picker_submit_for(&picker_id).await;
    assert!(matches!(result, Err(RetroError::Expired)));
    let row = env.memory.last_of_kind("chat.retro_capture_expired").await.unwrap();
    assert_eq!(row["payload"]["picker_post_id"], picker_id);
}
```

### AC #21 — context toggle

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac21_context_toggle_includes_neighbours() {
    let env = TestEnv::new().await;
    let ids = env.seed_channel_with_messages(5).await;
    let picker_id = env.start_picker_capturing(&ids).await;
    apply_action(&picker_id, &format!("toggle-{}", ids[2])).await.unwrap();
    apply_action(&picker_id, &format!("context-{}", ids[2])).await.unwrap();
    apply_action(&picker_id, "commit").await.unwrap();
    let mem = env.memory.latest_memory_in(format!("memories/chat/{}/captures/", env.channel_id)).await.unwrap();
    assert!(mem.body.contains(&format!("[context] ")));
    let mention_count = mem.body.matches("@").count();
    assert!(mention_count >= 3); // msg2, msg3, msg4 all included
}
```

### AC #22 — dedup against prior

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac22_dedup_against_prior_24h() {
    let env = TestEnv::new().await;
    let ids = env.seed_channel_with_messages(3).await;
    capture_set(&env, &ids).await.unwrap();
    let prior_path = env.latest_memory_path().await;
    // Immediate re-capture same set
    let resp = capture_set(&env, &ids).await;
    let last = env.mm.last_reply().await;
    assert!(last.message.contains("Already captured"));
    assert!(last.message.contains(&prior_path));
}
```

### AC #23 — date-range form

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac23_date_range_form() {
    let env = TestEnv::new().await;
    env.seed_channel_at_date("2026-05-15", 50).await;
    env.seed_channel_at_date("2026-05-16", 50).await;
    env.post_command("@lumi capture from 2026-05-15T00:00 to 2026-05-15T23:59").await;
    handle_retro_request_current(env.tenant_id()).await.unwrap();
    let picker = env.mm.last_ephemeral().await;
    let actions = picker.props["attachments"][0]["actions"].as_array().unwrap();
    let toggle_count = actions.iter().filter(|a| a["id"].as_str().unwrap().starts_with("toggle-")).count();
    assert_eq!(toggle_count, 50);
}
```

### AC #25 — ACL includes participants

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac25_memory_acl_has_all_participants() {
    let env = TestEnv::new().await;
    let alice_msg = env.seed_message_by("Alice").await;
    let bob_msg   = env.seed_message_by("Bob").await;
    let carol_msg = env.seed_message_by("Carol").await;
    let picker_id = env.start_picker_capturing(&[alice_msg, bob_msg, carol_msg]).await;
    apply_action(&picker_id, "select-all").await.unwrap();
    apply_action(&picker_id, "commit").await.unwrap();
    let mem = env.memory.latest_memory_in("memories/chat/").await.unwrap();
    let acl = mem.frontmatter["meta"]["acl"].as_array().unwrap();
    let acl_set: HashSet<_> = acl.iter().map(|v| v.as_str().unwrap()).collect();
    assert!(acl_set.contains("Alice"));
    assert!(acl_set.contains("Bob"));
    assert!(acl_set.contains("Carol"));
}
```

### AC #26 — split at 1MB

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac26_memory_split_at_1mb() {
    let huge_body = "x".repeat(5_000_000);
    let (parts, index) = split_if_oversized(&huge_body);
    assert!(parts.len() >= 5);
    assert!(index.is_some());
    for p in &parts { assert!(p.len() <= 1_048_576); }
    let index_md = index.unwrap();
    for i in 1..=parts.len() {
        assert!(index_md.contains(&format!("part-{}.md", i)));
    }
}
```

### AC #27 — --dry-run

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac27_dry_run_disables_commit() {
    let env = TestEnv::new().await;
    env.post_command("@lumi remember last 5 messages --dry-run").await;
    handle_retro_request_current(env.tenant_id()).await.unwrap();
    let picker = env.mm.last_ephemeral().await;
    let actions = picker.props["attachments"][0]["actions"].as_array().unwrap();
    let has_commit_disabled = actions.iter()
        .any(|a| a["id"].as_str().unwrap() == "commit-disabled");
    assert!(has_commit_disabled);
    let has_real_commit = actions.iter()
        .any(|a| a["id"].as_str().unwrap() == "commit");
    assert!(!has_real_commit);
}
```

### Compose markdown determinism

```rust
proptest! {
    #[test]
    fn compose_markdown_deterministic(msgs in prop::collection::vec(any_message_strategy(), 1..50)) {
        let a = compose_markdown(&msgs);
        let b = compose_markdown(&msgs);
        prop_assert_eq!(a, b);
    }
}
```

---

## §6 — Implementation skeleton

The Rust modules above are the skeleton. Operational wiring:

### §6.1 — Webhook endpoints

The chat-lumi service exposes 5 retro endpoints in addition to its main `/webhook`:

| Endpoint | Method | Purpose |
|---|---|---|
| `/retro/toggle`      | POST | Toggle per-message selection |
| `/retro/select-all`  | POST | Set all selected |
| `/retro/select-none` | POST | Clear all selected |
| `/retro/invert`      | POST | Invert selection |
| `/retro/commit`      | POST | Validate dedup + write memory + emit audit |
| `/retro/cancel`      | POST | Discard state + emit cancellation audit |
| `/retro/context-toggle/:id` | POST | Toggle per-message context inclusion |

Each endpoint receives the MM interactive-message webhook payload, loads picker state from Redis, mutates, persists, and re-renders the ephemeral message.

### §6.2 — Memory path canonical form

`memories/chat/<channel_id>/captures/<unix_ts>.md` — the unix timestamp ensures temporal ordering and uniqueness. Sub-second collisions append a random 4-char suffix: `<unix_ts>-<rand>.md`.

### §6.3 — Frontmatter schema (canonical)

```yaml
---
kind: chat_capture
sync_class: shareable|private
captured_by: <subject_id>
captured_at: <iso-8601>
source_channel_id: <id>
source_message_ids:
  - <id1>
  - <id2>
message_count: 12
meta:
  acl:
    - <subject_or_display>
    - <subject_or_display>
trace_id: <32-hex>
memory_chain_hash: <to-be-filled-on-commit>
---
```

### §6.4 — Context-toggle resolution

When the user toggles "include context" for message M_i, the aggregator fetches M_{i-1} and M_{i+1} from MM API at commit time (not picker time). Reason: messages may have arrived between picker open and commit; we want the freshest context.

### §6.5 — Dedup-window choice

24 hours is calibrated against operator hesitation: an operator who returns to re-capture after a day's gap is most likely starting a deliberate re-capture (intent change), not unintentional duplication. Within 24h, prompt for confirmation.

### §6.6 — Date-range query bound

`@lumi capture from <a> to <b>` enforces N ≤ 100 still: if the range contains > 100 messages, the picker shows the first 100 with a "showing first 100 of N" note and `chat.retro_capture_truncated` audit row.

### §6.7 — Memory split orchestration

When `split_if_oversized` returns > 1 part:
1. Write each part to `<ts>-part-<i>.md` with full frontmatter (each part is independently readable).
2. Write the index `<ts>-index.md` with frontmatter `{kind: chat_capture_index, parts: [...]}`.
3. The `chat.retro_capture_completed` audit row references the index path; downstream consumers follow links to parts.

### §6.8 — Picker state Redis schema

Key: `retro:picker:<picker_post_id>`
Value: msgpack-serialised `PickerState`
TTL: 3600s (1h, per §1 #16)

We use msgpack rather than JSON for compactness; pickers with 100 messages × 200-char preview can approach 50KB per state record.

### §6.9 — Trace_id end-to-end

| Hop | Source of trace_id |
|---|---|
| Command post | Set by Mattermost or inbound traceparent |
| Picker post props | Carried from command post |
| Toggle/commit webhooks | Carried in picker_post props |
| Memory frontmatter | Carried in PickerState |
| 3 audit rows | Each carries the same trace_id |

### §6.10 — Failure routing matrix

| Failure | Audit | User-visible |
|---|---|---|
| over_limit | (none) | reply: "Capture max is 100" |
| empty selection | chat.retro_capture_cancelled | reply: "Nothing selected" |
| picker expired | chat.retro_capture_expired | reply: "Picker expired; re-issue command" |
| commit + dedup hit | (pending — awaits confirmation) | reply: "Already captured; re-capture?" |
| memory write fail | chat.retro_capture_failed (SEV-2) | reply: "Capture failed; please retry" |
| MM ephemeral post fail | logged | falls back to DM via @lumi |

---

## §7 — Dependencies

- **FR-CHAT-005** — bridge picks up the picker request.
- **FR-CHAT-008** — @lumi mention dispatch.
- **FR-MEMORY-111** — redaction.
- **FR-MEMORY-101** — MemoryWriter.

---

## §8 — Example payloads

### `chat.retro_capture_started`

```json
{
  "kind": "chat.retro_capture_started",
  "ts_ns": 1747407100000000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "trace_id": "0af7651916cd43dd8448eb211c80319c",
  "payload": {
    "channel_id":       "01HVQX8ZG2K3R4TVA7P3WV5X8M",
    "requested_by":     "01HVQX8ZG2K3R4TVA7P3WV5X8K",
    "n_offered":        20,
    "command_post_id":  "01HVQX8ZG2K3R4TVA7P3WV5X8N",
    "picker_post_id":   "01HVQX8ZG2K3R4TVA7P3WV5X8P",
    "dry_run":          false
  }
}
```

### `chat.retro_capture_completed`

```json
{
  "kind": "chat.retro_capture_completed",
  "ts_ns": 1747407137000000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "trace_id": "0af7651916cd43dd8448eb211c80319c",
  "payload": {
    "channel_id":              "01HVQX8ZG2K3R4TVA7P3WV5X8M",
    "captured_by":             "01HVQX8ZG2K3R4TVA7P3WV5X8K",
    "message_count_requested": 20,
    "message_count_selected":  12,
    "memory_path":             "memories/chat/01HVQX8ZG2K3R4TVA7P3WV5X8M/captures/1747407137.md",
    "sync_class":              "private",
    "participants_acl":        ["alice", "bob", "carol"],
    "split_into_parts":        1
  }
}
```

### `chat.retro_capture_cancelled`

```json
{
  "kind": "chat.retro_capture_cancelled",
  "ts_ns": 1747407150000000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "payload": {
    "channel_id":     "01HVQX8ZG2K3R4TVA7P3WV5X8M",
    "by":             "01HVQX8ZG2K3R4TVA7P3WV5X8K",
    "picker_post_id": "01HVQX8ZG2K3R4TVA7P3WV5X8P",
    "reason":         "user_clicked_cancel"
  }
}
```

### `chat.retro_capture_expired`

```json
{
  "kind": "chat.retro_capture_expired",
  "ts_ns": 1747410800000000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "payload": {
    "picker_post_id":   "01HVQX8ZG2K3R4TVA7P3WV5X8P",
    "ttl_seconds":      3600,
    "elapsed_seconds":  3625
  }
}
```

### Memory frontmatter (canonical capture)

```yaml
---
kind: chat_capture
sync_class: private
captured_by: 01HVQX8ZG2K3R4TVA7P3WV5X8K
captured_at: 2026-05-16T14:32:17Z
source_channel_id: 01HVQX8ZG2K3R4TVA7P3WV5X8M
source_message_ids:
  - 01HVQX8ZG2K3R4TVA7P3WV5X8A
  - 01HVQX8ZG2K3R4TVA7P3WV5X8B
  - 01HVQX8ZG2K3R4TVA7P3WV5X8C
message_count: 3
meta:
  acl:
    - alice
    - bob
    - carol
trace_id: 0af7651916cd43dd8448eb211c80319c
---

[2026-05-16T14:31 @alice] Let me know if we should escalate <NAME> to <ROLE> next week.
[2026-05-16T14:31 @bob] +1, I'll draft the announcement.
[2026-05-16T14:32 @carol] LGTM — let's loop in <EMAIL> when ready.
```

### Memory index (split mode)

```yaml
---
kind: chat_capture_index
sync_class: shareable
captured_by: 01HVQX8ZG2K3R4TVA7P3WV5X8K
captured_at: 2026-05-16T14:32:17Z
source_channel_id: 01HVQX8ZG2K3R4TVA7P3WV5X8M
total_messages: 487
parts:
  - memories/chat/01HVQX8ZG2K3R4TVA7P3WV5X8M/captures/1747407137-part-1.md
  - memories/chat/01HVQX8ZG2K3R4TVA7P3WV5X8M/captures/1747407137-part-2.md
  - memories/chat/01HVQX8ZG2K3R4TVA7P3WV5X8M/captures/1747407137-part-3.md
trace_id: 0af7651916cd43dd8448eb211c80319c
---

# Capture index

This capture was split into 3 parts due to size.

- [Part 1](./part-1.md)
- [Part 2](./part-2.md)
- [Part 3](./part-3.md)
```

### MM ephemeral picker (rendered)

```
[ephemeral, visible to alice only]

Select messages to capture
Showing the last 10 messages. Toggle each to include in the memory capture.

[ alice · 3m ago · Let me know if we should escalate...]
[ bob   · 3m ago · +1, I'll draft the announcement.]
[ carol · 2m ago · LGTM — let's loop in alice@... when ready.]
...
[Select all] [Select none] [Invert] [Capture selected] [Cancel]
```

---

## §9 — Open questions

All resolved. Deferred:
- Date-range capture (not just last-N) — slice 4+.
- Auto-categorisation of captures by topic — slice 4+.
- Multi-channel capture (join 2 channels in one memory) — slice 5+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| N > 100 in command | parse_retro_command + bounds | reply "Capture max is 100"; metric outcome=over_limit | None |
| N == 0 | bounds check | reply "Capture needs N ≥ 1" | None |
| N is malformed (non-numeric) | regex fails | reply "Couldn't parse: `@lumi remember last 10 messages`"; help link | None |
| Channel deleted mid-picker | mm_api::fetch_last_n 404 | reply error; chat.retro_capture_failed (SEV-3) | None |
| User has no read on selected messages | RLS Err | filter to readable only; warn user about filtered count | None |
| memory put_memory fails | memory_writer err | SEV-2 chat.retro_capture_failed; reply "Capture failed; please retry" | Operator restores memory |
| PII scan crash | catch_unwind | chat.retro_capture_failed; reply error | Operator fixes ruleset |
| PII scan returns empty (over-redaction) | empty body check | proceed but warn user "Many messages were redacted; review the memory" | None |
| MM ephemeral msg post fails | API Err | fall back to DM via @lumi private channel; warn user | None |
| Picker submit after TTL | state lookup miss | reply "Picker expired"; chat.retro_capture_expired audit | None |
| Concurrent picker by same user | independent picker_post_ids; each in own Redis key | each independent | None |
| Memory path collision (same second) | append random 4-char suffix | safe | None |
| User cancels via "Cancel" button | submit handler | chat.retro_capture_cancelled; no memory write | None |
| User cancels by closing MM tab | TTL eventually expires | chat.retro_capture_expired | None |
| Aggregation > 1MB | split_if_oversized | parts + index file; chat.retro_capture_completed.split_into_parts > 1 | None |
| Aggregation > 100MB (theoretical) | refuse | reply "Aggregated body too large"; chat.retro_capture_failed | Operator |
| Context fetch fails (one neighbour) | log + omit that context | reduced context | None |
| Context fetch fails (both neighbours) | log + skip context for that msg | reduced context | None |
| Dedup window 24h check returns false negative (sets differ by 1 ID) | proceeds | two captures with 99% overlap | Operator manual review |
| Dedup confirmation rejected | abort | chat.retro_capture_cancelled (reason=dedup) | None |
| Date-range parse fails | parser returns None | reply "Couldn't parse date range" + help | None |
| Date-range > 100 messages | truncate to first 100 | chat.retro_capture_truncated; reply with note | Operator narrows range |
| Date-range with future end-date | proceed; will return empty if no messages | None | None |
| Date-range with start > end | reply "Invalid range" | None | None |
| Trace_id missing in command post | mint new | trace continuity broken | None |
| Memory frontmatter parse fails (yaml lib bug) | sanitised at write | chat.retro_capture_failed | Operator |
| ACL contains unresolvable subject_id | best-effort: include raw id | downstream consumer sees ID, not name | None |
| User's tenant has lumi_enabled=false | already blocked at FR-CHAT-008 | no retro reachable | None |
| User triggers --dry-run then clicks Commit | button disabled; no-op | None | None |
| User triggers --dry-run then waits 1h then triggers real | each picker independent | both complete normally | None |
| Mattermost interactive message limits (Mattermost has 20 actions/post) | split picker into multiple posts | rare; for N>20 messages | None |
| Picker shows N actions; user has scrolled away | MM persists ephemeral msg until refresh | works on next view | None |
| Two users issue retro in same channel | each picker is per-user-ephemeral | independent | None |
| User issues retro then changes MM password (logout/relogin) | session change | ephemeral msg disappears; user re-issues | None |
| Memory write succeeds but memory audit emit fails | logged; partial state | manual operator catch | Operator backfills audit |
| Captured message contains binary attachment ref | only text body included | attachment not in memory; warn in memory body | None |
| Captured message is a deleted post (post.delete_at > 0) | exclude from picker | None | None |
| Captured message is from an imported set | include normally (it's still a real post) | None | None |
| Lumi reply about capture itself contains @lumi | filtered by FR-CHAT-008 self-skip | None | None |
| Captured message contains a Lumi reply | included; appears in memory as @Lumi | None | None |
| Network error mid-commit | tx aborted in memory writer | chat.retro_capture_failed | Retry via re-issue |
| Redis state Redis cluster failover | brief blip; commit retries via secondary | None | None |
| TTL cleanup runs while user is mid-toggle | TTL pre-checked at each action | actions return Expired error | None |
| Memory split + last part empty | split_if_oversized handles | parts.last() empty body OK | None |
| Picker has duplicate message IDs (Mattermost bug) | dedup in state before persist | None | None |
| Pic ker submit POST replayed (network retry) | idempotency via picker_post_id + state check | second is no-op | None |

---

## §11 — Implementation notes

- Mattermost ephemeral messages are non-persisted; visible only to target user. They survive page reloads (cached by client) but are discarded server-side after the user's session ends. That's why we keep picker state in Redis, not in the ephemeral post.
- Checkbox state via Mattermost interactive message attachments (actions field). MM's `actions` are buttons, not native checkboxes — we re-render the ephemeral message with updated button styles ("✓ selected" vs "select") to simulate checkbox UX.
- 1h TTL via Redis EXPIRE; expired pickers naturally vanish. We don't need a cleanup job because Redis handles it.
- Memory path uses unix timestamp for filename; collision-safe at second resolution + random 4-char suffix on collision. Timestamps in human-readable form would risk locale issues.
- Reply uses MM REST API (not plugin context) for cross-process visibility — chat-lumi is a separate process from MM.
- 200-character preview was chosen because Mattermost button labels truncate around 300 chars; 200 + username prefix fits comfortably.
- Relative-time formatting ("3m ago") was chosen over absolute timestamps because picker UX values quick scanning; users can hover for absolute time if needed (slice-4+ enhancement).
- Bulk actions (select-all/none/invert) reduce common-case clicks from N to 1; calibrated against operator usage patterns (most retros want most messages with 1-2 exclusions).
- The "include surrounding context" toggle is per-message rather than a global flag because operators often want context for SOME messages (the ambiguous ones) but not all.
- Dedup-window choice (24h) balances false-positive friction (annoy operator with confirmation) against false-negative cost (silent duplicate memory). 24h captures the "deliberate re-capture" case.
- Date-range parsing accepts ISO-8601 with or without seconds; we don't accept relative forms ("yesterday") in MVP — operators can always compute exact dates.
- Memory split-at-1MB threshold was chosen for memory indexer performance; below 1MB the indexer can keep memories fully in memory; above 1MB it falls back to streaming.
- We chose plain-text concat with `[timestamp @username] body\n` format over JSON because: (a) markdown rendering is universal in memory consumers; (b) the format is human-readable for direct review; (c) trivial to re-parse if needed.
- The `[context]` marker prefix on adjacent messages makes the memory self-documenting; future readers see "this was included as context, not selected by the user."
- ACL containing all participant display names (not just subject_ids) was chosen because memory's downstream consumers (sharing UI) need names for human-readable display; subject_ids are unhelpful to operators.
- We didn't use Mattermost's built-in "Save Post" feature because: (a) saves are per-user, not channel-shared; (b) memory integration would still need to scrape saved posts; (c) the retro flow is a deliberate aggregation, not an arbitrary save.
- The picker's "Cancel" button explicitly emits a cancellation audit row so operators investigating "what happened to that capture" see a deterministic record, not just absence.
- TTL refresh on user interaction: when user clicks a button, we extend the Redis TTL by another hour. Prevents pickers from expiring while the user is actively engaging.
- The capture flow is deliberately SLOWER than a single click would be (multi-step: command → picker → toggle → commit). This is intentional: captures are durable memory writes; we want operator deliberation, not impulse.
- We considered a "quick capture" command (`@lumi remember this message` with no picker) for a one-message case but rejected it: it would create inconsistent UX (sometimes picker, sometimes not). Always-picker is simpler.
- Why we exclude system messages: they're noise. Mattermost emits dozens of them per active channel ("Alice joined", "Channel renamed"); operators capturing don't want them.
- The dedup mechanism uses set equality on selected_ids, not just intersection — partial overlaps don't trigger confirmation. This balances false-positive friction against the realistic case (exact-set re-capture).
- We considered shipping a "diff against prior capture" view in the picker but rejected as too much UX surface for slice-2; future enhancement.
- The picker's interactive buttons survive MM's WebSocket disconnect/reconnect via Redis state; user can disconnect, reconnect, and continue selection.
- `--dry-run` shows the picker but disables commit; this is a deliberate friction-checker for high-stakes captures (e.g. legal-channel retro). Operators see what would be captured before committing.
- Memory ACL: even for `sync_class: shareable` captures, we include the participant ACL because shareable memories may be exported, and tracking who contributed is a compliance / consent concern.
- `chat.retro_capture_started` and `chat.retro_capture_completed` form a closed pair — operators can spot dropped lifecycles (started without completed within 1h TTL → user abandoned).
- The 100-message cap is enforced even in date-range mode because the picker UX degrades past 100 actions; we'd rather truncate with a clear notice than ship a broken picker.

---

*End of FR-CHAT-009.*
