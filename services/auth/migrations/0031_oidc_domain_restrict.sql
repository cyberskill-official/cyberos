-- FR-AUTH-104 P0 - restrict an IdP to verified email domains (Google Workspace).
--
-- When allowed_domains is non-empty, the OIDC callback rejects any login whose
-- verified email domain is not in the list (e.g. only @cyberskill.world). An
-- empty array (the default) means no domain restriction, so existing IdP
-- configs keep their current behaviour.
ALTER TABLE oidc_idp_configs
  ADD COLUMN IF NOT EXISTS allowed_domains text[] NOT NULL DEFAULT '{}';
