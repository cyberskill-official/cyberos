---
id: TASK-KB-001
title: "KB Document schema — slug + markdown body + YAML frontmatter + closed category enum + 3-tier ACL + immutable versions + translation_of"
module: KB
priority: MUST
status: draft
verify: T
phase: P1
milestone: P1 · slice 1
slice: 1
owner: Stephen Cheng (CDO)
created: 2026-05-16
shipped: null
memory_chain_hash: null
related_tasks: [TASK-AUTH-003, TASK-AUTH-101, TASK-AI-003, TASK-MEMORY-101, TASK-KB-002, TASK-KB-003, TASK-KB-004, TASK-KB-005, TASK-KB-007, TASK-KB-008, TASK-KB-009]
depends_on: [TASK-AUTH-003, TASK-AUTH-101]
blocks: [TASK-KB-002, TASK-KB-003, TASK-KB-004, TASK-KB-005, TASK-KB-007, TASK-KB-008, TASK-KB-009]   # all 7 entries are placeholders — not yet specified (downstream consumers)

source_pages:
  - website/docs/modules/kb.html#what
  - website/docs/modules/kb.html#data-model
  - website/docs/modules/kb.html#categories
source_decisions:
  - DEC-240 (closed category enum at 5 values: how_to · reference · decision_log · policy · runbook)
  - DEC-241 (closed permission tier enum at 3 values: public · org_only · role_restricted)
  - DEC-242 (closed language enum at 2 values: vi · en; adding zh/ja/etc. is an ADR)
  - DEC-243 (every save creates a new immutable version; documents reference current_version_id; old versions are never UPDATEd or DELETEd)
  - DEC-244 (slug uniqueness is per-tenant per-language; same slug in vi + en is allowed and is the translation pair indicator)
  - DEC-245 (translation_of is a self-FK between documents in different languages; same-language link forbidden)
  - DEC-246 (markdown body 1–500_000 chars; frontmatter YAML-validated against schema; oversize is an FR-KB-2xx import problem, not a slice-1 case)
  - DEC-247 (REVOKE UPDATE, DELETE on document_versions from cyberos_app — append-only enforced at SQL grant)
  - "DEC-248 (frontmatter required keys: title, category, language, permission; optional: tags, owner_subject_id, translation_of, summary, applicability_tags for runbooks)"
  - DEC-249 (memory audit kinds: kb.document_created, kb.document_versioned, kb.document_acl_changed, kb.document_archived)
  - DEC-250 (role_restricted tier requires non-empty allowed_role_codes array referencing TASK-AUTH-101 closed role enum; unknown roles rejected at handler)
  - DEC-251 (archived documents: status='archived' marks the doc as no longer current; existing versions remain queryable but TASK-KB-002's search defaults to active-only; rendering still works)
  - DEC-252 (frontmatter `category` MUST match the closed enum; mismatch → 400 unknown_category at the API boundary; ADR to add a 6th category)
  - PDPL Art. 13 (data minimisation — KB docs storing PII subject to scrubbing per TASK-MEMORY-111 before memory ingest)
  - ISO 27001:2022 A.5.13 (information classification — KB's 3 permission tiers map to the standard's classify-and-protect requirement)

language: rust 1.81 + sql
service: cyberos/services/kb/
new_files:
  - services/kb/migrations/0001_documents.sql                         # documents + document_versions + ENUMs + RLS + REVOKE writes + uniqueness indexes
  - services/kb/migrations/0002_document_views.sql                    # current_documents_view + active_documents_view + version chain walker
  - services/kb/src/lib.rs                                            # crate root
  - services/kb/src/types.rs                                          # Document, DocumentVersion, Category, PermissionTier, DocumentStatus, Language enums
  - services/kb/src/frontmatter.rs                                    # YAML schema validator (serde_yaml + custom validation)
  - services/kb/src/repo/documents.rs                                 # CRUD: create + get + list + new_version + archive
  - services/kb/src/repo/versions.rs                                  # append-only version writer
  - services/kb/src/audit/doc_events.rs                               # canonical kb.document_* memory row builders
  - services/kb/src/handlers/documents.rs                             # POST/GET/PATCH/DELETE /v1/kb/documents + POST /versions
  - services/kb/Cargo.toml                                            # +sqlx, +uuid, +serde, +serde_yaml, +chrono, +async-trait, +cyberos-cli-exit
  - services/kb/tests/documents_crud_test.rs                          # happy + invalid + RLS + idempotent
  - services/kb/tests/category_enum_closed_test.rs                    # SQL enum + Rust enum cross-validation
  - services/kb/tests/permission_tier_enum_closed_test.rs             # same shape
  - services/kb/tests/frontmatter_validation_test.rs                  # required keys + closed values + tag bounds
  - services/kb/tests/version_append_only_test.rs                     # UPDATE/DELETE rejected by SQL grant
  - services/kb/tests/version_chain_test.rs                           # save N times → N versions; current_version_id always points at latest
  - services/kb/tests/translation_of_test.rs                          # self-FK across languages; same-language reject
  - services/kb/tests/slug_uniqueness_test.rs                         # same slug different language allowed; same slug same language rejected
  - services/kb/tests/role_restricted_validation_test.rs              # unknown role → 400; empty allowed_role_codes → 400
  - services/kb/tests/markdown_size_bounds_test.rs                    # 0 chars → 400; > 500_000 → 400
  - services/kb/tests/archive_workflow_test.rs                        # status=archived; new versions still allowed via separate handler; rendering preserved
modified_files:
  - services/auth/src/rls/templates.rs                                # add documents + document_versions to TENANT_SCOPED_TABLES

allowed_tools:
  - file_read: services/kb/**
  - file_read: services/auth/src/rls/**
  - file_write: services/kb/{src,tests,migrations}/**
  - bash: cd services/kb && cargo test
  - bash: psql -f services/kb/migrations/0001_documents.sql (local Postgres only)

disallowed_tools:
  - allow UPDATE on document_versions (per DEC-247; SQL-grant-enforced)
  - allow DELETE on document_versions (same)
  - add a 6th category without an ADR (per DEC-240 + DEC-252)
  - add a 4th permission tier without an ADR (per DEC-241)
  - allow translation_of self-link or same-language link (per DEC-245)
  - ship the rendering + sanitisation logic here (that is TASK-KB-002's responsibility)
  - ship search indexing logic here (TASK-KB-004/005/006)

effort_hours: 6
subtasks:
  - "1.0h: 0001_documents.sql — documents + document_versions tables + 4 closed enums + RLS + REVOKE writes on versions"
  - "0.5h: 0002_document_views.sql — current_documents_view (status!='archived') + active_documents_view"
  - "0.5h: types.rs — Document, DocumentVersion, Category(5), PermissionTier(3), DocumentStatus(3), Language(2)"
  - "0.8h: frontmatter.rs — YAML schema validator with required-key check + closed-enum value check + tag-bounds"
  - "0.5h: repo/documents.rs — create + get + list + new_version + archive"
  - "0.3h: repo/versions.rs — append-only writer"
  - "0.4h: audit/doc_events.rs — 4 row builders"
  - "0.5h: handlers/documents.rs — REST surface with idempotency"
  - "1.5h: tests — 11 test files covering enum closure, append-only, version chain, translation pair, slug uniqueness, role validation, frontmatter, archive flow"

risk_if_skipped: "KB is the canonical source for AI-grounded retrieval — every Genie answer, every OBS auto-runbook lookup, every CUO synthesis needs structured docs. Every downstream KB FR (TASK-KB-002 rendering, TASK-KB-003 permission tiers, TASK-KB-004 FTS5 search, TASK-KB-005 BGE-M3 semantic search, TASK-KB-007 'Ask this page' Q&A, TASK-KB-008 runbook category) reads from this schema. Without DEC-243's immutable versions, every save silently rewrites history — citations become unverifiable. Without DEC-244's per-tenant per-language uniqueness, the translation linkage breaks (the natural pair indicator is shared slug). Without DEC-247's SQL-grant append-only, a developer typo could mass-delete the corpus and the audit chain claim collapses. Without DEC-252's closed category enum, free-form categories proliferate and TASK-KB-008's runbook filter can't be precise. The 6h effort hardens the schema so every downstream KB FR can trust the invariants."
---

## §1 — Description (BCP-14 normative)

The KB service **MUST** ship the Document schema as the canonical structured-knowledge surface for the tenant. Each requirement:

1. **MUST** define `documents` with: `id UUID PRIMARY KEY`, `tenant_id UUID NOT NULL`, `slug TEXT NOT NULL`, `language language_code NOT NULL`, `title TEXT NOT NULL`, `category document_category NOT NULL`, `permission document_permission NOT NULL`, `allowed_role_codes TEXT[] NOT NULL DEFAULT '{}'`, `current_version_id UUID NOT NULL REFERENCES document_versions(id) DEFERRABLE INITIALLY IMMEDIATE`, `translation_of UUID REFERENCES documents(id)`, `status document_status NOT NULL DEFAULT 'active'`, `owner_subject_id UUID REFERENCES auth.subjects(id)`, `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`, `updated_at TIMESTAMPTZ NOT NULL DEFAULT now()`. Full DDL in §3.1.

2. **MUST** define `document_versions` with: `id UUID PRIMARY KEY`, `document_id UUID NOT NULL REFERENCES documents(id) ON DELETE RESTRICT`, `tenant_id UUID NOT NULL`, `version_number INT NOT NULL`, `markdown_body TEXT NOT NULL CHECK (length(markdown_body) BETWEEN 1 AND 500000)`, `frontmatter_yaml TEXT NOT NULL`, `body_sha256 CHAR(64) NOT NULL`, `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`, `created_by_subject_id UUID NOT NULL REFERENCES auth.subjects(id)`, `change_summary TEXT CHECK (change_summary IS NULL OR length(change_summary) BETWEEN 1 AND 500)`.

3. **MUST** declare the closed `document_category` Postgres enum with exactly 5 values (per DEC-240): `'how_to'`, `'reference'`, `'decision_log'`, `'policy'`, `'runbook'`. Adding a 6th category is an ADR.

4. **MUST** declare the closed `document_permission` Postgres enum with exactly 3 values (per DEC-241): `'public'`, `'org_only'`, `'role_restricted'`. Adding a 4th tier is an ADR.

5. **MUST** declare the closed `language_code` Postgres enum with exactly 2 values (per DEC-242): `'vi'`, `'en'`. Adding zh/ja/etc. is an ADR.

6. **MUST** declare the closed `document_status` Postgres enum with exactly 3 values: `'active'`, `'archived'`, `'draft'`. `draft` is the create-state until first publish; `active` is the normal state; `archived` removes from default search results but retains rendering.

7. **MUST** enforce RLS with both `USING` and `WITH CHECK` clauses on `documents` AND `document_versions`. Policy: `tenant_id = current_setting('auth.tenant_id')::uuid`. Cross-tenant reads return 0 rows; cross-tenant writes fail `permission_denied`.

8. **MUST** be **append-only** on `document_versions` at the SQL-grant layer (per DEC-247 + task-audit skill rule 12). `REVOKE UPDATE, DELETE ON document_versions FROM cyberos_app;`. Every "edit" creates a new version row and updates `documents.current_version_id` to point at it.

9. **MUST** enforce **slug uniqueness per (tenant_id, language)** (per DEC-244). `CREATE UNIQUE INDEX uniq_doc_slug ON documents (tenant_id, language, slug);`. Same slug in `vi` + `en` is allowed and indicates translation pairing (verified by translation_of FK).

10. **MUST** validate `translation_of` FK at INSERT/UPDATE: target document MUST exist in the same `tenant_id` AND have a different `language` (per DEC-245). Same-language self-link → 400 `translation_must_cross_language`; cross-tenant link → 400 `translation_cross_tenant`. A `BEFORE INSERT OR UPDATE` trigger enforces.

11. **MUST** validate `role_restricted` permission tier: `allowed_role_codes` MUST be non-empty AND every element MUST match the TASK-AUTH-101 closed role enum (per DEC-250). Unknown role → 400 `unknown_role: <code>`. Empty array with permission='role_restricted' → 400 `role_restricted_requires_roles`.

12. **MUST** ship the frontmatter validator at `services/kb/src/frontmatter.rs`. Required keys: `title`, `category`, `language`, `permission`. Optional keys: `tags` (string array ≤ 20 elements, each 1–32 chars), `owner_subject_id` (UUID), `translation_of` (slug — resolves to document_id at INSERT time), `summary` (1–200 chars), `applicability_tags` (object with `provider`, `region`, `severity` — only for `category=runbook`). Unknown top-level keys → 400 `unknown_frontmatter_key: <name>`.

13. **MUST** compute `body_sha256` server-side on every version write: `SHA-256(markdown_body)` as 64-char lowercase hex. Used by TASK-KB-002's render-cache invalidation and by ingestion replay. Identical body in successive saves → still a new version row (the operator wanted to record a change for some reason — maybe metadata-only changes).

14. **MUST** emit memory audit row `kb.document_created` on document creation (first version). The row carries `{document_id, tenant_id, slug, language, category, permission, owner_subject_id_hash16, created_by_subject_id_hash16, body_sha256, version_number, ts_ns}`.

15. **MUST** emit `kb.document_versioned` on every subsequent version write. Same shape as `kb.document_created` plus `prev_version_id` and `change_summary`.

16. **MUST** emit `kb.document_acl_changed` on every permission or allowed_role_codes change. Carries `{document_id, old_permission, new_permission, old_allowed_role_codes, new_allowed_role_codes, changed_by_subject_id_hash16, ts_ns}`. This is a sev-2 audit (TASK-OBS-007 notification target — ACL changes deserve operator attention).

17. **MUST** emit `kb.document_archived` on transition to `status='archived'`. Carries `{document_id, archived_by_subject_id_hash16, reason TEXT, ts_ns}`.

18. **MUST** maintain `current_version_id` pointer atomically on every version save: in one transaction, INSERT into `document_versions` AND UPDATE `documents.current_version_id`. The FK is `DEFERRABLE INITIALLY IMMEDIATE` so the document row can be UPDATEd with a still-uncommitted version id within the same transaction (Postgres-specific).

19. **MUST** expose REST handlers:
    - `POST /v1/kb/documents` — create document + initial version. Caller `Resource::KbDocument + Action::Write`.
    - `GET /v1/kb/documents/{id}` — return doc + current version. Caller `Action::Read`; permission tier additionally gates (public always allowed; org_only requires same-tenant subject; role_restricted requires intersection of caller roles with `allowed_role_codes`).
    - `GET /v1/kb/documents?slug=<>&language=<>&category=<>&status=<>` — list with cursor pagination.
    - `POST /v1/kb/documents/{id}/versions` — new version (same permission as create).
    - `PATCH /v1/kb/documents/{id}` — update metadata (title, permission, allowed_role_codes, owner_subject_id, translation_of); MUST NOT modify markdown_body (use POST versions for body changes).
    - `POST /v1/kb/documents/{id}/archive` — transition status to archived; reason required.

20. **MUST** complete create/get handlers in ≤ 100 ms p95. `documents_perf_test` asserts.

21. **MUST** support idempotent creation via `Idempotency-Key` header (same semantics as TASK-AUTH-002 §1 #6). Repeat POST with same key + same body → return existing document.

22. **MUST** emit OTel span `kb.document.{create,get,list,version,archive,patch}` with attributes: `tenant_id`, `document_id`, `slug`, `category`, `permission`, `outcome` (success | unknown_category | unknown_role | translation_cross_tenant | translation_same_language | body_too_large | permission_denied | not_found).

23. **MUST** emit OTel metrics:
    - `kb_document_create_total{outcome, category, permission}` (counter).
    - `kb_document_version_total{outcome}` (counter).
    - `kb_document_count{tenant_id, category, status}` (gauge).
    - `kb_document_acl_changes_total{from_tier, to_tier}` (counter).
    - `kb_document_archives_total{tenant_id}` (counter).

24. **MUST** ship `current_documents_view` filtering `status != 'archived'` and `active_documents_view` filtering `status = 'active'`. Downstream FRs querying "what is searchable?" use `active_documents_view`; "what is still resolvable for rendering?" uses `current_documents_view`.

25. **MUST** preserve the SQL function `document_version_chain(doc_id UUID) RETURNS SETOF document_versions` returning all versions ordered by `version_number ASC`. Used by TASK-KB-002's "history" UI and by Q&A retrieval that may cite an older version.

26. **MUST** ensure `version_number` is monotonic per document: a `BEFORE INSERT` trigger sets `NEW.version_number = COALESCE(MAX(version_number) FROM document_versions WHERE document_id = NEW.document_id, 0) + 1`. The first version is always 1; gaps are impossible.

27. **MUST** ship a `frontmatter_round_trip_test`: parse → emit → parse again produces the same struct. Catches any drift in the YAML round-tripper that would corrupt frontmatter on save.

---

## §2 — Why this design (rationale for humans)

**Why immutable versions instead of UPDATE-in-place (DEC-243, DEC-247)?** Every AI answer that cites a KB doc cites a *version* — "according to runbook X version 7, the remediation is…". If the doc is silently edited, the citation becomes false. Immutable versions preserve the citation forever; the audit chain (memory) records every save. The cost is one new row per save (small); the benefit is "Q&A answers are always defensibly grounded."

**Why closed category enum (DEC-240)?** Categories drive downstream behaviour: TASK-KB-008's runbook router filters `category='runbook'`; TASK-KB-007's "Ask this page" weights by category; OBS auto-triage prefers runbook + reference. Free-form categories would mean each consumer needs its own normalisation (drift); the closed 5-value enum is the contract. Adding a 6th is an ADR — forces consideration of cross-consumer impact.

**Why closed permission tier at 3 levels (DEC-241)?** Three tiers cover the natural permission shapes: `public` (Trust Center pages opt-in for external visibility), `org_only` (any subject in the tenant), `role_restricted` (specific TASK-AUTH-101 roles). A 4th tier (e.g. `per-subject`) would be ABAC creep — the closed enum prevents the slide.

**Why per-tenant per-language slug uniqueness (DEC-244)?** Same slug in `vi` + `en` is the natural translation pair indicator — `/policy/leave-types` exists in both languages with the same slug, and the `translation_of` FK confirms the pairing. Uniqueness scoped to tenant + language allows both rows to coexist. Without the language scope, the second translation would conflict with the first.

**Why translation_of crosses languages (DEC-245)?** A translation is by definition a cross-language pair. Same-language self-link makes no semantic sense; cross-tenant link would leak content via the FK. The trigger enforces both. The implementation cost is one trigger; the benefit is the translation_of FK is always meaningful.

**Why frontmatter required keys at 4 (`title`, `category`, `language`, `permission`)?** These are the four columns that the schema also stores; the frontmatter is a YAML view of the same data so editors can author both prose + metadata in one file. Required-key validation at the API boundary catches typos (`titlle:`) before they hit storage. Optional keys cover author intent (tags, owner) but don't drive schema columns.

**Why `applicability_tags` only for runbooks (§1 #12)?** Runbooks (TASK-KB-008) need provider/region/severity tags so the OBS auto-router can filter. Other categories don't need them; allowing them everywhere would invite operators to put them on how-tos (unused but noisy). Schema-level optional + category-conditional enforcement keeps the surface focused.

**Why `body_sha256` server-side computed (§1 #13)?** Two reasons. (1) TASK-KB-002's render cache uses this as the cache key; trusting client-supplied hash would let a misbehaving client invalidate other people's caches. (2) Ingestion replay (TASK-KB-005's BGE-M3 path) hashes the body to detect "is this version actually different from last embedded?". The server is the only trusted hasher.

**Why identical body still creates a new version (§1 #13)?** Operators may "re-publish" a doc to bump the timestamp (e.g. confirming a policy is still current). The cost of an extra row is trivial; preventing it would force operators to make trivial edits (whitespace change) just to trigger the version. The change_summary field captures the WHY.

**Why ACL change is a sev-2 memory row (§1 #16, DEC-249)?** Permission tier transitions affect data exposure. Moving a doc from `org_only` to `public` is a Trust Center publication event — operator should review. Moving from `role_restricted` to `org_only` widens visibility — same. Sev-2 emission ensures TASK-OBS-007 surfaces ACL changes in operator digests. Routine creates/edits are sev-3 (informational only).

**Why `status='draft' | 'active' | 'archived'` and not boolean (§1 #6)?** Draft is the create-state when the doc isn't ready to be findable (TASK-KB-004 search excludes drafts). Archived is "no longer current but rendering preserved" — old policies referenced by historical citations stay accessible. Boolean active/inactive collapses these to one bit and loses the draft/archived distinction.

**Why archived docs still render (DEC-251)?** Citation integrity. A KB doc cited by a Q&A answer 6 months ago must still render today even if archived — the answer's citation is fixed at that version. Search defaults to active-only; rendering accepts any status.

**Why `current_version_id` FK is DEFERRABLE INITIALLY IMMEDIATE (§1 #18)?** Atomic save: INSERT version + UPDATE document pointer in one transaction. The document row's FK targets a row that's only fully visible after the version INSERT commits within the transaction. Standard immediate FK would reject. DEFERRABLE INITIALLY IMMEDIATE evaluates at statement end — by then both rows exist.

**Why `version_number` monotonic via trigger (§1 #26)?** Application-layer numbering races: two concurrent saves could both compute `MAX + 1` simultaneously and produce duplicates. The trigger runs at INSERT time inside the same transaction; serialisable isolation ensures correctness even under contention.

**Why frontmatter as TEXT alongside columns (§1 #2)?** Editors want WYSIWYG markdown + YAML editing; the frontmatter is the editable representation. Columns are the queryable normalised form. Both are kept; a save updates both atomically. The cost is duplication; the benefit is "search is fast (column queries) AND editing preserves the YAML the author wrote."

**Why role_restricted validates against TASK-AUTH-101 enum (§1 #11, DEC-250)?** Free-form role strings drift; an `"allowed_role_codes": ["devops"]` row that AUTH doesn't recognise is silent over-permission. Validating against the AUTH closed enum at write time catches typos at the boundary.

**Why archive requires a reason (§1 #19)?** Archiving is consequential — the doc disappears from search; readers who relied on it discover the gap later. Requiring a reason captures the WHY (superseded by doc-X, no longer applicable, decision reversed). The memory audit row preserves it permanently.

**Why body bounds 1–500,000 chars (§1 #2)?** Below 1 = empty doc (caller should DELETE the row, not save an empty version). Above 500K = pathological copy-paste from a foreign source; TASK-KB-002's renderer would OOM on >MB markdown. 500K is ~100 pages of dense prose — well beyond any reasonable single doc.

**Why no slug-rename handler (§1 #19)?** Slug is the URL identity; renaming would break every external link, every memory citation. The operator wanting to rename creates a new doc + redirect (slice 2+). Slug is effectively immutable post-create.

**Why PATCH excludes markdown_body (§1 #19)?** Body changes go through versions, not PATCH. Allowing PATCH to mutate body would either silently create a new version (confusing) or UPDATE the version row (violates immutability). Separating endpoints clarifies semantics.

---

## §3 — API contract

### 3.1 — Migration 0001 — documents + versions

```sql
-- services/kb/migrations/0001_documents.sql

BEGIN;

CREATE TYPE document_category   AS ENUM ('how_to', 'reference', 'decision_log', 'policy', 'runbook');
CREATE TYPE document_permission AS ENUM ('public', 'org_only', 'role_restricted');
CREATE TYPE language_code       AS ENUM ('vi', 'en');
CREATE TYPE document_status     AS ENUM ('draft', 'active', 'archived');

CREATE TABLE document_versions (
    id                     UUID         PRIMARY KEY,
    document_id            UUID         NOT NULL,  -- FK added after documents created
    tenant_id              UUID         NOT NULL,
    version_number         INT          NOT NULL,
    markdown_body          TEXT         NOT NULL CHECK (length(markdown_body) BETWEEN 1 AND 500000),
    frontmatter_yaml       TEXT         NOT NULL,
    body_sha256            CHAR(64)     NOT NULL CHECK (body_sha256 ~ '^[0-9a-f]{64}$'),
    change_summary         TEXT         CHECK (change_summary IS NULL OR length(change_summary) BETWEEN 1 AND 500),
    created_at             TIMESTAMPTZ  NOT NULL DEFAULT now(),
    created_by_subject_id  UUID         NOT NULL REFERENCES auth.subjects(id) ON DELETE RESTRICT
);

CREATE TABLE documents (
    id                     UUID         PRIMARY KEY,
    tenant_id              UUID         NOT NULL,
    slug                   TEXT         NOT NULL CHECK (slug ~ '^[a-z0-9][a-z0-9-/]*[a-z0-9]$' AND length(slug) BETWEEN 2 AND 200),
    language               language_code NOT NULL,
    title                  TEXT         NOT NULL CHECK (length(title) BETWEEN 1 AND 200),
    category               document_category NOT NULL,
    permission             document_permission NOT NULL,
    allowed_role_codes     TEXT[]       NOT NULL DEFAULT '{}'::TEXT[],
    current_version_id     UUID         NOT NULL REFERENCES document_versions(id) DEFERRABLE INITIALLY IMMEDIATE,
    translation_of         UUID         REFERENCES documents(id) DEFERRABLE INITIALLY IMMEDIATE,
    status                 document_status NOT NULL DEFAULT 'draft',
    owner_subject_id       UUID         REFERENCES auth.subjects(id),
    created_at             TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at             TIMESTAMPTZ  NOT NULL DEFAULT now()
);

-- Now add the document FK to versions (was deferred because of circular dep)
ALTER TABLE document_versions ADD CONSTRAINT document_versions_doc_fk
    FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE RESTRICT;

-- Per-tenant per-language slug uniqueness
CREATE UNIQUE INDEX uniq_doc_slug ON documents (tenant_id, language, slug);
CREATE INDEX documents_tenant_category_idx ON documents (tenant_id, category, status);
CREATE INDEX documents_tenant_status_idx ON documents (tenant_id, status);
CREATE INDEX document_versions_doc_version_idx ON document_versions (document_id, version_number DESC);

-- RLS (per task-audit skill rule 13)
ALTER TABLE documents          ENABLE ROW LEVEL SECURITY;
ALTER TABLE document_versions  ENABLE ROW LEVEL SECURITY;

CREATE POLICY documents_tenant_isolation ON documents
    USING (tenant_id = current_setting('auth.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

CREATE POLICY document_versions_tenant_isolation ON document_versions
    USING (tenant_id = current_setting('auth.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

-- Append-only enforcement on versions (DEC-247)
REVOKE UPDATE, DELETE ON document_versions FROM cyberos_app;

-- translation_of cross-language + same-tenant enforcement (DEC-245)
CREATE OR REPLACE FUNCTION enforce_translation_of() RETURNS TRIGGER AS $$
DECLARE target RECORD;
BEGIN
    IF NEW.translation_of IS NULL THEN RETURN NEW; END IF;
    IF NEW.translation_of = NEW.id THEN
        RAISE EXCEPTION 'translation_self_link' USING ERRCODE = 'P0020';
    END IF;
    SELECT tenant_id, language INTO target FROM documents WHERE id = NEW.translation_of;
    IF NOT FOUND THEN
        RAISE EXCEPTION 'translation_target_missing' USING ERRCODE = 'P0021';
    END IF;
    IF target.tenant_id != NEW.tenant_id THEN
        RAISE EXCEPTION 'translation_cross_tenant' USING ERRCODE = 'P0022';
    END IF;
    IF target.language = NEW.language THEN
        RAISE EXCEPTION 'translation_must_cross_language' USING ERRCODE = 'P0023';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_documents_translation_of BEFORE INSERT OR UPDATE ON documents
    FOR EACH ROW EXECUTE FUNCTION enforce_translation_of();

-- role_restricted requires non-empty allowed_role_codes (DEC-250)
CREATE OR REPLACE FUNCTION enforce_role_restricted_codes() RETURNS TRIGGER AS $$
BEGIN
    IF NEW.permission = 'role_restricted' AND COALESCE(array_length(NEW.allowed_role_codes, 1), 0) = 0 THEN
        RAISE EXCEPTION 'role_restricted_requires_roles' USING ERRCODE = 'P0024';
    END IF;
    -- Handler additionally validates each code against TASK-AUTH-101 enum.
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_documents_role_restricted BEFORE INSERT OR UPDATE ON documents
    FOR EACH ROW EXECUTE FUNCTION enforce_role_restricted_codes();

-- version_number monotonic per document (§1 #26)
CREATE OR REPLACE FUNCTION assign_version_number() RETURNS TRIGGER AS $$
BEGIN
    IF NEW.version_number IS NULL OR NEW.version_number = 0 THEN
        NEW.version_number := COALESCE(
            (SELECT MAX(version_number) FROM document_versions WHERE document_id = NEW.document_id),
            0
        ) + 1;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_document_versions_assign_number BEFORE INSERT ON document_versions
    FOR EACH ROW EXECUTE FUNCTION assign_version_number();

-- documents.updated_at touched on update
CREATE OR REPLACE FUNCTION touch_documents_updated_at() RETURNS TRIGGER AS $$
BEGIN NEW.updated_at := now(); RETURN NEW; END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_documents_updated_at BEFORE UPDATE ON documents
    FOR EACH ROW EXECUTE FUNCTION touch_documents_updated_at();

COMMIT;
```

### 3.2 — Migration 0002 — views + chain walker

```sql
-- services/kb/migrations/0002_document_views.sql

BEGIN;

-- "Still resolvable for rendering" — excludes only fully-deleted (which don't exist; archived stays)
CREATE VIEW current_documents_view AS
    SELECT * FROM documents WHERE status != 'archived';

-- "What is searchable + active" — excludes draft + archived
CREATE VIEW active_documents_view AS
    SELECT * FROM documents WHERE status = 'active';

-- Walk all versions of a document in order (oldest first)
CREATE OR REPLACE FUNCTION document_version_chain(p_doc_id UUID)
RETURNS SETOF document_versions AS $$
    SELECT * FROM document_versions WHERE document_id = p_doc_id ORDER BY version_number ASC
$$ LANGUAGE sql STABLE;

COMMIT;
```

### 3.3 — Rust types

```rust
// services/kb/src/types.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Type, Serialize, Deserialize)]
#[sqlx(type_name = "document_category", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum DocumentCategory { HowTo, Reference, DecisionLog, Policy, Runbook }

impl DocumentCategory {
    pub const ALL: &'static [DocumentCategory] = &[
        DocumentCategory::HowTo, DocumentCategory::Reference, DocumentCategory::DecisionLog,
        DocumentCategory::Policy, DocumentCategory::Runbook,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Type, Serialize, Deserialize)]
#[sqlx(type_name = "document_permission", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum DocumentPermission { Public, OrgOnly, RoleRestricted }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Type, Serialize, Deserialize)]
#[sqlx(type_name = "language_code", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum LanguageCode { Vi, En }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Type, Serialize, Deserialize)]
#[sqlx(type_name = "document_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum DocumentStatus { Draft, Active, Archived }

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Document {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub slug: String,
    pub language: LanguageCode,
    pub title: String,
    pub category: DocumentCategory,
    pub permission: DocumentPermission,
    pub allowed_role_codes: Vec<String>,
    pub current_version_id: Uuid,
    pub translation_of: Option<Uuid>,
    pub status: DocumentStatus,
    pub owner_subject_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct DocumentVersion {
    pub id: Uuid,
    pub document_id: Uuid,
    pub tenant_id: Uuid,
    pub version_number: i32,
    pub markdown_body: String,
    pub frontmatter_yaml: String,
    pub body_sha256: String,
    pub change_summary: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by_subject_id: Uuid,
}
```

### 3.4 — Frontmatter validator

```rust
// services/kb/src/frontmatter.rs
use serde::{Deserialize, Serialize};
use serde_yaml;
use crate::types::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct Frontmatter {
    pub title: String,
    pub category: DocumentCategory,
    pub language: LanguageCode,
    pub permission: DocumentPermission,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub owner_subject_id: Option<uuid::Uuid>,
    #[serde(default)]
    pub translation_of: Option<String>,        // slug; resolved to UUID at INSERT
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub applicability_tags: Option<ApplicabilityTags>,
    // Unknown keys: caught by serde(deny_unknown_fields)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApplicabilityTags {
    pub provider: Option<String>,             // "aws" | "gcp" | "azure" | ...
    pub region: Option<String>,
    pub severity: Option<String>,             // "p0" | "p1" | "p2" | "p3"
}

#[derive(Debug, thiserror::Error)]
pub enum FrontmatterError {
    #[error("yaml_parse_error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("unknown_frontmatter_key: {0}")]
    UnknownKey(String),
    #[error("title_too_long: {0}")]
    TitleTooLong(usize),
    #[error("summary_too_long: {0}")]
    SummaryTooLong(usize),
    #[error("too_many_tags: {0}")]
    TooManyTags(usize),
    #[error("tag_invalid: {0}")]
    TagInvalid(String),
    #[error("applicability_tags_only_runbook")]
    ApplicabilityTagsOnlyRunbook,
}

pub fn parse_and_validate(yaml: &str) -> Result<Frontmatter, FrontmatterError> {
    let fm: Frontmatter = serde_yaml::from_str(yaml)?;
    if fm.title.len() > 200 { return Err(FrontmatterError::TitleTooLong(fm.title.len())); }
    if let Some(s) = &fm.summary { if s.len() > 200 { return Err(FrontmatterError::SummaryTooLong(s.len())); } }
    if fm.tags.len() > 20 { return Err(FrontmatterError::TooManyTags(fm.tags.len())); }
    for t in &fm.tags {
        if t.is_empty() || t.len() > 32 { return Err(FrontmatterError::TagInvalid(t.clone())); }
    }
    if fm.applicability_tags.is_some() && fm.category != DocumentCategory::Runbook {
        return Err(FrontmatterError::ApplicabilityTagsOnlyRunbook);
    }
    Ok(fm)
}
```

### 3.5 — REST handler excerpt

```rust
// services/kb/src/handlers/documents.rs
use axum::{Json, extract::{Path, State}, http::StatusCode};
use sha2::{Digest, Sha256};
use crate::types::*;
use crate::frontmatter::parse_and_validate;
use crate::audit::doc_events;
use cyberos_auth::rbac::{Resource, Action, Role};

#[derive(Deserialize)]
pub struct CreateDocumentRequest {
    pub slug: String,
    pub language: LanguageCode,
    pub title: String,
    pub category: DocumentCategory,
    pub permission: DocumentPermission,
    pub allowed_role_codes: Vec<String>,
    pub markdown_body: String,
    pub frontmatter_yaml: String,
    pub owner_subject_id: Option<Uuid>,
    pub translation_of_slug: Option<String>,
}

pub async fn create_document(
    State(state): State<AppState>,
    claims: Claims,
    Json(req): Json<CreateDocumentRequest>,
) -> Result<(StatusCode, Json<Document>), ApiError> {
    state.matrix.snapshot().require_permission(&claims.roles(), Resource::KbDocument, Action::Write)?;
    let fm = parse_and_validate(&req.frontmatter_yaml)?;

    // Validate allowed_role_codes against TASK-AUTH-101 closed enum.
    for code in &req.allowed_role_codes {
        if code.parse::<Role>().is_err() {
            return Err(ApiError::UnknownRole(code.clone()));
        }
    }
    if req.permission == DocumentPermission::RoleRestricted && req.allowed_role_codes.is_empty() {
        return Err(ApiError::RoleRestrictedRequiresRoles);
    }

    let doc_id = Uuid::new_v4();
    let version_id = Uuid::new_v4();
    let body_sha = hex::encode(Sha256::digest(req.markdown_body.as_bytes()));

    // Resolve translation_of slug → document_id (if specified)
    let translation_of_id = if let Some(slug) = &req.translation_of_slug {
        Some(state.repo.find_by_slug_other_lang(claims.tenant_id(), slug, req.language).await?)
    } else { None };

    let mut tx = state.db.begin().await?;
    sqlx::query(r#"
        INSERT INTO document_versions (id, document_id, tenant_id, version_number,
            markdown_body, frontmatter_yaml, body_sha256, created_by_subject_id)
        VALUES ($1, $2, $3, NULL, $4, $5, $6, $7)
    "#)
    .bind(version_id).bind(doc_id).bind(claims.tenant_id())
    .bind(&req.markdown_body).bind(&req.frontmatter_yaml).bind(&body_sha)
    .bind(claims.subject_id())
    .execute(&mut *tx).await?;

    let doc = sqlx::query_as!(Document, r#"
        INSERT INTO documents (id, tenant_id, slug, language, title, category, permission,
            allowed_role_codes, current_version_id, translation_of, status, owner_subject_id)
        VALUES ($1, $2, $3, $4::language_code, $5, $6::document_category, $7::document_permission,
                $8, $9, $10, 'draft'::document_status, $11)
        RETURNING *
    "#, /* ... */).fetch_one(&mut *tx).await?;

    doc_events::emit_document_created(&mut tx, &doc, &body_sha, claims.subject_id()).await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(doc)))
}
```

---

## §4 — Acceptance criteria

1. **Category enum closed at 5** — `DocumentCategory::ALL.len() == 5`; Postgres enum has exactly 5 labels.
2. **Permission tier closed at 3** — same shape.
3. **Language closed at 2** — same shape.
4. **Status closed at 3** — same shape.
5. **RLS isolates by tenant** — query as tenant-A returns 0 documents of tenant-B.
6. **Create document happy path** — 201 with Document JSON; 1 row in documents + 1 in document_versions; `kb.document_created` memory row emitted.
7. **Slug uniqueness per (tenant, language)** — second create with same (tenant, language, slug) → 409 `slug_taken`.
8. **Same slug different language allowed** — create with `language=vi` then `language=en` same slug → both succeed.
9. **translation_of self-link rejected** — set `translation_of = self.id` → 400 `translation_self_link`.
10. **translation_of same-language rejected** — link to another doc with same language → 400 `translation_must_cross_language`.
11. **translation_of cross-tenant rejected** — link to doc in different tenant → 400 `translation_cross_tenant`.
12. **role_restricted with empty roles** → 400 `role_restricted_requires_roles`.
13. **role_restricted with unknown role** → 400 `unknown_role: <code>`.
14. **UPDATE document_versions blocked** — `UPDATE document_versions SET markdown_body = 'x' WHERE id = $1` as cyberos_app → permission denied.
15. **DELETE document_versions blocked** — same.
16. **New version creates monotonic version_number** — first version = 1; second = 2; concurrent saves both succeed with sequential numbers.
17. **current_version_id points at latest** — after 3 versions, `documents.current_version_id` = id of version 3.
18. **body_sha256 server-computed** — request with client-supplied hash field is ignored; server computes from body.
19. **Markdown body < 1 char** → 400 `body_too_short` (DB CHECK fires).
20. **Markdown body > 500_000 chars** → 400 `body_too_large` (DB CHECK fires).
21. **Frontmatter unknown key rejected** — YAML with `xyz: abc` → 400 `unknown_frontmatter_key`.
22. **Frontmatter applicability_tags on non-runbook rejected** — category=policy with applicability_tags → 400.
23. **Archive emits memory row** — POST /archive → status=archived; `kb.document_archived` row with reason.
24. **ACL change emits sev-2 memory row** — PATCH permission tier → `kb.document_acl_changed` row.
25. **active_documents_view excludes draft + archived** — query returns only status='active' docs.
26. **document_version_chain returns oldest-first** — chain of 3 versions → 3 rows in order [1, 2, 3].
27. **Perf budget < 100 ms p95** — `documents_perf_test` 1000 iterations.
28. **Idempotent create** — same Idempotency-Key + same body → same document.
29. **OTel span `kb.document.create` emitted** — with `outcome=success`.
30. **Counter `kb_document_create_total{outcome=success, category=runbook, permission=org_only}` increments** — per create.

---

## §5 — Verification

```rust
// services/kb/tests/category_enum_closed_test.rs
#[test]
fn category_enum_has_exactly_5_values() {
    assert_eq!(DocumentCategory::ALL.len(), 5);
}

