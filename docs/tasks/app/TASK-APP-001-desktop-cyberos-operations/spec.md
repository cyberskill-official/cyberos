---
id: TASK-APP-001
title: Desktop CyberOS operations - build payload, install/update projects from the UI
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-07-14T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: app
status: implementing
priority: p0
depends_on: []
routed_back_count: 0
awh: N/A
---

# TASK-APP-001 - Desktop CyberOS operations UI

## Context

Employees should trigger CyberOS operations from the desktop app instead of the terminal: build the distributable payload, pick a project folder, check its installed vs available version, and install/update it. The desktop app is the Tauri + Svelte shell at `services/memory/desktop`; operations wrap the existing tooling (`tools/install/build.sh`, `install.sh`, `install.sh --check`) so the UI and the CLI can never behave differently.

## 1. Normative clauses

1. The app MUST expose an Operations view with: (a) "Build payload" - runs `tools/install/build.sh` in the configured CyberOS checkout; (b) a project picker - a scanned list of git repositories under `~/Projects` (showing each repo's installed CyberOS version) plus manual path entry (a native folder dialog is an optional follow-up); (c) "Check" - runs `install.sh --check <project>` and shows installed vs available version and whether an update exists; (d) "Init / Update" - runs `install.sh <project>` and shows the summary.
2. All operations MUST shell out to the canonical scripts (no reimplementation of init logic in the app). The CyberOS checkout path MUST be configurable in the UI and persisted; it defaults to `~/Projects/CyberSkill/cyberos`.
3. Command output (stdout+stderr) MUST be shown in the UI, and a non-zero exit MUST surface as a visible failure state - never silently swallowed.
4. The Tauri commands MUST validate that the configured checkout contains `tools/install/build.sh` (and the payload `dist/cyberos/install.sh` for check/init) before running, returning a structured error otherwise.
5. Init MUST NOT run against the CyberOS checkout itself (guard: target != checkout root).
6. No operation pushes, commits, or deletes anything in the target project; the commands only run the existing scripts, which are non-destructive and idempotent by design.

## 2. Acceptance criteria

- [ ] From the app: build payload succeeds and reports the payload path + version.
- [ ] Pick a project folder, Check shows `installed=X available=Y` (or "not initialised"), Init lays down `.cyberos/` and reports the summary; re-running Init is a clean update (store untouched).
- [ ] Failure paths render the error output (e.g. missing checkout, script failure).
- [ ] `cargo check` green on the tauri crate; frontend builds.

## 3. Gate

Machine: `cargo check` (desktop src-tauri) + frontend build + manual smoke on the operator's Mac. Review + final acceptance: HITL per STATUS-REFERENCE §1.4.
