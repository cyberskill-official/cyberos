-- 0007: schema repair for the live P0 Supabase DB.
--
-- The deploy-step migration (deploy/vps/migrate.sh) BASELINED the chat migrations 0001-0006 (recorded them
-- as applied WITHOUT running them, because chat was hand-migrated before that step existed). A Supabase DB
-- created from an earlier, partial hand-migration can therefore be missing columns that the message-list
-- query selects (parent_id, edited_at, deleted_at, attachment_id) or the channel `kind` - which surfaces as a
-- 500 on GET /v1/chat/channels/{id}/messages even though channels load fine.
--
-- This re-asserts those columns idempotently. Every statement is `IF NOT EXISTS`, so it adds only what is
-- missing and is safe to run on any DB. migrate.sh applies this as a NEW (non-baselined) file, so the next
-- deploy repairs the live schema; it is also safe to paste into the Supabase SQL editor directly.

ALTER TABLE chat_messages ADD COLUMN IF NOT EXISTS parent_id     uuid;
ALTER TABLE chat_messages ADD COLUMN IF NOT EXISTS edited_at     timestamptz;
ALTER TABLE chat_messages ADD COLUMN IF NOT EXISTS deleted_at    timestamptz;
ALTER TABLE chat_messages ADD COLUMN IF NOT EXISTS attachment_id uuid;
ALTER TABLE chat_channels ADD COLUMN IF NOT EXISTS kind text NOT NULL DEFAULT 'group';

-- chat_attachments (from migration 0003) was also baselined-not-run on the live DB, so uploads fail with
-- "relation chat_attachments does not exist". Re-create it idempotently: table + index + tenant RLS.
CREATE TABLE IF NOT EXISTS chat_attachments (
    id                   uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id            uuid NOT NULL,
    channel_id           uuid NOT NULL REFERENCES chat_channels(id) ON DELETE CASCADE,
    uploader_subject_id  uuid NOT NULL,
    filename             text NOT NULL,
    content_type         text NOT NULL,
    size_bytes           bigint NOT NULL,
    data                 bytea NOT NULL,
    created_at           timestamptz NOT NULL DEFAULT now()
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
