---
id: TASK-CHAT-007
title: "Zalo manual export importer — `cyberos-chat import zalo --bundle.zip` with VN-Unicode normalisation and Zalo-specific message kinds"
module: CHAT
priority: SHOULD
status: superseded
superseded_by: TASK-CHAT-101 (first-party native chat replaced the Mattermost fork wholesale; still-wanted intents re-homed as TASK-CHAT-102..106)
verify: T
phase: P1
milestone: P1 · slice 2
slice: 2
owner: Stephen Cheng
created: 2026-05-16
shipped: null
memory_chain_hash: null
related_tasks: [TASK-CHAT-005, TASK-CHAT-006, TASK-CHAT-010]
depends_on: [TASK-CHAT-006]
blocks: [TASK-CHAT-010]

source_pages:
  - website/docs/modules/chat.html#zalo-import
source_decisions:
  - DEC-480 (Zalo has no API; rely on manual export bundle (HTML + media))
  - DEC-481 (reuse TASK-CHAT-006 step pattern; Zalo-specific parser per source)

language: rust 1.81
service: cyberos/services/chat-importer/
new_files:
  - services/chat-importer/src/zalo/mod.rs
  - services/chat-importer/src/zalo/parse_html.rs
  - services/chat-importer/src/zalo/normalize.rs
  - services/chat-importer/tests/zalo_test.rs
modified_files:
  - services/chat-importer/src/main.rs               # add `zalo` subcommand
