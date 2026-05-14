# `audit_rubric@2.0` — machine-checkable audit rubric

> Sourced verbatim from `feature-request/FR_CREATE_AND_AUDIT.md` v2.0.0 §15. Rubric version `2.0` is locked; bumping requires a coordinated update of `cyberos/docs/contracts/feature-request/` (the contract body) and `cuo/cpo/fr-audit/SKILL.md` CONTRACT_ECHO. Each rule has a stable `rule_id`. Rule IDs MUST appear verbatim in the audit report so reports are diffable across iterations and operators.

## §15.1 Frontmatter — structural

| rule_id | Check | Severity |
| --- | --- | --- |
| `FM-001` | File begins with `---` on line 1; closing `---` exists; YAML between fences parses | error |
| `FM-002` | All keys are `snake_case` (lowercase ASCII letters, digits, underscores; no leading digit) | error |
| `FM-003` | No duplicate keys | error |
| `FM-004` | `template` key present and equals `feature_request@1` | error |

## §15.2 Frontmatter — per-field

| rule_id | Field | Rule | Severity |
| --- | --- | --- | --- |
| `FM-101` | `title` | required, string, length 1–72 chars after trimming | error |
| `FM-102` | `author` | required, matches `^@[A-Za-z0-9_.-]{1,38}$` | error |
| `FM-103` | `department` | required, one of: engineering, design, product, sales, operations, hr, client_success | error |
| `FM-104` | `status` | required, one of: draft, in_review, approved, in_progress, shipped, closed | error |
| `FM-105` | `priority` | required, one of: p0, p1, p2, p3 | error |
| `FM-106` | `created_at` | required, ISO 8601 with timezone | error |
| `FM-107` | `ai_authorship` | required, one of: none, assisted, co_authored, generated_then_reviewed | error |
| `FM-108` | `feature_type` | required, one of: user_facing, internal_tooling, integration, infrastructure | error |
| `FM-109` | `eu_ai_act_risk_class` | required, one of: not_ai, minimal, limited, high. `unacceptable` MUST be rejected (per `references/EU_AI_ACT_DECISION_TREE.md` Article 5) | error |
| `FM-110` | `target_release` | optional; if present, SemVer `^\d+\.\d+\.\d+(-[A-Za-z0-9.-]+)?$` OR quarter `^\d{4}-Q[1-4]$` | error |
| `FM-111` | `client_visible` | required, boolean (YAML true/false, not strings, not yes/no) | error |

## §15.3 Always-required sections

| rule_id | Heading | Severity |
| --- | --- | --- |
| `SEC-001` | `## Summary` | error |
| `SEC-002` | `## Problem` | error |
| `SEC-003` | `## Proposed Solution` | error |
| `SEC-004` | `## Alternatives Considered` | error |
| `SEC-005` | `## Success Metrics` | error |
| `SEC-006` | `## Scope` | error |
| `SEC-007` | `## Dependencies` | error |
| `SEC-008` | Each required H2 is non-empty (≥1 non-blank line of body) | error |
| `SEC-009` | Heading hierarchy well-formed (no H2→H4 jumps; one or zero H1s) | warning |

## §15.4 Conditionally-required sections

| rule_id | Trigger | Required | Severity |
| --- | --- | --- | --- |
| `COND-001` | `client_visible: true` | `## Customer Quotes` with ≥1 quote in `<untrusted_content>`, attribution outside | error |
| `COND-002` | `client_visible: true` | `## Sales/CS Summary` in plain English (no jargon — see QA-009) | error |
| `COND-003` | `eu_ai_act_risk_class ∈ {limited, high}` | `## AI Risk Assessment` with H3s `### Data Sources`, `### Human Oversight`, `### Failure Modes` in that order | error |
| `COND-004` | `ai_authorship != none` | `## AI Authorship Disclosure` with three bullets labeled `Tools used:`, `Scope:`, `Human review:` | error |

## §15.5 Quality heuristics (anti-patterns)

| rule_id | Anti-pattern | Detection | Severity |
| --- | --- | --- | --- |
| `QA-001` | Dodged risk class | `eu_ai_act_risk_class ∈ {minimal, not_ai}` AND body contains AI-generation cues + `client_visible: true` OR `feature_type: user_facing` | error → needs_human (`ai_act_risk_boundary`) |
| `QA-002` | High-risk indicator without `high` | Body mentions Annex III domain (biometrics / hiring / credit / education grading / emergency triage / law enforcement / migration / critical infra) while class < high | error → needs_human |
| `QA-003` | Article 5 / prohibited practice | Body describes social scoring, untargeted face scraping, workplace/education emotion inference, real-time biometric ID for law enforcement, subliminal manipulation | error → needs_human (`legal_compliance`) |
| `QA-004` | Vanity metric | Metric without baseline + target + deadline; or only signups/views/followers without definition | warning |
| `QA-005` | Vague Alternatives | <2 distinct alternatives; or filler-only ("considered other options") | warning |
| `QA-006` | Vague scope boundaries | `## Scope` lacks `### Out of scope` / `### Non-Goals`, or contains only one bullet | warning |
| `QA-007` | Unsourced numeric target | Metric uses a target value not derivable from inputs | error → needs_human (`success_metric_targets`) |
| `QA-008` | Cross-team dependency claim | `## Dependencies` names another team/module without ticket/owner/commitment | warning → needs_human (`cross_team_dependency`) |
| `QA-009` | Engineering jargon in Sales/CS Summary | Words detected: API, endpoint, schema, webhook, latency, payload, RBAC, JWT, idempotent, migration, raw HTTP verbs, file paths, regex | warning |
| `QA-TODO` | Skeleton TODO marker remaining | Body contains literal `TODO:` from a §16.5 stub | warning (open until human resolves) |

