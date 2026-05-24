-- FR-EMAIL-004 — DKIM/ARC/BIMI delivery-auth persistence.
--
-- This migration turns the slice from detached helpers into an auditable
-- service surface: DNS setup/verification state is durable, and every
-- signing/ARC/BIMI decision is an append-only event row.

BEGIN;

CREATE TABLE tenant_dns_setup (
    id                  UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id           UUID         NOT NULL,
    domain              TEXT         NOT NULL CHECK (domain ~ '^[a-z0-9.-]{3,253}$'),
    selector            TEXT         NOT NULL CHECK (selector ~ '^[a-z0-9-]{1,63}$'),
    dkim_txt_name       TEXT         NOT NULL,
    dkim_txt_value      TEXT         NOT NULL,
    spf_txt_value       TEXT         NOT NULL,
    dmarc_txt_name      TEXT         NOT NULL,
    dmarc_txt_value     TEXT         NOT NULL,
    bimi_txt_name       TEXT         NOT NULL,
    bimi_txt_value      TEXT         NOT NULL,
    status              TEXT         NOT NULL CHECK (status IN ('pending', 'verified', 'failed')) DEFAULT 'pending',
    last_checked_at     TIMESTAMPTZ,
    created_at          TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ  NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, domain, selector)
);

CREATE TABLE delivery_auth_events (
    id                  UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id           UUID         NOT NULL,
    message_id          UUID,
    event_kind          TEXT         NOT NULL CHECK (event_kind IN (
        'email.dkim_signed',
        'email.arc_chain_extended',
        'email.bimi_indicator_attached',
        'email.dns_verification_passed',
        'email.dns_verification_failed'
    )),
    domain              TEXT,
    selector            TEXT,
    outcome             TEXT CHECK (outcome IS NULL OR outcome IN (
        'signed_ed25519',
        'signed_rsa',
        'sign_failed_no_key',
        'sign_failed_kms'
    )),
    payload             JSONB        NOT NULL DEFAULT '{}'::jsonb,
    trace_id            TEXT,
    created_at          TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE INDEX tenant_dns_setup_tenant_idx ON tenant_dns_setup (tenant_id, domain);
CREATE INDEX delivery_auth_events_tenant_idx ON delivery_auth_events (tenant_id, created_at DESC);
CREATE INDEX delivery_auth_events_message_idx ON delivery_auth_events (message_id) WHERE message_id IS NOT NULL;

ALTER TABLE tenant_dns_setup ENABLE ROW LEVEL SECURITY;
ALTER TABLE tenant_dns_setup FORCE ROW LEVEL SECURITY;
ALTER TABLE delivery_auth_events ENABLE ROW LEVEL SECURITY;
ALTER TABLE delivery_auth_events FORCE ROW LEVEL SECURITY;

CREATE POLICY tenant_dns_setup_tenant_scoped ON tenant_dns_setup
    FOR ALL
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    );

CREATE POLICY delivery_auth_events_tenant_scoped ON delivery_auth_events
    FOR ALL
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    );

CREATE OR REPLACE FUNCTION email_touch_updated_at() RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END $$ LANGUAGE plpgsql;

CREATE TRIGGER tenant_dns_setup_updated_at_trg
    BEFORE UPDATE ON tenant_dns_setup
    FOR EACH ROW
    WHEN (OLD.* IS DISTINCT FROM NEW.*)
    EXECUTE FUNCTION email_touch_updated_at();

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'cyberos_app') THEN
        EXECUTE 'REVOKE UPDATE, DELETE ON delivery_auth_events FROM cyberos_app';
    END IF;
END $$;

COMMIT;
