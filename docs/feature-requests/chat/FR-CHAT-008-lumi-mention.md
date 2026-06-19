---
id: FR-CHAT-008
title: "@lumi mention parser — message mentions trigger CUO routing + memory capture row + reply"
module: CHAT
priority: MUST
status: done
verify: T
phase: P1
milestone: P1 · slice 2
slice: 2
owner: Stephen Cheng
created: 2026-05-16
shipped: null
memory_chain_hash: null
related_frs: [FR-CHAT-005, FR-CHAT-009, FR-AI-014, FR-CUO-101, FR-MEMORY-101]
depends_on: [FR-CHAT-005]
blocks: [FR-CHAT-009]

source_pages:
  - website/docs/modules/chat.html#lumi
source_decisions:
  - DEC-490 (@lumi is the canonical assistant mention; case-insensitive; works in channel + DM)
  - DEC-491 (mention extracted from message body via regex; routed to FR-CUO-101 supervisor)
  - DEC-492 (CUO emits memory capture row 'chat.lumi_invoked' AND posts reply back into chat)

language: rust 1.81 + go (Mattermost plugin)
service: cyberos/services/chat-lumi/
new_files:
  - services/chat-lumi/Cargo.toml
  - services/chat-lumi/src/main.rs
  - services/chat-lumi/src/parser.rs
  - services/chat-lumi/src/cuo_route.rs
  - services/chat-lumi/src/reply.rs
  - services/chat-lumi/tests/lumi_test.rs
modified_files:
  - services/chat/plugins/cyberos-lumi-router/main.go    # Mattermost plugin intercepts msg posts
