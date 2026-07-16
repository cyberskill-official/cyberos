---
id: TASK-IMP-070
title: "Remote update awareness - /update and install.sh --check compare installed vs latest published release, not the local payload"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: improvement
created_at: 2026-07-12T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: improvement
priority: p0
status: done
verify: T
phase: Wave A - version coupling
owner: Stephen Cheng (CTO)
created: 2026-07-12
shipped: 2026-07-12
memory_chain_hash: null
related_tasks: [TASK-IMP-068, TASK-IMP-069, TASK-APP-001]
depends_on: [TASK-IMP-069]
blocks: []
source_pages:
  - tools/install/install.sh
  - tools/install/plugin/commands/update.md
  - tools/install/plugin/commands/changelog.md
source_decisions:
  - "2026-07-12 investigation: with a stale local payload, install.sh --check reports installed=1.2.0 available=1.2.0 and /update says up to date while VERSION is 1.7.0. The available number must come from the published channel, not the laptop."
  - "No auto-update: the check only reports; applying an update stays an explicit operator action."
language: bash + markdown (command docs)
service: tools/install/
new_files:
  - tools/install/check-latest.sh
  - tools/install/tests/test_check_latest.sh
modified_files:
  - tools/install/build.sh   # deviation, recorded in review packet: vendor check-latest.sh beside install.sh
  - tools/install/install.sh
  - tools/install/plugin/commands/update.md
  - tools/install/plugin/commands/changelog.md
  - tools/install/README.md
---

# TASK-IMP-070: Remote update awareness

## §1 - Description

`install.sh --check` compares a target repo's `.cyberos/VERSION` against the LOCAL payload's VERSION, and the `/update` command trusts that answer. Once TASK-IMP-069 publishes every release, the honest comparison is against the latest published version. This task adds that third data point and rewrites the verdict logic around it.

Normative clauses:

