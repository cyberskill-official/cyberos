---
id: FR-CHAT-012
title: "DSAR export — Data Subject Access Request: every message a subject authored + chained memory audit hashes for tamper-evidence"
module: CHAT
priority: MUST
status: ready_to_implement
verify: T
phase: P1
milestone: P1 · slice 2
slice: 2
owner: Stephen Cheng
created: 2026-05-16
shipped: null
memory_chain_hash: null
related_frs: [FR-CHAT-005, FR-MEMORY-101, FR-AUTH-005]
depends_on: [FR-CHAT-005]
blocks: []

source_pages:
  - website/docs/modules/chat.html#dsar
  - website/docs/legal/pdpl-dsar.html
source_decisions:
  - DEC-530 (DSAR fulfils PDPL Art. 14 + GDPR Art. 15 — subject can request all data about themselves)
  - DEC-531 (export includes memory chain_anchors per message; recipient verifies tamper-evidence)
  - DEC-532 (export delivered as zip via short-lived signed S3 URL; subject-only access)

language: rust 1.81
service: cyberos/services/chat-dsar/
new_files:
  - services/chat-dsar/Cargo.toml
  - services/chat-dsar/src/main.rs
  - services/chat-dsar/src/exporter.rs
  - services/chat-dsar/src/manifest.rs
  - services/chat-dsar/tests/dsar_test.rs
modified_files:
  - services/chat/sql/init-dsar-requests.sql
