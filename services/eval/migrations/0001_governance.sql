-- FR-EVAL-001 slice 1: EVAL Phase-0 governance core.
--
-- The governance gate that makes wide, day-1 capture (FR-MEMORY-121/122) and downstream
-- evaluation (FR-EVAL-003) lawful, access-bounded, time-bounded, and tamper-evident BEFORE any
-- interaction event is captured or any evaluation is produced. Tables here are the disclosed
-- monitoring notice + per-subject acknowledgment ledger + data-category/purpose registry +
-- founder/manager-of/self access grants + per-category retention policy.
--
-- Reuses AUTH's per-tenant RLS GUC (app.current_tenant_id, FR-AUTH-003) and the L1 audit chain
-- (services/shared/cyberos-audit-chain). L1 (l1_audit_log) is the append-only source of truth and is
-- NEVER mutated here; this module governs the queryable L2 projections.
--
-- OUT OF SCOPE (DEC-2522, DEC-2525): keystroke logging, screen capture, microphone/camera capture,
-- location tracking, private-life monitoring, and fully-covert / no-notice collection. The disclosed
-- notice plus the acknowledgment gate are the boundary of what this system does. Scope is platform
-- work-interactions ONLY. The allowed data_category names are work-interaction categories such as
-- 'chat_message', 'module_usage', 'task_activity', 'signin_presence', 'document_activity'; a category
-- representing any out-of-scope surface MUST be rejected at the registry layer (registry/mod.rs).
--
-- QUIET OPERATING MODE (founder decision 2026-06-30): the product shows employees NO monitoring /
-- evaluation surface by default; access is founder + managers only; acknowledgment is normally the
-- signed employment-document clause recorded by HR, NOT an in-app click. Hence subject_acknowledgment
-- .ack_source DEFAULTs to 'signed_contract'; the in-app notice surface is OFF by default. The
-- signed-clause disclosure is the lawful basis - there is NO mode that captures a subject who has no
-- acknowledgment row at all (that would be the covert posture DEC-2525 forbids).
--
-- Requires pgcrypto (gen_random_uuid). The deploy/dev bootstrap enables it per database.

-- 1. Versioned per-tenant monitoring / data-processing notice (the disclosed clause). APPEND-ONLY:
--    a correction is a new version; publishing a new version flips the prior current row to false in
--    the same transaction (clause 1). Exactly one row per tenant is current.
CREATE TABLE IF NOT EXISTS monitoring_notice (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id     UUID NOT NULL,
    version       INT  NOT NULL,
    lang_en       TEXT NOT NULL,
    lang_vi       TEXT NOT NULL,
    lawful_basis  TEXT NOT NULL,
    is_current    BOOL NOT NULL DEFAULT TRUE,
    published_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    published_by  UUID NOT NULL,
    UNIQUE (tenant_id, version)
);
-- At most one current notice per tenant (the gate reads "the current version").
CREATE UNIQUE INDEX IF NOT EXISTS monitoring_notice_one_current
    ON monitoring_notice (tenant_id) WHERE is_current;
CREATE INDEX IF NOT EXISTS monitoring_notice_tenant_idx
    ON monitoring_notice (tenant_id);

-- 2. Per-subject acknowledgment of a specific notice version (the acknowledgment ledger). APPEND-ONLY.
--    In the quiet operating mode the normal ack_source is 'signed_contract' (the signed employment
--    document clause, recorded by HR) - hence the DEFAULT - and 'in_app' is the off-by-default surface.
--    A subject is consent-gated until a row here matches the tenant's current notice version.
CREATE TABLE IF NOT EXISTS subject_acknowledgment (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL,
    subject_id      UUID NOT NULL,
    notice_id       UUID NOT NULL REFERENCES monitoring_notice(id),
    notice_version  INT  NOT NULL,
    acknowledged_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    ack_source      TEXT NOT NULL DEFAULT 'signed_contract'
                    CHECK (ack_source IN ('signed_contract','in_app')),
    recorded_by     UUID NOT NULL,
    UNIQUE (tenant_id, subject_id, notice_version)
);
CREATE INDEX IF NOT EXISTS subject_acknowledgment_subject_idx
    ON subject_acknowledgment (tenant_id, subject_id);

