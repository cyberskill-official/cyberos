-- FR-CHAT-101: link a message to an uploaded attachment (replaces the client-side body marker).
-- Nullable; on attachment delete the message survives with a null link.
ALTER TABLE chat_messages
  ADD COLUMN IF NOT EXISTS attachment_id uuid REFERENCES chat_attachments(id) ON DELETE SET NULL;
