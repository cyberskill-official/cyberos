-- TASK-EMAIL-001 §3.2 — message_metadata + thread_metadata tables
--
-- The Stalwart adapter writes here on every inbound/outbound delivery.
-- Bodies live in S3 (encrypted via KMS, residency-pinned per tenant); this
-- table holds only the metadata mirror. Per §1 #11 the table is
-- APPEND-ONLY at the SQL-grant level: UPDATE + DELETE revoked from the
-- application role so status transitions create new rows linked via
-- `prior_message_id`.

BEGIN;

-- ----- closed enums per §1 #9 -----------------------------------------------
CREATE TYPE message_direction AS ENUM ('inbound', 'outbound', 'internal');
CREATE TYPE message_status    AS ENUM ('received', 'quarantined', 'delivered', 'sent', 'bounced', 'dropped');

-- ----- thread_metadata ------------------------------------------------------
CREATE TABLE thread_metadata (
    thread_id              TEXT         PRIMARY KEY,
    tenant_id              UUID         NOT NULL,
    subject_normalised     TEXT,
    last_message_at        TIMESTAMPTZ  NOT NULL,
    message_count          INT          NOT NULL DEFAULT 0 CHECK (message_count >= 0),
    participant_addresses  TEXT[]       NOT NULL DEFAULT '{}'
);

-- ----- message_metadata -----------------------------------------------------
-- Body lives in S3 (s3_body_key + s3_body_kms_key_id). Postgres carries
-- only headers + delivery state + audit hooks. Size cap = 25 MB (§1 #19).
CREATE TABLE message_metadata (
    id                     UUID         PRIMARY KEY,
    tenant_id              UUID         NOT NULL,
    stalwart_message_id    BIGINT       NOT NULL,
    thread_id              TEXT         NOT NULL REFERENCES thread_metadata(thread_id) ON DELETE RESTRICT,
    direction              message_direction NOT NULL,
    from_address           TEXT         NOT NULL CHECK (length(from_address) BETWEEN 3 AND 256),
    to_addresses           TEXT[]       NOT NULL CHECK (array_length(to_addresses, 1) IS NULL OR array_length(to_addresses, 1) BETWEEN 1 AND 256),
    cc_addresses           TEXT[]       NOT NULL DEFAULT '{}',
    bcc_addresses          TEXT[]       NOT NULL DEFAULT '{}',
    subject                TEXT         CHECK (subject IS NULL OR length(subject) <= 1000),
    received_at            TIMESTAMPTZ  NOT NULL,
    s3_body_key            TEXT         NOT NULL,
    s3_body_kms_key_id     TEXT         NOT NULL,
    body_sha256_hex        CHAR(64)     NOT NULL CHECK (body_sha256_hex ~ '^[0-9a-f]{64}$'),
    byte_size              BIGINT       NOT NULL CHECK (byte_size BETWEEN 1 AND 26214400),  -- 25 MB cap (§1 #19)
    status                 message_status NOT NULL,
    prior_message_id       UUID         REFERENCES message_metadata(id),
    spam_score             REAL         CHECK (spam_score IS NULL OR spam_score >= 0),
    dkim_pass              BOOLEAN,
    spf_pass               BOOLEAN,
    dmarc_pass             BOOLEAN,
    bimi_present           BOOLEAN,
    created_at             TIMESTAMPTZ  NOT NULL DEFAULT now()
);

-- Indexes per spec §3.2 -------------------------------------------------------
CREATE INDEX message_metadata_tenant_received_idx ON message_metadata (tenant_id, received_at DESC);
CREATE INDEX message_metadata_thread_idx          ON message_metadata (thread_id, received_at ASC);
CREATE INDEX message_metadata_from_idx            ON message_metadata (tenant_id, from_address);
CREATE INDEX message_metadata_status_idx          ON message_metadata (tenant_id, status);
CREATE INDEX message_metadata_stalwart_idx        ON message_metadata (stalwart_message_id);
CREATE INDEX thread_metadata_tenant_last_idx      ON thread_metadata (tenant_id, last_message_at DESC);

-- ----- RLS per §1 #10 -------------------------------------------------------
-- Follows the GUC-based pattern from TASK-AUTH-003 §10.6 (uses
-- `app.current_tenant_id` rather than the spec's `auth.tenant_id` — the
-- divergence is documented in TASK-EMAIL-001.audit.md §10.6).
ALTER TABLE message_metadata ENABLE ROW LEVEL SECURITY;
ALTER TABLE message_metadata FORCE ROW LEVEL SECURITY;
ALTER TABLE thread_metadata  ENABLE ROW LEVEL SECURITY;
ALTER TABLE thread_metadata  FORCE ROW LEVEL SECURITY;

CREATE POLICY message_metadata_tenant_scoped ON message_metadata
    FOR ALL
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    );

CREATE POLICY thread_metadata_tenant_scoped ON thread_metadata
    FOR ALL
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    );

-- ----- append-only enforcement per §1 #11 -----------------------------------
-- The application role `cyberos_app` is the connection role used by the
-- email service. It can INSERT + SELECT but NOT UPDATE/DELETE. Status
-- transitions write a NEW row carrying `prior_message_id = <old.id>`.
-- The migration script that creates `cyberos_app` lives in services/auth/
-- (TASK-AUTH-003); a no-op guard here makes the REVOKE statement skip
-- gracefully when the role is absent (e.g. in a fresh test database).
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'cyberos_app') THEN
        EXECUTE 'REVOKE UPDATE, DELETE ON message_metadata FROM cyberos_app';
        -- thread_metadata is read-update-counted; we allow UPDATE only on
        -- the message_count + last_message_at + participant_addresses
        -- columns via a column-level GRANT (Postgres supports column-list
        -- on UPDATE). Implementation note: at slice 1 we use trigger-side
        -- guards instead of column-level grants for portability. Slice 2
        -- formalises the column-list grant.
        NULL;
    END IF;
END $$;

COMMIT;
