# AGENTS.md — operating rules for AI agents in this repository

This repo ships AUDIT.md, an agent audit protocol, plus the machinery that improves it. You are either auditing a TARGET repo with it, or improving the protocol itself. Identify which job you were given before acting.

## Engineering standards (all CyberSkill projects)
- **Node.js >= 24 is the floor.** Every CyberSkill repo — this one, audited targets, and CI — runs on Node 24 or newer. Pin it locally with `.node-version` and `engines.node: ">=24"` in package.json; in GitHub Actions set `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24: true` at the workflow level so SHA-pinned actions run on the Node 24 runtime. Treat a Node < 24 toolchain as a finding.

## Job A — run the auditor on a target repo
1. Read `AUDIT.md` fully. Fill its CONFIG block from the user's answers; default to `MODE: gated` unless told otherwise.
2. Execute from PHASE 0 in the TARGET repo. Your only writes there are `docs/BACKLOG.md`, `docs/HANDOFF.md`, and approved task implementations.
3. Honesty gates you will be held to: every measured metric needs the literal command + raw output (R1); targets are cited-with-URL or `INTERNAL TARGET` (R2); statuses come from closed sets (R5); secrets are `[REDACTED:<kind>]` (R8). `python3 core/evals/validate.py --run <target>` checks your artifacts.

## Job B — improve the protocol (self-improvement cycle)
Follow `core/improve/CRITIC.md` step by step. Summary of its non-negotiables:
- ONE protocol change per cycle/version. Cite a trigger (failure-log row, retro item, or eval gap). Speculative edits are rejected.
- New rules require a recurred failure (Rule of Three). PATCH-level clarity fixes may proceed on first concrete evidence.
- Gate: `python3 core/evals/validate.py --all` green BEFORE release. If your change is testable, add/extend a fixture in the SAME cycle and register it in `core/evals/rules.json`.
- Release: bump version in AUDIT.md's title line → copy to `core/improve/versions/AUDIT-v<x.y.z>.md` → CHANGELOG.md entry → retro in `core/improve/retros/`.
- Campaign stop: 2 consecutive cycles with zero findings >= High. No lifetime cap on future campaigns.

## Hard invariants (both jobs — violating these is never acceptable)
- Never edit or delete anything in `core/improve/versions/` (immutable history).
- Never weaken, delete, or "adjust" an eval fixture to make a change pass. If a fixture is genuinely wrong, fixing it IS the cycle's one change.
- Never hand-edit `core/evals/baseline.json`; use `./core/evals/run-evals.sh --record`.
- Never remove failure-log rows; mark them promoted/deferred.
- Never commit secrets; this repo's outputs must satisfy R8 like any other.
- `docs/` in THIS repo holds committed documentation (CONTRIBUTING, SECURITY, COMPLIANCE). Self-run artifacts are gitignored BY NAME (`docs/BACKLOG.md`, `docs/HANDOFF.md`, `docs/AUDIT-WAIVERS.yaml`) — do not fight the gitignore, and do not commit self-run output.
- Docs follow changes, in the SAME commit. Anything a change makes stale — versions, counts, flags, file lists, behavior described in README / index.html / core/evals/README / core/improve/README / this file — gets corrected as part of that change. The mechanical subset (version + fixture-count surfaces) is enforced by `core/evals/scripts/check-docs-sync.py`, which CI runs; the rest is on you.
- Leave no leftovers. Before pushing: delete temp/scratch files you created, remove dependencies and references your change orphaned, and update or delete anything your change made outdated. `git status` and a read of your own diff are the checklist.

## Verification commands
```bash
python3 core/evals/validate.py --all          # full regression suite (must be ALL GREEN)
./core/evals/run-evals.sh --record            # suite + pin baseline to AUDIT.md sha256
python3 core/evals/scripts/check-docs-sync.py # version + fixture-count surfaces agree
python3 -c "import json;b=json.load(open('core/evals/baseline.json'));print(b['audit_md_version'],b['all_ok'])"
```

## File map (nature-divided: root shell · core engine · site page · docs)
**Root (distribution + tooling shell):** `README.md` front door · `AGENTS.md` this file (root because agent CLIs auto-read it there) · `action.yml` GitHub Action (root by resolution rules) · `pyproject.toml`/`package.json` packaging · `LICENSE`/`NOTICE`. **`core/` (the engine — CyberOS-absorbable as a unit):** `AUDIT.md` protocol (current) · `CHANGELOG.md` history · `improve/` the loop — see `core/improve/README.md` for the one-screen map (CRITIC, RETROSPECTIVE, FAILURE_LOG, BLINDSPOTS, versions/, retros/) · `evals/` regression gate (validate.py shim → code_audit_validator.py, fixtures/, rules.json, baseline.json, scripts/, TESTING-PROTOCOL.md) · `schemas/` published contracts (report.v1, feedback.v1). **`site/` (community page):** `index.html` + `assets/` (CyberSkill design system; deployed by `.github/workflows/pages.yml`, which ships ONLY site/). **`docs/` (human documents):** `CONTRIBUTING.md` · `SECURITY.md` · `COMPLIANCE.md` · `CODE_OF_CONDUCT.md` (GitHub surfaces health files from docs/). **`.github/workflows/`:** evals CI gate · publish (PyPI via OIDC) · pages — all actions SHA-pinned.