-- 4. Data-category + purpose + lawful-basis registry (clause 4, 5). A category MUST declare a purpose
--    and a lawful_basis from the closed enum; minimisation is normative. The registry layer additionally
--    rejects any out-of-scope category name (keystroke / screen / camera / mic / location / private life)
--    at register time - the allowed set is platform work-interactions ONLY (see file header).
CREATE TABLE IF NOT EXISTS data_category (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id     UUID NOT NULL,
    name          TEXT NOT NULL,
    purpose       TEXT NOT NULL CHECK (length(purpose) > 0),
    lawful_basis  TEXT NOT NULL CHECK (lawful_basis IN
                  ('legitimate_interest','contract_performance','legal_obligation','consent')),
    minimized     BOOL NOT NULL DEFAULT TRUE,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by    UUID NOT NULL,
    UNIQUE (tenant_id, name)
);
CREATE INDEX IF NOT EXISTS data_category_tenant_idx
    ON data_category (tenant_id);

-- 8. Access grants - who may read evaluation/monitoring data (clause 7, 8). Resolution is founder OR
--    manager_of(target) OR self OR an explicit active grant. A grant is active when revoked_at IS NULL.
--    APPEND-ONLY for the runtime role; a revoke sets revoked_at via the admin role.
CREATE TABLE IF NOT EXISTS access_grant (
    id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id          UUID NOT NULL,
    viewer_subject_id  UUID NOT NULL,
    target_subject_id  UUID NOT NULL,
    scope              TEXT NOT NULL CHECK (scope IN ('founder','manager_of','self')),
    granted_by         UUID NOT NULL,
    granted_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    revoked_at         TIMESTAMPTZ
);
-- Active-grant lookup by (viewer, target). Partial index = the hot may_read path.
CREATE INDEX IF NOT EXISTS access_grant_lookup_idx
    ON access_grant (tenant_id, viewer_subject_id, target_subject_id)
    WHERE revoked_at IS NULL;
CREATE INDEX IF NOT EXISTS access_grant_target_idx
    ON access_grant (tenant_id, target_subject_id);

-- Per-category retention policy (clause 6). The slice-1 store; the sweeper job (FR-EVAL-001 later
-- sub-task) deletes/redacts L2 projections older than retain_days. Nothing is retained without a policy.
CREATE TABLE IF NOT EXISTS retention_policy (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id        UUID NOT NULL,
    data_category_id UUID NOT NULL REFERENCES data_category(id),
    retain_days      INT  NOT NULL CHECK (retain_days > 0),
    basis            TEXT NOT NULL,
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_by       UUID NOT NULL,
    UNIQUE (tenant_id, data_category_id)
);
CREATE INDEX IF NOT EXISTS retention_policy_tenant_idx
    ON retention_policy (tenant_id);

-- 14. Row-level security: every governance row is scoped to its tenant via the FR-AUTH-003 GUC
--     app.current_tenant_id (set per transaction). The nil tenant bypasses for admin paths. Mirrors
--     services/chat/migrations/0001_chat_core.sql and services/auth/migrations/0021_sessions.sql.
DO $$
DECLARE t TEXT;
BEGIN
  FOREACH t IN ARRAY ARRAY[
      'monitoring_notice','subject_acknowledgment','data_category','access_grant','retention_policy'
  ] LOOP
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

-- 15. Append-only for the runtime role (clause 15). A notice correction is a new version; an
--     acknowledgment is never un-said; a grant is revoked by setting revoked_at (the one permitted
--     column update, reserved for the admin role) rather than deletion. The runtime role gets
--     SELECT + INSERT on the append-only tables (so RLS predicates fire) but NOT UPDATE/DELETE.
--     data_category and retention_policy are operator-configured and may be updated by the admin role;
--     the runtime role reads them on the hot path.
DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'cyberos_app') THEN
    -- Append-only governance ledgers: no UPDATE/DELETE for the runtime role.
    GRANT SELECT, INSERT ON monitoring_notice      TO cyberos_app;
    GRANT SELECT, INSERT ON subject_acknowledgment TO cyberos_app;
    GRANT SELECT, INSERT ON access_grant           TO cyberos_app;  -- revoke (UPDATE revoked_at) = admin role
    -- Operator-configured registry + retention: runtime reads; admin role manages.
    GRANT SELECT, INSERT ON data_category          TO cyberos_app;
    GRANT SELECT, INSERT ON retention_policy        TO cyberos_app;
  END IF;
END $$;
