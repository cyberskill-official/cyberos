# `fr-author` acceptance fixtures

Layer 2 (functional) regression tests per registry README Part 13.3. Empty in v0.2.1 — production fixtures pending the runtime build (Phase D of the host-adapter pipeline, registry README Part 9).

## What goes here

For each canonical scenario the skill must handle, ship one folder containing:

- `golden-input.json` — a known input envelope matching `../envelopes/fr-author.input.json`
- `golden-output.json` — the expected output envelope at BATCH_COMPLETE
- `golden-frs/FR-NNN-<slug>.md` — the expected FR markdowns the skill writes
- `golden-manifest.json` — the expected `fr-manifest@2` state file
- `description.md` — what scenario this covers and why

## Scenarios to cover (priority order)

The first three are Tier-1 — they exercise the contract:

1. **happy-path-tiny** — 1 PRD file, 1 FR generated, PASS verdict. Verifies CONTRACT_ECHO + PLAN + WORKER + envelope shape end-to-end.
2. **plan-halt-on-ambiguity** — PRD with insufficient detail forces ≥1 HITL question; verifies plan-approval Question primitive + halt + resume.
3. **eu-ai-act-escalation** — PRD describes a borderline AI feature; verifies QA-001/002/003 boundary detection + escalation to `cuo-clo`.

Tier-2 scenarios (cover related modes):

4. **batch-of-three** — PRD with enough material for 3 FRs, full batch.
5. **resume-after-hitl** — start, hit HITL, resume with answer, complete.
6. **stale-fr-disposition** — pre-existing FR's disk SHA differs from manifest; verifies STALE-001 surfacing.
7. **amendment-low-risk** — generation reveals a missing dep; verifies inline amendment apply.
8. **invariant-breach** — input designed to trigger INV-003 (coverage gap); verifies `refinement_proposal` envelope emission.

Tier-3 scenarios (quality-of-life):

9. **vietnamese-prd** — Vietnamese-language input; verifies localisation handoff per registry README Part 17.

## How to run (when the runtime ships)

The acceptance harness is a v0.3.0 follow-up (registry README Part 26). Until then: read the golden fixtures, paste the input into Claude.ai with the SKILL.md body as the system prompt, and diff manually against the golden output.

## Status

📋 **Empty (pending runtime + harness).** Adding the first three Tier-1 fixtures is a Tier-1 task on the v0.3.0 milestone.

## See also

- Registry README Part 11 (the canonical fr-author → fr-audit walk-through narrating an example trace).
- Recipe 8 in registry README Part 19 — the procedure for authoring acceptance fixtures.
- AGENTS.md §4.10 — ingestion-coverage discipline for sources used during fixture authoring.
