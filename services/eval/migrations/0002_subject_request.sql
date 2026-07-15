-- TASK-EVAL-001 slice 2: data-subject request store (clause 10b).
--
-- The one new table slice 2 needs: a queue for the data-subject rights requests a subject can file
-- about their OWN record via POST /v1/eval/me/requests. A request is RECORDED and QUEUED for a human;
-- it is NEVER auto-applied (clause 11 - keep a human in the loop for anything consequential). The
-- runtime role may read + insert; the resolution columns (status / resolved_*) are written by the admin
-- role, exactly like access_grant.revoked_at in 0001. Same per-tenant RLS GUC (app.current_tenant_id,
-- TASK-AUTH-003) and the same append-only REVOKE idiom as 0001_governance.sql.
--
-- QUIET OPERATING MODE: the employee self-view is off by default and a subject only ever sees / files
-- about their OWN subject_id; the handler enforces self-access deny-by-default, and RLS confines every
-- row to the caller's tenant.

-- Data-subject rights requests (clause 10). request_kind matches the §1 #10 enum; status starts 'open'
-- (queued for a human). Resolution columns are written by the admin role, not the runtime app role.
CREATE TABLE IF NOT EXISTS subject_request (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL,
    subject_id      UUID NOT NULL,
    kind            TEXT NOT NULL CHECK (kind IN
                    ('access','rectification','objection')),
    note            TEXT NOT NULL DEFAULT '',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    status          TEXT NOT NULL DEFAULT 'open' CHECK (status IN
                    ('open','acknowledged','actioned','declined')),
    resolved_by     UUID,
    resolved_at     TIMESTAMPTZ,
    resolution_note TEXT
);
-- Indexed by tenant + subject: the GET /v1/eval/me read pulls a subject's own requests by this pair.
CREATE INDEX IF NOT EXISTS subject_request_subject_idx
    ON subject_request (tenant_id, subject_id);

-- 14. Row-level security: every request row is scoped to its tenant via the TASK-AUTH-003 GUC
--     app.current_tenant_id (set per transaction). The nil tenant bypasses for admin paths. Same idiom
--     as 0001_governance.sql / services/chat/migrations/0001_chat_core.sql.
DO $$
DECLARE t TEXT;
BEGIN
  FOREACH t IN ARRAY ARRAY['subject_request'] LOOP
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

-- 15. Append-only for the runtime role (clause 15). A subject's filed request is never un-filed or
--     deleted by the runtime app role; a human resolves it by writing status / resolved_* via the admin
--     role (which bypasses this REVOKE). The runtime role gets SELECT + INSERT so RLS predicates fire.
DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'cyberos_app') THEN
    GRANT SELECT, INSERT ON subject_request TO cyberos_app;  -- resolution (UPDATE) = admin role
  END IF;
END $$;
