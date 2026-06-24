---
id: FR-GAM-003
title: "Consent-gated usage ranking from shell history"
module: GAM
priority: MUST
status: done
fidelity: as-built
shipped: 2026-06-23
owner: Stephen Cheng
source_repo: zintaen/gam @ f55d97c
related_frs: [FR-GAM-004]
---

## §1 — Description (BCP-14 normative)

gam MAY rank aliases by how often they are used, derived from local shell history. Because this reads the user's history, the behavior MUST be disclosed and MUST be reversible.

1. The app MAY read local shell history (zsh, bash, fish, PowerShell) to rank aliases by frequency of use.
2. Ranking is default-on, but the app MUST disclose, in plain language, that it reads shell history and that nothing read leaves the machine.
3. The user MUST be able to turn ranking off. Turning it off MUST stop reading history and MUST clear any cached ranking immediately.
4. Nothing read from shell history MUST ever leave the device (no network egress).
5. The consent choice MUST persist across sessions and MUST be loaded at startup (see FR-GAM-004).

## §2 — Why this design

Frequency ranking is genuinely useful, but reading shell history is sensitive, so it cannot be a silent default with no exit. The design makes the behavior visible (a disclosure panel), reversible (one toggle that both stops reading and purges the cache), and local-only (the data never crosses the process boundary to the network). Default-on is acceptable only because all three of those properties hold and the disclosure is in the user's face in the same panel as the toggle.

## §3 — Implementation

- `src-tauri/src/ranking_service.rs` — the ranking engine with an `enabled: bool` gate. `set_enabled(false)` clears the cached scores; `get_scores()` returns zeroed scores while disabled so no history is read.
- `src-tauri/src/commands.rs` — `get_history_ranking_enabled` and `set_history_ranking_enabled` IPC commands.
- `src/hooks/useHistoryRanking.ts` — frontend hook, storage key `gam-history-ranking-enabled`, default enabled.
- `src/components/settings/PrivacyPanel.tsx` — the disclosure and the toggle (`role="switch"`, aria-labeled), with the plain-language text about what is read and that nothing is sent anywhere.
- `src-tauri/src/lib.rs` — loads the consent setting at startup.
- Persisted to the app settings file as `historyRankingEnabled` (FR-GAM-004).

## §4 — Acceptance criteria

1. Default state is on; the Privacy panel discloses what is read and that nothing leaves the machine.
2. Turning the toggle off sets `historyRankingEnabled` to `false` in the settings file.
3. While off, `get_scores()` returns zeroed scores and history is not read.
4. The cached ranking is cleared on opt-out.
5. Turning it back on restores ranking and persists `true`.
6. The choice survives a restart (loaded at startup).

## §5 — Verification

- `tests/hooks/useHistoryRanking.test.ts` — default-enabled, reads stored opt-out, `setEnabled(false/true)` persists to storage.
- `tests/components/PrivacyPanel.test.tsx` — discloses "shell history" and "Nothing is sent anywhere"; toggle defaults on and turns off.
- `src-tauri/src/ranking_service.rs` integration test — disabled gate returns zero scores and clears the cache.
- Live GUI verification (2026-06-23): flipping the toggle wrote `"historyRankingEnabled":"false"` then `"true"` to the app settings file, round-trip confirmed on disk.

## §6 — Failure modes

| Failure | Detection | Outcome |
|---|---|---|
| Shell history unreadable | read error | empty ranking; app still works |
| Setting missing at startup | loader default | defaults to enabled (disclosed) |
| User opts out | enabled gate | reads stop, cache cleared |

## §7 — Notes

This is the one feature that reads user data, so it carries the project's only consent surface. The toggle is a 16px checkbox inside a `<label>`; clicking the label text flips it, which is how the live test drove it.

*End of FR-GAM-003. Fidelity: as-built (10/10 target).*
