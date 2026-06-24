---
nfr_id: NFR-GAM-003
audited: 2026-06-24
verdict: PASS (as-built)
score: 10/10
fidelity: as-built
template: as-built-verification@1
---

## §1 — Verdict summary

NFR-GAM-003 is satisfied. `gam-gate.yml` is self-contained, passes actionlint, installs the Linux Tauri libraries before cargo, and runs the full check set; the equivalent upstream CI is green on all three OSes.

## §2 — Statement to artefact traceability

| §1 Clause | Artefact | Verification | Status |
|---|---|---|---|
| #1 full check set | `gam-gate.yml` steps | upstream green run (lint/types/tests/build/clippy/cargo test/cargo-deny) | OK |
| #2 self-contained | `gam-gate.yml` uses only public actions | grep: no `cyberskill-world/.github`; actionlint OK | OK |
| #3 Linux Tauri libs first | apt step gated `runner.os == 'Linux'` | ubuntu job compiled glib after the fix | OK |
| #4 scoped to apps/gam | `paths:` filter | workflow trigger | OK |

## §3 — Verification record

```bash
actionlint .github/workflows/gam-gate.yml          # OK
grep -n 'cyberskill-world/.github' .github/workflows/gam-gate.yml   # no match
```

Upstream CI (zintaen/gam, post-rotation): Check ubuntu/macOS/windows + Security Audit + PR Title + Dependency Review all green.

## §4 — Status

`accepted → shipped`. Recommend marking required in branch protection on merge.

*End of NFR-GAM-003 audit.*