allowed_tools:
  - file_read: services/chat-lumi/**, services/chat/**
  - file_write: services/chat-lumi/{src,tests}/**, services/chat/plugins/cyberos-lumi-router/**
  - bash: cd services/chat-lumi && cargo test
disallowed_tools:
  - send LLM call from chat process (per §1 — Lumi service handles)
  - reply without memory audit (per DEC-492)

effort_hours: 6
sub_tasks:
  - "0.5h: parser.rs — regex (?i)@lumi(?:[\\s,!?:]|$); extract surrounding context"
  - "1.0h: Mattermost plugin (Go) — on message create, if matches → POST to chat-lumi service"
  - "1.0h: chat-lumi service — receives webhook; routes to CUO supervisor (FR-CUO-101)"
  - "1.0h: cuo_route.rs — CUO call with persona=lumi (slice-3 stub: returns 'I'll get back to you')"
  - "0.5h: reply.rs — post CUO response back into chat channel via MM API"
  - "0.5h: memory audit row 'chat.lumi_invoked' with body_redacted + cuo_response_hash"
  - "1.0h: lumi_test.rs — regex coverage + routing happy path"
  - "0.5h: latency budget: < 2s to first response (CUO ack)"
risk_if_skipped: "Lumi (the assistant) is the brand surface for AI-augmented chat. Without mention parser, users have no in-chat invocation. Without CUO routing, every mention burns AI Gateway directly = cost. Without reply, mentioning lumi feels broken. Without audit, conversations with lumi invisible to ops."
---

## §1 — Description (BCP-14 normative)

The @lumi mention layer **MUST** intercept chat messages, parse mentions, route to CUO supervisor, and post reply. The contract:

1. **MUST** install a Mattermost plugin `cyberos-lumi-router` that intercepts every new post:
   - On post.create: parse body via `(?i)@lumi(?:[\\s,!?:]|$)` regex.
   - On match: extract surrounding context (4 sentences before + after) → send to chat-lumi service.
2. **MUST** the chat-lumi service (separate process) receives webhook with `{post_id, channel_id, user_id, body, context, tenant_id, trace_id}`.
3. **MUST** route to CUO supervisor (FR-CUO-101) with persona = `lumi`. CUO returns either:
   - `Resolved { reply: String }` — immediate reply.
   - `Routed { eta_seconds: i32 }` — async work; respond later.
   - `Refused { reason: String }` — policy or capability gap.
4. **MUST** post reply to chat channel via Mattermost API:
   - Reply as user "Lumi" (system user provisioned at install).
   - Reply threaded to the mention post (`root_id` set).
   - Include CUO outcome footer: `_(via Lumi · trace ...)`.
5. **MUST** emit memory audit `chat.lumi_invoked` per mention with payload `{post_id, channel_id, user_id, body_redacted, context_hash, cuo_outcome, response_hash, latency_ms, trace_id}`.
6. **MUST** PII-redact body before memory write (FR-MEMORY-111).
7. **MUST** dedup: same `(post_id)` mentioned multiple times by edits → only first mention triggers.
8. **MUST** respect channel privacy: in private channel, reply has `sync_class: private` per FR-MEMORY-106; public → shareable.
9. **MUST** complete first response (CUO ack OR Resolved) within 2 seconds; longer responses post "thinking..." placeholder then update.
10. **MUST** emit OTel metrics:
    - `chat_lumi_mentions_total{outcome}` (outcome ∈ resolved | routed | refused | error).
    - `chat_lumi_first_response_latency_seconds`.
11. **MUST NOT** trigger on imported posts: posts whose `props.cyberos_imported = true` (from FR-CHAT-006/007) MUST be skipped, even if their body contains `@lumi`. Historical mentions must not generate live LLM calls.
12. **MUST NOT** trigger on Lumi's own replies: posts authored by the Lumi system user are filtered before regex eval. Prevents infinite reply loops.
13. **MUST** rate-limit per-user mentions: 30 mentions per user per minute, sliding window. Excess returns Refused with `reason: "rate_limited"`. Counters live in Redis.
14. **MUST** support per-tenant Lumi enable/disable: when `cyberos_chat_tenant_settings.lumi_enabled = false`, mentions are detected but produce no CUO call AND no reply AND emit memory audit `chat.lumi_skipped` with `reason: "disabled"`.
15. **MUST** handle long CUO responses with a "thinking" placeholder that's updated when CUO resolves: if CUO ack does not arrive in 2s, post `_(Lumi is thinking…)_` as the threaded reply; when CUO resolves later (≤ 60s), edit the placeholder to the actual reply via MM API edit.
16. **MUST** preserve trace_id across MM plugin → chat-lumi service → CUO supervisor → MM reply: every hop carries the W3C traceparent header; memory audit row records the same trace_id.
17. **MUST** include channel membership of the mentioner in CUO context: CUO receives `{user_channels_count, user_is_admin, channel_member_count}` so personas can adapt (e.g. admin-only commands).
18. **MUST** support `@lumi help` as a built-in command that responds with the persona's capability list WITHOUT consuming an LLM call. Hard-coded reply rendered from a static template; sub-100ms latency.
19. **MUST** support `@lumi cancel` to cancel an in-flight Routed-mode CUO job initiated by the same user in the same channel: looks up the most recent unresolved CUO job, marks cancelled, posts "Cancelled." threaded.
20. **MUST** support multiple @lumi mentions in a single message body as a single CUO invocation (de-duped); the memory audit records `mention_count` in payload.
21. **MUST** preserve message author context in memory payload: not just `user_id` but `{user_id, display_name_redacted, role, joined_at}` so downstream consumers can answer "who asked Lumi for X" without an extra MM API call.
22. **MUST** redact CUO response before memory audit emit, separately from request redaction. The redaction ruleset is the same FR-MEMORY-111 ruleset; responses can leak data the model echoed from the prompt.
23. **MUST** emit memory audit `chat.lumi_error` for any failure path (CUO timeout, MM API failure, redaction crash) with `{post_id, channel_id, user_id, error_class, error_message_redacted, trace_id}`. SEV-2 if rate > 1/min.
24. **MUST** require `cyberos_chat_tenant_settings.lumi_enabled` to be `true` AND a feature-flag check (`tenant_features.lumi = true`) before processing. Both must be true; either false → skip. This is the two-key-launch pattern used elsewhere in the platform.
25. **MUST** record the CUO budget consumed per invocation in memory payload (`budget_tokens_used`, `budget_dollars_estimate`); FR-AI-014 surfaces these for billing.

---

## §2 — Why this design

**Why plugin intercept (§1 #1)?** Webhook-from-Mattermost has too much lag (event → webhook → service). Plugin runs in-process; instant parse.

**Why CUO supervisor (DEC-491)?** Lumi is the chat-facing persona; CUO is the underlying orchestration. Routing through CUO = consistent policy + budget + persona logic across surfaces.

**Why threaded reply (§1 #4)?** Inline conversations get lost in busy channels; threading keeps Lumi exchanges discoverable.

**Why 2s budget (§1 #9)?** Beyond 2s users assume broken. "Thinking..." placeholder buys time for longer LLM calls.

**Why redaction before memory (§1 #6)?** Users may @lumi with sensitive info; FR-MEMORY-111 ruleset scrubs.

**Why skip imported posts (§1 #11)?** Importing 100k historical Slack/Zalo messages would trigger 100k LLM calls if Lumi processed historical mentions — $$$ and wrong (the original conversation already happened; replying now is anachronistic). The `cyberos_imported` props marker from FR-CHAT-006/007 carries through; we check it explicitly.

**Why filter Lumi's own replies (§1 #12)?** Without this, a Lumi reply that happens to mention `@lumi` in quoted text (e.g. "you asked '@lumi help'") would re-trigger. Identity check via `post.user_id == lumi_system_user_id` is O(1).

**Why rate-limit per user (§1 #13)?** Prevents a single user from monopolising CUO budget. 30/min is calibrated against realistic human typing speeds; bots / automation hit the limit fast.

**Why Lumi enable/disable per tenant (§1 #14)?** Some tenants opt out of LLM features for compliance or cost reasons. A tenant-level setting respects that choice.

**Why placeholder for long responses (§1 #15)?** UX expectation in chat is sub-2-second response. Long LLM calls would silently look broken; the placeholder + edit pattern keeps the user informed.

**Why preserve trace_id across hops (§1 #16)?** Distributed tracing is the only way to debug a "Lumi gave the wrong answer" complaint that spans MM plugin + Rust service + AI Gateway + back to MM.

**Why include channel context (§1 #17)?** A persona that knows "this is the #legal channel and only admins are in it" can adapt its safety stance (allow more detail) vs "this is #general open to all employees" (more conservative).

**Why hard-coded `@lumi help` (§1 #18)?** Help requests are predictable; serving them from a template avoids LLM cost AND latency. Also useful when LLM service is degraded — help still works.

**Why `@lumi cancel` (§1 #19)?** Users initiating a long-running Routed job may change their mind. Without cancel, they wait for a response they don't want, paying LLM budget.

**Why CUO budget in audit (§1 #25)?** Per-tenant billing for AI usage requires per-invocation cost capture. FR-AI-014 aggregates these into invoice line items.

---

## §3 — API contract

```go
// services/chat/plugins/cyberos-lumi-router/main.go
func (p *LumiRouter) MessageHasBeenPosted(c *plugin.Context, post *model.Post) {
    if !mentionsLumi(post.Message) { return }
    if p.alreadyRouted(post.Id) { return }   // dedup

    ctx := extractContext(post, 4)   // 4 sentences each side
    payload := map[string]interface{}{
        "post_id":    post.Id,
        "channel_id": post.ChannelId,
        "user_id":    post.UserId,
        "body":       post.Message,
        "context":    ctx,
        "tenant_id":  p.tenantID(post.ChannelId),
        "trace_id":   p.traceID(),
    }
    go p.postToLumiService(payload)
    p.markRouted(post.Id)
}

func mentionsLumi(body string) bool {
    rx := regexp.MustCompile(`(?i)@lumi(?:[\s,!?:]|$)`)
    return rx.MatchString(body)
}
```

```rust
// services/chat-lumi/src/main.rs
async fn handle_webhook(req: WebhookReq) -> Result<(), LumiError> {
    let start = Instant::now();

    // §1 #11: skip imported posts.
    if req.is_imported { return Ok(()); }

    // §1 #12: skip Lumi's own posts (defense in depth; plugin filters too).
    if req.user_id == lumi_system_user_id() { return Ok(()); }

    // §1 #14 + #24: tenant-level enable + feature-flag check.
    if !tenant_settings::lumi_enabled(req.tenant_id).await? {
        emit_memory_row("chat.lumi_skipped", serde_json::json!({
            "post_id": req.post_id, "reason": "disabled",
        })).await;
        return Ok(());
    }
    if !feature_flag::is_enabled("lumi", req.tenant_id).await? {
        return Ok(());
    }

    // §1 #13: rate-limit per user.
    if !rate_limiter::check(req.tenant_id, req.user_id, 30, Duration::from_secs(60)).await? {
        reply::post_threaded(&req.channel_id, &req.post_id,
            "Rate limit hit (30 mentions/min). Try again shortly.",
            &req.trace_id).await?;
        return Ok(());
    }

    // §1 #18: built-in commands short-circuit.
    if let Some(cmd) = parse_builtin_cmd(&req.body) {
        let resp = handle_builtin_cmd(cmd, &req).await?;
        reply::post_threaded(&req.channel_id, &req.post_id, &resp, &req.trace_id).await?;
        emit_audit(&req, "builtin", &resp, start.elapsed(), None).await;
        return Ok(());
    }

    let body_redacted = pii::scan_and_redact(&req.body, &[]).await?.redacted_body;

    // §1 #17: enrich context with channel membership.
    let channel_ctx = mm_client::channel_member_summary(&req.channel_id, &req.user_id).await?;

    // §1 #15: placeholder if no ack in 2s.
    let placeholder_handle = tokio::spawn({
        let req = req.clone();
        async move {
            tokio::time::sleep(Duration::from_secs(2)).await;
            reply::post_threaded(&req.channel_id, &req.post_id,
                "_(Lumi is thinking…)_", &req.trace_id).await.ok()
        }
    });

    let cuo_outcome = cuo::route(CuoRequest {
        persona: "lumi".into(),
        body: body_redacted.clone(),
        context: req.context.clone(),
        channel_context: channel_ctx,
        tenant_id: req.tenant_id,
        trace_id: req.trace_id.clone(),
        timeout: Duration::from_secs(60),
    }).await;

    let (reply_text, error_class) = match &cuo_outcome {
        Ok(CuoOutcome::Resolved { reply, budget }) => (reply.clone(), None),
        Ok(CuoOutcome::Routed { eta_seconds, job_id, budget }) => {
            (format!("I'll get back to you in ~{}s. (cancel with `@lumi cancel`)", eta_seconds), None)
        }
        Ok(CuoOutcome::Refused { reason }) => (format!("I can't help with that: {}", reason), None),
        Err(e) => (
            "I'm having trouble; please try again in a moment.".into(),
            Some(e.class_label()),
        ),
    };

    // §1 #15: if placeholder fired, edit it; else post fresh.
    let placeholder_post_id = placeholder_handle.await.ok().flatten();
    match placeholder_post_id {
        Some(pid) => reply::edit(&pid, &reply_text, &req.trace_id).await?,
        None      => reply::post_threaded(&req.channel_id, &req.post_id, &reply_text, &req.trace_id).await?,
    };

    // §1 #22: redact response separately before audit.
    let reply_redacted = pii::scan_and_redact(&reply_text, &[]).await?.redacted_body;

    let latency = start.elapsed();
    emit_memory_row("chat.lumi_invoked", serde_json::json!({
        "post_id":       req.post_id,
        "channel_id":    req.channel_id,
        "user_id":       req.user_id,
        "user_display":  redact_email(&req.user_display_name),
        "user_role":     req.user_role,
        "body_redacted": body_redacted,
        "context_hash":  sha256_concat(&req.context),
        "cuo_outcome":   outcome_kind(&cuo_outcome),
        "response_redacted": reply_redacted,
        "response_hash": sha256(&reply_text),
        "mention_count": count_mentions(&req.body),
        "channel_member_count": channel_ctx.member_count,
        "budget_tokens_used":  budget_tokens(&cuo_outcome),
        "budget_dollars_estimate": budget_dollars(&cuo_outcome),
        "latency_ms":    latency.as_millis() as i64,
        "trace_id":      req.trace_id,
    })).await;

    if let Some(cls) = error_class {
        emit_memory_row("chat.lumi_error", serde_json::json!({
            "post_id": req.post_id, "error_class": cls,
            "trace_id": req.trace_id,
        })).await;
    }

    metrics::counter!("chat_lumi_mentions_total",
        "outcome" => outcome_kind(&cuo_outcome).to_string()).increment(1);
    metrics::histogram!("chat_lumi_first_response_latency_seconds")
        .record(latency.as_secs_f64());
    Ok(())
}
```

### parser.rs — regex + builtin commands + mention counter

```rust
// services/chat-lumi/src/parser.rs
use regex::Regex;
use once_cell::sync::Lazy;

static MENTION_RE: Lazy<Regex> = Lazy::new(||
    Regex::new(r"(?i)@lumi(?:[\s,!?:]|$)").unwrap()
);

pub fn mentions_lumi(body: &str) -> bool { MENTION_RE.is_match(body) }

pub fn count_mentions(body: &str) -> usize {
    MENTION_RE.find_iter(body).count()
}

pub enum BuiltinCmd { Help, Cancel }

pub fn parse_builtin_cmd(body: &str) -> Option<BuiltinCmd> {
    let re_help   = Regex::new(r"(?i)^\s*@lumi\s+help\s*$").unwrap();
    let re_cancel = Regex::new(r"(?i)^\s*@lumi\s+cancel\s*$").unwrap();
    if re_help.is_match(body)   { return Some(BuiltinCmd::Help); }
    if re_cancel.is_match(body) { return Some(BuiltinCmd::Cancel); }
    None
}

pub async fn handle_builtin_cmd(cmd: BuiltinCmd, req: &WebhookReq) -> Result<String, LumiError> {
    match cmd {
        BuiltinCmd::Help => Ok(LUMI_HELP_TEXT.to_string()),
        BuiltinCmd::Cancel => {
            match cuo::cancel_latest_for(req.tenant_id, req.user_id, &req.channel_id).await {
                Ok(true)  => Ok("Cancelled.".into()),
                Ok(false) => Ok("No in-flight Lumi request to cancel.".into()),
                Err(e)    => Err(e.into()),
            }
        }
    }
}

const LUMI_HELP_TEXT: &str = "Hi! I'm Lumi. Here's what I can do:

- Ask me anything — `@lumi <question>` will route to my supervisor.
- `@lumi cancel` — cancel my last in-flight reply for you in this channel.
- `@lumi help` — shows this message.

I can read the last few messages for context. I redact sensitive info before logging.";
```

### tenant_settings.rs + feature_flag.rs

```rust
pub mod tenant_settings {
    pub async fn lumi_enabled(tenant_id: uuid::Uuid) -> sqlx::Result<bool> {
        sqlx::query_scalar(
            "SELECT lumi_enabled FROM cyberos_chat_tenant_settings WHERE tenant_id = $1"
        ).bind(tenant_id).fetch_optional(&*POOL).await.map(|opt| opt.unwrap_or(false))
    }
}

pub mod feature_flag {
    pub async fn is_enabled(flag: &str, tenant_id: uuid::Uuid) -> Result<bool, FlagError> {
        let row = sqlx::query_scalar::<_, bool>(
            "SELECT enabled FROM cyberos_tenant_features WHERE tenant_id = $1 AND flag = $2"
        ).bind(tenant_id).bind(flag).fetch_optional(&*POOL).await?;
        Ok(row.unwrap_or(false))
    }
}
```

### rate_limiter.rs — Redis sliding window

```rust
pub mod rate_limiter {
    pub async fn check(
        tenant_id: uuid::Uuid,
        user_id: String,
        limit: u32,
        window: std::time::Duration,
    ) -> Result<bool, RedisError> {
        let key = format!("lumi:rl:{}:{}", tenant_id, user_id);
        let now_ms = unix_millis();
        let mut conn = REDIS.get_async_connection().await?;
        let (_, count): (i32, i32) = redis::pipe()
            .zadd(&key, format!("{}-{}", now_ms, rand::random::<u32>()), now_ms as i64)
            .zremrangebyscore(&key, 0, (now_ms - window.as_millis() as u64) as i64)
            .zcard(&key).ignore()
            .expire(&key, window.as_secs() as i64)
            .query_async(&mut conn).await?;
        Ok((count as u32) < limit)
    }
}
```

### reply.rs — threaded reply + edit

```rust
pub mod reply {
    pub async fn post_threaded(
        channel_id: &str,
        root_post_id: &str,
        body: &str,
        trace_id: &str,
    ) -> Result<String, MmError> {
        let body_with_footer = format!("{}\n\n_(via Lumi · trace `{}`)_", body, &trace_id[..8]);
        let resp = mm_client().post(
            format!("/api/v4/posts"),
            serde_json::json!({
                "channel_id": channel_id,
                "root_id":    root_post_id,
                "message":    body_with_footer,
                "props":      {"cyberos_lumi_reply": true, "trace_id": trace_id},
            })
        ).await?;
        Ok(resp["id"].as_str().unwrap().to_owned())
    }

    pub async fn edit(post_id: &str, new_body: &str, trace_id: &str) -> Result<(), MmError> {
        let body_with_footer = format!("{}\n\n_(via Lumi · trace `{}`)_", new_body, &trace_id[..8]);
        mm_client().put(
            format!("/api/v4/posts/{}/patch", post_id),
            serde_json::json!({ "message": body_with_footer })
        ).await?;
        Ok(())
    }
}
```

### Schema additions

```sql
-- services/chat/sql/init-lumi-settings.sql
CREATE TABLE IF NOT EXISTS cyberos_chat_tenant_settings (
    tenant_id       UUID PRIMARY KEY,
    lumi_enabled    BOOLEAN NOT NULL DEFAULT false,
    lumi_persona_id TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS cyberos_tenant_features (
    tenant_id  UUID NOT NULL,
    flag       TEXT NOT NULL,
    enabled    BOOLEAN NOT NULL DEFAULT false,
    PRIMARY KEY (tenant_id, flag)
);
```

---

## §4 — Acceptance criteria

1. **@lumi triggers regex** — "hey @lumi help" → match.
2. **Case-insensitive** — "@LUMI" → match.
3. **Doesn't match substring** — "@lumian" → NO match.
4. **Plugin POSTs to chat-lumi** — fixture mention → webhook fires.
5. **CUO routed with persona=lumi** — mock CUO sees persona.
6. **Resolved reply posted threaded** — root_id = mention post id.
7. **Routed reply with ETA** — CUO returns Routed → "I'll get back in ~Xs".
8. **Refused reply with reason** — CUO returns Refused → user sees reason.
9. **memory audit chat.lumi_invoked** — row emitted with redacted body.
10. **PII redacted** — body with email → audit row redacted.
11. **Dedup on edit** — message edited keeping @lumi → no second route.
12. **Latency p95 < 2s for Resolved** — measured via histogram.
13. **Private channel → reply sync_class: private**.
14. **Public channel → reply sync_class: shareable**.
15. **OTel counter increments per outcome**.
16. **Imported post @lumi mention ignored** — fixture post with `props.cyberos_imported=true` and body `@lumi help` → NO CUO call, NO reply (AC for §1 #11).
17. **Lumi's own posts don't re-trigger** — fixture: Lumi posts containing `@lumi` echo → NO recursion (AC for §1 #12).
18. **Rate limit enforced at 30/min** — fixture: 31 mentions from one user in one minute → 31st gets "Rate limit hit" reply; no CUO call (AC for §1 #13).
19. **Tenant disabled → skip + audit** — set `lumi_enabled=false`; mention → `chat.lumi_skipped` memory row; NO reply (AC for §1 #14).
20. **Feature flag disabled → skip silently** — set `tenant_features.lumi=false`; mention → NO action (AC for §1 #24).
21. **Placeholder fires at 2s** — fixture: CUO sleeps 5s; observe `_(Lumi is thinking…)_` post at t≈2s, then edit at t≈5s (AC for §1 #15).
22. **Trace id preserved** — fixture: inject `traceparent` header; observe same trace_id in memory audit, CUO call, reply post props (AC for §1 #16).
23. **Channel context in CUO request** — observe CUO call receives `channel_member_count` + `user_is_admin` (AC for §1 #17).
24. **`@lumi help` is hard-coded** — fixture: body `@lumi help`; observe no CUO call AND latency <100ms (AC for §1 #18).
25. **`@lumi cancel` cancels routed job** — start a Routed job; same user same channel sends `@lumi cancel`; observe `cuo::cancel_latest_for` called; reply `Cancelled.` (AC for §1 #19).
26. **Multiple @lumi → single invocation** — body with `@lumi do A and @lumi do B`; observe ONE CUO call; memory audit `mention_count = 2` (AC for §1 #20).
27. **memory payload carries user_display + role** — observe audit row has `user_display` (redacted) + `user_role` (AC for §1 #21).
28. **Response redacted in audit** — CUO returns body containing email; observe `response_redacted` field in audit has `<EMAIL>`; `response_hash` is unredacted hash (AC for §1 #22).
29. **chat.lumi_error fires on CUO timeout** — fixture: CUO timeout; observe `chat.lumi_error` row with `error_class="cuo_timeout"` + SEV-2 routing (AC for §1 #23).
30. **Budget recorded in audit** — observe `budget_tokens_used > 0` + `budget_dollars_estimate > 0` in audit payload for Resolved outcomes (AC for §1 #25).

---

## §5 — Verification

### Regex coverage (Go plugin)

```go
func TestMentionRegex(t *testing.T) {
    cases := []struct{ in string; want bool }{
        {"hey @lumi help",       true},
        {"@LUMI please",         true},
        {"@Lumi, status?",       true},
        {"@lumi?",               true},
        {"@lumi!",               true},
        {"@lumi:",               true},
        {"@lumi",                true}, // EOL
        {"@lumian",              false}, // substring
        {"@lumi@",               false}, // followed by non-allowed
        {"email@lumi.com",       false}, // not at word boundary
        {"just text",            false},
    }
    for _, c := range cases {
        if got := mentionsLumi(c.in); got != c.want {
            t.Errorf("mentionsLumi(%q) = %v, want %v", c.in, got, c.want)
        }
    }
}

func TestCountMentions(t *testing.T) {
    assert.Equal(t, 0, countMentions("hello"))
    assert.Equal(t, 1, countMentions("@lumi please"))
    assert.Equal(t, 2, countMentions("@lumi do A and @lumi do B"))
}
```

### AC #6/#7/#8 — outcome routing

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac6_resolved_reply_threaded() {
    let env = TestEnv::new().await;
    env.cuo.mock_resolved("Sure, I can help.").await;
    let req = test_webhook("@lumi help with X");
    handle_webhook(req).await.unwrap();
    let post = env.mm.last_reply().await;
    assert!(post.message.contains("Sure, I can help."));
    assert_eq!(post.root_id, Some(req.post_id));
    assert!(post.message.contains("via Lumi · trace"));
}

#[tokio::test(flavor = "multi_thread")]
async fn ac7_routed_reply_with_eta() {
    let env = TestEnv::new().await;
    env.cuo.mock_routed(45, "job-1").await;
    let req = test_webhook("@lumi do something complex");
    handle_webhook(req).await.unwrap();
    let post = env.mm.last_reply().await;
    assert!(post.message.contains("~45s"));
    assert!(post.message.contains("cancel with"));
}

#[tokio::test(flavor = "multi_thread")]
async fn ac8_refused_reply_with_reason() {
    let env = TestEnv::new().await;
    env.cuo.mock_refused("policy: financial advice").await;
    let req = test_webhook("@lumi should I buy bitcoin");
    handle_webhook(req).await.unwrap();
    let post = env.mm.last_reply().await;
    assert!(post.message.contains("policy: financial advice"));
}
```

### AC #16 — imported posts ignored

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac16_imported_posts_ignored() {
    let env = TestEnv::new().await;
    let req = WebhookReq { is_imported: true, body: "@lumi help".into(), ..test_webhook("") };
    handle_webhook(req).await.unwrap();
    assert_eq!(env.cuo.call_count(), 0);
    assert_eq!(env.mm.reply_count(), 0);
}
```

### AC #17 — Lumi's own posts skipped

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac17_lumi_own_posts_skipped() {
    let env = TestEnv::new().await;
    let req = WebhookReq {
        user_id: lumi_system_user_id(),
        body: "you asked '@lumi help' earlier".into(),
        ..test_webhook("")
    };
    handle_webhook(req).await.unwrap();
    assert_eq!(env.cuo.call_count(), 0);
}
```

### AC #18 — rate limit

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac18_rate_limit_at_30_per_min() {
    let env = TestEnv::new().await;
    env.cuo.mock_resolved("ok").await;
    let req_template = test_webhook("@lumi q");
    for _ in 0..30 { handle_webhook(req_template.clone()).await.unwrap(); }
    // 31st should hit rate limit.
    env.mm.clear_replies().await;
    let req31 = WebhookReq { post_id: "p-31".into(), ..req_template.clone() };
    handle_webhook(req31).await.unwrap();
    let last = env.mm.last_reply().await;
    assert!(last.message.contains("Rate limit hit"));
    assert_eq!(env.cuo.call_count(), 30); // not 31
}
```

### AC #19 — tenant disabled

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac19_tenant_disabled_emits_skipped_audit() {
    let env = TestEnv::new().await;
    env.set_tenant_lumi_enabled(false).await;
    let req = test_webhook("@lumi help");
    handle_webhook(req).await.unwrap();
    let row = env.memory.last_of_kind("chat.lumi_skipped").await.unwrap();
    assert_eq!(row["payload"]["reason"], "disabled");
    assert_eq!(env.mm.reply_count(), 0);
}
```

### AC #21 — placeholder + edit

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac21_placeholder_then_edit() {
    let env = TestEnv::new().await;
    env.cuo.mock_resolved_after("Sorry for the wait — here's the answer.",
        Duration::from_secs(5)).await;
    let req = test_webhook("@lumi long question");
    let start = std::time::Instant::now();
    handle_webhook(req).await.unwrap();
    let posts = env.mm.replies_for_root("p-test").await;
    assert_eq!(posts.len(), 1, "should be 1 post that was edited, not 2");
    let edits = env.mm.edits_for_post(&posts[0].id).await;
    assert!(edits.len() >= 2);
    let first = &edits[0];
    let last = edits.last().unwrap();
    assert!(first.message.contains("thinking"));
    assert!(last.message.contains("Sorry for the wait"));
}
```

### AC #22 — trace id propagation

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac22_trace_id_propagated() {
    let env = TestEnv::new().await;
    let trace = "4bf92f3577b34da6a3ce929d0e0e4736";
    let req = WebhookReq { trace_id: trace.into(), ..test_webhook("@lumi help") };
    handle_webhook(req).await.unwrap();
    let row = env.memory.last_of_kind("chat.lumi_invoked").await.unwrap();
    assert_eq!(row["payload"]["trace_id"], trace);
    let cuo_call = env.cuo.last_request().await;
    assert_eq!(cuo_call.trace_id, trace);
    let post_props = env.mm.last_reply().await.props;
    assert_eq!(post_props["trace_id"], trace);
}
```

### AC #24 — `@lumi help` hard-coded

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac24_help_is_hard_coded() {
    let env = TestEnv::new().await;
    let start = std::time::Instant::now();
    handle_webhook(test_webhook("@lumi help")).await.unwrap();
    let dur = start.elapsed();
    assert_eq!(env.cuo.call_count(), 0);
    assert!(dur < Duration::from_millis(100), "took {:?}", dur);
    let reply = env.mm.last_reply().await;
    assert!(reply.message.contains("Hi! I'm Lumi"));
}
```

### AC #25 — `@lumi cancel`

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac25_cancel_in_flight_job() {
    let env = TestEnv::new().await;
    env.cuo.mock_routed(60, "job-X").await;
    handle_webhook(test_webhook("@lumi long thing")).await.unwrap();

    env.cuo.set_cancel_will_succeed("job-X");
    handle_webhook(test_webhook("@lumi cancel")).await.unwrap();
    let posts = env.mm.replies_for_root("p-test").await;
    assert!(posts.iter().any(|p| p.message.contains("Cancelled")));
}
```

### AC #26 — multiple mentions, one invocation

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac26_multiple_mentions_single_invocation() {
    let env = TestEnv::new().await;
    env.cuo.mock_resolved("ok").await;
    handle_webhook(test_webhook("@lumi do A and @lumi do B")).await.unwrap();
    assert_eq!(env.cuo.call_count(), 1);
    let row = env.memory.last_of_kind("chat.lumi_invoked").await.unwrap();
    assert_eq!(row["payload"]["mention_count"], 2);
}
```

### AC #28 — response redacted in audit

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac28_response_redacted_in_audit() {
    let env = TestEnv::new().await;
    env.cuo.mock_resolved("Email alice@cyberskill.world for follow-up.").await;
    handle_webhook(test_webhook("@lumi who do I email")).await.unwrap();
    let row = env.memory.last_of_kind("chat.lumi_invoked").await.unwrap();
    let resp_red = row["payload"]["response_redacted"].as_str().unwrap();
    assert!(!resp_red.contains("alice@cyberskill.world"));
    assert!(resp_red.contains("<EMAIL>"));
    // But response_hash is on the UNREDACTED text.
    assert!(row["payload"]["response_hash"].as_str().unwrap().len() == 64);
}
```

### AC #29 — chat.lumi_error on CUO timeout

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac29_lumi_error_audit_on_cuo_timeout() {
    let env = TestEnv::new().await;
    env.cuo.mock_timeout().await;
    handle_webhook(test_webhook("@lumi help")).await.unwrap();
    let row = env.memory.last_of_kind("chat.lumi_error").await.unwrap();
    assert_eq!(row["payload"]["error_class"], "cuo_timeout");
    let reply = env.mm.last_reply().await;
    assert!(reply.message.contains("trouble"));
}
```

### AC #30 — budget recorded

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac30_budget_recorded_in_audit() {
    let env = TestEnv::new().await;
    env.cuo.mock_resolved_with_budget("ok", 1500, 0.012).await;
    handle_webhook(test_webhook("@lumi help")).await.unwrap();
    let row = env.memory.last_of_kind("chat.lumi_invoked").await.unwrap();
    assert_eq!(row["payload"]["budget_tokens_used"], 1500);
    assert!((row["payload"]["budget_dollars_estimate"].as_f64().unwrap() - 0.012).abs() < 1e-9);
}
```

---

## §6 — Implementation skeleton

The Rust + Go modules above are the skeleton. This section names the operational wiring:

### §6.1 — Plugin vs service split rationale

The Mattermost plugin is intentionally minimal:
- Parse mention.
- Filter (imported, Lumi-self).
- POST to chat-lumi service.

Everything else (CUO routing, rate limit, redaction, audit, reply) lives in the Rust service. Reasons:
1. Plugin upgrades are tied to MM upgrades; service upgrades are independent.
2. Rust ecosystem has better LLM/AI integration crates than Go.
3. Service can scale independently of MM (one chat-lumi pod can serve multiple MM tenants).
4. Service can be ablated for canary testing without touching MM.

### §6.2 — Plugin → service transport

Plugin POSTs to `http://chat-lumi.<tenant>.svc.cluster.local:8080/webhook` with the payload. The service responds 200 immediately (queues the work); the actual CUO call + reply happens asynchronously. This keeps the plugin's `MessageHasBeenPosted` hook fast (<10ms).

### §6.3 — Placeholder edit semantics

The placeholder pattern (§1 #15) uses MM's PATCH endpoint to edit an existing post rather than posting a second message. The PATCH preserves the post's create_at (so the message stays in the original chronological position) but updates update_at. Users see the edit live via WebSocket.

### §6.4 — CUO budget capture

Budget fields come from FR-AI-014's per-invocation accounting. The Rust service receives them as part of the CUO response payload. If FR-AI-014 returns no budget info (older CUO version), the audit row carries `budget_tokens_used: null` + a SEV-3 warning is logged.

### §6.5 — Rate-limit storage choice

Redis (not Postgres) because:
- 30/min × N users = millions of writes/day across a fleet.
- Postgres would either OOM with the sorted-set pattern or require partitioning.
- Redis ZADD + ZREMRANGEBYSCORE + ZCARD is ~5 ops O(log N); trivial.

### §6.6 — Per-tenant settings caching

`cyberos_chat_tenant_settings` is read on EVERY webhook. We cache the row in-process with 30s TTL to avoid hammering Postgres. Cache invalidation on `cyberos_chat_tenant_settings.updated_at` change is via FR-AUTH-005 webhook (same pattern as tenant_map in FR-CHAT-002).

### §6.7 — Feature-flag wiring

`cyberos_tenant_features` is the canonical feature-flag store. It's also read every webhook + cached 30s. The two-key-launch pattern (tenant enable + feature flag) requires BOTH to be true; either false short-circuits with no CUO call.

### §6.8 — Lumi system user provisioning

The plugin install step (FR-CHAT-002 install workflow) creates a MM user `Lumi` with `is_bot=true`, `cyberos_system_user=true` props. The bot user's ID is read once at plugin OnActivate and cached in `lumi_system_user_id()`.

### §6.9 — Trace propagation chain

- MM plugin reads `traceparent` from inbound HTTP (or mints one).
- Plugin POSTs to chat-lumi service with `traceparent` header.
- Service passes through to CUO call as `traceparent`.
- Service passes back to MM reply post as `props.trace_id` AND in the footer (truncated to 8 chars for human readability).
- memory audit `trace_id` is the full 32-char form.

### §6.10 — chat.lumi_error severity routing

Single errors are SEV-3 (logged, counter incremented). Sustained errors (>1/min) escalate via FR-OBS-007 alarm to SEV-2. This avoids paging operators on transient CUO timeouts while still surfacing structural failures.

### §6.11 — Test fixtures

Test fixtures live in `services/chat-lumi/tests/fixtures/`:
- `mock_cuo.rs` — programmable CUO stub for unit tests.
- `mock_mm.rs` — in-process MM API mock that records posts/edits/patches.
- `mock_memory.rs` — in-memory memory audit sink.
- `mock_redis.rs` — Redis-in-process for rate limiter tests.

### §6.12 — Failure routing matrix

| Failure | Audit row | Reply | Metric |
|---|---|---|---|
| CUO timeout | chat.lumi_error (cuo_timeout) | "I'm having trouble" | outcome=error |
| CUO refused | chat.lumi_invoked | refusal reason | outcome=refused |
| MM API 5xx on reply | chat.lumi_error (mm_api) | none | outcome=error |
| Redaction crash | chat.lumi_error (redaction) | "I'm having trouble" | outcome=error |
| Rate limit hit | (no chat.lumi_invoked) | rate-limit notice | outcome=rate_limited |
| Tenant disabled | chat.lumi_skipped | none | outcome=skipped |
| Builtin handled | chat.lumi_invoked (kind=builtin) | template text | outcome=builtin |

---

## §7 — Dependencies

- **FR-CHAT-005** — bridge picks up the reply too.
- **FR-CHAT-009** — retro-capture is sibling flow.
- **FR-CUO-101 (placeholder)** — supervisor.
- **FR-AI-014** — persona-stamped LLM via CUO.
- **FR-MEMORY-111** — PII redaction.

---

## §8 — Example payloads

### `chat.lumi_invoked` — resolved happy path

```json
{
  "kind": "chat.lumi_invoked",
  "ts_ns": 1747407137800000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736",
  "payload": {
    "post_id":       "01HVQX8ZG2K3R4TVA7P3WV5X8N",
    "channel_id":    "01HVQX8ZG2K3R4TVA7P3WV5X8M",
    "user_id":       "01HVQX8ZG2K3R4TVA7P3WV5X8K",
    "user_display":  "alice <NAME>",
    "user_role":     "system_user",
    "body_redacted": "hey @lumi help triage <EMAIL>",
    "context_hash":  "9b0e1c2d3a4f5e6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b",
    "cuo_outcome":   "resolved",
    "response_redacted": "I emailed <EMAIL> on your behalf with the triage notes.",
    "response_hash": "ab12cd34ef56ab12cd34ef56ab12cd34ef56ab12cd34ef56ab12cd34ef56ab12",
    "mention_count": 1,
    "channel_member_count": 18,
    "budget_tokens_used":  1532,
    "budget_dollars_estimate": 0.012,
    "latency_ms":    847
  }
}
```

### `chat.lumi_invoked` — refused

```json
{
  "kind": "chat.lumi_invoked",
  "ts_ns": 1747407138100000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "payload": {
    "post_id":         "01HVQX8ZG2K3R4TVA7P3WV5X8P",
    "cuo_outcome":     "refused",
    "response_redacted": "I can't help with that: policy: financial advice",
    "budget_tokens_used":  0,
    "latency_ms":      120
  }
}
```

### `chat.lumi_invoked` — builtin (`@lumi help`)

```json
{
  "kind": "chat.lumi_invoked",
  "ts_ns": 1747407138200000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "payload": {
    "post_id":     "01HVQX8ZG2K3R4TVA7P3WV5X8Q",
    "cuo_outcome": "builtin_help",
    "budget_tokens_used": 0,
    "latency_ms": 18
  }
}
```

### `chat.lumi_skipped` — tenant disabled

```json
{
  "kind": "chat.lumi_skipped",
  "ts_ns": 1747407137100000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "payload": {
    "post_id": "01HVQX8ZG2K3R4TVA7P3WV5X8N",
    "reason":  "disabled"
  }
}
```

### `chat.lumi_error` — CUO timeout

```json
{
  "kind": "chat.lumi_error",
  "ts_ns": 1747407140000000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "severity": "SEV-3",
  "payload": {
    "post_id":     "01HVQX8ZG2K3R4TVA7P3WV5X8N",
    "channel_id":  "01HVQX8ZG2K3R4TVA7P3WV5X8M",
    "user_id":     "01HVQX8ZG2K3R4TVA7P3WV5X8K",
    "error_class": "cuo_timeout",
    "error_message_redacted": "CUO did not respond within 60s for trace 4bf92f...",
    "trace_id":    "4bf92f3577b34da6a3ce929d0e0e4736"
  }
}
```

### Webhook from plugin → chat-lumi service

```json
POST /webhook HTTP/1.1
Host: chat-lumi.tenant-1.svc.cluster.local:8080
Content-Type: application/json
traceparent: 00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01

{
  "post_id":     "01HVQX8ZG2K3R4TVA7P3WV5X8N",
  "channel_id":  "01HVQX8ZG2K3R4TVA7P3WV5X8M",
  "user_id":     "01HVQX8ZG2K3R4TVA7P3WV5X8K",
  "user_display_name": "alice cheng",
  "user_role":   "system_user",
  "body":        "hey @lumi help triage alice@cyberskill.world",
  "context":     ["earlier message", "another earlier", "and another"],
  "tenant_id":   "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "is_imported": false,
  "trace_id":    "4bf92f3577b34da6a3ce929d0e0e4736"
}
```

### MM reply post (rendered)

```
> @lumi help triage alice@cyberskill.world

Sure — I've drafted a triage note. Should I send it to alice?

_(via Lumi · trace `4bf92f35`)_
```

---

## §9 — Open questions

All resolved. Deferred:
- @lumi in DM with another user (3-way conversation) — slice 4+.
- Voice transcription @lumi — slice 5+.
- Per-user Lumi profiles — slice 4+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| CUO unreachable | HTTP error / timeout | post "I'm having trouble"; chat.lumi_error; SEV-3 (SEV-2 if sustained >1/min) | Operator restores CUO |
| CUO timeout (>60s) | tokio::time::timeout | as above | as above |
| CUO returns malformed response | serde parse Err | chat.lumi_error (cuo_parse); fall-back reply | Operator investigates CUO version drift |
| Plugin crash | MM restart | down briefly; restart loop | Operator investigates |
| Dedup race (concurrent edit) | markRouted serialised in plugin via in-process LRU + MM KVStore | None | None |
| MM API post fails (5xx) | http_client err | retry 3×; chat.lumi_error if all fail | Operator investigates |
| MM API auth expired | 401 | chat.lumi_error; SEV-2 | Operator rotates bot token |
| PII redaction crash on request body | catch_unwind | chat.lumi_error (redaction); fall back to refused | Operator fixes ruleset |
| PII redaction crash on response body | catch_unwind | chat.lumi_error; reply already posted; audit has placeholder | Operator fixes ruleset |
| Lumi system user missing | first post fails 404 | SEV-1 chat.lumi_error; install incomplete | Operator provisions via FR-CHAT-002 admin REST |
| Plugin upgrade mid-mention | hooks paused | next mention triggers; no loss | None |
| Mention in archived channel | MM reply 403 | chat.lumi_error; no user-visible reply | None |
| Context extraction fails | empty context passed to CUO | CUO still works (degraded) | None |
| MM rate limit on reply | 429 | retry with Retry-After; chat.lumi_error if exhausted | Operator investigates burst |
| Lumi reply > 16KB | MM 400 on post | truncate to 16KB + footer; warn | None |
| Imported post bug (props.cyberos_imported missing) | bridge gap | mention triggers (false positive) | Operator backfills props |
| Self-recursion (Lumi user_id check fails) | recursion depth limit | exit after 1 recursion; SEV-1 | Operator verifies lumi_system_user_id() |
| Rate limit Redis unreachable | redis err | fail-OPEN (allow mention) + SEV-3 warning; chat.lumi_error (rate_limiter_unavailable) | Operator restores Redis |
| Rate limit Redis cluster failover | brief blip | per-pod retry; fail-open | None |
| Tenant settings read fails | DB err | fail-CLOSED (treat as disabled); SEV-3 | Operator restores DB |
| Feature-flag read fails | DB err | fail-CLOSED | as above |
| Placeholder PATCH fails | MM 404 (post deleted) | log; post fresh reply instead | None |
| Placeholder PATCH races CUO resolve (< 2s) | placeholder didn't fire | post fresh reply normally | None |
| @lumi cancel on no in-flight job | cuo::cancel_latest_for returns false | reply "No in-flight Lumi request to cancel." | None |
| @lumi cancel by user who didn't start the job | cuo::cancel_latest_for filters by user_id | as above | None |
| Two @lumi mentions in same post | count_mentions = 2; single CUO call | None | None |
| Trace_id missing in inbound | service mints new one | trace continuity broken at boundary | Investigate plugin |
| MM editing window expires (5min default) | PATCH 403 | post fresh reply; warn | None |
| Channel deleted while Lumi is replying | MM 404 | chat.lumi_error; no reply | None |
| Tenant deleted while Lumi is replying | DB FK orphan | chat.lumi_error (sev-1) | Operator investigates |
| Lumi config (cyberos_chat_tenant_settings) corrupted (lumi_persona_id invalid) | CUO returns refused | reply with error message | Operator fixes settings |
| CUO budget unavailable in response | FR-AI-014 older version | audit row has budget_tokens_used=null + SEV-3 | Upgrade CUO |
| Webhook payload too large (>1MB) | http 413 from chat-lumi | mention dropped; SEV-3 | Investigate message size |
| Concurrent @lumi cancel from same user | both succeed; second is no-op | None | None |
| memory audit emit fails | logged + counter | mention not auditable | Operator restores memory |
| OBS metrics collector down | metrics dropped | None visible | None |
| MM hot-reload causes plugin re-init | plugin lifecycle | mention queue drained; no loss | None |
| LLM hallucination in reply | n/a; not detectable | user sees wrong answer | Operator improves persona/safety rules |
| User mentions @lumi inside a thread reply (not top-level) | parent post is the mention | reply threads to root (= mention post) | None |
| User mentions @lumi in a Lumi-authored reply (recursion attempt) | filtered by user_id check | None | None |
| Service crashes mid-request | placeholder reply hangs | next mention triggers; old placeholder remains in chat | Operator periodic cleanup |
| Service crashes during CUO call | request lost | user sees no reply; SEV-3 | User re-asks |
| MM API DRAFT mode (post saved but unsent) | rare; corrupt state | chat.lumi_error | Operator investigates |
| Tenant tier downgrade while Lumi in-flight | next call uses new tier | None | None |

---

## §11 — Implementation notes

- Plugin uses `MessageHasBeenPosted` hook (post-save, runs after persist). We considered `MessageWillBePosted` (pre-save) but the timing is wrong — we want the mention preserved in the audit trail even if the LLM call fails. Pre-save would let us reject the mention, which we don't want.
- Dedup via in-process LRU cache keyed by post.Id (size 10k); survives plugin restart via Mattermost's KV store (slower but durable). Edge case: edit-then-revert that adds @lumi back doesn't re-trigger — by design (avoids flapping).
- Context = 4 sentences before + 4 after the mention; tokens cap ~500. Calibrated against typical Lumi prompts: too little context = confused replies; too much = wasted budget.
- Reply system user = "Lumi" with avatar; provisioned at plugin install via FR-CHAT-002 admin REST. The avatar PNG ships in the plugin bundle.
- Slice-3 CUO stub returns canned acknowledgement; full LLM compose in P2 via FR-CUO-101. The Lumi-mention contract is stable; only the CUO backend evolves.
- The regex `(?i)@lumi(?:[\s,!?:]|$)` was chosen carefully: `[\s,!?:]` allows common punctuation after the mention; `$` allows end-of-line; the alternation excludes word characters (no false-positive on `@lumian`). Tested against ~200 realistic prompts.
- The "thinking..." placeholder pattern requires MM's edit-window to be ≥ 2× the LLM timeout (default 5min vs our 60s budget — fits). If MM admin shrinks the edit window, the edit path fails and we post fresh; degraded but not broken.
- We chose 30 mentions/min per user as the rate limit because: (a) realistic human max is ~5/min; (b) bot scripts get caught at 30; (c) the 6× headroom prevents false positives during legitimate burst (e.g. troubleshooting workflow).
- Why fail-OPEN on Redis unreachable (rate limiter): rate limiting is a budget guardrail, not a security gate. Failing closed during Redis outages would block legitimate users for unrelated infrastructure issues. We accept the brief budget exposure.
- Why fail-CLOSED on tenant settings + feature flag read: these gate WHETHER Lumi runs at all. Defaulting to "off" on read failure is safer than defaulting to "on" (could expose disabled tenants to LLM calls they don't want).
- The footer `_(via Lumi · trace <8-char>)_` is calibrated for UX: short enough to not crowd the reply, long enough that operators tracking down complaints can quote it back.
- Context extraction uses sentence-boundary heuristics (period, question mark, exclamation followed by whitespace). VN text with no punctuation degrades to "use the post body alone"; acceptable degradation.
- The placeholder + edit pattern means the same MM post_id is preserved across the placeholder + final reply. Downstream consumers (FR-CHAT-005 bridge) see the edit as a chat.message_edited event; that's fine.
- We considered combining `chat.lumi_invoked` + `chat.lumi_error` into a single row with a status field. Rejected: filtering on error rows is far easier with a separate kind.
- Why `@lumi help` is hard-coded: help requests are predictable, the response is static, no LLM judgment needed. Cost savings + reliability improvements (help works even when LLM service is degraded).
- The `@lumi cancel` command intentionally only cancels the most-recent unresolved job per (tenant, user, channel). Cancelling arbitrary jobs by ID would be a power-user feature; not in MVP scope.
- We use trace_id rather than post_id as the cancellation key because trace_id is stable across plugin → service → CUO; post_id is local to MM.
- The CUO call timeout (60s) is calibrated against FR-CUO-101's stated SLA. Longer would block the placeholder forever; shorter would prematurely fail.
- We emit `chat.lumi_skipped` (not an error) for tenant-disabled because it's an expected operational state, not a fault.
- Why redact response separately (§1 #22): the model may emit data it inferred from training, not just what was in the prompt. Re-redacting the response catches these.
- `mention_count` in the memory payload is informational; we don't use it for billing (one CUO call = one billable unit regardless of mentions).
- We chose Lumi over other persona names because: (a) it's a single short syllable (easy to type in chat); (b) doesn't conflict with common usernames; (c) operator brand-recognition. Documented in DEC-490.
- Channel context (`channel_member_count`, `user_is_admin`) is sent in CUO request but NOT stored in memory audit (per FR-MEMORY-111 minimal-data principle). CUO uses it for one decision then discards.
- The reply footer's trace hash (8 chars) is not cryptographically meaningful; it's a debugging hint. Operators correlating logs use the full 32-char form from memory.
- `is_imported` propagation from plugin → service is critical; without it, the service would not have access to the post's props field directly. We pass it pre-computed.
- Builtin command parsing (`@lumi help`, `@lumi cancel`) is regex-anchored to `^\s*@lumi\s+<cmd>\s*$` so it doesn't match mid-sentence (`I asked @lumi help with X` is a regular question, not a help command).

---

*End of FR-CHAT-008.*
