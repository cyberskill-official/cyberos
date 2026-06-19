# Retiring the standalone auto-work-harness

Goal: once CyberOS fully takes over verification, retire the standalone
`~/Projects/auto-work-harness`. "Retire" means archive first (history is never lost), then the
working copy is safe to delete. This honors the absorb-then-archive rule from the absorption
brief; do not `rm -rf` a repo whose history is not preserved elsewhere.

## Preconditions (all must hold before retiring)

1. Every roadmap module is green under the vendored gate. Run, on a machine with the
   toolchain: `bash scripts/awh_bootstrap_waves.sh`. Expect GREEN for memory, skill, cuo,
   auth, chat, proj, email.
2. The vendored copy is self-contained. Verified: nothing in the gate machinery
   (`tools/awh`, `scripts`, `.github/workflows/awh-gate.yml`, `.pre-commit-hooks/awh-gate.sh`,
   per-module `gate.sh`) references the external repo except a description string in
   `tools/awh/pyproject.toml`. Deleting the standalone breaks nothing in CyberOS.
3. The maturity ledger is migrated to `.awh/evolution-log.jsonl` (done; pass
   `--log .awh/evolution-log.jsonl` to every `awh maturity` call).

## Step A - upstream the gate fix before freezing (important)

The independent review found a real gate bypass in awh's `gate()` (a current task absent from
the baseline was silently skipped). It is fixed in the vendored copy AND in the standalone
working copy this session, with a test. The other repos that adopted awh
(`CyberSkill/shared`, `Personal/gam`, `CyberSkill/styx-landing-page`, `CyberSkill/cyber-click`,
`CyberSkill/shopass`) share the bug, so commit and push the fix to the standalone's origin so
they can pull it, BEFORE archiving:

```bash
cd ~/Projects/auto-work-harness
make verify
git add harness/stage1_measurement/runner.py tests/test_stage1_runner.py
git commit -m "fix(gate): fail closed when a current task is absent from the baseline"
git push
```

## Step B - archive (preserve history)

```bash
cd ~/Projects/auto-work-harness
git tag -a archive/pre-cyberos-absorb -m "archived after vendoring into cyberos 2026-06-19"
git push origin archive/pre-cyberos-absorb
git bundle create ~/Projects/auto-work-harness.bundle --all
```

History now lives in three places: the tag, origin, and the local bundle.

## Step C - delete the working copy (optional, now safe)

```bash
rm -rf ~/Projects/auto-work-harness
```

The canonical awh is now `tools/awh/` inside CyberOS. To recover the standalone later:
`git clone ~/Projects/auto-work-harness.bundle`.

## Step D - reflect the change

CyberOS docs already describe the native gate: `website/docs/architecture/verification-gate.html`,
the CHANGELOG unreleased section, and the per-module `.awh/`. Nothing further to do here once
Steps A through C are complete.
