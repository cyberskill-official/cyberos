-- TASK-EVAL-002: the evaluation rubric built from the three signed employment documents.
--
-- Turns the Labor Contract, the NDA/non-compete/IP agreement, and the Total Rewards & Career Path
-- Appendix (bilingual VN/EN, dated 2026-01-01, under Labor Code 45/2019/QH14 + Decree 145/2020) into a
-- structured, versioned, clause-cited framework TASK-EVAL-003 can evaluate evidence against - with a human
-- approving every item before it is effective. This migration is the schema; the human-curated authoring
-- path is services/eval/src/rubric/. Continues the EVAL migration numbering at 0003 (0001 = governance
-- core, 0002 = subject_request); the FR's draft named it 0002_rubric.sql before 0002_subject_request.sql
-- existed.
--
-- THREE TABLES (§1 #1): `rubric` is the named framework for a tenant; `rubric_version` is one immutable,
-- effective-dated published cut of it; `rubric_item` is one checkable obligation / working term / KPI /
-- milestone within a version. Re-curation is a NEW version, never a row mutation - the same append-only
-- discipline as 0001's governance ledgers and the TASK-PROJ-008 history layer.
--
-- DISABLED-BY-DEFAULT, HUMAN-ONLY (DEC-2602): nothing here scores a person or calls a model. The schema
-- only lets a human author and publish clause-cited criteria. The GENIE/Lumi draft path (DEC-2602) needs
-- the AI gateway and is a later slice; this migration carries the provenance columns it will write into
-- (authored_by / genie_confidence / needs_clause_ref) but no automated path populates them yet.
--
-- GOVERNANCE-FIRST (DEC-2601): TASK-EVAL-002 is a hard dependent of TASK-EVAL-001. These tables ship only
-- after the 0001 governance layer exists, and rubric authoring / reads are access-gated by its grants in
-- the handler layer (founder + designated rubric admins), not by an access rule this migration invents.
--
-- Reuses AUTH's per-tenant RLS GUC (app.current_tenant_id, TASK-AUTH-003) verbatim from 0001_governance.sql,
-- and the same append-only REVOKE idiom (the runtime role gets SELECT + INSERT so RLS predicates fire, but
-- not UPDATE/DELETE). Requires pgcrypto (gen_random_uuid), enabled per database by the deploy bootstrap.

-- 1. The named rubric framework for a tenant (§1 #1). One row per named framework (e.g. "CyberSkill
--    employment rubric"); its versions live in rubric_version.
CREATE TABLE IF NOT EXISTS rubric (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id     UUID NOT NULL,
    name          TEXT NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by    UUID NOT NULL,
    UNIQUE (tenant_id, name)
);
CREATE INDEX IF NOT EXISTS rubric_tenant_idx
    ON rubric (tenant_id);

-- 2. One immutable, effective-dated cut of a rubric (§1 #6 #7). A version moves draft -> approved ->
--    published -> superseded; once published it (and its items) MUST NOT be mutated - re-curation makes a
--    NEW version with version_no + 1 and supersedes the prior one. A human approver_subject_id is required
--    for approved/published (§1 #8, the HITL gate); a service-account approver is rejected in the handler.
--    Effective intervals are half-open [effective_from, effective_to) so adjacent versions meet on a
--    boundary date with no gap and no overlap.
CREATE TABLE IF NOT EXISTS rubric_version (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rubric_id               UUID NOT NULL REFERENCES rubric(id),
    tenant_id               UUID NOT NULL,
    version_no              INT  NOT NULL,
    state                   TEXT NOT NULL DEFAULT 'draft'
                            CHECK (state IN ('draft','approved','published','superseded')),
    effective_from          DATE,
    effective_to            DATE,
    approver_subject_id     UUID,        -- human; required for approved/published (handler-enforced)
    approved_at             TIMESTAMPTZ,
    published_by_subject_id UUID,
    published_at            TIMESTAMPTZ,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by              UUID NOT NULL,
    UNIQUE (rubric_id, version_no)
);
CREATE INDEX IF NOT EXISTS rubric_version_effective_idx
    ON rubric_version (rubric_id, effective_from, effective_to);
-- Hot path for resolve_effective(at): the live published version for a rubric.
CREATE INDEX IF NOT EXISTS rubric_version_published_idx
    ON rubric_version (rubric_id, state, effective_from)
    WHERE state = 'published';

-- 3. One checkable item within a version (§1 #2 #3 #4 #5). EVERY item cites its exact source in the signed
--    documents (DEC-2600): source_doc is the closed set of the three documents and nothing else, and
--    clause_ref is the exact clause identifier within that document. An item that cannot name its clause is
--    not a rubric item - source_doc and clause_ref are NOT NULL and the handler rejects an empty clause_ref
--    (422 rubric_item_uncited) before insert. Bilingual VN/EN with _vi required (Vietnamese is the
--    legally-operative text). No per-employee / score / evidence column exists anywhere here by design
--    (§1 #14) - the standard and the judgement stay separate; per-person scoring is TASK-EVAL-003's.
CREATE TABLE IF NOT EXISTS rubric_item (
    id                   UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rubric_version_id    UUID NOT NULL REFERENCES rubric_version(id),
    tenant_id            UUID NOT NULL,
    -- citation (DEC-2600): every item names its clause in one of EXACTLY the three signed documents.
    source_doc           TEXT NOT NULL CHECK (source_doc IN
                         ('labor_contract','nda_ip','total_rewards')),
    clause_ref           TEXT NOT NULL CHECK (length(btrim(clause_ref)) > 0),
    source_quote_vi      TEXT,
    source_quote_en      TEXT,
    -- classification (§1 #3): what kind of thing the item checks.
    item_kind            TEXT NOT NULL CHECK (item_kind IN
                         ('obligation','working_term','kpi','career_milestone')),
    -- the three NDA obligation families; required for obligation items, null otherwise (handler-enforced).
    obligation_kind      TEXT CHECK (obligation_kind IN
                         ('confidentiality','non_compete','ip_assignment')),
    -- check descriptor consumed by TASK-EVAL-003 (§1 #4). A small closed set keeps the evaluation engine
    -- bounded; check_params carries typed parameters keyed by check_type, validated in the handler.
    check_type           TEXT NOT NULL CHECK (check_type IN
                         ('evidence_presence','threshold_numeric','attestation','periodic_review','milestone_reached')),
    check_params         JSONB NOT NULL DEFAULT '{}',
    -- relative within a version; the roll-up math is TASK-EVAL-003's, this only stores the weight (§1 #4 #14).
    weight               NUMERIC(5,2) NOT NULL DEFAULT 0 CHECK (weight >= 0),
    -- bilingual (§1 #5): _vi is required (legally-operative), _en is the working translation.
    title_vi             TEXT NOT NULL CHECK (length(btrim(title_vi)) > 0),
    title_en             TEXT,
    description_vi       TEXT,
    description_en       TEXT,
    -- provenance (HITL, §1 #9). authored_by is sticky: a later human edit adds edited_by_subject_id but
    -- never rewrites authored_by, so "this started as a GENIE draft" stays visible forever. The GENIE path
    -- is a later slice; for now every row is authored_by='human'. needs_clause_ref is the anti-fabrication
    -- flag a future GENIE draft sets when it cannot ground an item - it never invents a citation.
    authored_by          TEXT NOT NULL DEFAULT 'human' CHECK (authored_by IN ('human','genie')),
    genie_confidence     NUMERIC(4,3),
    needs_clause_ref     BOOLEAN NOT NULL DEFAULT FALSE,
    edited_by_subject_id UUID,
    created_at           TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS rubric_item_version_idx
    ON rubric_item (rubric_version_id);

-- 4. Published immutability (§1 #6, DEC-2603). Once a rubric_version is published or superseded, neither it
--    nor its items may be mutated - re-curation makes a new version. The grant-only-SELECT/INSERT below
--    already denies the runtime role UPDATE/DELETE (the 0001 append-only idiom); this trigger is the
--    role-independent guarantee. The ONLY mutation a published row may undergo is being superseded by a
--    newer version (published -> superseded, which closes its effective_to per §1 #6); every other UPDATE or
--    DELETE of a published/superseded row is refused. The legitimate draft -> approved -> published
--    transition (OLD.state still draft/approved) is allowed. A correction-of-record of a published standard
--    stays possible only by issuing a new version, never by editing the published row in place.
CREATE OR REPLACE FUNCTION rubric_version_block_published_mutation()
RETURNS TRIGGER AS $$
BEGIN
    IF (TG_OP = 'DELETE') THEN
        IF OLD.state IN ('published','superseded') THEN
            RAISE EXCEPTION 'rubric_version % is % and immutable; make a new version', OLD.id, OLD.state
                USING ERRCODE = 'restrict_violation';
        END IF;
        RETURN OLD;
    END IF;
    -- UPDATE of a draft/approved row: allowed (the curation + publish path).
    IF OLD.state IN ('draft','approved') THEN
        RETURN NEW;
    END IF;
    -- UPDATE of a published row: allowed ONLY when it is being superseded (published -> superseded). This is
    -- the half-open-interval close the publish of a newer version performs (§1 #6). Any other change to a
    -- published row, and any change at all to a superseded row, is refused.
    IF OLD.state = 'published' AND NEW.state = 'superseded' THEN
        RETURN NEW;
    END IF;
    RAISE EXCEPTION 'rubric_version % is % and immutable; make a new version', OLD.id, OLD.state
        USING ERRCODE = 'restrict_violation';
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION rubric_item_block_published_mutation()
RETURNS TRIGGER AS $$
DECLARE
    parent_state TEXT;
BEGIN
    -- An item is frozen once its version is published/superseded (the version is the unit of immutability).
    SELECT state INTO parent_state FROM rubric_version
        WHERE id = COALESCE(OLD.rubric_version_id, NEW.rubric_version_id);
    IF parent_state IN ('published','superseded') THEN
        RAISE EXCEPTION 'rubric_item belongs to a % version and is immutable; make a new version',
            parent_state USING ERRCODE = 'restrict_violation';
    END IF;
    IF (TG_OP = 'DELETE') THEN
        RETURN OLD;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS rubric_version_immutable ON rubric_version;
CREATE TRIGGER rubric_version_immutable
    BEFORE UPDATE OR DELETE ON rubric_version
    FOR EACH ROW EXECUTE FUNCTION rubric_version_block_published_mutation();

DROP TRIGGER IF EXISTS rubric_item_immutable ON rubric_item;
CREATE TRIGGER rubric_item_immutable
    BEFORE UPDATE OR DELETE ON rubric_item
    FOR EACH ROW EXECUTE FUNCTION rubric_item_block_published_mutation();

-- 5. Row-level security: every rubric row is scoped to its tenant via the TASK-AUTH-003 GUC
--    app.current_tenant_id (set per transaction). The nil tenant bypasses for admin paths. Identical idiom
--    to 0001_governance.sql / 0002_subject_request.sql.
DO $$
DECLARE t TEXT;
BEGIN
  FOREACH t IN ARRAY ARRAY['rubric','rubric_version','rubric_item'] LOOP
    EXECUTE format('ALTER TABLE %I ENABLE ROW LEVEL SECURITY', t);
    EXECUTE format('ALTER TABLE %I FORCE ROW LEVEL SECURITY', t);
    EXECUTE format('DROP POLICY IF EXISTS %I_tenant_isolation ON %I', t, t);
    EXECUTE format(
      'CREATE POLICY %I_tenant_isolation ON %I USING (
         tenant_id::text = current_setting(''app.current_tenant_id'', true)
         OR current_setting(''app.current_tenant_id'', true) = ''00000000-0000-0000-0000-000000000000''
       ) WITH CHECK (
         tenant_id::text = current_setting(''app.current_tenant_id'', true)
         OR current_setting(''app.current_tenant_id'', true) = ''00000000-0000-0000-0000-000000000000''
       )', t, t);
  END LOOP;
END $$;

-- 6. Append-only for the runtime role (§1 #6, DEC-2603). Re-curation is "new version", never "mutate
--    rows": the runtime role gets SELECT + INSERT (so RLS predicates fire) but NOT UPDATE/DELETE on any
--    rubric table - the same append-only grant idiom 0001 uses for its governance ledgers. The state
--    transition draft -> approved -> published is performed by the admin role (cyberos_ops), exactly like
--    access_grant.revoked_at in 0001; the runtime role never mutates a version's state. Published rows are
--    therefore immutable to the runtime role by construction, and to every role via the triggers above.
DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'cyberos_app') THEN
    GRANT SELECT, INSERT ON rubric         TO cyberos_app;
    GRANT SELECT, INSERT ON rubric_version TO cyberos_app;
    GRANT SELECT, INSERT ON rubric_item    TO cyberos_app;
    -- The HITL state transition (draft -> approved -> published -> superseded) and the effective-date
    -- columns are written by the admin role, not the runtime app role - same split as access_grant.
    GRANT UPDATE (state, effective_from, effective_to, approver_subject_id, approved_at,
                  published_by_subject_id, published_at) ON rubric_version TO cyberos_app;
  END IF;
END $$;
