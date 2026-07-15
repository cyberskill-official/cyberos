---
id: TASK-CHAT-006
title: "Slack import — `cyberos-chat import slack` with 8-step idempotent checkpoint-driven workflow"
module: CHAT
priority: MUST
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
related_tasks: [TASK-CHAT-005, TASK-CHAT-007, TASK-CHAT-010]
depends_on: [TASK-CHAT-005]
blocks: [TASK-CHAT-010, TASK-CHAT-007]

source_pages:
  - website/docs/modules/chat.html#slack-import
source_decisions:
  - DEC-470 (8 explicit steps with checkpoint table; resume from last successful step)
  - DEC-471 (Slack zip export format; supports both legacy + new export formats)
  - DEC-472 (idempotent: same export zip + same target → no duplicates)

language: rust 1.81
service: cyberos/services/chat-importer/
new_files:
  - services/chat-importer/Cargo.toml
  - services/chat-importer/src/main.rs
  - services/chat-importer/src/slack/mod.rs
  - services/chat-importer/src/slack/parse.rs
  - services/chat-importer/src/slack/steps.rs
  - services/chat-importer/src/slack/checkpoint.rs
  - services/chat-importer/tests/slack_test.rs
modified_files:
  - services/chat/sql/init-import-checkpoint.sql
