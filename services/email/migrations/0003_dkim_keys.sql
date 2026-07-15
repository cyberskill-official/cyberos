-- TASK-EMAIL-001 §3.4 — per-tenant DKIM keystore + rotation history.
--
-- Per DEC-304 each tenant has its own DKIM signing key. Slice 1 emits
-- RSA-2048; Ed25519 (RFC 8463) deferred to slice 2. The private key is
-- stored as a KMS-encrypted blob — Stalwart syncs from this table at
-- boot and on rotation events.

BEGIN;

CREATE TABLE dkim_keys (
    id                              UUID         PRIMARY KEY,
    tenant_id                       UUID         NOT NULL,
    dkim_selector                   TEXT         NOT NULL DEFAULT 'cyberos' CHECK (dkim_selector ~ '^[a-z0-9-]{1,63}$'),
    key_algorithm                   TEXT         NOT NULL CHECK (key_algorithm IN ('rsa-2048', 'ed25519')),
    public_key_pem                  TEXT         NOT NULL CHECK (length(public_key_pem) BETWEEN 100 AND 10000),
    private_key_kms_encrypted_blob  BYTEA        NOT NULL CHECK (octet_length(private_key_kms_encrypted_blob) BETWEEN 100 AND 8192),
    kms_key_id                      TEXT         NOT NULL,
    status                          TEXT         NOT NULL CHECK (status IN ('active', 'rotated', 'revoked')) DEFAULT 'active',
    created_at                      TIMESTAMPTZ  NOT NULL DEFAULT now(),
    rotated_at                      TIMESTAMPTZ  CHECK (
        (status = 'active' AND rotated_at IS NULL)
        OR (status IN ('rotated', 'revoked') AND rotated_at IS NOT NULL)
    )
);

-- Exactly one active key per (tenant, selector).
CREATE UNIQUE INDEX uniq_active_dkim_key ON dkim_keys (tenant_id, dkim_selector) WHERE status = 'active';
CREATE INDEX dkim_keys_tenant_idx ON dkim_keys (tenant_id, created_at DESC);

ALTER TABLE dkim_keys ENABLE ROW LEVEL SECURITY;
ALTER TABLE dkim_keys FORCE ROW LEVEL SECURITY;

CREATE POLICY dkim_keys_tenant_scoped ON dkim_keys
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
        -- We allow UPDATE on dkim_keys.status (rotated/revoked transitions)
        -- but disallow updating the key material. Postgres column-level
        -- UPDATE grants enforce this. The boot-time validator (`cyberos
        -- doctor`) asserts the grant is exactly the expected column list.
        EXECUTE 'REVOKE UPDATE, DELETE ON dkim_keys FROM cyberos_app';
        EXECUTE 'GRANT UPDATE (status, rotated_at) ON dkim_keys TO cyberos_app';
    END IF;
END $$;

COMMIT;
