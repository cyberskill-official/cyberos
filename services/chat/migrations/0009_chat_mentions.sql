-- get-notified cluster: @-mentions. One row per (message, mentioned subject), tenant-scoped. A mention is
-- deleted with its message (ON DELETE CASCADE). "Unread mentions" are not tracked here - they are derived the
-- same way as unread (a mention on a message created after the reader's last-read marker), so marking a
-- channel read also clears its mention badge. Mirrors the chat_reactions cascade + RLS style (0008).

CREATE TABLE IF NOT EXISTS chat_mentions (
    message_id   UUID NOT NULL REFERENCES chat_messages(id) ON DELETE CASCADE,
    channel_id   UUID NOT NULL REFERENCES chat_channels(id) ON DELETE CASCADE,
    tenant_id    UUID NOT NULL,
    subject_id   UUID NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (message_id, subject_id)
);

CREATE INDEX IF NOT EXISTS chat_mentions_subject_channel_idx
    ON chat_mentions (subject_id, channel_id);

-- Row-level security: every row is scoped to its tenant. The GUC app.current_tenant_id is set per
-- transaction; the nil tenant bypasses for admin paths. Mirrors 0008_chat_reactions.sql.
DO $$
DECLARE t TEXT;
BEGIN
  FOREACH t IN ARRAY ARRAY['chat_mentions'] LOOP
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
