-- TASK-CHAT-269 §1 #17: retention on resolved reports.
--
-- Numbering: the task text specifies 0015, but 0015 is taken by TASK-CHAT-268's chat_blocks. Renumbered to
-- 0016; no other change to the shape.
--
-- The snapshot exists because we could not trust the sender not to destroy the evidence. Once the report is
-- resolved that justification expires, and what is left is a durable copy of exactly the content someone
-- asked us to remove, sitting in a table their employer can read. 90 days matches the window already
-- published at cyberskill.world/en/cyberos/delete-account - one published number, one behaviour.

ALTER TABLE chat_reports
    ADD COLUMN IF NOT EXISTS purge_after TIMESTAMPTZ NULL;

CREATE INDEX IF NOT EXISTS chat_reports_purge_idx
    ON chat_reports (purge_after) WHERE purge_after IS NOT NULL;

-- The purge itself is a JOB, not a trigger (see moderation::purge_resolved_reports). A trigger would delete
-- rows mid-transaction while an administrator is reading them. It deletes the row outright - snapshot
-- included - rather than just nulling the snapshot: a resolved report stripped of its evidence is not a
-- record of anything. The durable record is the `chat.report_resolved` audit row, which by design carries no
-- content at all.
