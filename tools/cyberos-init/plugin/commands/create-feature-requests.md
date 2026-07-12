---
description: Create feature requests from a PRD, spec, or a plain idea - draft them, audit them against the rubric, and land them ready_to_implement in the backlog, ready for /ship-feature-requests.
argument-hint: "[path to a PRD/spec, or just describe the idea]"
---
Author and audit feature requests for this repo. Input = ${1:-ask the user for the PRD/spec path, or the idea to turn into FRs}. This command CREATES the backlog; it never implements code. `/ship-feature-requests` is what implements.

Run the two skills in order. Both are bundled with this plugin (`${CLAUDE_PLUGIN_ROOT}/skills/`) and also vendored at `.cyberos/cuo/skills/` once `/init` has run.

1. Author - `feature-request-author`.
   - If given a PRD / spec / SRS document, expand it into `feature_request@1` FR markdowns.
   - If given only an idea (no document), use the skill's standalone interview to elicit scope, then draft from that.
   - It HALTS at PLAN approval: show the user the proposed FR set (ids, titles, class) and get their go-ahead before writing files. Respect that halt - do not auto-approve on their behalf.
   - Write FRs to `docs/feature-requests/` (module subfolder in a monorepo, flat otherwise). Each starts at `status: draft`. Cross-cutting hardening work is `class: improvement`; everything else is `class: product` (the default).

2. Audit - `feature-request-audit`.
   - Audit every FR just drafted against `audit_rubric@2.0` (the FM / SEC / COND / QA / SAFE / TRACE rule families).
   - A clean audit drives the `draft -> ready_to_implement` transition per `STATUS-REFERENCE.md`. Write the sibling `.audit.md` per FR plus the batch summary.
   - It HALTS on any `needs_human` verdict. Surface those to the user and stop - do not guess the verdict.

3. Backlog. Delegate every row to `backlog-state-update-author` + `backlog-state-update-audit` - one `mutation_kind: insert-row` mutation per landed FR (batching per module section allowed). Never edit `BACKLOG.md` inline: the pair is the single audited write path (same one /ship-feature-requests uses), with regenerator-identical row grammar (`(improvement)` suffix on `class: improvement` rows) and a uniqueness gate. FR frontmatter `status` stays the record of truth; the backlog is the index and must match it.

4. Report. List each FR: id, title, class, final status, and the audit verdict. Then state the next move plainly: the FRs now at `ready_to_implement` are ready, and `/ship-feature-requests` will drive the next eligible one through implement -> review -> test, halting at the two human-acceptance gates.

Never set `done`, never push, merge, or deploy. If the repo has no `.cyberos/` yet, tell the user to run `/init` first.