## §15.6 Untrusted-content safety

| rule_id | Check | Severity |
| --- | --- | --- |
| `SAFE-001` | `<untrusted_content>` blocks not nested | error |
| `SAFE-002` | No unclosed `<untrusted_content>` block at EOF | error |
| `SAFE-003` | Interior of `<untrusted_content>` scanned for prompt-injection markers (case-insensitive, NFC-normalised, zero-width stripped, confusables folded). Markers: `ignore previous`, `ignore all prior`, `disregard the above`, `system prompt`, `you are now`, `developer mode`, `DAN`, `jailbreak`, `<\|im_start\|>`, `<\|im_end\|>`, `[INST]`, `</s>`, `assistant:` at line start, `BEGIN SYSTEM`, `print your instructions`, `reveal your`, base64 blobs ≥80 chars with no surrounding prose | warning (error if ≥3 matches) |
| `SAFE-004` | Quote outside `<untrusted_content>` contains second-person commands targeting auditor (`do this`, `output X`) | warning |

## §15.7 Cross-skill rules (when chained from fr-author)

| rule_id | Check | Severity |
| --- | --- | --- |
| `STALE-001` | FR's on-disk `fr_hash` differs from `fr-author`'s manifest `frs[FR].fr_hash` (only when `upstream_context.from_skill == cuo/cpo/fr-author`) | error → needs_human (`stale_fr_disposition`) |

When `fr-audit` runs standalone (no `upstream_context`), STALE-001 is skipped — there's no manifest to check against.

## §15.8 Severity → exit-code mapping

- `0` = pass (no error, no warning, audit terminal)
- `1` = errors present
- `2` = warnings only

`needs_human` always implies HITL_PAUSE regardless of count.

The exit code is exposed via the output envelope's `exit_code` field for CI pipelines that gate on FR conformance.

## §15.9 Confidence-band reporting

> Anchor: `confidence-band-reporting`. INV-006 references this section.

Every audit-report row carries a non-null `confidence` ∈ `[0.0, 1.0]`. Two rule classes — mechanical and LLM-judgement — report different bands:

### Mechanical-rule majority (`confidence ≥ 0.95`)

The vast majority of rubric rules are **mechanical** — they execute deterministic regex / structural / hash / enum checks against the FR. There is no LLM inference in the verdict. These rules MUST report `confidence ≥ 0.95`. The 5% headroom below 1.0 acknowledges the small-but-non-zero rate of false positives in regex / enum-membership checks (e.g. a `department:` value with an invisible leading whitespace that NFKC normalisation should have stripped but didn't on a legacy host).

Mechanical rule families:

- **`FM-001..111`** — frontmatter structural and per-field checks. All mechanical (YAML parse, snake_case regex, enum membership, ISO-8601 regex, SemVer regex, boolean type). `confidence: 0.99`.
- **`SEC-001..009`** — section-presence and heading-hierarchy checks. All mechanical (substring search for required H2 headings, depth count). `confidence: 0.99`.
- **`COND-001..004`** — conditional-section presence based on a frontmatter field. All mechanical (frontmatter field-read + section-presence check). `confidence: 0.97`.
- **`QA-001..006`, `QA-008`** — quality-checks expressible as deterministic patterns (e.g. `QA-001` Article 5 detection via the EU AI Act decision tree's verbatim trigger phrases). `confidence: 0.95`.
- **`SAFE-001..004`** — untrusted-content boundary checks. Mechanical (substring search for the markers from `references/UNTRUSTED_CONTENT.md`). `confidence: 0.99`.
- **`STALE-001`** — hash comparison. `confidence: 1.00` (pure SHA-256 equality).

### LLM-judgement minority (`confidence` reflects model's actual band)

A small set of rules require natural-language judgement and consult a local LLM through the auditor's own model surface (no network calls — the model lives in the same process / sidecar as the auditor):

- **`QA-007`** — fabrication detection. The rule asks: "does this FR contain any quote, name, date, or attribution that is not present in the source documents?" The model returns a yes/no plus its confidence. Audit-row `confidence` is the model's reported band. Cap: `0.7` (per `confidence_band.default` in SKILL.md frontmatter — LLM-inferred verdicts cannot claim higher).
- **`QA-009`** — plain-English readability check. The rule asks: "is this paragraph free of jargon undefined elsewhere in the document?" The model returns a list of jargon terms with confidence. Audit-row `confidence` is the model's reported band. Cap: `0.7`.

LLM-judgement rules MUST be tagged `judgement_class: llm` in the rubric metadata so consumers (downstream skills, the supervisor's decision-tree) can weight them differently from mechanical rules.

### Why the split matters

Mechanical rules are **reproducible** — INV-001 (verdict determinism) holds because the same input always yields the same regex / hash / enum result. LLM-judgement rules are **band-reproducible** — the same input yields a verdict within the same confidence band, but the exact float can drift between model versions. The `confidence` field surfaces this distinction so downstream consumers can choose to gate on mechanical-only verdicts when byte-identical reproducibility is required (e.g., legal-compliance checks) and consult LLM-judgement verdicts as advisory signals (e.g., readability suggestions).

A rule's authors decide the class at rubric-authoring time. The class is documented next to the rule's table row via the severity column where appropriate (e.g., "warning · llm-judgement"). When the next rubric MAJOR (`audit_rubric@3.0`) lands, this section will gain a per-rule confidence-band column rather than the family-level summary above.
