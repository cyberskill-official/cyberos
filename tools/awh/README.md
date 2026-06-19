# tools/awh - vendored auto-work-harness

The out-of-band verification gate for CyberOS. Vendored from
github.com/zintaen/auto-work-harness at `c1f2c77`. Pure stdlib core plus PyYAML.

Install for dev / CI / pre-commit:

    pip install -e tools/awh

CLI: `awh adopt | eval | lock | firewall | power | mutate | worktree | maturity`.
Per-module golden sets live at `modules/<module>/.awh/goldenset.yaml`; the gate is
`awh eval <goldenset> --base-dir . --baseline <baseline> --max-regression 0.0`.

Maturity ledger: `.awh/evolution-log.jsonl` (migrated from the standalone repo,
6 prior adoptions). The standalone repo is archived only after the vendored gate
runs real work green across MEMORY + SKILL.
