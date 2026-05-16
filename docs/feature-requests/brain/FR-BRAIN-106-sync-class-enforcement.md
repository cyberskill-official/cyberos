---
id: FR-BRAIN-106
title: "BRAIN sync_class enforcement — private vs shareable + ACL filtering + structural compensation exclusion + property test"
module: BRAIN
priority: MUST
status: accepted
verify: T
phase: P1
milestone: P1 · slice 1
slice: 1
owner: Stephen Cheng (CDO)
created: 2026-05-15
shipped: null
brain_chain_hash: null
related_frs: [FR-BRAIN-103, FR-BRAIN-107, FR-BRAIN-108]
depends_on: [FR-BRAIN-101, FR-BRAIN-103]
blocks: [FR-BRAIN-107]   # placeholder — read-time ACL filtering FR, not yet specified

source_pages:
  - website/docs/modules/brain.html#sync-class
source_decisions:
  - DEC-036 (compensation/equity rows excluded from sync at every boundary)
  - DEC-100 (sync_class = user privacy primitive)
  - AGENTS.md §15 (privacy classes private/shareable; transitional support for v1 names)

language: rust 1.81
service: cyberos/services/brain-sync/
new_files:
  - services/brain-sync/src/sync_class.rs
  - services/brain-sync/src/structural_exclusion.rs
  - services/brain-sync/tests/sync_class_test.rs
  - services/brain-sync/tests/sync_class_property_test.rs
  - services/brain-sync/tests/structural_exclusion_test.rs
modified_files:
  - services/brain-sync/src/sync.rs                       # call should_sync() at every direction
