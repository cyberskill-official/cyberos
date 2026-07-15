-- TASK-CHAT-267: in-app content reporting. One row per report.
--
-- Numbering: the task text specifies 0013, but 0013_chat_hot_indexes.sql already exists. Renumbered to 0014;
-- no other change to the shape.
--
-- The snapshot_* columns are written once at INSERT and never updated. The reported message can be edited or
-- soft-deleted by its own sender afterwards (chat_messages.edited_at / .deleted_at, both sender-reachable),
-- and a moderation queue that renders "(deleted)" for every row it receives is not a moderation queue. The
-- snapshot IS the evidence (§1 #4).

CREATE TABLE IF NOT EXISTS chat_reports (
    id                     UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id              UUID NOT NULL,
    reporter_subject_id    UUID NOT NULL,

    target_kind            TEXT NOT NULL,
    target_message_id      UUID NULL REFERENCES chat_messages(id)    ON DELETE SET NULL,
    target_attachment_id   UUID NULL REFERENCES chat_attachments(id) ON DELETE SET NULL,
    target_subject_id      UUID NULL,
    channel_id             UUID NULL REFERENCES chat_channels(id)    ON DELETE SET NULL,

    reason                 TEXT NOT NULL,
    detail                 TEXT NULL,

    -- Evidence. Written at INSERT, never updated. See §1 #4.
    snapshot_body          TEXT NULL,
    snapshot_filename      TEXT NULL,
    snapshot_content_type  TEXT NULL,
    snapshot_size_bytes    BIGINT NULL,
    snapshot_sender_id     UUID NULL,
    snapshot_taken_at      TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- Written only by TASK-CHAT-269 (the moderation queue). This task creates the columns and never
    -- transitions them.
    status                 TEXT NOT NULL DEFAULT 'open',
    resolution             TEXT NULL,
    resolved_at            TIMESTAMPTZ NULL,
    resolved_by_subject_id UUID NULL,

    created_at             TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chat_reports_target_kind_enum
        CHECK (target_kind IN ('message', 'attachment', 'subject')),
    CONSTRAINT chat_reports_reason_enum
        CHECK (reason IN ('spam','harassment','hate','sexual','violence','self_harm','illegal','other')),
    CONSTRAINT chat_reports_status_enum
        CHECK (status IN ('open', 'actioned', 'dismissed')),
    CONSTRAINT chat_reports_detail_len
        CHECK (detail IS NULL OR char_length(detail) <= 1000),
    -- Exactly one target column is populated, and it matches target_kind.
    CONSTRAINT chat_reports_target_shape CHECK (
        (target_kind = 'message'    AND target_message_id    IS NOT NULL
                                    AND target_attachment_id IS NULL AND target_subject_id IS NULL) OR
        (target_kind = 'attachment' AND target_attachment_id IS NOT NULL
                                    AND target_message_id    IS NULL AND target_subject_id IS NULL) OR
        (target_kind = 'subject'    AND target_subject_id    IS NOT NULL
                                    AND target_message_id    IS NULL AND target_attachment_id IS NULL)
    ),
    -- A reporter cannot report themselves. Cheap guard against a confused client (§4 AC 6).
    CONSTRAINT chat_reports_not_self
        CHECK (target_subject_id IS NULL OR target_subject_id <> reporter_subject_id)
);

-- §1 #6: at most one OPEN report per (tenant, reporter, target). Postgres treats NULL as distinct from NULL
-- in a unique index, so an index over three nullable target columns would never fire; COALESCE the unused
-- arms to the nil UUID (never a real id - the same convention the auth module uses for the root tenant) so
-- the triple is comparable. Resolved reports do not block a new one: the same person can misbehave twice.
CREATE UNIQUE INDEX IF NOT EXISTS chat_reports_open_uniq
    ON chat_reports (tenant_id, reporter_subject_id, target_kind,
                     COALESCE(target_message_id,    '00000000-0000-0000-0000-000000000000'::uuid),
                     COALESCE(target_attachment_id, '00000000-0000-0000-0000-000000000000'::uuid),
                     COALESCE(target_subject_id,    '00000000-0000-0000-0000-000000000000'::uuid))
    WHERE status = 'open';

-- The moderation queue's read path (TASK-CHAT-269).
CREATE INDEX IF NOT EXISTS chat_reports_queue_idx
    ON chat_reports (tenant_id, status, created_at DESC);
-- The rate-limit count (§1 #7) - one index scan over at most 20 rows.
CREATE INDEX IF NOT EXISTS chat_reports_rate_idx
    ON chat_reports (reporter_subject_id, created_at DESC);

-- Row-level security mirrors 0009/0010/0012: tenant-scoped with the nil-tenant admin bypass, USING and
-- WITH CHECK both, FORCEd so even the table owner is subject to it (§1 #9).
DO $$
DECLARE t TEXT;
BEGIN
  FOREACH t IN ARRAY ARRAY['chat_reports'] LOOP
    EXECUTE format('ALTER TABLE %I ENABLE ROW LEVEL SECURITY', t);
    EXECUTE format('ALTER TABLE %I FORCE ROW LEVEL SECURITY', t);
    EXECUTE format('DROP POLICY IF EXISTS %I_tenant_isolation ON %I', t, t);
    EXECUTE format(
      'CREATE POLICY %I_tenant_isolation ON %I USING (
         tenant_id::text = current_setting(''app.current_tenant_id'', true)
         OR current_setting(''app.current_tenant_id'', true) = ''00000000-0000-0000-0000-000000000000''
       ) WITH CHECK (
         tenant_id::text = current_setting(''app.current_tenant_id'', true)
         OR current_setting(''app.current_tenant_id'', true) = ''00000000-0000-0000-0000-000000000000''
       )', t, t);
  END LOOP;
END $$;
