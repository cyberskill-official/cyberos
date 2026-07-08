# improvement — feature-request class index

Improvement FRs (`class: improvement`) are enterprise-hardening, refactoring, and audit-remediation work. They are NOT a separate track: each runs the full `ship-feature-requests` lifecycle with the mandatory human-acceptance gates, exactly like a product FR. This folder is the home and index for cross-cutting improvement FRs, and the migration record for the retired `docs/improvement/` backlogs.

See `modules/cuo/chief-technology-officer/workflows/ship-feature-requests.md` section 1a for the lifecycle and the gate profile.

## Where improvement FRs live

- Module-scoped hardening goes to `FR-<MODULE>-*` under `docs/feature-requests/<module>/` (memory hardening is `FR-MEMORY-*`, chat hardening is `FR-CHAT-*`, and so on). Same module index, same BACKLOG grooming as any product FR for that module.
- Cross-cutting hardening that spans modules (for example a repo-wide audit remediation) is tracked here as `FR-IMP-*`, with this README as the index.

`class: improvement` in the FR frontmatter is what marks an FR as hardening (the default is `class: product`). The class selects the gate profile (section 1a) and lets grooming and reporting separate hardening from net-new; it does not change the lifecycle or the two human-acceptance gates.

## Migration from the retired docs/improvement backlogs

The old separate improvement programs are folding into FRs. Current status:

| Old program | Old ids | New home | Status |
|---|---|---|---|
| memory enterprise (`docs/improvement/memory`) | `MEM-001..060` | `FR-MEMORY-*` (`class: improvement`) | pending migration; in-flight on `auto/memory-enterprise` (5 tasks in review) — migrate after that branch merges |
| chat enterprise (`docs/improvement/chat`) | `T-001..066` | `FR-CHAT-*` (`class: improvement`) | pending migration; in-flight on `auto/chat-enterprise` |
| deep audit (`docs/improvement`) | `IMP-001..062` | `FR-IMP-*` (cross-cutting, here) | pending migration |

Until a program migrates, its existing backlog under `docs/improvement/<program>/` remains the source of truth for that in-flight work. Do not double-track a task in both places.

### Open decision (operator)

Two choices gate the physical migration, and both are the operator's:

1. Id scheme. Keep the old ids as a sub-namespace (for example `FR-MEMORY-2xx` carrying `legacy_id: MEM-012`) or renumber cleanly to the next free FR number. Recommendation: keep `legacy_id` on each migrated FR so commit and ledger history stays traceable.
2. Timing. Migrate each program only after its in-flight branch (`auto/memory-enterprise`, `auto/chat-enterprise`) merges, so live `MEM-*` and `T-*` references in commits and ledgers are not invalidated mid-flight.

Once decided, migration is mechanical: convert each backlog row into an FR spec with `class: improvement` (and `legacy_id`), drop it under the right module (or here for cross-cutting), delete the old `docs/improvement/<program>/`, and regenerate `docs/feature-requests/BACKLOG.md` with the grooming script.
