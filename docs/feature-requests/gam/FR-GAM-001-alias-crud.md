---
id: FR-GAM-001
title: "Alias CRUD across local and global scope"
module: GAM
priority: MUST
status: done
fidelity: as-built
shipped: 2026-06-23
owner: Stephen Cheng
source_repo: zintaen/gam @ f55d97c
related_frs: [FR-GAM-004, FR-GAM-006, FR-GAM-007]
---

## §1 — Description (BCP-14 normative)

gam MUST let users create, read, update, and delete Git aliases at both `--local` and `--global` scope, driving every change through `git config`.

1. The app MUST support the four operations against `git config` aliases: add, get (list), update (rename and/or change command), delete.
2. The app MUST validate alias names and reject any name that does not match `^[a-zA-Z][\w-]*$`. Validation MUST happen before any `git` subprocess runs.
3. The app MUST reject creating an alias whose name already exists at the target scope (no silent overwrite).
4. The app MUST preserve multi-word command values verbatim (for example `commit -v`), including flags and quoting.
5. Local-scope operations MUST run against the repository gitconfig only and MUST NOT touch the user's global gitconfig.
6. The app MUST sort the listed aliases case-insensitively by name for stable display.

## §2 — Why this design

Aliases are stored in plain `git config`, so the app is a thin, transparent layer over the user's real configuration rather than a parallel store that can drift. Name validation before the subprocess prevents shell-meta and malformed keys from reaching `git`. Rejecting duplicates avoids destroying an existing alias by accident. Scope isolation matters because a local edit that leaked into the global config would silently change every other repository on the machine.

## §3 — Implementation

- `src-tauri/src/git_service.rs` — the `git config` driver: add/get/update/delete, name validation regex, duplicate check, scope selection (`--local` vs `--global`), and the case-insensitive sort (`sort_by_key(|a| a.name.to_lowercase())`).
- `src-tauri/src/commands.rs` — the IPC commands the frontend calls.
- `src/components/AliasForm.tsx` and the alias table — the create/edit UI.

## §4 — Acceptance criteria

1. Add a local alias, then get → it is present at local scope.
2. Update renames the alias and changes its command in one operation.
3. Delete removes the alias from the target scope.
4. A name failing `^[a-zA-Z][\w-]*$` is rejected before any `git` call.
5. Adding a duplicate name at the same scope is rejected.
6. A multi-word command (`commit -v`) round-trips unchanged.
7. Local operations never mutate the global gitconfig.

## §5 — Verification

`src-tauri/src/git_service.rs` ships integration tests that run real `git` against a throwaway temp repo (created with `git init`, never touching the global gitconfig):

- `integration_add_then_get_local_alias`
- `integration_update_renames_and_changes_command`
- `integration_delete_removes_alias`
- `integration_get_on_fresh_repo_is_empty`
- `integration_add_duplicate_is_rejected`
- `integration_invalid_name_is_rejected_before_git`
- `integration_multiword_command_roundtrips`

All run under `cargo test --lib --locked` in the gam gate (NFR-GAM-003).

## §6 — Failure modes

| Failure | Detection | Outcome |
|---|---|---|
| Invalid alias name | regex check pre-subprocess | rejected, no `git` call |
| Duplicate name at scope | get-before-add check | rejected |
| `git` not on PATH | subprocess spawn error | surfaced to the user |
| Global gitconfig unintended write | scope flag fixed per call | prevented by design |

## §7 — Notes

The case-insensitive sort was changed from `sort_by(|a, b| a.name.to_lowercase().cmp(...))` to `sort_by_key(|a| a.name.to_lowercase())` during hardening; the two produce identical ordering and the latter satisfies clippy 1.96.

*End of FR-GAM-001. Fidelity: as-built (10/10 target).*
