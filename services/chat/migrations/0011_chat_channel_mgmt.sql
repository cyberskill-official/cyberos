-- Find-and-organize cluster: channel management metadata.
--
-- topic       - a short purpose line shown in the header (groups only; cosmetic).
-- visibility  - 'private' (member-only, the historical behavior and the default for every existing row)
--               or 'public' (listed in the tenant's channel browser, self-joinable). DMs stay private.
-- archived_at - set = read-only channel (posts rejected, hidden from browse); NULL = live.

ALTER TABLE chat_channels ADD COLUMN IF NOT EXISTS topic TEXT NOT NULL DEFAULT '';
ALTER TABLE chat_channels ADD COLUMN IF NOT EXISTS visibility TEXT NOT NULL DEFAULT 'private';
ALTER TABLE chat_channels ADD COLUMN IF NOT EXISTS archived_at TIMESTAMPTZ;

DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'chat_channels_visibility_check') THEN
    ALTER TABLE chat_channels
      ADD CONSTRAINT chat_channels_visibility_check CHECK (visibility IN ('private', 'public'));
  END IF;
END $$;
