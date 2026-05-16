# Feature-Request Authoring Workflow

**Owner:** Stephen Cheng (CEO)
**Status:** v1.0.0 — adopted 2026-05-14 after the nuclear FR strip
**Source skill:** [`skill/skills/cuo/cpo/fr-author/SKILL.md`](../skill/skills/cuo/cpo/fr-author/SKILL.md)
**Companion skill:** [`skill/skills/cuo/cpo/fr-audit/SKILL.md`](../skill/skills/cuo/cpo/fr-audit/SKILL.md)
**Use when:** authoring a new feature request before any code lands.

This document is the canonical playbook. Every FR that ships into CyberOS starts here.

---

## §1 — The mental model

One FR = one atomic, testable, normative requirement. Smaller is better.

- **Atomic** — covers exactly one capability. If you can't test it independently with a single integration test, it's two FRs.
- **Testable** — the FR has a verification method (unit / integration / chaos / manual) and an acceptance signal.
- **Normative** — uses BCP-14 keywords (`MUST` / `SHOULD` / `COULD` / `MAY`) and is precise enough that two engineers reading it write the same code.

One FR → one task → (eventually) one PR. Three artefacts on the BRAIN audit chain per FR: the `feature_request@1` markdown, the `audit_response@1` from fr-audit, and the `task@1` you create when you accept it.

---

## §2 — File layout

```
docs/
└── feature-requests/                       ← single source of truth for live FRs
    ├── MANIFEST.json                       ← state file (resumable batches; managed by fr-author)
    ├── auth/                               ← one folder per module
    │   ├── FR-AUTH-001-tenant-create.md
    │   ├── FR-AUTH-001-tenant-create.audit.md      ← from fr-audit
    │   ├── FR-AUTH-002-subject-create.md
    │   └── …
    ├── brain/                              ← module already shipped; FRs added retroactively as we re-author
    ├── skill/
    ├── cuo/
    └── …
```

| Convention | Value |
|---|---|
| FR-ID format | `FR-{MOD}-{NNN}` where `{MOD}` is the closed module code from the catalogue (`AUTH`, `AI`, `MCP`, `OBS`, `CHAT`, `BRAIN`, `SKILL`, `CUO`, `EMAIL`, `PROJ`, `TIME`, `CRM`, `KB`, `HR`, `REW`, `LEARN`, `INV`, `ESOP`, `RES`, `OKR`, `DOC`, `PORTAL`, `TEN`) and `{NNN}` is zero-padded three digits, dense within the module (001, 002, 003 — never skip) |
| Filename | `FR-{MOD}-{NNN}-{slug}.md` where slug is kebab-case, ≤ 50 chars |
| Per-module folder | lowercase module code (`auth/`, `ai/`, etc.) |
| Status states | `draft` → `audited` → `accepted` → `building` → `shipped` (or `deferred` / `rejected`) |

The `docs/feature-requests/` path is *new* — it didn't exist before the strip. The first invocation of `fr-author` creates it.

---

## §3 — The two flows

### §3.1 Standalone flow (one-off · recommended for solo authoring)

You're at a terminal. You want to author one FR. Use this:

```bash
cd /Users/stephencheng/Projects/CyberSkill/cyberos
cd skill
cargo run -p cyberos-skill-cli -- run fr-author --executor script
```

The skill enters standalone mode and runs the [STANDALONE_INTERVIEW](../skill/skills/cuo/cpo/fr-author/STANDALONE_INTERVIEW.md) script:

