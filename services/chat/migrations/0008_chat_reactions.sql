-- TASK-CHAT-101: emoji reactions - a per-(message, subject, emoji) row, scoped to its tenant. A reaction is
-- deleted with its message (ON DELETE CASCADE), and a subject can react with a given emoji at most once
-- (UNIQUE), so add is idempotent and remove is exact. Mirrors the chat_messages cascade + RLS style.

CREATE TABLE IF NOT EXISTS chat_reactions (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_id   UUID NOT NULL REFERENCES chat_messages(id) ON DELETE CASCADE,
    channel_id   UUID NOT NULL REFERENCES chat_channels(id) ON DELETE CASCADE,
    tenant_id    UUID NOT NULL,
    subject_id   UUID NOT NULL,
    emoji        TEXT NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (message_id, subject_id, emoji)
);

CREATE INDEX IF NOT EXISTS chat_reactions_message_idx
    ON chat_reactions (message_id);

-- Row-level security: every row is scoped to its tenant. The GUC app.current_tenant_id is set per
-- transaction; the nil tenant bypasses for admin paths. Mirrors services/auth/migrations/0021_sessions.sql.
DO $$
DECLARE t TEXT;
BEGIN
  FOREACH t IN ARRAY ARRAY['chat_reactions'] LOOP
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
