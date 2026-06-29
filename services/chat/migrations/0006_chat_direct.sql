-- P0 DM support. A channel is either a 'group' (the existing named, multi-member
-- channel) or a 'direct' (a two-person DM, rendered by the partner's name rather
-- than a channel name). Existing rows default to 'group', so nothing changes for
-- channels created before this migration.
ALTER TABLE chat_channels
  ADD COLUMN IF NOT EXISTS kind text NOT NULL DEFAULT 'group';