1. **Q1**: *"Which requirements doc(s) should I work from?"*
   Answer with: `docs/prd/PRD.md` (or a narrower file you've drafted for the slice you're spec'ing).
2. **Q2**: *"Where should I write the FR markdowns?"*
   Answer with: `docs/feature-requests/auth/` (substitute your target module).
3. **Q3**: *"And the manifest?"*
   Answer with: `docs/feature-requests/MANIFEST.json` (shared across modules).

Optional override during the interview:
- `batch_size: 1` if you genuinely want one FR at a time (default is 3).
- `chain_to: []` to disable auto-chaining into `fr-audit` (you'll audit manually).

The skill drops an `FR-AUTH-001-tenant-create.md` (or similar) into the folder, emits an audit row to BRAIN, and surfaces the `HUMAN_SUMMARY` to your chat:

```
✅ Batch complete — wrote 1 feature request:
  - FR-AUTH-001 — tenant create · status: draft · <hash>...
📋 BRAIN updated: 1 audit row appended.
📊 Trace: <uuid>
```

### §3.2 Chained flow (batch · 5+ FRs at once)

Same skill, but you let it chain into `fr-audit`. Use this when you're spec'ing a whole module slice.

```bash
cargo run -p cyberos-skill-cli -- run fr-author \
  --executor script \
  --input '{
    "requirements_files": ["docs/prd/PRD.md"],
    "output_dir": "docs/feature-requests/auth/",
    "manifest_path": "docs/feature-requests/MANIFEST.json",
    "batch_size": 5,
    "chain_to": ["cuo/cpo/fr-audit"]
  }'
```

`fr-author` emits 5 FRs in a batch, then immediately invokes `fr-audit` against the batch, which produces `<fr-path>.audit.md` per FR. You review the audit, dispatch `APPROVE` / `REVISE` / `REJECT` per FR, and the next batch resumes.

[PIPELINE.md](../skill/skills/cuo/cpo/fr-author/PIPELINE.md) covers chained-mode details.

---

## §4 — The standard recipe (module slice 1)

Every module's first slice should ship with **5–7 FRs**. Less = too coarse, more = you're trying to do too much in one PR.

The recipe per module:

| Step | What | Where | Output |
|---|---|---|---|
| 1 | Read the module's spec page | `website/docs/modules/<mod>.html` | Mental model |
| 2 | Read the module's RFC | `services/<mod>/RFC.md` (template: `services/auth/RFC.md`) | Slice-1 scope |
| 3 | Write a slice-1 brief | inline in chat or `services/<mod>/SLICE_1_BRIEF.md` | 1-page narrative |
| 4 | Invoke fr-author standalone | per §3.1 | 5–7 FR markdowns in `docs/feature-requests/<mod>/` |
| 5 | Chain into fr-audit | per §3.2 (or invoke fr-audit standalone after) | `*.audit.md` per FR |
| 6 | Review + accept | manual | `status: accepted` in frontmatter |
| 7 | Create one task per FR | TodoWrite or your task tracker | `task@1` rows on BRAIN |
| 8 | Build (one FR per PR) | code | `status: building` then `shipped` |
| 9 | Surface back to docs | re-render `fr-catalog.html` from the live FR folder | catalog page comes back online |

Step 9 is the only step we'll need to *build* — there's no automation today to render the `docs/feature-requests/` tree into the `fr-catalog.html` page. The stub catalog page at `website/docs/reference/fr-catalog.html` will need a small build script (or just a periodic regenerate). Add to task list.

---

## §5 — How FRs surface back to the docs site

The strip emptied the catalog page + the per-module FR sections. As FRs land in `docs/feature-requests/`, they should re-appear on the docs site. The mechanism:

1. **Catalog page** (`website/docs/reference/fr-catalog.html`): generate-on-build from `docs/feature-requests/*/FR-*.md`. Today it's a stub; later we'll add a small build script that walks the FR folder, parses each markdown's YAML frontmatter, and emits the catalog grid.
2. **Per-module FR section** (e.g. `website/docs/modules/auth.html#functional-requirements`): same build step picks `docs/feature-requests/auth/FR-AUTH-*.md` and emits cards. Today these are stubs.
3. **Cross-refs** in NFR catalog + risk register + module pages: today they all say `(FR pending)`. As each FR lands, the relevant `(FR pending)` is replaced manually (or via grep) with the new FR-ID. We'll grow a `tools/fr-link.py` script later that does this automatically given an FR-ID and a context page.

For now: prioritise authoring the FRs themselves. Catalog-rendering can wait until you have ~20 FRs to show.

---

## §6 — Status lifecycle

Frontmatter on every FR markdown:

```yaml
---
id: FR-AUTH-001
title: "tenant create — root-level admin can create a new tenant"
module: AUTH
priority: MUST          # MUST | SHOULD | COULD | MAY
status: draft           # draft → audited → accepted → building → shipped (or deferred/rejected)
verify: T               # T (test) | I (inspection) | A (analysis) | D (demonstration)
phase: P0               # P0 | P1 | P2 | P3 | P4
owner: Stephen Cheng
created: 2026-05-14
shipped: null
slice: 1                # slice-1 of the AUTH module's 5-slice plan
brain_chain_hash: <hash> # set by fr-author when first written
related_frs: []         # cross-refs to other FR-IDs (added after audit)
depends_on: []          # FR-IDs that MUST ship before this can build
---
```

The fr-author skill manages most of these for you; `status` flows as you APPROVE / REVISE / REJECT.

---

## §7 — Task integration (one task per FR)

For every accepted FR, create exactly one task. Two paths:

### Path A — Cowork TaskCreate (this session, lightweight)

When working with me in Cowork, I'll automatically create a task via the TaskCreate tool when you say "accept" on an FR. The task shows up in your task widget and tracks status (pending → in_progress → completed).

### Path B — `TASKS.md` (persistent across sessions)

For long-running work, append to `TASKS.md`:

```markdown
## AUTH module · slice 1

- [ ] FR-AUTH-001 — tenant create  ·  status: accepted  ·  est: 4h
- [ ] FR-AUTH-002 — subject create  ·  status: accepted  ·  est: 4h
- [ ] FR-AUTH-003 — tenant RLS enforcement  ·  status: accepted  ·  est: 6h
- [ ] FR-AUTH-004 — admin REST: list tenants  ·  status: accepted  ·  est: 2h
- [ ] FR-AUTH-005 — admin REST: list subjects  ·  status: accepted  ·  est: 2h
```

When a PR merges that fulfills the FR, tick the box and update `shipped:` in the FR markdown frontmatter.

### Path C — Future: PROJ module (P1)

Once the PROJ module ships, every task becomes a PROJ issue with FR-ID as a label. Until then, paths A + B coexist.

---

## §8 — Worked example: FR-AUTH-001

A concrete walk-through. You author the first FR for the AUTH module.

**Slice-1 brief** (your input):

> AUTH slice 1 is "tenant + subject CRUD with RLS". Need ~5 FRs covering: tenant create, subject create, RLS isolation proof, admin REST endpoints, audit-chain integration.

**Invocation** (in terminal):

```bash
cd /Users/stephencheng/Projects/CyberSkill/cyberos/skill
cargo run -p cyberos-skill-cli -- run fr-author --executor script
```

**Interview**:

```
Q1: Which requirements doc(s) should I work from?
A:  services/auth/RFC.md, services/auth/SLICE_1_BRIEF.md
Q2: Where should I write the FR markdowns?
A:  ../docs/feature-requests/auth/
Q3: And the manifest?
A:  ../docs/feature-requests/MANIFEST.json
```

**Output** (skill writes):

`docs/feature-requests/auth/FR-AUTH-001-tenant-create.md`:

```markdown
---
id: FR-AUTH-001
title: tenant create — root-level admin can create a new tenant
module: AUTH
priority: MUST
status: draft
verify: T
phase: P0
slice: 1
owner: Stephen Cheng
created: 2026-05-14
brain_chain_hash: 4f9c...
---

## Description

A root-level admin (subject with `role: root-admin` in tenant `0`) MUST be able
to create a new tenant via `POST /v1/admin/tenants` with body `{ name, slug }`,
receiving back the created tenant's UUID, slug, and creation timestamp.

## Acceptance criteria

1. Returns 201 + JSON body containing `id`, `slug`, `name`, `created_at` on success.
2. Returns 409 + JSON error body if `slug` is already taken.
3. Returns 401 if caller is not authenticated.
4. Returns 403 if caller lacks `tenant.create` permission in tenant `0`.
5. Emits an audit row on the BRAIN audit chain: `actor`, `op: put`,
   `path: memories/decisions/auth-tenant-create-<id>.md`.

## Verification method

Integration test: `services/auth/tests/admin_rest_tenant_create.rs`. Spawns
a real Postgres + Redis container, seeds tenant `0` with a root-admin
subject, performs the POST, asserts response shape, then verifies the
audit row appears on the local BRAIN.

## Dependencies

- BRAIN module (shipped) — for audit row emission.
- AUTH RFC §3 slice 1 — for the underlying schema.

## Notes

This is the first FR-AUTH. Subsequent slice-1 FRs build on the same tenant
table and admin REST handler.
```

**Audit** (fr-audit chains automatically):

`docs/feature-requests/auth/FR-AUTH-001-tenant-create.audit.md`:

```markdown
---
fr_id: FR-AUTH-001
audited: 2026-05-14
auditor: cuo/cpo/fr-audit (v0.1.0)
verdict: PASS_WITH_REVISIONS
score: 8.5/10
---

## Findings

PASS — atomic, testable, BCP-14 compliant, has acceptance criteria.

## Suggested revisions

- §3 verify path: prefer `assert!(audit_row_present(...).await?)` over manual
  query — there's a helper in `services/auth/tests/common/mod.rs`.
- Add a verifier note: HTTP 500 should never leak the internal error; mention
  this in the acceptance criteria (NFR-SEC binds here).

## Decision

Accept with the two revisions inlined. Re-author? No — small enough to edit
in place.
```

**Your decision**: edit FR-AUTH-001 with the two revisions, set `status: accepted`. Create the task:

```markdown
- [ ] FR-AUTH-001 — tenant create  ·  status: accepted  ·  est: 4h
```

Now you (or any contributor) can pick up this task, write code in `services/auth/src/admin.rs` + `services/auth/migrations/0001_tenants.sql` + `services/auth/tests/admin_rest_tenant_create.rs`, open a PR, and the moment it merges you set `status: shipped` + `shipped: 2026-05-21` in the FR frontmatter.

---

## §9 — Frequently asked

**Q: What if I want to author an FR without invoking the skill (e.g. just write the markdown by hand)?**
A: Fine. Drop a properly-formatted markdown into `docs/feature-requests/<mod>/` with the frontmatter shown in §6. Then run `fr-audit` against it standalone to validate. The skill's value is automating the boring parts (numbering, manifest tracking, audit-chain emission); the markdown shape is the same either way.

**Q: What if I want to amend a shipped FR?**
A: Don't edit it in place. Open a new FR (e.g. `FR-AUTH-001a-tenant-create-revision-1`) that supersedes it, link `related_frs: [FR-AUTH-001]`, set the new one to `status: accepted`, and the old one to `status: deferred` (or `superseded`).

**Q: What about NFRs?**
A: NFRs use the same workflow but a different skill (planned: `nfr-author`). For now, edit `website/docs/reference/nfr-catalog.html` directly until that skill ships.

**Q: How do I find which FRs depend on which?**
A: `grep -rE '^depends_on: \[.*FR-AUTH-001' docs/feature-requests/`. We'll add a `tools/fr-graph.py` later.

**Q: Does fr-author work in chained mode without a human?**
A: Yes — CUO can invoke it as part of a longer pipeline. See `skill/skills/cuo/cpo/fr-author/PIPELINE.md` for the chaining contract.

---

## §10 — Where this fits in the build sequence

Per [`archive/2026-05-14/AUDIT_AND_PLAN.md §3.3`](archive/2026-05-14/AUDIT_AND_PLAN.md#33-recommended-build-sequence):

1. **For each module on the build-readiness list**, write the RFC (use `services/auth/RFC.md` as template).
2. **Run fr-author** against the RFC + slice-1 brief → 5–7 FRs.
3. **Run fr-audit** → audit reports per FR.
4. **Accept** → tasks created → code lands.
5. **Repeat** for slice 2 / 3 / 4 / 5 of the module.
6. **Repeat** for the next module.

The audit + plan doc gives you the module order. This doc gives you the per-FR workflow. Together they're the operating procedure for the next 24 weeks.

---

*End of workflow.* Keep this file open while authoring.
