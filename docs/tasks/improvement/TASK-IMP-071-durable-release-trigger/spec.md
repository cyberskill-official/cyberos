---
id: TASK-IMP-071
title: "Durable release trigger - version bumps drop [skip ci] so tag pushes fire release.yml natively"
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
phase: Wave E - 1.0.0 hardening closeout
owner: Stephen Cheng (CTO)
created: 2026-07-12
shipped: 2026-07-12
memory_chain_hash: null
related_tasks: [TASK-IMP-068, TASK-IMP-069, TASK-DOCS-007]
depends_on: []
blocks: []
source_pages:
  - .github/workflows/version.yml
  - docs/deploy/RELEASE.md
source_decisions:
  - "2026-07-12 field finding (v1.8.0 era, re-confirmed at v0.3.0): tags pointing at [skip ci] bump commits never trigger release.yml; every release needed a manual dispatch. Queued then; closed now for the 1.0.0 readiness call."
language: yaml + markdown
service: .github/workflows/
new_files: []
modified_files:
  - .github/workflows/version.yml
  - docs/deploy/RELEASE.md
---

# TASK-IMP-071: Durable release trigger

## §1 - Description

1. version.yml's bump commit MUST NOT carry `[skip ci]`; the message stays `chore(release): vX.Y.Z`.
2. The workflow's own loop protection MUST rest solely on the existing job guard (`!startsWith(head_commit.message, 'chore(release):')`), documented as the single brake.
3. The TASK-DOCS-007 §1 #4c deploy-dispatch workaround MUST be removed (deploy.yml's VERSION path filter now fires natively on the bump push); the `actions: write` permission it needed goes with it.
4. TASK-IMP-068 §1 #7's inline proof MUST remain (belt-and-suspenders now that payload-gate also sees bump commits), with its comment updated; both affected shipped tasks carry post-ship amendment notes.
5. RELEASE.md MUST document the new model: `git push origin vX.Y.Z` triggers the release; dispatch remains the fallback/retry path.

## §2 - Why this design

[skip ci] suppressed MORE than the loop it guarded against - it silenced tag pushes and the docs deploy, costing a manual dispatch per release plus a dispatch workaround. The message-prefix guard was already the real loop brake; removing the blunt instrument leaves exactly one, documented mechanism.

## §3 - Contract

Bump commit: `chore(release): X.Y.Z` (no suffix). Loop brake: version.yml job `if` guard only.

## §4 - Acceptance criteria

1. **No [skip ci] anywhere live** (§1 #1) - grep on version.yml's commit line is clean; RELEASE.md describes the new model.
2. **Guard is sole + documented** (§1 #2) - the job `if` survives; comments name it the single brake.
3. **Workaround retired** (§1 #3) - no `gh workflow run deploy.yml` in version.yml; `actions: write` gone.
4. **Amendments recorded** (§1 #4) - TASK-IMP-068 + TASK-DOCS-007 specs carry the notes.

## §5 - Verification

Grep-level asserts (executable): `! grep -q 'skip ci]' .github/workflows/version.yml` commit line; `grep -q "single brake\|ONLY loop guard" version.yml`; `! grep -q "workflow run deploy" version.yml`; amendment greps on both specs. Live proof: the next bump push must show payload-gate + deploy runs on the bump commit, and the next `git push origin vX.Y.Z` must start release.yml without dispatch (operator-observed, recorded at the next release).

## §6 - Implementation skeleton

Three-line yaml change + comment updates + doc + two amendment notes.

## §7 - Dependencies

None. Interacts with TASK-IMP-068 (inline proof kept) and TASK-DOCS-007 (workaround retired).

## §8 - Example payloads

`chore(release): 0.5.0` -> payload-gate ✓, deploy(docs) ✓, version.yml skipped by guard; `git push origin v0.5.0` -> release run starts.

## §9 - Open questions

None blocking.

## §10 - Failure modes inventory

1. Guard weakened in a future edit -> bump loop; the comment names it the single brake and RELEASE.md repeats it.
2. A human commit starting `chore(release):` skips version.yml -> by design (that IS the release-commit namespace).
3. payload-gate red on a bump commit -> impossible-by-construction drift becomes visible instead of silent; exactly what the gate is for.
4. Rulesets block the bump push -> unchanged degrade-to-warning path.
5. Old tags on [skip ci] commits -> historical; dispatch fallback still cuts them.

## §11 - Implementation notes

Batch-mode ship under the operator's standing verdict; live-proof clause lands at the next release (operator observation recorded then).

*End of TASK-IMP-071.*
