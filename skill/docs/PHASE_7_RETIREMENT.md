# Phase 7 — Legacy Retirement Runbook

> **Do not execute this runbook yet.** Phase 5 (WASM execution path) and
> Phase 6 (capability broker GA) must be fully green on the entire
> CyberSkill skill catalogue under real load before the legacy Python
> runners can be retired.

## Pre-conditions

Before kicking off retirement, ALL of these must hold:

- [ ] `cargo build --features wasm` succeeds on the target platform.
- [ ] `wasm32-wasi` target installed on every developer + CI machine.
- [ ] Every skill in `skill/skills/` has a `dist/skill.wasm` component
      compiled by the Bun toolchain.
- [ ] `python skill/tests/parity/run_parity.py` reports **zero** failures
      across all skills and all fixtures.
- [ ] Criterion benchmarks show Rust-host throughput >= 2x the Python-runner
      baseline (already proven for the registry layer; Phase 7 needs the
      same proof for end-to-end invocation).
- [ ] `cyberos-skill cap audit` shows no unaudited capability grants.
- [ ] The host has shipped on `--executor=wasm` as the default in
      `cyberos-skill run` for at least **one release cycle (~30 days)** with
      zero P0 incidents.
- [ ] Documentation updated: `skill/README.md`, `skill/docs/SPEC.md`,
      `skill/docs/PUBLISH.md` no longer reference the legacy Python tier.

## Execution steps

1. **Tag a pre-retirement release.**

   ```bash
   git tag -a skill-pre-retirement-$(date +%Y%m%d) \
     -m "Skill module state pre-Python-removal"
   ```

2. **Delete the Python runners.**

   ```bash
   git rm -r skill/runners/
   git rm skill/pyproject.toml
   ```

3. **Delete the legacy script tier from invoke handling.**

   In `skill/crates/cli/src/main.rs`:
   - Remove `primary_script()` and the script-path branch of `run_skill()`.
   - `pick_executor()` returns only `Ok("wasm")`; remove `"script"` and
     `"auto"` branches.

4. **Delete the parity harness.**

   ```bash
   git rm -r skill/tests/parity/
   ```

   (Parity has been proven; the harness is no longer load-bearing.)

5. **Strip script-tier references from skill SKILL.md docs.**

   Several SKILL.md files mention `scripts/*.py` as the entry point. Update
   each to reference `dist/skill.wasm` and the WIT-bindgen-generated trait.

6. **Bump the major version.**

   In `skill/Cargo.toml` workspace: `version = "1.0.0"`. Major bump signals
   the retirement to consumers.

7. **Update PUBLISH.md.**

   Drop the "Phase 1" tarball workflow paragraph; the unified workflow
   becomes: build -> componentize -> cosign -> push to OCI / agentskills.io.

8. **CHANGELOG entry.** Newest-first, dated. Subject: "Skill module —
   Python runner tier retired (Phase 7)." Body: list every script removed +
   every SKILL.md updated + version bump.

## Rollback

If a P0 hits within 7 days of retirement:

1. `git revert <retirement-commit>` (or
   `git reset --hard skill-pre-retirement-<date>` on a branch).
2. Re-publish the previous Cargo version.
3. Open a post-mortem; do not re-attempt retirement until the root cause is
   fixed AND a fresh >= 30-day soak window has elapsed.

## Notes

- The `vn-legal-compliance` skill is markdown-only (no scripts, no WASM).
  It is unaffected by the retirement.
- The `BaseSkillRunner` Python framework lived under `skill/runners/base.py`.
  Some downstream tooling (e.g. `runtime/skill_runners/` callers — none in
  the current repo, but check before deleting) may still import it. Grep
  before deleting.
- Cosign signature verification (`crates/resolver/`) is Phase 6 work, not
  Phase 7. It must be live before retirement so the OCI distribution path
  is trustworthy.
