# Traceability runbook

**Task:** TASK-DOCS-028 (enforcement surfaces: TASK-DOCS-009 / 019 / 020)  
**Rule:** Every change introduced by a commit must be linked to one or more tasks and
reflected on the Status page and in the Release Notes. No exceptions.

Git history is never rewritten to “fix” links. Retroactive recovery uses the reviewed
ledger only.

## Surfaces

| Surface | Entry | Default posture |
| --- | --- | --- |
| Local hook | `.githooks/commit-msg` (mothership) or consumer `commit-msg` → `.cyberos/lib/check_task_link.sh --msg` | Advisory warn; blocks when strict |
| CI | `.github/workflows/traceability.yml` (mothership); consumer template under `tools/install/ci/traceability/` (opt-in) | Hard-fail on PR/push range |
| Release | `bash scripts/check_task_link.sh --range <last-tag>..HEAD` (see `docs/deploy/RELEASE.md`) | **Warn** in `release.yml` today; blocking at v2.0.0 / Phase 6 |

Single implementation: `scripts/check_task_link.sh` (vendored to `.cyberos/lib/check_task_link.sh` on install).

Canonical citation: full id, e.g. `TASK-TEN-208` (or a `Task: TASK-TEN-208` trailer).
Shorthand like `TEN-208` is recovered on the status page but **does not** satisfy the gate.

## Cutoff policy

Resolution order:

1. `CYBEROS_TRACE_CUTOFF` env (full or prefix sha)
2. `.cyberos/config.yaml` → `traceability.cutoff`
3. Script default (mothership seed) / value written at install

Semantics: commits **at or before** the cutoff are history — visible on the status page,
never failed by the gate. New installs and upgrades write `cutoff: <HEAD at install>` when
none is recorded so no repo starts with retroactive violations.

```yaml
# .cyberos/config.yaml
traceability:
  cutoff: <sha>
  strict: false          # true → local --msg blocks
  scaffold_ci: false     # true → install copies consumer CI workflow when .github/ exists
```

`scaffold_ci` defaults to **false** (opt-in).

## When the gate fails

### Local (`--msg`) advisory

You see a loud warning and the commit still lands (unless strict). Before pushing:

1. Amend or create a follow-up that cites the task id in subject or body.
2. Confirm the task exists under `docs/tasks/**` and will show on the status page after regen.

### CI / `--range`

```bash
bash scripts/check_task_link.sh --range origin/main..HEAD
# or for a release candidate:
last=$(git describe --tags --abbrev=0 HEAD)
bash scripts/check_task_link.sh --range "${last}..HEAD"
```

For each `UNLINKED` line:

1. **Prefer amend / rebase** to add `TASK-…` to the message (only on unpushed or
   explicitly allowed history).
2. **Split plumbing** into an exempt type (`chore(release):`, `chore(web): rebuild`,
   merge/revert/fixup, `[skip ci]` automation) when the commit truly is not a change.
3. **Do not** rewrite shared main history. If the commit is already on a protected
   branch and cannot be amended, use the ledger (below) for status-page truth and treat
   the CI failure as a process debt until the next cut allows a clean range.

### Strict mode

Blocks locally when any of:

- `CYBEROS_STRICT_COMMITS=1`
- `CYBEROS_REQUIRE_TASK_LINK=1`
- `traceability.strict: true` in config

## Ledger procedure (retroactive links)

File: `docs/tasks/_state/commit-links.yaml`

```yaml
# short-or-full sha: [TASK-ID, ...]
a1b2c3d4: [TASK-DOCS-028]
```

Rules:

1. Only for history that cannot or must not be rewritten.
2. Every task id must exist in the live corpus (unknown task → generator fail).
3. Unknown hash prefixes warn; they do not invent coverage for missing commits.
4. Land ledger edits via normal PR review — the file is the reviewed backfill path.
5. The status page classification ladder prefers canonical/shorthand first; ledger is step 3.

## Status page freshness

- Pre-commit: full regen on `docs/tasks/**` / `CHANGELOG.md` / `VERSION`; otherwise
  `--coverage-only` (git cov + stamp). Loud-warn, never block on mothership.
- Truth window: committed page says coverage as of **parent**; deploy/CI rebuild includes
  pushed commits on the served site.
- Manual: `bash .cyberos/lib/status-page.sh .` (or `tools/install/lib/status-page.sh`).

## Related

- Contract: [`docs/reference/status-feed.md`](../reference/status-feed.md)
- Release preflight: [`docs/deploy/RELEASE.md`](../deploy/RELEASE.md) (Traceability preflight)
- Checker: `scripts/check_task_link.sh`
