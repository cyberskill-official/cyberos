# `# UNREVIEWED` marker disposition — Gate-1 decision brief (TASK-IMP-139)

- prepared: 2026-07-23, branch `batch/8-audit-hardening`, worktree at `be89966b`, by the TASK-IMP-139 unblocked-half worker
- decision owner: operator (explicit fork reserved by the hardening plan's approval boundary and by spec §1.1 / Implementation-note Gate 1)
- status of this document: EVIDENCE + RECOMMENDATION only. No marker has been touched. Per spec §1.1, no marker-touching commit may exist until a dated operator verdict selecting a branch — with the enumeration below attached — is recorded in this spec's `source_decisions`.

## What is being decided

The 2026-07-14 schema migration auto-set `ai_authorship` and `eu_ai_act_risk_class` on existing specs and attached this own-line comment above each field it guessed:

```
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
```

167 non-draft specs still carry these markers — the tasks left draft (148 of them reached `done`) without the confirmation ever happening. The marker IS the record that no human confirmed the fields; removing it without a verdict manufactures confirmation (spec §Alternatives, "the exact laundering FM-112 exists to prevent"). The operator picks how confirmation happens: one recorded batch acceptance, or a per-file re-audit wave.

## Measured census (2026-07-23, re-derived at Gate-1 time per spec edge case #1)

Method matters here — the linter (FM-112) sees markers only on top-level frontmatter lines; its frontmatter reader consumes block-list items in a sub-loop, so quoted mentions of the marker string inside `source_pages`-style lists and in body prose are NOT markers.

- **FM-112-visible marker set (the debt): 167 files / 333 marker lines**, all non-draft.
  - by status: 148 done · 12 implementing · 4 ready_to_implement · 2 closed · 1 on_hold
  - shape: 166 files carry exactly 2 markers (one per guessed field); 1 file carries 1 (TASK-EVAL-001, `ai_authorship` only).
- **Field values the verdict would accept:**
  - `ai_authorship`: 167/167 = `generated_then_reviewed` — note the irony: the migrated VALUE claims human review on exactly the files where the marker records that none happened.
  - `eu_ai_act_risk_class`: 166 = `not_ai`, **1 = `high`** — `docs/tasks/eval/TASK-EVAL-001-governance-consent-access-retention/spec.md` (status `on_hold`). Its remaining marker guards `ai_authorship`; the `high` line itself is already unmarked. (The only other non-draft `high` file, TASK-CUO-204 `done`, is already marker-free — per-file confirmation has happened organically at least once.)
  - `client_visible`: 167/167 = `false`. No client-facing exposure in the set.
- **Measurement reconciliation (audit 167 vs authoring 170 — resolved):** this measurement reproduces the audit's figures exactly (167 files / 148 done). The authoring-time 170/336/151 counted the marker STRING anywhere in the file and picked up three `done` specs that merely quote it in body prose (TASK-IMP-084, TASK-IMP-108, TASK-IMP-117). Nothing accrued between audit and authoring; the delta is methodology. Five files total quote the string without carrying markers (those three plus TASK-IMP-139/140 themselves).
  - **Concrete hazard this removes:** a bulk clear driven by naive `grep -rl '# UNREVIEWED'` would edit those five files and corrupt documentation that intentionally quotes the marker. The enumeration attached to the verdict — and the eventual `test_corpus_hygiene.sh` census (AC 2) — must use the FM-112-equivalent scan, not a substring grep.
- **Drafts:** 331 draft specs also carry markers; they stay (honest, per spec §1.2). Note for the implementer of either branch: `task-lint` FM-112 currently fires on drafts too (993 corpus-wide findings include them), while spec §1.2 says drafts keep their markers legitimately — FM-112's status-awareness needs reconciling in the implementation regardless of branch, or the corpus lint stays red on drafts after a successful sweep.

## Option A — Branch clear (one recorded bulk verdict)

**What happens.** The operator records one dated verdict: "I accept the migrated `ai_authorship`/`eu_ai_act_risk_class` values on the enumerated 167 files as-is." The markers are then removed mechanically (FM-112-equivalent scan, top-level frontmatter lines only); the verdict text ships in the commit body and CHANGELOG; `test_corpus_hygiene.sh` pins the zero-marker end state.

- **Effort:** ~1–2 h total. The enumeration tooling already exists (this brief's census); removal is a 30-line mechanical script; review of a 167-file, marker-lines-only diff is ~30 min.
- **Risk:** a batch acceptance of compliance-adjacent fields without per-file eyes. Bounded by the measured values: everything in the set is internal (`client_visible: false`), 166/167 are `not_ai`, and the uniform `generated_then_reviewed` matches how this corpus is in fact produced. The residual risk concentrates in ONE file (TASK-EVAL-001, `high`). Honest about being a batch: per-file confirmation is left undone forever, by recorded choice.
- **Precedent:** TASK-IMP-117's FM-001 sweep (497 files) is the mechanical-commit pattern — but it normalized FORMAT; this accepts VALUES. The value-acceptance precedent is thinner, which is exactly why the plan reserved the choice to the operator.

## Option B — Branch re-audit (per-file confirmation wave)

**What happens.** A wave re-runs task-audit's compliance families over the 167 files; a human confirms or corrects `ai_authorship` and `eu_ai_act_risk_class` per file; each file's markers drop only with its confirmation recorded in its audit record; markers survive on anything not yet confirmed.

- **Effort:** 167 files × 2 fields. With a prepared worksheet (file, title, type, current values) a disciplined pass runs 2–3 min/file → **6–9 focused operator-hours**, plus ~2–4 h of wave tooling/orchestration, plus the diff spread across 167 audit records. Calendar risk: this is the kind of wave that stalls; the corpus stays red meanwhile.
- **Risk:** confirmation fatigue. 166 of the 167 answers are foreseeably "yes, not_ai, generated_then_reviewed" — a wave of rubber-stamps manufactures per-file confirmation with less honesty than an explicit batch verdict, at ~10× the cost. The real value concentrates where judgment is needed, and the census says that is one file.
- **Value:** genuine per-file confirmation; catches a wrong risk class if one hides among the 166 (the census makes that unlikely but not impossible); produces the audit-record trail §1.2 wants per-file.
- **Precedent:** TASK-CUO-204 — a `done`, `high`-risk spec whose markers were individually cleared. Per-file confirmation is real practice here, applied where the stakes warranted it.

## Recommendation

**Branch clear for the enumerated set minus one file, with TASK-EVAL-001 individually confirmed** — i.e. adopt Option A's mechanics with a single carve-out, which the branch's own terms permit (the verdict covers exactly the attached enumeration; attach 166 files and record TASK-EVAL-001's confirmation separately, following the TASK-CUO-204 precedent). This is a minor hybrid and is named as such — the operator may instead choose pure A (fold EVAL-001 into the batch; it is `on_hold` and its `high` line is already unmarked) or pure B (if per-file confirmation trails on shipped work carry compliance weight beyond what one batch verdict provides).

Rationale in three facts: (1) the set is uniformly internal and 166/167 `not_ai`, so a re-audit wave would spend 6–9 operator-hours converting foreseeable yeses into rubber-stamps — manufactured confirmation, the exact failure §Alternatives warns about; (2) the one file where judgment is genuinely needed is identifiable NOW, so the honest per-file effort is one file, not 167; (3) the batch verdict is honest about what it is, ships in the commit body + CHANGELOG, and FM-112 keeps guarding the future — new markers cannot survive draft silently again.

**Proposed verdict text (for `source_decisions`, to adapt or reject):**

> 2026-07-XX operator (Gate 1, Branch clear + carve-out): I accept the migrated `ai_authorship: generated_then_reviewed` and `eu_ai_act_risk_class` values as-is on the 166 files enumerated in `assets/unreviewed-fork-brief.md` Appendix (FM-112-visible census of 2026-07-23), and separately confirm TASK-EVAL-001's `ai_authorship` after individual review. Markers on these files are cleared mechanically by the FM-112-equivalent scan; draft-status markers are retained.

## After either branch

Zero FM-112-visible markers on non-draft specs; confirmation trail per the chosen branch; FM-112 guards new drift (its draft-status semantics reconciled per the note above); `scripts/tests/test_corpus_hygiene.sh` (spec §1.6, built by the gated half) pins the end state with the FM-112-equivalent census.

## Appendix — the enumerated set (167 files, FM-112-visible census, 2026-07-23)

Columns: status · marker lines · field(s) guarded · path.

- closed · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/docs/TASK-DOCS-001-server-render-reference-pages/spec.md
- closed · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/improvement/TASK-IMP-028-ace-style-skill-curation-loop/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/ai/TASK-AI-001-cost-ledger-precheck/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/ai/TASK-AI-002-cost-ledger-postcall-reconcile/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/ai/TASK-AI-003-memory-audit-bridge/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/ai/TASK-AI-004-cost-hold-expiry-cleanup/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/ai/TASK-AI-005-tenant-policy-yaml-loader/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/ai/TASK-AI-006-model-alias-resolution/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/ai/TASK-AI-007-provider-cost-table-loader/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/ai/TASK-AI-008-multi-provider-router/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/ai/TASK-AI-009-circuit-breaker/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/ai/TASK-AI-010-streaming-sse/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/ai/TASK-AI-011-presidio-pii-redaction/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/ai/TASK-AI-012-vn-pii-plugin/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/ai/TASK-AI-013-vn-pii-recall-gate/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/ai/TASK-AI-014-persona-version-stamping/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/ai/TASK-AI-015-zdr-enforcement/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/ai/TASK-AI-016-residency-pinning/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/ai/TASK-AI-017-per-tenant-cache/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/ai/TASK-AI-018-cache-cross-tenant-leak-test/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/ai/TASK-AI-019-bge-m3-embeddings/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/ai/TASK-AI-020-bge-rerank/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/ai/TASK-AI-021-operator-cli/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/ai/TASK-AI-022-otel-trace-emission/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/app/TASK-APP-003-mac-app-store-distribution/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/app/TASK-APP-004-microsoft-store-distribution/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/app/TASK-APP-005-linux-store-distribution/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/app/TASK-APP-006-package-manager-distribution/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/auth/TASK-AUTH-001-tenant-create/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/auth/TASK-AUTH-002-subject-create/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/auth/TASK-AUTH-003-rls-enforcement/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/auth/TASK-AUTH-004-jwt-jwks/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/auth/TASK-AUTH-005-admin-rest/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/auth/TASK-AUTH-006-bootstrap-cli/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/auth/TASK-AUTH-101-rbac-catalogue/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/auth/TASK-AUTH-102-totp-webauthn-mfa/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/auth/TASK-AUTH-103-saml-sso/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/auth/TASK-AUTH-104-oidc-sso/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/auth/TASK-AUTH-105-passkey-enrolment-login/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/auth/TASK-AUTH-106-impossible-travel/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/auth/TASK-AUTH-107-hibp-breach-check/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/auth/TASK-AUTH-108-lumi-tenant-identity-jwt/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/auth/TASK-AUTH-109-stub-to-full-migration/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/auth/TASK-AUTH-110-oidc-provider/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/auth/TASK-AUTH-111-sso-display-name-from-id-token/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/chat/TASK-CHAT-101-native-chat-skeleton/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/chat/TASK-CHAT-267-in-app-content-reporting/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/chat/TASK-CHAT-268-user-blocking/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/chat/TASK-CHAT-269-moderation-queue/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/cuo/TASK-CUO-101-langgraph-supervisor/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/cuo/TASK-CUO-102-langgraph-postgres-checkpointer/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/cuo/TASK-CUO-103-trace-replay-rows/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/cuo/TASK-CUO-104-topological-chain-walk/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/cuo/TASK-CUO-105-per-step-rollback/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/cuo/TASK-CUO-106-supervisor-phase4-special-handlers/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/cuo/TASK-CUO-205-single-backlog-write-path/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/cuo/TASK-CUO-206-ship-run-state-manifest/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/cuo/TASK-CUO-207-gate-autodetect-portability/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/cuo/TASK-CUO-208-fr-template-profile/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/cuo/TASK-CUO-209-full-sdp-vendoring/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/docs/TASK-DOCS-002-docs-single-source-of-truth/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/docs/TASK-DOCS-003-release-roadmap-visualization/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/docs/TASK-DOCS-004-folder-per-fr-layout/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/docs/TASK-DOCS-005-fr-html-pages/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/docs/TASK-DOCS-006-status-hub/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/docs/TASK-DOCS-007-status-hub-v2/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/email/TASK-EMAIL-001-stalwart-deployment/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/email/TASK-EMAIL-004-dkim-arc-bimi/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/email/TASK-EMAIL-005-camel-dual-llm/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/email/TASK-EMAIL-009-outbound-1to1-send/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/email/TASK-EMAIL-011-dsar-message-export/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/improvement/TASK-IMP-068-payload-version-drift-gate/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/improvement/TASK-IMP-069-publish-payload-on-release/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/improvement/TASK-IMP-070-remote-update-awareness/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/improvement/TASK-IMP-071-durable-release-trigger/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/improvement/TASK-IMP-072-repo-wide-version-consistency/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/improvement/TASK-IMP-073-fix-capacitor-mobile-app-icon/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/improvement/TASK-IMP-074-ship-workflow-hardening/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/improvement/TASK-IMP-075-mas-updater-exclusion/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/improvement/TASK-IMP-076-root-cli-and-mcp-connector/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/improvement/TASK-IMP-077-ios-icon-alpha-flatten/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/improvement/TASK-IMP-078-store-build-number-monotonic/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/improvement/TASK-IMP-079-docs-ship-race/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/improvement/TASK-IMP-080-served-bundle-version-drift/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/improvement/TASK-IMP-081-web-console-bundle-ci-rebuild/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/mcp/TASK-MCP-001-spec-compliance/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/mcp/TASK-MCP-002-server-heartbeat-lifecycle/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/mcp/TASK-MCP-004-oauth-pkce/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/memory/TASK-MEMORY-101-layer2-ingest-pipeline/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/memory/TASK-MEMORY-102-layer2-rebuild-ci-gate/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/memory/TASK-MEMORY-103-multi-device-sync/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/memory/TASK-MEMORY-104-tauri-app/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/memory/TASK-MEMORY-105-doctor-watched-folders-invariants/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/memory/TASK-MEMORY-106-sync-class-enforcement/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/memory/TASK-MEMORY-107-fs-watcher/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/memory/TASK-MEMORY-108-search-api/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/memory/TASK-MEMORY-109-claude-code-hook-capture/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/memory/TASK-MEMORY-110-capture-daemon-health-restart/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/memory/TASK-MEMORY-111-pre-ingest-pii-detection/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/memory/TASK-MEMORY-112-episodic-memory/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/memory/TASK-MEMORY-113-recency-decay-recall/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/memory/TASK-MEMORY-114-write-time-importance/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/memory/TASK-MEMORY-115-cyberos-dream/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/memory/TASK-MEMORY-116-semantic-dedup-consolidate/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/memory/TASK-MEMORY-117-per-store-acl/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/memory/TASK-MEMORY-118-put-if-precondition/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/memory/TASK-MEMORY-119-session-transcript-ledger/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/memory/TASK-MEMORY-120-cyberos-history/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/obs/TASK-OBS-002-tenant-aware-grafana/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/obs/TASK-OBS-006-tail-sampling/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/proj/TASK-PROJ-001-issue-schema/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/proj/TASK-PROJ-002-memory-decision-anchoring/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/proj/TASK-PROJ-003-yjs-crdt-collaboration/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/proj/TASK-PROJ-004-issue-lifecycle-fsm/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/proj/TASK-PROJ-005-rate-card-schema/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/proj/TASK-PROJ-006-billable-cascade/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/proj/TASK-PROJ-007-billing-modes/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/proj/TASK-PROJ-008-memory-audit-row-per-mutation/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/proj/TASK-PROJ-009-memory-link-schema/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/proj/TASK-PROJ-010-citation-drift-detector/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/proj/TASK-PROJ-011-blocker-detector/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/proj/TASK-PROJ-012-cycle-review-draft/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/proj/TASK-PROJ-013-estimate-calibration/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/proj/TASK-PROJ-014-kanban-board/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/proj/TASK-PROJ-015-timeline-view/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/proj/TASK-PROJ-016-gantt-view/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/proj/TASK-PROJ-017-brief-modal/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/proj/TASK-PROJ-018-design-tokens-a11y-ci/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/skill/TASK-SKILL-101-memory-integration/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/skill/TASK-SKILL-102-oci-registry/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/skill/TASK-SKILL-103-frontmatter-extension/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/skill/TASK-SKILL-104-capability-broker/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/skill/TASK-SKILL-105-memory-capture-bundle/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/skill/TASK-SKILL-106-memory-sync-bundle/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/skill/TASK-SKILL-107-synthesis-author/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/skill/TASK-SKILL-108-vn-mst-validate/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/skill/TASK-SKILL-109-vn-bank-transfer/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/skill/TASK-SKILL-110-vn-vat-invoice/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/skill/TASK-SKILL-111-trigger-description-enrichment/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/skill/TASK-SKILL-112-trigger-tests-fixtures/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/skill/TASK-SKILL-113-frontmatter-xml-free/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/skill/TASK-SKILL-114-baseline-artefact/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/skill/TASK-SKILL-115-stale-placeholder-sweep/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/skill/TASK-SKILL-116-vendor-debugging-cycle-chain-coverage/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/skill/TASK-SKILL-117-architectural-spike-pair/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/skill/TASK-SKILL-118-thin-pair-contract-parity/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/skill/TASK-SKILL-119-stale-reference-sweep/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/skill/TASK-SKILL-120-deliverable-scaffolding/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/skill/TASK-SKILL-201-oci-registry-deploy/spec.md
- done · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/templates/TASK-TPL-001-templates-module-cds/spec.md
- implementing · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/app/TASK-APP-001-desktop-cyberos-operations/spec.md
- implementing · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/mcp/TASK-MCP-003-sep986-naming-validator/spec.md
- implementing · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/mcp/TASK-MCP-005-protected-resource-metadata/spec.md
- implementing · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/mcp/TASK-MCP-006-tool-annotation-gating/spec.md
- implementing · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/mcp/TASK-MCP-007-tasks-primitive/spec.md
- implementing · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/mcp/TASK-MCP-008-elicitation/spec.md
- implementing · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/obs/TASK-OBS-001-otel-collector/spec.md
- implementing · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/obs/TASK-OBS-003-red-metrics/spec.md
- implementing · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/obs/TASK-OBS-005-tracecontext-correlation/spec.md
- implementing · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/obs/TASK-OBS-007-alertmanager-cuo-runbook-routing/spec.md
- implementing · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/obs/TASK-OBS-008-compliance-view-scoping/spec.md
- implementing · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/obs/TASK-OBS-009-chain-of-custody-manifest/spec.md
- on_hold · 1 · ai_authorship · docs/tasks/eval/TASK-EVAL-001-governance-consent-access-retention/spec.md
- ready_to_implement · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/inv/TASK-INV-004-wise-webhook/spec.md
- ready_to_implement · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/obs/TASK-OBS-004-langsmith-ai-traces/spec.md
- ready_to_implement · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/ten/TASK-TEN-002-plan-tiers/spec.md
- ready_to_implement · 2 · ai_authorship,eu_ai_act_risk_class · docs/tasks/ten/TASK-TEN-004-four-axis-metering/spec.md

(total: 167 files / 333 marker lines)
