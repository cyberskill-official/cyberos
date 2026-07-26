# Status v3 — P2 visual review package (HITL)

Date: 2026-07-27  
Task: TASK-DOCS-013 / Gate P2  
Operator: Stephen  
**Human visual acceptance is NOT done** — this package only prepares the review.

## Open these

| Surface | URL |
| --- | --- |
| Local served (v3) | http://127.0.0.1:8877/index.html |
| Local served (legacy) | http://127.0.0.1:8877/status-legacy.html |
| Local `file://` (v3) | `file:///Users/stephencheng/Projects/CyberSkill/cyberos-wt-status-v3/docs/status/index.html` |
| Local `file://` (legacy) | `file:///Users/stephencheng/Projects/CyberSkill/cyberos-wt-status-v3/docs/status/status-legacy.html` |
| Live docs (v3) | https://os.cyberskill.world/docs/reference/status.html |
| Live docs (legacy) | https://os.cyberskill.world/docs/reference/status-legacy.html |

Re-serve locally if needed:

```bash
cd /Users/stephencheng/Projects/CyberSkill/cyberos-wt-status-v3
python3 -m http.server 8877 --bind 127.0.0.1 --directory docs/status
```

## Agent-captured screenshots (viewport)

| File | What |
| --- | --- |
| [`paper-top.png`](paper-top.png) | v3 paper theme — Pulse band |
| [`night-top.png`](night-top.png) | v3 night theme — Pulse band |
| [`legacy-top.png`](legacy-top.png) | v2 lenses page (`status-legacy.html`) |

## Machine evidence (already green)

- DOM suite: `bash tools/docs-site/tests/test_status_dom.sh` → **pass=50 fail=0**
- Feed unit tests: `bash tools/docs-site/tests/test_status_feed.sh` → **pass=13 fail=0**
- Offline cert: `bash tools/install/tests/test_offline_status_cert.sh` → **pass=9 fail=0**
- Regenerated mothership page: VERSION **1.11.0**, `status-hub@3`, `status-legacy.html` present, footer legacy link present

## Review checklist (Stephen)

Walk each item; reply **approve** or list **changes**.

1. [ ] **Paper theme** — first viewport reads as one composition; brand/title clear; no broken layout
2. [ ] **Night theme** — toggle works; contrast OK; charts/cards still readable
3. [ ] **`file://` offline** — open local `file://` URL; no CDN/network dependency; page usable with JS on
4. [ ] **Served** — local `http://127.0.0.1:8877/` and/or live docs URL behave the same as `file://` for core canvas
5. [ ] **Six bands** — Pulse / Roadmap / System map / Flow / Releases & traceability / Index all present and scroll-anchored
6. [ ] **Task drawer** — open a task; deps/cone/spec link look right
7. [ ] **Legacy link** — footer “Legacy status page (v2 lenses)” reaches `status-legacy.html`; Board/Table/Releases lenses work
8. [ ] **Deep links** — spot-check `#t/…`, `#m/…`, `#r/…` (and a legacy `#board` / `#roadmap` redirect)
9. [ ] **Traceability band** — unlinked count / rule text matches expectations (148 current-epoch is expected pre-cutoff truth)
10. [ ] **Noscript** — optional: disable JS, confirm static table still lists tasks

## Out of scope for P2

- Approving v2.0.0 / Release-As / tagging (P6)
- Fleet install/rollout (P7)
- Mass ledger backfill of the 148 unlinked commits (see `docs/notes/status-v3-do021-violation-triage.md`)
