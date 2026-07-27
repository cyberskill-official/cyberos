# Acceptance fixtures

Twenty reports: ten from run 1 (specification 1.0, 69-discipline ledger) and ten
from run 2 (specification 1.2, 75-discipline ledger) on the same targets at the
same commits. Both sets pass because the linter version-gates on `INSPECT-SPEC`.
Keeping both is deliberate: the pair is evidence of what the amendment changed.

| Fixture stem | Spec | Notes |
|---|---|---|
| `*.golden.md` (non-r2) | 1.0 | 69 rows; must stay valid |
| `*.r2.golden.md` | 1.2 | 75 rows |

Original five-fixture spread (my-cv, issue-hunter, dom-defender, gam,
kristen-calendar) still anchors severity and applicability range; the other five
targets and the r2 set extend coverage.

## Running them

```bash
node ../tools/inspect-lint.mjs --selftest
for f in *.golden.md; do node ../tools/inspect-lint.mjs "$f" || echo "FAIL $f"; done
```

All must exit 0. Do not edit a fixture to make a change pass — bump the contract
version instead.
