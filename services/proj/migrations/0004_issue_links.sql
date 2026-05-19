-- FR-PROJ-001 §3 — issue_links table.
--
-- Cross-module + intra-issue links. The link_type enum is closed at this
-- slice; symmetric types (blocks/blocked_by, duplicates/duplicated_by)
-- are auto-inserted bidirectionally per §1 #9.
--
-- RLS policy joins back to `issues` so the link inherits the tenant
-- scope of its anchor.

BEGIN;

CREATE TABLE issue_links (
    issue_id     UUID         NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    linked_to_id UUID         NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    link_type    TEXT         NOT NULL CHECK (link_type IN (
        'duplicates', 'duplicated_by',
        'blocks', 'blocked_by',
        'related',
        'derived_from_email_thread',
        'derived_from_chat_thread',
        'derived_from_meeting_decision'
    )),
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT now(),
    PRIMARY KEY (issue_id, linked_to_id, link_type)
);

CREATE INDEX issue_links_linked_to_idx ON issue_links (linked_to_id, link_type);

-- Forbid self-links (an issue can't block / duplicate itself).
ALTER TABLE issue_links ADD CONSTRAINT issue_links_no_self_ref CHECK (issue_id <> linked_to_id);

ALTER TABLE issue_links ENABLE ROW LEVEL SECURITY;
ALTER TABLE issue_links FORCE ROW LEVEL SECURITY;

-- The link is tenant-scoped via the issue row it anchors.
CREATE POLICY issue_links_tenant_scoped ON issue_links
    FOR ALL
    USING (
        EXISTS (
            SELECT 1 FROM issues
            WHERE id = issue_id
              AND (
                  tenant_id::text = current_setting('app.current_tenant_id', true)
                  OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
              )
        )
    )
    WITH CHECK (
        EXISTS (
            SELECT 1 FROM issues
            WHERE id = issue_id
              AND (
                  tenant_id::text = current_setting('app.current_tenant_id', true)
                  OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
              )
        )
    );

COMMIT;
