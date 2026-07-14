# CAF absorption design (2026-06-20)

## Goal

Absorb CyberSkill/code-audit-framework (CAF) and code-audit-field-data into CyberOS the way awh was
absorbed, so CyberOS can not only rerun tests independently (awh) but also audit code independently
(CAF). End state: every FR, and every project, goes implement (CUO) -> tests-pass gate (awh) ->
code-audit gate (CAF) -> local pass -> live pass. The 8-project trigger is deferred per the owner;
this document is the design only.

## What CAF actually is

Three executable pieces plus a self-protecting protocol, all in `code-audit-framework/core`:

1. `AUDIT.md` - a versioned (v1.5.0) audit protocol. An AI agent reads it and audits a target repo,
   writing `docs/BACKLOG.md` and `docs/HANDOFF.md` into the target. `run-audit.sh <target> [agent]`
   composes the kickoff prompt and optionally launches an agent CLI. This step needs an LLM.
2. `code-audit-validate` (`core/evals/code_audit_validator.py`, entry point `code_audit_validator:main`)
   - a DETERMINISTIC conformance checker over the audit artifacts. It enforces the machine-checkable
   subset of AUDIT.md: measured baselines must carry a verbatim fenced output (R1), targets must be
   cited or labelled (R2), DONE tasks may not touch protected areas (R3), valid statuses/severities/
   IDs (R5), blocked tasks need a root cause (R6), no unredacted secrets (R8), the handoff must cite a
   stop reason (P5) and a `Target health: PASS|FAIL` line (v1.5.0). Flags: `--run <dir>`,
   `--report json|sarif`, `--emit-feedback --run-id <id>`, `--fail-on any|Critical|High|Medium`,
   `--protected p1,p2`, `--all` (fixture suite). Exit 0 clean, 1 gated, 2 usage. No LLM.
3. `verify-target.sh <target>` - the Phase-5 TARGET HEALTH GATE. Reads the target's own RUN_COMMANDS
   (build / lint / typecheck / test) from `audit-profile.yaml` and runs each with a timeout, failing
   closed if any breaks. This is the executable half the validator cannot do. Its own FAILURE_LOG
   cites kymondongiap: "CI lint failure shipped because the target's RUN_COMMANDS were never run."

