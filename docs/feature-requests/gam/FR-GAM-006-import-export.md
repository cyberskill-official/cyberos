---
id: FR-GAM-006
title: "Import and export of aliases"
module: GAM
priority: SHOULD
status: done
fidelity: as-built
shipped: 2026-06-23
owner: Stephen Cheng
source_repo: zintaen/gam @ f55d97c
related_frs: [FR-GAM-001]
---

## §1 — Description (BCP-14 normative)

gam SHOULD let users move their alias set between machines.

1. The app SHOULD export the current alias set to a portable form.
2. The app SHOULD import an alias set from that form.
3. Import MUST apply the same name validation (`^[a-zA-Z][\w-]*$`) and duplicate rules as FR-GAM-001; it MUST NOT create invalid or duplicate aliases.
4. Import MUST go through the same `git config` write path as manual creation, so imported aliases are indistinguishable from hand-entered ones.

## §2 — Why this design

Developers set up new machines often; retyping aliases is friction that makes the tool feel disposable. Export/import removes that friction. Routing import through the same validation and write path as manual creation means there is exactly one code path that can mutate aliases, so an imported set can never bypass the rules that protect a hand-entered one.

## §3 — Implementation

- `src/components/settings/DataPanel.tsx` — the import/export UI.
- Import and export reuse `src-tauri/src/git_service.rs` for the read and write of aliases, so name validation and duplicate rejection from FR-GAM-001 apply unchanged.

## §4 — Acceptance criteria

1. Export produces a file representing the current aliases.
2. Import of that file recreates the aliases on another machine.
3. An import containing an invalid name is rejected for that entry (same rule as FR-GAM-001).
4. An import containing a duplicate of an existing alias is rejected for that entry.

## §5 — Verification

The import/export UI lives in the data panel and its component tests under `tests/`. The validation and write guarantees are inherited from FR-GAM-001's `git_service` path, which is covered by the seven integration tests listed there. Because import shares that path, the name-validation and duplicate-rejection guarantees are proven by the FR-GAM-001 suite.

## §6 — Failure modes

| Failure | Detection | Outcome |
|---|---|---|
| Import entry with invalid name | shared validation | that entry rejected |
| Import entry duplicates existing | shared duplicate check | that entry rejected |
| Malformed import file | parse guard | import declined; existing aliases untouched |

## §7 — Notes

Single write path is the key property: import cannot bypass FR-GAM-001's guarantees.

*End of FR-GAM-006. Fidelity: as-built (10/10 target).*
