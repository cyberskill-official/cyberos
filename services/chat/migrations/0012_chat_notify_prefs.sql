-- Notification preferences (remaining-gaps wave): a per-(channel, member) notify mode.
--   all       - the default (no row): every message notifies.
--   mentions  - only @-mentions reach the member's notify socket / badges.
--   none      - muted: nothing reaches the notify socket; the client also quiets the badge.
-- Rows exist only for non-default modes; setting back to 'all' deletes the row.

CREATE TABLE IF NOT EXISTS chat_channel_prefs (
    channel_id  UUID NOT NULL REFERENCES chat_channels(id) ON DELETE CASCADE,
    subject_id  UUID NOT NULL,
    tenant_id   UUID NOT NULL,
    notify      TEXT NOT NULL DEFAULT 'all' CHECK (notify IN ('all', 'mentions', 'none')),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (channel_id, subject_id)
);
CREATE INDEX IF NOT EXISTS chat_channel_prefs_subject_idx ON chat_channel_prefs (subject_id);

-- Row-level security mirrors 0009/0010: tenant-scoped with the nil-tenant admin bypass.
DO $$
DECLARE t TEXT;
BEGIN
  FOREACH t IN ARRAY ARRAY['chat_channel_prefs'] LOOP
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
