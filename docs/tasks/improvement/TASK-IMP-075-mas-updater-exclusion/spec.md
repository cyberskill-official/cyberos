---
id: TASK-IMP-075
title: "MAS updater exclusion — `mas` cargo feature compiles the self-updater out of the Mac App Store build"
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
eu_ai_act_risk_class: not_ai
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed
client_visible: false
type: improvement
created_at: 2026-07-13T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: improvement
priority: p0
status: done
verify: T
phase: "Wave 6 - go-live (Track B: store channels)"
owner: Stephen Cheng (CTO)
created: 2026-07-13
shipped: 2026-07-13
memory_chain_hash: null
related_tasks: [TASK-APP-003]
depends_on: []
blocks: []
source_pages:
  - "apps/desktop/src-tauri/src/lib.rs lines 75-107 (three #[cfg(desktop)] sites: spawn_update_check defn, plugin registration, launch call)"
  - "apps/desktop/src-tauri/tauri.conf.json plugins.updater (pubkey + GitHub endpoint CONFIGURED - the MAS bundle would actively self-update without this task)"
  - "docs/deploy/mac-app-store-submission.md 'Updater finding' + hard blocker #1 (TASK-APP-003's audit discovered this; fix explicitly deferred to this follow-up)"
source_decisions:
  - "2026-07-13 Stephen: 'Start' - go-ahead for the updater-exclusion follow-up TASK-APP-003 requires before MAS_RELEASE can ever flip."
language: rust 1.81 (cfg attributes + one feature flag), YAML (one CI flag)
service: apps/desktop/src-tauri
new_files: []
modified_files:
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/src/lib.rs
  - .github/workflows/release-mas.yml
  - docs/deploy/mac-app-store-submission.md
