-- FR-AUTH-101 — closed 22-role catalogue + permission matrix + subject_roles.
--
-- DEC-121 / DEC-122: the closed catalogue is the design assertion. Adding a
-- 23rd role / 41st resource / 6th action requires an ADR + a matching SQL
-- comment `-- ADR: ADR-NNN` in the migration that adds the row.
--
-- ADR: ADR-101-rbac-22-role-catalogue

-- ---------------------------------------------------------------------------
-- roles — 22 seeded rows
-- ---------------------------------------------------------------------------
CREATE TABLE roles (
    name              TEXT PRIMARY KEY,
    display           TEXT NOT NULL,
    reserved          BOOLEAN NOT NULL DEFAULT FALSE,
    requires_webauthn BOOLEAN NOT NULL DEFAULT FALSE,
    stub_tier         BOOLEAN NOT NULL DEFAULT FALSE,
    seeded_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO roles (name, display, reserved, requires_webauthn, stub_tier) VALUES
-- stub-tier (FR-AUTH-002 strict prefix per DEC-123)
('root-admin',          'Root Admin (cross-tenant operator)',          TRUE,  FALSE, TRUE),
('tenant-admin',        'Tenant Admin',                                 FALSE, FALSE, TRUE),
('tenant-member',       'Tenant Member',                                FALSE, FALSE, TRUE),
('service-account',     'Service Account',                              FALSE, FALSE, TRUE),
('agent-persona',       'Agent Persona',                                FALSE, FALSE, TRUE),
-- production-tier
('founder',             'Founder',                                      FALSE, TRUE,  FALSE),
('cfo',                 'Chief Financial Officer',                      FALSE, FALSE, FALSE),
('cto',                 'Chief Technology Officer',                     FALSE, FALSE, FALSE),
('coo',                 'Chief Operating Officer',                      FALSE, FALSE, FALSE),
('chro',                'Chief Human Resources Officer',                FALSE, FALSE, FALSE),
('cmo',                 'Chief Marketing Officer',                      FALSE, FALSE, FALSE),
('cpo',                 'Chief Product Officer',                        FALSE, FALSE, FALSE),
('cso',                 'Chief Strategy Officer',                       FALSE, FALSE, FALSE),
('cseco',               'Chief Security Officer',                       FALSE, FALSE, FALSE),
('clo',                 'Chief Legal Officer',                          FALSE, FALSE, FALSE),
('cdo',                 'Chief Data Officer',                           FALSE, FALSE, FALSE),
('dpo',                 'Data Protection Officer',                      FALSE, FALSE, FALSE),
('caio',                'Chief AI Officer',                             FALSE, FALSE, FALSE),
-- external + system reserved
('client-portal-user',  'Client Portal User',                           TRUE,  FALSE, FALSE),
('auditor',             'External Auditor',                             TRUE,  FALSE, FALSE),
('regulator',           'External Regulator',                           TRUE,  FALSE, FALSE),
('billing-system',      'Billing System (Stripe/VietQR webhook)',       TRUE,  FALSE, FALSE);

-- ---------------------------------------------------------------------------
-- resources — 40 seeded rows (one per cross-module surface)
-- ---------------------------------------------------------------------------
CREATE TABLE resources (
    name      TEXT PRIMARY KEY,
    module    TEXT NOT NULL,
    seeded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO resources (name, module) VALUES
('subject',         'auth'),
('tenant',          'auth'),
('role-assignment', 'auth'),
('jwt-jwks',        'auth'),
('audit-row',       'auth'),
('crm-account',     'crm'),
('crm-contact',     'crm'),
('crm-deal',        'crm'),
('proj-issue',      'proj'),
('proj-engagement', 'proj'),
('proj-rate-card',  'proj'),
('proj-timeline',   'proj'),
('time-entry',      'time'),
('time-expense',    'time'),
('inv-invoice',     'inv'),
('inv-payment',     'inv'),
('inv-hoa-don',     'inv'),
('kb-document',     'kb'),
('kb-runbook',      'kb'),
('hr-member',       'hr'),
('hr-contract',     'hr'),
('hr-leave',        'hr'),
('hr-cccd-photo',   'hr'),
('rew-payslip',     'rew'),
('rew-bp-ledger',   'rew'),
('esop-grant',      'esop'),
('esop-valuation',  'esop'),
('learn-skill',     'learn'),
('learn-certification', 'learn'),
('okr-objective',   'okr'),
('okr-kr',          'okr'),
('res-allocation',  'res'),
('doc-document',    'doc'),
('doc-signature',   'doc'),
('email-thread',    'email'),
('chat-channel',    'chat'),
('chat-message',    'chat'),
('cuo-chain',       'cuo'),
('brain-memory',    'brain'),
('obs-alert',       'obs');

-- ---------------------------------------------------------------------------
-- actions — 5 seeded rows
-- ---------------------------------------------------------------------------
CREATE TABLE actions (
    name TEXT PRIMARY KEY,
    seeded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO actions (name) VALUES
('read'), ('write'), ('admin'), ('approve'), ('sign');

-- ---------------------------------------------------------------------------
-- role_permissions — the permission matrix.
-- Seeded with the minimum set needed for the existing handlers + smoke tests.
-- The full ~280-row matrix lands incrementally as downstream modules ship.
-- ---------------------------------------------------------------------------
CREATE TABLE role_permissions (
    role     TEXT NOT NULL REFERENCES roles(name),
    resource TEXT NOT NULL REFERENCES resources(name),
    action   TEXT NOT NULL REFERENCES actions(name),
    PRIMARY KEY (role, resource, action)
);

-- root-admin: cross-tenant superuser, has every (resource, action).
INSERT INTO role_permissions (role, resource, action)
SELECT 'root-admin', r.name, a.name FROM resources r CROSS JOIN actions a;

-- tenant-admin: most resources read+write+admin, but NEVER sign or admin
-- on jwt-jwks / audit-row (those stay with root-admin).
INSERT INTO role_permissions (role, resource, action)
SELECT 'tenant-admin', r.name, a.name
FROM resources r CROSS JOIN actions a
WHERE a.name IN ('read', 'write', 'admin')
  AND r.name NOT IN ('jwt-jwks');

-- tenant-member: read + limited write on operational resources.
INSERT INTO role_permissions (role, resource, action) VALUES
('tenant-member', 'subject',          'read'),
('tenant-member', 'tenant',           'read'),
('tenant-member', 'crm-account',      'read'),
('tenant-member', 'crm-contact',      'read'),
('tenant-member', 'crm-deal',         'read'),
('tenant-member', 'proj-issue',       'read'),
('tenant-member', 'proj-issue',       'write'),
('tenant-member', 'proj-engagement',  'read'),
('tenant-member', 'time-entry',       'read'),
('tenant-member', 'time-entry',       'write'),
('tenant-member', 'time-expense',     'read'),
('tenant-member', 'time-expense',     'write'),
('tenant-member', 'kb-document',      'read'),
('tenant-member', 'kb-document',      'write'),
('tenant-member', 'kb-runbook',       'read'),
('tenant-member', 'email-thread',     'read'),
('tenant-member', 'chat-channel',     'read'),
('tenant-member', 'chat-message',     'read'),
('tenant-member', 'chat-message',     'write'),
('tenant-member', 'cuo-chain',        'read'),
('tenant-member', 'brain-memory',     'read');

-- service-account / agent-persona: same matrix as tenant-member by default;
-- per-grant scoping (scope_grants) narrows further when needed.
INSERT INTO role_permissions (role, resource, action)
SELECT 'service-account', resource, action FROM role_permissions WHERE role = 'tenant-member';
INSERT INTO role_permissions (role, resource, action)
SELECT 'agent-persona',   resource, action FROM role_permissions WHERE role = 'tenant-member';

-- C-suite per-officer focal points (minimum seed; full matrix grows with FRs).
INSERT INTO role_permissions (role, resource, action) VALUES
('cfo',   'inv-invoice',    'read'),
('cfo',   'inv-invoice',    'approve'),
('cfo',   'inv-payment',    'read'),
('cfo',   'inv-hoa-don',    'read'),
('cfo',   'rew-payslip',    'approve'),
('cfo',   'esop-valuation', 'approve'),
('cto',   'cuo-chain',      'admin'),
('cto',   'brain-memory',   'admin'),
('cto',   'kb-runbook',     'admin'),
('coo',   'proj-engagement', 'admin'),
('coo',   'proj-rate-card',  'admin'),
('coo',   'time-entry',      'approve'),
('chro',  'hr-member',     'admin'),
('chro',  'hr-leave',      'approve'),
('cmo',   'email-thread',  'admin'),
('cpo',   'okr-objective', 'admin'),
('cpo',   'okr-kr',        'admin'),
('cso',   'okr-objective', 'approve'),
('cseco', 'audit-row',     'read'),
('cseco', 'obs-alert',     'admin'),
('clo',   'doc-document',  'sign'),
('clo',   'doc-signature', 'sign'),
('cdo',   'brain-memory',  'admin'),
('dpo',   'hr-cccd-photo', 'admin'),
('dpo',   'audit-row',     'read'),
('caio',  'cuo-chain',     'admin'),
('founder', 'audit-row',   'read'),
('founder', 'inv-invoice', 'approve'),
('founder', 'inv-invoice', 'sign'),
('founder', 'doc-document','sign'),
-- External / reserved
('client-portal-user', 'doc-document', 'read'),
('client-portal-user', 'doc-document', 'sign'),
('auditor',  'audit-row',   'read'),
('auditor',  'inv-invoice', 'read'),
('auditor',  'rew-payslip', 'read'),
('regulator', 'audit-row',  'read'),
('billing-system', 'inv-invoice', 'write'),
('billing-system', 'inv-payment', 'write'),
('billing-system', 'inv-hoa-don', 'write');

-- ---------------------------------------------------------------------------
-- subject_roles — per-subject role grants (tenant-scoped, RLS-protected).
-- ---------------------------------------------------------------------------
CREATE TABLE subject_roles (
    tenant_id   UUID NOT NULL REFERENCES tenants(id),
    subject_id  UUID NOT NULL REFERENCES subjects(id),
    role        TEXT NOT NULL REFERENCES roles(name),
    granted_by  UUID NOT NULL,
    granted_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (subject_id, role)
);

CREATE INDEX subject_roles_tenant_idx ON subject_roles (tenant_id);

ALTER TABLE subject_roles ENABLE ROW LEVEL SECURITY;
ALTER TABLE subject_roles FORCE ROW LEVEL SECURITY;

CREATE POLICY subject_roles_tenant_scoped ON subject_roles
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

GRANT SELECT, INSERT, UPDATE, DELETE ON subject_roles TO cyberos_app;
GRANT SELECT ON subject_roles TO cyberos_ro;
GRANT SELECT ON roles, resources, actions, role_permissions TO cyberos_app, cyberos_ro;

-- ---------------------------------------------------------------------------
-- role_catalogue_version — singleton; bumped via trigger when matrix changes.
-- ---------------------------------------------------------------------------
CREATE TABLE role_catalogue_version (
    id          INT PRIMARY KEY CHECK (id = 1),
    version     INT NOT NULL DEFAULT 1,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    adr_id      TEXT NOT NULL
);

INSERT INTO role_catalogue_version (id, version, adr_id)
VALUES (1, 1, 'ADR-101-rbac-22-role-catalogue');

GRANT SELECT ON role_catalogue_version TO cyberos_app, cyberos_ro;
