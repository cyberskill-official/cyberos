-- FR-CHAT-101 slice 2: thread replies (parent_id), edits (edited_at), and soft-deletes (deleted_at).

ALTER TABLE chat_messages ADD COLUMN IF NOT EXISTS parent_id UUID NULL
    REFERENCES chat_messages(id) ON DELETE CASCADE;
ALTER TABLE chat_messages ADD COLUMN IF NOT EXISTS edited_at  TIMESTAMPTZ NULL;
ALTER TABLE chat_messages ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ NULL;

CREATE INDEX IF NOT EXISTS chat_messages_parent_idx
    ON chat_messages (parent_id) WHERE parent_id IS NOT NULL;
