---
id: TASK-APP-001
title: "Desktop CyberOS Ops UI — build/check/install from Tauri"
eu_ai_act_risk_class: not_ai
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-07-14T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: app
status: done
priority: p0
routed_back_count: 0
awh: N/A
depends_on: []
blocks: []
related_tasks: [TASK-MEMORY-104, TASK-IMP-142]
owner: Stephen Cheng (CTO)
created: 2026-07-14
shipped: null

source_decisions:
  - "2026-07-23 operator (IMP-139 Gate 2): resume unchanged — desktop ops actively shipping; dossier recommended resume over mechanical route_back. Evidence: assets/reconcile/TASK-APP-001.md."
  - "2026-07-24 batch/9c-app adopt: Check uses dist/cyberos/version.sh (IMP-070), not install.sh --check (never existed)."

language: rust + svelte
service: cyberos/services/memory/desktop/
new_files:
  - services/memory/desktop/src-tauri/src/ops.rs
  - tools/install/docs/guides/desktop-ops.md
modified_files:
  - services/memory/desktop/src-tauri/src/main.rs
  - services/memory/desktop/src/App.svelte

allowed_tools:
  - file_read: services/memory/desktop/**
  - file_write: services/memory/desktop/**
  - bash: cd services/memory/desktop/src-tauri && cargo check
  - bash: cd services/memory/desktop/src-tauri && cargo test

disallowed_tools:
  - reimplement install/init logic inside the Tauri layer
  - push / commit / delete from Ops commands
  - claim apps/desktop (console webview) as the APP-001 ops surface

effort_hours: 4
subtasks:
  - "1.0h: ops.rs Tauri commands (build/check/install/list/settings/guards)"
  - "1.0h: App.svelte CyberOS Ops tab"
  - "0.5h: desktop-ops.md operator guide"
  - "0.5h: residual guard unit tests"
  - "1.0h: batch/9c-app re-spec + audit"
risk_if_skipped: "Operators keep running install from the terminal only; checkout/self-init mistakes stay unguarded in UI; process drift leaves APP-001 stuck implementing."
---

# TASK-APP-001: Desktop CyberOS operations UI

## Summary

Expose CyberOS install operations from the Tauri + Svelte desktop shell at `services/memory/desktop/`: Build payload (`tools/install/build.sh`), list projects under `~/Projects`, Check via `dist/cyberos/version.sh`, and Install/Update via `dist/cyberos/install.sh`. As-built surface is `ops.rs` + the "CyberOS Ops" tab in `App.svelte`, documented in `tools/install/docs/guides/desktop-ops.md`.

## Problem

IMP-139 left this task at `implementing` (operator resume). The UI and Rust commands already shipped, but the engineering-spec still claimed `install.sh --check` (does not exist), lacked task@1 adopt grammar / authorship disclosure, and had no audit. Reconcile dossier velocity evidence also conflated `apps/desktop` (console webview) with the ops shell.

## Proposed Solution

Adopt the as-built ops path:

- `ops_build` → `bash tools/install/build.sh`
- `ops_check` → `bash dist/cyberos/version.sh <project>` with `CYBEROS_NONINTERACTIVE=1` (IMP-070 key=value lines)
- `ops_install` → `bash dist/cyberos/install.sh <project>`
- Settings at `~/.cyberos/desktop-ops.json`; default checkout `~/Projects/CyberSkill/cyberos`
- Guards: checkout must contain `tools/install/build.sh`; payload scripts must exist; target must be a git repo; target ≠ checkout root
- Residual unit tests for the checkout / project guards

## Alternatives Considered

- **Route back and rewrite.** Rejected: operator Gate-2 said resume; code is live.
- **Add `install.sh --check` alias.** Rejected: duplicates `version.sh`; IMP-070 already owns the machine-parseable contract.
- **Treat `apps/desktop` as this task.** Rejected: that crate is the console webview wrapper; ops live under `services/memory/desktop/`.

## Success Metrics

- Primary: Build / Check / Install shell the canonical scripts; guards fail closed; Ops tab surfaces stdout+stderr and non-zero exits.
- Guardrail: `cargo check` + `cargo test` green on `services/memory/desktop/src-tauri`.

## Scope

In scope (as-built):

- `services/memory/desktop/src-tauri/src/ops.rs` and command registration in `main.rs`
- Ops tab in `services/memory/desktop/src/App.svelte`
- `tools/install/docs/guides/desktop-ops.md`
- Guard unit tests in `ops.rs`

### Out of scope / Non-Goals

- `apps/desktop/` console webview releases (separate APP surface)
- Native folder picker dialog (optional follow-up)
- Reimplementing install/init inside Rust
- Live Mac GUI smoke in CI (operator Mac; machine gate is cargo check/test)

## Dependencies

`depends_on: []`. Soft: TASK-MEMORY-104 (desktop shell scaffold); TASK-IMP-142 (resume schedule).

## 1. Description (normative)

- 1.1 The Ops UI MUST expose Build payload, project list + manual path, Check, and Install/Update.
- 1.2 Operations MUST shell out to `tools/install/build.sh`, `dist/cyberos/version.sh`, and `dist/cyberos/install.sh` — no reimplementation of init logic in the app.
- 1.3 Checkout path MUST be configurable, persisted, and default to `~/Projects/CyberSkill/cyberos`.
- 1.4 Command stdout+stderr MUST be returned to the UI; non-zero exit MUST set `ok: false`.
- 1.5 Before running, the app MUST validate checkout (`tools/install/build.sh` present) and, for Check/Install, the required payload script under `dist/cyberos/`.
- 1.6 Install MUST refuse the CyberOS checkout itself and refuse non-git targets.
- 1.7 Ops MUST NOT push, commit, or delete anything beyond what the canonical scripts already do.
- 1.8 This adopt MUST NOT claim `apps/desktop` or `install.sh --check` as the shipped Check path.

## Acceptance criteria

- [ ] AC 1 (traces_to: #1.1,#1.2) - ops_build shells build.sh - verify: `services/memory/desktop/src-tauri/src/ops.rs::ops_build`
- [ ] AC 2 (traces_to: #1.2) - ops_check shells version.sh with CYBEROS_NONINTERACTIVE - verify: `services/memory/desktop/src-tauri/src/ops.rs::ops_check`
- [ ] AC 3 (traces_to: #1.2) - ops_install shells install.sh - verify: `services/memory/desktop/src-tauri/src/ops.rs::ops_install`
- [ ] AC 4 (traces_to: #1.3) - settings default + persist path - verify: `services/memory/desktop/src-tauri/src/ops.rs::ops_get_settings`
- [ ] AC 5 (traces_to: #1.4) - OpResult carries ok + combined stdout/stderr - verify: `services/memory/desktop/src-tauri/src/ops.rs::OpResult`
- [ ] AC 6 (traces_to: #1.5) - missing build.sh rejected - test: `services/memory/desktop/src-tauri/src/ops.rs::require_checkout_rejects_non_checkout`
- [ ] AC 7 (traces_to: #1.6) - checkout-as-project refused - test: `services/memory/desktop/src-tauri/src/ops.rs::require_project_rejects_checkout_root`
- [ ] AC 8 (traces_to: #1.6) - non-git path refused - test: `services/memory/desktop/src-tauri/src/ops.rs::require_project_rejects_non_git`
- [ ] AC 9 (traces_to: #1.1) - Ops tab invokes ops_* commands - verify: `services/memory/desktop/src/App.svelte`
- [ ] AC 10 (traces_to: #1.7,#1.8) - Out of scope lists apps/desktop, install.sh --check, and non-destructive Ops - verify: this spec Scope / Out of scope / disallowed_tools

## Verification

```bash
cd services/memory/desktop/src-tauri && cargo check
cd services/memory/desktop/src-tauri && cargo test
```

| Path | Covers |
|------|--------|
| `ops.rs` commands | Build / Check / Install / list / settings |
| `ops.rs` unit tests | Checkout + project guards |
| `App.svelte` | Ops tab wiring |
| `tools/install/docs/guides/desktop-ops.md` | Operator contract |

## AI Authorship Disclosure

- **Tools used:** Cursor agent (Composer) on branch `batch/9c-app`.
- **Scope:** Re-spec/adopt against as-built `services/memory/desktop` ops; residual guard tests; deferred native folder dialog ledgered.
- **Human review:** Session HITL override 2026-07-24 (IMP-139 resume path).

---

*batch/9c-app adopt — TASK-APP-001 re-spec against as-built memory desktop Ops.*
