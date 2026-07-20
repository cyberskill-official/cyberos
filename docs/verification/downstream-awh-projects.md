# Downstream AWH-lineage project verification (2026-06-19)

## Why this exists

After auto-work-harness (awh) was vendored into CyberOS as the verification gate and the standalone was retired, we checked the eight projects that shared awh lineage to confirm the absorption did not break them, and to make each one healthy both locally and on the live web before moving to the deployment stage.

Headline finding: the CyberOS absorption did not break any downstream project. The absorption is contained inside CyberOS. Only three of the eight repos use awh at all, and only as a local test gate; all three carry a committed baseline, so the fail-closed hardening cannot error on them. The one genuinely broken live site, the CCAF mock exam, is broken for reasons that predate and are unrelated to the absorption (a deploy misconfiguration plus two client bugs). A fix is prepared and type-checks; it needs an owner build and a Vercel preview deploy to go live.

## How this was verified, and the limits

This run can read every file, run a read-only TypeScript type-check, and fetch live URLs. It cannot build (cargo/next build), deploy, or write git on the machine. So local build, test, deploy, and commit are owner-run, with exact commands given per project below.

## Summary

| Project | Type | awh-wired | Live | Verdict |
| --- | --- | --- | --- | --- |
| Personal/claude-certified-architect-mock-exam | Next.js app | no | broken (stale build served) | fix prepared, type-checks; needs build + preview deploy |
| Personal/3d-preriodic-table | Vite SPA | no | serves (200) | likely fine; needs functional check + title fix |
| CyberSkill/kymondongiap | Python backend + frontend | no | frontend serves (200) | likely fine; needs functional + backend check |
| Personal/gam | Vite app | yes | no deploy config found | verify locally; confirm deploy target |
| CyberSkill/cyber-click | core-logic package | yes | n/a (library) | verify locally (npm run verify) |
| CyberSkill/shopass | core-logic package | yes | n/a (library) | verify locally (npm run verify) |
| CyberSkill/design-system-audit-framework | tool | no | n/a | verify locally (npm run verify) |
| CyberSkill/design-system | component library | no | n/a | verify locally (npm run verify:all) |

## Absorption impact, in detail

Only gam, cyber-click, and shopass contain a `.awh/` directory. The other five never used awh, so the absorption and the gate-bypass hardening cannot affect them.

For the three awh-wired repos, each has a committed `.awh/eval-baseline.json` (gam: 3 goldenset tasks, cyber-click: 2, shopass: 2). The gate-bypass fix only changes behavior by failing closed when a current goldenset task is absent from the baseline. With baselines present, their gates stay green unless a task id is missing, which their own `npm run verify` will surface. The only optional follow-up is pulling the gate-bypass fix into each repo's awh copy if it predates the fix; that is an improvement, not a breakage. See `tools/awh/RETIREMENT.md` step A.

## Project details and local verification

### claude-certified-architect-mock-exam (the reported bug)

Reported symptom: after finishing the test, users cannot see their points or result.

Root causes found, in order of impact:

1. Production serves a stale build, not the current app. `vercel.json` carried Vite-era settings (`outputDirectory: "dist"`, a catch-all rewrite of every path to `/index.html`, and Vite `/assets/*.ts` headers). The repo migrated to Next.js, but Vercel was still pointed at the tracked, stale `dist/` Vite SPA. So the live site at `ccaf.cyberskill.dev` is an old version of the app, and the Next.js `/api/exam/submit` route never runs in production, which also explains empty leaderboard and global stats.
2. The Submit button navigated unconditionally. In `src/app/exam/page.tsx` the handler called `engine.finishExam(false)` without awaiting it and then `router.push('/result')` regardless. When `finishExam` bailed (unanswered questions, or the user declined the confirm dialog), the app still navigated to `/result` with `store.finished` false, and the result page then redirected the user back to the home page.
3. A hydration race on a cold load of `/result`. The result page redirected home whenever the store looked empty, which is exactly its state during async zustand persist rehydration on a refresh, a shared link, or a restored tab.

Fixes applied to the working tree (type-checked with `tsc --noEmit`, exit 0):

- `vercel.json`: reduced to security headers only. Vercel now auto-detects Next.js, builds `.next`, and serves SSR plus API routes. No `outputDirectory`, no SPA rewrite.
- `src/hooks/useExamEngine.ts`: `finishExam` now returns a boolean (true only when the exam actually finished) and submits to Supabase in the background instead of blocking navigation on the network.
- `src/app/exam/page.tsx`: the Submit button awaits `finishExam` and navigates only when it returns true; the timer and anti-cheat paths navigate via `.then(...)`.
- `src/app/result/page.tsx`: redirect decision now waits for zustand persist hydration (`onFinishHydration` / `hasHydrated`), so a finished session is restored before any redirect.

Still owner-side, cannot be done from here:

- The authoritative scoring lives in an un-versioned Supabase RPC (`submit_exam_result`), with `get_global_stats` and `get_user_history` for the dashboard and leaderboard. `supabase/migrations` is empty. Version these RPCs into a migration and confirm the `submit_exam_result` parameters match the client payload: `p_email`, `p_pin_hash`, `p_score`, `p_wrong_answers`, `p_time_taken`, `p_nickname`.
- The stale, tracked `dist/` directory should be removed (`git rm -r dist`) and added to `.gitignore` so it is never served again.

