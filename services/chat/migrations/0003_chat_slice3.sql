-- TASK-CHAT-101 slice 3: Vietnamese-friendly message search + DB-backed file attachments.

-- Accent- and case-insensitive search. chat_norm(t) = lower(unaccent(t)); the 2-arg unaccent('unaccent', t)
-- form is IMMUTABLE so it can index. A GIN trigram index makes substring search fast.
CREATE EXTENSION IF NOT EXISTS unaccent;
CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE OR REPLACE FUNCTION chat_norm(t text) RETURNS text
    AS $$ SELECT lower(unaccent('unaccent', t)) $$ LANGUAGE sql IMMUTABLE;

CREATE INDEX IF NOT EXISTS chat_messages_search_idx
    ON chat_messages USING gin (chat_norm(body) gin_trgm_ops);

-- Attachments. Slice 3 stores bytes in Postgres (size-capped in the service); object storage is a later slice.
CREATE TABLE IF NOT EXISTS chat_attachments (
    id                   UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id            UUID NOT NULL,
    channel_id           UUID NOT NULL REFERENCES chat_channels(id) ON DELETE CASCADE,
    uploader_subject_id  UUID NOT NULL,
    filename             TEXT NOT NULL,
    content_type         TEXT NOT NULL,
    size_bytes           BIGINT NOT NULL,
    data                 BYTEA NOT NULL,
    created_at           TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS chat_attachments_channel_idx ON chat_attachments (channel_id);

DO $$
BEGIN
  EXECUTE 'ALTER TABLE chat_attachments ENABLE ROW LEVEL SECURITY';
  EXECUTE 'ALTER TABLE chat_attachments FORCE ROW LEVEL SECURITY';
  EXECUTE 'DROP POLICY IF EXISTS chat_attachments_tenant_isolation ON chat_attachments';
  EXECUTE 'CREATE POLICY chat_attachments_tenant_isolation ON chat_attachments USING (
             tenant_id::text = current_setting(''app.current_tenant_id'', true)
             OR current_setting(''app.current_tenant_id'', true) = ''00000000-0000-0000-0000-000000000000''
           ) WITH CHECK (
             tenant_id::text = current_setting(''app.current_tenant_id'', true)
             OR current_setting(''app.current_tenant_id'', true) = ''00000000-0000-0000-0000-000000000000''
           )';
END $$;
