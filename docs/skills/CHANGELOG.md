# CHANGELOG — `cyberos/docs/skills/` registry

> Format: [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/).
> SemVer at the registry level: MAJOR breaks the layout or the SKILL.md
> frontmatter contract; MINOR adds a new persona namespace or new contract
> sections; PATCH is editorial / typo fixes.

---

## v0.1.2 — 2026-05-05 (comprehensive guide + hello-world skill)

### Added

- `cuo/_shared/hello-world/` — the simplest possible CyberOS skill,
  authored as a teaching example. Carries the full 27-field frontmatter
  contract with the most trivial body (read a name → write a greeting
  markdown). Includes `acceptance/` golden-input + golden-output +
  golden-envelope fixtures (`greeting_sha256`:
  `ddd394ab7eaa5950ce5ab2ea9f7eb37199fd0d5d42a37be9fdf56ec490d39805`).
  Used as Example 1 throughout `GETTING_STARTED.md`.

### Changed

- `GETTING_STARTED.md` — substantially expanded into a comprehensive
  basic→advanced guide. Now organised into three tiers (🌱 Beginner,
  🌿 Intermediate, 🌳 Advanced) with 20 numbered sections, 6 embedded
  Mermaid diagrams (skill-as-folder, three trigger paths, frontmatter
  anatomy, chain sequence, validation pyramid, fine-tuning loop,
  skill lifecycle state diagram), 5 cookbook recipes
  (build / chain / debug / retire / add-persona), an FAQ section
  covering 8 common confusions, and a glossary of 22 terms.
- README.md and registry CHANGELOG entry for v0.1.1 unchanged but
  now point at the much more comprehensive guide.

### Driver

User feedback after v0.1.1: "comprehensive as possible, basic →
advanced; simple examples for newbies; visualisations help more than
text." The previous v0.1.1 GETTING_STARTED.md was a quick on-ramp; this
v0.1.2 expansion turns it into the canonical learning curriculum.

### Backwards compatibility

Pure additions. The hello-world skill is deliberately at v1.0.0 (not
v0.1.0) because its purpose — a teaching example — is locked. Future
v2.0.0 would mean a different skill entirely; bumping the existing one
is forbidden.

---

## v0.1.1 — 2026-05-05 (operational guide)

### Added

- `cyberos/docs/skills/GETTING_STARTED.md` — the operational view of the
  registry: 30-second mental model, the two unrelated meanings of "audit"
  (action_log row vs. fr-audit skill), the three trigger paths
  (direct / supervisor-routed / chained), a 5-command worked example for
  building a tiny new skill (`fr-priority-rebalance`), the three layers
  of skill validation (mechanical / functional / operational), the
  fine-tuning lifecycle (tightening, prompt refinement, acceptance-set
  growth, drift-signal feedback, replacement vs revision), a "what
  doesn't exist yet" section, and a TL;DR cookbook table.
- `acceptance/` folder convention referenced. Skills SHOULD ship
  golden-input + golden-output pairs for regression testing; the
  runner is not yet built.
- README.md updated to point at GETTING_STARTED.md as the entry point.

### Driver

User feedback after v0.1.0: "the structure is complicated, and after all I
still have no idea step by step about how to build a skill, trigger it
standalone/chained, audit it, validate it worked, fine-tune it." The
architecture docs answered "what" and "why" but not "how do I do this on
Tuesday afternoon." GETTING_STARTED.md is the missing operational
on-ramp.

### Backwards compatibility

Pure additions; no existing skill needs to change. Existing reference
docs continue to be authoritative; GETTING_STARTED.md cross-references
them in its "Map: when to read which architecture doc" section rather
than duplicating them.

---

## v0.1.0 — 2026-05-05 (initial registry bootstrap)

### Added

- `cyberos/docs/skills/README.md` — registry contract: layout (Option B,
  persona-grouped + nested workflow skills), SKILL.md frontmatter contract,
  the five inherited contracts (audit / chain / plug-in / versioning / trust),
  routing rules, and citations to the authoritative PRD/SRS/AGENTS.md sections.
- `cyberos/docs/skills/cuo/README.md` — CUO persona namespace index.
  Lists the 14 sub-personas (10 canonical + 4 emergent) per DEC-052;
  marks per-phase availability.
- `cyberos/docs/skills/cuo/cpo/SKILL.md` — first persona-card (Chief Product
  Officer). Owns FR backlog management.
