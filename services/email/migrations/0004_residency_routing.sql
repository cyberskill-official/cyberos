-- TASK-EMAIL-001 §3.5 — per-tenant residency → (S3 bucket, KMS key,
-- Postgres schema namespace) routing table.
--
-- The Stalwart inbound handler looks up the recipient's tenant residency
-- before writing the body to S3. Cross-residency writes are fail-closed
-- per §1 #12 (the handler asserts residency match BEFORE the S3 PUT).
--
-- This is a lightweight slice-1 table; TASK-AI-016 ships the broader
-- residency-policy framework which this resolver delegates to in slice 2.

BEGIN;

CREATE TABLE IF NOT EXISTS tenant_residency (
    tenant_id    UUID         PRIMARY KEY,
    residency    TEXT         NOT NULL CHECK (residency IN ('sg-1', 'vn-1', 'eu-1', 'us-1')),
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS tenant_residency_residency_idx ON tenant_residency (residency);

-- RLS — tenant can only read its own residency row; root can read any.
ALTER TABLE tenant_residency ENABLE ROW LEVEL SECURITY;
ALTER TABLE tenant_residency FORCE ROW LEVEL SECURITY;

CREATE POLICY tenant_residency_scoped ON tenant_residency
    FOR ALL
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    );

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'cyberos_app') THEN
        -- The application role can INSERT (provisioning) + SELECT (lookup)
        -- but cannot UPDATE residency once set (residency changes are an
        -- operator-action via the BYPASSRLS `cyberos_ops` role per
        -- TASK-AUTH-003 §1 #5).
        EXECUTE 'REVOKE UPDATE, DELETE ON tenant_residency FROM cyberos_app';
    END IF;
END $$;

COMMIT;
