# T4 cross-model results — Gemini (Antigravity) vs Claude, 2026-06-13

Stephen ran the runner kickoff (Phases 0–2, discovery only) via **Antigravity / Gemini 3 Pro** on all three pilot targets, each writing to its own `docs/BACKLOG-gemini-*.md` (Claude's backlogs untouched). This is the first real cross-model run — the evidence gate the queued protocol candidates were waiting on.

## The three diffs

### 1. kymondongiap — SAME issues, DIFFERENT severity (Gemini audited Claude's post-fix state)
Gemini ran as "Loop 2" on the working tree that already has Claude's 7 fixes, and independently **confirmed Claude's work** (240 passed, build exit 0, `require_user` present in main.py). Its 3 new Highs:
- **L2-T1 Correctness** — `HallucinationGuardrail._term_to_id` hardcoded 6-term stub.
- **L2-T2 Security** — CORS `allow_origin_regex=.*\.vercel\.app` + `allow_credentials=True`.
- **L2-T3 Maintainability** — failing linters (24 frontend, 29 ruff).

**All three were already in Claude's BELOW-FLOOR list** — zero net-new findings. The divergence is pure **severity calibration**: Gemini scored them High; Claude filed them below the High floor. **Sharpest signal:** Claude's own below-floor note on CORS said it *"becomes High the moment cookie auth is added (pairs with L1-T2)."* Claude then DID add auth (T2) — but never re-classified CORS. Gemini, auditing fresh, scored it High. **Gemini's elevation is vindicated by Claude's own stated rule.**

### 2. 3d-preriodic-table — Gemini caught a real High Claude MISSED
Gemini audited the post-fix state (suite already 11/11 green from Claude's localStorage fix → correctly did not re-find it) and surfaced **1 High Performance: two Three.js versions loaded** (0.184.0 + 0.170.0 via `stats-gl`) → `THREE.WARNING: Multiple instances of Three.js being imported`; fix via npm `overrides`. Claude noted the 2.2 MB bundle *size* below-floor but NOT the multi-instance correctness/runtime issue. **Net-new real finding — a Claude miss.**

### 3. mock-exam — Claude caught a real High Gemini MISSED
Gemini ran only the baseline commands (lint/test/build/`npm audit` — all pass) and concluded **"no significant findings."** It did **not** do the data-flow analysis that surfaces Claude's High: the client-trusted `p_score` → leaderboard-integrity issue. **A Gemini miss** — and a flag that "baseline commands pass ⇒ no findings" is a shallow-audit failure mode.

## What this means for the framework

1. **Severity calibration genuinely diverges** (kymondongiap). This is direct evidence toward the queued DEPTH-semantics / severity-floor candidate — but it's ONE run, so it is logged as evidence, not auto-promoted (Rule of Three).
2. **Coverage diverges BOTH ways** — Gemini caught what Claude missed (3d Three.js), Claude caught what Gemini missed (mock-exam score integrity). No single model is complete; a deeper pass or a second model materially changes recall. Strongest argument yet for cross-model (or two-pass) audits on high-stakes targets.
3. **New, sharp, actionable candidate** (FAILURE_LOG 2026-06-13): when a fix changes the *premise* of a below-floor item, the loop should **re-evaluate that item's severity** — the CORS-becomes-High-after-auth case is a clean miss the protocol currently allows. 1st observation; not promoted.
4. **Cross-model as independent validation** is a bonus: Gemini re-ran kymondongiap's suite (240 passed) and build (exit 0), corroborating Claude's reported metrics.

## Follow-ups for the live app (separate Loop 2, optional)
Gemini's kymondongiap Highs are now genuinely actionable (CORS + credentials is a real concern *with auth enabled*; the guardrail stub; the linters). Worth a focused Loop 2 if Stephen wants the app hardened beyond the audited Loop 1.
