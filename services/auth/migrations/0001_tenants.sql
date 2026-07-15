-- TASK-AUTH-001 — Tenant schema.
--
-- The tenant table is the anchor of every tenant-scoped table in CyberOS.
-- Per AUTHORING_DISCIPLINE §3.1 rule 1, the root tenant is `Uuid::nil()`
-- (000…000) — never numeric zero.

CREATE TABLE tenants (
    id              UUID PRIMARY KEY,
    slug            TEXT NOT NULL UNIQUE,            -- DNS-safe handle: 'acme-corp'
    display_name    TEXT NOT NULL,
    country         CHAR(2) NOT NULL DEFAULT 'VN',   -- ISO-3166-1 alpha-2
    plan_tier       TEXT NOT NULL DEFAULT 'starter', -- TASK-TEN-002: starter | team | enterprise
    status          TEXT NOT NULL DEFAULT 'active',  -- active | terminating | terminated | hostile
    residency       TEXT NOT NULL DEFAULT 'sg-1',    -- sg-1 | eu-1 | us-1 | vn-1
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT tenants_slug_format CHECK (slug ~ '^[a-z0-9][a-z0-9-]{0,62}$'),
    CONSTRAINT tenants_status_enum CHECK (status IN ('active', 'terminating', 'terminated', 'hostile')),
    CONSTRAINT tenants_plan_enum   CHECK (plan_tier IN ('starter', 'team', 'enterprise', 'sandbox'))
);

-- Seed the root tenant (TASK-AUTH-006 bootstrap CLI later upserts ops admin
-- here). The nil UUID is the canonical root per AUTHORING_DISCIPLINE §3.1 #1.
INSERT INTO tenants (id, slug, display_name, country, plan_tier, status, residency)
VALUES (
    '00000000-0000-0000-0000-000000000000',
    'root',
    'CyberOS Root Tenant',
    'VN',
    'enterprise',
    'active',
    'sg-1'
);