- `cyberos/docs/skills/cuo/_shared/feature-request-template/` — first
  cross-persona shared skill: holds the canonical `feature_request@1`
  template (sourced verbatim from `feature-request/FR_CREATE_AND_AUDIT.md`
  v2.0.0 §18).
- `cyberos/docs/skills/cuo/cpo/fr-create/` — port of the create-and-audit
  prompt's create half (sections §0–§14 + §18 of v2.0.0). Standalone
  trigger: PRD → backlog → FR markdowns. Produces FR files + a
  `fr-manifest@2` state file.
- `cyberos/docs/skills/cuo/cpo/fr-audit/` — port of the create-and-audit
  prompt's audit half (sections §15–§17 of v2.0.0, plus shared §7 HITL +
  §12 untrusted-content). Standalone trigger: existing FR markdowns →
  sibling audit reports. Chains naturally after `fr-create`.

### Layout decision (Option B trade-off)

Three layouts were considered (full diagram retained in the conversation
log of 2026-05-05). Option B was selected because:

1. It is the only layout that keeps each workflow as a standalone-trigger
   atom AND preserves persona grouping in the filesystem AND honours
   PRD §3.2's `cuo/<role>/` mandate AND DEC-061's reusable-skill clause
   (via `_shared/`).
2. The audit row schema in SRS §6.7 (`persona_id`, `skill_id`,
   `skill_version`, `row_kind`) maps 1:1 to the workflow leaf without
   requiring a sub-skill field.
3. Plug-in extraction works at three granularities (workflow / persona /
   whole-CUO) without restructuring.

### Skill self-test checklist (run before committing any new SKILL.md)

A skill is registry-valid when ALL of:

- [ ] Folder name is kebab-case and matches `name:` in frontmatter.
- [ ] `SKILL.md` parses as Markdown with one YAML frontmatter block, no
      mid-file `---` outside fenced code spans (AGENTS.md §4.3 + DEC-087).
- [ ] All 27 frontmatter fields from `cyberos/docs/skills/README.md` §3 are
      present (or explicitly `null` where allowed).
- [ ] `expects:` and `produces:` reference real JSON schemas reachable
      from this folder or `_shared/`.
- [ ] `allowed_brain_scopes.write` is empty UNLESS the skill is explicitly
      authorised to mutate BRAIN (separate decision per skill, recorded
      in CHANGELOG).
- [ ] `allowed_mcp_tools` is exhaustive — gateway will reject unlisted
      tools at call time.
- [ ] `audit.row_kind` matches the `produces.output_kind` enum.
- [ ] At least one `references/` doc OR a clear note that none are needed.
- [ ] `CHANGELOG.md` exists in the skill folder, with at least a v0.1.0
      entry.
- [ ] Adding the skill to `cyberos/docs/skills/README.md` §7 index does
      not duplicate an existing `(persona, name)` pair.

### Known follow-ups (tracked outside this CHANGELOG)

- Wire the registry into the CyberOS-PRD/SRS source-of-truth (a one-line
  reference from PRD Part 6 + SRS Part 6.2 pointing here). Parked because
  PRD/SRS are .docx and must be edited in Word; raised as a separate
  feature request once `fr-create` is operational and can self-host the
  request.
- Migrate the existing `feature-request/FR_CREATE_AND_AUDIT.md` repo into
  this registry as a soft-deprecation: leave the prompt in place, point its
  README to `cyberos/docs/skills/cuo/cpo/fr-create/` + `fr-audit/`. Bump
  that prompt's CHANGELOG to v2.1.0 with a "MOVED" note.
- Define `_shared/` for additional cross-persona skills as they emerge
  (e.g., `draft-payslip-explanation` from DEC-061's worked example, owned
  by neither CFO nor CHRO exclusively).

---

## How to add a future entry

For a new release, prepend a new
`## vX.Y.Z — <ISO date> (<one-line summary>)` block above v0.1.0. Standard
sub-sections:

- **Added** — new skills, new personas, new shared assets, new contracts.
- **Changed** — semantics changes that don't break the layout or
  frontmatter contract.
- **Deprecated** — skills moving to `superseded_by:` in their frontmatter.
- **Removed** — soft-deletions only; skill folders move to
  `cuo/<role>/_archive/<skill-id>/` with a tombstone CHANGELOG entry.
  The folder body is preserved for audit (per AGENTS.md §4.6).
- **Layout** — only on MAJOR bumps; describes the new tree shape.
- **Backwards compatibility** — what existing skills still validate, what
  needs migration.
