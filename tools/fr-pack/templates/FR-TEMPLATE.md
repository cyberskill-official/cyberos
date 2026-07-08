---
id: FR-001-slug
title: Short imperative title
status: draft            # draft | ready_to_implement | implementing | ready_to_review | reviewing | ready_to_test | testing | done | on_hold | closed (see machine/STATUS-REFERENCE.md)
class: product           # product = net-new feature | improvement = hardening/refactor/audit-remediation
priority: SHOULD         # MUST | SHOULD | COULD
depends_on: []           # ids that must be done first
routed_back_count: 0
awh: N/A                 # N/A unless this repo has a sealed goldenset for the touched area
---

# FR-001: Short imperative title

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

## 5. Protected invariants this FR must not weaken

List anything a gate must never be made green by weakening (auth model, tenant
isolation, audit integrity, consent, etc.). Weakening one is a fork: park and record.
