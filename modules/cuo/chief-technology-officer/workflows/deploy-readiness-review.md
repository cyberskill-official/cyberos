---
workflow_id: chief-technology-officer/deploy-readiness-review
workflow_version: 1.0.0
purpose: Go / no-go gate before a production release — author the deployment-readiness checklist + release notes, audit both to 10/10, capture DORA baseline.
persona: cuo/chief-technology-officer
cadence: per-release
status: shipped

inputs:
  - { name: release_candidate_sha, source: CI pipeline,                             format: full git SHA }
  - { name: release_id,            source: release manager,                         format: SemVer or release tag }
  - { name: changelog_range,       source: CHANGELOG diff between prior + RC tag,   format: markdown }
  - { name: target_environment,    source: workflow caller,                         format: "staging | canary | production" }
  - { name: deploy_window,         source: change ticket,                           format: ISO 8601 timestamp range }

outputs:
  - { name: release_notes,    format: release-notes@1,    recipient: customers + status page }
  - { name: deploy_checklist, format: deploy-checklist@1, recipient: cuo/cto + deploy owner + change-approval board }

skill_chain:
  - { step: 1, skill: release-notes-author,    inputs_from: { changelog_range: changelog_range, release_id: release_id, prior_release_id: <prior tag> }, outputs_to: release_notes_draft }
  - { step: 2, skill: release-notes-audit,     inputs_from: release_notes_draft, outputs_to: release_notes }
  - { step: 3, skill: deployment-checklist-author, inputs_from: { release_notes: release_notes, release_candidate_sha: release_candidate_sha, target_environment: target_environment, deploy_window: deploy_window }, outputs_to: deploy_checklist_draft }
  - { step: 4, skill: deployment-checklist-audit,  inputs_from: deploy_checklist_draft, outputs_to: deploy_checklist }

escalates_to:
  - { persona: cuo/chief-legal-officer,    when: "release-notes contains a security advisory (CVE patched) OR breaking change to data-handling — Compliance Notes required (COND-004)" }
  - { persona: cuo/chief-information-security-officer,         when: "deploy-checklist DEP-008 (security scans) reports new high-severity findings" }
  - { persona: cuo/chief-communications-officer, when: "audience: customer_public AND breaking: true — customer-comms drafting required" }

consults:
  - { persona: cuo/chief-financial-officer,          when: "release-notes mention infra-spend material changes — finance heads-up" }
  - { persona: cuo/chief-ai-officer,         when: "release contains an AI-model update — model-card delta required per COND-005 + DEP-018" }

audit_hooks:
  - each skill emits artefact_write rows per its frontmatter audit hook
  - workflow emits a workflow_complete row with the final go/no-go verdict + DORA baseline snapshot (deployment_frequency / lead_time / change_failure_rate / failed_deployment_recovery_time)
  - HITL pauses (typically at step 3 PLAN, step 4 DEP-011 baseline-capture sign-off) halt the chain
---

# Deploy-readiness review — `chief-technology-officer/deploy-readiness-review`

The CTO's deploy gate. Two skill pairs in sequence: release-notes first (customers need to know what's shipping), then deploy-checklist (does this release actually meet the bar?). The checklist explicitly captures the DORA baseline at deploy time — that's how `runbook` retros and `postmortem` audits later compute MTTR / change-failure-rate trends.

## When to invoke

CUO routes here when the user says things like:

- "We're ready to deploy v3.4.1 — go/no-go"
- "Run the release-readiness gate for the November release"
- "Sign off on the production push for <feature>"
- "Deploy checklist for RC-2026-05-17-abc1234"

## How to invoke

```bash
cyberos-cuo run cuo/chief-technology-officer/deploy-readiness-review \
  --input release_candidate_sha=abc1234567890def \
  --input release_id=v3.4.1 \
  --input changelog_range=./releases/v3.4.0-to-v3.4.1.md \
  --input target_environment=production \
  --input deploy_window=2026-05-20T14:00:00+07:00..2026-05-20T16:00:00+07:00 \
  --output-dir ./releases/v3.4.1/
```

## Expected duration

- **Happy path:** 20–40 minutes (release notes draft + audit + checklist draft + audit, no major escalations).
- **With one security advisory** (CVE-related release notes + CISO escalation): +2-4 hours for advisory drafting + CLO review.
- **With breaking change + public audience** (full migration guide + CCO-Communications customer-comms drafting): +4-8 hours.
- **No-go path** (deploy-checklist verdict fails): the workflow halts; operator addresses the failing DEP-NNN items and re-runs.

