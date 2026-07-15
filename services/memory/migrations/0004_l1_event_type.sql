-- TASK-OBS-008 - a generated `event_type` column on l1_audit_log so the compliance views can query audit
-- rows by kind. The audit row's kind is the `event_type` field of its JSON body; querying it directly
-- with `body::jsonb` is unsafe because memory-file `put` rows carry non-JSON markdown bodies, so the
-- cast would error mid-query. A generated column backed by an error-swallowing immutable extractor makes
-- the kind a real, indexable column that is NULL for non-JSON bodies.

-- Returns the JSON `event_type` of `body`, or NULL when `body` is null or not a JSON object. Marked
-- IMMUTABLE (the mapping is deterministic) so it can back a generated column.
CREATE OR REPLACE FUNCTION cyberos_audit_event_type(body TEXT)
RETURNS TEXT
LANGUAGE plpgsql
IMMUTABLE
PARALLEL SAFE
AS $$
BEGIN
    RETURN (body::jsonb ->> 'event_type');
EXCEPTION WHEN others THEN
    RETURN NULL;
END;
$$;

ALTER TABLE l1_audit_log
    ADD COLUMN event_type TEXT
    GENERATED ALWAYS AS (cyberos_audit_event_type(body)) STORED;

-- The compliance-view query is tenant + kind + time window.
CREATE INDEX l1_audit_log_tenant_event_type_idx
    ON l1_audit_log (tenant_id, event_type, ts_ns);
