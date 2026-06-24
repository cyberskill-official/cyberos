---
id: FR-GAM-007
title: "Alias suggestions and command library"
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

gam SHOULD help users create aliases faster with a built-in library of common names and commands.

1. The app SHOULD suggest alias names and commands from a built-in library.
2. Selecting a suggestion MUST pre-fill the alias form.
3. After a suggestion pre-fills the form, every field MUST remain editable before save, so the suggestion is a starting point, not a lock-in.
4. A saved alias built from a suggestion MUST pass the same validation as any other (FR-GAM-001).

## §2 — Why this design

New users do not know what aliases are worth having; a curated library is a fast on-ramp. But a suggestion that cannot be edited is a trap, so selecting one only pre-fills the form and leaves everything editable. The save path is the same validated path as a hand-entered alias, so suggestions cannot smuggle in anything invalid.

## §3 — Implementation

- The suggestion engine under `src/services/` (command/name library and matching).
- `src/components/AliasForm.tsx` — selecting a suggestion pre-fills the form fields, which stay editable until save.
- Save goes through the FR-GAM-001 `git_service` path.

## §4 — Acceptance criteria

1. The library offers suggestions for alias names and commands.
2. Selecting a suggestion pre-fills the form.
3. Every pre-filled field can still be edited before save.
4. Saving a suggestion-derived alias applies normal validation.

## §5 — Verification

- `tests/services/suggestion-service.test.ts` — the suggestion engine.
- `tests/components/AliasForm.test.tsx` — library selection pre-fills the form, the alias name can be customized after selection, fields can be overwritten in edit mode, and the command stays editable after a library selection.

## §6 — Failure modes

| Failure | Detection | Outcome |
|---|---|---|
| No matching suggestion | empty list | manual entry, unaffected |
| Edited suggestion now invalid | FR-GAM-001 validation at save | rejected with reason |

## §7 — Notes

`AliasForm.test.tsx` explicitly asserts post-selection editability, which is the property that keeps suggestions helpful rather than restrictive.

*End of FR-GAM-007. Fidelity: as-built (10/10 target).*
