---
nfr_id: NFR-GAM-001
audited: 2026-06-24
verdict: PASS (as-built)
score: 10/10
fidelity: as-built
template: as-built-verification@1
---

## §1 — Verdict summary

NFR-GAM-001 is satisfied. The shell and opener plugins were removed from capabilities, dependencies, and registrations, and the gate stayed green, proving nothing relied on them.

## §2 — Statement to artefact traceability

| §1 Clause | Artefact | Verification | Status |
|---|---|---|---|
| #1/#2 no dead grants; shell+opener removed | `capabilities/default.json`, `Cargo.toml`, `lib.rs` | diff removes both plugins; `open` crate retained | OK |
| #3 capability needs a call site | review policy | documented here | OK |
| #4 kept plugins each used | dialog/updater/process retained | each maps to a feature (file dialogs, FR-GAM-008, process control) | OK |

## §3 — Verification record

```bash
cd apps/gam/src-tauri
cargo clippy --locked -- -D warnings   # green after removal
cargo test --lib --locked              # green after removal
```

Green gate after removal is the proof that open-in-browser and open-folder still function through the `open` crate.

## §4 — Status

`accepted → shipped`.

*End of NFR-GAM-001 audit.*
