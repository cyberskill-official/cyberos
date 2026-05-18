-- FR-AUTH-001 — Idempotency for /v1/admin/* endpoints.
--
-- Idempotency-Key is required on every admin POST. We persist the first
-- response keyed by (route, tenant_id, idempotency_key) for 24h so
-- retried calls return the original response body bit-for-bit.

CREATE TABLE admin_idempotency (
    idempotency_key TEXT NOT NULL,
    route           TEXT NOT NULL,                    -- 'POST /v1/admin/tenants'
    tenant_id       UUID NOT NULL,
    response_status SMALLINT NOT NULL,                -- HTTP status of the first call
    response_body   JSONB NOT NULL,
    expires_at      TIMESTAMPTZ NOT NULL DEFAULT NOW() + INTERVAL '24 hours',
    PRIMARY KEY (idempotency_key, route, tenant_id)
);

CREATE INDEX admin_idempotency_expires_idx ON admin_idempotency (expires_at);
