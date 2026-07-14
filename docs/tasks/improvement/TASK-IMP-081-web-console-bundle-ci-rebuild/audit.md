---
task_id: TASK-IMP-081
audited: 2026-07-13
verdict: PASS (after revision)
score_pre_revision: 9/10
score_post_revision: 10/10
issues_resolved: 1
template: engineering-spec@1
---
- ISS-001: draft's `deploy` job added `rebuild-web` to `needs:` with no `continue-on-error`, relying on the job's own `if:` (gated on `web_src`) to keep it out of the way. That only covers the SKIPPED case cleanly. A genuine build failure inside `rebuild-web` (a real `tsc`/`vite` error on `main`, not just a blocked push) is a different job outcome - `failure()` - and `deploy`'s existing `if: always() && !failure() && !cancelled()` would then correctly-per-its-own-logic but wrongly-for-this-FR block the entire services roll for a completely unrelated `apps/web` breakage, since `services/` and `apps/web` are independent deployables. Resolved: `rebuild-web` set to `continue-on-error: true` (job still shows red in the Actions UI on a genuine failure; only its blast radius on the unrelated services pipeline is contained). §1 clause 7 and §10 added; re-validated with actionlint (clean) after the fix.
Score = 10/10.
