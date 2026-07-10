# {{PROJECT}} FR backlog

Source of truth for FR state = each FR's frontmatter `status`. This file indexes them.
ONE backlog for ALL work: net-new features (`class: product`, the default) and
hardening/refactor/audit-remediation (`class: improvement`) live here together —
improvement is not a separate track and never gets a second backlog file. Tag
improvement rows with `(improvement)`; untagged rows are product.

FR files live under `docs/feature-requests/`: flat (`FR-001-slug.md`) for small
repos, or grouped in subfolders by module for larger ones. `improvement/` is a
normal subfolder there for cross-cutting hardening FRs.

The `ship-feature-requests` workflow reads this file, picks the first eligible FR
(`ready_to_implement` with all `depends_on` done), and drives it through the
lifecycle. HITL is required: the agent halts at review acceptance and final
acceptance for a recorded human verdict, and never sets `done` itself.

Lifecycle: draft -> ready_to_implement -> implementing -> ready_to_review -> reviewing ->
ready_to_test -> testing -> done. Off-ramps: on_hold, closed. See
`.cyberos/cuo/STATUS-REFERENCE.md`.

## ready_to_implement

- (none yet - add rows as `- [ready_to_implement] FR-001-slug - title`; append `(improvement)` for hardening FRs)

## in flight

- (implementing / reviewing / testing FRs appear here)

## done

- (shipped FRs, for the audit trail)

## on_hold / closed

- (deferred or killed FRs)
