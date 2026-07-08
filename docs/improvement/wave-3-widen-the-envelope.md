# Wave 3 - widen the envelope (IMP-023..030)

Goal: the dream loop earns wider autonomy - ranked proposals, spend/latency/drift gates, auto-revert - then `mode: auto` for the docs/skills envelope, the skill-curation loop, and the first fine-tuning pilot. Nothing here flips without the Wave 1-2 measurement layer running. Report: Stages 2-4.

Standing rule for this wave: the dream-loop denylist (auth, audit, RLS, PII, cost ledger, secrets, deploy, tooling) is untouchable. Tasks below extend gates around the loop, never exceptions through it.

---

### IMP-023: groom draft FRs with value and confidence

`refs: R49 | prio: p1 | effort: m | deps: - | area: process`

Context: 141 draft FRs carry priority/effort but no value signal; the proposal ranker (IMP-024) needs that metadata to order work by expected impact.

Scope:
- Extend the FR frontmatter schema with `value: 1-5` and `confidence: 1-5` (documented anchors: value 5 = revenue/security-critical, 1 = cosmetic; confidence 5 = well-understood, 1 = speculative).
- Agent proposes scores for all 141 drafts in batches of ~30, each batch as a separate commit with a scoring-rationale table in the commit message; operator adjusts freely in review.
- Update `scripts/rebaseline_fr_status.py` to tolerate and report the fields; BACKLOG.md view gains a top-20-by-value/effort section.

Acceptance:
- [ ] All draft FRs scored; rebaseline runs clean.
- [ ] Scoring anchors documented in the FR template.
- [ ] Operator has reviewed at least the top-20 list (recorded in ledger).

Touches: `docs/feature-requests/**`, `scripts/rebaseline_fr_status.py`.

---

### IMP-024: dream proposal ranking

`refs: Stage 2 | prio: p1 | effort: s | deps: IMP-023 | area: cuo`

Context: the dream loop evaluates proposals in generation order; high-impact low-risk work can queue behind trivia.

Scope:
- Add a ranking step in `modules/cuo/cuo/core/dream_loop.py` between generation and gating: score = f(value, confidence, risk_class, effort), spec'd and unit-tested; `eu_ai_act_risk_class: high` and denylist-adjacent proposals rank last and never auto-apply regardless of mode.
- Rank inputs read from FR frontmatter (IMP-023) when the proposal maps to an FR, else from the proposal's own declared class.
- Ranked queue recorded in the dream audit rows so the morning review shows what was considered and why.

Acceptance:
- [ ] Unit tests: ordering, high-risk demotion, tie-breaks.
- [ ] A propose-mode run in dev records a ranked queue in the audit trail.
- [ ] `python -m pytest modules/cuo` green.

Touches: `modules/cuo/cuo/core/dream_loop.py`, `modules/cuo/cuo/core/proposal_applier.py`, tests.

---

### IMP-025: dream budget, latency and drift gates

`refs: Stage 2 | prio: p1 | effort: m | deps: IMP-009 | area: cuo`

Context: today a prompt change that doubles inference time or spend would pass the gates; and nothing proves the loop has stayed inside its envelope historically.

Scope:
- Budget gate: per-window LLM spend cap read from the IMP-009 ledger; exceeding it halts the cycle with a `cuo.dream_budget_halt` audit row.
- Latency gate: for proposals touching prompts/skills with a goldenset, compare mean eval latency before/after; regression beyond threshold (config in dream.yaml) fails the proposal.
- Envelope-drift validator: `cyberos-cuo verify-envelope` replays historical `cuo.dream_*` audit rows against the current allowlist/denylist and alerts on any action that today's envelope would forbid; wire into the nightly schedule.
- All three thresholds live in `modules/cuo/config/dream.yaml` with commented defaults.

Acceptance:
- [ ] Each gate has a forced-failure test (seeded spend, slow prompt, synthetic out-of-envelope history row).
- [ ] verify-envelope runs green on the real audit history.
- [ ] dream.yaml documents every new knob.

Touches: `modules/cuo/` (loop, config, CLI), `deploy/vps/` (schedule).

---

### IMP-026: auto-revert on gate regression

`refs: Stage 2 | prio: p1 | effort: s | deps: IMP-008 | area: cuo`

Context: `cuo.dream_reverted` exists but reverting waits for human review; an auto-applied change that regresses the next gate run should undo itself.

Scope:
- After any auto-applied change (relevant only once IMP-027 flips auto for docs/skills), the loop schedules an immediate goldenset re-run for the touched module; on failure it reverts the exact diff (content-addressed put makes this deterministic), emits `cuo.dream_reverted` with the failing case id, and demotes the proposal source (cool-down list so the same proposal is not retried next window).
- Revert must itself pass the write gates (precondition on body hash); on conflict, freeze the loop (`FROZEN_HUMAN` semantics) and alert.

Acceptance:
- [ ] Simulated regression in dev: apply, fail goldenset, observe revert + audit rows + cool-down.
- [ ] Conflict path freezes and alerts rather than force-writing.
- [ ] Tests cover both paths.

Touches: `modules/cuo/cuo/core/dream_loop.py`, tests.

---

### IMP-027: enable auto mode for docs/skills envelope

