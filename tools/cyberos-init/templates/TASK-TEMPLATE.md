---
id: TASK-<MODULE>-<NNN>  # module-scoped, e.g. TASK-AUTH-001. Not a bare TASK-001.
title: Short imperative title
template: task@1
type: feature            # FM-108, REQUIRED: feature | bug | improvement | chore
                         #   feature     net-new capability
                         #   bug         something is broken (adds severity: below)
                         #   improvement hardening / refactor / audit-remediation
                         #   chore       maintenance, no behaviour change
module: <module>
status: draft            # draft | ready_to_implement | implementing | ready_to_review | reviewing | ready_to_test | testing | done | on_hold | closed (see cuo/STATUS-REFERENCE.md)
priority: p2             # p0 | p1 | p2 | p3
# severity: sev2         # BUG ONLY. Impact if left unfixed — distinct from priority.
author: "@your-handle"   # quote it: a bare @ is a reserved YAML indicator
department: engineering
created_at: <ISO 8601 with timezone>
ai_authorship: none      # none | assisted | co_authored | generated_then_reviewed
eu_ai_act_risk_class: not_ai   # not_ai | minimal | limited | high
client_visible: false
depends_on: []           # ids that must be done first
routed_back_count: 0
awh: N/A                 # N/A unless this repo has a sealed goldenset for the touched area
---

<!-- Until 2026-07-15 this template shipped `class: product` and `priority: SHOULD` — a
     schema retired on 2026-07-14. FM-108 requires `type` at ERROR severity and priority
     is p0-p3, so the first task a new repo authored from this file failed the audit gate
     immediately. install.sh:651 hands this to every new project, which makes it the FIRST
     artifact anyone touches.

     Nothing caught it because a template is never executed — it is prompt text. Now gated
     by scripts/tests/test_template_schema.sh (t06). -->

# TASK-<MODULE>-<NNN>: Short imperative title

## 1. Description (normative)

State the requirement in normative clauses. Number them; each clause is a testable
promise. The reviewer maps every clause to a named test, and the tester proves each
named test passes before final acceptance.

- 1.1 The system SHALL ...
- 1.2 The system MUST reject ... with <specific error>.
- 1.3 When <condition>, the system SHALL ...

## 2. Acceptance criteria

- [ ] AC for 1.1 - <observable outcome> - test: `<test name>`
- [ ] AC for 1.2 - <observable outcome> - test: `<test name>`
- [ ] AC for 1.3 - <observable outcome> - test: `<test name>`

## 3. Edge cases

Null / empty inputs; extreme bounds; malformed payloads; concurrency / races;
security-class (auth bypass, tenant escape, injection). Each security-class row needs a
test or an ADR.

## 4. Out of scope / non-goals

- ...

## 5. Protected invariants this task must not weaken

List anything a gate must never be made green by weakening (auth model, tenant
isolation, audit integrity, consent, etc.). Weakening one is a fork: park and record.