1. A script `tools/install/check-latest.sh` MUST resolve the latest published version and print exactly one line `latest=<X.Y.Z|unknown> source=<url|offline>`. Resolution order: `$CYBEROS_RELEASE_ENDPOINT` when set (an https URL or a local file path returning either a bare `X.Y.Z` or GitHub's `/releases/latest` JSON, from which `tag_name` minus the `v` is taken), else the repo's public `releases/latest` page redirect (Location header names the tag; immune to API rate limits), with the `releases/latest` API URL as fallback (amended post-ship 2026-07-12 after a live 403 rate-limit). Total network budget MUST be <= 3 seconds (curl `--max-time 3`); any failure MUST yield `latest=unknown source=offline` with exit 0 - the script never breaks a caller.
2. `install.sh --check <repo>` MUST report three values - `installed` (`<repo>/.cyberos/VERSION`), `payload` (the local payload's VERSION), `latest` (via check-latest.sh, skipped when `CYBEROS_OFFLINE=1`) - followed by exactly one verdict line: `verdict=up_to_date` (installed == latest, or latest unknown and installed == payload), `verdict=repo_stale` (installed < payload or installed < latest), or `verdict=payload_stale` (payload < latest). `installed >= latest` MUST count as up to date - the check never advises a downgrade. Each non-clean verdict MUST print the exact next command (`bash <payload>/install.sh <repo>` for repo_stale; the TASK-IMP-069 download one-liner or `build.sh` for payload_stale).
3. Version comparison MUST be numeric semver (major, minor, patch), not string comparison.
4. `plugin/commands/update.md` MUST direct the agent to run the extended `--check` and act on the verdict: on `payload_stale`, fetch the latest release payload (or rebuild from a current checkout) BEFORE re-running init, and never report "up to date" from the local-payload comparison alone. On `latest=unknown` it MUST say the remote check was skipped and the answer is only as fresh as the local payload.
5. `plugin/commands/changelog.md` MUST, when `latest` is newer than `installed`, link the GitHub Releases page as the span to read (installed+1 .. latest), in addition to the local `.cyberos/manifest.yaml` details.
6. Offline behavior MUST be first-class: `CYBEROS_OFFLINE=1` (or network failure) degrades `--check` to today's local comparison plus an explicit `latest=unknown source=offline` note. Exit code semantics of `--check` (0 = ran) MUST NOT change.
7. Nothing in this task mutates a target repo; the check remains read-only and auto-update stays out of scope.

## §2 - Why this design

The three-value report makes the two failure directions distinguishable: a stale TARGET repo (fix: rerun init) versus a stale LOCAL payload (fix: download/rebuild), which today collapse into one misleading "up to date". A separate `check-latest.sh` keeps network code out of `install.sh`'s critical path, gives the desktop Ops tab (TASK-APP-001) and `/update` one shared resolver, and makes the endpoint overridable so tests run on `file://`-style fixtures with zero network.

## §3 - Contract

```
check-latest.sh
  env: CYBEROS_RELEASE_ENDPOINT (optional), CYBEROS_OFFLINE=1 -> immediate unknown
  stdout: latest=1.8.0 source=https://api.github.com/repos/cyberskill-official/cyberos/releases/latest
      or: latest=unknown source=offline
  exit: always 0

install.sh --check <repo>   (extended output, order fixed)
  installed=1.2.0
  payload=1.7.0
  latest=1.8.0 source=<url>
  verdict=repo_stale
  next: bash <payload>/install.sh <repo>
```

## §4 - Acceptance criteria

1. **Endpoint override, bare version** (§1 #1) - with `CYBEROS_RELEASE_ENDPOINT` pointing at a fixture file containing `1.8.0`, the script prints `latest=1.8.0` with that source.
2. **Endpoint override, GitHub JSON** (§1 #1) - with a fixture containing `{"tag_name": "v1.8.0", ...}`, the script prints `latest=1.8.0`.
3. **Failure degrades, never breaks** (§1 #1, #6) - with an unreachable endpoint, output is `latest=unknown source=offline`, exit 0, and `--check` still completes with a verdict.
4. **Three values + verdict, all four states** (§1 #2) - fixtures produce each verdict: installed==payload==latest -> `up_to_date`; installed<payload -> `repo_stale` with the init command; payload<latest -> `payload_stale` with the fetch instruction; latest unknown + installed==payload -> `up_to_date` with the offline note.
5. **Numeric semver compare** (§1 #3) - 1.10.0 ranks above 1.9.0 (string compare would invert it).
6. **CYBEROS_OFFLINE honored** (§1 #6) - with `CYBEROS_OFFLINE=1`, no network attempt is made (endpoint fixture untouched) and the offline note appears.
7. **/update doc drives the verdicts** (§1 #4) - update.md names the three values, the two stale verdicts with their distinct actions, and forbids the local-only "up to date" claim.
8. **/changelog links the release span** (§1 #5) - changelog.md instructs linking the Releases page when latest > installed.

## §5 - Verification

```bash
# tools/install/tests/test_check_latest.sh
# Fixtures under $TMP: bare-version file, tag_name JSON file, unreachable path,
# plus a scratch repo + scratch payload for the --check verdict matrix.

t01_bare_version_endpoint()      # AC 1
t02_github_json_endpoint()       # AC 2
t03_unreachable_degrades()       # AC 3
t04_verdict_matrix()             # AC 4  (4 sub-cases, exact verdict + next line asserted)
t05_numeric_semver()             # AC 5
t06_offline_env()                # AC 6
t07_update_doc_contract()        # AC 7  (greps update.md for the three keys + both verdicts)
t08_changelog_doc_contract()     # AC 8
```

## §6 - Implementation skeleton

`check-latest.sh`: ~40 lines - endpoint resolution, `curl -sf --max-time 3` (or `cat` for a local path), parse bare/JSON, print. `install.sh --check`: keep existing reads, add the resolver call + a `ver_ge()` numeric comparator + verdict table from §3.

## §7 - Dependencies

Depends on TASK-IMP-069 (there must BE a published latest). TASK-APP-001's Check button inherits the richer output for free since it shells out to `install.sh --check`.

## §8 - Example payloads

```
$ CYBEROS_OFFLINE=1 bash dist/cyberos/install.sh --check ~/Projects/clientA
installed=1.6.0
payload=1.7.0
latest=unknown source=offline
verdict=repo_stale
next: bash dist/cyberos/install.sh /Users/stephen/Projects/clientA
```

## §9 - Open questions

None blocking. A cached last-known-latest under `~/.cache/cyberos/` was considered and dropped: a 3-second budget per explicit check does not need a cache, and a cache adds a staleness surface of its own.

## §10 - Failure modes inventory

1. GitHub API rate limit (60/h unauthenticated) - resolver returns unknown; verdict falls back to local compare with the offline note. Never a hard failure.
2. Endpoint returns HTML (proxy portal) - parse yields no semver; treated as unknown, not as a garbage version (regex-gated).
3. Pre-release tags (v1.8.0-rc1) - resolver takes `tag_name` as-is; non `X.Y.Z` strings are rejected by the same regex and reported unknown (pre-releases are not "latest" for updaters).
4. Clock-skew style confusion (installed NEWER than latest, e.g. developing ahead of the last tag) - `installed > latest` maps to `up_to_date` (never advise a downgrade).
5. Two payloads on one machine (repo checkout + downloaded release) - the report prints which payload path it compared, so the operator sees which source answered.

## §11 - Implementation notes

Keep the output keys machine-parseable (`key=value`, one per line) - TASK-APP-001 parses them, and future telemetry can too. update.md and changelog.md are agent-facing prose; keep the verdict names verbatim so agents can branch on them.

*End of TASK-IMP-070.*
