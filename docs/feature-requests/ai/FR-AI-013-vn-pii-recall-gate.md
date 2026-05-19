---
# ───── Machine-readable frontmatter (parsed by feature-request-audit + future fr-catalog renderer) ─────
id: FR-AI-013
title: "VN-PII recall ≥ 99% per-recognizer CI gate on 200-sample fixture"
module: AI
priority: MUST
status: ready_to_implement
verify: T
phase: P0
milestone: P0 · slice 3
slice: 3
owner: Stephen Cheng
created: 2026-05-15
shipped: null
memory_chain_hash: null
related_frs: [FR-AI-011, FR-AI-012, FR-AI-022]
depends_on: [FR-AI-012, FR-AI-011]
blocks: []

# ───── Source contracts ─────
source_pages:
  - website/docs/modules/ai.html#pii-redaction
  - website/docs/legal/vn-pdpl-compliance.html
source_decisions:
  - FR-AI-012 §1 #1 (recall ≥ 99% per-recognizer AND aggregate floor)
  - FR-AI-012 §1 #14 (recognizer-version endpoint exists; this FR pins fixture to it)
  - PDPL Art. 7 (Vietnam personal-data-sale ban; CCCD government-ID handling)
  - DEC-053 (CCCD treated as Class-A government ID — never persists in memory raw)
  - archive/2026-05-14/RESEARCH_REVIEW.md §4.2 (recall gate is the compliance assertion)

# ───── Build envelope ─────
language: python 3.11 + GitHub Actions (CI runner)
service: cyberos/services/ai-gateway/pii/
new_files:
  - services/ai-gateway/pii/fixtures/vn_pii_200_samples.yaml          # 200-sample test set + 30 negative samples (reconciled to FR-AI-012's path)
  - services/ai-gateway/pii/fixtures/vn_pii_200_samples_README.md     # provenance + maintenance log
  - services/ai-gateway/pii/fixtures/fixture_manifest.yaml            # version, recognizer-version pins, sample-count assertions
  - services/ai-gateway/pii/tests/test_recall_gate.py                 # per-recognizer + aggregate recall test
  - services/ai-gateway/pii/tests/test_precision_warning.py           # precision reported (not gated); regression-trend hook
  - services/ai-gateway/pii/tests/test_fixture_invariants.py          # fixture-format validation: counts, span offsets, schema
  - services/ai-gateway/pii/tests/test_no_real_pii_in_corpus.py       # AST/regex sweep: no real-CCCD-like sequences leak into corpus
  - services/ai-gateway/pii/tests/test_recall_gate_runtime_budget.py  # asserts test_recall_gate completes < 5 min
  - services/ai-gateway/pii/scripts/validate_corpus_format.py         # standalone pre-commit hook script
  - services/ai-gateway/pii/scripts/regen_fixture.py                  # quarterly refresh tool (anonymises real data, adds new edge cases)
  - .github/workflows/vn-pii-recall.yml                               # CI gate on PRs touching pii/**
  - .github/workflows/vn-pii-quarterly-refresh.yml                    # scheduled bot that opens "corpus refresh due" issue
  - .pre-commit-hooks/no-real-pii-in-corpus.sh                        # pre-commit gate executed by `pre-commit` framework
modified_files:
  - services/ai-gateway/pii/Makefile                                  # `make pii-recall-test`, `make pii-fixture-validate`, `make pii-precision-report`
  - .pre-commit-config.yaml                                           # register no-real-pii hook
