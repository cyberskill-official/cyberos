-- TASK-CHAT-101 slice 4: read receipts (chat_read_markers) and device registration (chat_devices) for push.

CREATE TABLE IF NOT EXISTS chat_read_markers (
    channel_id            UUID NOT NULL REFERENCES chat_channels(id) ON DELETE CASCADE,
    tenant_id             UUID NOT NULL,
    subject_id            UUID NOT NULL,
    last_read_message_id  UUID NULL,
    last_read_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (channel_id, subject_id)
);

CREATE TABLE IF NOT EXISTS chat_devices (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id   UUID NOT NULL,
    subject_id  UUID NOT NULL,
    platform    TEXT NOT NULL,
    token       TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (subject_id, token)
);
CREATE INDEX IF NOT EXISTS chat_devices_subject_idx ON chat_devices (subject_id);

DO $$
DECLARE t TEXT;
BEGIN
  FOREACH t IN ARRAY ARRAY['chat_read_markers','chat_devices'] LOOP
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
