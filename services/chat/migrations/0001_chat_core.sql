-- FR-CHAT-101 slice 1: native chat core - channels, members, messages, with per-tenant RLS.
-- Requires pgcrypto (gen_random_uuid). The deploy/dev bootstrap enables it per database.

CREATE TABLE IF NOT EXISTS chat_channels (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id    UUID NOT NULL,
    name         TEXT NOT NULL,
    created_by   UUID NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS chat_channel_members (
    channel_id   UUID NOT NULL REFERENCES chat_channels(id) ON DELETE CASCADE,
    tenant_id    UUID NOT NULL,
    subject_id   UUID NOT NULL,
    role         TEXT NOT NULL DEFAULT 'member',
    joined_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (channel_id, subject_id)
);

CREATE TABLE IF NOT EXISTS chat_messages (
    id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id          UUID NOT NULL,
    channel_id         UUID NOT NULL REFERENCES chat_channels(id) ON DELETE CASCADE,
    sender_subject_id  UUID NOT NULL,
    body               TEXT NOT NULL,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS chat_messages_channel_created_idx
    ON chat_messages (channel_id, created_at DESC);

CREATE INDEX IF NOT EXISTS chat_channel_members_subject_idx
    ON chat_channel_members (subject_id);

-- Row-level security: every row is scoped to its tenant. The GUC app.current_tenant_id is set per
-- transaction; the nil tenant bypasses for admin paths. Mirrors services/auth/migrations/0021_sessions.sql.
DO $$
DECLARE t TEXT;
BEGIN
  FOREACH t IN ARRAY ARRAY['chat_channels','chat_channel_members','chat_messages'] LOOP
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