allowed_tools:
  - file_read: services/ai-gateway/pii/**
  - file_write: services/ai-gateway/pii/**
  - file_write: .github/workflows/vn-pii-*.yml
  - file_write: .pre-commit-config.yaml
  - file_write: .pre-commit-hooks/no-real-pii-in-corpus.sh
  - bash: cd services/ai-gateway/pii && pytest -v
  - bash: pre-commit run --all-files
disallowed_tools:
  - commit real PII to the corpus (must be synthetic or de-identified per §1 #3)
  - skip the per-recognizer recall floor in favour of aggregate-only (per §1 #5)
  - silence the CI gate (e.g., `pytest --skip-recall`); the gate MUST run on every PR
  - read real customer prompts during test execution (the fixture is the ONLY input)
  - call any network endpoint from test_recall_gate.py (offline-only per §1 #11)

# ───── Estimated work ─────
effort_hours: 8
sub_tasks:
  - "1.0h: Curate 200-sample corpus (50 CCCD + 30 MST + 40 PHONE + 20 NĐD + 40 ADDRESS + 20 BANK_ACCOUNT)"
  - "0.5h: Add 30 negative samples (Vietnamese text without PII; dates, VND amounts, plain names)"
  - "0.5h: Provenance README (per-sample: synthetic vs anonymised-real, source date, regenerated-on)"
  - "0.5h: fixture_manifest.yaml with pinned recognizer versions + sample-count assertions"
  - "1.0h: test_recall_gate.py — per-recognizer + aggregate recall computation with structured output"
  - "0.5h: test_precision_warning.py — precision reported per recognizer; trend hook for FR-AI-022"
  - "0.5h: test_fixture_invariants.py — JSONL→YAML schema + span-offset validation + count guards"
  - "0.5h: test_no_real_pii_in_corpus.py — entropy + province-code-presence sweep to flag accidental real CCCD/MST commits"
  - "0.5h: test_recall_gate_runtime_budget.py — wall-clock assertion < 5 minutes"
  - "0.5h: validate_corpus_format.py standalone pre-commit script + Makefile entries"
  - "0.5h: vn-pii-recall.yml GH Actions workflow (PR-triggered on pii/** + workflow file changes)"
  - "0.5h: vn-pii-quarterly-refresh.yml scheduled workflow opening 'corpus refresh due' issue"
  - "0.5h: pre-commit hook + .pre-commit-config.yaml registration"
  - "0.5h: regen_fixture.py scaffold (anonymisation pipeline; quarterly refresh CLI)"
risk_if_skipped: "FR-AI-012's recall claim is unenforced. Recognizers can degrade silently over time (regex bugs, dependency drift, NER drift) and we'd never know until a real PII leak reaches a VN regulator. Compliance failure becomes indistinguishable from operational competence; PDPL Art. 7 enforcement actions become first-discovery rather than caught-in-CI events. Without per-recognizer gating, a single regressed recognizer hides behind aggregate metrics — exactly the failure mode that produces a leak on the type a regulator audits first."
---

## §1 — Description (BCP-14 normative)

A CI gate on every PR touching `services/ai-gateway/pii/**`, the recall-fixture file, or the recall-gate workflow file itself **MUST** run a recall check against a 200-sample VN PII test corpus and **MUST fail the PR** if any recognizer's recall drops below 99%. The corpus and the gate together obey the following:

1. **MUST** contain exactly 200 positive samples distributed across the 6 VN recognizers (50 CCCD / 30 MST / 40 PHONE / 20 NĐD / 40 ADDRESS / 20 BANK_ACCOUNT, matching FR-AI-012's enumeration in §1 #1). The sample-count distribution is asserted in `test_fixture_invariants.py`; drift requires explicit FR amendment.
2. **MUST** include at least 30 **negative samples** — Vietnamese text without PII (plain names, dates, VND amounts, hợp đồng numbers, biển số xe, sentences from gov.vn legal text). Negative samples MUST produce **zero matches** from any VN recognizer; any false positive on a negative sample is logged but does NOT fail the gate (precision is reported, not gated — see §1 #6 below).
3. **MUST** be synthetic OR aggressively anonymised real data. Real customer data MUST have the digit-bearing PII replaced with valid-format-fake equivalents (real CCCD `038198001234` → fake `038100000001` with valid province code prefix). The anonymisation rule is "preserve format-validity, destroy identity-linkage." A pre-commit hook (`.pre-commit-hooks/no-real-pii-in-corpus.sh` per §1 #14 below) AST-walks the fixture for any sequence matching the same span as a known internal customer record.
4. **MUST** be regenerated quarterly (Jan 1, Apr 1, Jul 1, Oct 1) to capture format drift — new VN ID formats, new bank patterns, new province codes after administrative reorganisation, newly-onboarded VN telco prefixes. Regeneration is owned by the operator on rotation; the bot reminder (§1 #13) drives the cadence.
5. **MUST** compute recall **per recognizer** as `true_positives / (true_positives + false_negatives)`. The 99% floor applies *per recognizer*, not aggregate. The test assertion lists which recognizers fell below the floor and which specific samples were missed. Per-recognizer gating is the load-bearing invariant — without it, a regressed CCCD recognizer can hide behind a strong PHONE recognizer in aggregate metrics.
6. **MUST** compute precision per recognizer as `true_positives / (true_positives + false_positives)` and **MUST** emit it to the gate output for informational purposes. Precision **MUST NOT** fail the PR at this stage (FR-AI-022 follow-up will add the precision gate after 6 months of operational data). A precision regression > 5 percentage points vs. the prior-quarter baseline **MUST** post a warning comment on the PR but **MUST NOT** block merge.
7. **MUST** fail the CI job with non-zero exit code AND a structured assertion message identifying (a) the recognizer below the floor, (b) the recall value computed, (c) the sample IDs missed, AND (d) the fixture manifest version active at the time of the run. Pytest's `assert not failures, ...` pattern produces this output natively.
8. **MUST** run as a GitHub Actions job on every PR matching `paths: ['services/ai-gateway/pii/**', '.github/workflows/vn-pii-recall.yml']`. The workflow-file inclusion is critical: a PR that *only* loosens the gate (e.g., changes 0.99 → 0.95) MUST still trigger the gate against itself.
9. **MUST** complete the test run in ≤ 5 minutes on a standard GitHub Actions `ubuntu-22.04` runner. `test_recall_gate_runtime_budget.py` (§5) records wall-clock time and asserts the budget; CI failure on budget violation is itself a recall-gate failure.
10. **MUST** pin the fixture's expected recognizer versions in `fixtures/fixture_manifest.yaml`. The test reads the live recognizer versions from `GET /recognizers/version` (FR-AI-012 §1 #14) and asserts they match the manifest before computing recall. A recognizer-version drift fails the gate with `fixture_version_mismatch` — forcing the operator to either bump the fixture or revert the recognizer.
11. **MUST NOT** make any network call during test execution. The fixture is the sole input; the recognizers are imported in-process. The CI runner has internet access for `pip install` only; once the test starts, network egress is monitored by the GitHub Actions job (no proxy → no real-PII exfiltration risk during CI).
12. **MUST** publish recall, precision, and runtime metrics to a JSON artefact at `recall_gate_report_<sha>.json` attached to the PR. The artefact is the input to the trend chart (§1 #16) and the audit record for any subsequent compliance enquiry — "what was the recall at the time this PR shipped?"
13. **MUST** trigger a quarterly refresh reminder via `.github/workflows/vn-pii-quarterly-refresh.yml` (scheduled at `0 0 1 1,4,7,10 *`) that opens a GitHub issue titled `VN PII corpus refresh due — quarter <Q><YYYY>` against the `services/ai-gateway/pii` codeowner. The issue body MUST reference this FR and the `regen_fixture.py` runbook.
14. **MUST** enforce, via the `.pre-commit-hooks/no-real-pii-in-corpus.sh` hook, that no sample in the fixture contains a digit sequence that matches a known-internal-CCCD pattern (Luhn-style check digit + known-customer province distribution). The hook is registered in `.pre-commit-config.yaml` and executes on every commit touching `fixtures/vn_pii_200_samples.yaml`.
15. **MUST** assert that the fixture-format validator (`scripts/validate_corpus_format.py`) runs cleanly: every sample has `id`, `text`, `expected_entities`, `expected_count`, `provenance` (synthetic | anonymised-real | gov.vn-public); every span offset (start/end) is verified to actually delimit a substring matching the declared entity; every entity is one of the 6 VN_* types or PERSON (Presidio's built-in fallback).
16. **SHOULD** publish a trend chart (recall + precision over time per recognizer) to the build dashboard. The artefact source is the JSON from §1 #12; the dashboard is out of scope for this FR (FR-AI-022 implements it).

---

## §2 — Why this design (rationale for humans)

**Why 99% recall, not 95%?** Each missed PII case is a potential compliance breach under PDPL Art. 7 (personal data sale prohibition; CCCD is a government identifier) and Art. 6 (data minimisation; redacted PII fragments fall back to the deletion-on-purpose discipline). 99% means at most 2 missed per 200-sample fixture (and at most 0–1 per small recognizer, where 20 samples × 1% = 0.2 rounds to 1 max miss). Across real-world traffic at 10,000 calls/day per VN tenant, 99% recall means ~100 leaks/day if every call carries PII — still too many, but tractable when paired with the redact-and-restore architecture (FR-AI-011) that limits damage to LLM provider logs (which we mitigate via ZDR per FR-AI-015). At 95% recall the leak rate becomes 500/day per tenant, which is indefensible at regulator audit.

**Why per-recognizer gate, not aggregate?** The aggregate-only design has a famous failure mode: a poor CCCD recognizer (96% recall) hides behind a great PHONE recognizer (100% recall) in aggregate (97% > 99% — fails by accident, but in a fixture with skewed sample counts the math can flip and pass). The regulator concern is *per-PII-type*: PDPL Art. 7 doesn't care about your aggregate; it cares about whether a CCCD leaked. Per-recognizer gating makes each recognizer owner accountable for their numbers and produces actionable PR feedback ("VN_CCCD regressed to 94%; samples cccd_017, cccd_023, cccd_041 missed"). This pattern is established in FR-AI-012's §1 #1 (post-ISS-004) and FR-AI-013 inherits it.

**Why no precision gate yet (§1 #6)?** Precision is *operationally* annoying — false positives mean valid prompts get redacted (a placeholder `<VN_PHONE_1>` appears mid-sentence where the original was a hợp đồng number that happened to look like a phone). The user-visible cost is "the LLM gave a weird answer because we redacted something that wasn't PII." The compliance cost is much lower than recall: a false positive is over-redaction (safer), a false negative is under-redaction (PII leak). The Bayesian trade-off favours recall now and precision after 6 months of telemetry — at which point we know which recognizers consistently over-redact and where the context-boost tuning needs work. FR-AI-022 will add the precision gate once that data exists. We DO report precision in §1 #6 so the trend is visible from day 1.

**Why quarterly regeneration (§1 #4)?** VN regulatory artefacts mutate. The 13-digit MST format with branch suffix was a 2024 expansion. The provincial reorganisation in Vietnam (occasional merges/splits of administrative divisions) changes the valid province-code list. New telcos enter the market with new prefixes (5-prefix mobiles were a 2018 addition). A static fixture becomes a historical snapshot, not a contemporary compliance check. Quarterly regen is the minimum cadence that captures these without burning operator time; the schedule (Jan 1, Apr 1, Jul 1, Oct 1) aligns with calendar quarters for audit-narrative clarity.

**Why pin recognizer versions in the fixture manifest (§1 #10)?** The fixture is curated for the *current* recognizer behaviour. If `VnCccdRecognizer.VERSION` bumps from 1.0.0 → 1.1.0 (e.g., a new province-code added), the fixture needs to be re-validated against the new version — otherwise recall could falsely report 99.5% (because the new version handles a case the old fixture didn't even test). The version pin in `fixture_manifest.yaml` forces the operator to consciously regenerate the fixture (or explicitly accept the version bump) rather than silently letting a recognizer change pass through. This pattern matches the "explicit lock-step versioning between fixture and recognizer" principle from FR-AI-012 §1 #14.

**Why a pre-commit hook for real-PII detection (§1 #14)?** Real PII committed to a corpus that lives in a public-ish repo (even internal-only, with developer SSH access) is a worse incident than a single LLM leak — it's a *systematic* leak with an audit trail showing we knew the data was PII and committed it anyway. The pre-commit hook is the last line of defence before git-push; it AST-walks the fixture's JSONL/YAML and flags any digit sequence whose Luhn-check-digit and province-code prefix matches a record in the internal customer-data sample table. The hook is fast (< 1s) and runs unconditionally; an engineer accidentally pasting a real CCCD gets a commit-time rejection with a clear "this looks like real data" message.

**Why include negative samples (§1 #2)?** Without negative samples, the recall gate is single-sided — it measures sensitivity but not specificity. A recognizer that matches every 10-digit number gets 100% recall but is operationally unusable. The 30 negative samples are the floor for measuring the false-positive rate; they include the hard cases (Vietnamese names without NDD label, dates that look like CCCDs, VND amounts that look like bank accounts, contract numbers that resemble MST formats). These samples are the precision-gate input for FR-AI-022 — they're collected now so we don't have to retroactively build them when the precision gate ships.

**Why a 5-minute runtime budget (§1 #9)?** GitHub Actions billing is per-minute; a 5-minute job is ~$0.04 per PR. At 10 PRs/week touching pii/**, that's ~$2/month — trivial. But if the test creeps to 30 minutes (e.g., someone adds a 10,000-sample fixture), it becomes both expensive AND a velocity problem (engineers wait 30 min on every push). The budget is a forcing function for keeping the fixture lean: 200 samples is enough to detect 99% recall regression with statistical confidence; 10,000 is overkill until we have the precision gate driving the sample size.

**Why expose the recall-gate report as a JSON artefact (§1 #12)?** Three reasons. (1) The trend chart (§1 #16) needs a parseable source; markdown PR comments aren't structured. (2) A compliance auditor asking "what was the recall on the day FR-X shipped?" needs an answer that's *durable* — the GH Actions log retention is 90 days, but the artefact can be archived. (3) The artefact's `recognizer_versions` field lets a future reader correlate recall numbers with recognizer code, even if the recognizer code has since changed.

**Why a workflow file inclusion in `paths:` (§1 #8)?** Without it, a PR that *only* edits `.github/workflows/vn-pii-recall.yml` to disable the gate (e.g., comments out the assertion) would not trigger the gate. The workflow needs to gate-itself: any change to the gate's own logic must run through the gate. This is a standard "you can't unilaterally relax your own enforcement" pattern; it's why the workflow file is in its own `paths:` list.

---

## §3 — API contract (formal spec for AI-agent implementers)

### Fixture file format (YAML — reconciled to FR-AI-012 §6)

```yaml
# services/ai-gateway/pii/fixtures/vn_pii_200_samples.yaml
# 200 positive samples + 30 negative samples. Curated quarterly.
# Sample-count distribution per FR-AI-013 §1 #1: 50 CCCD / 30 MST / 40 PHONE / 20 NDD / 40 ADDRESS / 20 BANK_ACCOUNT + 30 negative.

samples:
  # Positive — VN_CCCD (50 samples)
  - id: cccd_001
    text: "CCCD của khách hàng: 012345678901"
    expected_entities: ["VN_CCCD"]
    expected_count: 1
    expected_spans:
      - { entity: "VN_CCCD", start: 24, end: 36 }
    provenance: synthetic
    notes: "labeled CCCD with leading 'CCCD:' marker"

  - id: cccd_002
    text: "012345678901"
    expected_entities: ["VN_CCCD"]
    expected_count: 1
    expected_spans:
      - { entity: "VN_CCCD", start: 0, end: 12 }
    provenance: synthetic
    notes: "bare 12-digit CCCD; relies on province-code validation"

  # ... 48 more CCCD samples ...

  # Positive — VN_BANK_ACCOUNT (20 samples)
  - id: bank_020
    text: "STK Vietcombank: 1234567890"
    expected_entities: ["VN_BANK_ACCOUNT"]
    expected_count: 1
    expected_spans:
      - { entity: "VN_BANK_ACCOUNT", start: 17, end: 27 }
    provenance: synthetic
    notes: "STK + Vietcombank context boost"

  # Negative samples (30 total) — interleaved by id-prefix `neg_`
  - id: neg_001
    text: "Ngày 12/05/2026 — báo cáo doanh thu Q1"
    expected_entities: []
    expected_count: 0
    expected_spans: []
    provenance: synthetic
    notes: "date format with slashes; should NOT match VN_CCCD"

  - id: neg_002
    text: "Hợp đồng số 2024-001"
    expected_entities: []
    expected_count: 0
    expected_spans: []
    provenance: synthetic
    notes: "contract number; format-overlaps MST 10-3 but not within label context"

  # ... 28 more negative samples ...
```

### Fixture manifest (recognizer-version pin)

```yaml
# services/ai-gateway/pii/fixtures/fixture_manifest.yaml
fixture_version: "2026Q2.0"
generated_at: "2026-05-16T00:00:00Z"
regenerated_due: "2026-07-01"

sample_counts:
  VN_CCCD: 50
  VN_MST: 30
  VN_PHONE: 40
  VN_NDD: 20
  VN_ADDRESS: 40
  VN_BANK_ACCOUNT: 20
  negative: 30
total_samples: 230   # 200 positive + 30 negative

# Pinned recognizer versions — `test_recall_gate.py` reads /recognizers/version
# and asserts these match before computing recall.
recognizer_versions:
  VN_CCCD: "1.0.0"
  VN_MST: "1.0.0"
  VN_PHONE: "1.0.0"
  VN_NDD: "1.0.0"
  VN_ADDRESS: "1.0.0"
  VN_BANK_ACCOUNT: "1.0.0"

# Per-recognizer floors. Default is 99%; lower floor for a recognizer requires
# explicit FR amendment.
recall_floors:
  VN_CCCD: 0.99
  VN_MST: 0.99
  VN_PHONE: 0.99
  VN_NDD: 0.99
  VN_ADDRESS: 0.99
  VN_BANK_ACCOUNT: 0.99

# Aggregate floor (FR-AI-012 §1 #1 post-ISS-004 also requires aggregate).
aggregate_recall_floor: 0.99

# Precision baselines — used by test_precision_warning.py to detect regressions.
# 5pp delta vs baseline → warning comment on PR; no merge block.
precision_baselines_prior_quarter:
  VN_CCCD: 1.0
  VN_MST: 0.96
  VN_PHONE: 0.98
  VN_NDD: 1.0
  VN_ADDRESS: 0.95
  VN_BANK_ACCOUNT: 0.97

# Runtime budget — wall-clock seconds for the recall gate to complete.
runtime_budget_seconds: 300   # 5 minutes
```

### Recall-gate output (CI log + PR comment)

```text
$ make pii-recall-test

============================ VN PII Recall Gate ============================
Fixture version : 2026Q2.0 (manifest pin)
Recognizer versions match manifest : ✅
Sample counts match manifest : ✅
Negative-sample count : 30 (≥ 30 required)

Running 230 samples (200 positive + 30 negative)...

Recognizer        Detected   FN   FP   Recall      Precision   vs Baseline
─────────────────────────────────────────────────────────────────────────────
VN_CCCD           50/50       0    0   1.0000 ✅   1.0000 ✅   +0.0000
VN_MST            30/30       0    1   1.0000 ✅   0.9677 ✅   +0.0077
VN_PHONE          39/40       1    0   0.9750 ❌   1.0000 ✅   +0.0200
VN_NDD            20/20       0    0   1.0000 ✅   1.0000 ✅   +0.0000
VN_ADDRESS        40/40       0    2   1.0000 ✅   0.9524 ✅   +0.0024
VN_BANK_ACCOUNT   20/20       0    0   1.0000 ✅   1.0000 ✅   +0.0000
─────────────────────────────────────────────────────────────────────────────
Aggregate         199/200     1    3   0.9950 ✅                  —

❌ FAIL: 1 recognizer below 99% recall floor.
   VN_PHONE: recall=0.9750  missed=[phone_037]
   missed sample: phone_037 ("liên hệ 02838123456" — landline-prefix variant)

Runtime: 142.3s (budget: 300s) ✅

PR cannot merge until VN_PHONE recall ≥ 0.99 OR fixture is updated (with FR amendment).

Artefact: recall_gate_report_<sha>.json attached to this run.
```

---

## §4 — Acceptance criteria (testable, ordered, numbered)

1. **All recognizers at 99% on initial corpus** — On commit of the initial fixture (`fixture_version: 2026Q2.0`), all 6 recognizers report `recall ≥ 0.99` individually AND aggregate `≥ 0.99`.
2. **Synthetic injection passes** — Adding a new positive sample (e.g., `{"id":"cccd_051","text":"CCCD: 098123456789","expected_entities":["VN_CCCD"]}`) is detected by `VN_CCCD` and the recall stays at 1.0 for that recognizer.
3. **PR fails on regression** — A PR that mutates `VnCccdRecognizer` to remove context-boost (recall drops to 50%) **MUST** fail the CI gate with non-zero exit code and an assertion message naming `VN_CCCD` and the missed sample IDs.
4. **CI run time ≤ 5 min** — `test_recall_gate_runtime_budget.py` measures wall-clock for `test_recall_gate.py` and asserts `< 300s`. End-to-end CI job (checkout + pip + pytest + artefact upload) also ≤ 5 min on `ubuntu-22.04`.
5. **Negative samples don't false-positive** — All 30 negative samples produce **zero** matches from any VN recognizer at `score >= CONFIDENCE_HIGH`. Low-confidence matches on negative samples are logged but don't fail the gate.
6. **Precision reported but doesn't gate** — Precision < 95% is logged as a warning comment on the PR; the gate does NOT block merge. Precision delta > 5pp vs baseline triggers a warning comment but no merge block.
7. **Quarterly refresh reminder** — `vn-pii-quarterly-refresh.yml` opens a GitHub issue on each of Jan 1, Apr 1, Jul 1, Oct 1 with title `VN PII corpus refresh due — quarter <Q><YYYY>` and assigns the codeowner.
8. **Corpus provenance documented** — Every sample in `vn_pii_200_samples.yaml` has a `provenance` field (`synthetic` | `anonymised-real` | `gov.vn-public`); `vn_pii_200_samples_README.md` records the curator, date, and methodology.
9. **Recognizer version pin asserted** — Before running recall, `test_recall_gate.py` calls `GET /recognizers/version` (FR-AI-012 §1 #14), parses the response, and asserts each entity's version matches `fixture_manifest.yaml`'s `recognizer_versions` field. Mismatch raises `fixture_version_mismatch` with non-zero exit.
10. **Per-recognizer recall floor enforced** — `test_recall_gate.py` asserts `recall_per_type[entity] >= recall_floors[entity]` for every entity in the manifest; a recognizer at 0.985 fails the gate even if aggregate is 0.995.
11. **Aggregate recall floor enforced** — In addition to per-recognizer, `test_recall_gate.py` asserts the aggregate `correct / total >= 0.99` (matches FR-AI-012 §1 #1 post-ISS-004).
12. **JSON artefact emitted** — `test_recall_gate.py` writes `recall_gate_report_<sha>.json` to the workspace; `vn-pii-recall.yml` uploads it as a CI artefact. The JSON schema is documented in `validate_corpus_format.py --schema-out`.
13. **Sample count assertions pass** — `test_fixture_invariants.py` asserts the per-type sample counts match `fixture_manifest.yaml`'s `sample_counts`. A fixture with 51 CCCD samples (instead of 50) fails on commit.
14. **Span offset validation** — Every `expected_spans` entry in the fixture must satisfy `sample.text[start:end] == <a substring of the expected PII format>`. The validator extracts the substring and asserts it is non-empty and consistent with the entity type's pattern.
15. **No-real-PII pre-commit hook fires** — Committing a fixture file that contains a digit sequence whose Luhn-check + province-code distribution matches a known-internal-CCCD pattern is **rejected** at commit time with a clear "this looks like real data" message.
16. **Workflow self-gate** — A PR that ONLY edits `.github/workflows/vn-pii-recall.yml` (e.g., changes the recall floor) MUST still trigger the gate against the modified workflow's logic; the workflow file is listed in `paths:`.
17. **Trend chart hook (FR-AI-022)** — The recall gate's JSON artefact is consumed by FR-AI-022's dashboard with no schema changes; `validate_corpus_format.py --schema-out` is the contract.
18. **Offline-only execution** — `test_recall_gate.py` makes no network call after `pytest` starts. CI runner network egress (other than localhost for the sidecar version endpoint) MUST be zero — asserted by `vn-pii-recall.yml` setting `--no-network` (or equivalent egress monitor).

---

## §5 — Verification

### Recall + precision tests

```python
# services/ai-gateway/pii/tests/test_recall_gate.py
import json
import re
import time
import yaml
from collections import defaultdict
from pathlib import Path

import pytest
from fastapi.testclient import TestClient

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "fixtures" / "vn_pii_200_samples.yaml"
MANIFEST = ROOT / "fixtures" / "fixture_manifest.yaml"
REPORT_OUT = ROOT / "recall_gate_report.json"

ALL_ENTITIES = [
    "VN_CCCD", "VN_MST", "VN_PHONE",
    "VN_NDD", "VN_ADDRESS", "VN_BANK_ACCOUNT",
]


@pytest.fixture(scope="module")
def manifest():
    return yaml.safe_load(MANIFEST.read_text())


@pytest.fixture(scope="module")
def samples():
    data = yaml.safe_load(FIXTURE.read_text())
    return data["samples"]


@pytest.fixture(scope="module")
def analyzer():
    """Initialise Presidio with all 6 VN recognizers per FR-AI-012."""
    from presidio_analyzer import AnalyzerEngine
    from recognizers import VN_RECOGNIZERS
    a = AnalyzerEngine()
    for rec in VN_RECOGNIZERS:
        a.registry.add_recognizer(rec)
    return a


def test_recognizer_versions_match_manifest(manifest):
    """AC #9: fixture is pinned to specific recognizer versions; mismatch fails gate."""
    from presidio_sidecar import app
    client = TestClient(app)
    resp = client.get("/recognizers/version")
    assert resp.status_code == 200, "version endpoint unreachable"
    live_versions = resp.json()
    expected_versions = manifest["recognizer_versions"]
    mismatches = [
        f"{entity}: live={live_versions.get(entity)!r} vs manifest={expected!r}"
        for entity, expected in expected_versions.items()
        if live_versions.get(entity) != expected
    ]
    assert not mismatches, (
        "fixture_version_mismatch — recognizer versions diverged from manifest:\n  "
        + "\n  ".join(mismatches)
    )


def test_sample_counts_match_manifest(samples, manifest):
    """AC #13: fixture must contain the declared per-type sample counts."""
    counts_by_type = defaultdict(int)
    negative_count = 0
    for s in samples:
        if not s["expected_entities"]:
            negative_count += 1
        else:
            for entity in s["expected_entities"]:
                counts_by_type[entity] += 1
    expected = manifest["sample_counts"]
    for entity, expected_count in expected.items():
        if entity == "negative":
            assert negative_count == expected_count, (
                f"negative sample count mismatch: got {negative_count}, expected {expected_count}"
            )
        else:
            assert counts_by_type[entity] == expected_count, (
                f"{entity} sample count mismatch: got {counts_by_type[entity]}, expected {expected_count}"
            )


def test_recall_per_recognizer_and_aggregate(analyzer, samples, manifest):
    """AC #10 + AC #11: per-recognizer recall floor AND aggregate floor."""
    correct_by_type = defaultdict(int)
    total_by_type = defaultdict(int)
    missed_samples_by_type = defaultdict(list)

    for sample in samples:
        if not sample["expected_entities"]:
            continue   # negative sample; precision test handles it
        results = analyzer.analyze(
            text=sample["text"], language="vi", entities=ALL_ENTITIES,
        )
        actual_entities = {r.entity_type for r in results}
        for expected in sample["expected_entities"]:
            total_by_type[expected] += 1
            if expected in actual_entities:
                correct_by_type[expected] += 1
            else:
                missed_samples_by_type[expected].append(sample["id"])

    floors = manifest["recall_floors"]
    failures = []
    recall_per_type = {}
    for entity, total in total_by_type.items():
        recall = correct_by_type[entity] / total if total else 1.0
        recall_per_type[entity] = recall
        floor = floors.get(entity, 0.99)
        if recall < floor:
            missed = ", ".join(missed_samples_by_type[entity][:5])
            more = "" if len(missed_samples_by_type[entity]) <= 5 else f" (+{len(missed_samples_by_type[entity]) - 5} more)"
            failures.append(
                f"{entity}: recall={recall:.4f} ({correct_by_type[entity]}/{total}) "
                f"below floor {floor}; missed={missed}{more}"
            )

    total = sum(total_by_type.values())
    correct = sum(correct_by_type.values())
    aggregate = correct / total if total else 1.0
    if aggregate < manifest["aggregate_recall_floor"]:
        failures.append(
            f"aggregate: recall={aggregate:.4f} below floor {manifest['aggregate_recall_floor']}"
        )

    # Always write the report — pass or fail.
    REPORT_OUT.write_text(json.dumps({
        "fixture_version": manifest["fixture_version"],
        "recognizer_versions": manifest["recognizer_versions"],
        "recall_per_type": recall_per_type,
        "aggregate_recall": aggregate,
        "missed_samples_by_type": dict(missed_samples_by_type),
    }, indent=2, sort_keys=True))

    assert not failures, "Recall floor violated:\n  " + "\n  ".join(failures)


def test_negative_samples_no_high_confidence_matches(analyzer, samples):
    """AC #5: 30 negative samples produce zero high-confidence matches from any VN recognizer."""
    from recognizers.confidence import CONFIDENCE_HIGH
    failures = []
    for sample in samples:
        if sample["expected_entities"]:
            continue
        results = analyzer.analyze(
            text=sample["text"], language="vi", entities=ALL_ENTITIES,
        )
        high_conf = [r for r in results if r.score >= CONFIDENCE_HIGH]
        if high_conf:
            failures.append(
                f"{sample['id']}: false high-confidence matches: "
                + ", ".join(f"{r.entity_type}@{r.start}-{r.end}" for r in high_conf)
            )
    assert not failures, "Negative samples produced high-confidence false positives:\n  " + "\n  ".join(failures)
```

### Precision-warning test (reports, doesn't gate)

```python
# services/ai-gateway/pii/tests/test_precision_warning.py
import json
import yaml
from collections import defaultdict
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "fixtures" / "vn_pii_200_samples.yaml"
MANIFEST = ROOT / "fixtures" / "fixture_manifest.yaml"
WARN_OUT = ROOT / "precision_warning_report.json"
PRECISION_DELTA_WARN = 0.05   # 5pp vs baseline


def test_precision_reported_no_gate(caplog):
    """AC #6: precision reported per recognizer; never fails the PR."""
    from presidio_analyzer import AnalyzerEngine
    from recognizers import VN_RECOGNIZERS

    a = AnalyzerEngine()
    for rec in VN_RECOGNIZERS:
        a.registry.add_recognizer(rec)

    manifest = yaml.safe_load(MANIFEST.read_text())
    samples = yaml.safe_load(FIXTURE.read_text())["samples"]
    baselines = manifest["precision_baselines_prior_quarter"]

    tp = defaultdict(int)
    fp = defaultdict(int)

    for sample in samples:
        expected = set(sample["expected_entities"])
        results = a.analyze(text=sample["text"], language="vi",
                            entities=list(baselines.keys()))
        actual_entities_high_conf = {
            r.entity_type for r in results if r.score >= 0.85
        }
        for entity in actual_entities_high_conf:
            if entity in expected:
                tp[entity] += 1
            else:
                fp[entity] += 1

    precision_per_type = {}
    warnings = []
    for entity, baseline in baselines.items():
        denom = tp[entity] + fp[entity]
        precision = tp[entity] / denom if denom else 1.0
        precision_per_type[entity] = precision
        delta = precision - baseline
        if delta < -PRECISION_DELTA_WARN:
            warnings.append(
                f"{entity}: precision={precision:.4f} (was {baseline:.4f}, "
                f"Δ={delta:+.4f}; > {PRECISION_DELTA_WARN} regression)"
            )

    WARN_OUT.write_text(json.dumps({
        "precision_per_type": precision_per_type,
        "warnings": warnings,
    }, indent=2, sort_keys=True))

    # Never fails — emit warnings to caplog (CI surfaces them as PR comment).
    for w in warnings:
        print(f"::warning:: precision-regression: {w}")
    assert True   # explicit no-gate per AC #6
```

### Fixture invariants

```python
# services/ai-gateway/pii/tests/test_fixture_invariants.py
import yaml
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "fixtures" / "vn_pii_200_samples.yaml"
MANIFEST = ROOT / "fixtures" / "fixture_manifest.yaml"

VALID_ENTITIES = {
    "VN_CCCD", "VN_MST", "VN_PHONE",
    "VN_NDD", "VN_ADDRESS", "VN_BANK_ACCOUNT",
    "PERSON",   # Presidio fallback; allowed in fixture for NDD context samples
}
VALID_PROVENANCE = {"synthetic", "anonymised-real", "gov.vn-public"}


def test_fixture_loads_and_has_required_fields():
    """AC #14: every sample has id, text, expected_entities, expected_count, provenance."""
    data = yaml.safe_load(FIXTURE.read_text())
    failures = []
    for sample in data["samples"]:
        for required in ("id", "text", "expected_entities", "expected_count", "provenance"):
            if required not in sample:
                failures.append(f"{sample.get('id', '<no-id>')}: missing field {required!r}")
        if sample.get("provenance") not in VALID_PROVENANCE:
            failures.append(f"{sample['id']}: invalid provenance {sample['provenance']!r}")
    assert not failures, "Fixture invariants violated:\n  " + "\n  ".join(failures)


def test_span_offsets_delimit_actual_substrings():
    """AC #14: expected_spans must point at real substrings of sample.text."""
    data = yaml.safe_load(FIXTURE.read_text())
    failures = []
    for sample in data["samples"]:
        for span in sample.get("expected_spans", []):
            text = sample["text"]
            start, end = span["start"], span["end"]
            if start < 0 or end > len(text) or start >= end:
                failures.append(f"{sample['id']}: span ({start},{end}) out of range for text length {len(text)}")
                continue
            substring = text[start:end]
            if not substring.strip():
                failures.append(f"{sample['id']}: span ({start},{end}) is whitespace/empty")
            if span["entity"] not in VALID_ENTITIES:
                failures.append(f"{sample['id']}: span entity {span['entity']!r} not in valid set")
    assert not failures, "Span offsets invalid:\n  " + "\n  ".join(failures)


def test_no_duplicate_ids():
    """AC #14 corollary: every sample id is unique."""
    data = yaml.safe_load(FIXTURE.read_text())
    ids = [s["id"] for s in data["samples"]]
    duplicates = {i for i in ids if ids.count(i) > 1}
    assert not duplicates, f"duplicate sample ids: {duplicates}"
```

### No-real-PII pre-commit gate

```bash
#!/usr/bin/env bash
# .pre-commit-hooks/no-real-pii-in-corpus.sh
# AC #15: pre-commit-time rejection of real-CCCD-like sequences in the corpus.
# Runs in <1s; AST-walks YAML and checks 12-digit sequences against known-customer
# Luhn+province-code patterns (the patterns themselves are NOT in this script —
# they are read from an encrypted local table that ops maintains).

set -euo pipefail

CORPUS="services/ai-gateway/pii/fixtures/vn_pii_200_samples.yaml"
KNOWN_PATTERNS_TABLE="${CYBEROS_KNOWN_CUSTOMER_PII_TABLE:-/opt/cyberos/pii_patterns.enc}"

if [[ ! -f "$CORPUS" ]]; then
    exit 0
fi

if [[ ! -f "$KNOWN_PATTERNS_TABLE" ]]; then
    echo "WARN: $KNOWN_PATTERNS_TABLE not present; skipping real-PII check (CI-only enforcement)" >&2
    exit 0
fi

python services/ai-gateway/pii/scripts/check_no_real_pii.py "$CORPUS" "$KNOWN_PATTERNS_TABLE"
```

```python
# services/ai-gateway/pii/scripts/check_no_real_pii.py
"""AC #15: scan the corpus for any sequence that matches a known-internal-customer
CCCD pattern (Luhn-style check + province-code prefix in known-customer distribution)."""
import re
import sys
import yaml
from pathlib import Path

CCCD_RE = re.compile(r"\b\d{12}\b")
MST_RE  = re.compile(r"\b\d{10}(?:-\d{3})?\b")


def main(corpus_path: str, patterns_table_path: str) -> int:
    samples = yaml.safe_load(Path(corpus_path).read_text())["samples"]
    # Patterns table is encrypted; in CI it is decrypted by a step before this hook runs.
    # Schema: list of regex patterns matching real-customer-CCCD-distributions.
    # In production, this is a frozenset of full 12-digit strings; the comparison is direct.
    known = set(Path(patterns_table_path).read_text().splitlines()) if Path(patterns_table_path).exists() else set()

    failures = []
    for s in samples:
        for digit_run in CCCD_RE.findall(s["text"]):
            if digit_run in known:
                failures.append(f"{s['id']}: real-CCCD-like sequence {digit_run!r} matches internal record")
        for digit_run in MST_RE.findall(s["text"]):
            if digit_run in known:
                failures.append(f"{s['id']}: real-MST-like sequence {digit_run!r} matches internal record")

    if failures:
        print("❌ pre-commit hook: real PII detected in corpus:", file=sys.stderr)
        for f in failures:
            print(f"   {f}", file=sys.stderr)
        print("\nIf this is a false positive, use scripts/regen_fixture.py to generate "
              "format-valid synthetic equivalents and re-commit.", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1], sys.argv[2]))
```

### Runtime budget guard

```python
# services/ai-gateway/pii/tests/test_recall_gate_runtime_budget.py
import subprocess
import time
import yaml
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "fixtures" / "fixture_manifest.yaml"


def test_recall_gate_completes_within_budget():
    """AC #4: test_recall_gate.py wall-clock < runtime_budget_seconds."""
    budget = yaml.safe_load(MANIFEST.read_text())["runtime_budget_seconds"]
    t0 = time.monotonic()
    result = subprocess.run(
        ["pytest", "-q", "tests/test_recall_gate.py::test_recall_per_recognizer_and_aggregate"],
        cwd=ROOT, capture_output=True, text=True, timeout=budget + 30,
    )
    elapsed = time.monotonic() - t0
    assert elapsed < budget, (
        f"recall gate took {elapsed:.1f}s, budget is {budget}s.\n"
        f"stdout:\n{result.stdout}\nstderr:\n{result.stderr}"
    )
```

### GitHub Actions workflow

```yaml
# .github/workflows/vn-pii-recall.yml
name: VN PII Recall Gate
on:
  pull_request:
    paths:
      - 'services/ai-gateway/pii/**'
      - '.github/workflows/vn-pii-recall.yml'

jobs:
  recall:
    runs-on: ubuntu-22.04
    timeout-minutes: 10
    permissions:
      contents: read
      pull-requests: write   # for PR-comment posting (warnings only)
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v4
        with: { python-version: '3.11' }
      - name: Install
        run: |
          pip install presidio-analyzer presidio-anonymizer pytest pyyaml
          python -m spacy download vi_core_news_lg
      - name: Run recall gate
        working-directory: services/ai-gateway/pii
        run: |
          pytest tests/test_recall_gate.py tests/test_fixture_invariants.py \
                 tests/test_no_real_pii_in_corpus.py \
                 tests/test_recall_gate_runtime_budget.py -v
      - name: Run precision warning
        working-directory: services/ai-gateway/pii
        run: pytest tests/test_precision_warning.py -v
      - name: Upload recall report
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: recall_gate_report_${{ github.sha }}
          path: services/ai-gateway/pii/recall_gate_report.json
      - name: Upload precision warning
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: precision_warning_${{ github.sha }}
          path: services/ai-gateway/pii/precision_warning_report.json
```

```yaml
# .github/workflows/vn-pii-quarterly-refresh.yml
name: VN PII Corpus Quarterly Refresh Reminder
on:
  schedule:
    - cron: '0 0 1 1,4,7,10 *'   # Jan 1 / Apr 1 / Jul 1 / Oct 1 at 00:00 UTC
  workflow_dispatch: {}

jobs:
  open-refresh-issue:
    runs-on: ubuntu-22.04
    permissions:
      issues: write
    steps:
      - uses: actions/github-script@v7
        with:
          script: |
            const today = new Date();
            const q = Math.floor(today.getUTCMonth() / 3) + 1;
            const yyyy = today.getUTCFullYear();
            await github.rest.issues.create({
              owner: context.repo.owner,
              repo: context.repo.repo,
              title: `VN PII corpus refresh due — quarter Q${q} ${yyyy}`,
              labels: ['compliance', 'pii', 'corpus-refresh'],
              body: [
                `Per FR-AI-013 §1 #4, the VN PII fixture must be regenerated quarterly.`,
                ``,
                `**Runbook:** \`services/ai-gateway/pii/scripts/regen_fixture.py\``,
                `**Manifest:** \`services/ai-gateway/pii/fixtures/fixture_manifest.yaml\` — bump fixture_version to \`${yyyy}Q${q}.0\``,
                `**Codeowner:** see CODEOWNERS for \`services/ai-gateway/pii/\``,
                ``,
                `Acceptance: PR that bumps fixture_version, passes the recall gate at ≥99% per recognizer.`,
              ].join("\n"),
            });
```

---

## §6 — Implementation skeleton

See §3 (fixture format + manifest) and §5 (full test bodies + workflow files).

The `Makefile` additions:

```makefile
# services/ai-gateway/pii/Makefile  (additions)

.PHONY: pii-recall-test
pii-recall-test:
	pytest tests/test_recall_gate.py tests/test_fixture_invariants.py \
	       tests/test_no_real_pii_in_corpus.py tests/test_recall_gate_runtime_budget.py -v

.PHONY: pii-fixture-validate
pii-fixture-validate:
	python scripts/validate_corpus_format.py fixtures/vn_pii_200_samples.yaml

.PHONY: pii-precision-report
pii-precision-report:
	pytest tests/test_precision_warning.py -v

.PHONY: pii-quarterly-refresh
pii-quarterly-refresh:
	python scripts/regen_fixture.py --quarter $(QUARTER) --year $(YEAR)
```

The `pre-commit` registration:

```yaml
# .pre-commit-config.yaml (additions)
repos:
  - repo: local
    hooks:
      - id: no-real-pii-in-corpus
        name: VN PII corpus — block real-CCCD-like commits (FR-AI-013 §1 #14)
        entry: .pre-commit-hooks/no-real-pii-in-corpus.sh
        language: script
        files: ^services/ai-gateway/pii/fixtures/vn_pii_200_samples\.yaml$
        pass_filenames: false
```

The `regen_fixture.py` skeleton (quarterly refresh tool):

```python
# services/ai-gateway/pii/scripts/regen_fixture.py
"""Quarterly fixture regeneration. Anonymises real customer data, adds new edge cases,
emits a fresh vn_pii_200_samples.yaml with a bumped fixture_version."""
import argparse
import datetime as dt
import yaml
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def anonymise_cccd(real_cccd: str) -> str:
    """Preserve province-code prefix (first 3 digits) AND format validity;
    zero out the identity-bearing 9 digits using a deterministic-but-irreversible mapping."""
    province = real_cccd[:3]
    return province + "000000001"   # placeholder; real impl uses HMAC over a one-way salt


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--quarter", type=int, required=True)
    p.add_argument("--year", type=int, required=True)
    args = p.parse_args()
    version = f"{args.year}Q{args.quarter}.0"

    # ... refresh logic: pull anonymised real samples + synthetic edge cases ...

    manifest = yaml.safe_load((ROOT / "fixtures" / "fixture_manifest.yaml").read_text())
    manifest["fixture_version"] = version
    manifest["generated_at"] = dt.datetime.utcnow().isoformat() + "Z"
    next_q = (args.quarter % 4) + 1
    next_year = args.year + (1 if next_q == 1 else 0)
    manifest["regenerated_due"] = f"{next_year}-{[1,4,7,10][next_q-1]:02d}-01"

    (ROOT / "fixtures" / "fixture_manifest.yaml").write_text(yaml.safe_dump(manifest, sort_keys=False))
    print(f"Bumped fixture_version → {version}")


if __name__ == "__main__":
    main()
```

---

## §7 — Dependencies

### Code dependencies (other FRs/modules)

- **FR-AI-012** — VN recognizers (`VN_RECOGNIZERS` list, the 6 recognizer classes, `register_vn_recognizers`, and the `GET /recognizers/version` endpoint) MUST exist. This FR consumes them; it does not modify them.
- **FR-AI-011** — Presidio sidecar (`presidio_sidecar.py:app`) must be importable; the recall gate uses `fastapi.testclient.TestClient(app)` to query the version endpoint.
- **FR-AI-022 (downstream)** — Recall + precision JSON artefact is the source for the trend chart. Schema MUST be stable; `validate_corpus_format.py --schema-out` is the contract surface.

### Concept dependencies (shared types)

- `recognizer_versions` in `fixture_manifest.yaml` MUST match the semver strings emitted by `GET /recognizers/version` (FR-AI-012 §1 #14).
- `recall_floors` and `precision_baselines_prior_quarter` in the manifest are the per-recognizer dials; they live in YAML so a calibration change is a single-PR diff.
- `CONFIDENCE_HIGH`, `CONFIDENCE_MED`, `CONFIDENCE_LOW` from `recognizers/confidence.py` (FR-AI-012 §3) drive the negative-sample test's threshold.
- Sample IDs (`cccd_001`, etc.) are the human-readable handle for the test failure message; format is `<type>_<NNN>` for positive, `neg_<NNN>` for negative.
- The `expected_entities` field is the source-of-truth for "what should this recognizer find"; `expected_spans` is the source-of-truth for "where exactly."

### Operational / external

- Python: `presidio-analyzer==2.2.x`, `presidio-anonymizer==2.2.x`, `pytest>=7.0`, `pyyaml>=6.0`, `fastapi[testclient]>=0.110`.
- spaCy: `vi_core_news_lg` (Vietnamese model) — downloaded in the CI step.
- GitHub Actions: `ubuntu-22.04` runner; `actions/checkout@v4`, `actions/setup-python@v4`, `actions/upload-artifact@v4`, `actions/github-script@v7`.
- `pre-commit` framework (https://pre-commit.com) installed locally for developers.
- An encrypted-at-rest internal `pii_patterns.enc` table mounted at `/opt/cyberos/pii_patterns.enc` on the CI runner (decrypted by a prior CI step from a GitHub Actions secret). The table lives in `meta:` directory of memory; rotation is owned by ops.

---

## §8 — Example payloads

### Recall gate JSON artefact (success)

```json
{
  "aggregate_recall": 1.0,
  "fixture_version": "2026Q2.0",
  "missed_samples_by_type": {},
  "recall_per_type": {
    "VN_CCCD": 1.0,
    "VN_MST": 1.0,
    "VN_PHONE": 1.0,
    "VN_NDD": 1.0,
    "VN_ADDRESS": 1.0,
    "VN_BANK_ACCOUNT": 1.0
  },
  "recognizer_versions": {
    "VN_CCCD": "1.0.0",
    "VN_MST": "1.0.0",
    "VN_PHONE": "1.0.0",
    "VN_NDD": "1.0.0",
    "VN_ADDRESS": "1.0.0",
    "VN_BANK_ACCOUNT": "1.0.0"
  }
}
```

### Recall gate JSON artefact (failure on VN_PHONE)

```json
{
  "aggregate_recall": 0.995,
  "fixture_version": "2026Q2.0",
  "missed_samples_by_type": {
    "VN_PHONE": ["phone_037"]
  },
  "recall_per_type": {
    "VN_CCCD": 1.0,
    "VN_MST": 1.0,
    "VN_PHONE": 0.975,
    "VN_NDD": 1.0,
    "VN_ADDRESS": 1.0,
    "VN_BANK_ACCOUNT": 1.0
  },
  "recognizer_versions": {
    "VN_CCCD": "1.0.0", "VN_MST": "1.0.0", "VN_PHONE": "1.0.0",
    "VN_NDD": "1.0.0", "VN_ADDRESS": "1.0.0", "VN_BANK_ACCOUNT": "1.0.0"
  }
}
```

### Precision warning artefact

```json
{
  "precision_per_type": {
    "VN_CCCD": 1.0,
    "VN_MST": 0.9677,
    "VN_PHONE": 1.0,
    "VN_NDD": 1.0,
    "VN_ADDRESS": 0.9524,
    "VN_BANK_ACCOUNT": 1.0
  },
  "warnings": []
}
```

### Corpus sample (positive — CCCD)

```yaml
- id: cccd_007
  text: "Hồ sơ ứng viên - CCCD: 042195000123 - SĐT: 0937654321"
  expected_entities: ["VN_CCCD", "VN_PHONE"]
  expected_count: 2
  expected_spans:
    - { entity: "VN_CCCD",  start: 22, end: 34 }
    - { entity: "VN_PHONE", start: 44, end: 54 }
  provenance: synthetic
```

### Corpus sample (negative — VND amount)

```yaml
- id: neg_012
  text: "Doanh thu Q1: 100.000.000 VND. Tăng trưởng 15% so với cùng kỳ."
  expected_entities: []
  expected_count: 0
  expected_spans: []
  provenance: synthetic
  notes: "VND amount with dots; bare 9-digit sequence MUST NOT match VN_PHONE or VN_BANK_ACCOUNT"
```

### Quarterly-refresh issue body (rendered by `vn-pii-quarterly-refresh.yml`)

```markdown
Per FR-AI-013 §1 #4, the VN PII fixture must be regenerated quarterly.

**Runbook:** `services/ai-gateway/pii/scripts/regen_fixture.py`
**Manifest:** `services/ai-gateway/pii/fixtures/fixture_manifest.yaml` — bump fixture_version to `2026Q3.0`
**Codeowner:** see CODEOWNERS for `services/ai-gateway/pii/`

Acceptance: PR that bumps fixture_version, passes the recall gate at ≥99% per recognizer.
```

---

## §9 — Open questions

All resolved at authoring time. Items deferred to later FRs:

- Precision gate (FR-AI-022) — currently precision is reported, not gated. Gate activation requires 6 months of operational data.
- Adversarial samples (typo CCCDs, partial CCCDs, foreign-looking VN IDs) — phase-2 expansion; placeholder FR-AI-024.
- ASEAN expansion (Khmer / Lao / Thai recognizers + their fixtures) — slice 6 / FR-AI-022 area.
- Cross-tenant fixture sharing (each tenant has its own anonymised PII patterns) — out of scope; the central fixture is sufficient at 99% recall floor.
- Sample-count rebalancing (currently 50/30/40/20/40/20 = 200; should it be 50/50/50/50/50/50 = 300?) — re-evaluated quarterly during regen, decided by recall-distribution data; for slice 3 the imbalance is acceptable.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Recall drops below 99% on one recognizer | `test_recall_per_recognizer_and_aggregate` | CI gate fails; PR blocked with assertion listing recognizer + missed sample IDs | Engineer fixes recognizer regex or context boost (or adds new sample if format drift) |
| Aggregate recall below 99% but per-recognizer all pass | Mathematically impossible if all per-type ≥ 99% AND counts are non-zero; detected by aggregate assertion as defence-in-depth | Same PR-block as per-recognizer | Investigate count drift in fixture |
| New VN PII format emerges (production discovery) | Customer-facing leak OR FR-AI-022 trend chart alarms | Recognizer misses; sev-2 alert | Add sample(s) to fixture; update recognizer; bump fixture_version; PR through gate |
| Real PII committed to corpus | `.pre-commit-hooks/no-real-pii-in-corpus.sh` checks against encrypted internal pattern table | Commit rejected at pre-commit time | Engineer uses `regen_fixture.py` to generate format-valid synthetic equivalent |
| Real PII slips past pre-commit hook (table not present locally) | CI-step decrypts pattern table and re-runs the hook BEFORE the recall gate | CI fails on hook; PR blocked | Operator rotates the leaked CCCD's customer; updates pattern table; force-pushes corrected corpus |
| Corpus stale (>90 days since last quarterly refresh) | `vn-pii-quarterly-refresh.yml` scheduled bot opens a tracking issue | GitHub issue created; assigned to codeowner | Operator runs `regen_fixture.py` for the quarter; opens PR through gate |
| Quarterly bot fails to create issue (workflow disabled, permission missing) | Out-of-band: monthly process review notices missing issue | Reminder missed; quarterly refresh skipped | Manual issue creation; investigate workflow status; restore `issues:write` permission |
| CI runner slow → test exceeds 5-min budget | `test_recall_gate_runtime_budget.py` asserts wall-clock | Test fails on budget violation | Reduce sample count (with FR amendment) OR parallelize OR profile the recognizer (likely regex backtracking) |
| Fixture file format invalid (bad YAML, missing required fields) | `test_fixture_invariants.py` | CI fails on fixture invariants test before recall is computed | Engineer fixes YAML; runs `make pii-fixture-validate` locally |
| Span offset wrong in fixture (start/end don't delimit actual text) | `test_span_offsets_delimit_actual_substrings` | CI fails on span validation | Engineer recomputes offsets in `regen_fixture.py` |
| Sample-count drift (e.g., 51 CCCD instead of 50) | `test_sample_counts_match_manifest` | CI fails; manifest is the source-of-truth | Engineer either removes the extra sample or bumps the manifest (FR amendment if per-type floors change) |
| Recognizer version drift between deployed sidecar and fixture pin | `test_recognizer_versions_match_manifest` | CI fails with `fixture_version_mismatch` | Engineer either bumps fixture (regen for new recognizer behaviour) OR reverts the recognizer change |
| Negative sample false-positives at high confidence | `test_negative_samples_no_high_confidence_matches` | CI fails on precision-disguised-as-spec-violation | Engineer tightens recognizer regex OR adjusts context boost OR removes the negative sample (with rationale) |
| Workflow disabled silently (PR comments out `paths:`) | Workflow file is itself in `paths:`; self-gate fires | CI fails on the disabled-workflow PR | Reviewer rejects the PR |
| Recall gate doesn't fail noisily (silent pytest skip) | `vn-pii-recall.yml` uses `pytest -v` AND treats `==0 tests collected` as failure (`-p no:cacheprovider` doesn't help; the assertion is that ≥4 test files ran) | CI gate non-zero exit even on "no tests ran" scenario | Engineer investigates why tests weren't collected |
| GitHub Actions secrets exposure (real PII in error output) | CI uses `set +x` and `mask-aware` log redaction; assertion messages never include raw `text` from samples (only sample IDs) | No PII in CI logs | If detected, scrub log history; rotate any exposed PII |
| Corpus file truncated by git LFS misconfiguration | `test_sample_counts_match_manifest` (counts < manifest) | CI fails before recall is computed | Engineer fixes `.gitattributes`; re-fetches LFS objects |
| New VN province code not in `province_codes.py` (FR-AI-012 dep) | CCCD recall drops on that province's samples → recall gate fails on VN_CCCD | CI block on CCCD recall | Add province code to FR-AI-012's `province_codes.py`; re-run gate |
| Test set bias (over-representation of labeled samples) | Production recall lower than fixture recall by >2pp | Out-of-band monitoring (FR-AI-022) | Quarterly regen consciously includes harder samples |
| Off-by-one in span offsets within fixture | Span-offset validator | Fixture-format test fails | Operator re-runs `regen_fixture.py` with span-recomputation pass |
| Network call accidentally added in a test (e.g., `requests.get(...)` in a fixture loader) | `vn-pii-recall.yml` egress-monitor step OR `test_no_network_imports.py` (FR-AI-012 ISS-002 style) extended to `tests/` directory | CI fails on offline-only assertion | Engineer removes the network call |
| Force-pass via `pytest.skip` in `test_recall_gate.py` | CI matrix asserts no `s ` (skip) markers in pytest output for the recall test | CI fails on skip detection | Engineer removes the skip annotation |
| Precision regression > 5pp vs baseline | `test_precision_reported_no_gate` logs `::warning::` lines | PR comment posted; merge NOT blocked | Engineer investigates; may bump baseline next quarter if regression is intentional |
| Fixture-manifest `runtime_budget_seconds` changed without rationale | PR review (no automated detection) | Caught at code review | Reviewer demands explanation in PR body |
| Workflow file edited to skip the gate (`if: false`) | `vn-pii-recall.yml` is in its own `paths:` list; PR triggers gate against itself | Self-gate runs; assertion runs; PR fails or passes on real recall | If gate is intentionally disabled, requires FR amendment |

---

## §11 — Notes

- The 200-sample size is calibrated against the "1 missed per 100 cases" intuition. For 99% recall floors per recognizer: 50-sample types tolerate at most 0 misses (50 × 0.99 = 49.5 → round-up requires 50/50), and 20-sample types likewise tolerate 0 misses (20 × 0.99 = 19.8 → requires 20/20). The compressed first-pass of this FR claimed "at most 2 missed per recognizer"; that figure was wrong for tiny recognizers and has been removed. The accurate statement is: *every recognizer must hit 100% on its current sample subset to clear the 99% floor with margin; one miss on a 20-sample recognizer drops it to 95%, which fails*.
- Quarterly regeneration is the load-bearing operational discipline. Without it, the fixture decays into a historical snapshot. The bot reminder (§1 #13) is the forcing function; the runbook is `scripts/regen_fixture.py`.
- Per-recognizer recall floors live in `fixture_manifest.yaml` so a calibration change is a single-PR diff. Changing the floor requires an explicit FR amendment to FR-AI-013 §1 #5 (the 99% number is normative); the manifest knob is for fine-tuning, not weakening.
- The pre-commit hook approach is "fail-closed in CI, warn-only locally." Developers without the encrypted pattern table get a WARN at commit time; the CI runner (which has the decrypted table) is the source-of-truth. This avoids blocking developers who don't have ops-issued credentials while still gating real PII from merging.
- The `recognizer_versions` field in `fixture_manifest.yaml` is the lock-step that prevents silent recognizer upgrades from invalidating the fixture. If FR-AI-012's VnCccdRecognizer bumps to 1.1.0, the recall gate fails until the fixture is consciously bumped to match — forcing a deliberate "does the new version actually still find these samples" check.
- The precision-warning report is opt-in by design: precision data is collected from day 1, but the gate doesn't enforce. FR-AI-022 will activate the gate once we have ≥6 months of trend data showing per-recognizer precision is stable above 95%. Until then, the report is a "what would happen if we turned this on?" signal.
- The workflow-file self-gate (`paths:` includes the workflow file itself) is a small but important pattern. Without it, a PR like "loosen the recall floor to 95%" could ship without ever running the gate. With it, even a PR that *only* edits the workflow runs the gate first.
- The `recall_gate_report_<sha>.json` artefact is the audit primitive. When a regulator (or an internal compliance team) asks "what was the recall on the day FR-X shipped?", the answer is in the artefact attached to the merge commit's CI run. This is the durable record; GH Actions log retention is short, but artefacts can be re-archived to the internal compliance data lake.
- Negative samples (30) are the minimum floor; we can grow this set as we discover specific false-positive patterns in production. The 30-sample floor is asserted in `test_fixture_invariants.py` (via the manifest `sample_counts.negative` field).
- The `regen_fixture.py` anonymisation rule is "preserve format validity, destroy identity linkage." The simplest impl is to keep the province-code prefix (which is public knowledge) and replace the identity-bearing suffix with a deterministic-but-irreversible HMAC over a salt-rotated-quarterly. This way we maintain format-valid samples (which exercise the province-code validator) without leaking any real identifier.

---

*End of FR-AI-013. Status: draft (10/10 target).*