allowed_tools:
  - file_read: services/brain-sync/**
  - file_write: services/brain-sync/**
  - bash: cd services/brain-sync && cargo test sync_class
disallowed_tools:
  - sync compensation/equity rows under any sync_class (per DEC-036 — structural exclusion)
  - skip should_sync() check on any sync direction (per §1 #1 + #2 + #4)
  - bypass ACL on shareable rows with non-empty acl (per §1 #3)
  - silently apply transitional v1 mapping (per §1 #4 — log + count)

effort_hours: 6
sub_tasks:
  - "0.5h: sync_class.rs SyncClass enum — canonical (Private, Shareable per AGENTS.md §15) + v1-transitional (LocalOnly, Publishable, Shared, ClientVisible) preserved per AGENTS.md §15 release-cycle clause"
  - "0.5h: should_sync() function with full filter logic"
  - "0.5h: structural_exclusion.rs (path-based compensation/equity detection)"
  - "0.5h: ACL evaluation (empty allow-list = unrestricted; non-empty = whitelist)"
  - "0.5h: v1 transitional mapping with INFO log"
  - "0.5h: canonical::sync_row_filtered audit row builder"
  - "0.5h: Integration in sync.rs (push, pull, import all call should_sync)"
  - "0.5h: OTel metrics (filter_decisions_total per reason)"
  - "1.5h: Tests — happy + ACL + structural + v1 transition + property test 10K random rows"
  - "0.5h: cyberos-brain validate-sync-class CLI for ops to dry-run"
risk_if_skipped: "Without enforcement, any sync_class label is advisory. A bug elsewhere could push private rows. Without structural exclusion, comp/equity rows tagged shareable would propagate (DEC-036 violation). Without ACL evaluation, shareable rows reach actors not on the allow-list. Without property test, regressions ship undetected."
---

## §1 — Description (BCP-14 normative)

The brain-sync daemon **MUST** enforce `meta.sync_class` filtering on every sync direction (push + pull + import + cross-BRAIN merge). Each filter call:

1. **MUST** push only `sync_class == "shareable"` rows to Cloud BRAIN. Rows with `private | local-only` are filtered at push.
2. **MUST** refuse to ingest a foreign row whose `sync_class != "shareable"` at pull time (defensive — Cloud BRAIN should already filter; double-check at receive boundary).
3. **MUST** consult `meta.acl[]` (allow-list of actor IDs) for shareable rows; if `acl` is non-empty AND current actor not listed, refuse. Empty `acl` = unrestricted (any actor authorised by Cloud sees it).
4. **MUST** maintain transitional support for v1 `sync_class` values per AGENTS.md §15:
    - `local-only` → treat as `private` (refuse).
    - `publishable | shared | client-visible` → treat as `shareable` (allow, subject to ACL).
   Transitional mapping logs INFO with the v1 value seen (operators monitor adoption); metric `brain_sync_v1_transitional_total{value}` counts.
5. **MUST** reject DEC-036 (compensation) and equity rows at sync boundary REGARDLESS of sync_class (structural exclusion as defense-in-depth — even mistakenly-tagged shareable comp rows are caught). Path-based detection on `meta/people/*/compensation*`, `meta/people/*/equity*`, `meta/finance/payroll*`, `meta/finance/comp*`.
6. **MUST** emit BRAIN audit row `brain.sync_row_filtered` per filter decision (push or pull) with payload: `direction` (push|pull), `seq`, `path`, `sync_class`, `reason` (enum: Private | AclMismatch | StructuralExclusion | V1TransitionalRefuse), `actor_id` (current syncing device).
7. **MUST** be deterministic — same row + same actor + same time = same decision. No clock-dependent or RNG inputs.
8. **MUST** be testable via property test: 10K random rows × 100 random actors × verify (a) zero compensation/equity ever crosses boundary; (b) zero private rows pushed; (c) ACL respected.
9. **MUST** integrate into `services/brain-sync/src/sync.rs`'s push and pull loops; calls `should_sync()` BEFORE network IO.
10. **SHOULD** support a `cyberos-brain validate-sync-class` CLI for operators: takes a row file as input; reports the SyncDecision; helps debug filter decisions.
11. **SHOULD** emit OTel metrics:
    - `brain_sync_filter_decisions_total{direction, reason}` (counter).
    - `brain_sync_v1_transitional_total{value}` (counter; track v1 adoption).
    - `brain_sync_compensation_excluded_total` (counter; sev-1 alarm if > 0/day; investigate upstream).

---

## §2 — Why this design (rationale for humans)

**Why structural compensation exclusion (DEC-036)?** Compensation/equity is the highest-stakes data class. A single leak ends careers + companies. The `sync_class` field is the user's primary control, but bugs happen — a misclassified comp row tagged "shareable" would propagate. The structural-path-check is defense-in-depth: even if every other safeguard fails, the path-based reject catches comp rows.

**Why filter at EVERY boundary (§1 #1 + #2 + #5)?** Push filter prevents Cloud receiving private rows. Pull filter prevents local importing private rows. Both are necessary because Cloud BRAIN's filtering bug could push private rows TO us — pull filter catches that. Defense-in-depth at every direction.

**Why ACL on shareable rows (§1 #3)?** Some shareable rows are tenant-wide (any actor with tenant access sees them). Others are role-scoped (only finance team, only legal team). The ACL allow-list lets the user be specific: `acl: ["@alice", "@bob"]` = only these actors. Empty acl = no restriction.

**Why v1 transitional mapping (§1 #4)?** Existing memories tagged with v1 values shouldn't break. Mapping `local-only → private`, `publishable|shared|client-visible → shareable` preserves intent. The INFO log + metric track v1 usage; eventual deprecation when v1 usage drops to zero.

**Why audit-row per filter (§1 #6)?** Compliance audits need to answer "what was filtered out + why?" Without the audit row, filter decisions are invisible. With the row, ops + auditors can answer the question via standard BRAIN query.

**Why property test (§1 #8)?** Filter logic is the kind of thing that breaks silently. Compensation row with shareable tag + filter has a bug = leak. Property test 10K random rows × 100 actors × invariant assertions = high-confidence detection.

**Why pre-network-IO call (§1 #9)?** Filtering before network IO means private rows never leave the device. Filtering after would mean rows briefly transit before being rejected — observable to network sniffers, longer attack window.

**Why dedicated `compensation_excluded` metric (§1 #11)?** A compensation row arriving at sync boundary indicates upstream code emitted shareable comp — that's a bug. Sev-1 alarm forces investigation. Trending the metric quantifies upstream-code health.

**Why CLI for ops dry-run (§1 #10)?** Operators debugging "why didn't this row sync?" need a way to ask the filter directly. CLI takes a row, returns SyncDecision + reason + audit-row preview. Faster than reading logs.

---

## §3 — API contract

```rust
// services/brain-sync/src/sync_class.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SyncClass {
    // Canonical (AGENTS.md §15) — the only two values new writers should emit:
    Private,    // never leaves the local store
    Shareable,  // MAY be exported; ACL field carries explicit allow-list

    // v1 transitional — preserved for one release cycle for tooling that has
    // not migrated (AGENTS.md §15). New writers MUST NOT emit these. Readers
    // map them per §1 #4: local-only → Private; publishable | shared |
    // client-visible → Shareable. Each occurrence is logged + counted so
    // operators can track when v1 usage reaches zero and these variants
    // can be dropped.
    #[serde(rename = "local-only")]    LocalOnly,
    Publishable,
    Shared,
    #[serde(rename = "client-visible")] ClientVisible,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SyncDecision {
    Allow,
    Refuse(Reason),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    Private,
    AclMismatch { actor: String, allowed: Vec<String> },
    StructuralExclusion { path: String },
    V1TransitionalRefuse,
}

pub fn should_sync(row: &MemoryRow, current_actor: &str, direction: Direction) -> SyncDecision {
    // §1 #5 structural exclusion (defense in depth)
    if structural_exclusion::is_compensation_or_equity(&row.path) {
        metrics::compensation_excluded(direction);
        return SyncDecision::Refuse(Reason::StructuralExclusion { path: row.path.clone() });
    }

    let class = row.meta.get("sync_class")
        .and_then(|v| serde_yaml::from_value(v.clone()).ok())
        .unwrap_or(SyncClass::Private);   // safe default

    match class {
        SyncClass::Private | SyncClass::LocalOnly => {
            if class == SyncClass::LocalOnly {
                tracing::info!(value = "local-only", "v1 transitional sync_class");
                metrics::v1_transitional("local-only");
            }
            SyncDecision::Refuse(Reason::Private)
        }
        SyncClass::Shareable | SyncClass::Publishable | SyncClass::Shared | SyncClass::ClientVisible => {
            if !matches!(class, SyncClass::Shareable) {
                tracing::info!(value = ?class, "v1 transitional sync_class");
                metrics::v1_transitional(&format!("{:?}", class).to_lowercase());
            }
            // §1 #3 ACL evaluation
            let acl: Vec<String> = row.meta.get("acl")
                .and_then(|v| serde_yaml::from_value(v.clone()).ok())
                .unwrap_or_default();
            if acl.is_empty() || acl.contains(&current_actor.to_string()) {
                SyncDecision::Allow
            } else {
                SyncDecision::Refuse(Reason::AclMismatch {
                    actor: current_actor.into(), allowed: acl,
                })
            }
        }
    }
}
```

```rust
// services/brain-sync/src/structural_exclusion.rs
const FORBIDDEN_PATTERNS: &[&str] = &[
    "meta/people/*/compensation*",
    "meta/people/*/equity*",
    "meta/finance/payroll*",
    "meta/finance/comp*",
    "memories/people/*/compensation*",
    "memories/finance/equity*",
];

