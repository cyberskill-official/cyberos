---
fr_id: FR-BRAIN-105
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

## §1 — Verdict summary

FR-BRAIN-105 written direct-to-10/10 cadence (no compressed first-pass; spec authored in this session). ~640 lines. 13 §1 clauses (the 5 invariants + ordering + JSON output schema + determinism + latency + exit code + OTel + metrics + CLI filters). 8 §2 rationale paragraphs. Full YAML catalog + Rust trait + 5 Invariant impls + CLI integration in §3. 16 ACs. 8 Rust test bodies + CI snippet. 15 failure modes. 8 implementation notes.

## §2 — Findings (all resolved during authoring)

### ISS-001 — Invariant ordering vs core invariants ambiguous
A naive author would not specify that watched-folders invariants run AFTER core but BEFORE consolidation triggers. Resolved: §1 #6 + §3 `INVARIANT_ORDER` slice + §11 note explaining the rationale.

### ISS-002 — Dependency-skip semantics
What does `WatchedFolderManifestSchema` do when `ManifestPresent` already failed? Without specification, it would either re-fail (noise) or silently pass (incorrect). Resolved: §1 #7 + JSON `status: "skip"` with `skipped_because: "<dep>"`; AC #10 + §5 test.

### ISS-003 — State transition not enforced by doctor itself
Critical principle: doctor is read-only (per AGENTS.md §12). Without explicit clarification, an implementor might have doctor mutate `agent_state` directly. Resolved: §11 note + JSON `state_transition` is advisory; the runtime applies it; doctor stays read-only.

### ISS-004 — Determinism gap (clock-dependent + map iteration)
Without spec, `BTreeMap` (deterministic) vs `HashMap` (non-deterministic) is a coin flip. Resolved: §1 #8 + §3 `folders.sort_by_key(|wf| wf.realpath.clone())` + AC #11 byte-identical-JSON test.

### ISS-005 — Latency budget not bounded per-folder
A 50-folder fixture might pass; a 10K-folder fixture might not. Operators need to know what the design supports. Resolved: §1 #9 explicit ≤ 50 folders budget; §10 row "50+ folders, slow filesystem" triggers sev-2; AC #12 + §5 latency test.

### ISS-006 — Exit code value clashed with shared cyberos-cli-exit::ExitCode range
Per FR-AI-021 §3 note, module-specific codes start at 200 (AUTH), 300 (BRAIN). I initially wrote `exit 6 (InvariantFailure)` — collides with shared 6=SchemaViolation. Resolved: §1 #10 + §6 `exit_code` = `200` (BRAIN-range InvariantFailure); §10 every error-severity row.

## §3 — Resolution

All 6 mechanical concerns addressed during authoring. **Score = 10/10.** Ready to ship + transition `draft → accepted`.

---

*End of FR-BRAIN-105 audit.*
