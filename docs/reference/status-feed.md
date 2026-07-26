---
title: status-feed@1 · CyberOS
task: TASK-DOCS-028
---

# status-feed@1

Versioned JSON contract for the CyberOS status page (status-hub@3). Produced only by
`tools/docs-site/render-status-hub.mjs` (via `status-feed.mjs`). Consumed by the
tabless client in `modules/templates/html/status-app.js`.

Machine schema (tests): `tools/docs-site/tests/status-feed.schema.json`.

## Where it appears

| Surface | Path / slot |
| --- | --- |
| Embedded in the page | `<script type="application/json" id="sv3-data">…</script>` |
| Tooling sidecar | `docs/status/data/status-feed.json` (or `reference/data/` in site builds) |
| Spec chunks (lazy) | `docs/status/data/task/<TASK-ID>.js` — not part of the feed; linked via `specDir` |

Markdown under `docs/tasks/**`, root `CHANGELOG.md`, and `VERSION` remain the record of
truth. The feed only renders them.

## Identity and provenance

| Field | Meaning |
| --- | --- |
| `feed` | Contract major: always `1` for this document |
| `project` | Page title (repo basename; mothership forces `CyberOS`) |
| `version` | Contents of root `VERSION` |
| `snapshot` | Short date of coverage tip (`YYYY-MM-DD`) |
| `head` | Short sha at render time |
| `coverageAsOf` | Same tip as `head` — disclosed in the UI as **coverage as of parent &lt;sha&gt;** (a commit that stages the page cannot include its own hash; CI/deploy closes the gap) |
| `branch`, `repoUrl`, `tags` | Git identity; `repoUrl` from `origin` (ssh/https → https form), empty when absent |
| `fp` | `fp-` + 12 hex of sha256 over the feed JSON (without `fp` itself) |
| `rule`, `enforcement` | Verbatim RULE + active enforcement mode string |
| `noGit` | Present when git history was unavailable (empty coverage; page still renders) |

Page-only enrichments on `#sv3-data` (not required in the sidecar JSON): `commit`
(corpus fingerprint stamp), `specDir`, `frBase`, optional ops reports.

## Corpus

| Field | Content |
| --- | --- |
| `tasks[]` | `i` id, `k` folder key, `t` title, `m` module, `c` type, `p` priority, `s` status, `b` bucket (`done`/`active`/`hold`/`draft`), `ph` phase, `pg` phase group, `o` owner, `cr`/`sh` dates, `e` effort, `d[]` depends_on, `bl[]` blocks, `rl[]` related, `sm` summary |
| `modules[]`, `medges[]` | Per-module counts + layout coords; cross-module edges with weights |
| `phases[]` | P0–P5 / PRE / TRACK / POST lanes with spans and counts |
| `releases[]` | Notes: Added→`features`, Fixed→`fixes`, else→`improvements`; `cov` classified commits; `rc` marker hash; `lg` first-epoch fold |
| `unreleased` | Staged notes + pending `cov` |
| `burnup` | Cumulative shipped-by-day `{d,n}` |

## Commit classification (exact order)

For each commit in git log (newest → oldest):

1. Canonical `TASK-[A-Z]+-\d+` in subject+body → linked (`via` omitted or `id`)
2. Shorthand `PREFIX-NNN` (and slash lists) → linked only if the task exists (`via: shorthand`)
3. Reviewed ledger `docs/tasks/_state/commit-links.yaml` → linked (`via: ledger`)
4. Exempt: `chore(release):…`, `chore(web): rebuild…`, merges, reverts, fixups/squash/amend, `[skip ci]`
5. Else → **unlinked** (violation)

Epoch bucketing uses `chore(release): X.Y.Z` markers; a `chore(release): roll back` folds older history into a first-epoch entry (`lg: 1`).

## Build validation

Honest failures (or WARN in `CYBEROS_HUB_LENIENT=1`):

- Unknown task status values (strict fail optional via env)
- Dangling `depends_on` above `CYBEROS_FEED_GHOST_MAX` (ghosts always reported)
- Ledger entries citing unknown tasks
- Malformed changelog release headers (via note parsing)

The **page** stamp (`commit` / `fp-` on the HTML) fingerprints task specs, batches,
CHANGELOG, VERSION, ledger, manifest, and primary templates — not the live commit-set
(avoids the HEAD chase). Feed `fp` covers the emitted feed body including coverage.

## Versioning policy

- **`feed: 1`** is the only supported consumer contract today.
- Additive optional fields (`coverageAsOf`, `ghosts`, `noGit`, …) may appear without
  bumping `feed` when old clients ignore unknown keys.
- Breaking renames, required-field removals, or semantic changes to classification /
  bucketing require `status-feed@2` (new `feed` integer) and a coordinated client cut.
- Generators must remain deterministic: same inputs → byte-identical JSON (stable key
  order from the builder; double-run equality is gated in tests).

## Related

- Traceability runbook: [`docs/runbooks/traceability.md`](../runbooks/traceability.md)
- Generator: `tools/docs-site/render-status-hub.mjs`, `tools/docs-site/status-feed.mjs`
- Legacy escape hatch: `status-legacy.html` (status-hub@2 lenses) for one minor cycle;
  `CYBEROS_STATUS_LEGACY=1` makes it primary