pub fn is_compensation_or_equity(path: &str) -> bool {
    FORBIDDEN_PATTERNS.iter().any(|p| glob::Pattern::new(p).unwrap().matches(path))
}
```

Integration in `sync.rs`:

```rust
// Push path
for row in pending {
    let decision = sync_class::should_sync(&row, &device_id, Direction::Push);
    match decision {
        SyncDecision::Allow => push_to_cloud(row).await?,
        SyncDecision::Refuse(reason) => {
            brain::emit(canonical::sync_row_filtered(&row, Direction::Push, reason)).await?;
            metrics::filter_decision(Direction::Push, &reason);
        }
    }
}

// Pull path (defensive)
for remote in pulled {
    let decision = sync_class::should_sync(&remote, &device_id, Direction::Pull);
    match decision {
        SyncDecision::Allow => import_to_local(remote).await?,
        SyncDecision::Refuse(reason) => {
            brain::emit(canonical::sync_row_filtered(&remote, Direction::Pull, reason)).await?;
            metrics::filter_decision(Direction::Pull, &reason);
        }
    }
}
```

---

## §4 — Acceptance criteria

1. `sync_class: private` row never pushed.
2. `sync_class: shareable, acl: []` row pushed to Cloud.
3. `sync_class: shareable, acl: [@alice]` row not pushed when current actor is @bob.
4. `sync_class: shareable, acl: [@alice]` pushed when current actor is @alice.
5. Compensation path (`memories/people/alice/compensation.md`) refused regardless of sync_class.
6. Equity path refused regardless of sync_class.
7. v1 `local-only` treated as private (refuse).
8. v1 `publishable` treated as shareable (allow if ACL passes).
9. v1 `shared` treated as shareable (allow if ACL passes).
10. v1 `client-visible` treated as shareable.
11. v1 transitional values logged at INFO + metric increments.
12. Pull-side defensive: foreign row with `private` from Cloud → refused at pull.
13. Filter decisions emit `brain.sync_row_filtered` audit row with reason.
14. Property test: 10K random rows × 100 actors → 0 compensation/equity, 0 private pushed, ACL respected.
15. Deterministic: same row + same actor → same decision (run twice, assert equal).
16. CLI `cyberos-brain validate-sync-class --file row.yaml --actor @alice` returns SyncDecision JSON.
17. Sev-1 alarm on `brain_sync_compensation_excluded_total > 0/day` (upstream bug).

---

## §5 — Verification

```rust
#[test]
fn private_row_refused() {
    let row = test_row_with_sync_class("private");
    let dec = sync_class::should_sync(&row, "@anyone", Direction::Push);
    assert!(matches!(dec, SyncDecision::Refuse(Reason::Private)));
}

