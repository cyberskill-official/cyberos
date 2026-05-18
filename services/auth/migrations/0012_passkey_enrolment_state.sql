-- FR-AUTH-105 — WebAuthn/Passkey enrolment + login transient state.
--
-- The webauthn-rs crate hands us a `PasskeyRegistration` / `PasskeyAuthentication`
-- object at the begin step that the finish step must consume. We persist it
-- per (subject, ceremony_id) keyed row so a load-balancer cannot break the
-- ceremony across two pods.
--
-- Active credentials themselves live in `mfa_factors` (migration 0009) with
-- factor_type='webauthn', cred_id, public_key, sign_count populated.
--
-- ADR: ADR-101-rbac-22-role-catalogue (founder requires WebAuthn — DEC-128).

CREATE TABLE passkey_enrolment_state (
    ceremony_id     UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL REFERENCES tenants(id),
    subject_id      UUID NOT NULL REFERENCES subjects(id),
    flow            TEXT NOT NULL,                              -- 'enrol' | 'login'
    state_json      JSONB NOT NULL,                             -- webauthn-rs Passkey{Registration|Authentication} blob
    label           TEXT,                                       -- caller-supplied label for the new factor
    expires_at      TIMESTAMPTZ NOT NULL DEFAULT NOW() + INTERVAL '5 minutes',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT passkey_state_flow_enum CHECK (flow IN ('enrol', 'login'))
);

CREATE INDEX passkey_enrolment_state_expires_idx ON passkey_enrolment_state (expires_at);

ALTER TABLE passkey_enrolment_state ENABLE ROW LEVEL SECURITY;
ALTER TABLE passkey_enrolment_state FORCE ROW LEVEL SECURITY;
CREATE POLICY passkey_state_tenant_scoped ON passkey_enrolment_state
    FOR ALL
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) =
           '00000000-0000-0000-0000-000000000000'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) =
           '00000000-0000-0000-0000-000000000000'
    );

GRANT SELECT, INSERT, DELETE ON passkey_enrolment_state TO cyberos_app;
GRANT SELECT ON passkey_enrolment_state TO cyberos_ro;

-- Convenience cleanup: anything older than 1 hour is fossilised state from a
-- crashed ceremony. A periodic sweeper (deferred) DELETE WHERE expires_at < NOW().
