---
fr_id: FR-GAM-003
audited: 2026-06-24
verdict: PASS (as-built)
score: 10/10
fidelity: as-built
template: as-built-verification@1
---

## §1 — Verdict summary

FR-GAM-003 is shipped, tested at both layers, and additionally proven by live GUI verification with on-disk confirmation. The consent surface meets all three required properties: disclosed, reversible (stop + cache clear), and local-only.

## §2 — Clause to artefact traceability

| §1 Clause | Artefact | Test | Status |
|---|---|---|---|
| #1 read shell history for ranking | `ranking_service.rs` | ranking_service integration test | OK |
| #2 disclosure + default-on | `PrivacyPanel.tsx` | `PrivacyPanel.test.tsx` (discloses shell history + "Nothing is sent anywhere") | OK |
| #3 reversible: stop + clear cache | `ranking_service.rs` `set_enabled(false)` clears cache; `get_scores` zeroed | ranking_service integration test | OK |
| #4 local-only, no egress | no network call in the read path | code review; nothing in `tauri.conf.json` permits it | OK |
| #5 consent persists + startup load | `commands.rs` IPC + `lib.rs` startup load + settings (`historyRankingEnabled`) | `useHistoryRanking.test.ts`; live settings.json round-trip | OK |

## §3 — Verification record

```bash
cd apps/gam && pnpm test          # useHistoryRanking + PrivacyPanel suites
cd src-tauri && cargo test --lib --locked   # ranking_service enabled-gate test
```

Live GUI (2026-06-23): on a real release build, opening the Privacy panel and flipping "Usage ranking" off then on wrote `"historyRankingEnabled":"false"` then `"true"` to `~/Library/Application Support/com.github.zintaen.gam/settings.json`. The file did not exist before the first flip, proving the write path end to end.

## §4 — Status

`accepted → shipped`. This is the strongest-verified FR in the module (unit + integration + live on-disk).

*End of FR-GAM-003 audit.*
