-- TASK-AUTH-103 slice-2 — per-IdP `allow_unsigned` flag replaces the
-- `AUTH_SAML_ALLOW_UNSIGNED=1` env-var escape hatch. With the column
-- defaulting to FALSE, fresh tenants reject unsigned/unverified Responses
-- by default and operators must explicitly opt-in per IdP config — usually
-- only against dev-fixture IdPs.
--
-- The env-var escape hatch is removed in the same commit; future deploys
-- must rely on the column.

ALTER TABLE saml_idp_configs
    ADD COLUMN IF NOT EXISTS allow_unsigned BOOLEAN NOT NULL DEFAULT FALSE;

COMMENT ON COLUMN saml_idp_configs.allow_unsigned IS
    'TASK-AUTH-103 slice-2 — when TRUE, the ACS endpoint accepts Responses '
    'whose <ds:Signature> fails verification. Production IdPs MUST keep '
    'this FALSE; dev fixtures may set TRUE.';
