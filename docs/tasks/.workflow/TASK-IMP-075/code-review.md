# TASK-IMP-075 — batch run record + review packet
Batch member 2 of {TASK-IMP-074, TASK-IMP-075}. Context: lib.rs read fresh - exactly 3 #[cfg(desktop)] updater sites. Status: `reviewing`, HALTED at HITL gate 1.

## §1 clause → evidence
| Clause | Evidence | ✓ |
|---|---|---|
| 1 mas feature, default unchanged | Cargo.toml `mas = []`; cfg is strictly narrower only when feature on | ✅ |
| 2 three sites widened | V8: grep not(feature = "mas") == 3 AND bare #[cfg(desktop)] == 0 | ✅ |
| 3 CI passes --features mas, loud-fail posture | release-mas.yml build step + in-file comment | ✅ |
| 4 answer sheet blocker #1 resolved + residual disclosed | mac-app-store-submission.md updated (dead-code note, cargo check ×2 on first toolchain) | ✅ |
Expected-pending (honest): cargo check both ways - no Rust toolchain in this environment; first real run is Stephen's machine or the first MAS_RELEASE build. Attribute syntax is the only risk surface and it is minimal.
Verdict needed: "TASK-IMP-075 review: approved" or rejected+reason.
