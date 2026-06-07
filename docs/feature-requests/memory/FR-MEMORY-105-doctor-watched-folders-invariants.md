---
# ───── Machine-readable frontmatter (parsed by feature-request-audit + fr-catalog renderer) ─────
id: FR-MEMORY-105
title: "cyberos doctor — watched-folders integrity invariants (manifest ↔ filesystem ↔ HEAD reconciliation; 5 new invariants in memory.invariants.yaml)"
module: memory
priority: MUST
status: ready_to_implement
verify: T
phase: P1
milestone: P1 · slice 2
slice: 2
owner: Stephen Cheng
created: 2026-05-16
shipped: 2026-05-23
memory_chain_hash: null
related_frs: [FR-MEMORY-101, FR-MEMORY-102, FR-MEMORY-103, FR-MEMORY-104, FR-MEMORY-107, FR-MEMORY-110]
depends_on: [FR-MEMORY-101, FR-MEMORY-103, FR-MEMORY-104]
blocks: [FR-MEMORY-107, FR-MEMORY-110]

# ───── Source contracts ─────
source_pages:
  - website/docs/modules/memory.html#doctor
  - website/docs/runbooks/memory-doctor-runbook.html
source_decisions:
  - AGENTS.md §12 (FROZEN_RECOVERABLE / FROZEN_HUMAN states; doctor surfaces invariant failures)
  - AGENTS.md §7 (consolidation walk uses memory.invariants.yaml)
  - DEC-130 (watched-folders are first-class memory citizens; doctor MUST validate every registered folder)
  - DEC-131 (per-folder manifest fragment; symlink resolution; cross-volume drift detection)

# ───── Build envelope ─────
language: rust 1.81
service: cyberos/services/memory/
new_files:
  - services/memory/src/doctor/watched_folders.rs
  - services/memory/src/doctor/invariants_v2.rs
  - services/memory/memory.invariants.yaml.d/watched-folders-v2.yaml
  - services/memory/tests/doctor_watched_folders_test.rs
  - services/memory/tests/doctor_invariants_v2_test.rs
modified_files:
  - services/memory/src/doctor/mod.rs                       # register watched-folders module
  - services/memory/src/doctor/cli.rs                       # surface new invariant ids in `cyberos doctor --json`
  - services/memory/memory.invariants.yaml                  # include directive for watched-folders-v2.yaml
  - services/memory/docs/doctor.md                          # document the 5 new invariants
