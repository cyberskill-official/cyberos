# `fr-audit/references/` — index

> Auxiliary documents the auditor loads at runtime. Each is a Markdown file the skill body references via `references/<NAME>.md`. Mirrors the structure of `cuo/cpo/fr-author/references/` with audit-side specialisations.

## Inventory

| File | Purpose | Shared with `fr-author`? |
| --- | --- | --- |
| `ANTI_FABRICATION.md` | What the auditor MUST NEVER assume the FR contains | Yes — DIVERGES (see §"Per-skill divergence" below) |
| `EU_AI_ACT_DECISION_TREE.md` | Article 5 / Annex III / Article 50 decision tree (read-side) | Yes — DIVERGES |
| `FAILURE_MODES.md` | BOOT codes specific to audit-side failures (BOOT-001/002/003/004/006/007) | No (audit-only) |
| `HITL_PROTOCOL.md` | HITL_BATCH_REQUEST format + RESUME protocol; rule_id values originate here | Yes — DIVERGES |
| `UNTRUSTED_CONTENT.md` | Read-time detection of `<untrusted_content>` markers + SAFE-001..004 mapping | Yes — DIVERGES |

## Per-skill divergence (4 files)

The four "Yes — DIVERGES" rows above are also present in `cuo/cpo/fr-author/references/` but are **NOT byte-identical** between the two skills. The divergence is intentional. See [`cuo/cpo/fr-author/references/README.md` §"Per-skill divergence"](../../fr-author/references/README.md#per-skill-divergence-4-files) for the full rationale (why each file diverges, why v0.2.0 didn't unify them, and when to revisit).

The audit-side specifically tunes each file to the validate-time / read-time perspective:

- **`HITL_PROTOCOL.md`** — defines the audit-side HITL categories (`stale_fr_disposition`, `low_confidence_field`, `untrusted_marker_hit`) that originate in the rubric's rule semantics.
- **`UNTRUSTED_CONTENT.md`** — adds the SAFE-001..SAFE-004 detection rules that fr-author doesn't need (fr-author wraps; fr-audit detects).
- **`ANTI_FABRICATION.md`** — phrases the prohibitions from the auditor's perspective (e.g. "auditor MUST NOT invent customer quotes during a fix" rather than the create-side "MUST NOT invent quotes during generation").
- **`EU_AI_ACT_DECISION_TREE.md`** — surfaces the trigger phrases QA-001..003 detect, rather than the questions fr-author asks during PLAN approval.

## When to revisit

Same trigger as the create-side: if the divergence grows or the maintenance cost of keeping two copies in sync becomes a pain-point, file a refinement_proposal pointing at this section and the create-side counterpart. The audit at registry v0.2.2 confirmed all four files diverge by SHA-256 as of 2026-05-06.

## Loading discipline

Same as the create-side: references load via `read_file('references/<NAME>.md')`, missing file = `BOOT-002`, references do NOT appear in `depends_on_contracts:`, semantic changes require a coordinated CHANGELOG entry in this skill's CHANGELOG.md.

## Citations

- **DEC-090** — registry v0.2.0 contracts split.
- **`cuo/cpo/fr-author/references/README.md`** — the create-side counterpart.
- **registry v0.2.2 audit** — SHA-256 diff confirmation.
