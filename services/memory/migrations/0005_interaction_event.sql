-- FR-MEMORY-121 — interaction-event indexing on l1_audit_log.
--
-- NO NEW TABLE (DEC-2703): interaction-events are aux rows on the existing hash-chained l1_audit_log,
-- written through cyberos-audit-chain exactly like auth.token_issued and the obs rows. A second store
-- would fork the system of record, double the RLS surface, and break memory's single-reconcile invariant.
-- This migration adds ONLY generated columns + partial indexes so the BRAIN ingestion (FR-MEMORY-123)
-- and the console viewer (FR-APP-005) can scan interaction-events by subject / module / event_type
-- without parsing JSON per row.
--
-- The audit-row `event_type` column (FR-OBS-008, migration 0004) already equals 'memory.interaction_event'
-- for these rows. The interaction's OWN module / verb / class live inside the JSON payload
-- (body.payload.*); these generated columns reach INTO the payload and surface them as typed, indexable
-- columns. They follow migration 0004's error-swallowing IMMUTABLE extractor pattern, because non-JSON
-- markdown `put` bodies must not error a `body::jsonb` cast mid-query.

-- Returns a string field from `body.payload`, or NULL when `body` is null / not JSON / lacks the field.
-- IMMUTABLE + PARALLEL SAFE so it can back a STORED generated column, mirroring cyberos_audit_event_type.
CREATE OR REPLACE FUNCTION cyberos_iev_payload_field(body TEXT, field TEXT)
RETURNS TEXT
LANGUAGE plpgsql
IMMUTABLE
PARALLEL SAFE
AS $$
BEGIN
    RETURN (body::jsonb -> 'payload' ->> field);
EXCEPTION WHEN others THEN
    RETURN NULL;
END;
$$;

-- The interaction's own module (body.payload.module) — distinct from the row-level event_type.
-- ADD COLUMN IF NOT EXISTS keeps the migration idempotent (re-apply safe; the integration tests apply it
-- against a possibly-already-migrated dev DB, mirroring eval's idempotent governance migration).
ALTER TABLE l1_audit_log
    ADD COLUMN IF NOT EXISTS iev_module TEXT
    GENERATED ALWAYS AS (cyberos_iev_payload_field(body, 'module')) STORED;

-- The interaction's own verb (body.payload.event_type, e.g. 'chat.message_created').
ALTER TABLE l1_audit_log
    ADD COLUMN IF NOT EXISTS iev_event_type TEXT
    GENERATED ALWAYS AS (cyberos_iev_payload_field(body, 'event_type')) STORED;

-- The interaction's coarse class (body.payload.event_class).
ALTER TABLE l1_audit_log
    ADD COLUMN IF NOT EXISTS iev_event_class TEXT
    GENERATED ALWAYS AS (cyberos_iev_payload_field(body, 'event_class')) STORED;

-- The interaction's own event_id (body.payload.event_id) — backs the replay dedup guard (§1 #17).
ALTER TABLE l1_audit_log
    ADD COLUMN IF NOT EXISTS iev_event_id TEXT
    GENERATED ALWAYS AS (cyberos_iev_payload_field(body, 'event_id')) STORED;

-- Partial indexes scoped to interaction-event rows ONLY (kept small; other audit kinds are excluded by
-- the WHERE on the row-level event_type column from migration 0004).
CREATE INDEX IF NOT EXISTS l1_iev_subject_idx
    ON l1_audit_log (tenant_id, subject_id, ts_ns DESC)
    WHERE event_type = 'memory.interaction_event';

CREATE INDEX IF NOT EXISTS l1_iev_module_class_idx
    ON l1_audit_log (tenant_id, iev_module, iev_event_class, ts_ns DESC)
    WHERE event_type = 'memory.interaction_event';

-- Dedup guard for replay / CDC re-delivery (§1 #17): the interaction's event_id is unique per tenant
-- among interaction-event rows. A retried emit reusing the same event_id collides here rather than
-- double-counting. Scoped to the row kind so it never constrains other audit rows.
CREATE UNIQUE INDEX IF NOT EXISTS l1_iev_event_id_uq
    ON l1_audit_log (tenant_id, iev_event_id)
    WHERE event_type = 'memory.interaction_event';
