---
id: FR-SKILL-202
title: "cyberskill-sg vertical pack - Singapore ACRA, GST e-invoice, CPF, PayNow, IRAS, and PDPA helpers"
module: SKILL
priority: SHOULD
status: ready_to_implement
verify: T
phase: P4
milestone: P4 - vertical-pack marketplace
slice: 1
owner: Stephen Cheng (CTO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_frs: [FR-SKILL-107, FR-TEN-201, FR-TEN-103]
depends_on: [FR-SKILL-107]
blocks: []

source_pages:
  - website/docs/modules/skill/index.html#vertical-packs
  - website/docs/architecture/strategy.html#vertical-packs
source_decisions:
  - DEC-SKILL-202-1 - cyberskill-sg is the first non-Vietnam SEA vertical pack and must follow the public-pack pattern.
  - DEC-SKILL-202-2 - The pack ships as six small skills instead of one monolith so host routing can select the exact helper.

build_envelope:
  language: rust 1.81 + SKILL.md bundles
  service: cyberos/modules/skill/
  new_files:
    - modules/skill/cyberskill-sg/SKILL.md
    - modules/skill/cyberskill-sg/README.md
    - modules/skill/cyberskill-sg/tests/uen_gst_cpf_test.py
    - modules/skill/cyberskill-sg/acceptance/TRIGGER_TESTS.md
  modified_files:
    - modules/skill/MODULE.md
    - website/docs/modules/skill/index.html
  allowed_tools:
    - file_read: modules/skill/**
    - file_write: modules/skill/cyberskill-sg/**
    - bash: PYTHONPATH=modules/cuo python3 -m pytest modules/cuo/tests/test_trigger_tests.py
  disallowed_tools:
    - call live ACRA, IRAS, CPF, or bank APIs without sandbox credentials
    - persist raw UEN, GST registration, or employee salary data in logs

effort_hours: 16
sub_tasks:
  - "2h: pack SKILL.md with trigger descriptions and allowed tools"
  - "3h: UEN and GST reference validators"
  - "3h: CPF contribution estimator with configurable rates"
  - "2h: PayNow metadata helper"
  - "2h: IRAS tax-filing checklist helper"
  - "2h: PDPA compliance checklist helper"
  - "2h: trigger fixtures and docs"
risk_if_skipped: "Singapore customers and the SG HoldCo flip path lack localized operating helpers. Operators would handle ACRA/GST/CPF checks manually, increasing compliance drift and making vertical-pack marketplace claims weaker."
---

## §1 - Description (BCP-14 normative)

The SKILL module **MUST** ship a `cyberskill-sg` vertical pack with six Singapore operating helpers:

1. **MUST** expose `sg-uen-validate` for Singapore UEN-shaped identifier checks, including entity-type prefix validation and checksum fixtures.
2. **MUST** expose `sg-gst-invoice` for GST invoice metadata: GST registration reference, invoice number, supply date, currency, and tax amount.
3. **MUST** expose `sg-cpf-contrib` for CPF contribution estimates from configured employee/employer rates.
4. **MUST** expose `sg-paynow-transfer` for PayNow payment reference metadata generation without initiating live bank transfers.
5. **MUST** expose `sg-iras-tax-filing` for IRAS filing checklist generation and due-date reminders.
6. **MUST** expose `sg-pdpa-compliance` for PDPA handling checklist output covering consent, purpose limitation, retention, and breach escalation.
7. **MUST** declare precise trigger descriptions in each `SKILL.md` so the router can select the SG helper instead of the Vietnam helpers.
8. **MUST** keep live-government and bank integrations mocked unless sandbox credentials are configured.
9. **MUST** redact UEN/GST references in logs to prefix + last 4.
10. **MUST** emit memory audit rows for every generated compliance checklist or payment metadata document with raw identifiers redacted.
11. **MUST** include acceptance trigger fixtures for all six helpers.
12. **MUST NOT** share mutable code with `cyberskill-id` except via generic pack utilities.

## §2 - API Contract

Each helper accepts JSON input and emits a JSON result:

```json
{
  "country": "SG",
  "helper": "sg-gst-invoice",
  "input_ref_redacted": "2019****345A",
  "result": {},
  "warnings": [],
  "trace_id": "32-hex"
}
```

## §3 - Acceptance Criteria

1. Pack indexes as `cyberskill-sg`.
2. Six helper trigger fixtures route to the SG pack.
3. UEN valid/invalid fixtures pass.
4. GST invoice metadata computes tax amount deterministically.
5. CPF estimator handles employee and employer shares separately.
6. PayNow helper emits metadata only and never sends funds.
7. IRAS checklist includes due-date fields.
8. PDPA checklist includes consent, retention, breach, and export checks.
9. Raw identifiers are redacted in logs.
10. Memory audit emission is covered by a mock writer test.

## §4 - Verification

```bash
PYTHONPATH=modules/cuo python3 -m pytest modules/cuo/tests/test_trigger_tests.py
python3 modules/skill/tests/run_corpus.py --skill-id cyberskill-sg
```

## §7 - Dependencies

**Upstream:** FR-SKILL-107.
**Cross-module:** FR-TEN-201 and FR-TEN-103 may consume SG pack outputs.

## §10 - Failure Modes

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Helper routes to wrong country pack | trigger fixture | fail build | tighten description |
| Live API unavailable | sandbox flag absent | mocked response | configure sandbox |
| Raw identifier logged | log scan | fail test | redact |
