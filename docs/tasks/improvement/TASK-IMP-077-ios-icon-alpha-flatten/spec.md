---
id: TASK-IMP-077
title: "iOS icon alpha flatten — ASC 90717 hotfix: 1024x1024 marketing icon must carry no alpha channel"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: improvement
created_at: 2026-07-13T00:00:00+07:00
department: engineering
author: @stephencheng
template: task@1
module: improvement
priority: p0
status: done
verify: T
phase: "Wave 6 - go-live (Track B: mobile shells)"
owner: Stephen Cheng (CTO)
created: 2026-07-13
shipped: 2026-07-13
memory_chain_hash: null
related_tasks: [TASK-IMP-073]
depends_on: []
blocks: []
source_pages:
  - "v1.0.0 ios run log 2026-07-12T19:11: altool ERROR 90717 'Invalid large app icon ... can't be transparent or contain an alpha channel' - archive/export succeeded, upload rejected"
  - "apps/web/ios/.../AppIcon-512@2x.png pre-fix: PIL mode RGBA, alpha extrema 254-255 (visually opaque - invisible to any human check; the channel's PRESENCE is what ASC rejects)"
  - "apps/desktop/src-tauri/icons/android/values/ic_launcher_background.xml: #fff (the brand adaptive-icon background - grounded flatten color)"
source_decisions:
  - "2026-07-13 Stephen: 'ios CI failed again' + full run log attached."
language: python3/PIL (one-time flatten), YAML (guard amendment), markdown
service: apps/web
new_files: []
modified_files:
  - apps/web/ios/App/App/Assets.xcassets/AppIcon.appiconset/AppIcon-512@2x.png
  - .github/workflows/release.yml
  - docs/deploy/RELEASE.md
effort_hours: 1
subtasks:
  - "Flatten the repo icon onto #fff (RGBA -> RGB, 1024x1024 preserved) - DONE, PIL-verified"
  - "Amend the ios guard: byte-identity -> present + 1024x1024 + sips hasAlpha:no (the actual submission invariants) - DONE"
  - "RELEASE.md recopy runbook: copy-then-flatten step - DONE"
risk_if_skipped: "Every iOS TestFlight upload fails at ASC validation (90717) after a full successful archive+export - the exact v1.0.0 failure, repeating forever."
---
## §1
1. The repo iOS marketing icon **MUST** be alpha-free RGB 1024x1024 (flattened onto #fff, the brand adaptive background). DONE - PIL: mode RGB, size (1024,1024).
2. TASK-IMP-073's iOS byte-identity guard **MUST** become a derived-asset guard: present + 1024x1024 (sips) + hasAlpha: no - byte-identity is impossible once the copy is legitimately flattened, and the submission invariant (no alpha) is what ASC actually enforces. Android's 15-file hash guard is untouched (adaptive icons legitimately use alpha).
3. RELEASE.md's re-scaffold runbook **MUST** carry the copy-then-flatten step so a future recopy cannot reintroduce the channel.

*Lean profile: one asset transform + one guard amendment, defect and fix both machine-verified in-session; ASC acceptance is proven by the next tag run.*

## §5 (run 2026-07-13)
- PIL pre: RGBA extrema 254-255 → post: RGB (1024,1024). PASS
- Guard YAML parses; sips invocations are macos-runner native. PASS (parse; sips executes on the runner)
- Android guard untouched (15-file hash loop intact). PASS
- Testing pass 2026-07-13 (post gate-1 "approve all"): PIL RGB/1024x1024 re-verified, both guards re-verified, release.yml parses. PASS. Store-side proof already live: the re-tag's iOS lane went green and build 10706 reached TestFlight.
## §9
- Why did TASK-IMP-073's checks miss this? Hash-equality proved copy fidelity, AC #3's visual check cannot see a 254-255 alpha channel - exactly §10 row 3's predicted blind spot. This task converts the blind spot into a standing machine check.
## §10
| Failure | Detection | Recovery |
|---|---|---|
| future recopy reintroduces alpha | ios guard hasAlpha check fails pre-upload | runbook flatten step |
| flatten color wrong vs brand | human review of the store listing preview | re-flatten with corrected color |
| sips output format drifts on future runners | guard fails loud (grep miss = error path) | adjust parse |
*End of TASK-IMP-077.*
