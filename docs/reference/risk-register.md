---
title: Risk register
source: website/docs/reference/risk-register.html
migrated: TASK-DOCS-002
---

# Risk register

The register tracks the top risks reviewed in the Founder weekly sync, across six categories: technical, compliance, operational, strategic, financial, and legal. Each risk carries a likelihood x impact score, an owner, a mitigation, a contingency, and a status. The set is pulled from the "Top 15 risks" and extended with the risks one would expect for a 24-month, 23-module, regulated-market platform build. Severity is re-cast here as impact for heatmap clarity.

Numbering: RSK-01 through RSK-15 are the canonical top risks; R-EXT-* additions are inferred from project context and marked with their rationale. Summary counters on the site page track risks tracked, high/catastrophic, open, mitigated, accepted, and the six categories.

## Likelihood x impact heatmap

The generated site renders the register as a likelihood x impact heatmap plus a filterable table (category, likelihood, impact, status, free-text search). Cells are colour-coded by composite score:

- low: low concern
- med: monitor
- high: active mitigation required
- crit: sprint-blocking, Founder review

Any high-likelihood / high-impact cell is "sprint-blocking": it auto-creates a Question to the Founder via the Compliance Cockpit.

The risk rows themselves (ID, title, category, owner, likelihood, impact, score, status, description, mitigation, contingency, last reviewed, reference) are rendered client-side by the interactive page on the generated site; the row data did not survive the HTML-to-markdown migration, so it is not reproduced on this page.

## Operational rules

- Severity x likelihood produces a heat-mapped score; any High-High lands on the Compliance Cockpit and triggers a Question to the Founder.
- Risks are reviewed weekly during the Founder weekly sync; status (Open / Mitigated / Realised / Closed) is updated.
- A realised risk triggers an AAR (After-Action Review); the AAR is captured in memory Layer 2 with the `lesson-learned` tag and is surfaced in future similar contexts via GraphRAG.
- New risks added between phases require Founder approval; they are not auto-accepted from CUO's suggestion stream.

## R-EXT extensions — 2026-07-23 deep audit (TASK-IMP-140)

Seven rows added per the register's documented convention ("R-EXT-* additions are inferred
from project context and marked with their rationale"). Rationale: each row is a failure
class the operator-approved 2026-07-23 deep audit verified first-hand in this repository;
each names the benchmark gate(s) that prevent its recurrence (definitions:
`docs/verification/benchmark-gates.md`) and the recovery path. Field set per the audit's
content contract: description, cause, impact, detection, prevention, recovery, automation
tier. These rows are the first R-EXT entries carried on this page itself (the pre-audit
rows live in the site page's client-side data, which did not survive the HTML-to-markdown
migration — see above).

### R-EXT-01 — Self-approval / skipped HITL

- **Description:** An agent advances a task across the two human-acceptance gates
  (`reviewing -> ready_to_test`, `testing -> done`) without a recorded human verdict;
  unreviewed work is marked done.
- **Cause:** No mechanical lock — the HITL requirement lived in prose (doctrine, spec
  frontmatter) that a non-compliant or confused agent could simply not read.
- **Impact:** Shipped work nobody accepted; the acceptance story on `done` tasks is
  fiction; downstream tasks build on unreviewed foundations.
- **Detection:** The audit chain lacks `status_overridden` verdict rows for gate
  transitions; `task-reconcile` reds on `done` tasks with no audit artefacts.
- **Prevention:** G2 (HITL mechanical lock — `backlog-mutate flip` refuses the two
  transitions without `--verdict-by` + `--verdict-evidence`).
- **Recovery:** `task-reconcile` the affected tasks; operator flips `done ->
  ready_to_review` to force re-review; verdict recorded on the re-cross.
- **Automation tier:** ci (verdicts themselves stay human).

### R-EXT-02 — Vacuous green gates

- **Description:** The machine-gate floor reports GREEN having run zero commands, and the
  green is consumed as evidence at both HITL gates.
- **Cause:** Fail-open `run-gates.sh` (empty command set fell through to GREEN) plus
  autodetect that learned nothing on monorepo layouts.
- **Impact:** False confidence exactly where confidence is load-bearing; every "gates
  green" claim in the affected window is unsupported.
- **Detection:** G1's scratch-install probe (empty gate env must exit RED); reading a
  gates transcript that lists no GATE lines before GREEN.
- **Prevention:** G1 (gate-floor non-vacuous: empty env exits RED unless the operator
  sets the explicit empty-ack, which prints `EMPTY-ACKNOWLEDGED`, never GREEN).
- **Recovery:** Restore or configure real commands in `.cyberos/config.yaml`
  (`gates.build/lint/test/coverage`), re-run the gates, re-evaluate anything accepted on
  the vacuous green.
- **Automation tier:** ci.

### R-EXT-03 — Config wipe on reinstall

- **Description:** Reinstall/update regenerates `gates.env` and the operator's only
  working test command survives nowhere but a timestamped `.bak` (observed: C1 — a
  working `TEST_CMD` found only in `gates.env.bak.1784761166`).