Self-protection: `rules.json` + `baseline.json` + `fixtures/` (B## bad, G## good) + `validate.py --all`
is the regression harness that blocks any change weakening a rule - the same "never weaken the gate"
principle as awh. `core/improve/` (CRITIC, RETROSPECTIVE, FAILURE_LOG, BLINDSPOTS, versions) is the
self-improvement loop. `code-audit-field-data` holds one `feedback@1` record per run on a client repo;
it is the calibration substrate that feeds that loop.

## CAF vs awh, and why both

awh answers "do the tests still pass" by rerunning them against a sealed baseline. CAF answers "is the
code actually sound" by auditing it and by running the target's own build/lint/typecheck/test. They are
complementary: awh catches test regressions; CAF catches the class of defect awh cannot see - a changed
data contract, a route that 404s, a lint/build break scoped to the wrong directory. The CCAF episode is
the proof: the V2 rewrite silently changed the Supabase field contract, 404'd /about, and broke the
build vs the dashboard's expectations. awh's test rerun would not flag any of those; CAF's target-health
gate and audit protocol would.

## Where caf-gate slots into ship-tasks

The workflow today: step 27 task-audit (post-impl closure), step 28 awh-gate (out-of-band
test rerun, GREEN required), then the testing->done flip. Insert caf-gate as a new step and make the
done-flip conditional on BOTH gates. As implemented, caf-gate is step 29 and the done-flip steps
renumber to 30/31 - the cuo runtime formats step ids as integers (`step{n:02d}`), so a fractional
step like 28.5 is not used:

- step 28  awh-gate       -> awh_gate_report (tests rerun GREEN)
- step 29  caf-gate       -> caf_gate_report (code audit clean)
- step 30  testing->done  -> condition: awh_gate_report.outcome == GREEN AND
                             caf_gate_report.outcome == CLEAN  (else route back to ready_to_implement,
                             routed_back_count += 1)

The caf-gate has a deterministic floor and an LLM half, mirroring how awh (deterministic rerun) pairs
with cuo authoring (LLM):

- Deterministic floor (no LLM, runs in the gate): `verify-target.sh modules/<m>` (target health:
  build/lint/typecheck/test pass) AND `code-audit-validate --run modules/<m> --fail-on High` against a
  sealed audit baseline (no new High/Critical findings). This alone catches the CCAF/kymondongiap class
  of regression and can run today.
- LLM half (authoring): `run-audit.sh modules/<m> <agent>` generates `docs/BACKLOG.md`/`HANDOFF.md`.
  Like cuo authoring it needs an executor (ANTHROPIC_API_KEY or a Cowork host LLM via --invoker brief).
  The deterministic validator then checks that artifact. Generating the audit is a workflow step;
  validating it is the gate.

MVP recommendation: wire the deterministic floor (verify-target.sh + code-audit-validate of a committed
audit) as caf-gate now; add the LLM audit-generation as a workflow authoring step in a later pass. This
gets the highest-value, no-key protection (target health) into the gate immediately.

## Vendoring plan (mirrors tools/awh)

- `tools/caf/` <- `code-audit-framework` at its pinned sha. Keep `core/` (AUDIT.md, evals/, improve/,
  schemas/), `pyproject.toml`, `action.yml`. Run the validator from source:
  `PYTHONPATH=tools/caf/core/evals python3 -m code_audit_validator --run modules/<m> --fail-on High`,
  or `pip install -e tools/caf` to expose `code-audit-validate`.
- Per-module `modules/<m>/audit-profile.yaml` carries that module's RUN_COMMANDS so verify-target.sh
  works (e.g. `cd services && cargo test -p cyberos-<m>` for Rust modules; npm/pytest for others).
- `tools/caf/field-data/` <- code-audit-field-data (records/, reports/, schemas/) as the calibration
  store; new CyberOS runs emit `--emit-feedback` records here.
- Per-module `modules/<m>/.caf/` holds the sealed audit baseline (the committed BACKLOG/HANDOFF +
  baseline.json the validator gates against), exactly parallel to `modules/<m>/.awh/`.
- `scripts/caf_gate.sh` (turnkey, mirrors awh_ai_gate.sh) and a `.pre-commit-hooks/caf-gate.sh` that
  fail closed when a module has an audit profile but no baseline.

## Decision points for the owner

1. caf-gate scope now: deterministic floor only (verify-target.sh + validate; no key, ship now), or
   also the LLM audit-generation (needs an executor). Recommended: floor now.
2. Executor for the LLM audit half, when added: self-hosted (LM Studio), external (ANTHROPIC_API_KEY),
   or Cowork host LLM (--invoker brief) - same choice made for cuo authoring.
3. Retirement of standalone CAF: after vendoring and self-containment are verified, archive
   code-audit-framework + code-audit-field-data the way awh was (tag + bundle + optional delete), with
   a RETIREMENT.md. The field-data feedback loop continues from inside CyberOS.

## Status / next steps (this absorption)

Done (2026-06-20, branch auto/awh-absorb, uncommitted):

1. Vendored `tools/caf/` (validator self-test 40/40 GREEN, no install) + `tools/caf/field-data/`.
2. `scripts/caf_gate.sh` - deterministic floor (verify-target.sh target health + code-audit-validate
   of a sealed `.caf/` audit when present), fail-closed. `scripts/caf_precommit_check.sh` - structural
   fail-closed (every gated module must declare a profile).
3. `audit-profile.yaml` for all 8 gated modules (ai, auth, proj, email, skill, chat, cuo, memory),
   RUN_COMMANDS mirroring the awh-green suites (Rust crates hop `cd ../../services`; cuo/memory run
   pytest in place; memory adds the cargo crate).
4. Wired into `ship-tasks.md` v2.1.0: `caf_gate_report` output, step 28.5 (caf-gate),
   step-29 done-flip now requires `awh GREEN AND caf CLEAN`, §10 outcome table + cross-refs updated.
5. `tools/caf/RETIREMENT.md` written.

Verified in-sandbox (no toolchain needed): scripts bash -n clean; validator `--all` 40/40 exit 0;
pre-commit check GREEN (8/8); verify-target.sh PASS on a good command, FAIL (exit 1) on a red one,
fail-closed (exit 2) when RUN_COMMANDS is absent; all 8 profiles parse to the expected commands.

Remaining (owner-run on a build machine, or a later pass):

1. Run `bash scripts/caf_gate.sh <module>` per module to confirm target health GREEN with the
   toolchain up (ai needs Redis at 127.0.0.1:6379). Then tighten each profile's RUN_COMMANDS to add
   the lint/build line noted in its DOMAIN_NOTES (e.g. `cargo clippy -- -D warnings`) once confirmed clean.
2. Generate + seal each module's audit at `modules/<m>/.caf/` via `tools/caf/core/evals/run-audit.sh`
   (the LLM half - needs an executor: ANTHROPIC_API_KEY, a self-hosted model, or the Cowork host LLM
   via `--invoker brief`). Until then the gate runs the target-health-only floor, which already catches
   the CCAF/kymondongiap class.
3. Commit on auto/awh-absorb; run the RETIREMENT.md procedure once a real FR has passed the caf-gate.
