-- Partial indexes for the hot read paths (deep audit 2026-07-03). Each matches the exact predicate the query
-- uses so the planner can index-scan instead of filtering, and each is partial so it stays small.

-- Top-level channel page (list): non-deleted, non-reply messages newest-first per channel.
CREATE INDEX IF NOT EXISTS chat_messages_toplevel_idx
    ON chat_messages (channel_id, created_at DESC)
    WHERE deleted_at IS NULL AND parent_id IS NULL;

-- Unread range + around/before/after windows: non-deleted messages per channel by time.
CREATE INDEX IF NOT EXISTS chat_messages_unread_range_idx
    ON chat_messages (channel_id, created_at)
    WHERE deleted_at IS NULL;

-- Thread replies + the reply-count fold: non-deleted replies grouped by their parent.
CREATE INDEX IF NOT EXISTS chat_messages_parent_idx
    ON chat_messages (parent_id)
    WHERE deleted_at IS NULL AND parent_id IS NOT NULL;

-- Channel browser: public, non-archived group channels per tenant, ordered by name.
CREATE INDEX IF NOT EXISTS chat_channels_browse_idx
    ON chat_channels (tenant_id, lower(name))
    WHERE kind = 'group' AND visibility = 'public' AND archived_at IS NULL;
