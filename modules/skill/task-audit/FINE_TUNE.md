# `task-audit` — fine-tune discipline override

Default discipline at `../docs/FINE_TUNE.md`. This file documents the **task-audit-specific overrides**.

## Why task-audit is different

`task-audit` ships with `audit_rubric@2.0` ported verbatim from the proven legacy `cuo/cpo/task-audit` skill (2026-02 vintage). The rubric was battle-tested against 50+ FRs in `cyberos/docs/tasks/` before being ported. Bumping it requires extra scrutiny because:

1. The rule IDs (FM-001..111, SEC-001..009, COND-001..004, QA-001..009, SAFE-001..004, STALE-001) are referenced by **active audit reports across the cyberos project** + are baked into the FR catalog's `audit_score: 10/10` claim made in 50+ FR documents. Renames are catastrophic.
2. The EU AI Act decision-tree rules (QA-001..003) are compliance-relevant; loosening them creates regulatory exposure.
3. The task-author + task-audit pair is the cyberos project's primary feature-tracking workflow. Audit instability shakes confidence in every FR's status.

## Locked behaviour

The following are LOCKED until a major version bump with full governance review (CPO + CSecO + CLO):

- All FM-100s field enums (`status`, `priority`, `ai_authorship`, `feature_type`, `eu_ai_act_risk_class`, `client_visible`).
- The full SEC-001..009 required-sections list.
- The QA-001..003 EU AI Act decision-tree rules (these encode legal positions).
- The SAFE-003 injection-marker scan list (changing the marker set requires CSecO sign-off).
- The XCHAIN-001/002 chain hashes (changing them breaks chain-of-custody for the existing FR catalog).

## Permitted minor changes

| Change | Bump | Reviewer |
|---|---|---|
| Add a new QA-* anti-pattern rule (e.g. for new fabrication failure modes discovered in operation) | minor (`2.0 → 2.1`) | CPO |
| Tighten an existing QA-* warning to `error` | minor | CPO |
| Add a new conditional COND-* trigger (e.g. new compliance regime) | minor | CPO + CLO |
| Add a new injection-marker pattern to SAFE-003 | minor | CSecO |
| Editorial-only (wording, examples) | patch (`2.0 → 2.0.1`) | self-approve |

## Forbidden without major version bump

- Renaming ANY rule ID.
- Removing ANY rule.
- Changing the FM-100s field enums.
- Changing the QA-001..003 EU AI Act position.
- Changing the cross-skill XCHAIN-001/002 contract with task-author.

## Acceptance regression requirement

Every minor bump SHALL add:

1. A fixture FR file under `acceptance/golden-v<new>-<rule-id>-input.md` that triggers the new/changed rule.
2. The expected audit report at `acceptance/golden-v<new>-<rule-id>-input.audit.md`.
3. A regression test that exercises the new rule against the prior pass-clean fixture (must still pass — no false positives).

## Blackout windows

- **Audit week** — last week of each quarter where external SOC 2 / ISO 27001 evidence is gathered. No rubric changes during this window unless emergency.
- **Annual rubric review** — first week of each fiscal year. Rubric updates batched here when not time-sensitive.

## Cross-references

- `RUBRIC.md` — the locked rubric body.
- `../docs/FINE_TUNE.md` — master default discipline.
- `./task-audit skill` — the operator-side authoring discipline this rubric audits (sibling file; absorbed 2026-05-18 from the former `cyberos/task-audit skill`).