#[sqlx::test]
async fn pg_category_enum_has_exactly_5_labels(pool: sqlx::PgPool) {
    let labels: Vec<String> = sqlx::query_scalar(
        "SELECT unnest(enum_range(NULL::document_category))::text ORDER BY 1"
    ).fetch_all(&pool).await.unwrap();
    assert_eq!(labels, vec!["decision_log", "how_to", "policy", "reference", "runbook"]);
}
```

```rust
// services/kb/tests/version_append_only_test.rs
#[sqlx::test]
async fn update_blocked(pool: sqlx::PgPool) {
    set_role_app(&pool).await;
    let id = seed_version(&pool).await;
    let err = sqlx::query("UPDATE document_versions SET markdown_body = 'x' WHERE id = $1")
        .bind(id).execute(&pool).await.unwrap_err();
    assert!(format!("{err}").contains("permission denied"));
}
```

```rust
// services/kb/tests/translation_of_test.rs
#[sqlx::test]
async fn same_language_link_rejected(ctx: TestCtx) {
    let a = ctx.create_doc(LanguageCode::Vi, "/policy/leave").await;
    let b = ctx.create_doc(LanguageCode::Vi, "/policy/leave2").await;
    let err = ctx.patch_translation_of(b, a).await.unwrap_err();
    assert!(format!("{err}").contains("translation_must_cross_language"));
}

