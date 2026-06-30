-- Editable profile: a small avatar image stored as a base64 data URL on the subject. Nullable; the client
-- downscales before upload and the API size-caps it. Idempotent so it is safe to apply on any existing DB.
ALTER TABLE subjects ADD COLUMN IF NOT EXISTS avatar TEXT;
