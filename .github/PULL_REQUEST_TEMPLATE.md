<!-- PR title format: feat(<scope>): <subject>  (Conventional Commits) -->
<!-- Reference the FR in the title or body: FR-AUTH-001 -->

## What

<!-- One-paragraph summary of the change. -->

## Why

<!-- Link to the FR. Quote the relevant Scope acceptance criteria covered. -->

Refs: FR-XXX-NNN

## How

<!-- Implementation notes. Schema changes? Cross-module touches? -->

## Compliance touch

- [ ] No personal data added without DPIA refresh.
- [ ] No compensation / equity / health / bank / government-ID fields added to BRAIN.
- [ ] If audit-relevant, audit events are emitted via `@cyberos/audit-events`.
- [ ] If AI surface changed, persona-version + skill-version updated and dual-signed.
- [ ] If destructive / financial action, step-up auth required at endpoint level.
- [ ] If user-visible, both `vi-VN` and `en-US` strings present.

## Tests

- [ ] Unit
- [ ] Integration
- [ ] Acceptance (Gherkin from FR Scope)
- [ ] Anti-regression suite passes locally

## Status discipline

After merge, update the FR's `status` frontmatter:

- `ready_for_review` → `in_implementation` when work starts
- `in_implementation` → `shipped` when this PR merges to main
- `shipped` → `closed` when retired
