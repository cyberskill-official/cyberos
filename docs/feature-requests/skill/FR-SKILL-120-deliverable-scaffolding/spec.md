---
id: FR-SKILL-120
title: "Workflow/skill wiring for visual deliverables - FR folder scaffolding + template@1 citations in authoring contracts"
module: SKILL
priority: SHOULD
status: done
class: improvement
verify: T
phase: Wave D - visual deliverables
owner: Stephen Cheng (CTO)
created: 2026-07-12
shipped: 2026-07-12
memory_chain_hash: null
related_frs: [FR-DOCS-004, FR-TPL-001, FR-CUO-205]
depends_on: [FR-DOCS-004]
blocks: []
source_pages:
  - tools/cyberos-init/plugin/commands/create-feature-requests.md
  - modules/skill/feature-request-author/SKILL.md
source_decisions:
  - "2026-07-12 operator decision: workflows/skills use the templates module to create new deliverables; new FRs are born as folders with assets/."
language: markdown (contracts + command docs)
service: modules/skill/ + tools/cyberos-init/plugin/
new_files: []
modified_files:
  - tools/cyberos-init/plugin/commands/create-feature-requests.md
  - modules/skill/feature-request-author/SKILL.md
  - modules/skill/feature-request-audit/SKILL.md
  - modules/cuo/chief-technology-officer/workflows/ship-feature-requests.md
  - tools/cyberos-init/init.sh
---

# FR-SKILL-120: Deliverable scaffolding wiring

## §1 - Description

The authoring path catches up with the new layout and the templates module, so every future deliverable is born in the right shape.

Normative clauses:

1. `/create-feature-requests` step flow MUST scaffold each new FR as `<module>/<STEM>/spec.md` (+ `assets/` on first asset) and write the audit as `<STEM>/audit.md`; all path examples in the command doc updated.
2. `feature-request-author` MUST document the folder layout in its artefact section (spec.md naming, sibling audit, relative asset references) and name `modules/templates/contracts/TEMPLATE.md` as the presentation contract its output is rendered through (informative pointer - authoring stays markdown).
3. `feature-request-audit` MUST resolve `<STEM>/audit.md` as the report path for folder-layout FRs while keeping `.audit.md` sibling resolution for any legacy flat file it is pointed at (transition note, one release).
4. The ship workflow doc's artefact-path examples and the backlog-state-update evidence citations MUST use the folder paths; `init.sh` MUST scaffold `docs/feature-requests/` ready for folder-layout FRs (no flat-file example remains in scaffolded docs).
5. Asset discipline MUST be stated where authors read it (author SKILL.md + command doc): assets live in the FR's own `assets/`, referenced relatively; no cross-FR asset reaches into another FR's folder.

## §2 - Why this design

Contracts drive agents; if the contracts still describe flat files, every new FR fights the migration. One doc-wiring FR right after the layout FR keeps the fleet coherent.

## §4 - Acceptance criteria

1. **Command scaffolds folders** (§1 #1) - command doc contains the folder grammar and no flat-file instruction (grep).
2. **Author/audit contracts updated** (§1 #2, #3) - artefact sections name spec.md/audit.md + the TEMPLATE.md pointer; legacy transition note present in the audit skill.
3. **Ship + init coherent** (§1 #4) - workflow doc examples use folder paths; init.sh-scaffolded docs carry none of the old grammar.
4. **Asset discipline stated** (§1 #5) - both author-facing docs carry the own-assets rule (grep).

## §5 - Verification

Doc assertions (grep-level, executable in `scripts/tests/test_fr_layout.sh` t07-t10 extension): command grammar, contract sections, workflow examples, asset rule. (AC 1-4.)

## §3 - Contract

Scaffold grammar: `docs/feature-requests/<module>/<STEM>/spec.md`; audit: `<STEM>/audit.md`; assets: `<STEM>/assets/<file>` referenced as `assets/<file>`.

## §6 - Implementation skeleton

Four doc edits + init.sh template sweep; the t07-t10 asserts ride the FR-DOCS-004 suite.

## §7 - Dependencies

FR-DOCS-004 (layout must exist first). Informative link to FR-TPL-001's contract.

## §8 - Example payloads

Author scaffold: `mkdir -p docs/feature-requests/ten/FR-TEN-301-rls-sweep && $EDITOR .../spec.md`

## §9 - Open questions

None blocking. Whether OTHER deliverable classes (SOW, PRD...) also become folders is decided per-contract later; TEMPLATE.md already renders them.

## §10 - Failure modes inventory

1. Agent follows stale cached contract - trigger tests unchanged, but contract text is what ships in the payload; next /init update refreshes fleets.
2. Legacy flat FR appears post-migration (old branch merge) - audit skill's transition note covers reading it; regen warns loudly (FR-DOCS-004 #4) making it visible.
3. Asset reach-across - stated rule + review; renderer resolves only own-folder assets (FR-DOCS-005 fails missing assets loudly).
4. init scaffolds diverge from repo docs - t09 greps the scaffolded output too.
5. Plugin copy drift - payload rebuild vendors the edited docs; version-sync gate carries them.

## §11 - Implementation notes

Keep the transition note dated; one release later it drops (same sunset pattern as backlog-state-update @1).

*End of FR-SKILL-120.*
