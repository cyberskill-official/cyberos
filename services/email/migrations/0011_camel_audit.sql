-- FR-EMAIL-005 — CaMeL variable store + immutable audit.

BEGIN;

CREATE TABLE camel_variables (
    id                  UUID         PRIMARY KEY,
    tenant_id           UUID         NOT NULL,
    source_email_id     UUID         NOT NULL,
    schema_name         TEXT         NOT NULL CHECK (length(schema_name) BETWEEN 1 AND 200),
    value_hash16        CHAR(16)     NOT NULL CHECK (value_hash16 ~ '^[0-9a-f]{16}$'),
    expires_at          TIMESTAMPTZ  NOT NULL,
    created_at          TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE TABLE camel_trust_list (
    id                  UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id           UUID         NOT NULL,
    domain              TEXT         NOT NULL CHECK (domain ~ '^[a-z0-9.-]{3,253}$'),
    op_kind             TEXT         NOT NULL CHECK (op_kind IN ('read', 'summarize', 'classify', 'write', 'execute')),
    full_bypass         BOOLEAN      NOT NULL DEFAULT false,
    ciso_audit_row_id   UUID,
    created_by          UUID         NOT NULL,
    created_at          TIMESTAMPTZ  NOT NULL DEFAULT now(),
    CHECK (full_bypass = false OR ciso_audit_row_id IS NOT NULL),
    UNIQUE (tenant_id, domain, op_kind)
);

CREATE TABLE camel_audit (
    id                  UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id           UUID         NOT NULL,
    plan_id             UUID         NOT NULL,
    event_kind          TEXT         NOT NULL CHECK (event_kind IN (
        'email.camel_plan_built',
        'email.camel_quarantined_extracted',
        'email.camel_executed',
        'email.camel_blocked',
        'email.camel_failed'
    )),
    outcome             TEXT         NOT NULL CHECK (outcome IN ('safe', 'suspicious_marked', 'hard_blocked', 'error')),
    variables           UUID[]       NOT NULL DEFAULT '{}',
    payload             JSONB        NOT NULL DEFAULT '{}'::jsonb,
    trace_id            TEXT,
    created_at          TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE INDEX camel_variables_tenant_source_idx ON camel_variables (tenant_id, source_email_id, created_at DESC);
CREATE INDEX camel_variables_expiry_idx ON camel_variables (expires_at);
CREATE INDEX camel_audit_tenant_idx ON camel_audit (tenant_id, created_at DESC);

ALTER TABLE camel_variables ENABLE ROW LEVEL SECURITY;
ALTER TABLE camel_variables FORCE ROW LEVEL SECURITY;
ALTER TABLE camel_trust_list ENABLE ROW LEVEL SECURITY;
ALTER TABLE camel_trust_list FORCE ROW LEVEL SECURITY;
ALTER TABLE camel_audit ENABLE ROW LEVEL SECURITY;
ALTER TABLE camel_audit FORCE ROW LEVEL SECURITY;

CREATE POLICY camel_variables_tenant_scoped ON camel_variables
    FOR ALL
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    );

CREATE POLICY camel_trust_list_tenant_scoped ON camel_trust_list
    FOR ALL
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    );

CREATE POLICY camel_audit_tenant_scoped ON camel_audit
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
        EXECUTE 'REVOKE UPDATE, DELETE ON camel_audit FROM cyberos_app';
    END IF;
END $$;

COMMIT;
