-- FR-AUTH-109 — stub→full migration state.
--
-- Per DEC-125 + AUTHORING_DISCIPLINE §3.4: existing FR-AUTH-002 / 005 / 006
-- access tokens (issued before FR-AUTH-101 RBAC catalogue shipped) carry no
-- `rbac_v` claim. The verifier honours them as implicit `rbac_v = 1` for a
-- 30-day grace window after FR-AUTH-101 lands. After grace closes,
-- missing-claim tokens are rejected with `401 rbac_version_required`.
--
-- This table is a singleton state row: when the FR-AUTH-101 migration ran,
-- and when the grace closes. The verifier reads it on every request via
-- a cached `MigrationState` snapshot refreshed alongside the RBAC matrix.
--
-- ADR: ADR-101-rbac-22-role-catalogue

CREATE TABLE auth_migration_state (
    id                      INT PRIMARY KEY CHECK (id = 1),
    fr_auth_101_shipped_at  TIMESTAMPTZ NOT NULL,        -- when migration 0007 ran (RBAC catalogue)
    grace_window_days       INTEGER NOT NULL DEFAULT 30,
    grace_closes_at         TIMESTAMPTZ NOT NULL,        -- computed at row insert
    extended_by             UUID,                        -- subject_id of operator who extended
    extension_reason        TEXT,
    last_updated_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Seed the singleton: the migration 0007 ship date is the timestamp of THIS
-- migration's apply (close enough — operators care about the boundary in
-- days, not seconds).
INSERT INTO auth_migration_state
    (id, fr_auth_101_shipped_at, grace_window_days, grace_closes_at)
VALUES
    (1, NOW(), 30, NOW() + INTERVAL '30 days');

GRANT SELECT ON auth_migration_state TO cyberos_app, cyberos_ro;
GRANT UPDATE ON auth_migration_state TO cyberos_app;