## Skill chain — step by step

### Step 1: `release-notes-author`
- **What it does:** Authors Keep-a-Changelog + SemVer 2.0.0 format release notes from the CHANGELOG diff between the prior tag and this RC.
- **Inputs:** `changelog_range`, `release_id`, `prior_release_id`.
- **Outputs:** `release_notes_draft`.
- **Pause point:** PLAN approval on audience scope (customer_public / customer_enterprise / internal_only / partner) — drives which conditional sections fire.

### Step 2: `release-notes-audit`
- **What it does:** Validates against `release_notes_rubric@1.0`. Common hits: QA-CVE-001 (fabricated CVE format), QA-BREAK-001 (breaking-change disguised in §Changed), QA-JARGON-001 (engineering jargon in customer-public audience).
- **Inputs:** `release_notes_draft`.
- **Outputs:** `release_notes` at 10/10.
- **Pause point:** HITL on COND-001 (breaking → required Upgrade Notes + Migration Guide) or COND-002 (CVE-patched → required §Security entries).

### Step 3: `deployment-checklist-author`
- **What it does:** Authors the 12-item deploy-readiness checklist per `deployment-checklist-audit/RUBRIC.md` `DEP-001..012`, plus any conditional rows (production / breaking / large migration / regulated / canary / AI-model update).
- **Inputs:** `{release_notes, release_candidate_sha, target_environment, deploy_window}`.
- **Outputs:** `deploy_checklist_draft`.
- **Pause point:** PLAN approval on progressive_delivery choice (canary / blue_green / feature_flag / rolling / direct).

### Step 4: `deployment-checklist-audit`
- **What it does:** Validates against `deploy_checklist_rubric@1.0`. Critical rules: every ✅ has an evidence link (QA-EVIDENCE-001); DEP-011 has DORA baseline captured (QA-DORA-001); rollback plan references a specific runbook section (QA-ROLLBACK-001).
- **Inputs:** `deploy_checklist_draft`.
- **Outputs:** `deploy_checklist` at 10/10 — this is the go/no-go artefact.
- **Pause point:** HITL on QA-RUN-001 if `progressive_delivery: direct` without rationale; HITL on STALE-001 if the release-candidate SHA changed mid-chain.

## Failure modes — per step

| Step | Code | What happens | Recovery |
|---|---|---|---|
| 1 | BOOT-001 | changelog_range input missing | Compute from `git log prior_tag..HEAD` and supply; resume |
| 2 | needs_human (QA-BREAK-001) | Breaking change snuck into §Changed | Operator moves it to §Upgrade Notes + adds Migration Guide |
| 3 | HITL (PLAN) | progressive_delivery choice unclear | Operator picks; resume |
| 4 | needs_human (QA-EVIDENCE-001) | A ✅ row has no evidence link | Operator supplies the runbook section / dashboard URL / scan report |
| 4 | needs_human (QA-DORA-001) | DORA baseline values are placeholders | Operator queries the metrics backend; fills in; resume |
| 4 | STALE-001 | RC SHA changed (someone pushed during the chain) | Reset all ✅ to ⏳; re-verify each |

## Operator-side decisions

The CTO (or deploy owner delegate) is pulled in at:

1. **Audience scope at step 1 PLAN** — customer_public vs internal_only changes which conditional sections fire.
2. **CVE / advisory triage at step 2** — for COND-002 hits, validate CVE IDs against MITRE; coordinate with CLO + CISO on disclosure timing.
3. **Progressive-delivery choice at step 3** — canary vs blue_green vs feature_flag vs direct.
4. **Conditional-DEP sign-off at step 4** — production deploy: customer-comms drafted? DBA-approved migration plan present? Compliance sign-off captured?
5. **No-go decision** — if any DEP-NNN is `❌` or any QA-* rule fires `error`, the deploy is no-go. The workflow does NOT auto-pass below 10/10.

## Cross-references

- `../README.md` — CTO 9-block spec.
- `../../../docs/Software Development Process.md` §2(i) + Template §4.7 — Deployment + Deployment Readiness Checklist source.
- DORA Accelerate State of DevOps — four key metrics that DEP-011 captures.
- `../../../skill/release-notes-{author,audit}/SKILL.md`, `../../../skill/deploy-checklist-{author,audit}/SKILL.md`.
