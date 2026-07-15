-- TASK-AUTH-004 — RS256 signing keys + JWKS publication.
--
-- One active key per tenant (or per cluster for the shared issuer). Older
-- keys remain published in JWKS until expiry so in-flight JWTs verify.
-- Per AUTHORING_DISCIPLINE §3.2 rule 6: audit-row kind `auth.key_*` covers
-- create / rotate / retire — emitted by the rotation cron, not this schema.

CREATE TABLE auth_signing_keys (
    kid             TEXT PRIMARY KEY,                 -- 'auth-2026-05-01' style
    algorithm       TEXT NOT NULL DEFAULT 'RS256',
    public_pem      TEXT NOT NULL,                    -- PEM-encoded SPKI
    private_pem     TEXT NOT NULL,                    -- PEM-encoded PKCS8; KMS-wrapped in prod
    status          TEXT NOT NULL DEFAULT 'active',   -- 'active' | 'retired'
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    activated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    retired_at      TIMESTAMPTZ,
    expires_at      TIMESTAMPTZ NOT NULL,             -- key drops out of JWKS at this point

    CONSTRAINT signing_keys_status_enum CHECK (status IN ('active', 'retired'))
);

-- Index for the JWKS endpoint query (active + non-expired).
-- Note: retired-key visibility window is enforced at query time, not here,
-- because Postgres requires index predicates to use IMMUTABLE functions only.
CREATE INDEX auth_signing_keys_published_idx
    ON auth_signing_keys (status, expires_at)
    WHERE status = 'active';

-- The signing keys table is NOT tenant-scoped — it's cluster-wide. Skip RLS.
-- Access is restricted via the `cyberos_app` role grant.
GRANT SELECT ON auth_signing_keys TO cyberos_app;
GRANT INSERT, UPDATE ON auth_signing_keys TO cyberos_app;
GRANT SELECT ON auth_signing_keys TO cyberos_ro;