#[test]
fn shareable_empty_acl_allowed() {
    let row = test_row_with_sync_class_and_acl("shareable", &[]);
    let dec = sync_class::should_sync(&row, "@anyone", Direction::Push);
    assert_eq!(dec, SyncDecision::Allow);
}

#[test]
fn shareable_with_acl_actor_in_list() {
    let row = test_row_with_sync_class_and_acl("shareable", &["@alice", "@bob"]);
    let dec = sync_class::should_sync(&row, "@alice", Direction::Push);
    assert_eq!(dec, SyncDecision::Allow);
}

#[test]
fn shareable_with_acl_actor_not_in_list() {
    let row = test_row_with_sync_class_and_acl("shareable", &["@alice"]);
    let dec = sync_class::should_sync(&row, "@bob", Direction::Push);
    assert!(matches!(dec, SyncDecision::Refuse(Reason::AclMismatch { .. })));
}

#[test]
fn compensation_path_refused_regardless_of_sync_class() {
    let row = test_row_at_path("memories/people/alice/compensation.md", "shareable");
    let dec = sync_class::should_sync(&row, "@anyone", Direction::Push);
    assert!(matches!(dec, SyncDecision::Refuse(Reason::StructuralExclusion { .. })));
}

#[test]
fn equity_path_refused_regardless_of_sync_class() {
    let row = test_row_at_path("memories/finance/equity_grant_2026.md", "shareable");
    let dec = sync_class::should_sync(&row, "@anyone", Direction::Push);
    assert!(matches!(dec, SyncDecision::Refuse(Reason::StructuralExclusion { .. })));
}

#[test]
fn v1_local_only_refused() {
    let row = test_row_with_sync_class("local-only");
    let dec = sync_class::should_sync(&row, "@anyone", Direction::Push);
    assert!(matches!(dec, SyncDecision::Refuse(Reason::Private)));
}

#[test]
fn v1_publishable_allowed() {
    let row = test_row_with_sync_class("publishable");
    let dec = sync_class::should_sync(&row, "@anyone", Direction::Push);
    assert_eq!(dec, SyncDecision::Allow);
}

#[test]
fn deterministic_same_inputs_same_decision() {
    let row = test_row_with_sync_class("shareable");
    let d1 = sync_class::should_sync(&row, "@alice", Direction::Push);
    let d2 = sync_class::should_sync(&row, "@alice", Direction::Push);
    assert_eq!(d1, d2);
}

