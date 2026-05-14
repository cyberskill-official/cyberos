# `fr-author/references/` — index

> Auxiliary documents the skill loads at runtime for protocol details, decision trees, and failure-mode dictionaries. Each is a Markdown file the skill body references via `references/<NAME>.md`. New references are added as the skill grows; obsolete references are tombstoned per the registry's MAJOR-bump rules.

## Inventory

| File | Purpose | Shared with `fr-audit`? |
| --- | --- | --- |
| `AMENDMENT_PROTOCOL.md` | Amendment record schema, risk-class table, batch aggregation, inline-apply | No (create-only) |
| `ANTI_FABRICATION.md` | What the skill MUST NEVER invent | Yes — DIVERGES (see §"Per-skill divergence" below) |
| `EU_AI_ACT_DECISION_TREE.md` | Article 5 / Annex III / Article 50 decision tree | Yes — DIVERGES |
| `FAILURE_MODES.md` | BOOT-001..008, CONTRACT_DRIFT, INPUTS_CHANGED, EXHAUSTED, STALE | No (create-only) |
| `HITL_PROTOCOL.md` | HITL_BATCH_REQUEST format, RESUME protocol | Yes — DIVERGES |
| `MANIFEST_SCHEMA.md` | `fr-manifest@2` schema, hashing rules, re-entrancy invariants | No (create-only) |
| `PLAN_RENDER.md` | Plan-approval block format | No (create-only) |
| `UNTRUSTED_CONTENT.md` | `<untrusted_content>` wrapping rules + injection-marker scan | Yes — DIVERGES |

## Per-skill divergence (4 files)

The four "Yes — DIVERGES" rows above are present in BOTH `cuo/cpo/fr-author/references/` AND `cuo/cpo/fr-audit/references/` but are **NOT byte-identical** between the two skills. The divergence is intentional and pre-dates the v0.2.0 contracts split (DEC-090):

- **`HITL_PROTOCOL.md`** — fr-author's version emphasises the create-side gate ordering (PLAN approval → amendment → exhausted disposition). fr-audit's version emphasises the audit-side categories (`stale_fr_disposition`, `low_confidence_field`, `untrusted_marker_hit`).
- **`UNTRUSTED_CONTENT.md`** — fr-author's version specifies wrapping rules at write-time. fr-audit's version specifies detection rules at read-time and includes a SAFE-001..SAFE-004 mapping that fr-author doesn't need.
- **`ANTI_FABRICATION.md`** — fr-author's version lists what the LLM MUST NEVER invent during generation. fr-audit's version lists what the auditor MUST NEVER assume the FR contains during checking.
- **`EU_AI_ACT_DECISION_TREE.md`** — fr-author's version emphasises which questions to ask the user during PLAN approval. fr-audit's version emphasises which trigger phrases QA-001..003 detect.

### Why the divergence wasn't unified in v0.2.0

The v0.2.0 contracts split (DEC-090) promoted the FR template to a single shared contract because the template body IS byte-identical between consumers. The four reference docs above are NOT byte-identical because each is tuned to its consumer's lifecycle phase (write-side vs. read-side, generate-time vs. validate-time). Promoting them to shared contracts would require either:

- (A) a single super-doc that covers both lifecycle phases, with conditional sections — risks being twice as long, half as readable; OR
- (B) splitting into `<NAME>.shared.md` (the common parts) + per-skill `<NAME>.local.md` (the deltas) — risks fragmentation that complicates updates.

Neither option clearly improves on the current state, so the divergence is documented here as **intentional** and left as-is.

### When to revisit

The audit at registry v0.2.2 confirmed all four files diverge by SHA-256. If a future audit shows the divergence has grown (e.g., one side has drifted significantly while the other has stayed fixed), or if the cost of keeping the two copies in sync becomes a maintenance pain-point, file a refinement_proposal pointing at this section and propose option A or B. Track the decision under `memories/refinements/` in the BRAIN.

## Loading discipline

Skills load reference docs at runtime via `read_file('references/<NAME>.md')`. The files MUST be present at deployment time (the skill bundle includes them); a missing reference file is a `BOOT-002` failure (per `FAILURE_MODES.md`).

References are NOT contracts — they are skill-internal protocol documents. They do NOT appear in `depends_on_contracts:`. They DO appear in the skill's MAJOR bump checklist: any change to a reference file's semantics requires a coordinated changelog entry in this skill's CHANGELOG.md.

## Citations

- **DEC-090** — registry v0.2.0 contracts split (why some files moved up to `cyberos/docs/contracts/` and these did not).
- **`cuo/cpo/fr-audit/references/README.md`** — the audit-side counterpart with its own per-skill divergence note.
- **registry v0.2.2 audit** — confirmed by SHA-256 diff that all four "Yes — DIVERGES" files differ between the two skills as of 2026-05-06.
