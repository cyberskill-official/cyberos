---
id: FR-SKILL-203
title: "cyberskill-id vertical pack - Indonesia NPWP, e-Faktur, BPJS, BRI transfer, DJP tax, and UU PDP helpers"
module: SKILL
priority: COULD
status: ready_to_implement
verify: T
phase: P4
milestone: P4 - vertical-pack marketplace
slice: 1
owner: Stephen Cheng (CTO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_frs: [FR-SKILL-107]
depends_on: [FR-SKILL-107]
blocks: []

source_pages:
  - website/docs/modules/skill/index.html#vertical-packs
  - website/docs/architecture/strategy.html#vertical-packs
source_decisions:
  - DEC-SKILL-203-1 - cyberskill-id follows the vertical-pack pattern after cyberskill-sg.
  - DEC-SKILL-203-2 - Finance and compliance helpers must use mocked government/bank adapters until sandbox credentials exist.

build_envelope:
  language: rust 1.81 + SKILL.md bundles
  service: cyberos/modules/skill/
  new_files:
    - modules/skill/cyberskill-id/SKILL.md
    - modules/skill/cyberskill-id/README.md
    - modules/skill/cyberskill-id/tests/npwp_efaktur_bpjs_test.py
    - modules/skill/cyberskill-id/acceptance/TRIGGER_TESTS.md
  modified_files:
    - modules/skill/MODULE.md
    - website/docs/modules/skill/index.html
  allowed_tools:
    - file_read: modules/skill/**
    - file_write: modules/skill/cyberskill-id/**
    - bash: PYTHONPATH=modules/cuo python3 -m pytest modules/cuo/tests/test_trigger_tests.py
  disallowed_tools:
    - call live DJP, BPJS, BRI, or e-Faktur services without sandbox credentials
    - persist raw NPWP or invoice payloads in logs

effort_hours: 16
sub_tasks:
  - "2h: pack SKILL.md with trigger descriptions and allowed tools"
  - "3h: NPWP normalization and checksum fixtures"
  - "3h: e-Faktur XML metadata helper"
  - "2h: BPJS contribution estimator"
  - "2h: BRI transfer metadata helper"
  - "2h: DJP tax filing checklist helper"
  - "2h: UU PDP privacy checklist helper"
risk_if_skipped: "Indonesia customers lack localized operating helpers. Manual NPWP/e-Faktur/BPJS handling increases compliance risk and weakens the SEA vertical-pack roadmap."
---

## §1 - Description (BCP-14 normative)

The SKILL module **MUST** ship a `cyberskill-id` vertical pack with six Indonesia operating helpers:

1. **MUST** expose `id-npwp-validate` for NPWP normalization, format validation, and checksum fixtures.
2. **MUST** expose `id-efaktur-metadata` for e-Faktur XML metadata generation with deterministic invoice references.
3. **MUST** expose `id-bpjs-contrib` for BPJS contribution estimates from configured employer and employee rates.
4. **MUST** expose `id-bri-transfer` for BRI transfer reference metadata without initiating live payments.
5. **MUST** expose `id-djp-tax-filing` for DJP tax filing checklist generation and due-date reminders.
6. **MUST** expose `id-uupdp-compliance` for UU PDP privacy checklist output covering lawful basis, consent, retention, and breach escalation.
7. **MUST** declare trigger descriptions that distinguish Indonesia requests from Vietnam and Singapore packs.
8. **MUST** keep live external integrations mocked unless sandbox credentials are configured.
9. **MUST** redact NPWP and invoice identifiers in logs to prefix + last 4.
10. **MUST** emit memory audit rows for every compliance checklist or payment metadata document with raw identifiers redacted.
11. **MUST** include acceptance trigger fixtures for all six helpers.
12. **MUST NOT** share mutable country-specific rules with `cyberskill-sg`.

## §2 - API Contract

Each helper accepts JSON input and emits a JSON result:

```json
{
  "country": "ID",
  "helper": "id-efaktur-metadata",
  "input_ref_redacted": "01.***.1234",
  "result": {},
  "warnings": [],
  "trace_id": "32-hex"
}
```

## §3 - Acceptance Criteria

1. Pack indexes as `cyberskill-id`.
2. Six helper trigger fixtures route to the ID pack.
3. NPWP valid/invalid fixtures pass.
4. e-Faktur metadata XML is deterministic.
5. BPJS estimator handles employee and employer shares separately.
6. BRI helper emits metadata only and never sends funds.
7. DJP checklist includes due-date fields.
8. UU PDP checklist includes consent, retention, breach, and export checks.
9. Raw identifiers are redacted in logs.
10. Memory audit emission is covered by a mock writer test.

## §4 - Verification

```bash
PYTHONPATH=modules/cuo python3 -m pytest modules/cuo/tests/test_trigger_tests.py
python3 modules/skill/tests/run_corpus.py --skill-id cyberskill-id
```

## §7 - Dependencies

**Upstream:** FR-SKILL-107.
**Cross-module:** none.

## §10 - Failure Modes

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Helper routes to wrong country pack | trigger fixture | fail build | tighten description |
| Live API unavailable | sandbox flag absent | mocked response | configure sandbox |
| Raw identifier logged | log scan | fail test | redact |
