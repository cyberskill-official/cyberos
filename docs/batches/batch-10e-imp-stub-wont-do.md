---
batch: batch/10e-imp-stub-wont-do
recorded: 2026-07-25
actor: Stephen Cheng (session operator)
---

# batch/10e — close platform/product stubs as won't-do for CyberOS 1.x

**Operator session override (continuum):** auto-approve/accept; pause only for real decisions.

**Ruling applied:** CyberOS 1.x is the **payload** (install, skills, workflows, docs tooling). Platform `services/*` / go-live tracks / research pilots are out of 1.x scope per `docs/tasks/_audits/POST-1.0.0-IMPROVEMENT-BACKLOG.md` §9.1 ("0 of 67 are pre-1.0.0 blockers") and the operator instruction to ship what is still valid for 1.x, closing the rest with reasons (no pause needed when the fork is clear).

## Closed this PR (39) — `draft → closed`

| Theme | IDs |
|---|---|
| Ops / deploy / obs | 004, 005, 006, 016, 017, 018, 019, 049, 050 |
| Eval / LLM / dream research | 009, 010, 020, 021, 027, 029, 030 |
| Platform refactors | 031, 032, 035, 036, 037 |
| Cloud AI | 033 |
| Security / data platform | 038, 042, 045, 051, 052, 053, 054 |
| Perf / soak / build | 039, 040, 048 |
| Product / frontend | 007, 057 |
| Go-live tracks | 063, 064, 065, 066, 067 |

Each spec carries a `## Groom note (batch/10e, 2026-07-25)` with the won't-do reason.

## Still draft — valid for 1.x (author + ship next)

| Theme | IDs | Notes |
|---|---|---|
| Supply-chain / CI | 001, 003, 043, 044 | cargo-audit/deny, secret scan, SBOM/sign, dependabot |
| Prod boot hardening | 002 | CORS refuse (if boot surface still exists) |
| Gate tooling | 008, 011, 012, 013, 022, 026 | goldensets, taxonomy, coverage ratchet, contracts, assert ban, auto-revert |
| Security docs | 041, 046, 047 | secrets inventory, backup/restore, rebuild-60 runbooks |
| Docs / governance | 055, 056, 058, 059, 061, 062 | module manifest, API versioning, ADR, wiki links, BRAIN consent, quarterly ritual |

## Already handled elsewhere

| Action | IDs | Where |
|---|---|---|
| duplicate/closed | 014, 015, 023, 024, 025, 034, 060 | PR #143 |
| closed | 028 | earlier → duplicate_of IMP-110 |
| ship | 125 | PR #144 |
| ship | 127, 128, 129 | PR #145 |
| ship | 124 | PR #146 |

## IMP-122 investigation (operator Q3)

**Outcome: UNBLOCKED — remain on hold until #145 merges; then unpark and ship (IMP-127).**

| Check | Result |
|---|---|
| Defect still present on main? | **Yes.** `version.sh` / `update-check.sh` / `audit-fleet.sh` still `grep` stored `rules_sha`; no `sha256sum`/`shasum` recompute. `build.sh` cone still `find cuo plugin mcp cli memory`. |
| Subsumed by IMP-127? | **No.** IMP-127 §Relationship: complementary — 127 makes the *build* read from git (reproducible fingerprint); 122 makes the *installed* side recompute (detect post-install drift) and fixes the cone to the vendored set. |
| Blocker? | Digests pinned in AC 10 must be **re-measured** after IMP-127's git materialise lands (cone content changes). Implement on `batch/10c` tip or after #145 merges. |
| Decision pause? | **None.** Spec is fully authored; path is clear. |

## Decision pauses

**None** for this batch. Go-live tracks and cloud routers closed as won't-do for 1.x without pause (clear product/platform fork vs payload).

## Operator merge order (updated)

1. #143 groom (7 close/dup)
2. #144 IMP-125
3. #145 IMP-127/128/129
4. #146 IMP-124
5. **This PR (#147 / batch 10e)** stub won't-do closes
6. IMP-122 ship PR (next — rebase onto #145)
7. Remaining ~20 valid-1.x stubs (author + ship by theme)