#[sqlx::test]
async fn cross_language_pair_allowed(ctx: TestCtx) {
    let vi = ctx.create_doc(LanguageCode::Vi, "/policy/leave").await;
    let en = ctx.create_doc(LanguageCode::En, "/policy/leave").await;
    ctx.patch_translation_of(en, vi).await.unwrap();
}
```

```rust
// services/kb/tests/version_chain_test.rs
#[sqlx::test]
async fn version_numbers_monotonic(ctx: TestCtx) {
    let doc = ctx.create_doc(LanguageCode::Vi, "/x").await;
    let v2 = ctx.new_version(doc.id, "body v2").await;
    let v3 = ctx.new_version(doc.id, "body v3").await;
    assert_eq!(v2.version_number, 2);
    assert_eq!(v3.version_number, 3);
    let chain: Vec<i32> = ctx.version_chain(doc.id).await;
    assert_eq!(chain, vec![1, 2, 3]);
    let current: Uuid = ctx.get_current_version_id(doc.id).await;
    assert_eq!(current, v3.id);
}
```

---

## §6 — Implementation skeleton

(API contract above is the skeleton. The 4 memory row builders in `audit/doc_events.rs` follow the canonical pattern.)

---

## §7 — Dependencies

**Upstream:**
- **TASK-AUTH-003** — RLS enforcement.
- **TASK-AUTH-101** — RBAC catalogue (`Resource::KbDocument + Action::Read/Write/Admin`) + role validation for `allowed_role_codes`.

**Downstream (7 placeholders):**
- **TASK-KB-002** — server-side renderer (markdown → HTML); consumes `markdown_body` + `body_sha256` for cache.
- **TASK-KB-003** — three permission tiers + share-link tokens; consumes `permission` + `allowed_role_codes`.
- **TASK-KB-004** — FTS5 + PGroonga lexical search; indexes `active_documents_view`.
- **TASK-KB-005** — BGE-M3 semantic search via memory Layer 2 ingest; consumes versioned bodies.
- **TASK-KB-007** — "Ask this page" Q&A with span citations; consumes documents + linked translation_of.
- **TASK-KB-008** — runbook category with applicability tags for OBS triage.
- **TASK-KB-009** — dual-language `translation_of` link + locale-aware reader display.

**Cross-module:**
- **TASK-AI-003** — memory audit bridge; receives `kb.document_created`, `kb.document_versioned`, `kb.document_acl_changed`, `kb.document_archived`.
- **TASK-MEMORY-111** — PII scrubbing for body before memory Layer 2 ingest (TASK-KB-005 path).

---

## §8 — Example payloads

### 8.1 — POST /v1/kb/documents request

```json
{
  "slug": "policy/leave-types",
  "language": "en",
  "title": "Leave Types Policy",
  "category": "policy",
  "permission": "org_only",
  "allowed_role_codes": [],
  "markdown_body": "# Leave Types\n\nThis tenant offers 8 leave types per Decree 145/2020 ...",
  "frontmatter_yaml": "title: Leave Types Policy\ncategory: policy\nlanguage: en\npermission: org_only\ntags: [hr, leave, decree-145]\nowner_subject_id: 9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d\n",
  "owner_subject_id": "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d"
}
```

### 8.2 — 201 CREATED response

```json
{
  "id": "01HG7V8B0K8M4Z8Z8M8M8M8M8M",
  "tenant_id": "5e8f1d2a-...",
  "slug": "policy/leave-types",
  "language": "en",
  "title": "Leave Types Policy",
  "category": "policy",
  "permission": "org_only",
  "allowed_role_codes": [],
  "current_version_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8N",
  "translation_of": null,
  "status": "draft",
  "owner_subject_id": "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d",
  "created_at": "2026-05-16T10:00:00Z",
  "updated_at": "2026-05-16T10:00:00Z"
}
```

### 8.3 — kb.document_created memory row

```json
{
  "kind": "kb.document_created",
  "tenant_id": "5e8f1d2a-...",
  "document_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8M",
  "slug": "policy/leave-types",
  "language": "en",
  "category": "policy",
  "permission": "org_only",
  "owner_subject_id_hash16": "9b1deb4d3b7d4bad",
  "created_by_subject_id_hash16": "8a7c8c8012344567",
  "body_sha256": "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
  "version_number": 1,
  "ts_ns": 1747920731000000000
}
```

### 8.4 — kb.document_acl_changed memory row (sev-2)

```json
{
  "kind": "kb.document_acl_changed",
  "severity": "sev-2",
  "tenant_id": "5e8f1d2a-...",
  "document_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8M",
  "old_permission": "org_only",
  "new_permission": "public",
  "old_allowed_role_codes": [],
  "new_allowed_role_codes": [],
  "changed_by_subject_id_hash16": "8a7c8c8012344567",
  "ts_ns": 1747920731000000000
}
```

### 8.5 — kb.document_archived memory row

```json
{
  "kind": "kb.document_archived",
  "tenant_id": "5e8f1d2a-...",
  "document_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8M",
  "archived_by_subject_id_hash16": "8a7c8c8012344567",
  "reason": "Superseded by /policy/leave-types-v2 after 2026 reorganisation",
  "ts_ns": 1747920731000000000
}
```

---

## §9 — Open questions

Deferred:
- **Notion import (preserve links + categories)** — FR-KB-2xx (slice 2+).
- **Slug-rename with redirect** — slice 2+; slice 1 treats slug as effectively immutable.
- **Trust Center opt-in flag at tenant level** — FR-KB-2xx.
- **Per-page or per-tree markdown export** — FR-KB-2xx.
- **Bulk frontmatter migration** — slice 2+ tool.
- **Dead-link detection nightly job** — FR-KB-2xx.
- **Comments / annotations on docs** — out of scope; PROJ handles comments.

All other questions resolved.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| UPDATE on document_versions | SQL grant | Permission denied | None — designed |
| DELETE on document_versions | SQL grant | Permission denied | None — designed |
| Duplicate slug per (tenant, language) | UNIQUE index | 409 slug_taken | Use different slug |
| translation_of self-link | trigger `translation_self_link` | 400 | Fix payload |
| translation_of same-language | trigger `translation_must_cross_language` | 400 | Fix payload |
| translation_of cross-tenant | trigger `translation_cross_tenant` | 400 | Fix payload |
| role_restricted with empty roles | trigger `role_restricted_requires_roles` | 400 | Add at least one role |
| Unknown role in allowed_role_codes | handler validates against TASK-AUTH-101 enum | 400 unknown_role | Use valid role |
| Markdown body < 1 char | DB CHECK | 400 body_too_short | Add content |
| Markdown body > 500_000 chars | DB CHECK | 400 body_too_large | Split into multiple docs |
| Frontmatter unknown key | serde(deny_unknown_fields) | 400 unknown_frontmatter_key | Remove typo |
| Frontmatter title > 200 chars | validator | 400 title_too_long | Shorten |
| Frontmatter tags > 20 | validator | 400 too_many_tags | Reduce |
| Frontmatter tag length out of bounds | validator | 400 tag_invalid | Fix |
| applicability_tags on non-runbook | validator | 400 applicability_tags_only_runbook | Remove or change category |
| body_sha256 client-supplied mismatch | server overrides | None | Designed |
| Cross-tenant version FK | RLS denies SELECT | 0 rows | None — designed |
| version_number collision under concurrent saves | trigger computes serially within tx | None — atomic | Designed |
| Current_version_id points at deleted version | FK ON DELETE RESTRICT prevents | DELETE fails | None — designed |
| Archive without reason | handler required field | 400 missing_reason | Provide |
| ACL change without sev-2 emission | audit test asserts | CI fails | Fix builder |
| Status transition draft→archived without going active | FSM allows (no intermediate state required) | None | Designed |
| Frontmatter YAML malformed | serde_yaml::Error | 400 yaml_parse_error | Fix YAML |
| Slug regex violation | DB CHECK (`^[a-z0-9][a-z0-9-/]*[a-z0-9]$`) | INSERT fails | Use kebab/path-friendly slug |
| Title > 200 chars | DB CHECK | INSERT fails | Shorten |
| Slug too short (< 2 chars) | DB CHECK | INSERT fails | Use longer slug |
| change_summary > 500 chars | DB CHECK | INSERT fails | Shorten |
| Concurrent translation_of pair creation (A→B + B→A) | Each insert sees a stable target | Both succeed — pair is symmetric in intent but FK is one-directional | None — operator may set both directions deliberately |
| PII in markdown_body not scrubbed for memory ingest | TASK-MEMORY-111 in TASK-KB-005's path | Out of scope here | TASK-KB-005 |
| Document deleted while versions exist | FK ON DELETE RESTRICT | DELETE fails | Archive instead |
| Subject deleted while documents reference owner_subject_id | FK on owner without RESTRICT; SET NULL acceptable | None | Designed (owner is optional) |
| Subject deleted while versions reference created_by | FK RESTRICT | DELETE fails | Designed |
| current_documents_view performance on 100K docs | Standard tenant index supports | OK | None |

---

## §11 — Implementation notes

- **Immutable versions are the design contract** — every AI citation references a version_id; mutating would invalidate citations silently.
- **Two enums for slug pairs**: per-tenant per-language uniqueness allows `/policy/leave-types` in `vi` AND `en` simultaneously; the natural translation pair indicator.
- **frontmatter as both YAML + columns**: YAML for the author (editor experience), columns for queries (TASK-KB-004 search filters). Both updated atomically.
- **`serde(deny_unknown_fields)` on Frontmatter struct** — catches typos at the API boundary; downstream consumers don't have to defend.
- **`applicability_tags` validator runs ONLY when category == Runbook** — avoids forcing every doc to specify a tag they don't need.
- **body_sha256 server-computed via `sha2::Sha256`** — deterministic; hex-encoded; matches the format other modules use.
- **`document_version_chain` is a SQL function** — usable from views and from app code; same semantics either way.
- **`translation_of` is a self-FK on documents** — not on the version. Translations are paired at the document level; both translations have their own version chains.
- **Cross-language pairing is operator-confirmed** — TASK-KB-009 (slice 4+) may auto-suggest pairings; slice 1 is manual.
- **Slug regex `^[a-z0-9][a-z0-9-/]*[a-z0-9]$`** — allows path-like slugs (`policy/leave-types`) while preventing trailing/leading slashes or hyphens.
- **archive vs delete**: archive preserves rendering + citations; delete would require cleanup of every citation (which we can't enumerate). Archive is the only way to "remove" a doc.
- **`updated_at` touched by trigger** — single source of truth for staleness; TASK-KB-005's ingestion checks this for re-embed.
- **`change_summary` 1–500 chars** — short enough to discourage prose, long enough for "Updated section 3 to reflect Decree 145 amendment".
- **Permission validation at retrieval time, not after** — TASK-KB-003 enforces; this FR provides the data. Acceptable: this FR's GET handler also enforces (defence in depth).
- **`allowed_role_codes TEXT[]` not enum array** — Postgres enums don't compose well into arrays; storing as text and validating at boundary is simpler.
- **`status='draft'` is the create default** — operators must explicitly publish (slice 2 publish handler); prevents accidental exposure.
- **`current_documents_view` excludes only archived** — drafts ARE current (for the author who's editing); they're just not searchable (active_documents_view excludes them).
- **`tenant_id` denormalised on `document_versions`** — RLS policy can filter without join.
- **`current_version_id` FK is `DEFERRABLE INITIALLY IMMEDIATE`** — needed for atomic INSERT-version-then-UPDATE-pointer pattern in one transaction.
- **`document_versions` PK is UUID not BIGSERIAL** — UUIDs allow client-generated ids for idempotency; BIGSERIAL would create races on retry.
- **Markdown size cap 500K = ~100 pages** — well beyond practical single-doc limit; protects renderer from OOM.
- **owner_subject_id is OPTIONAL** — some docs are tenant-owned (no specific human owner); allowing NULL avoids forcing a fake owner.
- **`kb.document_acl_changed` is sev-2** — operator notification; routine creates are sev-3 (informational only).

---

*End of TASK-KB-001.*
