-- FR-EMAIL-009 — outbound 1:1 send queue + suppression list.

BEGIN;

CREATE TABLE outbound_messages (
    id                    UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id             UUID         NOT NULL,
    sender_subject_id     UUID         NOT NULL,
    to_addresses          TEXT[]       NOT NULL CHECK (array_length(to_addresses, 1) BETWEEN 1 AND 256),
    cc_addresses          TEXT[]       NOT NULL DEFAULT '{}',
    bcc_addresses         TEXT[]       NOT NULL DEFAULT '{}',
    subject               TEXT         NOT NULL CHECK (length(subject) BETWEEN 1 AND 1000),
    body_text_sha256      CHAR(64)     NOT NULL CHECK (body_text_sha256 ~ '^[0-9a-f]{64}$'),
    body_html_sha256      CHAR(64)     CHECK (body_html_sha256 IS NULL OR body_html_sha256 ~ '^[0-9a-f]{64}$'),
    in_reply_to           TEXT,
    status                TEXT         NOT NULL CHECK (status IN (
        'drafting', 'queued', 'sent', 'bounced_hard', 'bounced_soft', 'complaint', 'suppressed'
    )) DEFAULT 'drafting',
    confirm_token_sha256  CHAR(64)     NOT NULL CHECK (confirm_token_sha256 ~ '^[0-9a-f]{64}$'),
    confirm_expires_at    TIMESTAMPTZ  NOT NULL,
    dkim_outcome          TEXT CHECK (dkim_outcome IS NULL OR dkim_outcome IN (
        'signed_ed25519', 'signed_rsa', 'sign_failed_no_key', 'sign_failed_kms'
    )),
    queued_at             TIMESTAMPTZ,
    sent_at               TIMESTAMPTZ,
    last_bounce_at        TIMESTAMPTZ,
    created_at            TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at            TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE TABLE email_suppression_list (
    id                    UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id             UUID         NOT NULL,
    address_hash16        CHAR(16)     NOT NULL CHECK (address_hash16 ~ '^[0-9a-f]{16}$'),
    reason                TEXT         NOT NULL CHECK (reason IN ('hard_bounce', 'complaint', 'manual')),
    source_message_id     UUID         REFERENCES outbound_messages(id) ON DELETE SET NULL,
    suppressed_at         TIMESTAMPTZ  NOT NULL DEFAULT now(),
    unsuppressed_at       TIMESTAMPTZ,
    UNIQUE (tenant_id, address_hash16)
);

CREATE TABLE outbound_delivery_events (
    id                    UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id             UUID         NOT NULL,
    message_id            UUID         NOT NULL REFERENCES outbound_messages(id) ON DELETE RESTRICT,
    event_kind            TEXT         NOT NULL CHECK (event_kind IN (
        'email.send_queued',
        'email.send_delivered',
        'email.send_bounced',
        'email.send_complaint',
        'email.send_suppressed'
    )),
    payload               JSONB        NOT NULL DEFAULT '{}'::jsonb,
    trace_id              TEXT,
    created_at            TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE INDEX outbound_messages_sender_hour_idx ON outbound_messages (tenant_id, sender_subject_id, created_at DESC);
CREATE INDEX outbound_messages_status_idx ON outbound_messages (tenant_id, status, created_at DESC);
CREATE INDEX email_suppression_active_idx ON email_suppression_list (tenant_id, address_hash16) WHERE unsuppressed_at IS NULL;
CREATE INDEX outbound_delivery_events_message_idx ON outbound_delivery_events (message_id, created_at DESC);

ALTER TABLE outbound_messages ENABLE ROW LEVEL SECURITY;
ALTER TABLE outbound_messages FORCE ROW LEVEL SECURITY;
ALTER TABLE email_suppression_list ENABLE ROW LEVEL SECURITY;
ALTER TABLE email_suppression_list FORCE ROW LEVEL SECURITY;
ALTER TABLE outbound_delivery_events ENABLE ROW LEVEL SECURITY;
ALTER TABLE outbound_delivery_events FORCE ROW LEVEL SECURITY;

CREATE POLICY outbound_messages_tenant_scoped ON outbound_messages
    FOR ALL
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    );

CREATE POLICY email_suppression_list_tenant_scoped ON email_suppression_list
    FOR ALL
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    );

CREATE POLICY outbound_delivery_events_tenant_scoped ON outbound_delivery_events
    FOR ALL
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    );

CREATE TRIGGER outbound_messages_updated_at_trg
    BEFORE UPDATE ON outbound_messages
    FOR EACH ROW
    WHEN (OLD.* IS DISTINCT FROM NEW.*)
    EXECUTE FUNCTION email_touch_updated_at();

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'cyberos_app') THEN
        EXECUTE 'REVOKE UPDATE, DELETE ON outbound_delivery_events FROM cyberos_app';
    END IF;
END $$;

COMMIT;