- **Cause:** `gates.env` is machine-owned and regenerated by design, but durable operator
  overrides had no durable home until `.cyberos/config.yaml` landed — edits to the env
  file were silently churned.
- **Impact:** The repo's own test gate silently degrades to SKIP; nothing red appears
  anywhere; the loss is discovered by archaeology.
- **Detection:** G16's double-install diff + config-survival assert; comparing `gates.env`
  against its newest `.bak` after any install.
- **Prevention:** G16 (idempotent reinstall: pre-set `.cyberos/config.yaml` survives
  byte-identical) + the config.yaml override layer as the documented durable home.
- **Recovery:** Restore the command from the newest `.cyberos/gates.env.bak.*` into
  `.cyberos/config.yaml` (`gates.test`), never back into `gates.env`.
- **Automation tier:** ci.

### R-EXT-04 — Prompt injection via repo files

- **Description:** A repo-reading vendored skill ingests attacker-influencable file
  content (specs, docs, code comments) in a consumer repo and treats it as instructions —
  steering workflows, exfiltrating context, or laundering a protocol change.
- **Cause:** Skills that read repositories shipped without `untrusted_inputs`
  declarations or wrapping rules; injection posture was a virtue of the best skills, not
  a floor.
- **Impact:** A consumer repo's own files can steer the workflow that audits them —
  worst-case, the gate asks the defendant for the verdict.
- **Detection:** Injection-marker scans over skill inputs; G8's presence/shape scan
  failing on any repo-reading skill lacking the discipline.
- **Prevention:** G8 (injection-discipline coverage: `untrusted_inputs` frontmatter +
  non-empty per-skill `references/UNTRUSTED_CONTENT.md` on every repo-reading vendored
  skill); §11 of the memory protocol for the BRAIN side.
- **Recovery:** Quarantine the affected run's outputs; re-run with the patched skill;
  treat any protocol/status change authorised only by file content as void (§16.4).
- **Automation tier:** ci+human (the wrapping rules' quality stays a human review).

### R-EXT-05 — Payload/doc divergence

- **Description:** Vendored docs/skills reference tools or files the built payload does
  not deliver (observed: `skill-log.mjs` named by ship-tasks step 27, vendored by
  nothing).
- **Cause:** The vendor list in `tools/install/build.sh` and the references in vendored
  prose evolve independently; nothing walked one against the other.
- **Impact:** Consumer workflows halt mid-step on missing tools; the workflow's own floor
  is unreachable in every installed repo.
- **Detection:** G5's reference walker over a scratch-built payload (zero unresolved
  intra-payload paths).
- **Prevention:** G5 (payload completeness), with the `benchmark-gates:exempt` inline
  marker keeping illustrative paths honest instead of weakening the walk.
- **Recovery:** Add the missing file to the vendor list (or fix the referencing doc);
  rebuild; consumers pick it up on their next update.
- **Automation tier:** ci.

### R-EXT-06 — Partial install window

- **Description:** The install's vendor step removes then re-copies the `.cyberos/`
  tree, so a concurrent reader (an agent mid-task, a gates run) can observe a missing or
  half-written machine.
- **Cause:** rm/cp vendoring with no staging — the tree is briefly absent by
  construction, and a crash mid-copy leaves it broken until the next install.
- **Impact:** Broken `.cyberos/` mid-install: workflows that read the spine during the
  window fail confusingly or, worse, proceed on a partial rule set.
- **Detection:** G16's reader poll during reinstall (the vendored spine file must never
  be absent mid-loop).
- **Prevention:** Staged-directory + atomic-rename install (TASK-IMP-137) + G16 keeping
  the loop honest afterwards.
- **Recovery:** Re-run the install (idempotent per G16); no operator data is at risk —
  the corpus and config live outside the vendored tree.
- **Automation tier:** ci.

### R-EXT-07 — BRAIN frozen-by-layout

- **Description:** The live memory store fails layout invariants (stray non-canonical
  trees), so every protocol-compliant agent must refuse writes — the audit trail silently
  stops accumulating while work continues.
- **Cause:** Store pollution: artefacts written outside the canonical layout (observed:
  stray `adrs/` + `impl-plans/` failing `layout-root-canonical`), freezing the store at
  `FROZEN_RECOVERABLE`.
- **Impact:** Decisions, audits, and plans of an entire working period are recorded
  nowhere durable; the repo's own doctrine (record into the BRAIN) is unsatisfiable —
  including for the audit that found this.
- **Detection:** `cyberos doctor` (state below READY); G9 wiring doctor into the machine
  gates where memory is installed, so the freeze cannot stay quiet.
- **Prevention:** G9 (BRAIN health in gates) + walker invariants on the canonical layout.
- **Recovery:** Operator-gated store repair per TASK-MEMORY-303 (canonical `move` of the
  stray trees through the writer), then execute the deferred recording per
  `docs/tasks/improvement/TASK-IMP-140-benchmark-gates-drift-protection/brain-recording-checklist.md`.
- **Automation tier:** ci (detection); the repair itself is operator-gated.

## Changelog

History lives in the [changelog](./changelog.html); this page describes only the current state.