allowed_tools:
  - cargo feature flag + cfg attributes only - no runtime detection, no config-merge tricks (rejected in TASK-APP-003's answer sheet as unverifiable submission-safety)
disallowed_tools:
  - Anything that changes the Developer ID build's updater behavior (default build MUST keep self-updating exactly as today)
effort_hours: 2
subtasks:
  - "Cargo.toml: declare `mas = []` marker feature (0.5h)"
  - "lib.rs: widen the three #[cfg(desktop)] gates to #[cfg(all(desktop, not(feature = \"mas\")))] (0.5h)"
  - "release-mas.yml: pass --features mas at build; note loud-failure fallback if the CLI flag name differs (0.5h)"
  - "answer sheet: blocker #1 -> resolved-pending-first-build; residual dead-code note (0.5h)"
risk_if_skipped: "MAS_RELEASE can never be flipped: the sandboxed bundle would download-install-restart over itself on launch - an App Sandbox violation and an App Store policy rejection (TASK-APP-003 hard blocker #1). Every other MAS prerequisite is account-side; this is the sole engineering blocker."
---

## §1 — Description

1. `apps/desktop/src-tauri/Cargo.toml` **MUST** declare a `mas` feature (marker, no deps). Default builds (no feature) **MUST** be byte-for-byte behavior-identical to today: updater registered, launch check runs.
2. All three updater code sites in `lib.rs` (`spawn_update_check` definition, the `tauri_plugin_updater` registration block, the setup-time launch call) **MUST** change `#[cfg(desktop)]` to `#[cfg(all(desktop, not(feature = "mas")))]` — with `--features mas` the plugin is never registered and no update check is compiled in, so the configured `plugins.updater` block in tauri.conf.json becomes inert for the MAS target without touching that shared config (TASK-APP-003 §1 #1 forbids mutating it).
3. `release-mas.yml`'s build step **MUST** pass the feature (`--features mas`). If the Tauri CLI's cargo-feature passthrough flag differs on the pinned CLI major, the job fails loudly at build - an acceptable first-gated-run discovery, noted in-file (no macOS/cargo toolchain exists in this authoring environment to pre-verify).
4. `docs/deploy/mac-app-store-submission.md` hard blocker #1 **MUST** flip to resolved-pending-first-build, with one honest residual noted: the target-scoped dependency still compiles into the binary as dead code (registration excluded); shrinking it to an optional dependency is possible later on a real toolchain but is not submission-blocking the way active self-update was.

*Length note: sanctioned lean profile — a two-attribute Rust change plus one CI flag; §5's checks are the complete machine surface available pre-toolchain.*

## §2 — Why this design

Compile-time exclusion is the only path TASK-APP-003's answer sheet accepts: runtime checks and config-overlay null-outs both leave the self-update code reachable and were explicitly rejected there as unverifiable submission-safety. A marker feature + `not(feature)` widening is the minimal diff that keeps the default build provably unchanged (the attribute is strictly narrower only when the feature is on).

## §3 — API contract

```toml
[features]
custom-protocol = ["tauri/custom-protocol"]
mas = []   # Mac App Store target: compiles the self-updater OUT (TASK-IMP-075)
```
```rust
#[cfg(all(desktop, not(feature = "mas")))]   // x3, replacing #[cfg(desktop)]
```
```yaml
run: npx --yes @tauri-apps/cli@2 build --config src-tauri/tauri.mas.conf.json --bundles app --target universal-apple-darwin --features mas
```

## §4 — Acceptance criteria

1. Default-build neutrality: `grep -c '#\[cfg(all(desktop, not(feature = "mas")))\]' src/lib.rs` == 3 and `grep -c '#\[cfg(desktop)\]' src/lib.rs` == 0 — no site left half-gated.
2. `mas = []` present in Cargo.toml `[features]`.
3. release-mas.yml build step carries `--features mas`; YAML parses.
4. Answer sheet blocker #1 updated; residual dead-code note present.
5. First real toolchain run (`cargo check` then `cargo check --features mas`, later the gated CI build) compiles both ways — expected-pending here, executed on Stephen's machine or the first MAS_RELEASE run.

## §5 — Verification

```bash
grep -c 'not(feature = "mas")' apps/desktop/src-tauri/src/lib.rs        # 3
grep -c '#\[cfg(desktop)\]' apps/desktop/src-tauri/src/lib.rs           # 0
grep -n '^mas = \[\]' apps/desktop/src-tauri/Cargo.toml
grep -n 'features mas' .github/workflows/release-mas.yml
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/release-mas.yml'))"
# pending real toolchain: cargo check && cargo check --features mas
```

## §6 — Implementation skeleton

§3 is exhaustive.

## §7 — Dependencies

Upstream: TASK-APP-003 (done) discovered and scoped this. Downstream: unblocks `MAS_RELEASE=true` (remaining blockers are Stephen's account-side items). Batched with TASK-IMP-074 (cone-independent).

## §8 — Example payloads

Post-change gate output: `mas-entitlement-lint: OK` unchanged; MAS build log shows no `cyberos updater:` lines.

## §9 — Open questions

- Optional-dependency shrink (dead code removal) — later, on a real toolchain, if App Review flags the compiled-but-unregistered plugin (unlikely; no update behavior exists to observe).
- Tauri CLI `--features` passthrough name on the pinned major — confirmed at first gated run (clause 3 loud-failure posture).

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| cfg typo breaks the DEFAULT build | next desktop build/tag fails compile loudly | no silent ship | trivial attribute fix |
| CLI drops/renames --features | MAS job fails at build step | loud, pre-submission | adjust flag per CLI --help |
| feature on but a 4th updater site appears later | grep AC #1 in this spec + review norm | gate catches count drift | widen the new site |
| Apple flags dead plugin code | review feedback (external) | resubmit cycle | optional-dep follow-up (§9) |
| default build accidentally passes --features mas | only possible in release-mas.yml, which never builds the Developer ID channel | none | structural separation of workflows |

## §11 — Implementation notes

The three-site count (not two, not four) comes from reading lib.rs, not convention — AC #1's grep pins it. Batched with TASK-IMP-074 under v2.5.0 §11a.

*End of TASK-IMP-075.*