`refs: Stage 2 | prio: p1 | effort: s | deps: IMP-020, IMP-021, IMP-024, IMP-025, IMP-026 | area: cuo`

Context: the enablement ladder (off -> propose -> auto) is triple-locked by design; this task is the governed flip, not a code feature.

Scope:
- Pre-flight evidence pack: 14 consecutive days of propose-mode ranked queues, two clean nightly scorecards (IMP-021), verify-envelope green, revert drill passed (IMP-026).
- Operator ritual documented in `docs/auto-work/auto-mode-runbook.md`: review dream.yaml (especially denylist), set `mode: auto`, run with `--allow-auto-apply`, keep `CYBEROS_DREAM_KILL` procedure one command away.
- First week in auto: daily morning digest of applied changes to chat; any revert event pauses auto (back to propose) until reviewed.
- Record the flip as a DEC entry.

Acceptance:
- [ ] Evidence pack assembled and linked in the ledger.
- [ ] Runbook merged; DEC entry written by the operator.
- [ ] First-week digest mechanism proven in propose mode beforehand.

Touches: `modules/cuo/config/dream.yaml` (operator edit), `docs/auto-work/`, `.cyberos-memory` DEC entry (operator).

---

### IMP-028: ACE-style skill curation loop

`refs: Stage 3 | prio: p1 | effort: l | deps: IMP-008 | area: cuo`

Context: SKILL.md files are the skill library and the published lesson is that the library is the performance; today lessons from sessions and gate logs never flow back into skills systematically.

Scope:
- Reflector: batch job mining session transcripts (AGENTS.md §18 ledger where enabled), `.awh` promotion/failure logs (IMP-011 taxonomy), and DEC entries for candidate lessons (failure patterns, discovered conventions, better prompts).
- Curator: converts accepted lessons into compact delta proposals against the relevant SKILL.md/workflow file - dedup against existing content, aging rule for stale advice - emitted as ordinary dream-loop proposals so every existing gate (envelope, risk class, goldenset, IMP-029 pairing) applies.
- Cap: max N skill deltas per week (dream.yaml knob); every delta cites its evidence (log line, session id, DEC id).
- Connect to the FR-MEMORY-115 detector outputs where present rather than duplicating them.

Acceptance:
- [ ] End-to-end dev run: seeded gate failure -> Reflector lesson -> Curator delta -> proposal visible in the ranked queue with evidence links.
- [ ] Dedup proven (same lesson twice produces one delta).
- [ ] Unit tests for Reflector extraction and Curator merge.

Touches: `modules/cuo/` (new reflector/curator modules), `modules/*/skills/`, `docs/auto-work/`.

---

### IMP-029: paired-trajectory skill audits

`refs: Stage 3 | prio: p1 | effort: m | deps: IMP-008 | area: cuo`

Context: a skill edit currently proves itself only by tests passing; the cheap stronger check is running old vs new skill on the same tasks and requiring non-regression.

Scope:
- Extend the dream gate for proposals touching `modules/*/skills/**`: execute the module goldenset twice (baseline skill, proposed skill) via the local model path; require new-pass ⊇ old-pass (no previously-passing case may fail) and report per-case diffs.
- Store the paired result in the proposal's audit row; surface in the morning digest.
- Deterministic settings (temperature 0, fixed seeds where the adapter allows) documented.

Acceptance:
- [ ] A deliberately degraded skill edit is rejected with the failing case named.
- [ ] A neutral edit passes and records the pairing evidence.
- [ ] Runtime per audit stays under the dream-cycle wall clock (600 s) for the seeded sets.

Touches: `modules/cuo/cuo/core/` (gate), `scripts/awh_ai_gate.sh` (shared runner hooks), docs.

---

### IMP-030: QLoRA fine-tuning pilot (obs triage)

`refs: Stage 4 | prio: p1 | effort: l | deps: IMP-008, IMP-009, IMP-021 | area: ai`

Context: the obs-triage assistant has a recorded fabrication failure (invented runbook URL) - a contained, measurable target for the first tuning pass; the alias map in ai-gateway is the deployment and rollback mechanism.

Scope:
- Data: curate 300-800 ChatML examples for triage (accepted triage notes from logs/sessions, synthetic negatives for fabrication, retrieval-grounded answers); store the curation script and dataset manifest (not raw PII) under `modules/ai/finetune/obs-triage/`.
- Train: QLoRA adapter on the current local 7-8B base (Qwen-class) via MLX on the Mac (or a documented rented-GPU recipe); training config committed, weights kept out of git (path + hash recorded).
- Evaluate: obs goldenset + the IMP-021 triage rubric, adapter vs base; the fabrication regression case must flip to pass; report table committed.
- Deploy: only on eval win, point a dedicated alias (`obs.triage`) at the adapter in the ai-gateway model map; rollback = alias revert; monitor a week of ledger rows (IMP-009) for latency/cost drift.

Acceptance:
- [ ] Dataset manifest, training config, and eval report merged; weights hash recorded in the ledger.
- [ ] Adapter beats base on the rubric and passes the fabrication case.
- [ ] Alias flip + revert both exercised once with the operator.

Touches: `modules/ai/finetune/`, `services/ai-gateway/` (model map entry), `docs/verification/`.
