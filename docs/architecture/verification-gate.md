---
title: Verification gate (awh) — CyberOS
source: website/docs/architecture/verification-gate.html
migrated: FR-DOCS-002
---

## The principle

An agent that writes the code, writes the tests, and grades its own work has no independent check. The gate separates the grader from the author: it runs `awh eval` outside the authoring session, reruns the module's real build and test, and compares the result against a baseline that is captured once and then locked read-only. The agent can change the code, but it cannot change the bar it is measured against.

## How it is wired

The gate attaches at four seams without refactoring the platform:

**1\. The CUO workflow.** `ship-feature-requests` gained step 28 `awh-gate` and step 29 `caf-gate` between the post-implementation audit (step 27) and the `testing -> done` flip (step 30). The flip is conditional on `awh_gate_report.outcome == GREEN AND caf_gate_report.outcome == CLEAN`; either RED routes the FR back to `ready_to_implement` per STATUS-REFERENCE section 1.3.

**2\. Pre-commit.** `.pre-commit-hooks/awh-gate.sh` reruns the changed module's gate. A golden set with no committed baseline fails closed (an ungated eval always exits 0, so it must never be used as a gate).

**3\. CI.** `.github/workflows/awh-gate.yml` runs the gate for every changed module on a pull request, and is marked required in branch protection so it is the merge gate to `main`.

**4\. The merge gate.** Because CI is required, `main` stays green by construction.

Concept mapping: an FR's section 1 cited tests become the golden set; the coverage gate becomes the awh eval; held-out acceptance seals `done`; results emit into the memory audit chain (proposed row kind `memory.awh_gate_result`, gated on protocol change P23 section 6).

## Per-module golden sets

Each gated module carries `modules/<module>/.awh/goldenset.yaml` whose weighted tasks call that unit's real build and test, plus a held-out acceptance task. The vendored tool lives at `tools/awh/`; the maturity ledger at `.awh/evolution-log.jsonl`. A golden set task looks like:
    
    
    tasks:
      - id: memory-module-suite
        cmd: "cd modules/memory && python -m pytest -q"
        weight: 3.0
        timeout_sec: 900
      - id: acceptance-fr-memory-116    # held-out, sealed read-only via awh lock
        cmd: "cd modules/memory && python -m pytest tests/core/test_consolidate_semantic_dedup.py -q"
        weight: 5.0
        timeout_sec: 300

## Running the gate

Install the vendored gate, then bootstrap every module in one command:
    
    
    pip install -e tools/awh
    bash scripts/awh_bootstrap_waves.sh          # capture baseline, seal, report green/red per module

Or one module at a time:
    
    
    awh eval modules/auth/.awh/goldenset.yaml --base-dir . --seeds 1 --out modules/auth/.awh/eval-baseline.json
    awh lock services/auth/tests --write-policy
    awh eval modules/auth/.awh/goldenset.yaml --base-dir . --seeds 1 \
        --baseline modules/auth/.awh/eval-baseline.json --max-regression 0.0

Helper scripts: `scripts/awh_gate_coverage.py` (platform coverage view), `scripts/awh_build_order.py` (dependency layers), `scripts/awh_goldenset_from_fr.py` and `scripts/awh_cited_fixups.py` (derive golden sets and audit or correct cited-test drift).

## Auto-evolve, defined

To avoid over-promising: every FR ships through the evidence gate plus eval, and each module carries a maturity ledger entry (`awh maturity`) that reports convergence as modules are added and re-verified. The eval blocks any regression against the recorded baseline. It is a verification substrate and a convergence tracker, not an autonomous self-improver. awh does not rewrite CyberOS on its own; it gates and measures, and humans plus the CUO workflow drive the changes.

Read the verdict with `awh maturity report --log .awh/evolution-log.jsonl`. A clean streak of three adoptions with a low recent change rate marks the tool READY.

## Coverage status

MEMORY and SKILL are green under the gate. CUO, AUTH, CHAT, PROJ, and EMAIL are staged and run by the bootstrap script. AI is now green too (its three failing tests were fixed and its golden set wired). The other modules are still draft (not yet implemented), so they are gated when they are built. Run `scripts/awh_gate_coverage.py` for the live table.

For the caf gate, all eight gated modules (AI, AUTH, PROJ, EMAIL, SKILL, CHAT, CUO, MEMORY) carry a `modules/<module>/audit-profile.yaml`; run `bash scripts/caf_gate.sh <module>` for one or `bash scripts/caf_precommit_check.sh` to confirm every gated module declares a profile. See `docs/verification/caf-absorption-design.md` and the local loop in `docs/verification/local-run-and-verify.md`.

## Retiring the standalone awh

The vendored copy at `tools/awh/` is self-contained: nothing in the gate machinery depends on the external repo. Once every roadmap module is green under the vendored gate, the standalone `auto-work-harness` can be retired. Archive first so history is never lost, then remove the working copy:
    
    
    cd ~/Projects/auto-work-harness
    git tag -a archive/pre-cyberos-absorb -m "archived after vendoring into cyberos"
    git push origin archive/pre-cyberos-absorb
    git bundle create ~/Projects/auto-work-harness.bundle --all
    # history preserved in the tag, on origin, and in the bundle; the working copy is now safe to delete

See `tools/awh/RETIREMENT.md` for the full checklist, including the fix that should reach the other repos that adopted awh before the standalone is frozen.
