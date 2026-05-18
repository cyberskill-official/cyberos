-- FR-AUTH-101 §1 #10 — SQL function `auth.has_role(role_name)` that consults
-- the per-session GUC `app.roles` (comma-separated kebab-case role names,
-- set by the JWT-verification middleware on every connection acquisition).
--
-- RLS policies on sensitive tables (subjects, audit_row, billing_*, etc.)
-- can compose this function with the tenant-id check:
--   USING (
--     tenant_id::text = current_setting('app.current_tenant_id', true)
--     AND (auth.has_role('tenant-admin') OR auth.has_role('cfo'))
--   )
--
-- ADR: ADR-101-rbac-22-role-catalogue

CREATE SCHEMA IF NOT EXISTS auth;

CREATE OR REPLACE FUNCTION auth.has_role(role_name TEXT)
RETURNS BOOLEAN
LANGUAGE plpgsql
STABLE
AS $$
DECLARE
    csv TEXT;
BEGIN
    csv := current_setting('app.roles', TRUE);
    IF csv IS NULL OR csv = '' THEN
        RETURN FALSE;
    END IF;
    RETURN role_name = ANY(string_to_array(csv, ','));
END;
$$;

COMMENT ON FUNCTION auth.has_role(TEXT) IS
    'FR-AUTH-101 §1 #10 — true if the kebab-case role_name appears in the
     per-session app.roles GUC. Returns false for any session that didn''t
     set the GUC (legacy connections, direct DB access, etc.) — fail-closed.';

-- Convenience: stable function reflecting the active catalogue version.
CREATE OR REPLACE FUNCTION auth.rbac_version()
RETURNS INTEGER
LANGUAGE sql
STABLE
AS $$
    SELECT version FROM role_catalogue_version WHERE id = 1
$$;

COMMENT ON FUNCTION auth.rbac_version() IS
    'FR-AUTH-101 §1 #8 — returns the live RBAC catalogue version for
     verifier-side replay-resistance checks.';

GRANT USAGE ON SCHEMA auth TO cyberos_app, cyberos_ro;
GRANT EXECUTE ON FUNCTION auth.has_role(TEXT) TO cyberos_app, cyberos_ro;
GRANT EXECUTE ON FUNCTION auth.rbac_version() TO cyberos_app, cyberos_ro;