Owner runbook (do not promote to production until the preview passes):

```
cd ~/Projects/Personal/claude-certified-architect-mock-exam
npm ci
npm run build                 # next build also type-checks; must pass
npx vercel deploy             # PREVIEW deploy (not --prod)
# open the preview URL: take a short timed exam, submit, confirm the result page shows the score,
# then refresh /result and confirm it still shows (no bounce to home)
# only if the preview is correct:
git checkout -b fix/ccaf-deploy-and-results
git rm -r dist && echo "/dist" >> .gitignore
git add -A && git commit -m "fix(ccaf): serve the Next app on Vercel; stop losing results on submit and refresh"
git push -u origin fix/ccaf-deploy-and-results
npx vercel deploy --prod      # promote after review
```

Post-deploy 404 (seen 2026-06-19): the build and deploy succeeded but every route returned 404, including the raw deployment URL. Cause: the Vercel PROJECT settings still carried the old Vite preset (Framework "Other", Output Directory "dist"). With `dist` removed and `outputDirectory` no longer in `vercel.json`, Vercel served an empty `dist` and 404'd everywhere. Fix, either works: (a) `vercel.json` now sets `"framework": "nextjs"`, which forces the Next.js builder to serve `.next` - redeploy; or (b) in the Vercel dashboard, Project > Settings > Build and Deployment, set Framework Preset to Next.js and clear the Output Directory override, then redeploy. Verify the raw deployment URL, not just the alias, returns the app.

Rollback: if `npm run build` or the preview fails, revert `vercel.json` and the three source files; production keeps serving the current (stale) build, so there is no outage risk from holding back.

### 3d-preriodic-table

Vite SPA, live at `https://3d-preriodic-table.vercel.app/` (HTTP 200). Its `vercel.json` is correct for a Vite SPA (a PubChem API proxy rewrite plus the SPA fallback to `/index.html`), so no deploy change is needed. Two minor items: the page title is the scaffold default "temp-app" (set a real title and metadata), and the working tree has four uncommitted changes to review. No awh wiring, so the absorption does not touch it.

Local verification:

```
cd ~/Projects/Personal/3d-preriodic-table
npm ci && npm run build && npm test
git status   # review the 4 uncommitted changes
```

### kymondongiap

Full-stack: a Python backend (`backend/`, `api/`, docker-compose) and a separate frontend, live at `https://kymondongiap-gamma.vercel.app/` (HTTP 200, frontend shell renders). `vercel.json` rewrites to `/frontend`. No awh wiring. The working tree has eleven uncommitted changes, including a note that mypy was wired as a blocking gate. The frontend serves; the backend and the end-to-end flow need a functional check on your machine.

Local verification:

```
cd ~/Projects/CyberSkill/kymondongiap
# backend
python -m mypy . && (pytest -q || true)
# frontend
cd frontend && npm ci && npm run build
git status   # review the 11 uncommitted changes before committing
```

### gam

Vite app, react 19, consumes `@cyberskill/shared` (bumped to 3.21.0 in the awh-adopt commit). It has a `.awh/` gate (3 goldenset tasks, baseline present). No Vercel config was found, so confirm where or whether it is deployed. Clean working tree.

Local verification:

```
cd ~/Projects/Personal/gam
npm ci && npm run build && npm test
bash .awh/gate.sh           # or the repo's awh gate entry; baseline is present, expect green
```

### cyber-click and shopass

Private core-logic packages (no UI), each gated by awh (baseline present; cyber-click 2 tasks, shopass 2). Not web-deployed. cyber-click has nine uncommitted changes to review; shopass is clean.

Local verification (each repo):

```
npm ci && npm run verify        # runs the test + awh gate; expect eval 100%
git status                      # cyber-click: review the 9 uncommitted changes
```

### design-system-audit-framework (DSAF) and design-system (DS)

DSAF is the audit tool; DS is the component library it audits. Both private, both clean, neither awh-wired. Verified by their own scripts.

Local verification:

```
cd ~/Projects/CyberSkill/design-system && npm ci && npm run verify:all
cd ~/Projects/CyberSkill/design-system-audit-framework && npm ci && npm run verify
# DSAF audits DS, so run DS first, then DSAF
```

## Consolidated owner actions, in order

1. Mock exam: `npm run build`, preview-deploy, validate the result and refresh flows, then commit the prepared fix, remove `dist/`, and promote. Version the Supabase scoring RPCs into a migration.
2. Run `npm run verify` (or `verify:all`) in gam, cyber-click, shopass, DSAF, DS and confirm green. Review the uncommitted changes in cyber-click (9), kymondongiap (11), and 3d (4).
3. 3d: set a real page title and metadata; functional check on the live SPA.
4. kymondongiap: backend plus end-to-end functional check.
5. Optional: pull the awh gate-bypass fix into gam, cyber-click, shopass if their awh copy predates it (`tools/awh/RETIREMENT.md` step A).

Stage 2 (deploying the CyberOS core modules) should start only after the eight projects are confirmed green locally and live.
