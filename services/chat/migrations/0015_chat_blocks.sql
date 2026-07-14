-- TASK-CHAT-268: user blocking. One row per (blocker, blocked) pair.
--
-- Numbering: the FR text specifies 0014, but 0014 is taken by TASK-CHAT-267's chat_reports. Renumbered to
-- 0015; no other change to the shape.
--
-- Directional: A blocking B says nothing about whether B blocks A. Private: only the blocker ever reads
-- their own rows, and B is never told (§1 #2, #8 — see the FR's §2 for why telling B is the dangerous
-- design, not the honest one).

CREATE TABLE IF NOT EXISTS chat_blocks (
    tenant_id           UUID NOT NULL,
    blocker_subject_id  UUID NOT NULL,
    blocked_subject_id  UUID NOT NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    PRIMARY KEY (blocker_subject_id, blocked_subject_id),
    CONSTRAINT chat_blocks_not_self CHECK (blocker_subject_id <> blocked_subject_id)
);

-- The PK's leading column already answers the hot question ("every subject THIS caller has blocked"), asked
-- once per message-list, realtime fan-out and DM list. This index answers the REVERSE question, which only
-- the notification fan-out asks: "of the members I am about to notify, which have blocked this sender?"
CREATE INDEX IF NOT EXISTS chat_blocks_blocked_idx
    ON chat_blocks (blocked_subject_id, blocker_subject_id);

-- RLS mirrors 0009/0010/0012/0014: tenant-scoped with the nil-tenant admin bypass, USING and WITH CHECK
-- both, FORCEd so even the table owner is subject to it (§1 #13).
DO $$
DECLARE t TEXT;
BEGIN
  FOREACH t IN ARRAY ARRAY['chat_blocks'] LOOP
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
