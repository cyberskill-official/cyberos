# TASK-IMP-074 — batch run record + review packet (v2.5.0 first live use)
Batch: {TASK-IMP-074, TASK-IMP-075} — cone-independent (workflow/hooks/CI-docs vs apps/desktop Rust). Steps 1-12 lean bundle: context grounded in .githooks/pre-commit (41 lines, no --page wiring - migrate-frs.sh's own comment was aspirational), run-gates.sh 82-88, deploy.yml paths, build.sh manifest block. ADR skipped (infra wiring). No mocks. Status: `reviewing`, HALTED at HITL gate 1.

## §1 clause → evidence
| Clause | Evidence | ✓ |
|---|---|---|
| 1-2 status sync + non-blocking | pre-commit block (trigger regex, --page call, git add docs/status, warn-not-block); LIVE-PROVEN: the commit landing this packet regenerated+staged docs/status itself | ✅ |
| 3 workflow rule recorded | §11a status-sync bullet | ✅ |
| 4-7 batch selection/execution/HITL/rescan | §11a all four bullets; HITL restated per-FR; this very batch is the first sanctioned use | ✅ |
| 8 rules_sha + gate | build.sh fingerprint (V3 deterministic, V6 sensitive) + check-version-sync 5b (V4 pass, V5 negative) | ✅ |
| 9 hook chain build/release/deploy | pre-commit (existing) + payload-gate (existing) + release payload job (existing) + deploy.yml paths ADDED + documented in Distribution sync section | ✅ |
| 10 pull-side follow-up | §9 records client-side comparison as designated follow-up | ✅ |
Machine gates: V1-V7,V9 all PASS (see commit body). Coverage N/A declared (bash/YAML/md).
Verdict needed: "TASK-IMP-074 review: approved" or rejected+reason.
