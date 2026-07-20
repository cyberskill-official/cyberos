---
batch: batch/7-outer-loop-and-economics
members: [TASK-IMP-110, TASK-IMP-114]
started: 2026-07-17T09:40:04Z
ended: 2026-07-17T10:13:03Z
route_backs: 0
gate_reasks: 0
tokens: unknown
---
# batch 7 - the outer loop and its own economics

§11d's first real row, written at the close of the batch it describes rather than reconstructed later. TASK-IMP-114 shipped in this batch and its ledger is this file: the feature's first exercise is itself.

## Why each field says what it says

Both instants are UTC. The first cut mixed notations - `started` carried git's `+07:00` local offset and `ended` came from `date -u` with a `Z` - which parses correctly and renders the right 33m, but reads as though the batch ended before it started. A ledger whose two timestamps use different clocks invites a reader to distrust the one number the row exists to carry. (External review, 2026-07-17.)

`route_backs: 0` and `gate_reasks: 0` are MEASURED, not assumed. Neither member routed back; each acceptance gate was asked once and answered once. §11d says an unrecorded field reads `unknown` and never `0` - these are recorded, and they are zero. That distinction is the whole point of the rule, so it is worth being explicit that these are the honest kind of zero.

`tokens: unknown`. The harness reported the two sub-agents' spend (233,563 and 223,002) but not the parent's, and the parent did the batch selection, the full gate suite, four reconciles, and both gate presentations. Recording 456,565 would name a number that is precise, verifiable, and wrong for the field it sits in - the batch cost more than that and I cannot say how much. §11d already says a number nobody measured must not be asserted; a number measured for the wrong scope is the same lie with better sourcing. When the harness reports parent spend, this becomes a real figure.

`shipped` is absent by design: the page derives it from each member's own frontmatter, because `status` already IS "did this task ship". Copying it here would create a second place for it to disagree with the first.

## What this batch actually cost, in the terms §11d cannot yet hold

Both members went through both gates without a route-back. The batch's real cost was not in the members: it was in what the run FOUND while shipping them - four defects, all of them mechanisms missing under promises that were already written down.
