---
id: NFR-GAM-001
title: "Least-privilege desktop surface"
module: GAM
status: done
fidelity: as-built
shipped: 2026-06-23
owner: Stephen Cheng
source_repo: zintaen/gam @ f55d97c
related: [FR-GAM-001, NFR-GAM-002]
---

## §1 — Statement (BCP-14 normative)

gam MUST request only the Tauri capabilities it actually uses.

1. The app MUST NOT grant a Tauri plugin or capability that has no real call site.
2. The shell and opener plugins MUST remain removed: no code path needs them, and opening a browser or folder MUST go through the `open` crate instead.
3. Adding any capability back MUST be justified by a concrete call site, not by anticipation.
4. The capabilities the app does keep (dialog, updater, process) MUST each correspond to a used feature.

## §2 — Why this matters

Every granted capability is attack surface and a permission the user implicitly trusts. The shell and opener plugins are particularly broad, and gam never called them, so keeping them was pure liability. Removing dead grants shrinks the surface to exactly what the app does, which is both safer and easier to reason about during review.

## §3 — Implementation

- `src-tauri/capabilities/default.json` — trimmed grants (shell and opener removed).
- `src-tauri/Cargo.toml` — `tauri-plugin-shell` and `tauri-plugin-opener` dependencies removed; the `open` crate handles opening a browser or folder.
- `src-tauri/src/lib.rs` — the corresponding `.plugin()` registrations removed.
- Kept: dialog, updater, process plugins, each tied to a real feature.

## §4 — Verification

- `src-tauri/capabilities/default.json` shows the reduced grant set.
- `src-tauri/Cargo.toml` shows no shell/opener plugins.
- `cargo clippy --locked -- -D warnings` and `cargo test --lib --locked` are green after the removal (gam gate, NFR-GAM-003), proving open-in-browser/open-folder still work via the `open` crate.

## §5 — Failure modes

| Failure | Detection | Outcome |
|---|---|---|
| New feature needs a capability | code review | add capability with its call site documented |
| Dead grant reintroduced | review of `default.json` diffs | reject without a call site |

## §6 — Notes

The trim was a deliberate hardening step: the plugins were dead grants, and `open` covers the only real need (launching a URL or folder).

*End of NFR-GAM-001. Fidelity: as-built (10/10 target).*