allowed_tools:
  - file_read: services/chat-importer/**
  - file_write: services/chat-importer/{src,tests}/**
  - bash: cd services/chat-importer && cargo test zalo
disallowed_tools:
  - call Zalo API (per DEC-480 — none exists)
  - skip Unicode normalisation (NFC required per VN compose forms)

effort_hours: 8
subtasks:
  - "0.5h: main.rs subcommand `zalo <bundle.zip>`"
  - "0.5h: zalo/mod.rs — module skeleton"
  - "2.0h: parse_html.rs — scraper crate; extract messages from Zalo HTML export"
  - "1.0h: normalize.rs — Unicode NFC + Zalo-specific emoji codes → standard"
  - "1.0h: map Zalo conversation_id → MM channel"
  - "1.0h: map Zalo user_id → MM user (with display_name fallback when no email)"
  - "0.5h: handle Zalo group vs 1-1 (group → MM private channel; 1-1 → MM DM)"
  - "0.5h: media files (images, voice, video) → MM file upload"
  - "1.0h: zalo_test.rs — synthetic Zalo bundle + 30+ message variants"
  - "0.5h: same checkpoint table as TASK-CHAT-006"
  - "0.5h: memory audit shared with import.rs"
risk_if_skipped: "Vietnamese SMB users live on Zalo (90% market share); without import, they abandon years of business chat. Zalo HTML quirks (mixed encodings, emoji shortcodes) break naive parsers. Without checkpoints, large bundles fail mid-import."
---

## §1 — Description (BCP-14 normative)

The Zalo importer **MUST** parse a manual Zalo HTML export bundle and import into Mattermost. The contract:

1. **MUST** accept zip bundle structure: `messages/<conv>/<date>.html` + `media/<sha>/...` + `metadata.json`.
2. **MUST** use 6 steps (skip "channel_members" + "threads" — Zalo has no thread primitives):
1. Validate, 2. Users, 3. Channels, 4. Messages, 5. Files, 6. Verify.
3. **MUST** parse HTML via `scraper` crate; extract `<div class="msg" data-user-id="..." data-ts="..."><span class="body">...</span></div>` blocks.
4. **MUST** NFC-normalise all text bodies before insert; Zalo uses mixed NFC/NFD.
5. **MUST** map Zalo-specific emoji codes (e.g. `:>>` → 😄) via curated table.
6. **MUST** map conversation types:
- Group → MM private channel; channel name = group name; members = group participants.
- 1-1 → MM DM channel between the two users.
7. **MUST** handle Zalo users without email: synthesize MM email `zalo-user-<zalo_id>@imported.cyberos.local`; display_name from Zalo profile.
8. **MUST** dedup at message level by `(channel_id, zalo_msg_id)` stored in MM post props.
9. **MUST** reuse TASK-CHAT-006's checkpoint table (`import_jobs` with `source='zalo'`).
10. **MUST** emit memory audit `chat.import_*` rows same as TASK-CHAT-006.
11. **MUST** support `--dry-run` + `--resume` flags same as TASK-CHAT-006.
12. **MUST** RLS-enforce.
13. **MUST** emit OTel metrics with label `source=zalo`.
14. **MUST** pin the exporter version in `metadata.json` and refuse to import bundles produced by an unsupported exporter version. Supported set is enumerated in `services/chat-importer/src/zalo/supported_versions.rs`. Unknown version → exit 1 with SEV-1 `chat.import_unsupported_zalo_version` audit.
15. **MUST** detect Zalo's two HTML schema generations:
- **Gen-1** (pre-2023): `<div class="msg" data-...>` with attributes on the div.
- **Gen-2** (2023+): `<article data-zalo-msg-id="...">` with semantic HTML. Each generation has its own parser; selection is by metadata.json `schema_version`.
16. **MUST** handle Zalo's per-message reactions: `<div class="reaction" data-emoji="..." data-user-ids="u1,u2">`. Mapped to MM `reactions` row per user.
17. **MUST** preserve Zalo voice-message audio files: `audio/<msg_id>.m4a` extracted from bundle, uploaded to MM file store, and the parent post body MUST contain `[voice message · <duration>s]` placeholder so the MM UI can render an audio attachment.
18. **MUST** preserve Zalo video files and image files: bundle structure `media/<sha>/...` with metadata in `metadata.json::media[]`. Files step (#5) uploads each via MM API; dedup by `(zalo_workspace_id, media_sha)`.
19. **MUST** handle Zalo stickers: each sticker is a PNG in `stickers/<sticker_id>.png`; mapped to MM custom emoji namespace `zalo-sticker-<id>`. Posts referencing the sticker use `:zalo-sticker-<id>:` in the body.
20. **MUST** preserve Zalo's reply primitive (Zalo has "reply to message" arrows, like Slack thread replies). Mapped to MM `root_id` via `(channel_id, zalo_parent_msg_id)` lookup. Orphan replies handled per TASK-CHAT-006 pattern (top-level with `props.zalo_reply_orphan = true`).
21. **MUST** preserve Zalo's deleted-message tombstones: `<div class="msg deleted" data-msg-id="...">` MUST produce a MM post whose body is `[message recalled]` and `delete_at > 0` AND `props.zalo_was_recalled = true`. This preserves the audit trail without exposing recalled content.
22. **MUST** PII-redact filenames (per Vietnamese conventions, filenames frequently embed full names — `CV_TrinhThaiAnh_2026.pdf`) before memory audit emit; same pattern as TASK-CHAT-006 §1 #17.
23. **MUST** detect Zalo's mixed timestamp formats: bundles MAY contain `data-ts` as seconds (older) OR milliseconds (newer) OR ISO-8601 (rare). Heuristic: if value < 10^10 → seconds (× 1000); if 10^10 ≤ value < 10^14 → milliseconds; if string contains `T` → ISO-8601. Ambiguous → fail with SEV-2 `chat.import_timestamp_ambiguous`.
24. **MUST** preserve Zalo's group-membership history events as MM channel-member events: `<div class="event" data-type="user_joined" data-user-id="..." data-ts="...">` → MM `chat.user_joined_channel` memory row.
25. **MUST** support `--bundle-encoding` flag: Zalo's older exports use Windows-1258 for VN text in some fields; default decoder is UTF-8; flag overrides for legacy exports.
26. **MUST** verify NFC-normalised output is a valid Unicode string (round-trip through `.chars().collect::<String>()`); invalid sequences → SEV-2 warning + best-effort replacement with `U+FFFD`.
27. **MUST** distinguish 1-1 chats from MPIM by participant count in `metadata.json::conversations[].participants[]`: 2 → DM; 3+ → MPIM (group channel).
28. **MUST** support `--strict` flag that converts warnings (orphans, missing media, emoji unmapped) into hard errors. Used in CI to catch bundle quality issues; not the default for production imports.

---

## §2 — Why this design

**Why HTML scraper (DEC-480)?** Zalo provides no API; manual export is HTML. Scraping is fragile but unavoidable. Brittle parsers + clear test fixtures = manageable.

**Why NFC normalisation (§1 #4)?** Vietnamese text often has combining diacritics (NFD) mixed with composed forms (NFC). Search + comparison need consistent form. NFC is the canonical choice (matches TASK-CHAT-004 tokeniser).

**Why synthesise email (§1 #7)?** MM requires email. Zalo users have phone+display_name; no email. Synthesised email is unique per Zalo ID; operator-recognisable suffix.

**Why share checkpoint table (§1 #9)?** Operator imports both Slack + Zalo for same tenant; shared schema = one query for status.

**Why pin exporter version (§1 #14)?** Zalo's HTML export format is undocumented and changes silently. Refusing unknown versions = explicit failure that operator can route to Zalo support OR a parser update; silent best-effort would land partial data. Operators prefer loud failure here.

**Why detect Gen-1 vs Gen-2 (§1 #15)?** Same Zalo product, two export formats — same problem as Mattermost v9 vs v10 schema drift. A single parser would be hairy and fragile. Two parsers, version-selected, are cleaner.

**Why preserve voice messages (§1 #17)?** Vietnamese SMB workflows lean heavily on Zalo voice — short audio recordings instead of typed messages. Losing them = losing the conversational record. The `[voice message · <duration>s]` placeholder lets MM render an audio block; the audio file is the source of truth.

**Why stickers as custom emoji (§1 #19)?** Zalo's sticker primitive is heavy (animated, branded), MM has no direct equivalent. The custom-emoji namespace is the closest match; preserves visual identity without imposing animation infrastructure.

**Why deleted-message tombstones (§1 #21)?** Zalo's "recall" feature deletes the message from the conversation but leaves a visible "[recalled]" marker. Preserving this respects the source-of-truth state and matches compliance expectations (recalled-but-known-to-have-existed is a different state from never-existed).

**Why heuristic for mixed timestamp formats (§1 #23)?** Zalo's bundles inconsistently use seconds vs ms vs ISO-8601 depending on exporter version and locale. A static parse choice would silently mis-import ~10x range of dates. Magnitude-based heuristic is reliable for any timestamp in the human era (post-1970, pre-year-5000).

**Why preserve membership events (§1 #24)?** TASK-CHAT-008 mention resolution and TASK-CHAT-012 DSAR both need user-channel membership timeline. Zalo exports surface these as `<div class="event">` elements; capturing them mirrors the Slack importer's channelmember support.

**Why `--bundle-encoding` flag (§1 #25)?** Legacy Zalo Windows exports use cp1258 (Vietnamese variant of cp1250). Defaulting to UTF-8 would corrupt some VN characters silently; a flag lets operators specify when they have a legacy bundle, without breaking the modern default.

**Why `--strict` flag (§1 #28)?** Operators want two modes: production (best-effort, warnings logged) and CI (strict, any warning fails the build so we know when bundles regress). Single mode would force either silent acceptance or production-breaking strictness.

---

## §3 — API contract

```rust
// services/chat-importer/src/zalo/parse_html.rs
use scraper::{Html, Selector};

pub struct ZaloMessage {
    pub zalo_msg_id: String,
    pub user_id: String,
    pub ts_ms: i64,
    pub body: String,
    pub media_paths: Vec<String>,
}

pub fn parse_conversation_html(html: &str) -> Vec<ZaloMessage> {
    let doc = Html::parse_document(html);
    let msg_sel = Selector::parse("div.msg").unwrap();
    let body_sel = Selector::parse("span.body").unwrap();
    let mut out = Vec::new();
    for msg in doc.select(&msg_sel) {
        let id = msg.value().attr("data-msg-id").unwrap_or_default();
        let user = msg.value().attr("data-user-id").unwrap_or_default();
        let ts: i64 = msg.value().attr("data-ts").and_then(|s| s.parse().ok()).unwrap_or(0);
        let body = msg.select(&body_sel).next()
            .map(|n| n.text().collect::<String>())
            .map(|s| crate::zalo::normalize::nfc_emoji(&s))
            .unwrap_or_default();
        out.push(ZaloMessage { zalo_msg_id: id.into(), user_id: user.into(), ts_ms: ts, body, media_paths: vec![] });
    }
    out
}
```

```rust
// services/chat-importer/src/zalo/normalize.rs
use unicode_normalization::UnicodeNormalization;

pub fn nfc_emoji(s: &str) -> String {
    let nfc: String = s.nfc().collect();
    // Replace Zalo-specific emoji codes
    nfc.replace(":>>", "😄")
       .replace(":<<", "😢")
       .replace(":-D", "😀")
       // ~30 more from curated table
}
```

```rust
// services/chat-importer/src/main.rs (extended)
#[derive(Subcommand)]
enum Cmd {
    Slack(SlackArgs),
    Zalo(ZaloArgs),
    Abort { job_id: uuid::Uuid },
    Cleanup { job_id: uuid::Uuid, #[arg(long)] yes_i_know: bool },
}

#[derive(Parser, Debug)]
struct ZaloArgs {
    bundle: PathBuf,
    #[arg(long, env = "CYBEROS_TENANT_ID")] tenant: uuid::Uuid,
    #[arg(long)] resume: bool,
    #[arg(long)] dry_run: bool,
    #[arg(long)] strict: bool,
    #[arg(long, default_value = "utf-8")] bundle_encoding: String,
    #[arg(long)] workspace_id: Option<String>,
}

async fn cmd_zalo(args: ZaloArgs) -> ExitCode {
    crate::zalo::run_all(&args.bundle, args.tenant, args.into()).await
        .map(|_| ExitCode::Ok)
        .unwrap_or(ExitCode::InternalError)
}
```

### supported_versions.rs — version pinning

```rust
// services/chat-importer/src/zalo/supported_versions.rs
pub const SUPPORTED_EXPORT_VERSIONS: &[&str] = &[
    "1.0",  // pre-2023 Gen-1
    "1.1",  // 2022 Gen-1 minor
    "2.0",  // 2023 Gen-2 launch
    "2.1",  // 2024 Gen-2 minor
    "2.2",  // 2025 Gen-2 minor (current)
];

pub fn is_supported(version: &str) -> bool {
    SUPPORTED_EXPORT_VERSIONS.contains(&version)
}

pub fn schema_generation(version: &str) -> SchemaGen {
    let major: u32 = version.split('.').next().and_then(|s| s.parse().ok()).unwrap_or(0);
    match major {
        1 => SchemaGen::Gen1,
        2 => SchemaGen::Gen2,
        _ => SchemaGen::Unknown,
    }
}

pub enum SchemaGen { Gen1, Gen2, Unknown }
```

### parse_html.rs — Gen-1 + Gen-2 + reactions + voice + deleted

```rust
// services/chat-importer/src/zalo/parse_html.rs
use scraper::{Html, Selector};

#[derive(Debug, Clone)]
pub struct ZaloMessage {
    pub zalo_msg_id:        String,
    pub user_id:            String,
    pub ts_ms:              i64,
    pub body:               String,
    pub media_refs:         Vec<MediaRef>,
    pub voice_ref:          Option<VoiceRef>,
    pub sticker_ref:        Option<StickerRef>,
    pub parent_msg_id:      Option<String>,
    pub reactions:          Vec<ZaloReaction>,
    pub is_recalled:        bool,
}

#[derive(Debug, Clone)]
pub struct MediaRef { pub sha: String, pub mime: String, pub filename: Option<String> }

#[derive(Debug, Clone)]
pub struct VoiceRef { pub path: String, pub duration_sec: f64 }

#[derive(Debug, Clone)]
pub struct StickerRef { pub sticker_id: String }

#[derive(Debug, Clone)]
pub struct ZaloReaction { pub emoji: String, pub user_ids: Vec<String> }

#[derive(Debug, Clone)]
pub struct ZaloMembershipEvent {
    pub event_type: ZaloEventType,    // joined | left
    pub user_id:    String,
    pub ts_ms:      i64,
}

pub enum ZaloEventType { Joined, Left }

pub fn parse_conversation_html(
    html: &str,
    gen: SchemaGen,
) -> anyhow::Result<(Vec<ZaloMessage>, Vec<ZaloMembershipEvent>)> {
    match gen {
        SchemaGen::Gen1 => parse_gen1(html),
        SchemaGen::Gen2 => parse_gen2(html),
        SchemaGen::Unknown => Err(anyhow::anyhow!("unknown schema generation")),
    }
}

fn parse_gen1(html: &str) -> anyhow::Result<(Vec<ZaloMessage>, Vec<ZaloMembershipEvent>)> {
    let doc = Html::parse_document(html);
    let msg_sel      = Selector::parse("div.msg").unwrap();
    let body_sel     = Selector::parse("span.body").unwrap();
    let media_sel    = Selector::parse("div.media").unwrap();
    let voice_sel    = Selector::parse("div.voice").unwrap();
    let sticker_sel  = Selector::parse("div.sticker").unwrap();
    let reaction_sel = Selector::parse("div.reaction").unwrap();
    let event_sel    = Selector::parse("div.event").unwrap();

    let mut messages = Vec::new();
    let mut events = Vec::new();

    for msg in doc.select(&msg_sel) {
        let id   = msg.value().attr("data-msg-id").unwrap_or_default().to_owned();
        let user = msg.value().attr("data-user-id").unwrap_or_default().to_owned();
        let ts_raw = msg.value().attr("data-ts").unwrap_or("0");
        let ts_ms = normalise_ts(ts_raw)?;

        let is_recalled = msg.value().attr("class").map(|c| c.contains("deleted")).unwrap_or(false);
        let parent_msg_id = msg.value().attr("data-reply-to").map(String::from);

        let body = if is_recalled {
            "[message recalled]".to_owned()
        } else {
            msg.select(&body_sel).next()
               .map(|n| n.text().collect::<String>())
               .map(|s| crate::zalo::normalize::nfc_emoji(&s))
               .unwrap_or_default()
        };

        let media_refs: Vec<MediaRef> = msg.select(&media_sel).map(|m| MediaRef {
            sha: m.value().attr("data-sha").unwrap_or_default().to_owned(),
            mime: m.value().attr("data-mime").unwrap_or("application/octet-stream").to_owned(),
            filename: m.value().attr("data-filename").map(String::from),
        }).collect();

        let voice_ref: Option<VoiceRef> = msg.select(&voice_sel).next().map(|v| VoiceRef {
            path: v.value().attr("data-path").unwrap_or_default().to_owned(),
            duration_sec: v.value().attr("data-duration").and_then(|s| s.parse().ok()).unwrap_or(0.0),
        });

        let sticker_ref: Option<StickerRef> = msg.select(&sticker_sel).next().map(|s| StickerRef {
            sticker_id: s.value().attr("data-sticker-id").unwrap_or_default().to_owned(),
        });

        let reactions: Vec<ZaloReaction> = msg.select(&reaction_sel).map(|r| ZaloReaction {
            emoji: r.value().attr("data-emoji").unwrap_or_default().to_owned(),
            user_ids: r.value().attr("data-user-ids").unwrap_or_default()
                .split(',').filter(|s| !s.is_empty()).map(String::from).collect(),
        }).collect();

        let body = if let Some(s) = &sticker_ref {
            format!("{} :zalo-sticker-{}:", body, s.sticker_id)
        } else if let Some(v) = &voice_ref {
            format!("[voice message · {:.0}s]", v.duration_sec)
        } else {
            body
        };

        messages.push(ZaloMessage {
            zalo_msg_id: id, user_id: user, ts_ms,
            body, media_refs, voice_ref, sticker_ref,
            parent_msg_id, reactions, is_recalled,
        });
    }

    for ev in doc.select(&event_sel) {
        let ts = normalise_ts(ev.value().attr("data-ts").unwrap_or("0"))?;
        let user = ev.value().attr("data-user-id").unwrap_or_default().to_owned();
        let event_type = match ev.value().attr("data-type").unwrap_or("") {
            "user_joined" => ZaloEventType::Joined,
            "user_left"   => ZaloEventType::Left,
            other => { tracing::warn!(?other, "unknown event type"); continue; }
        };
        events.push(ZaloMembershipEvent { event_type, user_id: user, ts_ms: ts });
    }
    Ok((messages, events))
}

fn parse_gen2(html: &str) -> anyhow::Result<(Vec<ZaloMessage>, Vec<ZaloMembershipEvent>)> {
    // Same shape; selectors are <article data-zalo-msg-id> etc.
    // Implementation parallels parse_gen1 but with semantic-HTML selectors.
    // ... elided for length parity with gen-1 ...
    todo!()
}

fn normalise_ts(raw: &str) -> anyhow::Result<i64> {
    // ISO-8601?
    if raw.contains('T') {
        let dt: chrono::DateTime<chrono::Utc> = chrono::DateTime::parse_from_rfc3339(raw)?.into();
        return Ok(dt.timestamp_millis());
    }
    let n: i64 = raw.parse()?;
    Ok(match n {
        n if n < 10_000_000_000 => n * 1000,        // seconds → ms
        n if n < 100_000_000_000_000 => n,           // already ms
        _ => anyhow::bail!("timestamp {} out of human range; refusing", n),
    })
}
```

### normalize.rs — full emoji table + NFC + cp1258 decoder

```rust
// services/chat-importer/src/zalo/normalize.rs
use unicode_normalization::UnicodeNormalization;

static EMOJI_MAP: &[(&str, &str)] = &[
    (":>>",      "😄"),
    (":<<",      "😢"),
    (":-D",      "😀"),
    (":-)",      "🙂"),
    (":-(",      "🙁"),
    (":-P",      "😛"),
    (":-O",      "😮"),
    (";-)",      "😉"),
    (":heart:",  "❤️"),
    (":pray:",   "🙏"),
    (":fire:",   "🔥"),
    (":100:",    "💯"),
    (":vn:",     "🇻🇳"),
    ("(yes)",    "👍"),
    ("(no)",     "👎"),
    ("(ok)",     "👌"),
    ("(clap)",   "👏"),
    // ... ~30 more entries from the Zalo-2026 sticker set crosswalk
];

pub fn nfc_emoji(s: &str) -> String {
    let mut out: String = s.nfc().collect();
    for (k, v) in EMOJI_MAP { out = out.replace(k, v); }
    // Replace invalid Unicode with U+FFFD.
    out.chars().map(|c| if (c as u32) == 0 { '\u{FFFD}' } else { c }).collect()
}

pub fn decode(bytes: &[u8], encoding: &str) -> anyhow::Result<String> {
    use encoding_rs::*;
    let enc = match encoding {
        "utf-8" => UTF_8,
        "windows-1258" => WINDOWS_1258,
        other => anyhow::bail!("unsupported bundle encoding: {}", other),
    };
    let (cow, _, had_errors) = enc.decode(bytes);
    if had_errors {
        tracing::warn!("decoding errors with {}; some chars replaced", encoding);
    }
    Ok(cow.into_owned())
}
```

### metadata.json schema

```json
{
  "exporter_version": "2.2",
  "schema_version": "2.0",
  "exported_at": "2026-05-15T08:00:00Z",
  "workspace_id": "Z-acme",
  "conversations": [
    {
      "id": "conv-1",
      "type": "group" | "1on1" | "mpim",
      "name": "Sales VN",
      "participants": ["zu-001", "zu-002", "zu-003"]
    }
  ],
  "users": [
    { "id": "zu-001", "display_name": "Trần Văn A", "phone": "+84...", "email": null }
  ],
  "media": [
    { "sha": "abc123", "mime": "image/jpeg", "filename": "IMG_2026.jpg" }
  ],
  "stickers": [
    { "id": "100", "preview_path": "stickers/100.png" }
  ]
}
```

## §4 — Acceptance criteria

1. **CLI `zalo` subcommand exists**.
2. **HTML parser extracts messages** — fixture HTML with 10 messages → 10 ZaloMessage.
3. **NFC normalisation applied** — input "cà phê" (NFD) → output "cà phê" (NFC); same bytes.
4. **Emoji codes replaced** — ":>>" in body → "😄".
5. **Group conversation → private channel**.
6. **1-1 conversation → DM**.
7. **User without email → synthesised**.
8. **Message dedup by zalo_msg_id**.
9. **Checkpoint reused (source=zalo row in import_jobs)**.
10. **--dry-run no DB writes**.
11. **--resume picks up from last step**.
12. **memory audit emits with source=zalo**.
13. **RLS tenant isolation**.
14. **Synthetic test bundle imports end-to-end**.
15. **Media files uploaded to MM file store**.
16. **Unsupported exporter_version refused** — bundle with `exporter_version: "3.0"` → exit 1 with SEV-1 `chat.import_unsupported_zalo_version`; importer name + supported set printed.
17. **Gen-1 vs Gen-2 selected from metadata.json** — bundles with `schema_version: "1.0"` use parse_gen1; `"2.0"` use parse_gen2; verified by trace log + per-parser test fixtures.
18. **Reactions imported** — fixture msg with `<div class="reaction" data-emoji="thumbsup" data-user-ids="zu-001,zu-002">` → 2 MM reaction rows.
19. **Voice messages preserved** — fixture msg with `<div class="voice" data-path="audio/m1.m4a" data-duration="12.5">` → MM post body `[voice message · 13s]` + MM FileInfo for audio.
20. **Stickers mapped to custom emoji namespace** — fixture msg with sticker_id "100" → post body contains `:zalo-sticker-100:`.
21. **Replies map to MM root_id** — fixture with parent + 2 replies → both replies have `root_id` = parent's MM id; no orphan props.
22. **Recalled messages produce `[message recalled]` post** — fixture `<div class="msg deleted">` → MM post body `[message recalled]`, `delete_at > 0`, `props.zalo_was_recalled = true`.
23. **Filenames PII-redacted in memory audit** — file named `CV_TrinhThaiAnh_2026.pdf` → memory payload shows `CV_<NAME>_2026.pdf`; MM FileInfo retains original.
24. **Mixed timestamp formats handled** — fixture with `data-ts="1700000000"` (seconds), `data-ts="1700000000000"` (ms), `data-ts="2026-05-16T08:00:00Z"` (ISO); all three convert to correct ms.
25. **Membership events imported** — fixture with `<div class="event" data-type="user_joined">` → memory `chat.user_joined_channel` row.
26. **bundle_encoding flag honoured** — bundle with cp1258 + flag → VN characters render correctly; without flag → SEV-2 decoding warning logged.
27. **NFC validity verified** — fixture with malformed Unicode → MM post body contains U+FFFD replacement; SEV-2 warning audit.
28. **MPIM detected by participant count** — fixture conversation with 5 participants and `type: "1on1"` → SEV-2 warning + treated as MPIM (group channel); fixture with 2 participants and `type: "group"` → SEV-2 warning + treated as DM.
29. **--strict converts warnings to errors** — same fixture as #28 with `--strict` flag → exit 1.
30. **--strict + orphan reply → exit 1** — fixture with orphan reply + `--strict` → exit 1; without flag → top-level post with marker.

---

## §5 — Verification

Fixtures live in `services/chat-importer/tests/fixtures/zalo/`:
- `zalo-small.zip` — 3 users, 2 conversations, 10 messages.
- `zalo-gen1.zip` — Gen-1 schema sample.
- `zalo-gen2.zip` — Gen-2 schema sample.
- `zalo-mixed-timestamps.zip` — sec/ms/ISO timestamps in one bundle.
- `zalo-cp1258.zip` — legacy Windows-1258 encoded.
- `zalo-recalled.zip` — message with `<div class="msg deleted">`.
- `zalo-voice.zip` — message with voice attachment.
- `zalo-sticker.zip` — message with sticker.
- `zalo-reactions.zip` — message with 2-user reaction.
- `zalo-orphan-reply.zip` — reply whose parent is absent.
- `zalo-unsupported.zip` — exporter_version=3.0.

### AC #2 — parser extracts messages

```rust
#[test]
fn ac2_parses_msg_elements() {
    let html = r#"<div class="msg" data-msg-id="m1" data-user-id="u1" data-ts="1700000000000">
        <span class="body">Hello :>></span>
      </div>"#;
    let (msgs, _) = parse_conversation_html(html, SchemaGen::Gen1).unwrap();
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].zalo_msg_id, "m1");
    assert_eq!(msgs[0].body, "Hello 😄");
    assert_eq!(msgs[0].ts_ms, 1700000000000);
}
```

### AC #3 — NFC normalisation

```rust
#[test]
fn ac3_nfc_normalises_vietnamese() {
    // U+0061 U+0300 (a + combining grave) vs U+00E0 (à) and U+0065 U+0302 U+0309 (ê + hook)
    let nfd: String = "cà phê".chars().nfd().collect();
    let nfc = nfc_emoji(&nfd);
    // Round-trip: NFC of NFC is identity.
    assert_eq!(nfc, nfc_emoji(&nfc));
    // No combining diacritics in NFC output.
    for c in nfc.chars() {
        assert!(unicode_normalization::char::canonical_combining_class(c) == 0,
            "NFC output contains combining char: {:?}", c);
    }
}
```

### AC #4 — emoji codes replaced

```rust
#[rstest]
#[case(":>>",  "😄")]
#[case(":<<",  "😢")]
#[case(":-D",  "😀")]
#[case("(yes)","👍")]
#[case(":vn:", "🇻🇳")]
fn ac4_emoji_codes(#[case] input: &str, #[case] expected: &str) {
    let out = nfc_emoji(input);
    assert_eq!(out, expected);
}
```

### AC #5/#6 — conversation type mapping

```rust
#[rstest]
#[case("group",  vec!["zu-001","zu-002","zu-003"], "P")]
#[case("1on1",   vec!["zu-001","zu-002"],          "D")]
#[case("mpim",   vec!["zu-001","zu-002","zu-003"], "G")]
fn ac5_6_conversation_to_mm_channel(
    #[case] type_: &str,
    #[case] participants: Vec<&str>,
    #[case] expected: &str,
) {
    let conv = ZaloConversation { type_: type_.into(), participants: participants.iter().map(|s| s.to_string()).collect(), ..Default::default() };
    let mm = map_to_mm_channel(&conv);
    assert_eq!(mm.channel_type, expected);
}
```

### AC #16 — unsupported version refused

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac16_unsupported_version_refused() {
    let env = TestEnv::new().await;
    let r = run_all(env.fixture("zalo-unsupported.zip"), env.tenant_id(), Opts::default()).await;
    assert!(r.is_err());
    let row = env.memory.last_of_kind("chat.import_unsupported_zalo_version").await.unwrap();
    assert_eq!(row["severity"], "SEV-1");
    assert!(row["payload"]["got_version"].as_str().unwrap() == "3.0");
}
```

### AC #17 — Gen-1 vs Gen-2 selection

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac17_gen_selection_from_metadata() {
    let env = TestEnv::new().await;
    run_all(env.fixture("zalo-gen1.zip"), env.tenant_id(), Opts::default()).await.unwrap();
    assert!(env.has_log_entry_containing("parser=gen1"));

    run_all(env.fixture("zalo-gen2.zip"), env.tenant_id(), Opts::default()).await.unwrap();
    assert!(env.has_log_entry_containing("parser=gen2"));
}
```

### AC #18 — reactions

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac18_reactions_per_user() {
    let env = TestEnv::new().await;
    run_all(env.fixture("zalo-reactions.zip"), env.tenant_id(), Opts::default()).await.unwrap();
    let n = env.count_reactions_for_emoji("thumbsup").await;
    assert_eq!(n, 2);
}
```

### AC #19 — voice messages

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac19_voice_messages_preserved() {
    let env = TestEnv::new().await;
    run_all(env.fixture("zalo-voice.zip"), env.tenant_id(), Opts::default()).await.unwrap();
    let post = env.first_post_with_voice().await;
    assert!(post.message.starts_with("[voice message · "));
    assert!(post.message.ends_with("s]"));
    let fi = env.fileinfo_for_post(&post.id).await.unwrap();
    assert!(fi.mime_type.starts_with("audio/"));
}
```

### AC #20 — stickers

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac20_stickers_to_custom_emoji() {
    let env = TestEnv::new().await;
    run_all(env.fixture("zalo-sticker.zip"), env.tenant_id(), Opts::default()).await.unwrap();
    let post = env.find_post_containing("zalo-sticker").await;
    assert!(post.message.contains(":zalo-sticker-100:"));
}
```

### AC #21 — reply linking

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac21_replies_to_root_id() {
    let env = TestEnv::new().await;
    run_all(env.fixture("zalo-threads.zip"), env.tenant_id(), Opts::default()).await.unwrap();
    let parent = env.find_post_by_zalo_id("m-parent").await.unwrap();
    let replies = env.posts_with_root_id(&parent.id).await;
    assert_eq!(replies.len(), 2);
    for r in replies { assert!(!r.props.contains_key("zalo_reply_orphan")); }
}
```

### AC #22 — recalled tombstone

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac22_recalled_message_tombstone() {
    let env = TestEnv::new().await;
    run_all(env.fixture("zalo-recalled.zip"), env.tenant_id(), Opts::default()).await.unwrap();
    let post = env.find_post_by_zalo_id("m-recalled").await.unwrap();
    assert_eq!(post.message, "[message recalled]");
    assert!(post.delete_at > 0);
    assert_eq!(post.props["zalo_was_recalled"], serde_json::json!(true));
}
```

### AC #24 — timestamp formats

```rust
#[rstest]
#[case("1700000000",          1700000000000)]   // seconds
#[case("1700000000000",       1700000000000)]   // milliseconds
#[case("2023-11-14T22:13:20Z",1700000000000)]   // ISO-8601
fn ac24_timestamp_normalisation(#[case] input: &str, #[case] expected: i64) {
    assert_eq!(normalise_ts(input).unwrap(), expected);
}

#[test]
#[should_panic(expected = "out of human range")]
fn ac24_timestamp_out_of_range() {
    normalise_ts("9999999999999999999").unwrap();
}
```

### AC #25 — membership events

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac25_membership_events_imported() {
    let env = TestEnv::new().await;
    run_all(env.fixture("zalo-membership.zip"), env.tenant_id(), Opts::default()).await.unwrap();
    let row = env.memory.last_of_kind("chat.user_joined_channel").await.unwrap();
    assert!(row["payload"]["channel_id"].is_string());
    assert!(row["payload"]["user_id"].is_string());
}
```

### AC #26 — cp1258 decoding

```rust
#[test]
fn ac26_cp1258_decodes_vn_correctly() {
    let bytes = include_bytes!("../fixtures/zalo/cp1258-sample.bin");
    let decoded = decode(bytes, "windows-1258").unwrap();
    assert!(decoded.contains("Tiếng Việt"));
}
```

### AC #27 — invalid Unicode replaced

```rust
#[test]
fn ac27_invalid_unicode_replaced() {
    let body_with_null: String = ['a', '\0', 'b'].iter().collect();
    let out = nfc_emoji(&body_with_null);
    assert!(out.contains('\u{FFFD}'));
    assert!(!out.contains('\0'));
}
```

### AC #28/#29/#30 — strict mode

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac28_mpim_count_mismatch_warning() {
    let env = TestEnv::new().await;
    run_all(env.fixture("zalo-1on1-with-3-participants.zip"), env.tenant_id(), Opts::default()).await.unwrap();
    let row = env.memory.last_of_kind("chat.import_warning").await.unwrap();
    assert!(row["payload"]["reason"].as_str().unwrap().contains("participant_count_mismatch"));
}

#[tokio::test(flavor = "multi_thread")]
async fn ac29_strict_promotes_warning_to_error() {
    let env = TestEnv::new().await;
    let r = run_all(env.fixture("zalo-1on1-with-3-participants.zip"), env.tenant_id(),
        Opts { strict: true, ..Opts::default() }).await;
    assert!(r.is_err());
}

#[tokio::test(flavor = "multi_thread")]
async fn ac30_strict_with_orphan_reply_fails() {
    let env = TestEnv::new().await;
    let r = run_all(env.fixture("zalo-orphan-reply.zip"), env.tenant_id(),
        Opts { strict: true, ..Opts::default() }).await;
    assert!(r.is_err());
}
```

### AC #14 — end-to-end synthetic bundle

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac14_synthetic_bundle_end_to_end() {
    let env = TestEnv::new().await;
    run_all(env.fixture("zalo-small.zip"), env.tenant_id(), Opts::default()).await.unwrap();
    let job = env.fetch_job_by_zip(&env.fixture_sha("zalo-small.zip")).await.unwrap();
    assert_eq!(job.status, "completed");
    assert_eq!(job.source, "zalo");
    assert!(job.total_messages_imported >= 10);
}
```

### Property test — NFC idempotency

```rust
proptest! {
    #[test]
    fn nfc_is_idempotent(s in "\\PC{0,100}") {
        let once = nfc_emoji(&s);
        let twice = nfc_emoji(&once);
        prop_assert_eq!(once, twice);
    }
}
```

---

## §6 — Implementation skeleton

The Rust modules above are the skeleton. This section names the operational wiring:

### §6.1 — Reuse of TASK-CHAT-006 infrastructure

The Zalo importer is a sibling of the Slack importer, sharing:
- `import_jobs` table (with `source = 'zalo'`).
- Checkpoint resume + abort + cleanup semantics.
- memory audit row kinds (`chat.import_started`, `chat.import_step_completed`, etc.; `payload.source = "zalo"`).
- MM API client + rate-limiter.
- Fargate process model.

Zalo-specific code lives only in `src/zalo/`; the orchestrator and CLI shell are shared.

### §6.2 — Version-pinning workflow

When Zalo ships a new exporter version, the path is:
1. Operator captures a fresh bundle.
2. Engineer adds the version to `SUPPORTED_EXPORT_VERSIONS`.
3. If schema is unchanged → version bump is the only change.
4. If schema changed → add `parse_gen3` (or similar) + dispatch in `parse_conversation_html`.
5. Ship updated importer image; re-run any rejected imports.

### §6.3 — Schema-generation dispatch

`metadata.json::schema_version` is the dispatch key. We chose this over content-sniffing because:
- Sniffing is fragile (Zalo's two formats look similar to a regex check).
- Metadata is authoritative (Zalo writes it).
- Future generations can be added without retraining a sniffer.

### §6.4 — Timestamp heuristic boundary values

| Range | Interpretation |
|---|---|
| 0 ..< 10^10 | seconds (Unix epoch up to year 2286) |
| 10^10 ..< 10^14 | milliseconds (Unix epoch up to year 5138) |
| 10^14 ..< 10^17 | microseconds (extreme; treat as suspicious) |
| ≥ 10^17 | reject |
| string containing `T` | ISO-8601 |

The boundary values are documented inline in `normalise_ts` so future maintenance is informed.

### §6.5 — Voice-message MM rendering

The MM client renders `[voice message · 13s]` as plain text, but the FileInfo attachment surfaces the actual audio file in the message UI. Users see "Tap to listen" + duration. This is the MM-conventional way to surface non-image attachments.

### §6.6 — Sticker custom-emoji namespace

`zalo-sticker-<id>` is a reserved namespace. We do NOT auto-upload the sticker PNGs as MM custom emoji (would require system-admin permission and bumps MM's emoji count beyond practical limits). Operators wanting visual stickers run a separate one-time `cyberos-chat import zalo-stickers <bundle>` (slice 4+).

### §6.7 — Recalled-message preservation

Some compliance regimes require that recalled messages remain in the audit record (the FACT of recall is non-erasable; the CONTENT was redacted at recall time). Importing recalled messages as `[message recalled]` posts with `delete_at > 0` matches this expectation: MM shows them as deleted, but downstream TASK-CHAT-012 DSAR can surface them.

### §6.8 — Filename PII redaction

Zalo filenames carry PII more often than Slack filenames because Zalo's mobile-first UX encourages photo uploads with auto-generated names that include user identifiers. The redaction pipeline (TASK-MEMORY-111 ruleset) handles VN-specific names that the Slack-tuned ruleset might miss.

### §6.9 — Membership event timing

Zalo events can arrive in the export out of order (the export is per-conversation, but a user might have joined/left over time). We process them after step 4 (channels) but before step 5 (messages), so MM sees the membership in chronological order.

### §6.10 — Encoding-detection fallback

If `--bundle-encoding` is not provided AND `metadata.json::exporter_version` < "2.0", the importer attempts UTF-8 first; if VN tonal characters appear corrupted (heuristic: presence of `?` runs ≥ 3 chars in display_name fields), it falls back to cp1258 and warns. Strict mode (`--strict`) fails fast instead of falling back.

### §6.11 — Per-conversation HTML file size cap

A single conversation HTML file can be tens of MB if the conversation is long-lived. We stream the parse in chunks of 16MB to avoid loading the whole DOM. For >256MB conversations (rare), the importer recommends `--split-conversation <id>` (slice 4+) to break into per-day chunks.

### §6.12 — Failure routing matrix (mirrors TASK-CHAT-006 §6.12)

Same as TASK-CHAT-006 with adjustments:
- Step 1 (validate) additionally checks `exporter_version` and `schema_version`.
- Step 5 (messages) is sequential per conversation (not per channel; Zalo conversations map 1:1).
- Step 6 (verify) samples per TASK-CHAT-006 §1 #28.

---

## §7 — Dependencies

- **TASK-CHAT-006** — checkpoint table + step pattern reused.
- **TASK-CHAT-005** — bridge picks up imported posts.

---

## §8 — Example payloads

### `chat.import_step_completed` — Zalo messages step

```json
{
  "kind": "chat.import_step_completed",
  "ts_ns": 1747407250000000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "payload": {
    "job_id":      "01HVQX8ZG2K3R4TVA7P3WV5X8N",
    "source":      "zalo",
    "step":        4,
    "name":        "messages",
    "count":       2_841,
    "duration_ms": 12_340,
    "voice_messages":  47,
    "stickers":        12,
    "reactions":       186,
    "recalled":         3
  }
}
```

### `chat.import_unsupported_zalo_version`

```json
{
  "kind": "chat.import_unsupported_zalo_version",
  "ts_ns": 1747407100000000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "severity": "SEV-1",
  "payload": {
    "got_version":         "3.0",
    "supported_versions":  ["1.0","1.1","2.0","2.1","2.2"],
    "remediation":         "request a re-export at supported version OR update the importer"
  }
}
```

### `chat.import_timestamp_ambiguous`

```json
{
  "kind": "chat.import_timestamp_ambiguous",
  "ts_ns": 1747407225000000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "severity": "SEV-2",
  "payload": {
    "raw_value":       "9999999999999999999",
    "conv_id":         "conv-3",
    "remediation":     "request operator review of conversation timestamps"
  }
}
```

### `chat.import_warning` — participant-count mismatch

```json
{
  "kind": "chat.import_warning",
  "ts_ns": 1747407226000000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "payload": {
    "job_id":    "01HVQX8ZG2K3R4TVA7P3WV5X8N",
    "step":      3,
    "reason":    "participant_count_mismatch",
    "context": {
      "conv_id":              "conv-7",
      "metadata_type":        "1on1",
      "actual_participants":   5,
      "treated_as":           "mpim"
    }
  }
}
```

---

## §9 — Open questions

All resolved. Deferred:
- Zalo voice transcription (audio → text) — slice 4+.
- Zalo sticker mapping — slice 4+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Malformed HTML | scraper Err on parse_document | log + SEV-2 warning + skip conversation | Operator inspects bundle |
| Missing metadata.json | step 1 explicit check | exit 1; SEV-1 | Operator re-exports |
| metadata.json schema_version not in supported set | step 1 explicit check | exit 1; SEV-1 `chat.import_unsupported_zalo_version` | Add version to allowlist OR update parser |
| metadata.json missing schema_version | step 1 explicit check | exit 1; SEV-1 | Operator re-exports with newer Zalo version |
| Unknown event_type in `<div class="event">` | per-event warn | event skipped | None |
| User without display_name | fallback to "Zalo User <zalo_id>" | placeholder MM user | Operator post-import edit |
| User without email AND without phone | synthesised email `zalo-user-<id>@imported.cyberos.local` | MM user has placeholder email | None |
| Media file referenced in HTML but absent from bundle | step 5 logs + continues | message imported without attachment | Operator re-exports |
| Media file size > MM cap | MM 413 | SEV-2 warning; file skipped | Operator bumps tenant cap |
| Voice message missing from `audio/` directory | step 5 logs | post body still has placeholder; no MM FileInfo | Operator re-exports |
| Voice message duration is 0 or negative | log; default to 0 | post body shows `[voice message · 0s]` | None |
| Sticker referenced but `stickers/<id>.png` absent | log; post body still has `:zalo-sticker-<id>:` | rendered as text | Operator imports stickers separately |
| Emoji code unmapped (not in EMOJI_MAP) | unchanged in body | text-as-typed | Operator extends map |
| Same bundle re-imported | checkpoint detects | exits 0 "already imported" | None |
| NFC normalisation produces invalid Unicode | char-by-char replace with U+FFFD | message body sanitised + SEV-2 warning | None |
| Mixed conversation types in bundle | per-conv detection | each handled | None |
| Large bundle (> 1GB) | streaming zip | None visible | None |
| Single conversation HTML > 256MB | bounded read refuses | exit 1; suggest --split-conversation (slice 4+) | Operator splits manually |
| Zalo timestamp out of human range (≥ 10^17) | normalise_ts rejects | SEV-2 `chat.import_timestamp_ambiguous` | Operator investigates source |
| Zalo timestamp negative | normalise_ts rejects | SEV-2 | Operator investigates |
| Group with 0 members | skip channel | logged | Operator investigates |
| Group with 1 member | imported as DM (degenerate case) | works as 1-1 chat with self | None |
| Recalled message references parent that's also recalled | both imported as tombstones | thread chain preserved | None |
| Reply parent absent (orphan) | top-level + `props.zalo_reply_orphan=true` | None | None |
| Reply parent appears later in same file | step 4 inserts in document order; step-end pass relinks | None | None |
| Reaction with empty user_ids | reaction skipped | None | None |
| Reaction with user_id pointing to unimported user | reaction skipped + warn | None | Operator re-imports |
| HTML schema change in future Zalo export | parse_genN fails OR version pinning rejects | SEV-1; operator updates parser | Update SUPPORTED_EXPORT_VERSIONS + add parser |
| Bundle encoding mismatch (cp1258 read as UTF-8) | corrupted VN chars in output | SEV-2 warning; operator should use --bundle-encoding | None |
| `--strict` + any warning | exit 1; warning audit row emitted | None | Operator addresses warning OR removes --strict |
| Bundle .zip integrity check fails | CRC mismatch | exit 1; SEV-1 | Operator re-exports |
| Two operators run --abort against same job concurrently | UPDATE serialises | one wins; second no-op | None |
| Encoding detection heuristic false-positive (UTF-8 mistaken for cp1258) | fallback runs | display_name might become more corrupted | Operator passes --bundle-encoding explicitly |
| metadata.json::conversations[] empty | step 3 no-op | exit 0; SEV-3 warning | Operator inspects bundle |
| metadata.json::users[] empty | step 2 no-op | exit 0; SEV-3 | Operator inspects bundle |
| metadata.json::media[] entry without sha | step 5 cannot dedup | sha computed from bytes; warn | None |
| Sticker PNG corrupted | MM upload fails | log; sticker not uploaded; body still has emoji code | None |
| Voice m4a corrupted | MM upload fails | log; body has placeholder; no audio | None |
| Membership event for user_id not in users.json | event skipped + warn | None | Operator investigates |
| Two conversations claim the same conv_id | second skipped | warn | Operator investigates |
| Audio duration > 10 minutes | imported; MM stores | None | None |
| Sticker pack has cross-references (one sticker depends on another) | each imported independently | None | None |
| Bundle missing `media/` directory but messages reference media | step 5 skips | warn | Operator re-exports |
| `import_jobs` row for this zip+tenant exists in `failed` state without --resume | start_or_resume errors with hint | exit 1 with instructive message | Operator uses --resume |

---

## §11 — Implementation notes

- `scraper` crate is HTML5-tolerant; handles Zalo's loose HTML. We considered `html5ever` directly for lower-level control but `scraper`'s ergonomic API outweighed any perf concerns for our bundle sizes.
- `unicode-normalization` crate provides `nfc()` iterator. NFC is the canonical form for Vietnamese text — TASK-CHAT-004's bigram tokenizer assumes NFC. Without normalisation, a search for "cà phê" wouldn't match a NFD-encoded message containing the same word.
- Emoji code table lives in `normalize.rs` as `static MAP: &[(&str, &str)]`. We curated the ~30-entry list from a sample of 2k real Vietnamese SMB messages; covers >95% of in-the-wild Zalo emoji codes.
- Synthesised email format `zalo-user-<zalo_id>@imported.cyberos.local` — operator-recognisable. The `.local` TLD is reserved (RFC 6762) so the synthesised email never collides with a real domain.
- Files step downloads from bundle's `media/` folder (no external HTTP). This is unlike Slack which requires fetching from Slack's CDN; Zalo's manual-export model puts everything in the zip.
- Replies map to MM root_id (we initially said "Threads skipped: Zalo doesn't have thread primitive" — this was incorrect; Zalo does have a "reply to message" arrow primitive that exports as `data-reply-to`).
- The Gen-1 vs Gen-2 split is a hard discontinuity: Zalo's Gen-2 export is semantic HTML with `<article>` tags; Gen-1 is `<div>` soup. A unified parser would be hairy; separate parsers are cleaner.
- Sticker handling intentionally stops short of uploading PNGs as MM custom emoji — the namespace explosion would be unmanageable (Zalo has 10k+ stickers; uploading all of them is operator policy, not importer default).
- `[message recalled]` placeholder text was chosen to match Zalo's own UX wording. We considered preserving the original content (some legal regimes allow this) but defaulted to redaction; an operator who needs the original content can scrape the source bundle (where the recalled message body MAY still be present, depending on Zalo's export behaviour).
- VN-specific PII patterns (full names with diacritics, +84 phone numbers, ID-card numbers) are tuned in `cyberos-memory-pii::vn-rules`. The Slack importer also uses these rules but Zalo bundles hit them harder due to mobile-first usage patterns.
- We chose to put the Zalo importer in the same `chat-importer` crate as the Slack importer rather than separate crates because: (a) shared checkpoint + audit infrastructure, (b) operator runs same binary with subcommand, (c) avoids duplicating MM API client boilerplate.
- The `--bundle-encoding` flag is the explicit escape hatch for legacy bundles; the auto-detection in §6.10 is a convenience that operators can override.
- We considered making encoding detection automatic-only (no flag) but operators handling truly mixed bundles wanted explicit control. The flag is opt-in for safety.
- Per-conversation parallelism is bounded by the MM API rate limit, same as Slack importer. Step 4 (messages) for Zalo is sequential per conversation, not per channel.
- The `--strict` flag exists specifically for CI gates: bundles that import cleanly in strict mode are guaranteed to have no orphans, no missing media, no encoding warnings. Operators promoting from staging to prod run `--strict` first.
- We do NOT auto-create MM channels for empty Zalo conversations because they're often noise (test conversations, abandoned threads). Operators can manually create those if needed.
- The encoding-detection heuristic (presence of `?` runs ≥ 3 chars) is conservative — false positives mean "we tried cp1258 unnecessarily but it round-trips OK"; false negatives mean "we shipped corrupted display_names with UTF-8 decoding."
- Operators looking for "complete restoration of Zalo history" are warned in docs that some Zalo features (voice transcriptions, sticker animations, location pins) are not preserved. The MVP scope is text + media + reactions + replies + recall + membership; the rest is slice 4+.
- We chose UUID-v7 for `import_jobs.id` so per-tenant import lists sort chronologically — same rationale as TASK-CHAT-006.
- The `chat.import_unsupported_zalo_version` audit row severity is SEV-1 because an import that silently fails (importer rejects bundle, operator doesn't notice) leads to the customer missing months of Zalo history. Loud failure is mandatory.
- Why we don't ship a `--force-version` override flag: forcing parser version against a bundle's claimed version is "we trust the parser more than we trust the export." That's almost never correct — the export carries semantic meaning the parser may miss. If a version genuinely needs to be added to the supported set, that's a code change with audit + review, not a flag.
- The timestamp normalisation boundary (10^10 = year 2286 in seconds; 10^14 = year 5138 in ms) was chosen with future-decade margin so the heuristic doesn't break before we've all retired.
- Voice message duration is captured for UX but not used for indexing or search — MM's audio player needs it to render the timeline. We trust Zalo's claimed duration rather than re-computing from the m4a header (would require a slow audio parser).
- The Zalo importer's CLI subcommand shares `--workspace-id` semantics with Slack: a single tenant can import from multiple Zalo workspaces (acquisitions). Each import is checkpointed independently.
- For long-running imports, we considered emitting a per-conversation progress row (e.g. `chat.import_conversation_completed`) but the heartbeat from TASK-CHAT-005 plus the per-step `chat.import_step_completed` already covers operator visibility.

---

*End of TASK-CHAT-007.*
