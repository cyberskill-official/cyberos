---
id: TASK-IMP-079
title: "Docs-ship race — shared staging dir between deploy.yml and release.yml docs jobs; fixed by a single ship script with per-run staging + flock'd swap"
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
eu_ai_act_risk_class: not_ai
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed
client_visible: false
type: improvement
created_at: 2026-07-13T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: improvement
priority: p0
status: done
verify: T
phase: "Wave 6 - go-live (docs channel)"
owner: Stephen Cheng (CTO)
created: 2026-07-13
shipped: 2026-07-13
memory_chain_hash: null
related_tasks: [TASK-IMP-074, TASK-DOCS-003]
depends_on: []
blocks: []
source_pages:
  - "deploy run #127 docs job log 2026-07-13: extract green, then \"mv: cannot stat '/home/***/cyberos/apps/console/docs.new': No such file or directory\" - the same run's release-workflow docs job was green"
  - ".github/workflows/deploy.yml docs job + .github/workflows/release.yml docs job (pre-fix): byte-identical inline snippet, BOTH staging into the shared ~/cyberos/apps/console/docs.new"
  - "deploy/vps/deploy.sh: audited clean - no git clean, no rm on the console tree; the deploy job did not do it"
source_decisions:
  - "2026-07-13 Stephen: deploy #127 screenshot + 'fix then continue'."
language: bash (one shared script), YAML (two call sites)
service: tools/docs-site + .github/workflows
new_files:
  - tools/docs-site/ship.sh
modified_files:
  - .github/workflows/deploy.yml
  - .github/workflows/release.yml
effort_hours: 1.5
subtasks:
  - "tools/docs-site/ship.sh: per-run staging (docs.new.<run_id>.<attempt>), flock'd swap, 2h stale-staging sweep - DONE, simulated"
  - "deploy.yml + release.yml docs jobs call the script (inline snippets deleted) - DONE, YAML parses"
risk_if_skipped: "Stephen's standard release sequence (git push, then the tag push seconds later) triggers both docs jobs concurrently; one deterministically-timed interleaving fails the loser at mv (a red run on healthy content), and a worse one deletes the live docs dir while the replacement staging is already gone - the published site drops until the next successful ship."
---
## §1
1. Both docs jobs **MUST** ship through one implementation, `tools/docs-site/ship.sh` - the duplicated inline snippet is how the shared-staging race was born, and TASK-IMP-074's rules-to-channels principle applies to the shippers themselves.
2. Staging **MUST** be per-run unique (`docs.new.${GITHUB_RUN_ID}.${GITHUB_RUN_ATTEMPT}`, `local$$` fallback): no shipper can ever name, and therefore never delete, another's in-flight staging.
3. The swap (`rm -rf docs && mv <stage> docs`) **MUST** run under a remote `flock` on `~/cyberos/apps/console/.docs-ship.lock`: swaps serialize, the docs-absent window stays single-threaded and sub-second, last writer wins (all writers build the same main, so content is equivalent).
4. Abandoned staging dirs **MUST** be swept without endangering live ones: the flock'd section removes `docs.new*` entries untouched for 2h+ only - an in-flight extract continuously refreshes its dir mtime and completes in seconds. This also retires the legacy shared `docs.new` name on its first post-fix ship.
5. Transport invariants inherited unchanged: tar streamed over ssh (no runner tgz, no VPS /tmp - the scp era shipped a truncated archive), `set -euo pipefail`, size echo for the deploy log.

*Lean profile: one script + two call-site swaps; the race, the fix, and the sweep semantics are all machine-verified in-session by a two-racer simulation of the exact remote command shape.*

## §5 (run 2026-07-13)
- `bash -n ship.sh` PASS; both workflows YAML-parse post-edit. PASS
- Two-racer simulation (exact remote command shape, tiny payloads, shared console dir): both exit 0, final `docs` = last swap's content, no cannot-stat. PASS (pre-fix design loses one racer by construction - observed live in #127)
- Sweep semantics: pre-seeded stale `docs.new` (3h) swept; fresh foreign `docs.new.other.0` survived. PASS
- `flock` present in util-linux (verified in the ubuntu sandbox; the VPS is ubuntu - if ever absent, the step fails loud at "command not found", not silently).
- Testing pass 2026-07-13 (post gate-1 "approve all"): two-racer simulation re-run green (both exit 0, last swap wins, sweep semantics intact); ship.sh bash -n + both workflows YAML re-verified.

## §9
- The docs-absent window during the swap (sub-second, now serialized) is accepted, as it was pre-fix. A symlink-flip scheme (docs -> release dir, atomic `ln -sfn`) would remove it entirely; adopt only if a monitoring blip ever attributes to it.
- deploy.yml's docs job and release.yml's docs job still both exist by design (TASK-DOCS-003: tags refresh the site too); deduplicating the JOBS is out of scope - this task dedupes the shipper.

## §10
| Failure | Detection | Recovery |
|---|---|---|
| two shippers race (the #127 event) | none needed - per-run staging + flock make it a supported case | last swap wins; both green |
| shipper killed mid-extract | orphan staging dir on the VPS | swept by the next ship's 2h sweep |
| flock missing on a future VPS image | step fails loud: command not found | install util-linux / adjust script |
| sweep window vs a >2h extract | impossible in practice (18M extracts in seconds); would surface as a swept-staging mv failure | raise -mmin threshold |
| release.yml tag ref predates ship.sh | docs job fails loud at "no such file" | re-tag current main (the standard sequence) |
*End of TASK-IMP-079.*