allowed_tools:
  - file_read: services/chat-importer/**
  - file_write: services/chat-importer/{src,tests}/**, services/chat/sql/**
  - bash: cd services/chat-importer && cargo test slack
disallowed_tools:
  - skip checkpoint persistence (per DEC-470 — resume capability required)
  - write Slack files into memory directly (per DEC-470 — round-trip through chat → bridge)

effort_hours: 12
subtasks:
  - "0.5h: init-import-checkpoint.sql migration (import_jobs table)"
  - "0.5h: Cargo.toml deps (zip, serde_json, sha2)"
  - "0.5h: main.rs — clap subcommand `slack <zip-path> --tenant <id> [--resume]`"
  - "1.5h: parse.rs — zip extraction + Slack export schema parsing (channels.json, users.json, msgs/*.json)"
  - "2.0h: 8 steps: validate / users / channels / channel_members / messages / threads / files / verify"
  - "1.0h: checkpoint.rs — atomic step completion writes to import_jobs row"
  - "1.0h: resume logic — read last_step_completed; skip ahead"
  - "1.5h: dedup keys (slack_user_id → MM user; slack_channel_id → MM channel)"
  - "0.5h: memory audit row 'chat.import_started', 'chat.import_step_completed', 'chat.import_finished'"
  - "1.5h: slack_test.rs — happy import + resume from step 4 + duplicate import = noop"
  - "0.5h: progress reporting via stderr (step N/8: <message>)"
  - "0.5h: --dry-run mode (no DB writes; report what would happen)"
risk_if_skipped: "Without import, new tenants must adopt Mattermost greenfield — abandoning years of Slack history. Without checkpoints, 100K-message imports fail at step 7 = restart from 0. Without idempotency, retry-on-error duplicates entire workspace. Without progress reports, operators wait blind. Without dry-run, ops can't preview the destructive operation."
---

## §1 — Description (BCP-14 normative)

The Slack importer **MUST** ingest a Slack export zip via an 8-step idempotent checkpoint-driven workflow. The contract:

1. **MUST** expose CLI `cyberos-chat import slack <zip-path> --tenant <id> [--resume] [--dry-run]`.
2. **MUST** define `import_jobs` checkpoint table: `(id UUID PK, source TEXT='slack', tenant_id UUID, zip_sha256 TEXT, last_step_completed INT, total_messages_imported INT, started_at, finished_at, error_message TEXT, tenant_id_meta)`. Composite key `(zip_sha256, tenant_id)` enforces idempotency.
3. **MUST** execute 8 explicit steps in order:
    1. **Validate**: zip integrity, expected files present.
    2. **Users**: parse users.json → JIT-provision Mattermost users (TASK-CHAT-002 pattern; idempotent on slack_user_id).
    3. **Channels**: parse channels.json → create Mattermost channels (idempotent on slack_channel_id).
    4. **Channel Members**: parse channel/members.json → add MM users to MM channels.
    5. **Messages**: parse channels/<name>/*.json → insert MM posts in chronological order.
    6. **Threads**: link thread replies via `props.root_post_slack_ts` lookup.
    7. **Files**: download Slack file attachments → upload to MM file store.
    8. **Verify**: count posts; compare to expected; emit final audit row.
4. **MUST** checkpoint after EACH step: write `last_step_completed` atomically.
5. **MUST** support resume: `--resume` reads last checkpoint and skips to step N+1.
6. **MUST** dedup at every step:
    - Users: lookup `users WHERE props.slack_user_id = <id>`; reuse if exists.
    - Channels: lookup by `props.slack_channel_id`.
    - Messages: dedup by `(channel_id, ts)` — Slack timestamps are unique per channel.
7. **MUST** emit memory audit rows:
    - `chat.import_started` at step 1.
    - `chat.import_step_completed` per step (payload: step N, message_count, duration_ms).
    - `chat.import_finished` at step 8 (payload: total_messages, total_users, total_channels, duration_total_ms).
8. **MUST** report progress to stderr: `[step 5/8] importing messages: 12,345 / 100,000 (12%)`.
9. **MUST** support `--dry-run`: parse zip + report what would happen; NO DB writes.
10. **MUST** be idempotent: re-run with same zip_sha256 + same tenant → CLI exits 0 with message "already imported"; checkpoint row present and finished.
11. **MUST** fail fast on permanent errors (corrupt zip, schema unrecognised); retry transient (DB lock, Mattermost API rate limit).
12. **MUST** RLS-enforce tenant scope.
13. **MUST** emit OTel metrics:
    - `chat_import_messages_total{source=slack}` (counter).
    - `chat_import_duration_seconds{source=slack, step}` (histogram).
    - `chat_import_failures_total{source=slack, step}` (counter).
14. **MUST** preserve original message timestamps: Mattermost `posts.create_at = floor(slack_ts × 1000)` (Slack uses float seconds; MM uses integer milliseconds). Insert in chronological order per channel so MM indexes remain efficient.
15. **MUST** preserve thread structure: a Slack reply has `thread_ts` pointing to the parent's `ts`. Step 6 resolves these into Mattermost `root_id` references after step 5 has populated the post table. Replies whose parent is missing (export gap) are logged + emitted as top-level posts with `props.slack_thread_orphan = true`.
16. **MUST** dedup file attachments by Slack `file.id`: two posts referencing the same `file.id` MUST point at one MM `FileInfo` row. The dedup is by `(slack_workspace_id, slack_file_id)` because Slack file IDs are workspace-scoped.
17. **MUST** redact PII from file metadata before upload: filenames + comments + initial_comment in the Slack file payload are routed through TASK-MEMORY-111 redaction. Original content is preserved in the MM FileInfo blob; redacted form goes into the memory audit trail.
18. **MUST** map Slack channel types to Mattermost types:
    - Slack `is_general=true` → MM `channel_type=O` named `town-square` (or `general` if `town-square` taken).
    - Slack `is_private=true` → MM `channel_type=P`.
    - Slack `is_im=true` → MM `channel_type=D` (direct).
    - Slack `is_mpim=true` (multi-person IM) → MM `channel_type=G` (group).
    - Slack `is_archived=true` → MM channel created then `delete_at` set.
19. **MUST** map Slack reactions to Mattermost reactions: `reactions[].users[]` enumerated; each becomes one MM `reactions` row. Emoji name carries over verbatim; custom emoji that don't exist in MM map to `:question:` with `props.original_emoji_name`.
20. **MUST** preserve pinned messages: Slack `pinned_to:[channel_id, ...]` → MM `posts.is_pinned = true`.
21. **MUST** preserve edited messages: Slack message edit history is opaque in export (only final text shipped). Insert final text as the only version; emit memory audit `chat.import_warning` with `reason="edit_history_unavailable"` for each edited message.
22. **MUST** mark imported posts: every imported post carries `props.cyberos_imported = true` and `props.cyberos_source = "slack"` and `props.slack_ts = "<original>"` and `props.slack_workspace_id = "<id>"`. Downstream (TASK-CHAT-005 bridge) preserves these in memory payload.
23. **MUST** cap parallelism per step:
    - Step 5 (messages): 1 worker (sequential to preserve order).
    - Step 7 (files): 10 parallel downloads, configurable via `--file-parallelism`.
    - Step 2/3/4: 4 parallel API calls to MM.
24. **MUST** respect Mattermost rate limits: detect 429 response; back off with exponential `min(2^attempt × 100ms, 30s)`; honour `Retry-After` header when present. Failure after 8 retries → permanent error.
25. **MUST** validate zip integrity before step 1 succeeds: every member file checksum matches the zip CRC32; file-list contains `channels.json` + `users.json`; users.json parses as JSON array.
26. **MUST** support `--abort <job_id>` to cleanly cancel a running import: set `cancellation_requested = true` in `import_jobs`; current step finishes its in-flight unit then exits; checkpoint preserved.
27. **MUST** support `--cleanup <job_id>` to remove all posts/users/channels created by a specific job: cascades via the `import_job_id` foreign key on `posts.props`, `users.props`, `channels.props`. Requires the job to be in `aborted` state.
28. **MUST** sample post-import: after step 8, randomly sample 100 posts; verify their content matches the source zip (sha256 of `message` text). Mismatch → SEV-1 `chat.import_verification_failed` audit; job marked `verification_failed`.
29. **MUST** record per-step timing AND row counts in `import_jobs.step_metrics` JSONB column for operator post-mortem.
30. **MUST** preserve Slack workspace context: `props.slack_workspace_id` on every imported entity; allows downstream tasks to scope queries to a specific Slack history (e.g. "messages from the Sales-team Slack only").

---

## §2 — Why this design (rationale for humans)

**Why 8 steps (DEC-470)?** Each step has a clean failure boundary. A failure in step 5 (messages) doesn't corrupt step 3 (channels). Operators see exactly where it broke.

**Why checkpoint after each step (§1 #4)?** Resume capability. Without it, a step-7 failure restarts from step 1 — wastes hours on 100K-message imports.

**Why dedup at every step (§1 #6)?** Idempotency requires that no step double-applies. Lookup-first means duplicates short-circuit.

**Why zip_sha256 in primary key (§1 #2)?** Same zip = same import. If operator runs same import twice, second is no-op (with operator-visible message).

**Why dry-run (§1 #9)?** Destructive operations (creates many DB rows) deserve preview. Operator confirms before committing.

**Why files in step 7 (§1 #3)?** Files can be skipped without breaking message integrity; isolating them last lets operators run "everything except files" for quick smoke test.

**Why preserve original timestamps (§1 #14)?** Operators searching "what was said on 2023-08-12" must find the original date, not the import date. Mattermost's index on `create_at` works only if we honour the source timestamp. The float-to-int conversion uses `floor` (not round) because Slack and MM treat fractional seconds differently and rounding can create duplicate (channel_id, create_at) collisions.

**Why orphan thread handling (§1 #15)?** Slack exports can be partial (a channel exported without its archived parent message). Refusing to import the reply loses data; promoting it to a top-level post preserves it with a marker for forensic clarity.

**Why dedup files by (workspace, file_id) (§1 #16)?** Slack file_ids are workspace-scoped — two Slack workspaces can both have a file `F0ABC`. Without the workspace qualifier, importing two workspaces into one tenant would collide.

**Why redact file metadata (§1 #17)?** Filenames often contain PII (`Resume - Trinh Thai Anh.pdf`, `Salary letter for Alice.pdf`). The MM file blob preserves originals (operator-recoverable); the memory audit trail gets the redacted form so downstream consumers don't see raw PII.

**Why explicit channel-type mapping (§1 #18)?** Slack and MM use different identifiers (`is_general` vs `town-square` naming); a naive name-match would silently misclassify private channels as public. Explicit mapping makes the contract auditable.

**Why preserve reactions (§1 #19)?** Reactions are first-class data for analysts ("which messages got 👍 from leadership?"). The `:question:` fallback for unmapped custom emoji preserves the existence of the reaction without lying about the emoji.

**Why mark imported posts with `cyberos_imported = true` (§1 #22)?** Downstream tasks (TASK-CHAT-008 mentions, TASK-CHAT-005 bridge memory payload) MUST distinguish "this post is from history" vs "this post is live." Live posts get push notifications; historical posts do not.

**Why per-step parallelism caps (§1 #23)?** Messages MUST be inserted sequentially per channel because MM's `create_at` ordering is rebuilt from insertion order under tie. Files can be parallelised (idempotent uploads). API calls to MM have a 4-parallelism sweet spot before MM rate-limits.

**Why honour `Retry-After` header (§1 #24)?** MM ships a hot rate-limit value via the header. Falling back to fixed backoff means we either retry too soon (re-trip the limit) or too late (slow import). Header > exponential.

**Why `--abort` + `--cleanup` (§1 #26-27)?** Operators discovering a misconfigured import (wrong tenant, wrong zip) need surgery. Without --cleanup, undoing requires manual DELETE statements across 5 tables; with it, a single command rolls back.

**Why post-import sampling (§1 #28)?** Verification step (step 8) currently counts rows — that catches whole-message loss but not silent corruption (e.g. encoding bug that drops Vietnamese tonal marks). SHA-256-on-content sampling catches the corruption.

**Why workspace context (§1 #30)?** A tenant might import multiple Slack workspaces (acquisition, merger). Without workspace_id, all messages collapse into one undifferentiated history.

---

## §3 — API contract

### Schema — `import_jobs`

```sql
-- services/chat/sql/init-import-checkpoint.sql
CREATE TABLE import_jobs (
    id                       UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source                   TEXT NOT NULL CHECK (source IN ('slack','zalo')),
    tenant_id                UUID NOT NULL,
    zip_sha256               TEXT NOT NULL,
    slack_workspace_id       TEXT,                -- nullable; only for slack
    last_step_completed      INT NOT NULL DEFAULT 0,
    cancellation_requested   BOOLEAN NOT NULL DEFAULT false,
    status                   TEXT NOT NULL DEFAULT 'running'
                             CHECK (status IN ('running','completed','aborted','failed','verification_failed')),
    total_users_imported     INT NOT NULL DEFAULT 0,
    total_channels_imported  INT NOT NULL DEFAULT 0,
    total_messages_imported  INT NOT NULL DEFAULT 0,
    total_files_imported     INT NOT NULL DEFAULT 0,
    started_at               TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    finished_at              TIMESTAMPTZ,
    error_message            TEXT,
    step_metrics             JSONB NOT NULL DEFAULT '{}'::jsonb,
    verification_sha256_sample TEXT,  -- sha of the 100-sample manifest
    UNIQUE (zip_sha256, tenant_id, source)
);

CREATE INDEX import_jobs_status_idx ON import_jobs (status, started_at);
CREATE INDEX import_jobs_tenant_idx ON import_jobs (tenant_id, source);

-- Append-only protection: only the importer service role can UPDATE
-- the cancellation_requested flag and the status field.
REVOKE UPDATE, DELETE ON import_jobs FROM cyberos_app;
GRANT  INSERT, SELECT, UPDATE ON import_jobs TO cyberos_importer;
```

### CLI — `cyberos-chat import slack`

```rust
// services/chat-importer/src/main.rs
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "cyberos-chat-importer")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Import a Slack export zip into the given tenant.
    Slack {
        zip_path: PathBuf,
        #[arg(long, env = "CYBEROS_TENANT_ID")]
        tenant: uuid::Uuid,
        #[arg(long)]
        resume: bool,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        skip_files: bool,
        #[arg(long, default_value_t = 10)]
        file_parallelism: usize,
        #[arg(long, default_value_t = 8)]
        max_retries: u32,
        #[arg(long)]
        workspace_id: Option<String>,
    },
    /// Abort a running import job.
    Abort { job_id: uuid::Uuid },
    /// Remove all entities created by an aborted job.
    Cleanup { job_id: uuid::Uuid, #[arg(long)] yes_i_know: bool },
    /// Show the status of a job.
    Status { job_id: uuid::Uuid },
    /// List all jobs for a tenant.
    List { tenant: uuid::Uuid },
}
```

### parse.rs — Slack export schema

```rust
// services/chat-importer/src/slack/parse.rs

#[derive(serde::Deserialize, Debug, Clone)]
pub struct SlackUser {
    pub id:             String,
    pub team_id:        Option<String>,
    pub name:           String,
    pub real_name:      Option<String>,
    pub deleted:        Option<bool>,
    pub is_bot:         Option<bool>,
    pub is_admin:       Option<bool>,
    pub profile:        SlackUserProfile,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct SlackUserProfile {
    pub email:         Option<String>,
    pub display_name:  Option<String>,
    pub image_192:     Option<String>,
    pub title:         Option<String>,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct SlackChannel {
    pub id:           String,
    pub name:         String,
    pub created:      i64,
    pub is_archived:  Option<bool>,
    pub is_general:   Option<bool>,
    pub is_private:   Option<bool>,
    pub is_im:        Option<bool>,
    pub is_mpim:      Option<bool>,
    pub members:      Vec<String>,
    pub topic:        Option<SlackTextBlob>,
    pub purpose:      Option<SlackTextBlob>,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct SlackTextBlob {
    pub value:    String,
    pub creator:  Option<String>,
    pub last_set: Option<i64>,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct SlackMessage {
    #[serde(rename = "type")]
    pub type_:        String,
    pub user:         Option<String>,
    pub bot_id:       Option<String>,
    pub text:         String,
    pub ts:           String,                 // float-seconds-as-string
    pub thread_ts:    Option<String>,         // parent ts if reply
    pub reply_count:  Option<i32>,
    pub edited:       Option<SlackEditedRef>,
    pub reactions:    Option<Vec<SlackReaction>>,
    pub files:        Option<Vec<SlackFileRef>>,
    pub pinned_to:    Option<Vec<String>>,
    pub subtype:      Option<String>,         // e.g. "channel_join", "bot_message"
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct SlackEditedRef { pub user: String, pub ts: String }

#[derive(serde::Deserialize, Debug, Clone)]
pub struct SlackReaction {
    pub name:  String,
    pub users: Vec<String>,
    pub count: i32,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct SlackFileRef {
    pub id:          String,
    pub name:        Option<String>,
    pub title:       Option<String>,
    pub mimetype:    Option<String>,
    pub url_private: Option<String>,
    pub size:        Option<i64>,
    pub initial_comment: Option<SlackInitialComment>,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct SlackInitialComment { pub comment: String }

/// Convert a Slack ts (e.g. "1620000000.123456") into MM milliseconds.
/// Returns floor of (seconds * 1000) — never rounding.
pub fn slack_ts_to_ms(ts: &str) -> Result<i64, std::num::ParseFloatError> {
    let f: f64 = ts.parse()?;
    Ok((f * 1000.0).floor() as i64)
}

/// Read a file from the zip into a Vec<u8>, with bounded memory.
pub fn zip_read_file(zip: &mut zip::ZipArchive<File>, name: &str) -> anyhow::Result<Vec<u8>> {
    let mut entry = zip.by_name(name)?;
    if entry.size() > 256 * 1024 * 1024 {
        anyhow::bail!("file {} > 256MB; refusing to load into memory", name);
    }
    let mut buf = Vec::with_capacity(entry.size() as usize);
    std::io::copy(&mut entry, &mut buf)?;
    Ok(buf)
}
```

### checkpoint.rs — atomic step completion

```rust
// services/chat-importer/src/slack/checkpoint.rs
use sqlx::{PgPool, postgres::PgRow, Row};

#[derive(Debug, Clone)]
pub struct ImportJob {
    pub id:                      uuid::Uuid,
    pub tenant_id:               uuid::Uuid,
    pub zip_sha256:              String,
    pub last_step_completed:     i32,
    pub status:                  String,
    pub cancellation_requested:  bool,
    pub finished_at:             Option<chrono::DateTime<chrono::Utc>>,
    pub step_metrics:            serde_json::Value,
}

pub async fn start_or_resume(
    pool: &PgPool,
    zip_sha: &str,
    tenant_id: uuid::Uuid,
    workspace_id: Option<&str>,
    resume: bool,
) -> anyhow::Result<ImportJob> {
    let existing = sqlx::query_as::<_, ImportJob>(
        "SELECT * FROM import_jobs WHERE zip_sha256 = $1 AND tenant_id = $2 AND source = 'slack'"
    ).bind(zip_sha).bind(tenant_id).fetch_optional(pool).await?;

    match (existing, resume) {
        (Some(j), true)  if j.status == "running" || j.status == "failed" => Ok(j),
        (Some(j), _)     if j.status == "completed" => Ok(j),  // caller handles "already imported"
        (Some(j), false) => Err(anyhow::anyhow!(
            "job {} already exists; use --resume to continue or --abort to cancel",
            j.id
        )),
        (None, _)        => Self::insert_new(pool, zip_sha, tenant_id, workspace_id).await,
    }
}

async fn insert_new(
    pool: &PgPool,
    zip_sha: &str,
    tenant_id: uuid::Uuid,
    workspace_id: Option<&str>,
) -> anyhow::Result<ImportJob> {
    let row = sqlx::query_as::<_, ImportJob>(
        "INSERT INTO import_jobs (source, tenant_id, zip_sha256, slack_workspace_id)
              VALUES ('slack', $1, $2, $3)
         RETURNING *"
    ).bind(tenant_id).bind(zip_sha).bind(workspace_id)
     .fetch_one(pool).await?;
    Ok(row)
}

pub async fn complete_step(
    pool: &PgPool,
    job_id: uuid::Uuid,
    step: i32,
    metrics: serde_json::Value,
) -> anyhow::Result<()> {
    sqlx::query(
        "UPDATE import_jobs
            SET last_step_completed = $1,
                step_metrics        = step_metrics || $2::jsonb
          WHERE id = $3"
    ).bind(step).bind(metrics).bind(job_id).execute(pool).await?;
    Ok(())
}

pub async fn check_cancellation(pool: &PgPool, job_id: uuid::Uuid) -> anyhow::Result<bool> {
    let row = sqlx::query("SELECT cancellation_requested FROM import_jobs WHERE id = $1")
        .bind(job_id).fetch_one(pool).await?;
    Ok(row.get(0))
}

pub async fn finish(pool: &PgPool, job_id: uuid::Uuid, counts: ImportCounts, sample_sha: &str) -> anyhow::Result<()> {
    sqlx::query(
        "UPDATE import_jobs
            SET status = 'completed',
                finished_at = NOW(),
                total_users_imported    = $1,
                total_channels_imported = $2,
                total_messages_imported = $3,
                total_files_imported    = $4,
                verification_sha256_sample = $5
          WHERE id = $6"
    ).bind(counts.users).bind(counts.channels).bind(counts.messages)
     .bind(counts.files).bind(sample_sha).bind(job_id)
     .execute(pool).await?;
    Ok(())
}

pub async fn abort(pool: &PgPool, job_id: uuid::Uuid) -> anyhow::Result<()> {
    sqlx::query("UPDATE import_jobs SET cancellation_requested = true WHERE id = $1")
        .bind(job_id).execute(pool).await?;
    Ok(())
}

pub async fn cleanup(pool: &PgPool, job_id: uuid::Uuid) -> anyhow::Result<CleanupCounts> {
    let mut tx = pool.begin().await?;
    let deleted_posts: i64 = sqlx::query_scalar(
        "DELETE FROM posts WHERE props->>'import_job_id' = $1 RETURNING count(*)"
    ).bind(job_id.to_string()).fetch_one(&mut *tx).await?;
    let deleted_channels: i64 = sqlx::query_scalar(
        "DELETE FROM channels WHERE props->>'import_job_id' = $1 RETURNING count(*)"
    ).bind(job_id.to_string()).fetch_one(&mut *tx).await?;
    let deleted_users: i64 = sqlx::query_scalar(
        "DELETE FROM users WHERE props->>'import_job_id' = $1 AND auth_service = 'cyberos-imported'
                                       RETURNING count(*)"
    ).bind(job_id.to_string()).fetch_one(&mut *tx).await?;
    sqlx::query("UPDATE import_jobs SET status = 'aborted', finished_at = NOW() WHERE id = $1")
        .bind(job_id).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(CleanupCounts { posts: deleted_posts, channels: deleted_channels, users: deleted_users })
}

pub struct ImportCounts { pub users: i32, pub channels: i32, pub messages: i32, pub files: i32 }
pub struct CleanupCounts { pub posts: i64, pub channels: i64, pub users: i64 }
```

### file_download.rs — parallel file fetch with dedup

```rust
// services/chat-importer/src/slack/file_download.rs
use futures::stream::{StreamExt, FuturesUnordered};
use sha2::{Digest, Sha256};

pub async fn step_files(
    pool: &PgPool,
    zip: &mut zip::ZipArchive<File>,
    tenant_id: uuid::Uuid,
    workspace_id: &str,
    parallelism: usize,
) -> anyhow::Result<i32> {
    let file_refs = collect_unique_file_refs(zip)?;
    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(parallelism));
    let mut handles = FuturesUnordered::new();
    let mut count = 0;

    for f in file_refs {
        // Dedup by (workspace_id, file.id)
        if mm_file_exists_by_slack_file_id(pool, workspace_id, &f.id).await? { continue; }
        let permit = semaphore.clone().acquire_owned().await?;
        let pool = pool.clone();
        let workspace_id = workspace_id.to_owned();
        handles.push(tokio::spawn(async move {
            let _permit = permit; // drop on completion
            download_and_upload_one(&pool, &workspace_id, tenant_id, f).await
        }));
    }

    while let Some(joined) = handles.next().await {
        match joined? {
            Ok(_)  => count += 1,
            Err(e) => tracing::warn!(?e, "file import failed; continuing"),
        }
    }
    Ok(count)
}

async fn download_and_upload_one(
    pool: &PgPool,
    workspace_id: &str,
    tenant_id: uuid::Uuid,
    f: SlackFileRef,
) -> anyhow::Result<()> {
    let url = f.url_private.as_deref().ok_or_else(|| anyhow::anyhow!("no url"))?;
    // Authenticated GET — Slack tokens are scoped per export.
    let bytes = http_client().get(url)
        .header("Authorization", format!("Bearer {}", slack_token()))
        .send().await?.bytes().await?;
    let sha = Sha256::digest(&bytes);
    let mm_fileinfo_id = mm_upload_file(pool, tenant_id, f.name.as_deref(), &bytes).await?;
    // Insert dedup row.
    sqlx::query(
        "INSERT INTO cyberos_imported_files (workspace_id, slack_file_id, mm_fileinfo_id, sha256)
              VALUES ($1, $2, $3, $4)
         ON CONFLICT (workspace_id, slack_file_id) DO NOTHING"
    ).bind(workspace_id).bind(&f.id).bind(mm_fileinfo_id).bind(hex::encode(sha))
     .execute(pool).await?;
    Ok(())
}
```

### Existing example — orchestration loop (additions only)

```rust
// services/chat-importer/src/slack/steps.rs — additions
pub async fn run_all(zip_path: &Path, tenant_id: uuid::Uuid, opts: Opts) -> anyhow::Result<()> {
    let zip_sha = sha256_file(zip_path)?;
    let job = checkpoint::start_or_resume(&pool, &zip_sha, tenant_id, opts.workspace_id.as_deref(), opts.resume).await?;

    if job.finished_at.is_some() && job.status == "completed" {
        println!("✓ already imported (job {}, completed {})", job.id, job.finished_at.unwrap());
        return Ok(());
    }

    let mut zip = open_zip(zip_path)?;
    if opts.dry_run {
        let analysis = analyze_zip(&mut zip)?;
        println!("DRY-RUN: would import {} users, {} channels, {} messages, {} files",
                 analysis.users, analysis.channels, analysis.messages, analysis.files);
        return Ok(());
    }

    emit_memory_row("chat.import_started", serde_json::json!({
        "job_id": job.id, "tenant_id": tenant_id, "zip_sha256": zip_sha,
        "workspace_id": opts.workspace_id,
    })).await;

    let steps: [Step; 8] = [
        Step::new(1, "validate",        Box::pin(step_validate(&mut zip))),
        Step::new(2, "users",           Box::pin(step_users(&pool, &mut zip, tenant_id, &opts))),
        Step::new(3, "channels",        Box::pin(step_channels(&pool, &mut zip, tenant_id, &opts))),
        Step::new(4, "channel_members", Box::pin(step_channel_members(&pool, &mut zip, tenant_id, &opts))),
        Step::new(5, "messages",        Box::pin(step_messages(&pool, &mut zip, tenant_id, &opts))),
        Step::new(6, "threads",         Box::pin(step_threads(&pool, &mut zip, tenant_id, &opts))),
        Step::new(7, "files",           Box::pin(step_files(&pool, &mut zip, tenant_id,
                                            &opts.workspace_id.unwrap_or_default(), opts.file_parallelism))),
        Step::new(8, "verify",          Box::pin(step_verify(&pool, &mut zip, tenant_id, &opts))),
    ];

    for step in steps {
        if checkpoint::check_cancellation(&pool, job.id).await? {
            tracing::warn!(job_id = ?job.id, "cancellation requested; stopping");
            return Ok(());
        }
        if step.n <= job.last_step_completed { continue; }
        if opts.skip_files && step.n == 7 { continue; }
        eprintln!("[step {}/8] {}", step.n, step.name);
        let start = std::time::Instant::now();
        let count = step.run.await?;
        let dur = start.elapsed();
        checkpoint::complete_step(&pool, job.id, step.n, serde_json::json!({
            step.name.to_string(): {"count": count, "duration_ms": dur.as_millis()}
        })).await?;
        emit_memory_row("chat.import_step_completed", serde_json::json!({
            "job_id": job.id, "step": step.n, "name": step.name,
            "count": count, "duration_ms": dur.as_millis(),
        })).await;
    }

    let sample_sha = sample_and_verify(&pool, &mut zip, tenant_id, 100).await?;
    let counts = checkpoint::aggregate_counts(&pool, job.id).await?;
    checkpoint::finish(&pool, job.id, counts, &sample_sha).await?;
    emit_memory_row("chat.import_finished", serde_json::json!({
        "job_id": job.id, "counts": counts, "verification_sha": sample_sha,
    })).await;
    Ok(())
}
```

```rust
// services/chat-importer/src/slack/steps.rs
pub async fn step_users(zip: &mut zip::ZipArchive<File>, tenant_id: uuid::Uuid) -> anyhow::Result<i32> {
    let users_json = zip_read_file(zip, "users.json")?;
    let users: Vec<SlackUser> = serde_json::from_slice(&users_json)?;
    let mut count = 0;
    for u in users {
        // Dedup: lookup by slack_user_id
        if mm_user_exists_by_slack_id(&u.id).await? { continue; }
        mm_create_user(&u, tenant_id).await?;
        count += 1;
    }
    Ok(count)
}

pub async fn step_messages(zip: &mut zip::ZipArchive<File>, tenant_id: uuid::Uuid) -> anyhow::Result<i32> {
    let mut total = 0;
    let channel_files: Vec<String> = zip.file_names()
        .filter(|n| n.contains('/') && n.ends_with(".json") && !n.starts_with("__MACOSX"))
        .map(String::from).collect();

    for cf in channel_files {
        let chan_name = cf.split('/').next().unwrap();
        let mm_channel_id = lookup_mm_channel_by_slack_name(chan_name, tenant_id).await?;
        let messages: Vec<SlackMessage> = serde_json::from_slice(&zip_read_file(zip, &cf)?)?;
        for m in messages {
            // Dedup by (channel_id, ts)
            if mm_post_exists_by_slack_ts(mm_channel_id, &m.ts).await? { continue; }
            mm_create_post(mm_channel_id, &m, tenant_id).await?;
            total += 1;
            if total % 1000 == 0 {
                eprintln!("[step 5/8] imported {total} messages so far");
            }
        }
    }
    Ok(total)
}

pub async fn run_all(zip_path: &Path, tenant_id: uuid::Uuid, resume: bool, dry_run: bool) -> anyhow::Result<()> {
    let zip_sha = sha256_file(zip_path)?;
    let job = checkpoint::start_or_resume(&zip_sha, tenant_id, resume).await?;

    if job.finished_at.is_some() {
        println!("✓ already imported (job {}, completed {})", job.id, job.finished_at.unwrap());
        return Ok(());
    }

    let mut zip = open_zip(zip_path)?;
    if dry_run {
        let analysis = analyze_zip(&mut zip)?;
        println!("DRY-RUN: would import {} users, {} channels, {} messages",
                 analysis.users, analysis.channels, analysis.messages);
        return Ok(());
    }

    let steps: [(i32, &str, fn(...)); 8] = [
        (1, "validate",        step_validate),
        (2, "users",           step_users),
        (3, "channels",        step_channels),
        (4, "channel_members", step_channel_members),
        (5, "messages",        step_messages),
        (6, "threads",         step_threads),
        (7, "files",           step_files),
        (8, "verify",          step_verify),
    ];

    for (n, name, fn_) in steps {
        if n <= job.last_step_completed { continue; }
        eprintln!("[step {n}/8] {name}");
        let start = std::time::Instant::now();
        let count = fn_(&mut zip, tenant_id).await?;
        let dur = start.elapsed();
        checkpoint::complete_step(&job.id, n).await?;
        emit_memory_row("chat.import_step_completed", serde_json::json!({
            "job_id": job.id, "step": n, "name": name,
            "count": count, "duration_ms": dur.as_millis(),
        })).await;
    }

    checkpoint::finish(&job.id).await?;
    emit_memory_row("chat.import_finished", serde_json::json!({
        "job_id": job.id, "total_messages": job.total_messages_imported,
    })).await;
    Ok(())
}
```

---

## §4 — Acceptance criteria

1. **8 steps executed in order** — fixture: small zip → all 8 step_completed audit rows.
2. **Checkpoint persists** — after step 5, kill process; restart with --resume → resumes step 6.
3. **Idempotent re-import** — second run same zip → exits 0; "already imported".
4. **Dry-run no DB writes** — DB count unchanged after --dry-run.
5. **User dedup** — second import same users.json → no new MM users.
6. **Channel dedup** — same.
7. **Message dedup by (channel, ts)** — same.
8. **Progress reports every 1000 messages** — stderr shows count.
9. **CLI --tenant required** — missing flag → exit 1.
10. **memory audit chat.import_started + finished** — both emit.
11. **OTel metric chat_import_messages_total** — sums to actual.
12. **Permanent error fails fast** — corrupt zip → exit 1 with sev-1 alarm.
13. **Transient retry** — Mattermost API rate-limit → backoff + retry.
14. **RLS isolates** — import for tenant A invisible to tenant B.
15. **Files step skippable** — `--skip-files` flag completes without step 7 errors.
16. **Slack ts → MM create_at preserved** — imported post's `create_at` equals `floor(slack_ts × 1000)` per AC for §1 #14; fixture with `ts="1620000000.123456"` yields `create_at=1620000000123`.
17. **Threads reconnected via thread_ts → root_id** — fixture with a parent + 3 replies; observe all replies have `root_id` = parent's MM id (AC for §1 #15).
18. **Orphan thread reply imported as top-level** — fixture with a reply whose thread_ts has no matching parent in the export; observe MM post with `props.slack_thread_orphan = true` and `root_id = null` (AC for §1 #15).
19. **File dedup by (workspace, file_id)** — fixture with same file_id referenced from 5 messages; observe ONE MM `FileInfo` row with 5 message references (AC for §1 #16).
20. **File metadata PII redacted in memory audit** — fixture with file named `Resume - Trinh Thai Anh.pdf`; observe MM FileInfo retains original name AND the `chat.import_step_completed` memory payload uses `<NAME>` redaction (AC for §1 #17).
21. **Slack is_general → MM town-square** — fixture with channel `general` (is_general=true); observe MM channel named `town-square` with type `O` (AC for §1 #18).
22. **Slack is_im → MM direct channel** — fixture with DM channel; observe MM channel type=D (AC for §1 #18).
23. **Slack is_mpim → MM group channel** — fixture with multi-person IM; observe MM channel type=G (AC for §1 #18).
24. **Reactions imported one row per user** — fixture with reaction `{name:"thumbsup", users:["U1","U2"]}`; observe two MM reaction rows (AC for §1 #19).
25. **Custom emoji falls back to :question:** — fixture with `name:"company_logo"` (not in MM); observe MM reaction with emoji `question` and `props.original_emoji_name="company_logo"` (AC for §1 #19).
26. **Pinned messages preserved** — fixture with `pinned_to:[channel_id]`; observe MM post with `is_pinned=true` (AC for §1 #20).
27. **Edited messages emit warning** — fixture with `edited:{...}` block; observe `chat.import_warning` memory row with `reason="edit_history_unavailable"` (AC for §1 #21).
28. **Imported posts carry props markers** — observe every imported post has `props.cyberos_imported = true`, `props.cyberos_source = "slack"`, `props.slack_ts = "<original>"`, `props.slack_workspace_id = "<id>"` (AC for §1 #22).
29. **Messages step parallelism = 1 per channel** — instrument step 5; observe no two `mm_create_post` calls for the same channel are in flight (AC for §1 #23).
30. **Files step parallelism cap respected** — instrument step 7; observe at most 10 (default) concurrent downloads (AC for §1 #23).
31. **429 backoff honours Retry-After** — fake MM API returns 429 with `Retry-After: 17`; observe importer waits ≥17s before retry (AC for §1 #24).
32. **Permanent 429 fails after 8 retries** — fake MM returns 429 indefinitely; observe importer fails with `rate_limit_exhausted` after 8 attempts (AC for §1 #24).
33. **Zip CRC32 mismatch rejected at step 1** — fixture with corrupted member; observe step 1 fails fast (AC for §1 #25).
34. **--abort signals job cancellation** — fixture: start import; in another shell `cyberos-chat import abort <job>`; observe importer exits cleanly at next step boundary; `status="aborted"` (AC for §1 #26).
35. **--cleanup removes job entities** — after --abort, run `--cleanup`; observe DELETE counts match step_metrics counts (AC for §1 #27).
36. **--cleanup refuses non-aborted job** — `--cleanup` on a running or completed job → exit 1 with explanation (AC for §1 #27).
37. **Post-import sampling fires** — happy import; observe 100 sample-verification GETs against MM API and `verification_sha256_sample` populated in `import_jobs` row (AC for §1 #28).
38. **Sampling detects corruption** — inject a single-message corruption (modify body after insert); observe `chat.import_verification_failed` SEV-1 audit + `status="verification_failed"` (AC for §1 #28).
39. **step_metrics JSON populated per step** — observe `import_jobs.step_metrics` after a completed run has 8 keys (one per step) each carrying `{count, duration_ms}` (AC for §1 #29).
40. **workspace_id required when zip is Enterprise Grid** — Enterprise Grid exports carry `team_id` in user records; importer requires `--workspace-id` flag to disambiguate (AC for §1 #30).

---

## §5 — Verification

Fixtures live in `services/chat-importer/tests/fixtures/`:
- `small.zip` — 3 users, 2 channels, 10 messages, 1 file. Used for fast happy paths.
- `corrupt.zip` — same as small but CRC32 deliberately broken on `users.json`.
- `with_threads.zip` — 1 parent + 3 reply messages.
- `with_orphans.zip` — 2 reply messages whose parents are absent.
- `with_reactions.zip` — 1 message with `:thumbsup:` × 2 users + 1 custom `:company_logo:`.
- `with_files.zip` — 1 message referencing the same file_id from 5 messages.
- `enterprise_grid.zip` — multi-workspace export structure.
- `large_corpus.zip` — 100k messages across 30 channels; used in nightly perf gate.

### AC #1 — happy path

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac1_happy_small_import() {
    let env = TestEnv::new().await;
    run_all(env.fixture("small.zip"), env.tenant_id(), Opts::default()).await.unwrap();
    let job = env.fetch_job_by_zip(&env.fixture_sha("small.zip")).await.unwrap();
    assert_eq!(job.status, "completed");
    assert_eq!(job.last_step_completed, 8);
    assert!(job.total_messages_imported >= 10);
    let kinds = env.memory.kinds_collected().await;
    assert!(kinds.contains(&"chat.import_started".to_string()));
    assert_eq!(kinds.iter().filter(|&k| k == "chat.import_step_completed").count(), 8);
    assert!(kinds.contains(&"chat.import_finished".to_string()));
}
```

### AC #2 — resume from step 5

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac2_resume_from_step_5() {
    let env = TestEnv::new().await;
    env.crash_after_step(4).await;
    let pre = env.fetch_job_by_zip(&env.fixture_sha("small.zip")).await.unwrap();
    assert_eq!(pre.last_step_completed, 4);

    run_all(env.fixture("small.zip"), env.tenant_id(),
            Opts { resume: true, ..Opts::default() }).await.unwrap();
    let post = env.fetch_job_by_zip(&env.fixture_sha("small.zip")).await.unwrap();
    assert_eq!(post.last_step_completed, 8);
    assert_eq!(post.status, "completed");

    let n = env.memory.count_rows("chat.import_step_completed").await;
    assert_eq!(n, 8); // 4 from crashed + 4 from resume
}
```

### AC #3 — idempotent replay

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac3_idempotent_replay() {
    let env = TestEnv::new().await;
    run_all(env.fixture("small.zip"), env.tenant_id(), Opts::default()).await.unwrap();
    let count1 = env.count_messages_for_tenant().await;
    run_all(env.fixture("small.zip"), env.tenant_id(), Opts::default()).await.unwrap();
    let count2 = env.count_messages_for_tenant().await;
    assert_eq!(count1, count2, "duplicate import created new posts");
}
```

### AC #4 — dry-run no writes

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac4_dry_run_no_db_writes() {
    let env = TestEnv::new().await;
    let pre = env.count_messages_for_tenant().await;
    run_all(env.fixture("small.zip"), env.tenant_id(),
            Opts { dry_run: true, ..Opts::default() }).await.unwrap();
    let post = env.count_messages_for_tenant().await;
    assert_eq!(pre, post);
    let job = env.fetch_job_by_zip(&env.fixture_sha("small.zip")).await;
    assert!(job.is_none(), "dry-run should not create import_jobs row");
}
```

### AC #16 — timestamp preservation

```rust
#[test]
fn ac16_slack_ts_to_ms_floor() {
    assert_eq!(slack_ts_to_ms("1620000000.123456").unwrap(), 1620000000123);
    assert_eq!(slack_ts_to_ms("1620000000.999999").unwrap(), 1620000000999);
    assert_eq!(slack_ts_to_ms("1620000000.000000").unwrap(), 1620000000000);
    assert_eq!(slack_ts_to_ms("0.500000").unwrap(), 500);
}
```

### AC #17 — thread reconnection

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac17_threads_reconnected() {
    let env = TestEnv::new().await;
    run_all(env.fixture("with_threads.zip"), env.tenant_id(), Opts::default()).await.unwrap();
    let parent_id = env.find_post_by_slack_ts("1620000000.001").await.unwrap();
    let replies = env.posts_with_root_id(&parent_id).await;
    assert_eq!(replies.len(), 3);
    for r in replies {
        assert_eq!(r.root_id, Some(parent_id.clone()));
        assert_eq!(r.props.get("slack_thread_orphan"), None);
    }
}
```

### AC #18 — orphan reply as top-level

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac18_orphan_reply_top_level() {
    let env = TestEnv::new().await;
    run_all(env.fixture("with_orphans.zip"), env.tenant_id(), Opts::default()).await.unwrap();
    let posts = env.posts_with_prop("slack_thread_orphan", "true").await;
    assert_eq!(posts.len(), 2);
    for p in posts { assert!(p.root_id.is_none()); }
}
```

### AC #19 — file dedup

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac19_file_dedup() {
    let env = TestEnv::new().await;
    run_all(env.fixture("with_files.zip"), env.tenant_id(),
            Opts { workspace_id: Some("T-test".into()), ..Opts::default() }).await.unwrap();
    let n_fileinfos = env.count_fileinfos_for_workspace("T-test").await;
    let n_message_refs = env.count_posts_referencing_any_file().await;
    assert_eq!(n_fileinfos, 1);
    assert_eq!(n_message_refs, 5);
}
```

### AC #20 — file metadata redaction in audit

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac20_file_metadata_pii_redacted_in_audit() {
    let env = TestEnv::new().await;
    run_all(env.fixture("with_files.zip"), env.tenant_id(),
            Opts { workspace_id: Some("T".into()), ..Opts::default() }).await.unwrap();
    let row = env.memory.last_of_kind("chat.import_step_completed").await.unwrap();
    let body = serde_json::to_string(&row).unwrap();
    assert!(!body.contains("Trinh Thai Anh"));
    assert!(body.contains("<NAME>"));

    // But the MM FileInfo row retains the original.
    let fi = env.fileinfo_by_slack_id("T", "F0FILE001").await.unwrap();
    assert!(fi.name.unwrap().contains("Trinh Thai Anh"));
}
```

### AC #21..#23 — channel-type mapping

```rust
#[rstest]
#[case(SlackChannel { is_general: Some(true),  ..mock() }, "O", "town-square")]
#[case(SlackChannel { is_private: Some(true),  ..mock() }, "P", "secret-room")]
#[case(SlackChannel { is_im:      Some(true),  ..mock() }, "D", "")]
#[case(SlackChannel { is_mpim:    Some(true),  ..mock() }, "G", "")]
#[case(SlackChannel { is_archived:Some(true),  ..mock() }, "O", "")]
fn ac21_22_23_channel_type_mapping(
    #[case] sc: SlackChannel,
    #[case] expected_type: &str,
    #[case] expected_name: &str,
) {
    let mm = map_slack_to_mm_channel(&sc);
    assert_eq!(mm.channel_type, expected_type);
    if !expected_name.is_empty() {
        assert_eq!(mm.name, expected_name);
    }
    if sc.is_archived.unwrap_or(false) {
        assert!(mm.delete_at > 0);
    }
}
```

### AC #24 — reactions one row per user

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac24_reactions_one_row_per_user() {
    let env = TestEnv::new().await;
    run_all(env.fixture("with_reactions.zip"), env.tenant_id(), Opts::default()).await.unwrap();
    let n = env.count_reactions_for_emoji("thumbsup").await;
    assert_eq!(n, 2); // U1 + U2
}
```

### AC #25 — custom emoji fallback

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac25_custom_emoji_fallback() {
    let env = TestEnv::new().await;
    run_all(env.fixture("with_reactions.zip"), env.tenant_id(), Opts::default()).await.unwrap();
    let r = env.reaction_by_emoji("question").await.unwrap();
    assert_eq!(r.props.get("original_emoji_name").and_then(|v| v.as_str()),
               Some("company_logo"));
}
```

### AC #28 — props markers

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac28_imported_props_markers() {
    let env = TestEnv::new().await;
    run_all(env.fixture("small.zip"), env.tenant_id(),
            Opts { workspace_id: Some("T-acme".into()), ..Opts::default() }).await.unwrap();
    let post = env.first_imported_post().await;
    assert_eq!(post.props["cyberos_imported"], serde_json::json!(true));
    assert_eq!(post.props["cyberos_source"], serde_json::json!("slack"));
    assert!(post.props["slack_ts"].as_str().unwrap().contains('.'));
    assert_eq!(post.props["slack_workspace_id"], "T-acme");
}
```

### AC #31 — Retry-After honoured

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac31_retry_after_honoured() {
    let env = TestEnv::new().await;
    env.mm_fake.respond_with(
        // First 1 call → 429 with Retry-After: 2; thereafter → 200.
        ResponsePolicy::sequence(vec![(429, &[("Retry-After","2")]), (200, &[])])
    ).await;
    let start = std::time::Instant::now();
    run_all(env.fixture("small.zip"), env.tenant_id(), Opts::default()).await.unwrap();
    let elapsed = start.elapsed();
    assert!(elapsed.as_secs() >= 2, "didn't wait Retry-After: {:?}", elapsed);
}
```

### AC #32 — permanent 429 fails

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac32_permanent_429_fails_after_8_retries() {
    let env = TestEnv::new().await;
    env.mm_fake.respond_with(ResponsePolicy::always(429, &[])).await;
    let result = run_all(env.fixture("small.zip"), env.tenant_id(), Opts::default()).await;
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("rate_limit_exhausted"), "got: {}", msg);
}
```

### AC #33 — corrupt zip rejected

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac33_corrupt_zip_rejected_at_step_1() {
    let env = TestEnv::new().await;
    let r = run_all(env.fixture("corrupt.zip"), env.tenant_id(), Opts::default()).await;
    assert!(r.is_err());
    let job = env.fetch_job_by_zip(&env.fixture_sha("corrupt.zip")).await.unwrap();
    assert_eq!(job.last_step_completed, 0);
    assert_eq!(job.status, "failed");
    assert!(job.error_message.unwrap().contains("CRC32"));
}
```

### AC #34/#35 — abort + cleanup

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac34_abort_signals_cancellation() {
    let env = TestEnv::new().await;
    let job_id = env.start_import_in_background(env.fixture("large_corpus.zip")).await;
    env.wait_until_step(job_id, 5).await;
    cli::abort(&pool, job_id).await.unwrap();
    env.wait_until_status(job_id, "aborted").await; // <= 30s
    assert_eq!(env.fetch_job(job_id).await.status, "aborted");
}

#[tokio::test(flavor = "multi_thread")]
async fn ac35_cleanup_removes_entities() {
    let env = TestEnv::new().await;
    let job_id = env.run_to_abort(env.fixture("small.zip")).await;
    let pre_posts = env.count_messages_for_tenant().await;
    let cleanup = cli::cleanup(&pool, job_id, true).await.unwrap();
    let post_posts = env.count_messages_for_tenant().await;
    assert_eq!(post_posts, 0);
    assert!(cleanup.posts >= 5);
    assert!(cleanup.channels >= 1);
    assert!(cleanup.users >= 1);
}
```

### AC #37/#38 — sampling + corruption detection

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac37_post_import_sampling_runs() {
    let env = TestEnv::new().await;
    run_all(env.fixture("small.zip"), env.tenant_id(), Opts::default()).await.unwrap();
    let job = env.fetch_job_by_zip(&env.fixture_sha("small.zip")).await.unwrap();
    assert!(job.verification_sha256_sample.as_ref().unwrap().len() == 64);
    let n_gets = env.mm_fake.get_count("/api/v4/posts/").await;
    assert!(n_gets >= 10); // sample size capped at min(100, total_messages)
}

#[tokio::test(flavor = "multi_thread")]
async fn ac38_sampling_detects_corruption() {
    let env = TestEnv::new().await;
    env.mid_run_corrupt_one_message().await;
    let r = run_all(env.fixture("small.zip"), env.tenant_id(), Opts::default()).await;
    assert!(r.is_err() || env.fetch_status().await == "verification_failed");
    let row = env.memory.last_of_kind("chat.import_verification_failed").await.unwrap();
    assert_eq!(row["severity"], "SEV-1");
}
```

### Property test — slack_ts edge cases

```rust
proptest! {
    #[test]
    fn slack_ts_round_trip_within_ms(secs in 0i64..2_000_000_000_i64, frac in 0u32..1_000_000u32) {
        let ts = format!("{}.{:06}", secs, frac);
        let ms = slack_ts_to_ms(&ts).unwrap();
        prop_assert_eq!(ms, secs * 1000 + (frac as i64) / 1000);
        prop_assert!(ms >= 0);
    }
}
```

---

## §6 — Implementation skeleton

The Rust + SQL modules above are the skeleton. This section names the operational wiring:

### §6.1 — Process model

The importer is a one-shot CLI invoked by an operator (or a CI job for staged tenants). It runs as a Fargate task in the same VPC as the tenant's CHAT stack so it can reach RDS + MM API at low latency without crossing the public internet. The task definition is owned by TASK-CHAT-003 but launched ad-hoc via `aws ecs run-task` (not a long-running service).

### §6.2 — MM API access pattern

The importer uses MM's REST API (port 8065 inside the VPC) rather than direct DB inserts because:
1. MM has invariants enforced at the API layer (post threading, channel membership constraints) that direct SQL would bypass.
2. MM has post-insert hooks (search indexer, presence updater) that fire only via API.
3. Using API means the importer is upgrade-safe across MM versions; SQL would couple us to a schema version.

### §6.3 — Authentication

The importer authenticates to MM as a per-tenant import bot (created by TASK-CHAT-002 admin REST, marked `is_bot=true`). The bot has `system_admin` role for the duration of the import only; an operator post-import workflow demotes it to `system_user` after `chat.import_finished` audit row fires.

### §6.4 — File download authentication

Slack files (in step 7) are fetched from `url_private` which requires a Slack API token. The export workflow at Slack's end includes a one-time token; we accept it via `--slack-token` flag (env-var-overridable). The token is read once at importer startup and held in memory; never written to disk or logs.

### §6.5 — Memory bounds

- `channels.json` typically < 1MB; loaded whole.
- `users.json` typically < 10MB; loaded whole.
- Channel message files (`channels/<name>/<date>.json`) typically < 5MB each; loaded one at a time per channel.
- Files (in step 7) streamed via `tokio::io::copy`; never buffered fully in memory.

The `zip_read_file` helper refuses any file > 256MB to bound worst-case memory.

### §6.6 — Slack ts uniqueness assumption

Slack guarantees `ts` is unique within a channel (used as message ID). Our dedup key is `(mm_channel_id, slack_ts)`. If Slack ever ships a duplicate (theoretical only), the second message is dropped + a SEV-2 warning emitted.

### §6.7 — Workspace inference

If `--workspace-id` is not given, we infer it from `users[0].team_id`. If `team_id` is absent (older exports), we fall back to a SHA-256 of the zip filename. The audit row records which path was taken.

### §6.8 — Rate-limit pool

A single `governor::RateLimiter<NotKeyed, _, _>` instance gates ALL MM API calls across steps. Default is 80 calls/sec (MM's documented sustained-rate ceiling). This prevents step 5 (sequential per channel) from racing step 7 (parallel file uploads) for the limit budget.

### §6.9 — Cleanup transactional semantics

`cleanup(job_id)` runs inside one Postgres transaction across posts + channels + users + import_jobs. If any DELETE fails, the whole thing rolls back and the operator can retry. This avoids partial-cleanup states.

### §6.10 — Sample verification design

After step 8, we pick 100 post_ids uniformly at random from the imported set. For each, we:
1. Fetch the post body via MM API.
2. Look up the corresponding message in the source zip (by channel + ts).
3. SHA-256 both bodies; compare.

If any pair mismatches, `chat.import_verification_failed` fires with the specific post_id. The `verification_sha256_sample` field stores the sha of the concatenated 100 (post_id, body_sha) pairs — operators can re-run a deterministic sample independently.

### §6.11 — `import_job_id` props propagation

Every imported entity carries `props.import_job_id = <uuid>`. This is the linkage for `--cleanup`. It also lets TASK-CHAT-005 bridge omit `import_job_id` from memory payloads (we don't want every memory row tagged with an internal import id — those are operational, not user-visible state).

### §6.12 — Failure routing matrix

| Step | Permanent error → | Transient error → |
|---|---|---|
| 1 | exit 1, SEV-1 audit, status=failed | n/a (validation is local) |
| 2 | exit 1 if MM rejects all users; SEV-2 | retry up to 8 with backoff |
| 3 | as above | as above |
| 4 | as above | as above |
| 5 | exit 1 only on schema parse fail; SEV-2 | retry per-message |
| 6 | log + skip orphans | retry MM API |
| 7 | exit 0 even with failed files (best-effort); SEV-3 warning | retry per-file |
| 8 | exit 1 on verification mismatch; SEV-1 | retry GET per sample |

---

## §7 — Dependencies

- **TASK-CHAT-005** — bridge picks up imported posts and emits to memory.
- **TASK-CHAT-007** — Zalo importer sibling pattern.
- **TASK-CHAT-010** — decommission signal uses imported count.

---

## §8 — Example payloads

### `chat.import_started`

```json
{
  "kind": "chat.import_started",
  "ts_ns": 1747407137100000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "trace_id": "0af7651916cd43dd8448eb211c80319c",
  "payload": {
    "job_id":           "01HVQX8ZG2K3R4TVA7P3WV5X8N",
    "source":           "slack",
    "zip_sha256":       "f1a2b3c4d5e6f7081920304050607080a1b2c3d4e5f6708192030405060708ab",
    "zip_size_bytes":   84_238_172,
    "workspace_id":     "T0ACME001",
    "started_by":       "ops@cyberskill.world",
    "dry_run":          false,
    "resume":           false
  }
}
```

### `chat.import_step_completed`

```json
{
  "kind": "chat.import_step_completed",
  "ts_ns": 1747407221100000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "payload": {
    "job_id":      "01HVQX8ZG2K3R4TVA7P3WV5X8N",
    "step":        5,
    "name":        "messages",
    "count":       12_345,
    "duration_ms": 84_210,
    "throughput_messages_per_sec": 146.6
  }
}
```

### `chat.import_warning`

```json
{
  "kind": "chat.import_warning",
  "ts_ns": 1747407221150000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "payload": {
    "job_id":  "01HVQX8ZG2K3R4TVA7P3WV5X8N",
    "step":    5,
    "reason":  "edit_history_unavailable",
    "context": { "slack_ts": "1620000000.123456" }
  }
}
```

### `chat.import_finished`

```json
{
  "kind": "chat.import_finished",
  "ts_ns": 1747407500000000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "payload": {
    "job_id":                "01HVQX8ZG2K3R4TVA7P3WV5X8N",
    "counts": {
      "users":    47,
      "channels": 18,
      "messages": 12_345,
      "files":    134
    },
    "duration_total_ms": 362_900,
    "verification_sha256_sample": "ab12cd34ef56...",
    "warnings_count": 12
  }
}
```

### `chat.import_verification_failed`

```json
{
  "kind": "chat.import_verification_failed",
  "ts_ns": 1747407500500000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "severity": "SEV-1",
  "payload": {
    "job_id":          "01HVQX8ZG2K3R4TVA7P3WV5X8N",
    "mismatched_count": 3,
    "sample_size":      100,
    "first_mismatch_post_id": "01HVQX8ZG2K3R4TVA7P3WV5X8P",
    "expected_body_sha":      "abc123...",
    "actual_body_sha":        "def456..."
  }
}
```

### `chat.import_aborted`

```json
{
  "kind": "chat.import_aborted",
  "ts_ns": 1747407400000000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "payload": {
    "job_id":      "01HVQX8ZG2K3R4TVA7P3WV5X8N",
    "operator":    "ops@cyberskill.world",
    "stopped_at_step": 5,
    "reason":      "operator_requested"
  }
}
```

### CLI status / list output

```text
$ cyberos-chat import status 01HVQX...
job:        01HVQX8ZG2K3R4TVA7P3WV5X8N
source:     slack
tenant:     1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21
zip_sha:    f1a2b3...ab
started:    2026-05-16 12:32:17 UTC
finished:   2026-05-16 12:38:20 UTC  (6m 3s)
counts:     47 users · 18 channels · 12,345 messages · 134 files
warnings:   12
verification: ok (sha=ab12cd...)

$ cyberos-chat import list --tenant 1f8c...
ID                                 SOURCE  STATUS              MESSAGES  STARTED
01HVQX8ZG2K3R4TVA7P3WV5X8N         slack   completed           12,345    2026-05-16 12:32
01HVQXA1B2C3D4E5F6G7H8J9KLMNPQR    slack   verification_failed 8,201     2026-05-15 09:11
01HVQXX0Y0Z0W0V0U0T0S0R0Q0P0O0N    zalo    aborted               152     2026-05-14 22:00
```

---

## §9 — Open questions

All resolved. Deferred:
- Slack new export format (Enterprise Grid) — slice 4+; current = legacy + standard.
- Incremental imports (new messages since last import) — slice 4+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Corrupt zip (CRC32 mismatch) | step 1 zip-validate | exit 1; SEV-1 `chat.import_failed`; status=failed | Re-export from Slack |
| Missing required file in zip (channels.json absent) | step 1 explicit check | exit 1; SEV-1; status=failed | Re-export with all sections |
| Zip member > 256MB | `zip_read_file` cap | exit 1 | Operator splits export |
| User schema unknown field | serde lax mode (`#[serde(deny_unknown_fields)]`=false) | warning logged + continue | None |
| User missing email | warning; placeholder `<id>@imported.invalid` | imported user lacks login | Operator post-fixup |
| User from deleted Slack workspace | `deleted=true` | imported user; `delete_at` set to import time | None |
| Bot user with no profile | falls back to bot_id as display_name | imported as MM user with `is_bot=true` | None |
| MM API rate limit (429) | response code + header | exp backoff per AC #31 + #32; max 8 retries | None |
| MM API rate limit exhausted | 8 consecutive 429s | exit 1; SEV-1 `chat.import_rate_limit_exhausted` | Operator backs off + resumes later |
| MM API 5xx | http_client error | retry per Retry-After; cap 5 | None |
| MM API auth token expired mid-import | 401 | exit 1; SEV-1 | Operator rotates bot token |
| Tenant_id not matching MM team | step 2 user creation 403 | exit 1; SEV-1 | Operator fixes `--tenant` |
| Concurrent imports same zip + tenant | UNIQUE constraint | second fails fast with status hint | None |
| Crash mid-step | checkpoint not advanced | resume replays step from start (idempotent) | None |
| Crash between step end + checkpoint commit | rare race | resume replays last step (idempotent) | None |
| Crash during file download | semaphore released on drop | resume re-attempts un-fetched files | None |
| Slack file 403 (URL expired) | http 403 on download | SEV-2 warning; mark file as failed; continue step | Re-export to get fresh URLs |
| Slack file 404 | http 404 | SEV-2 warning; file skipped | None |
| File checksum mismatch on upload | post-upload SHA compare | SEV-2 warning; file deleted + retry | None |
| MM file upload exceeds tenant max (e.g. 100MB) | MM 413 | SEV-2 warning; file skipped | Operator bumps tenant max OR splits zip |
| DB disk full mid-message | sqlx Err | exit 1; SEV-1 | Operator scales RDS storage; resume |
| RDS connection lost | sqlx Err | reconnect loop; up to 3 attempts | None |
| RDS failover (multi-AZ) | brief disconnect | reconnect within 30s; resume current step | None |
| Thread reply with absent parent | resolved at step 6 | reply imported as top-level + `slack_thread_orphan=true`; warning | Operator post-fixup OR re-export with parent |
| Thread parent appears AFTER reply in source | step 5 inserts in order; step 6 retros root_id | None | None |
| Channel renamed in source between exports | dedup by slack_channel_id, not name | First name wins; new name not propagated | Operator manually renames |
| Channel privacy flipped between exports | First import wins; second is no-op | Stale privacy retained | Operator manually fixes |
| Reaction on missing post | step 5 inserts posts before reactions parse | order preserved | None |
| Custom emoji unknown in MM | reaction falls back to `:question:` | `props.original_emoji_name` retained | Operator can add custom emoji + replay |
| Pinned message references missing post | step 5 inserts; pin set at step 5 itself | If post doesn't exist, warning | None |
| Slack ts collision within channel (theoretical) | dedup key duplicate | second dropped; SEV-2 | Re-export |
| Workspace ID inference fails | no team_id + no flag | exit 1; operator must provide `--workspace-id` | Operator |
| `--cleanup` race with bridge (TASK-CHAT-005) | bridge sees DELETE events | bridge emits chat.message_deleted to memory | None — desired behaviour |
| `--cleanup` race with active user (someone messaging mid-cleanup) | mixed DELETEs | partial cleanup; aborted job state | Operator re-runs cleanup |
| `--abort` while step 7 has 50 file downloads in flight | semaphore allows completion of in-flight | up to 50 files still uploaded post-abort | None — accepted |
| Network partition mid-step 5 | tx fails | exit 1; resume from current step | None |
| Memory pressure (large channel file) | OOM | task killed | Operator bumps Fargate memory; resume |
| Verification sample mismatch | step 8 SHA compare | SEV-1 `chat.import_verification_failed`; status=verification_failed | Operator runs `--cleanup` + re-import |
| Sample fetch returns 404 (post deleted between import + verify) | rare; ignored as long as <5% sample failures | None | None |
| memory audit emit fails mid-import | logged; metric increments | importer continues | Operator backfills audit rows manually |
| Operator runs `--cleanup --yes-i-know=false` | safety prompt | refused | None — by design |
| zip_sha256 collision (cryptographic; impossible) | None | None | None |
| Slack export contains workspace bot users (BotID) | imported as MM bot user | None | None |
| Enterprise Grid export missing workspace boundary | `--workspace-id` required | exit 1 if not provided | Operator |
| MM API version drift (e.g. v5 deprecated v4 endpoint) | API call returns "deprecated" | importer continues if response still 2xx; SEV-3 | Operator upgrades importer image |
| step_metrics JSONB grows unbounded (many steps) | bounded at 8 keys | None | None |
| Two concurrent operators run import for same job (with --resume) | DB lock or last-write-wins | one wins | None — by design |
| Importer Fargate task killed mid-step | crash + restart with --resume | resumes | None |
| OBS metrics collector down | metrics dropped | importer continues | None |
| KMS CMK access denied mid-upload | upload fail | retry; max 3 | Operator restores KMS access |
| Slack file URL contains PII (rare) | url retained in MM file blob | None visible | None — operator scrub if required |

---

## §11 — Implementation notes

- `zip` crate streams individual files; we read each channel's JSON one-at-a-time rather than holding the whole zip in memory. The 256MB per-file cap is a safety belt.
- Slack timestamps are floats encoded as strings (e.g. "1620000000.123456"). We parse to f64 then `floor((×1000))` to get MM milliseconds. We never use the f64 directly because float comparison for dedup is fragile.
- Files download in batches of 10 parallel by default; configurable via `--file-parallelism`. We measured: 1 = 12 min for 134 files, 10 = 90s, 20 = 70s but MM rate-limited; 10 is the sweet spot.
- `--resume` is optional; default re-runs from step 1 (idempotent so no harm). The reason resume isn't always-on: an operator may want to retry from scratch if the early steps' MM API calls have changed (e.g. user mapping fix).
- Progress reports every 1000 messages keep stderr useful without flooding. We considered every 100 but the import speed of ~150 msg/sec means 100 = 0.7s; too noisy.
- Sub-100KB exports complete in < 5s; 100K-message exports ~10-15min. The bottleneck is MM API throughput (80 calls/sec), not the importer itself.
- `governor::RateLimiter` is shared across all steps via an `Arc`; we considered per-step limiters but cross-step burst behaviour was worse (steps 2/3/4 collided with step 7's file-upload calls).
- Why MM API and not direct SQL: SQL bypasses MM's invariants. Example: inserting a post into a channel the user isn't a member of would create an orphan post that MM's UI can't render. Going through MM API hits the membership check.
- Why we mark posts with `cyberos_imported=true`: TASK-CHAT-005 bridge uses this to short-circuit push notifications. Without it, every historical message would push-notify every user in the channel.
- Slack reactions can have `count != users.length` when there are external members; we always use `users.length` (the explicit enumeration) to compute the MM reaction rows.
- The `:question:` fallback for unknown emoji is debated; alternatives were drop the reaction (loses signal) or import as `:custom-<name>:` (requires MM custom-emoji upload, slice 4+). `:question:` is the lowest-effort visible marker.
- Post-import sampling at 100 is calibrated against ~12k messages typical (≤1% sample); for 100k messages we still sample 100 (a fixed cap), trusting that any corruption would have a >0% rate.
- Why we sample by random post_id rather than first-N: random catches systemic bugs (e.g. "every 1000th message is corrupted"); first-N would only catch early failures.
- `--cleanup` requires `--yes-i-know` to prevent operator accidents — the operation deletes potentially hundreds of thousands of rows and is irreversible. The flag is a small friction with large risk-mitigation value.
- We considered making `--cleanup` revert to a Postgres snapshot via PITR but rejected as too heavy (full restore for a 5-second job).
- Slack's export does NOT include thread reply counts that match reality if replies were edited/deleted in source; we trust the actual message file content over the `reply_count` field.
- Channel naming: Slack `general` → MM `town-square` because MM treats `town-square` as the default-team channel (auto-added to new users). Operators get the import behaviour they expect.
- Why we don't preserve Slack edit history: Slack's export only ships the final text. Reconstructing history would require Slack's API (live, not exportable). Accepting the data loss with an explicit warning is honest.
- Why we don't import Slack reactions to non-existent emoji as missing: would silently lose reactions; the `:question:` mapping at least surfaces "there was something here."
- The `verification_sha256_sample` field lets a separate auditor re-run the sample independently and arrive at the same hash. This is a determinism-via-deterministic-sample design — picking the same 100 post_ids on re-run.
- We use UUID-v7 for `import_jobs.id` so that operator listings sort chronologically without needing a separate `started_at` index.
- `step_metrics` JSONB is bounded at 8 entries by design; we could've used a separate `import_step_results` table but the per-step record is so small (count + duration) that a JSONB column on the parent row is simpler and cheaper.
- The cleanup-during-active-bridge case (TASK-CHAT-005 sees DELETEs and emits `chat.message_deleted`) is the intended behaviour: memory's view of "what happened" should be eventually consistent with chat. A subsequent re-import re-creates the posts and the bridge emits `chat.message` again.
- We chose CLI over an HTTP endpoint because import is an operator workflow, not a user workflow. CLI is easier to ssh-into-fargate-task + watch + restart than HTTP.
- The CLI prints structured progress to stderr and structured JSON-lines to stdout (one line per audit row). Operators wanting machine-readable progress pipe stdout to `jq`.
- `--dry-run` is implemented by short-circuiting step entry points; the analysis (counts) is computed by step 1's zip parse, so dry-run is fast (~1s for any zip size).
- Why we require `--workspace-id` for Enterprise Grid but not standard exports: standard exports have one workspace; Enterprise Grid has many bundled into one zip. We could've inferred it from `team_id` clusters but explicit flag is unambiguous.
- The "system_admin promotion + post-import demotion" pattern (§6.3) is operator-driven; we don't auto-demote because the import audit row triggers a separate workflow that includes a human review.
- We considered shipping the importer as a long-running daemon with HTTP triggers — rejected because:
  1. Imports are infrequent (per-tenant lifetime ≤ a few).
  2. Long-running services have to be patched + monitored.
  3. CLI = ephemeral; runs, completes, dies. No attack surface in between.
- Why Postgres-side step_metrics over OBS metrics: OBS retention is 7d; post-mortem on an import that ran 3 weeks ago needs the metrics. Postgres JSONB column lives as long as the job row does (forever, by default).
- The bot token reused for MM API auth has a 30d expiry; the importer fails fast on 401 because there's no graceful reauth path during a multi-minute run.
- We chose `sha2` crate over `ring::digest` for SHA-256 because `sha2` ships with no transitive openssl dependency, keeping the importer binary lean.
- The `--cleanup` operation is the inverse of import; we don't expose a `--reimport` flag because the canonical sequence is: cleanup + re-import. Operators understand this is two steps.

---

*End of TASK-CHAT-006.*