allowed_tools:
  - file_read: services/chat-dsar/**, services/chat/**
  - file_write: services/chat-dsar/{src,tests}/**, services/chat/sql/**
  - bash: cd services/chat-dsar && cargo test
disallowed_tools:
  - export without subject-verified request (per DEC-530 — must be subject-initiated OR admin-with-justification)
  - skip chain_anchor inclusion (per DEC-531)

effort_hours: 6
sub_tasks:
  - "0.5h: init-dsar-requests.sql migration"
  - "0.5h: Cargo.toml deps (zip, sha2, aws-sdk-s3)"
  - "1.0h: exporter.rs — query posts WHERE user_id = subject; include channel + thread context"
  - "1.0h: manifest.rs — list every message with chain_anchor from FR-CHAT-005 memory row"
  - "0.5h: zip composition (messages.jsonl + manifest.json + verification README)"
  - "0.5h: S3 upload with KMS-encrypt + 7-day TTL signed URL"
  - "0.5h: REST endpoint POST /api/dsar/request (subject-only OR admin-with-justification)"
  - "0.5h: memory audit 'chat.dsar_requested' + 'chat.dsar_delivered'"
  - "1.0h: dsar_test.rs — happy + verify chain_anchor + admin path"
risk_if_skipped: "PDPL Art. 14 + GDPR Art. 15 are legally mandatory. Non-compliance = fines + brand damage. Without chain_anchor, recipient can't prove the export is authentic (could be operator-edited). Without short-lived URL, the export sits in S3 forever. Without admin-with-justification path, ops cannot fulfill on behalf of subjects who left the company."
---

## §1 — Description (BCP-14 normative)

The DSAR export **MUST** generate a tamper-evident archive of every message a subject authored. The contract:

1. **MUST** define `dsar_requests` table: `(id UUID PK, subject_id UUID, requested_by UUID, justification TEXT, status TEXT (`pending|exporting|delivered|failed`), s3_url TEXT, expires_at TIMESTAMPTZ, created_at, completed_at, tenant_id)`.
2. **MUST** expose REST `POST /api/dsar/request` with body `{subject_id: <UUID>}` AND auth header. Authorisation:
   - `requested_by == subject_id` (subject self-request): always allowed.
   - `requested_by` is `tenant_admin` AND body includes non-empty `justification`: allowed; logged with extra audit detail.
   - Otherwise: 403.
3. **MUST** the exporter:
   - Query `SELECT * FROM posts WHERE user_id = $subject AND tenant_id = $tenant`; include channels metadata.
   - For each post, fetch chain_anchor from FR-CHAT-005 memory row (joined by `props.memory_anchor`).
4. **MUST** compose the zip:
   - `messages.jsonl`: one JSON line per message `{post_id, channel_id, channel_name, body, created_at, edited_at, memory_chain_anchor}`.
   - `manifest.json`: counts + zip_sha256 + total_messages + export_signing_hash.
   - `README.md`: explains chain_anchor verification procedure for recipient.
   - `verify.sh`: bundled script that recomputes chain hashes locally.
5. **MUST** upload to S3 with KMS encryption (tenant's CMK); generate presigned URL with 7-day TTL.
6. **MUST** deliver URL to subject's registered email + post in their DM channel; URL is one-time-use (S3 access logs detect re-use; revoke on second hit).
7. **MUST** emit memory audit rows:
    - `chat.dsar_requested` on request creation.
    - `chat.dsar_exporting` when export starts.
    - `chat.dsar_delivered` with payload `{subject_id, message_count, zip_sha256, expires_at, requested_by, justification}`.
    - `chat.dsar_failed` on error.
8. **MUST** support up to 100K messages per export; > 100K → split into 100K-msg shards.
9. **MUST** complete within 1h for ≤ 10K messages; sev-2 alarm above.
10. **MUST** include the chain_anchor verification procedure in README.md so subject can independently verify export authenticity using public verification tools.
11. **MUST** RLS-enforce: exporter queries scoped to (subject, tenant).
12. **MUST** emit OTel metrics:
    - `chat_dsar_requests_total{outcome}` (outcome ∈ delivered | failed | over_limit).
    - `chat_dsar_export_duration_seconds`.
    - `chat_dsar_messages_exported` (histogram).
13. **MUST** include all messages the subject WROTE AND messages the subject WAS MENTIONED IN, as two separate sections of the export. PDPL Art. 14 covers "data about the subject" which includes mentions.
14. **MUST** include channel memberships (channels the subject ever joined/left): `channel_memberships.jsonl` with `{channel_id, channel_name, joined_at, left_at}`. Pulls from FR-CHAT-005 `chat.user_joined_channel`/`chat.user_left_channel` memory rows.
15. **MUST** include reactions the subject placed (their emoji reactions to other people's messages) and reactions placed on the subject's messages.
16. **MUST** include file attachments the subject uploaded with full metadata (filename, size, mime, upload timestamp) AND a recoverable reference (file_id) so subjects can request the file content separately if needed.
17. **MUST** include push device registrations (FR-CHAT-011): historical record of which devices the subject registered, with `{platform, registered_at, deregistered_at}`. Tokens themselves NOT included (security: tokens are device credentials).
18. **MUST** include Lumi interactions (FR-CHAT-008): every `chat.lumi_invoked` memory row where the subject was the user_id. Payload preserves both the redacted body AND the response.
19. **MUST** include retro captures (FR-CHAT-009): every memory the subject created OR was a participant in.
20. **MUST** include DSAR history: every prior DSAR request by/about this subject (the audit trail of who has accessed their data).
21. **MUST** encrypt the zip with a passphrase derived from the subject's verified email (HKDF-SHA256); the passphrase is NOT included in the URL or the email — subject derives it on their end from their email + a salt provided in the README. Defence against URL leak.
22. **MUST** include an `export_chain_hash` in the manifest that's the SHA-256 of all included files concatenated (deterministic order). Recipient verifies the export hasn't been tampered with mid-flight.
23. **MUST** support partial-fulfilment: if the export shards (>100K messages), each shard delivers independently; final `chat.dsar_fully_delivered` memory row fires after all shards confirmed.
24. **MUST** complete the export within 30 days from request (GDPR Art. 12(3) max); SLA target 24h. SEV-1 if request older than 25 days still pending.
25. **MUST** offer the subject a "preview" before final delivery: a manifest-only export (no message bodies) so the subject sees the scope of what's being exported. They confirm before the full export runs.
26. **MUST** record an `acknowledgement` from the subject after delivery: subject signs (clicks confirmation) acknowledging receipt; emits `chat.dsar_acknowledged` memory row.
27. **MUST** support tenant-admin "redact-on-export" rules: certain PII categories (medical, financial) MAY be excluded from exports per tenant policy `dsar_redact_categories`. Default = none redacted.

---

## §2 — Why this design

**Why subject-self OR admin-with-justification (DEC-530)?** PDPL allows subject-initiated AND business-need-with-documentation. Admin path needs justification logged — auditor reviews later.

**Why chain_anchor in export (DEC-531)?** Subject verifies "this is the actual content the company had on Apr 12" by recomputing hash. Tampering after export becomes detectable.

**Why short-lived S3 URL (DEC-532)?** Long-lived URLs leak (forwarded emails, screenshots). 7-day window matches recipient handling time without indefinite exposure.

**Why one-time-use URL (§1 #6)?** Detects compromise (someone else accessed the link); revokes immediately.

**Why bundled verify.sh (§1 #10)?** Subject shouldn't depend on CyberOS infra to verify; standalone script works offline.

**Why 100K shard cap (§1 #8)?** Single zip > 100K messages = > 100MB zip = email-delivery friction. Sharding keeps each piece manageable.

**Why include mentions (§1 #13)?** PDPL/GDPR's "data about the subject" includes mentions — Alice was discussed even when Alice didn't write. Compliance regulators consistently interpret this broadly.

**Why memberships + reactions + files (§1 #14-16)?** Each is an independently meaningful data point about the subject's activity. A complete DSAR is "everything the company knows," not just message bodies.

**Why exclude tokens from device export (§1 #17)?** Device tokens are credentials. Including them in a DSAR creates a credential-leak path. Operators report on existence-of-registration, not the tokens themselves.

**Why Lumi interactions (§1 #18)?** LLM interactions reveal what the subject asked + what the LLM said. PDPL recognises AI interactions as personal data under recent guidance.

**Why DSAR history (§1 #20)?** Subjects need to know "who has requested my data and when" as a meta-right. This is the audit-of-audits.

**Why passphrase-protect (§1 #21)?** S3 URL leak (forwarded email, screenshot, browser history) would expose the DSAR contents. Passphrase derived from subject email = only subject can decrypt even if URL leaks. The salt in README forces a deliberate derivation step (defence against blind script-scraping of URLs).

**Why export_chain_hash (§1 #22)?** Per-message chain anchors verify each message; export_chain_hash verifies the export as a whole hasn't been tampered with. Belt + suspenders.

**Why partial-fulfilment (§1 #23)?** Subjects want to start reviewing immediately; sharding lets them begin with shard 1 while shards 2-5 are still generating.

**Why 30-day SLA (§1 #24)?** GDPR Art. 12(3) hard limit; PDPL similar (typically 30 days). SLA target 24h is best-practice; the 30-day max prevents legal exposure.

**Why preview-before-full (§1 #25)?** A subject who sees "5,000 messages including 200 from #legal" may want to refine scope ("just my own messages, please") before committing to download.

**Why subject acknowledgement (§1 #26)?** Closes the loop: we know the subject received the export. Without ack, "delivered" only means "URL sent." Acknowledgement establishes legal compliance receipt.

**Why redact-on-export rules (§1 #27)?** Some tenants are subject to additional regulations (HIPAA for healthcare, PCI for payments) that prohibit certain categories from leaving the system even to the subject themselves (the subject's lawyer might leak; the redaction acts as belt-and-suspenders).

---

## §3 — API contract

```sql
-- services/chat/sql/init-dsar-requests.sql
CREATE TABLE dsar_requests (
    id                    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subject_id            UUID NOT NULL,
    requested_by          UUID NOT NULL,
    justification         TEXT,
    request_mode          TEXT NOT NULL DEFAULT 'full' CHECK (request_mode IN ('preview','full')),
    preview_acknowledged  BOOLEAN NOT NULL DEFAULT false,
    redact_categories     TEXT[] NOT NULL DEFAULT '{}',
    status                TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending','preview_ready','exporting','delivered','failed','acknowledged','expired')),
    shard_count           INT NOT NULL DEFAULT 1,
    shards_delivered      INT NOT NULL DEFAULT 0,
    s3_urls               TEXT[] NOT NULL DEFAULT '{}',
    expires_at            TIMESTAMPTZ,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    preview_delivered_at  TIMESTAMPTZ,
    completed_at          TIMESTAMPTZ,
    acknowledged_at       TIMESTAMPTZ,
    error_message         TEXT,
    sla_deadline          TIMESTAMPTZ NOT NULL DEFAULT NOW() + INTERVAL '30 days',
    tenant_id             UUID NOT NULL
);

CREATE INDEX idx_dsar_pending      ON dsar_requests (status) WHERE status = 'pending';
CREATE INDEX idx_dsar_sla_at_risk  ON dsar_requests (sla_deadline) WHERE status NOT IN ('delivered','acknowledged','expired','failed');
CREATE INDEX idx_dsar_subject      ON dsar_requests (subject_id, created_at DESC);

ALTER TABLE dsar_requests ENABLE ROW LEVEL SECURITY;
CREATE POLICY dsar_tenant_iso ON dsar_requests
    USING       (tenant_id = current_setting('app.tenant_id')::uuid)
    WITH CHECK  (tenant_id = current_setting('app.tenant_id')::uuid);

REVOKE UPDATE, DELETE ON dsar_requests FROM cyberos_app;
GRANT  INSERT, SELECT, UPDATE ON dsar_requests TO cyberos_dsar_worker;

-- Append-only access logs for S3 URL accesses (one-time-use detection).
CREATE TABLE dsar_url_access_log (
    request_id    UUID NOT NULL REFERENCES dsar_requests(id),
    shard_index   INT NOT NULL,
    accessed_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    accessor_ip   INET,
    accessor_ua   TEXT,
    PRIMARY KEY (request_id, shard_index, accessed_at)
);

REVOKE UPDATE, DELETE ON dsar_url_access_log FROM cyberos_app;
GRANT  INSERT, SELECT  ON dsar_url_access_log TO cyberos_dsar_worker;

-- Per-tenant redaction policy.
ALTER TABLE cyberos_chat_tenant_settings ADD COLUMN IF NOT EXISTS
    dsar_redact_categories TEXT[] NOT NULL DEFAULT '{}';
```

```rust
// services/chat-dsar/src/exporter.rs
pub async fn run_export(req_id: uuid::Uuid) -> anyhow::Result<()> {
    let req = fetch_request(req_id).await?;
    update_status(&req.id, "exporting", None, None).await?;
    emit_memory_row("chat.dsar_exporting", json!({ "request_id": req.id })).await;

    let messages = sqlx::query!(
        r#"SELECT p.id, p.channel_id, c.name as channel_name,
                  p.message, p.create_at, p.update_at,
                  p.props->>'memory_anchor' as memory_anchor
           FROM posts p JOIN channels c ON c.id = p.channel_id
           WHERE p.user_id = $1 AND p.delete_at IS NULL"#,
        req.subject_id
    ).fetch_all(&pool).await?;

    if messages.len() > 100_000 {
        return shard_export(req, messages).await;
    }

    let zip_bytes = compose_zip(&messages, &req).await?;
    let zip_sha = sha256(&zip_bytes);
    let s3_url = upload_to_s3(&zip_bytes, &req, &zip_sha).await?;
    let expires_at = chrono::Utc::now() + chrono::Duration::days(7);

    update_status(&req.id, "delivered", Some(&s3_url), Some(expires_at)).await?;

    emit_memory_row("chat.dsar_delivered", json!({
        "request_id": req.id,
        "subject_id": req.subject_id,
        "message_count": messages.len(),
        "zip_sha256": zip_sha,
        "expires_at": expires_at,
        "requested_by": req.requested_by,
        "justification": req.justification,
        "trace_id": current_trace_id(),
    })).await;
    metrics::counter!("chat_dsar_requests_total", "outcome" => "delivered").increment(1);
    metrics::histogram!("chat_dsar_messages_exported").record(messages.len() as f64);

    notify_subject(&req.subject_id, &s3_url, expires_at).await?;
    Ok(())
}

async fn compose_zip(messages: &[Message], req: &DsarRequest) -> anyhow::Result<Vec<u8>> {
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    // messages.jsonl
    zip.start_file("messages.jsonl", Default::default())?;
    for m in messages {
        let line = serde_json::to_string(&MessageExport {
            post_id: m.id.clone(),
            channel_id: m.channel_id.clone(),
            channel_name: m.channel_name.clone(),
            body: m.message.clone(),
            created_at: m.create_at,
            edited_at: m.update_at,
            memory_chain_anchor: m.memory_anchor.clone(),
        })?;
        writeln!(zip, "{line}")?;
    }
    // manifest.json
    zip.start_file("manifest.json", Default::default())?;
    let manifest = Manifest {
        export_id: req.id,
        subject_id: req.subject_id,
        total_messages: messages.len(),
        generated_at: chrono::Utc::now(),
        verification_endpoint: "https://verify.cyberos.world/memory-chain",
    };
    zip.write_all(&serde_json::to_vec_pretty(&manifest)?)?;
    // README.md
    zip.start_file("README.md", Default::default())?;
    zip.write_all(include_bytes!("../templates/dsar-readme.md"))?;
    // verify.sh
    zip.start_file("verify.sh", Default::default())?;
    zip.write_all(include_bytes!("../templates/verify.sh"))?;

    Ok(zip.finish()?.into_inner())
}
```

```bash
# templates/verify.sh (bundled in zip)
#!/usr/bin/env bash
# Recompute the chain hash for each exported message; print PASS/FAIL.
# Also verifies the export-wide hash against the manifest.
set -euo pipefail

PASS=0
FAIL=0

# Per-message chain anchors.
while IFS= read -r line; do
    post_id=$(echo "$line" | jq -r '.post_id')
    body=$(echo "$line" | jq -r '.body')
    anchor=$(echo "$line" | jq -r '.memory_chain_anchor')
    if [[ "$anchor" == "null" ]]; then
        echo "SKIP: $post_id (no chain anchor; pre-FR-CHAT-005 message)"
        continue
    fi
    computed=$(printf '%s' "$body" | sha256sum | cut -d' ' -f1)
    if [[ "$anchor" == "$computed" ]]; then
        echo "PASS: $post_id"
        PASS=$((PASS+1))
    else
        echo "FAIL: $post_id (expected $anchor, got $computed)"
        FAIL=$((FAIL+1))
    fi
done < messages.jsonl

# Export-wide hash.
expected_export_hash=$(jq -r '.export_chain_hash' manifest.json)
computed_export_hash=$(cat messages.jsonl mentions.jsonl channel_memberships.jsonl \
    reactions.jsonl files.jsonl device_registrations.jsonl lumi_interactions.jsonl \
    retro_captures.jsonl dsar_history.jsonl 2>/dev/null | sha256sum | cut -d' ' -f1)
if [[ "$expected_export_hash" == "$computed_export_hash" ]]; then
    echo "EXPORT_HASH: PASS"
else
    echo "EXPORT_HASH: FAIL (expected $expected_export_hash, got $computed_export_hash)"
    FAIL=$((FAIL+1))
fi

echo "---"
echo "Verification summary: $PASS passed, $FAIL failed"
exit $([[ "$FAIL" -gt 0 ]] && echo 1 || echo 0)
```

### exporter.rs — multi-section composition

```rust
// services/chat-dsar/src/exporter.rs
pub async fn compose_full_export(
    pool: &sqlx::PgPool,
    req: &DsarRequest,
) -> anyhow::Result<Vec<u8>> {
    let messages       = fetch_messages_authored(pool, req).await?;
    let mentions       = fetch_messages_mentioning(pool, req).await?;
    let memberships    = fetch_channel_memberships(pool, req).await?;
    let reactions      = fetch_reactions_by_subject(pool, req).await?;
    let files          = fetch_files_uploaded(pool, req).await?;
    let devices        = fetch_device_registrations(pool, req).await?;
    let lumi           = fetch_lumi_interactions(pool, req).await?;
    let retro          = fetch_retro_captures(pool, req).await?;
    let dsar_history   = fetch_dsar_history(pool, req).await?;

    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let opts = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    write_jsonl(&mut zip, "messages.jsonl",              &messages,    opts)?;
    write_jsonl(&mut zip, "mentions.jsonl",              &mentions,    opts)?;
    write_jsonl(&mut zip, "channel_memberships.jsonl",   &memberships, opts)?;
    write_jsonl(&mut zip, "reactions.jsonl",             &reactions,   opts)?;
    write_jsonl(&mut zip, "files.jsonl",                 &files,       opts)?;
    write_jsonl(&mut zip, "device_registrations.jsonl",  &devices,     opts)?;
    write_jsonl(&mut zip, "lumi_interactions.jsonl",     &lumi,        opts)?;
    write_jsonl(&mut zip, "retro_captures.jsonl",        &retro,       opts)?;
    write_jsonl(&mut zip, "dsar_history.jsonl",          &dsar_history, opts)?;

    // Per-tenant redaction.
    if !req.redact_categories.is_empty() {
        // (Apply redaction inline; produce a `redacted_categories.json` describing what was removed.)
        apply_redactions(&mut zip, &req.redact_categories)?;
    }

    // Manifest with export_chain_hash.
    let manifest = build_manifest(req, &messages, &mentions, &memberships, &reactions,
                                  &files, &devices, &lumi, &retro, &dsar_history)?;
    zip.start_file("manifest.json", opts)?;
    zip.write_all(&serde_json::to_vec_pretty(&manifest)?)?;

    zip.start_file("README.md", opts)?;
    zip.write_all(render_readme(req, &manifest).as_bytes())?;

    zip.start_file("verify.sh", opts.unix_permissions(0o755))?;
    zip.write_all(include_bytes!("../templates/verify.sh"))?;

    Ok(zip.finish()?.into_inner())
}

fn build_manifest(
    req: &DsarRequest,
    messages: &[MessageExport],
    mentions: &[MessageExport],
    memberships: &[MembershipExport],
    reactions: &[ReactionExport],
    files: &[FileExport],
    devices: &[DeviceExport],
    lumi: &[LumiExport],
    retro: &[RetroExport],
    dsar_history: &[DsarHistoryExport],
) -> anyhow::Result<Manifest> {
    let mut hasher = sha2::Sha256::new();
    for items in [
        &serde_json::to_vec(messages)?,
        &serde_json::to_vec(mentions)?,
        &serde_json::to_vec(memberships)?,
        &serde_json::to_vec(reactions)?,
        &serde_json::to_vec(files)?,
        &serde_json::to_vec(devices)?,
        &serde_json::to_vec(lumi)?,
        &serde_json::to_vec(retro)?,
        &serde_json::to_vec(dsar_history)?,
    ] {
        hasher.update(items);
    }
    let export_chain_hash = hex::encode(hasher.finalize());

    Ok(Manifest {
        export_id: req.id,
        subject_id: req.subject_id,
        tenant_id: req.tenant_id,
        generated_at: chrono::Utc::now(),
        generated_by: req.requested_by,
        justification: req.justification.clone(),
        sections: ManifestSections {
            messages_count: messages.len() as i64,
            mentions_count: mentions.len() as i64,
            memberships_count: memberships.len() as i64,
            reactions_count: reactions.len() as i64,
            files_count: files.len() as i64,
            devices_count: devices.len() as i64,
            lumi_interactions_count: lumi.len() as i64,
            retro_captures_count: retro.len() as i64,
            dsar_history_count: dsar_history.len() as i64,
        },
        verification_endpoint: "https://verify.cyberos.world/memory-chain".into(),
        export_chain_hash,
        salt_for_passphrase: hex::encode(rand::random::<[u8; 32]>()),
        passphrase_derivation: "HKDF-SHA256(input=subject_email, salt=<salt>, info='cyberos-dsar-v1')".into(),
    })
}
```

### preview.rs — preview mode

```rust
// services/chat-dsar/src/preview.rs
pub async fn compose_preview(
    pool: &sqlx::PgPool,
    req: &DsarRequest,
) -> anyhow::Result<Vec<u8>> {
    // Same shape as full export but bodies are SHA-256 hashes, not content.
    let messages = fetch_messages_authored_count_only(pool, req).await?;
    let manifest = serde_json::json!({
        "preview_mode": true,
        "subject_id": req.subject_id,
        "messages_count": messages.total,
        "mentions_count": messages.mention_total,
        "channels_referenced": messages.channels,
        "earliest_message_at": messages.earliest,
        "latest_message_at": messages.latest,
        "estimated_export_size_bytes": messages.total * 500,
        "note": "This is a preview. Click `Confirm` to receive the full export."
    });
    Ok(serde_json::to_vec_pretty(&manifest)?)
}
```

### S3 upload + presigned URL

```rust
pub async fn upload_to_s3(
    bytes: &[u8],
    req: &DsarRequest,
    tenant_cmk_arn: &str,
    shard: i32,
) -> anyhow::Result<String> {
    let key = format!("dsar/{}/{}-{:02}.zip", req.tenant_id, req.id, shard);
    let s3 = aws_sdk_s3::Client::new(&aws_config::load_defaults(BehaviorVersion::latest()).await);
    s3.put_object()
        .bucket(&config().dsar_bucket)
        .key(&key)
        .body(bytes.to_vec().into())
        .server_side_encryption(ServerSideEncryption::AwsKms)
        .ssekms_key_id(tenant_cmk_arn)
        .send().await?;
    let presigned = s3.get_object()
        .bucket(&config().dsar_bucket)
        .key(&key)
        .presigned(PresigningConfig::expires_in(std::time::Duration::from_secs(7 * 24 * 3600))?)
        .await?;
    Ok(presigned.uri().to_string())
}
```

### One-time-use detection (via S3 access log → CloudWatch → Lambda)

```python
# services/chat-dsar/lambda/url_access_alarm.py
def handler(event, _):
    """Triggered by CloudWatch on each S3 GetObject for dsar/ prefix.
    If access count > 1, alarm + invalidate URL."""
    for record in event['Records']:
        bucket = record['s3']['bucket']['name']
        key = record['s3']['object']['key']
        request_id = key.split('/')[2].split('-')[0]
        # Check our access log.
        count = postgres_count_for(request_id, key)
        log_access(request_id, key, record['requestParameters']['sourceIPAddress'])
        if count >= 1:
            # Second access. Revoke + alarm.
            s3.put_object_acl(Bucket=bucket, Key=key, ACL='private')
            emit_memory_row('chat.dsar_url_reused', {
                'request_id': request_id,
                'shard': key,
                'accessor_ip': record['requestParameters']['sourceIPAddress'],
                'severity': 'SEV-1',
            })
            send_pagerduty_alarm('dsar_url_reuse_detected', request_id)
```

---

## §4 — Acceptance criteria

1. **Subject self-request authorised** — POST with subject_id == auth.sub → 201.
2. **Admin self-request another's data fails without justification** — 403.
3. **Admin with justification authorised** — 201; justification logged.
4. **Non-admin third-party rejected** — 403.
5. **Export contains messages.jsonl + manifest.json + README.md + verify.sh**.
6. **Each message has memory_chain_anchor field**.
7. **Zip uploaded to S3 with KMS encryption**.
8. **Signed URL TTL = 7 days**.
9. **One-time use detected** — second access → S3 revokes; sev-1 audit.
10. **memory audit chat.dsar_delivered with zip_sha256**.
11. **memory audit on failure**.
12. **Shard at > 100K messages**.
13. **Export latency p95 < 1h for 10K msgs**.
14. **verify.sh bundled and executable** — extract zip → bash verify.sh → PASS lines.
15. **Subject notified via DM + email**.
16. **RLS isolates per-tenant**.
17. **Mentions section included** — fixture: subject is mentioned in 5 posts; export `mentions.jsonl` has 5 entries (AC for §1 #13).
18. **Channel memberships included** — fixture: subject joined+left 3 channels; export `channel_memberships.jsonl` has 3 entries with joined_at + left_at (AC for §1 #14).
19. **Reactions in both directions** — fixture: subject placed 4 reactions + received 7; export `reactions.jsonl` has 11 entries with direction flag (AC for §1 #15).
20. **Files uploaded with metadata** — fixture: subject uploaded 3 files; `files.jsonl` has 3 entries with filename + size + mime (AC for §1 #16).
21. **Device tokens NOT included** — fixture: subject registered 2 devices; `device_registrations.jsonl` has 2 entries but `device_token` field absent (AC for §1 #17).
22. **Lumi interactions included** — fixture: subject @lumi'd 5 times; `lumi_interactions.jsonl` has 5 entries with redacted body + response (AC for §1 #18).
23. **Retro captures included** — fixture: subject made 2 retro captures; `retro_captures.jsonl` lists them (AC for §1 #19).
24. **DSAR history included** — fixture: subject had 2 prior DSARs; `dsar_history.jsonl` has 2 entries (AC for §1 #20).
25. **Passphrase derivation documented in README** — observe README explains HKDF + salt; passphrase derivation reproducible (AC for §1 #21).
26. **export_chain_hash in manifest** — observe manifest contains SHA-256 of concatenated section files; verify.sh recomputes match (AC for §1 #22).
27. **Sharded export chains across shards** — fixture: 250K messages → 3 shards; each shard delivered separately; `chat.dsar_fully_delivered` audit after all 3 acked (AC for §1 #23).
28. **SLA deadline 30 days from request** — observe `sla_deadline = created_at + 30 days`; pending request older than 25 days → SEV-1 alarm (AC for §1 #24).
29. **Preview mode delivers manifest only** — fixture: request with mode=preview; export contains only manifest.json with counts + no bodies (AC for §1 #25).
30. **Acknowledgement recorded** — subject clicks confirm; `chat.dsar_acknowledged` row emitted; `acknowledged_at` populated (AC for §1 #26).
31. **Tenant redact policy honoured** — set `dsar_redact_categories=['financial']`; export excludes messages tagged with PII category 'financial'; `redacted_categories.json` lists what was removed (AC for §1 #27).

---

## §5 — Verification

### AC #1/#2/#3/#4 — authorisation

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac1_subject_self_request_succeeds() {
    let env = TestEnv::new().await;
    let req = post_dsar_request(env.subject_id(), env.subject_id(), None).await.unwrap();
    assert_eq!(req.status, "pending");
    run_export(req.id).await.unwrap();
    let final_req = env.fetch_request(req.id).await;
    assert_eq!(final_req.status, "delivered");
}

#[tokio::test(flavor = "multi_thread")]
async fn ac2_admin_without_justification_rejected() {
    let env = TestEnv::new().await;
    let admin = env.create_admin().await;
    let result = post_dsar_request(admin.id, env.subject_id(), None).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("justification required"));
}

#[tokio::test(flavor = "multi_thread")]
async fn ac3_admin_with_justification_authorised() {
    let env = TestEnv::new().await;
    let admin = env.create_admin().await;
    let req = post_dsar_request(admin.id, env.subject_id(),
        Some("ex-employee deletion under PDPL Art 18".into())).await.unwrap();
    run_export(req.id).await.unwrap();
    let row = env.memory.last_of_kind("chat.dsar_delivered").await.unwrap();
    assert_eq!(row["payload"]["justification"], "ex-employee deletion under PDPL Art 18");
    assert_eq!(row["payload"]["requested_by"], admin.id);
}

#[tokio::test(flavor = "multi_thread")]
async fn ac4_third_party_rejected() {
    let env = TestEnv::new().await;
    let other_user = env.create_user("bob").await;
    let result = post_dsar_request(other_user.id, env.subject_id(), Some("just curious".into())).await;
    assert!(result.is_err());
}
```

### AC #5/#6 — zip contents

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac5_zip_contains_required_files() {
    let env = TestEnv::new().await;
    let zip_bytes = compose_full_export(&env.pool, &fixture_request()).await.unwrap();
    let zip = zip::ZipArchive::new(std::io::Cursor::new(zip_bytes)).unwrap();
    let names: Vec<_> = zip.file_names().collect();
    for required in &["messages.jsonl", "manifest.json", "README.md", "verify.sh",
                      "mentions.jsonl", "channel_memberships.jsonl", "reactions.jsonl",
                      "files.jsonl", "device_registrations.jsonl",
                      "lumi_interactions.jsonl", "retro_captures.jsonl", "dsar_history.jsonl"] {
        assert!(names.contains(&required), "missing {}", required);
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn ac6_messages_have_chain_anchor() {
    let env = TestEnv::new().await;
    env.seed_messages_for(env.subject_id(), 5).await;
    let zip_bytes = compose_full_export(&env.pool, &env.test_request()).await.unwrap();
    let messages_jsonl = extract_file(&zip_bytes, "messages.jsonl");
    for line in messages_jsonl.lines() {
        let m: MessageExport = serde_json::from_str(line).unwrap();
        assert!(m.memory_chain_anchor.is_some(), "post {} has no chain anchor", m.post_id);
    }
}
```

### AC #14 — bundled verify.sh

```bash
#!/usr/bin/env bash
# tests/verify-sh-integration.sh
set -e
TMPDIR=$(mktemp -d)
cyberos-chat-dsar request --tenant test-t --subject test-s
ZIP=$(get-latest-dsar-zip test-s)
unzip -d "$TMPDIR" "$ZIP"
cd "$TMPDIR"
bash verify.sh | grep -q "EXPORT_HASH: PASS"
```

### AC #17 — mentions section

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac17_mentions_section_included() {
    let env = TestEnv::new().await;
    let subject = env.create_user("alice").await;
    let other = env.create_user("bob").await;
    for _ in 0..5 { env.post_message_by(&other.id, "hi @alice").await; }
    let zip = compose_full_export(&env.pool, &dsar_for(subject.id)).await.unwrap();
    let mentions_jsonl = extract_file(&zip, "mentions.jsonl");
    let count = mentions_jsonl.lines().count();
    assert_eq!(count, 5);
}
```

### AC #21 — device tokens excluded

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac21_device_tokens_excluded() {
    let env = TestEnv::new().await;
    env.register_device(env.subject_id(), "apns", "secret-token-123").await;
    let zip = compose_full_export(&env.pool, &env.test_request()).await.unwrap();
    let devices_jsonl = extract_file(&zip, "device_registrations.jsonl");
    assert!(devices_jsonl.contains("apns"));
    assert!(!devices_jsonl.contains("secret-token-123"));
}
```

### AC #25 — passphrase derivation in README

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac25_passphrase_derivation_documented() {
    let env = TestEnv::new().await;
    let zip = compose_full_export(&env.pool, &env.test_request()).await.unwrap();
    let readme = extract_file(&zip, "README.md");
    assert!(readme.contains("HKDF-SHA256"));
    assert!(readme.contains("salt"));
    let manifest_str = extract_file(&zip, "manifest.json");
    let manifest: serde_json::Value = serde_json::from_str(&manifest_str).unwrap();
    assert!(manifest["salt_for_passphrase"].as_str().unwrap().len() == 64);
}
```

### AC #26 — export_chain_hash

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac26_export_chain_hash_matches() {
    let env = TestEnv::new().await;
    let zip = compose_full_export(&env.pool, &env.test_request()).await.unwrap();
    let manifest_str = extract_file(&zip, "manifest.json");
    let manifest: serde_json::Value = serde_json::from_str(&manifest_str).unwrap();
    let expected_hash = manifest["export_chain_hash"].as_str().unwrap();
    // Recompute as verify.sh would.
    let concat: String = ["messages.jsonl","mentions.jsonl","channel_memberships.jsonl",
                          "reactions.jsonl","files.jsonl","device_registrations.jsonl",
                          "lumi_interactions.jsonl","retro_captures.jsonl","dsar_history.jsonl"]
        .iter().map(|n| extract_file(&zip, n)).collect();
    let computed = sha2::Sha256::digest(concat.as_bytes());
    assert_eq!(expected_hash, hex::encode(computed));
}
```

### AC #27 — sharding

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac27_sharding_at_100k() {
    let env = TestEnv::new().await;
    env.seed_messages_for(env.subject_id(), 250_000).await;
    let req = env.create_dsar_request().await;
    run_export(req.id).await.unwrap();
    let final_req = env.fetch_request(req.id).await;
    assert_eq!(final_req.shard_count, 3);
    assert_eq!(final_req.shards_delivered, 3);
    let row = env.memory.last_of_kind("chat.dsar_fully_delivered").await.unwrap();
    assert_eq!(row["payload"]["shard_count"], 3);
}
```

### AC #28 — SLA deadline

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac28_sla_deadline() {
    let env = TestEnv::new().await;
    let req = env.create_dsar_request().await;
    assert!((req.sla_deadline - req.created_at).num_days() == 30);

    // Simulate 26 days passing without completion.
    env.set_request_age(req.id, chrono::Duration::days(26)).await;
    env.run_sla_check().await;
    let alarm = env.obs.latest_alert().await;
    assert_eq!(alarm.severity, "SEV-1");
    assert_eq!(alarm.kind, "dsar_sla_at_risk");
}
```

### AC #29 — preview mode

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac29_preview_mode() {
    let env = TestEnv::new().await;
    env.seed_messages_for(env.subject_id(), 1000).await;
    let req = env.create_dsar_request_with_mode("preview").await;
    run_export(req.id).await.unwrap();
    let zip = env.fetch_zip(req.id).await;
    let names: HashSet<_> = zip_file_names(&zip).into_iter().collect();
    assert!(names.contains(&"manifest.json".to_string()));
    assert!(!names.contains(&"messages.jsonl".to_string()));
    let manifest: serde_json::Value = serde_json::from_slice(&extract_bytes(&zip, "manifest.json")).unwrap();
    assert_eq!(manifest["preview_mode"], true);
    assert_eq!(manifest["messages_count"], 1000);
}
```

### AC #30 — acknowledgement

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac30_acknowledgement_recorded() {
    let env = TestEnv::new().await;
    let req = env.deliver_dsar().await;
    post_dsar_ack(req.id, env.subject_id()).await.unwrap();
    let final_req = env.fetch_request(req.id).await;
    assert_eq!(final_req.status, "acknowledged");
    assert!(final_req.acknowledged_at.is_some());
    let row = env.memory.last_of_kind("chat.dsar_acknowledged").await.unwrap();
    assert_eq!(row["payload"]["request_id"], req.id.to_string());
}
```

### AC #31 — redact-on-export

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac31_redact_categories_applied() {
    let env = TestEnv::new().await;
    env.set_tenant_redact_categories(env.tenant_id(), vec!["financial".into()]).await;
    env.post_message_with_pii_category(env.subject_id(), "salary is 5000 USD", "financial").await;
    env.post_message_with_pii_category(env.subject_id(), "hello world", "none").await;
    let zip = compose_full_export(&env.pool, &env.test_request()).await.unwrap();
    let messages = extract_file(&zip, "messages.jsonl");
    assert!(!messages.contains("salary is 5000 USD"));
    assert!(messages.contains("hello world"));
    let redact_meta = extract_file(&zip, "redacted_categories.json");
    let r: serde_json::Value = serde_json::from_str(&redact_meta).unwrap();
    assert!(r["categories"].as_array().unwrap().iter().any(|v| v == "financial"));
}
```

### One-time-use detection (integration)

```bash
#!/usr/bin/env bash
# tests/url-reuse-integration.sh
URL=$(create-dsar-and-get-url)
curl -o /tmp/dsar1.zip "$URL"  # First access
curl -o /tmp/dsar2.zip "$URL"  # Second access (should fail or trigger alarm)
sleep 5
memory_query --kind chat.dsar_url_reused | jq -e '.payload.severity == "SEV-1"'
```

---

## §6 — Implementation skeleton

The Rust modules + Lambda above are the skeleton. Operational wiring:

### §6.1 — Service deployment

The chat-dsar service runs as a Fargate task in each tenant's CHAT cluster (FR-CHAT-003). It exposes:
- `POST /api/dsar/request` — create request (returns request_id + status).
- `GET /api/dsar/:id` — request status.
- `POST /api/dsar/:id/confirm-preview` — confirm preview and start full export.
- `POST /api/dsar/:id/ack` — subject acknowledgement.
- `GET /api/dsar/:id/url/:shard` — generate signed URL for a specific shard.

A background worker polls `dsar_requests WHERE status='pending'` and runs exports sequentially per tenant.

### §6.2 — Preview-then-full workflow

When a request is created in `preview` mode:
1. Worker generates preview (manifest only); status → `preview_ready`.
2. Subject receives preview URL via email + DM.
3. Subject reviews and clicks "Confirm and download full export" → POST /confirm-preview.
4. Worker generates full export; status → `delivered`.

If the subject doesn't confirm within 7 days, the request expires (status → `expired`); memory audit `chat.dsar_expired` fires.

### §6.3 — Sharding semantics

When `messages_count > 100_000`:
1. Worker creates N shards (100K messages each).
2. Each shard is uploaded independently to S3.
3. Each shard's signed URL is delivered to the subject in order.
4. Subject acks each shard; after final shard ack, `chat.dsar_fully_delivered` fires.

Shard ordering: chronological by message create_at. Shard 1 = oldest messages.

### §6.4 — One-time-use detection

A CloudWatch Logs subscription on the S3 access log triggers a Lambda. The Lambda:
1. Looks up the request_id from the S3 key.
2. Queries `dsar_url_access_log` for prior accesses.
3. If count ≥ 1, the URL is being reused: revoke + SEV-1 alarm + PagerDuty.
4. Logs the access regardless.

This is "log-based" not "request-based" because we can't intercept S3 GETs server-side without violating the presigned URL pattern.

### §6.5 — Passphrase derivation

Operator-side (in the service): the salt is randomly generated per export and stored in manifest.json. Subject-side (via README + verify.sh): subject runs `derive-passphrase.sh <email> <salt>` which outputs the passphrase. Subject then `unzip -P <passphrase> dsar.zip`.

This means even if the URL leaks, the leaker needs the subject's verified email + the salt to decrypt. (Since the salt is in the same zip, this only protects against blind URL access; the real protection is the one-time-use detection.)

### §6.6 — SLA monitoring

Nightly cron runs `SELECT * FROM dsar_requests WHERE sla_deadline < NOW() + INTERVAL '5 days' AND status NOT IN ('delivered','acknowledged')` and emits SEV-1 alarms. Operators have 5-day window to address before SLA breach.

### §6.7 — Failure routing matrix

| Failure | Audit | Operator action |
|---|---|---|
| Export query fails | chat.dsar_failed (SEV-2) | Investigate DB |
| S3 upload fails | retry 3×; then chat.dsar_failed (SEV-2) | Investigate S3/KMS |
| KMS revoked | chat.dsar_failed (SEV-1) | Operator restores key |
| Subject email invalid | chat.dsar_delivered (with email_undelivered note) | Operator follows up |
| URL reused | chat.dsar_url_reused (SEV-1) | Investigate; revoke remaining |
| Preview not confirmed in 7d | chat.dsar_expired | None (subject can re-request) |
| Full export expires (7d TTL) | chat.dsar_expired | Operator notifies subject for re-issue |
| SLA at risk | dsar_sla_at_risk alarm (SEV-1) | Operator expedites |
| Worker crash mid-export | request stuck in `exporting` | next worker tick recovers via timeout |

### §6.8 — Operator CLI

```text
$ cyberos-chat dsar list --tenant <id>
ID                          SUBJECT     STATUS      AGE    SLA-DAYS
01HVQX...                   alice       delivered   2d     28
01HVQY...                   bob         pending     5d     25 [WARN: 5d remaining]

$ cyberos-chat dsar fulfil --subject <id> --tenant <id> --justification "<text>"
✓ Request created: 01HVQZ...

$ cyberos-chat dsar status 01HVQZ...
status:        delivered
shards:        1/1 delivered, 1/1 acked
created:       2026-05-16T14:32Z
delivered:     2026-05-16T15:48Z
acked:         2026-05-16T17:02Z
expires:       2026-05-23T14:32Z
```

---

## §7 — Dependencies

- **FR-CHAT-005** — memory_anchor source.
- **FR-MEMORY-101** — chain verification reference.
- **FR-AUTH-005** — admin role check.

---

## §8 — Example payloads

### `chat.dsar_requested`

```json
{
  "kind": "chat.dsar_requested",
  "ts_ns": 1747407100000000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "trace_id": "0af7651916cd43dd8448eb211c80319c",
  "payload": {
    "request_id":   "01HVQX8ZG2K3R4TVA7P3WV5X8N",
    "subject_id":   "01HVQX8ZG2K3R4TVA7P3WV5X8K",
    "requested_by": "01HVQX8ZG2K3R4TVA7P3WV5X8K",
    "request_mode": "preview",
    "justification": null,
    "sla_deadline": "2026-06-15T14:32:00Z"
  }
}
```

### `chat.dsar_delivered` — preview

```json
{
  "kind": "chat.dsar_delivered",
  "ts_ns": 1747407137000000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "payload": {
    "request_id":   "01HVQX8ZG2K3R4TVA7P3WV5X8N",
    "subject_id":   "01HVQX8ZG2K3R4TVA7P3WV5X8K",
    "request_mode": "preview",
    "messages_count":   4287,
    "mentions_count":     87,
    "memberships_count":  12,
    "reactions_count":    245,
    "files_count":        18,
    "lumi_interactions_count": 24,
    "retro_captures_count":     3,
    "expires_at": "2026-05-23T14:32:00Z",
    "shard_count": 1
  }
}
```

### `chat.dsar_delivered` — full (sharded)

```json
{
  "kind": "chat.dsar_delivered",
  "ts_ns": 1747409137000000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "payload": {
    "request_id":    "01HVQX8ZG2K3R4TVA7P3WV5X8N",
    "subject_id":    "01HVQX8ZG2K3R4TVA7P3WV5X8K",
    "request_mode":  "full",
    "messages_count": 250_000,
    "shard_count":    3,
    "shard_index":    1,
    "zip_sha256":     "9b0e8c5dcafe...",
    "expires_at":    "2026-05-23T14:32:00Z",
    "export_chain_hash": "abc123...",
    "salt_for_passphrase": "ff00...",
    "justification": "ex-employee deletion under PDPL Art 18",
    "requested_by":  "01HVQX8ZG2K3R4TVA7P3WV5X8A"
  }
}
```

### `chat.dsar_fully_delivered` — final shard ack

```json
{
  "kind": "chat.dsar_fully_delivered",
  "ts_ns": 1747410137000000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "payload": {
    "request_id":   "01HVQX8ZG2K3R4TVA7P3WV5X8N",
    "shard_count":   3,
    "shards_delivered": 3,
    "total_duration_seconds": 2400
  }
}
```

### `chat.dsar_acknowledged`

```json
{
  "kind": "chat.dsar_acknowledged",
  "ts_ns": 1747416137000000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "payload": {
    "request_id":  "01HVQX8ZG2K3R4TVA7P3WV5X8N",
    "subject_id":  "01HVQX8ZG2K3R4TVA7P3WV5X8K",
    "ack_ip":      "203.0.113.42"
  }
}
```

### `chat.dsar_url_reused`

```json
{
  "kind": "chat.dsar_url_reused",
  "ts_ns": 1747420137000000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "severity": "SEV-1",
  "payload": {
    "request_id":  "01HVQX8ZG2K3R4TVA7P3WV5X8N",
    "s3_key":      "dsar/.../01HVQX...-01.zip",
    "accessor_ip": "198.51.100.7",
    "previous_access_at": "2026-05-16T15:48:00Z",
    "current_access_at":  "2026-05-17T03:21:00Z"
  }
}
```

### Manifest excerpt

```json
{
  "export_id": "01HVQX8ZG2K3R4TVA7P3WV5X8N",
  "subject_id": "01HVQX8ZG2K3R4TVA7P3WV5X8K",
  "tenant_id":  "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "generated_at": "2026-05-16T14:32:17Z",
  "generated_by": "01HVQX8ZG2K3R4TVA7P3WV5X8K",
  "justification": null,
  "sections": {
    "messages_count":            4287,
    "mentions_count":            87,
    "memberships_count":         12,
    "reactions_count":           245,
    "files_count":               18,
    "devices_count":             3,
    "lumi_interactions_count":   24,
    "retro_captures_count":      3,
    "dsar_history_count":        1
  },
  "verification_endpoint": "https://verify.cyberos.world/memory-chain",
  "export_chain_hash": "abc123def456abc123def456abc123def456abc123def456abc123def456abc1",
  "salt_for_passphrase": "ff00ee11dd22cc33bb44aa55ff00ee11dd22cc33bb44aa55ff00ee11dd22cc33",
  "passphrase_derivation": "HKDF-SHA256(input=subject_email, salt=<salt>, info='cyberos-dsar-v1')"
}
```

### README excerpt

```markdown
# Your CyberOS Data Subject Access Request

Hi! This zip contains all the data CyberOS holds about you in our chat service.

## What's inside

- `messages.jsonl`     — every message you wrote.
- `mentions.jsonl`     — every message that mentioned you.
- `channel_memberships.jsonl` — your channel join/leave history.
- `reactions.jsonl`    — emoji reactions you placed AND reactions on your messages.
- `files.jsonl`        — file attachments you uploaded.
- `device_registrations.jsonl` — mobile devices you registered (tokens redacted).
- `lumi_interactions.jsonl` — your @lumi conversations.
- `retro_captures.jsonl` — memories you captured.
- `dsar_history.jsonl` — every previous DSAR about you.
- `manifest.json`      — metadata + verification hashes.
- `verify.sh`          — script to verify tamper-evidence.

## How to unlock

This zip is encrypted. Derive your passphrase:

```bash
derive-passphrase.sh "<your-email>" "<salt-from-manifest>"
```

Then `unzip -P <passphrase> dsar.zip`.

## How to verify

```bash
bash verify.sh
```

You should see `EXPORT_HASH: PASS`. If you see `FAIL`, contact us immediately.

## Your rights

You can request data deletion (right to erasure) by emailing legal@cyberskill.world.
You acknowledge receipt of this export by visiting:
https://chat.cyberskill.world/dsar/ack/01HVQX8ZG2K3R4TVA7P3WV5X8N
```

---

## §9 — Open questions

All resolved. Deferred:
- Cross-module DSAR (chat + project + crm in one export) — slice 4+; complex schema.
- DSAR for deleted subjects (account closed) — slice 4+; legal holds.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Subject has no messages | empty sections; manifest still created | delivered with counts=0 | None |
| > 100K messages | sharding triggered | multiple zips delivered sequentially | None |
| > 1M messages (extreme) | sharding produces 10+ shards | sequential delivery; SEV-3 if >24h total | Operator monitors |
| S3 upload fails (network) | sdk Err | retry 3×; final = failed; SEV-2 | Operator |
| S3 upload fails (quota) | S3 quota error | SEV-1 | Operator increases quota |
| KMS key revoked | encryption Err | SEV-1; export aborts | Operator restores |
| KMS key rotation mid-upload | upload uses old key version | re-upload on retry | None |
| URL re-accessed | S3 access log → Lambda | revoke + SEV-1 audit + PagerDuty | Operator investigates source |
| URL re-accessed by same IP within 1s (double-click) | suppress | first access counts; second ignored | None |
| Subject email invalid | SendGrid Err | URL delivered via DM only; SEV-3 warning | Operator follows up |
| Subject DM channel doesn't exist | MM API Err | URL stays in S3; SEV-3 + operator email | Operator |
| Tenant deleted mid-export | RLS catches | export aborts; SEV-2 | Operator |
| Tenant data residency change mid-export | export uses old region | inconsistent; rare | Operator |
| memory chain_anchor missing for pre-FR-CHAT-005 messages | anchor field is null | export degraded; SKIP in verify | Operator backfills (slice-4+) |
| memory chain query slow (> 10s for 100K messages) | timeout | export degraded with partial anchors | Investigate memory |
| Export latency > 1h for 10K messages | SEV-2 latency alarm | None visible to subject (still delivered) | Operator investigates |
| Export latency > 24h for 100K | SEV-1 | Operator manually checks worker | Operator |
| memory audit emit fails | export delivered; audit lost | SEV-2 logged | Operator restores |
| verify.sh has bug | recipient sees FAIL false-positive | embarrassing | Author fixes + reissue |
| Justification too long (>10KB) | TEXT type accepts; UI truncates display | None | None |
| Concurrent requests for same subject | each independent | both deliver | None |
| Concurrent requests by same subject (self + admin) | each independent | two exports | None |
| Subject acks before all shards delivered | ack waits for fully_delivered | ack rejected with "shards remaining" | None |
| Subject doesn't ack within SLA | reminder email + audit | escalates to legal team after 14d | Operator |
| Preview confirm received after preview expired | expired status; new preview required | UI shows "Preview expired, re-request" | None |
| Subject changes email mid-preview | passphrase derivation broken | subject re-derives with old email OR re-request | None |
| Subject lost passphrase | re-request from scratch | New salt issued | None |
| Subject's MM user soft-deleted but RLS-visible | export proceeds | None | None |
| Subject's MM user hard-deleted | RLS returns empty | export delivers with empty sections + note | None |
| Section query partial fail (e.g. lumi service down) | that section is empty + note in manifest | partial delivery | Operator |
| Files section: file blob deleted | section has metadata but no recoverable file | None visible to subject | None |
| Channel renamed mid-export | export uses current name | None | None |
| Tenant settings (redact policy) changes mid-export | export uses snapshot at start | None | None |
| Redact policy excludes ALL messages | export delivers with redact note only | subject sees empty export | Operator |
| Compliance category mismatch (tag exists but category not in policy) | included | None | Operator updates policy |
| Subject was renamed/merged | export uses current subject_id | historical messages by old id missing | Operator |
| Subject was renamed across tenants | export per-tenant only | other-tenant data not included | None (per-tenant DSAR by design) |
| Cross-tenant request (admin from tenant A requests subject in tenant B) | RLS rejects | 403 | None |
| Export bucket has lifecycle rule deletion | exports expire normally | URL fails after expiry | None |
| Worker process crash mid-shard | shard partial in S3 | next worker tick retries; idempotent on shard_index | None |
| Worker process killed by k8s (SIGKILL) | shard upload aborted | next worker tick retries | None |
| Lambda for URL detection fails | re-use undetected silently | SEV-2 if widespread | Operator |
| Lambda Postgres connection fails | access logged anyway via CloudWatch | reuse detection delayed | None |
| Subject sees verify.sh output FAIL on legitimate export | rare; usually due to body re-encoding | regenerate + retest | Author fixes verify.sh edge case |
| Manifest JSON parse fails on subject side | bug in verify.sh | re-issue export | Author fixes |
| zip extraction fails (corrupted zip) | rare; checksum mismatch in S3 | re-upload | Operator |
| Subject's IP geo-blocked by S3 (country restriction) | S3 returns 403 | subject contacts support | Operator opens region |
| Subject's network blocks S3 (corporate proxy) | URL access fails | subject contacts support | None |
| Sender blocking on subject email (spam filter) | SendGrid bounce | URL via DM only | Operator |
| Time-zone confusion in SLA deadline | always UTC + 30d | None | None |
| Multiple DSARs in same minute | unique constraint on (subject, request_id) | each gets unique id | None |
| Subject creates DSAR right before tenant deletion | tenant deletion blocked? or proceeds? | per tenant lifecycle FR | Operator coordinates |
| Audit log queries fail | export degraded (no dsar_history.jsonl) | SEV-3 | Operator |
| Justification reveals confidential admin info | by design (admin chose to include) | None | Operator reviews |
| Re-issued DSAR after expiry uses new request_id | each is independent | history shows both | None |
| Encryption passphrase derivation fails for non-ASCII email | normalise NFC then HKDF | None | None |
| Subject email contains "+" (Gmail tag) | salt-derived passphrase still works | None | None |

---

## §11 — Implementation notes

- S3 KMS uses tenant's CMK (per FR-CHAT-003 §1 #7). Cross-tenant decryption is cryptographically prevented.
- Signed URL TTL 7 days via `S3 presign --expires-in 604800`. We considered 30d but URLs that long are more likely to leak; 7d balances handling time against exposure.
- Notify subject via existing chat DM + SendGrid email. Dual delivery (DM + email) increases the chance subject receives it; if one channel fails, the other still works.
- Shard naming: `dsar-<request_id>-<shard_n>-of-<total>.zip`. Subject sees the count in their inbox; intuitive ordering.
- verify.sh requires `jq` + `sha256sum` (standard *nix tools). Windows users use WSL or git-bash; macOS users get them via `brew install jq` (sha256sum is part of coreutils).
- Background export worker polls `dsar_requests WHERE status='pending'` every 30s. We considered LISTEN/NOTIFY but polling is simpler and the latency is acceptable (DSAR is not real-time).
- The preview-then-full pattern was added based on operator feedback: subjects who saw the full export sometimes regretted the scope (wanted to refine). Preview is a cheap check.
- Per-tenant redact-on-export is opt-in default-off; the conservative default is "give the subject everything" per PDPL/GDPR; tenants under additional regulation opt in.
- We chose HKDF-SHA256 for passphrase derivation (not PBKDF2 or Argon2) because: (a) HKDF is purpose-built for key derivation; (b) Argon2 / PBKDF2 are designed to slow brute-force, which doesn't apply (the salt isn't secret); (c) HKDF is simpler to implement in verify.sh client-side.
- The salt is in the zip itself (manifest.json), not separately delivered. The rationale: if the URL leaks, the leaker doesn't have the subject's email; salt + URL alone don't decrypt.
- The export_chain_hash protects against tampering after delivery (e.g. if subject's machine is compromised and someone modifies the zip). Per-message anchors protect against tampering within the zip.
- We use Deflate compression (not Zstd) for broad client-side compatibility; some recipients use older zip tools.
- Sections are written as JSONL (not JSON arrays) so subjects can `head -n 100 messages.jsonl` to preview without loading the whole file.
- The verify.sh script intentionally avoids `set -e` for the per-message loop because we want it to report all FAIL messages, not stop at the first.
- The Lambda for URL-reuse detection runs in our AWS account (not tenant accounts) because: (a) shared infrastructure; (b) operator visibility across tenants; (c) reuse detection is a meta-service.
- We considered building DSAR as a query-language ("show me all messages where ...") but rejected as too much surface; full export is simpler and matches subjects' actual asks.
- The `requested_by` field in audit distinguishes self-requests from admin-requests; operators reviewing compliance logs filter on this.
- Justification is stored in `dsar_requests.justification` AND in the memory audit payload, allowing post-export review without DB access.
- The 30-day SLA gives a 25-day operator window before SEV-1 fires; this is calibrated against typical operator response times in our org.
- Why we DON'T include subject's authentication history (login times, IP addresses): those are FR-AUTH-004 owned; that FR has its own DSAR mechanism (slice 4+). Chat DSAR is scoped to chat data.
- The `chat.dsar_url_reused` event is informational; it doesn't automatically rotate the subject's other URLs (that's an operator decision).
- We considered making the URL contain a one-time token (instead of relying on S3 access log) but S3 doesn't natively support this. The log-based detection is a workable substitute.
- The subject acknowledgement is OPTIONAL (subject can choose to not click); after 14 days unacked, operator reminder fires; after 30 days, the export is considered "delivered, ack-pending" indefinitely.
- The redact-on-export policy stores the categories that were removed in `redacted_categories.json` so the subject knows their export is incomplete and can request a DPO review.
- For tenants with many subjects (1000+), DSAR throughput may bottleneck on the single worker. Slice-4+ may parallelise; for MVP, sequential is acceptable (DSARs are infrequent).
- We chose to include device registrations (without tokens) because PDPL recognises device-as-personal-data; excluding would be more conservative but less helpful to the subject.
- The README's example commands assume Unix; Windows users get a `verify.bat` (slice-4+) or are guided to WSL.
- The `salt_for_passphrase` is 32 bytes hex (64 chars) — generous for HKDF; we use the full output as the key derivation input.
- Per-shard URL = per-shard one-time-use detection; if shard 1 is re-accessed but shard 2-3 aren't, only shard 1 is revoked.

---

*End of FR-CHAT-012.*