allowed_tools:
  - file_read: services/memory/**
  - file_write: services/memory/{src,tests,memory.invariants.yaml.d,docs}/**
  - bash: cd services/memory && cargo test doctor
  - bash: cd services/memory && cargo run --bin cyberos -- doctor --json
disallowed_tools:
  - mutating memory state from inside doctor (read-only per AGENTS.md §12 — doctor diagnoses, repair is a separate command)
  - silently downgrading invariant failures to warnings (every failure transitions agent state per §12)
  - skipping symlink resolution before stat (per AGENTS.md §0.4 — must resolve through every symlink)

# ───── Estimated work ─────
effort_hours: 7
sub_tasks:
  - "0.5h: watched-folders-v2.yaml — declare 5 new invariants with id, severity, description, fixture refs"
  - "0.5h: invariants_v2.rs — invariant trait + WatchedFolderManifestPresent struct"
  - "0.5h: WatchedFolderResolvable invariant (symlink → real path → stat ENOENT detection)"
  - "0.5h: WatchedFolderNotShadowed invariant (same realpath under two registrations = ambiguous)"
  - "0.5h: WatchedFolderHEADConsistent invariant (folder HEAD ≤ root HEAD; no time-travel)"
  - "0.5h: WatchedFolderManifestSchema invariant (folder manifest fragment matches schema v1)"
  - "0.5h: doctor/mod.rs registration + ordering (run AFTER core invariants, BEFORE consolidation)"
  - "0.5h: cli.rs --json output: per-invariant {id, severity, status, details}"
  - "1.0h: doctor_watched_folders_test.rs — happy + 5 failure paths"
  - "1.0h: doctor_invariants_v2_test.rs — fixture generator + property test (1000 random folder states)"
  - "0.5h: doctor.md — operator-facing remediation guide per invariant"
  - "0.5h: integration into FR-MEMORY-110 health check (doctor exit code propagates)"
risk_if_skipped: "Without these invariants, a watched-folder registration can silently rot: the folder is unmounted, the symlink dangles, two registrations point at the same realpath, the folder's HEAD diverges from root, or its manifest schema drifts. None of these are caught by core memory invariants (those only audit `<memory-root>/audit/*.binlog` and HEAD). The first symptom of any of these is a capture-write failure with a confusing error message — operator wastes an hour discovering the dangling symlink instead of seeing it on `cyberos doctor`. FR-MEMORY-110's health-check daemon depends on this invariant set to know when to refuse-to-start."
---

## §1 — Description (BCP-14 normative)

The `cyberos doctor` command **MUST** extend its invariant set with five new invariants specific to watched-folders. Each invariant is declared in `memory.invariants.yaml.d/watched-folders-v2.yaml` (included into the master `memory.invariants.yaml` via `include:` directive at the bottom of the master file). The five invariants:

1. **MUST** add invariant `WatchedFolderManifestPresent` (severity `error`): for every entry in `manifest.watched_folders[]`, the file `<folder>/manifest.json` MUST exist, be readable, and parse as valid JSON conforming to `memory.schema.json#/definitions/WatchedFolderManifest`. Failure → agent state transitions to `FROZEN_RECOVERABLE` (per AGENTS.md §12); repair via `cyberos memory watch <path>` re-bootstraps the fragment OR `cyberos memory unwatch <path>` removes the registration.

2. **MUST** add invariant `WatchedFolderResolvable` (severity `error`): for every entry in `manifest.watched_folders[]`, the path MUST resolve through every symlink (per AGENTS.md §0.4) and the final target MUST exist (`stat()` succeeds, not ENOENT). Failure → `FROZEN_RECOVERABLE`; repair via remount the volume OR `cyberos memory unwatch <path>` if the folder is permanently gone.

3. **MUST** add invariant `WatchedFolderNotShadowed` (severity `error`): no two entries in `manifest.watched_folders[]` MAY resolve to the same realpath. Ambiguous registration is forbidden by construction. Failure → `FROZEN_RECOVERABLE`; repair via `cyberos memory unwatch` on one of the duplicates (the operator picks which).

4. **MUST** add invariant `WatchedFolderHEADConsistent` (severity `error`): for every watched folder, the folder's `HEAD` (8-byte LE u64 seq counter per AGENTS.md §2) MUST be `<= manifest.root_head`. A folder cannot have advanced past the root's published chain tip (would imply time-travel writes). Failure → `FROZEN_HUMAN` (catastrophic divergence per AGENTS.md §12); repair requires explicit `cyberos doctor --repair --reason "<text>"`.

5. **MUST** add invariant `WatchedFolderManifestSchema` (severity `warning`): for every watched folder, the per-folder `manifest.json` fragment MUST validate against `memory.schema.json#/definitions/WatchedFolderManifest`. Unknown keys → warning only (forward-compat); missing required keys → error escalation to `WatchedFolderManifestPresent` failure.

6. **MUST** run the five new invariants AFTER the core invariants (per AGENTS.md §7.2 walk order) and BEFORE consolidation triggers (per §7.6). The execution order is locked in `doctor/mod.rs`'s `INVARIANT_ORDER` slice; tests assert it.

7. **MUST** emit per-invariant output to stdout (human) AND `cyberos doctor --json` (machine):
   - Human format: `[<severity>] <invariant-id>: <status> — <details>` (one line each, terminal-aware colours).
   - JSON format: `{ "id": "WatchedFolderManifestPresent", "severity": "error", "status": "pass" | "fail" | "skip", "details": "...", "affected_paths": [...] }`.
   - Skipping is permitted IFF the dependency invariant has already failed (e.g. `WatchedFolderManifestSchema` skips when `WatchedFolderManifestPresent` failed for the same folder); `status: "skip"` carries `skipped_because: "<dependency-invariant-id>"`.

8. **MUST** be deterministic — running doctor twice on the same on-disk state produces byte-identical JSON output. Folder iteration is sorted by realpath; no `Date.now()`, no map-iteration-order assumptions.

9. **MUST** complete in ≤ 5 seconds for ≤ 50 watched folders on commodity hardware (M-series Mac or equivalent x86_64). Latency budget per invariant: ≤ 100ms for `WatchedFolderManifestPresent`/`Resolvable`/`NotShadowed`/`HEADConsistent`; ≤ 500ms for `WatchedFolderManifestSchema` (jsonschema validation is heavier). Asserted by `doctor_watched_folders_test.rs::latency_test`.

10. **MUST** integrate into `cyberos doctor` exit code: any error-severity failure → exit 2 (`InvariantFailure`); any warning-only failures → exit 0 with warnings printed; no failures → exit 0. Exit codes per the shared `cyberos-cli-exit::ExitCode` (slot reserved at `200` range for memory-specific codes per FR-AI-021 §3 note; `200 = InvariantFailure`).

11. **MUST** emit OTel span `memory.doctor.invariant_check` per invariant call with attributes `invariant_id`, `severity`, `status`, `duration_ms`, `affected_path_count` (when applicable). Aggregated by FR-OBS-005 for trend dashboards.

12. **SHOULD** support `--invariant <id>` filter to run a single invariant (operator debugging) and `--only watched-folders` filter to scope to this category (CI optimisation).

13. **SHOULD** emit metric `memory_doctor_invariants_failed_total{invariant_id, severity}` (counter) and `memory_doctor_invariant_duration_seconds{invariant_id}` (histogram with FR-OBS-003 standardised buckets).

---

## §2 — Why this design (rationale for humans)

**Why declare invariants in YAML, not Rust?** Per AGENTS.md §7 the consolidation walk reads `memory.invariants.yaml`. Keeping invariants as data (not code) means external tools (audit pipelines, third-party doctor wrappers) can read the catalogue without compiling Rust. The Rust impl is the *executor*; the YAML is the *spec*. Adding an invariant becomes a YAML edit + a Rust impl PR — discoverable + reviewable.

**Why 5 specific invariants?** Each catches a distinct watched-folder failure mode:
- `ManifestPresent` — operator forgot to commit/sync the per-folder manifest fragment.
- `Resolvable` — volume unmounted, symlink target moved, ENOSPC on the target FS.
- `NotShadowed` — operator ran `memory watch` twice with different relative paths that resolve to the same realpath.
- `HEADConsistent` — clock skew, manual file corruption, or a (bug-induced) write that bypassed the canonical writer.
- `ManifestSchema` — version drift between writer and reader; forward-compat probe.

Each failure mode has a distinct remediation, so they earn distinct invariants. Coalescing them into one "folder OK" invariant would hide which class of failure the operator is dealing with.

**Why `FROZEN_HUMAN` for HEAD inconsistency (§1 #4)?** A folder's HEAD exceeding root HEAD implies the folder was written-to OUTSIDE the canonical writer (per AGENTS.md §14.1 only the writer mutates HEAD). This is either a malicious actor or a catastrophic bug. The operator must explicitly intervene via `cyberos doctor --repair --reason "<text>"` so the action is auditable; auto-repair would erase the evidence.

**Why ordering matters (§1 #6)?** Core invariants (e.g. `LedgerChainIntact`) MUST pass first — if the audit chain itself is broken, watched-folder integrity is moot. Conversely, watched-folder invariants MUST run BEFORE consolidation triggers because consolidation walks all memories including those in watched folders; if a folder is unreachable, the walk would fail mid-way and corrupt the consolidation state.

**Why deterministic JSON output (§1 #8)?** CI gates downstream of doctor (e.g. FR-MEMORY-110's health check, FR-OBS-007's auto-triage) diff doctor's output across runs. Non-deterministic output → false-positive diffs → operators ignore real failures. Sort-by-realpath is the canonical iteration order.

**Why 5-second latency budget (§1 #9)?** Doctor runs as part of FR-MEMORY-110's daemon health check at startup AND on a 60-second interval. 5 seconds is the upper bound that lets the daemon remain responsive; missing the budget triggers a sev-2 alarm. For ≤ 50 folders the budget is generous; the threshold exists to catch pathological cases (e.g. 10,000 dangling symlinks would blow the budget).

**Why `--invariant <id>` filter (§1 #12)?** Operator debugging workflow: "doctor is failing on `WatchedFolderManifestSchema` — let me run just that one with `--json --verbose` to see exactly which folder fails." Without the filter, the operator re-runs all invariants (slow + noisy). Same for CI: `--only watched-folders` lets a folder-touching PR's CI skip the unrelated core-invariant suite (fast).

**Why per-invariant OTel span (§1 #11)?** Aggregated dashboards answer "which invariant is the slowest 95p?" and "which invariant failed how often this week?" Without per-invariant attribution, the only signal is "doctor took N seconds today" — not actionable.

---

## §3 — API contract

### Invariant YAML

```yaml
# services/memory/memory.invariants.yaml.d/watched-folders-v2.yaml
schema_version: v2
invariants:
  - id: WatchedFolderManifestPresent
    severity: error
    description: "Every entry in manifest.watched_folders[] MUST have a readable manifest.json fragment at <folder>/manifest.json conforming to WatchedFolderManifest schema."
    state_on_failure: FROZEN_RECOVERABLE
    repair_hint: "cyberos memory watch <path>   # re-bootstrap the fragment"
    fixture: tests/fixtures/wf_no_manifest/
    order: 100   # after core (0..99)

  - id: WatchedFolderResolvable
    severity: error
    description: "Every watched folder path MUST resolve through symlinks to an extant inode (stat() != ENOENT)."
    state_on_failure: FROZEN_RECOVERABLE
    repair_hint: "remount the volume OR cyberos memory unwatch <path>"
    fixture: tests/fixtures/wf_dangling_symlink/
    order: 101

  - id: WatchedFolderNotShadowed
    severity: error
    description: "No two entries in manifest.watched_folders[] MAY resolve to the same realpath."
    state_on_failure: FROZEN_RECOVERABLE
    repair_hint: "cyberos memory unwatch <one of the duplicates>"
    fixture: tests/fixtures/wf_two_same_realpath/
    order: 102

  - id: WatchedFolderHEADConsistent
    severity: error
    description: "Every watched folder's HEAD MUST be <= manifest.root_head (no time-travel)."
    state_on_failure: FROZEN_HUMAN
    repair_hint: "cyberos doctor --repair --reason \"investigated: <root cause>\""
    fixture: tests/fixtures/wf_head_ahead_of_root/
    order: 103

  - id: WatchedFolderManifestSchema
    severity: warning
    description: "Every watched folder's manifest fragment MUST validate against WatchedFolderManifest schema; unknown keys are warned, missing required keys escalate to WatchedFolderManifestPresent."
    state_on_failure: READY   # warning, not error
    repair_hint: "Verify cyberos version matches manifest fragment schema_version; upgrade or downgrade as appropriate."
    fixture: tests/fixtures/wf_manifest_schema_drift/
    order: 104
```

### Invariant Rust trait

```rust
// services/memory/src/doctor/invariants_v2.rs
use crate::doctor::{Severity, State, InvariantResult};
use crate::manifest::Manifest;
use std::path::{Path, PathBuf};
use std::time::Instant;

pub trait Invariant {
    fn id(&self) -> &'static str;
    fn severity(&self) -> Severity;
    fn check(&self, manifest: &Manifest, root: &Path) -> InvariantResult;
}

pub struct WatchedFolderManifestPresent;
pub struct WatchedFolderResolvable;
pub struct WatchedFolderNotShadowed;
pub struct WatchedFolderHEADConsistent;
pub struct WatchedFolderManifestSchema;

impl Invariant for WatchedFolderManifestPresent {
    fn id(&self) -> &'static str { "WatchedFolderManifestPresent" }
    fn severity(&self) -> Severity { Severity::Error }
    fn check(&self, manifest: &Manifest, _root: &Path) -> InvariantResult {
        let start = Instant::now();
        let mut affected = Vec::new();
        // Sort by realpath for deterministic output (§1 #8)
        let mut folders: Vec<_> = manifest.watched_folders.iter().collect();
        folders.sort_by_key(|wf| wf.realpath.clone());

        for wf in folders {
            let frag = wf.realpath.join("manifest.json");
            match std::fs::read(&frag) {
                Ok(bytes) => {
                    if serde_json::from_slice::<crate::manifest::WatchedFolderManifest>(&bytes).is_err() {
                        affected.push(wf.realpath.clone());
                    }
                }
                Err(_) => affected.push(wf.realpath.clone()),
            }
        }

        if affected.is_empty() {
            InvariantResult::pass(self.id(), start.elapsed())
        } else {
            InvariantResult::fail(self.id(), Severity::Error, start.elapsed())
                .with_details(format!("{} watched folder(s) missing or unreadable manifest.json", affected.len()))
                .with_affected_paths(affected)
                .with_state_transition(State::FrozenRecoverable)
        }
    }
}

// WatchedFolderResolvable: stat() the realpath via std::fs::metadata
// (which follows symlinks). ENOENT → fail.
impl Invariant for WatchedFolderResolvable {
    fn id(&self) -> &'static str { "WatchedFolderResolvable" }
    fn severity(&self) -> Severity { Severity::Error }
    fn check(&self, manifest: &Manifest, _root: &Path) -> InvariantResult {
        let start = Instant::now();
        let mut affected = Vec::new();
        let mut folders: Vec<_> = manifest.watched_folders.iter().collect();
        folders.sort_by_key(|wf| wf.realpath.clone());

        for wf in folders {
            // canonicalize() resolves all symlinks per AGENTS.md §0.4
            match std::fs::canonicalize(&wf.path) {
                Ok(real) => {
                    if std::fs::metadata(&real).is_err() {
                        affected.push(wf.path.clone());
                    }
                }
                Err(_) => affected.push(wf.path.clone()),
            }
        }
        if affected.is_empty() {
            InvariantResult::pass(self.id(), start.elapsed())
        } else {
            InvariantResult::fail(self.id(), Severity::Error, start.elapsed())
                .with_affected_paths(affected)
                .with_state_transition(State::FrozenRecoverable)
        }
    }
}

// WatchedFolderNotShadowed: build realpath → [orig_paths] map; any
// realpath with > 1 orig path is a shadow.
impl Invariant for WatchedFolderNotShadowed {
    fn id(&self) -> &'static str { "WatchedFolderNotShadowed" }
    fn severity(&self) -> Severity { Severity::Error }
    fn check(&self, manifest: &Manifest, _root: &Path) -> InvariantResult {
        let start = Instant::now();
        let mut by_realpath: std::collections::BTreeMap<PathBuf, Vec<PathBuf>> = Default::default();
        for wf in &manifest.watched_folders {
            if let Ok(real) = std::fs::canonicalize(&wf.path) {
                by_realpath.entry(real).or_default().push(wf.path.clone());
            }
        }
        let shadowed: Vec<PathBuf> = by_realpath
            .into_iter()
            .filter(|(_, v)| v.len() > 1)
            .flat_map(|(_, v)| v)
            .collect();

        if shadowed.is_empty() {
            InvariantResult::pass(self.id(), start.elapsed())
        } else {
            InvariantResult::fail(self.id(), Severity::Error, start.elapsed())
                .with_details(format!("{} path(s) resolve to a duplicated realpath", shadowed.len()))
                .with_affected_paths(shadowed)
                .with_state_transition(State::FrozenRecoverable)
        }
    }
}

// WatchedFolderHEADConsistent: read folder/HEAD (LE u64) and compare
// with manifest.root_head.
impl Invariant for WatchedFolderHEADConsistent {
    fn id(&self) -> &'static str { "WatchedFolderHEADConsistent" }
    fn severity(&self) -> Severity { Severity::Error }
    fn check(&self, manifest: &Manifest, _root: &Path) -> InvariantResult {
        let start = Instant::now();
        let mut affected = Vec::new();
        let mut folders: Vec<_> = manifest.watched_folders.iter().collect();
        folders.sort_by_key(|wf| wf.realpath.clone());

        for wf in folders {
            let head_path = wf.realpath.join("HEAD");
            if let Ok(bytes) = std::fs::read(&head_path) {
                if bytes.len() == 8 {
                    let folder_head = u64::from_le_bytes(bytes.try_into().unwrap());
                    if folder_head > manifest.audit_chain_head {
                        affected.push(wf.realpath.clone());
                    }
                }
            }
        }
        if affected.is_empty() {
            InvariantResult::pass(self.id(), start.elapsed())
        } else {
            InvariantResult::fail(self.id(), Severity::Error, start.elapsed())
                .with_details("folder HEAD exceeds root chain tip — implies write outside canonical writer".into())
                .with_affected_paths(affected)
                .with_state_transition(State::FrozenHuman)
        }
    }
}
```

### Doctor integration

```rust
// services/memory/src/doctor/mod.rs (excerpt)
const INVARIANT_ORDER: &[&dyn Invariant] = &[
    // Core (0..99) — registered in core_invariants.rs
    &core::LedgerChainIntact,
    &core::ManifestParseable,
    &core::HEADExists,
    // ... other core invariants ...

    // Watched-folders (100..199) — registered here per FR-MEMORY-105
    &invariants_v2::WatchedFolderManifestPresent,
    &invariants_v2::WatchedFolderResolvable,
    &invariants_v2::WatchedFolderNotShadowed,
    &invariants_v2::WatchedFolderHEADConsistent,
    &invariants_v2::WatchedFolderManifestSchema,
];

pub fn run_all(manifest: &Manifest, root: &Path) -> Vec<InvariantResult> {
    INVARIANT_ORDER.iter().map(|inv| {
        // Dependency-skip: WatchedFolderManifestSchema skips when
        // WatchedFolderManifestPresent failed for the SAME folder set.
        // (Generalised dependency table lives in InvariantResult::skipped_when)
        inv.check(manifest, root)
    }).collect()
}
```

### CLI JSON output schema

```json
{
  "schema_version": "v1",
  "command": "cyberos doctor",
  "exit_code": 2,
  "agent_state": "FROZEN_RECOVERABLE",
  "invariants": [
    {
      "id": "LedgerChainIntact",
      "severity": "error",
      "status": "pass",
      "duration_ms": 42
    },
    {
      "id": "WatchedFolderManifestPresent",
      "severity": "error",
      "status": "fail",
      "duration_ms": 11,
      "details": "1 watched folder(s) missing or unreadable manifest.json",
      "affected_paths": ["/Users/stephencheng/Documents/work-notes"],
      "state_transition": "FROZEN_RECOVERABLE",
      "repair_hint": "cyberos memory watch /Users/stephencheng/Documents/work-notes"
    },
    {
      "id": "WatchedFolderManifestSchema",
      "severity": "warning",
      "status": "skip",
      "skipped_because": "WatchedFolderManifestPresent"
    }
  ]
}
```

---

## §4 — Acceptance criteria

1. **All 5 invariants registered** — `cyberos doctor --list-invariants` includes `WatchedFolder{ManifestPresent,Resolvable,NotShadowed,HEADConsistent,ManifestSchema}` in the documented order.
2. **YAML matches Rust** — `tests::yaml_matches_rust` walks `memory.invariants.yaml.d/watched-folders-v2.yaml` and asserts each declared invariant has a corresponding `impl Invariant` in `invariants_v2.rs`.
3. **Happy path: 0 failures** — fixture `tests/fixtures/wf_happy/` with 3 valid watched folders; all 5 invariants pass; exit code 0.
4. **ManifestPresent fails on missing fragment** — fixture deletes one folder's `manifest.json`; `WatchedFolderManifestPresent` fails; exit 2; affected_paths lists the folder.
5. **Resolvable fails on dangling symlink** — fixture creates `wf/symlink → /nonexistent`; `WatchedFolderResolvable` fails; exit 2.
6. **NotShadowed fails on two registrations of same realpath** — fixture registers `/Users/x/notes` and `/Users/x/notes/` and `/Users/x/./notes` (all same realpath); `WatchedFolderNotShadowed` fails with 3 affected paths.
7. **HEADConsistent fails on HEAD > root** — fixture manually writes a folder HEAD = root_head + 5; `WatchedFolderHEADConsistent` fails with state transition `FROZEN_HUMAN`; exit 2.
8. **ManifestSchema warns on unknown key** — fixture adds `{"future_key": 42}` to a folder manifest; invariant reports `severity: warning`; exit code remains 0; warning printed.
9. **ManifestSchema escalates on missing required key** — fixture removes `schema_version` from a folder manifest; `WatchedFolderManifestPresent` fails (not the schema-only warning); exit 2.
10. **Dependency skip works** — when `ManifestPresent` fails for a folder, `ManifestSchema` reports `status: "skip"` with `skipped_because: "WatchedFolderManifestPresent"` for that folder only.
11. **Determinism: byte-identical JSON** — `cyberos doctor --json` run twice on the same fixture produces byte-identical output (sorted paths, no clock-dependent fields).
12. **Latency budget** — 50-folder fixture completes `doctor --only watched-folders` in ≤ 5 seconds wall-clock (`doctor_watched_folders_test::latency_test`).
13. **Filter: `--invariant <id>`** — `cyberos doctor --invariant WatchedFolderNotShadowed` runs only that invariant; other invariants absent from output.
14. **Filter: `--only watched-folders`** — runs all 5 watched-folder invariants; core invariants skipped.
15. **OTel span per invariant** — exporter receives 5 spans named `memory.doctor.invariant_check` with attribute `invariant_id`; assert via in-memory exporter.
16. **Metric increments on failure** — failure path increments `memory_doctor_invariants_failed_total{invariant_id, severity}`; assert via in-process Prometheus registry.

---

## §5 — Verification

```rust
// services/memory/tests/doctor_watched_folders_test.rs

#[test]
fn all_invariants_registered() {
    let ids = doctor::list_invariant_ids();
    for required in [
        "WatchedFolderManifestPresent",
        "WatchedFolderResolvable",
        "WatchedFolderNotShadowed",
        "WatchedFolderHEADConsistent",
        "WatchedFolderManifestSchema",
    ] {
        assert!(ids.contains(&required), "missing invariant: {required}");
    }
}

#[test]
fn yaml_matches_rust() {
    let yaml = include_str!("../memory.invariants.yaml.d/watched-folders-v2.yaml");
    let parsed: WatchedFoldersV2 = serde_yaml::from_str(yaml).unwrap();
    let rust_ids: HashSet<_> = doctor::list_invariant_ids().into_iter().collect();
    for inv in parsed.invariants {
        assert!(rust_ids.contains(inv.id.as_str()),
                "YAML declares invariant {} with no Rust impl", inv.id);
    }
}

#[test]
fn happy_path_exit_0() {
    let fixture = TestFixture::wf_happy();
    let result = doctor::run_all(&fixture.manifest(), fixture.root());
    let exit = doctor::exit_code(&result);
    assert_eq!(exit, 0);
    assert!(result.iter().all(|r| matches!(r.status, Status::Pass)));
}

#[test]
fn manifest_present_fails_on_missing_fragment() {
    let mut fixture = TestFixture::wf_happy();
    fs::remove_file(fixture.watched_folder(0).join("manifest.json")).unwrap();
    let result = doctor::run_all(&fixture.manifest(), fixture.root());
    let inv = result.iter().find(|r| r.id == "WatchedFolderManifestPresent").unwrap();
    assert_eq!(inv.status, Status::Fail);
    assert_eq!(inv.affected_paths.len(), 1);
}

#[test]
fn not_shadowed_detects_three_paths_same_realpath() {
    let fixture = TestFixture::wf_three_same_realpath();
    let result = doctor::run_all(&fixture.manifest(), fixture.root());
    let inv = result.iter().find(|r| r.id == "WatchedFolderNotShadowed").unwrap();
    assert_eq!(inv.status, Status::Fail);
    assert_eq!(inv.affected_paths.len(), 3);
}

#[test]
fn head_consistent_triggers_frozen_human() {
    let mut fixture = TestFixture::wf_happy();
    // Manually advance folder HEAD past root HEAD (simulating bypass writer)
    let folder = fixture.watched_folder(0);
    let bad_head = fixture.manifest().audit_chain_head + 5;
    fs::write(folder.join("HEAD"), bad_head.to_le_bytes()).unwrap();

    let result = doctor::run_all(&fixture.manifest(), fixture.root());
    let inv = result.iter().find(|r| r.id == "WatchedFolderHEADConsistent").unwrap();
    assert_eq!(inv.status, Status::Fail);
    assert_eq!(inv.state_transition, Some(State::FrozenHuman));
}

#[test]
fn manifest_schema_warning_keeps_exit_0() {
    let mut fixture = TestFixture::wf_happy();
    // Add unknown key (warning only)
    let frag_path = fixture.watched_folder(0).join("manifest.json");
    let mut frag: serde_json::Value = serde_json::from_slice(&fs::read(&frag_path).unwrap()).unwrap();
    frag["future_key"] = serde_json::json!(42);
    fs::write(&frag_path, serde_json::to_vec_pretty(&frag).unwrap()).unwrap();

    let result = doctor::run_all(&fixture.manifest(), fixture.root());
    let exit = doctor::exit_code(&result);
    assert_eq!(exit, 0);  // warning-only does not change exit
    let inv = result.iter().find(|r| r.id == "WatchedFolderManifestSchema").unwrap();
    assert_eq!(inv.severity, Severity::Warning);
}

#[test]
fn determinism_byte_identical_json() {
    let fixture = TestFixture::wf_happy();
    let json_1 = doctor::run_all_json(&fixture.manifest(), fixture.root());
    let json_2 = doctor::run_all_json(&fixture.manifest(), fixture.root());
    assert_eq!(json_1, json_2);
}

#[test]
fn latency_test_50_folders_under_5_seconds() {
    let fixture = TestFixture::wf_50_folders();
    let start = Instant::now();
    let _ = doctor::run_all(&fixture.manifest(), fixture.root());
    assert!(start.elapsed() < Duration::from_secs(5));
}
```

```bash
# CI integration (snippet from doctor.yml workflow)
- name: doctor (watched-folders only)
  run: |
    cyberos doctor --only watched-folders --json > doctor-report.json
    # Fail CI if any error-severity invariant failed
    jq -e '.invariants | map(select(.severity == "error" and .status == "fail")) | length == 0' doctor-report.json
```

---

## §6 — Implementation skeleton

```rust
// services/memory/src/doctor/mod.rs  (orchestrator excerpt)

use std::path::Path;
use crate::manifest::Manifest;

pub mod core;
pub mod invariants_v2;
pub mod cli;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity { Error, Warning, Info }

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status { Pass, Fail, Skip }

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum State { Ready, FrozenRecoverable, FrozenHuman }

#[derive(Clone, Debug, serde::Serialize)]
pub struct InvariantResult {
    pub id: &'static str,
    pub severity: Severity,
    pub status: Status,
    pub duration_ms: u128,
    #[serde(skip_serializing_if = "Option::is_none")] pub details: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]   pub affected_paths: Vec<std::path::PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")] pub state_transition: Option<State>,
    #[serde(skip_serializing_if = "Option::is_none")] pub skipped_because: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] pub repair_hint: Option<String>,
}
```

```rust
// services/memory/src/doctor/cli.rs

use clap::Parser;

#[derive(Parser)]
pub struct DoctorArgs {
    /// Run a single invariant by id
    #[arg(long)] pub invariant: Option<String>,
    /// Run only invariants in a category (core | watched-folders)
    #[arg(long)] pub only: Option<String>,
    /// Output machine-readable JSON
    #[arg(long)] pub json: bool,
    /// List all registered invariants and exit 0
    #[arg(long)] pub list_invariants: bool,
}

pub fn main(args: DoctorArgs, manifest: &Manifest, root: &Path) -> i32 {
    if args.list_invariants {
        for id in list_invariant_ids() { println!("{id}"); }
        return 0;
    }
    let results = run_filtered(args.invariant.as_deref(), args.only.as_deref(), manifest, root);
    if args.json {
        println!("{}", serde_json::to_string_pretty(&JsonReport::new(&results)).unwrap());
    } else {
        for r in &results { print_human(r); }
    }
    exit_code(&results)
}

pub fn exit_code(results: &[InvariantResult]) -> i32 {
    if results.iter().any(|r| r.severity == Severity::Error && r.status == Status::Fail) {
        200  // shared cyberos-cli-exit::ExitCode (memory-range InvariantFailure)
    } else { 0 }
}
```

```yaml
# services/memory/memory.invariants.yaml (master file, append include)
include:
  - memory.invariants.yaml.d/core.yaml
  - memory.invariants.yaml.d/watched-folders-v2.yaml
```

---

## §7 — Dependencies

- **FR-MEMORY-101 (upstream)** — defines `manifest.watched_folders[]` schema, the data this FR audits.
- **FR-MEMORY-102 (related)** — `cyberos memory watch/unwatch` is the repair path for `WatchedFolderManifestPresent` failures.
- **FR-MEMORY-103 (related)** — multi-device sync writes folder HEADs; if it bypasses the canonical writer, this FR catches it.
- **FR-MEMORY-107 (downstream)** — FS watcher's startup uses `doctor --only watched-folders` as a gate; refuses to start if any error-severity invariant fails.
- **FR-MEMORY-110 (downstream)** — health-check daemon runs doctor on a 60-second interval and reports failures via OTel.
- **FR-OBS-003, FR-OBS-005, FR-OBS-007 (cross-module)** — metrics + spans flow into observability pillar.

---

## §8 — Example payloads

### Happy doctor run (JSON)

```json
{
  "schema_version": "v1",
  "command": "cyberos doctor --only watched-folders --json",
  "exit_code": 0,
  "agent_state": "READY",
  "invariants": [
    {"id": "WatchedFolderManifestPresent", "severity": "error",   "status": "pass", "duration_ms": 12},
    {"id": "WatchedFolderResolvable",      "severity": "error",   "status": "pass", "duration_ms":  8},
    {"id": "WatchedFolderNotShadowed",     "severity": "error",   "status": "pass", "duration_ms":  6},
    {"id": "WatchedFolderHEADConsistent",  "severity": "error",   "status": "pass", "duration_ms":  9},
    {"id": "WatchedFolderManifestSchema",  "severity": "warning", "status": "pass", "duration_ms": 41}
  ]
}
```

### Failure cascade (human)

```text
[error] WatchedFolderManifestPresent: FAIL — 1 watched folder(s) missing or unreadable manifest.json
        affected: /Users/stephencheng/Documents/work-notes
        repair:   cyberos memory watch /Users/stephencheng/Documents/work-notes
        state:    FROZEN_RECOVERABLE

[error] WatchedFolderResolvable:      PASS  (8ms)
[error] WatchedFolderNotShadowed:     PASS  (6ms)
[error] WatchedFolderHEADConsistent:  PASS  (9ms)
[warn]  WatchedFolderManifestSchema:  SKIP — skipped because WatchedFolderManifestPresent failed for the same folder

Agent state: FROZEN_RECOVERABLE
Exit code:   200
```

### `--invariant <id>` filter

```bash
$ cyberos doctor --invariant WatchedFolderNotShadowed --json
{
  "schema_version": "v1",
  "command": "cyberos doctor --invariant WatchedFolderNotShadowed --json",
  "exit_code": 0,
  "agent_state": "READY",
  "invariants": [
    {"id": "WatchedFolderNotShadowed", "severity": "error", "status": "pass", "duration_ms": 6}
  ]
}
```

---

## §9 — Open questions

All resolved. Deferred:
- Invariant priorities + ordering knobs (operator might want to skip certain invariants in dev) — slice 3+.
- Self-healing: should `WatchedFolderResolvable` auto-unwatch a 7-day-dangling folder? — slice 3+; needs UX design.
- `--repair` mode that re-bootstraps `manifest.json` from current folder state — slice 3+; conflicts with audit-before-action principle.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Watched folder unmounted | `WatchedFolderResolvable` fails | exit 200, FROZEN_RECOVERABLE | Operator remounts volume + reruns doctor |
| Dangling symlink (target deleted) | `WatchedFolderResolvable` fails | exit 200, FROZEN_RECOVERABLE | Operator `cyberos memory unwatch <path>` |
| Two registrations to same realpath | `WatchedFolderNotShadowed` fails | exit 200, FROZEN_RECOVERABLE | Operator `unwatch` one duplicate |
| Folder manifest.json deleted | `WatchedFolderManifestPresent` fails | exit 200, FROZEN_RECOVERABLE | Operator `cyberos memory watch <path>` re-bootstraps |
| Folder manifest.json corrupt JSON | `WatchedFolderManifestPresent` fails | exit 200, FROZEN_RECOVERABLE | Operator restores from backup OR re-watches |
| Folder manifest.json missing required key | `WatchedFolderManifestPresent` fails | exit 200, FROZEN_RECOVERABLE | Operator upgrades / re-watches |
| Folder manifest.json has unknown key (forward-compat) | `WatchedFolderManifestSchema` warns | exit 0, READY | Informational; consider upgrading reader |
| Folder HEAD > root HEAD | `WatchedFolderHEADConsistent` fails | exit 200, FROZEN_HUMAN | Manual `cyberos doctor --repair --reason "<text>"` |
| Folder HEAD file size != 8 bytes | `WatchedFolderHEADConsistent` skips that folder | exit 0 if no other failures | Operator investigates; may need `cyberos memory watch` |
| Doctor itself crashes mid-run | panic handler emits sev-1 + exit 1 | exit 1 (not 200) | Operator files bug; runs with `--verbose` for stack |
| Invariant exceeds latency budget | metric `memory_doctor_invariant_duration_seconds` p95 alarm | sev-2 alarm via FR-OBS-007 | Operator investigates folder count + filesystem health |
| 50+ folders, slow filesystem | `latency_test` may exceed 5s on CI | sev-2 alarm | Operator considers folder-count cap or fast-path |
| YAML invariant declared without Rust impl | `yaml_matches_rust` test fails in CI | PR blocked | Author adds the impl OR removes from YAML |
| Rust impl present without YAML declaration | warning at `cyberos doctor` startup | exit 0; warning in logs | Author adds YAML entry |
| Manifest.watched_folders[] empty | All 5 invariants pass trivially | exit 0, READY | By design — no folders to audit |
| State transition collision (two invariants ask for different states) | doctor picks the strictest (HUMAN > RECOVERABLE > READY) | reported in JSON | By design |

---

## §11 — Implementation notes

- The invariant catalog file (`watched-folders-v2.yaml`) MUST be loaded at startup; failing to find it is a hard error (the master `memory.invariants.yaml` references it via `include:`). This is symmetric with how `core.yaml` is loaded — both are first-class.
- `std::fs::canonicalize` is the chosen symlink-resolver; it returns the absolute realpath and follows every symlink. Per AGENTS.md §0.4 this is correct; do not roll a custom resolver.
- `WatchedFolderHEADConsistent` reads HEAD as 8 little-endian bytes per AGENTS.md §2. If the file is missing or has wrong size, the invariant logs a warning and treats the folder as "unknown HEAD" — does NOT fail (the missing-HEAD case is caught by `WatchedFolderManifestPresent` which requires the full fragment including HEAD).
- The `Skip` status with `skipped_because` is a first-class result variant — not a missing entry. JSON consumers iterate the array; they should not assume "missing = skip".
- The `state_transition` field is the AGENT state the failure WOULD trigger; the doctor itself does NOT mutate agent state. The `cyberos` runtime reads doctor's JSON output and applies the strictest transition. This preserves the invariant that doctor is read-only.
- The `--invariant` and `--only` filters are CLI-only conveniences; programmatic callers use `doctor::run_filtered(invariant: Option<&str>, only: Option<&str>)`.
- The CI integration uses `jq -e` to fail on any error-severity failure; warnings do not block CI but appear in the job log for operator review.
- Latency budgets are per-invariant per-folder; the wall-clock test bounds total run time. Tests use a fixture generator that creates N folders deterministically (no Date.now()) so the test is reproducible.
- The `tests/fixtures/wf_*` directories are committed to the repo (small JSON fragments + symlinks); they're the canonical demonstrations of each failure mode.

---

*End of FR-MEMORY-105.*
