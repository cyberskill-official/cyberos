-- Richer-messages cluster: multi-file messages + pluggable attachment byte storage.
--
-- 1. chat_attachments learns WHERE its payload lives: storage='db' keeps the original bytea behavior;
--    storage='fs' stores bytes on the chat container's attachment volume under storage_key
--    (<tenant>/<attachment-id>), so `data` becomes nullable. Existing rows stay 'db' via the default.
-- 2. chat_message_attachments links a message to N attachments (ordered). The legacy single
--    chat_messages.attachment_id column stays populated (first attachment) so already-open clients on the
--    cached PWA shell keep rendering.

ALTER TABLE chat_attachments ADD COLUMN IF NOT EXISTS storage TEXT NOT NULL DEFAULT 'db';
ALTER TABLE chat_attachments ADD COLUMN IF NOT EXISTS storage_key TEXT;
ALTER TABLE chat_attachments ALTER COLUMN data DROP NOT NULL;

CREATE TABLE IF NOT EXISTS chat_message_attachments (
    message_id     UUID NOT NULL REFERENCES chat_messages(id) ON DELETE CASCADE,
    attachment_id  UUID NOT NULL REFERENCES chat_attachments(id) ON DELETE CASCADE,
    tenant_id      UUID NOT NULL,
    ord            SMALLINT NOT NULL DEFAULT 0,
    PRIMARY KEY (message_id, attachment_id)
);
CREATE INDEX IF NOT EXISTS chat_message_attachments_att_idx
    ON chat_message_attachments (attachment_id);

-- Row-level security: every row is scoped to its tenant. The GUC app.current_tenant_id is set per
-- transaction; the nil tenant bypasses for admin paths. Mirrors 0009_chat_mentions.sql.
DO $$
DECLARE t TEXT;
BEGIN
  FOREACH t IN ARRAY ARRAY['chat_message_attachments'] LOOP
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