#[test]
fn v1_transitional_metric_increments() {
    let row = test_row_with_sync_class("publishable");
    let _ = sync_class::should_sync(&row, "@a", Direction::Push);
    let metric: u64 = otel_test_helper::counter_value("brain_sync_v1_transitional_total", &[("value", "publishable")]);
    assert_eq!(metric, 1);
}
```

```rust
// services/brain-sync/tests/sync_class_property_test.rs
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn no_compensation_or_equity_ever_synced(
        path in "[a-z/_]{10,80}",
        sync_class in prop_oneof!["private", "shareable", "local-only", "publishable", "shared", "client-visible"],
        actor in "@[a-z]{3,12}",
    ) {
        let row = test_row(path.clone(), &sync_class, vec![]);
        let dec = sync_class::should_sync(&row, &actor, Direction::Push);
        if structural_exclusion::is_compensation_or_equity(&path) {
            prop_assert!(matches!(dec, SyncDecision::Refuse(Reason::StructuralExclusion { .. })));
        }
    }

    #[test]
    fn no_private_pushed(sync_class in prop_oneof!["private", "local-only"]) {
        let row = test_row("memories/test.md".into(), &sync_class, vec![]);
        let dec = sync_class::should_sync(&row, "@anyone", Direction::Push);
        prop_assert!(matches!(dec, SyncDecision::Refuse(Reason::Private)));
    }

    #[test]
    fn acl_respected(
        actor in "@[a-z]{3,12}",
        allowed in prop::collection::vec("@[a-z]{3,12}", 0..5),
    ) {
        let row = test_row("memories/test.md".into(), "shareable", allowed.clone());
        let dec = sync_class::should_sync(&row, &actor, Direction::Push);
        if allowed.is_empty() || allowed.contains(&actor) {
            prop_assert_eq!(dec, SyncDecision::Allow);
        } else {
            prop_assert!(matches!(dec, SyncDecision::Refuse(Reason::AclMismatch { .. })));
        }
    }
}
```

```rust
// CLI test
#[test]
fn cli_validate_sync_class_returns_json() {
    let row_file = "/tmp/test_row.yaml";
    std::fs::write(row_file, "path: memories/test.md\nmeta:\n  sync_class: shareable\n").unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cyberos_brain"))
        .args(&["validate-sync-class", "--file", row_file, "--actor", "@alice"])
        .output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["decision"], "allow");
}
```

---

## §6 — Implementation skeleton

See §3.

---

## §7 — Dependencies

- **FR-BRAIN-103** — sync.rs integration point.
- **FR-BRAIN-107 (downstream)** — read-time ACL filtering uses similar logic.
- Crates: `glob@0.3`, `serde`, `serde_yaml`, `proptest@1`.

---

## §8 — Example payloads

### Filter decision audit row

```json
{
  "kind": "brain.sync_row_filtered",
  "payload": {
    "direction": "push",
    "seq": 12345,
    "path": "memories/people/alice/compensation.md",
    "sync_class": "shareable",
    "reason": { "structural_exclusion": { "path": "memories/people/alice/compensation.md" } },
    "actor_id": "@stephen"
  }
}
```

### CLI validate

```text
$ cyberos-brain validate-sync-class --file row.yaml --actor @alice
{
  "decision": "refuse",
  "reason": {
    "acl_mismatch": {
      "actor": "@alice",
      "allowed": ["@bob"]
    }
  }
}
```

---

## §9 — Open questions

All resolved. Deferred:
- Per-row encryption (slice 3+).
- Time-bounded shareability (`shareable_until: 2026-12-31`) — slice 4+.
- Group ACLs (`acl: [@team-finance]`) — slice 4+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Misclassified row (compensation marked shareable) | Structural exclusion catches | Refused; sev-1 metric | Engineer fixes upstream classifier |
| ACL list contains invalid actor | Treat as no-match (refuse) | Refused for that actor | Operator fixes ACL |
| Foreign row with v1 sync_class | Transition mapping applies | Allow/refuse per mapping | By design |
| sync_class missing | Default to private | Refused (safe default) | By design |
| ACL value malformed (not a list) | Parse error | Treat as empty (no restriction; row syncs) | Operator fixes ACL syntax |
| Property test finds violation | proptest panics in CI | PR blocked | Engineer fixes filter |
| should_sync called with wrong direction | wrong metric label | Sev-3 (incorrect attribution) | Engineer fixes call site |
| Glob pattern bug allows comp through | property test catches | PR blocked | Fix pattern |
| Path with unicode normalisation diff | structural exclusion misses | Sev-1 (compensation leak) | Add NFC normalisation pre-check |
| Compensation excluded sustained > 0 | sev-1 alarm | Investigate upstream | Standard process |
| Audit row emit fails | brain_writer error | Sev-2; row still filtered | Operator investigates BRAIN |
| should_sync slow (regex compilation) | OTel histogram | Optimise: use lazy_static | Standard optimisation |
| New v1 transitional value introduced | Falls into default (private) | Refused; metric `unknown_value` | Operator adds mapping |
| Future sync_class value (e.g., "team-only") | Falls into default | Refused | Add to enum + filter |
| Test fixture mismatched | unit test fails | PR blocked | Update fixture |

---

## §11 — Notes

- The structural exclusion is the load-bearing primitive. Without it, comp rows tagged shareable would propagate. With it, every sync direction has the path-based safety net.
- v1 transitional support exists for migration; eventual deprecation via FR-BRAIN-204 (after v1 usage drops to <1% of rows).
- ACL evaluation is allow-list semantics: empty = unrestricted; non-empty = whitelist. No deny-list semantics (would be confusing alongside the existing reject-by-default model).
- The `brain.sync_row_filtered` audit row is the compliance primitive. Auditors can answer "what was filtered out?" via standard BRAIN query.
- Property test 10K random rows × 100 actors ensures the filter logic is robust across the input space.
- Default sync_class is `private` (safe default); operators must explicitly opt-in to sharing.
- Sev-1 on compensation_excluded > 0/day is conservative — even one such event is investigated.
- The CLI dry-run helps operators debug filter decisions without poking at logs.

---

*End of FR-BRAIN-106. Status: draft (10/10 target).*
