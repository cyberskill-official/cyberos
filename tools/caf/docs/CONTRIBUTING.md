# Contributing

This repository ships **AUDIT.md** (an agent audit protocol) plus the machinery that improves it. Its governance is unusual on purpose: the protocol may only change through its own self-improvement loop. Read `AGENTS.md` first — it is the contract both humans and AI agents are held to.

## The two kinds of change

**1. Protocol changes (`AUDIT.md`)** — go through `core/improve/CRITIC.md`, one change per cycle, every time. No exceptions, including "obvious" fixes.

- Cite a trigger: a `core/improve/FAILURE_LOG.md` row, a retro item, or an eval gap. Speculative style edits are rejected.
- New rules need a recurred failure (Rule of Three). PATCH-level clarity fixes may proceed on first concrete evidence.
- If the change is testable, add or extend a fixture in the **same** cycle and register it in `core/evals/rules.json`.
- Release ritual: bump the title version → snapshot to `core/improve/versions/AUDIT-v<x.y.z>.md` → `CHANGELOG.md` entry → retro in `core/improve/retros/` → `./core/evals/run-evals.sh --record` → regenerate the social card (`python3 site/assets/make-social-card.py` — it reads version and fixture count itself; re-upload in Settings → Social preview) → tag `v<x.y.z>` and move the floating `v1` tag to the release commit (consumers pin the Action and pipx installs to it).

**2. Infrastructure changes (everything else)** — validator, fixtures, CI, docs, product page. These land as ordinary commits and may be batched, but the eval suite must stay green and fixture expectations may only be strengthened.

## Hard invariants (PRs violating these are closed)

- Never edit or delete anything in `core/improve/versions/` (immutable history).
- Never weaken, delete, or "adjust" an eval fixture to make a change pass. If a fixture is genuinely wrong, fixing it IS the cycle's one change.
- Never hand-edit `core/evals/baseline.json`; use `./core/evals/run-evals.sh --record`.
- Never remove `core/improve/FAILURE_LOG.md` rows; mark them promoted/deferred.
- Never commit secrets. This repo's own artifacts must satisfy R8.
- Keep AUDIT.md under 200 lines; if your change adds net rules, say what you trimmed to pay for it.
- Docs follow changes, in the same PR: update every doc your change makes stale (README, index.html, core/evals/README, core/improve/README, AGENTS.md). `python3 core/evals/scripts/check-docs-sync.py` enforces the version and fixture-count surfaces and runs in CI.
- Leave no leftovers: temp files, orphaned dependencies, and outdated references introduced by your change are part of your diff to clean.

## Before you open a PR

```bash
python3 core/evals/validate.py --all     # must end ALL GREEN
./core/evals/run-evals.sh --record       # if (and only if) AUDIT.md changed
```

CI enforces: the full fixture suite, version sync across AUDIT.md / package.json / baseline.json, baseline sha256 integrity, and that the current protocol matches its `core/improve/versions/` snapshot.

## Reporting protocol failures (most valuable contribution)

The loop runs on evidence. If an agent run satisfied AUDIT.md's letter while violating its intent, that observation is worth more than code: open an issue titled `failure: <one line>` with the protocol version, the model/agent used, the artifact excerpt (BACKLOG/HANDOFF — redact secrets per R8), and what you expected. Two independent observations of the same family meet the promotion bar and become a protocol change.

## Security

See `SECURITY.md`. Do not open public issues for vulnerabilities.

## License

By contributing you agree your contributions are licensed under the Apache License 2.0 (see `LICENSE`).
