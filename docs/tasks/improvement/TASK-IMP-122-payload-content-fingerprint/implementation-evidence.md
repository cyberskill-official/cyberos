# IMP-122 implementation evidence

- `tools/install/lib/rules-cone.sh` — shared cone + exclusions + `_rsha` / `_rules_sha_of`
- `tools/install/build.sh` — sources shared lib; reconciler (§1.6/§1.7) with `CYBEROS_SKIP_RULES_RECONCILE` escape for fixtures
- `tools/install/version.sh`, `lib/update-check.sh`, `audit-fleet.sh` — installed side recomputed
- Noise filter: `__pycache__/` + `*.pyc` skipped
- Suite: `tools/install/tests/test_rules_sha_recompute.sh` registered in `scripts/tests/run_all.sh`
