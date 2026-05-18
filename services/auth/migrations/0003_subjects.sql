-- FR-AUTH-002 — Subject (user / agent / system principal) schema.
--
-- Every subject belongs to exactly one tenant. RLS (migration 0005) restricts
-- reads to subjects within the caller's tenant. Service-role subjects (kind
-- = 'system' / 'agent') are exempt from password but still tenant-scoped.

CREATE TABLE subjects (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL REFERENCES tenants(id),
    handle          TEXT NOT NULL,                    -- '@stephen', '@cuo', '@brain-ingest'
    display_name    TEXT,
    email           TEXT,                             -- nullable for agents/systems
    kind            TEXT NOT NULL DEFAULT 'human',    -- 'human' | 'agent' | 'system'
    password_hash   TEXT,                             -- bcrypt; null for agent/system
    status          TEXT NOT NULL DEFAULT 'active',   -- 'active' | 'revoked' | 'pending'
    roles           TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT subjects_handle_per_tenant UNIQUE (tenant_id, handle),
    CONSTRAINT subjects_handle_format CHECK (handle ~ '^@[a-zA-Z0-9_.-]{1,38}$'),
    CONSTRAINT subjects_kind_enum CHECK (kind IN ('human', 'agent', 'system')),
    CONSTRAINT subjects_status_enum CHECK (status IN ('active', 'revoked', 'pending')),
    CONSTRAINT subjects_human_has_password CHECK (
        kind != 'human' OR password_hash IS NOT NULL OR status = 'pending'
    )
);

CREATE INDEX subjects_tenant_idx ON subjects (tenant_id);
CREATE INDEX subjects_email_idx  ON subjects (lower(email)) WHERE email IS NOT NULL;
