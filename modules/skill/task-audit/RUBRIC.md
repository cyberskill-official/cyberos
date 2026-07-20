# `audit_rubric@2.0` — machine-checkable Task rubric

> Sourced from `cyberos/skill/contracts/task/CONTRACT.md` (the task contract body) and `../../../modules/cuo/docs/module.md` §2(b) Requirements. Rubric version `2.0` is locked; bumping requires a coordinated update of the contract body and this skill's CONTRACT_ECHO. Each rule has a stable `rule_id`. Rule IDs MUST appear verbatim in audit reports so reports are diffable across iterations and operators.

This rubric is a port of the proven rule set from the legacy `cuo/cpo/task-audit` skill (audit_rubric@2.0, locked since 2026-02). It is preserved here verbatim because it has been battle-tested against 50+ tasks in the cyberos `docs/tasks/` catalog. Bumping to 3.0 requires governance sign-off.

---

## §1  Frontmatter — structural

| rule_id | Check | Severity | Auto-fixable |
| ------- | ----- | -------- | ------------ |
| `FM-001` | File begins with `---` on line 1; closing `---` exists; YAML between fences parses | error | false |
| `FM-002` | All keys are `snake_case` (lowercase ASCII letters, digits, underscores; no leading digit) | error | true |
| `FM-003` | No duplicate keys | error | false |
| `FM-004` | `template` key present and equals `task@1` | error | true |

## §2  Frontmatter — per-field

| rule_id | Field | Rule | Severity | Auto-fixable |
| ------- | ----- | ---- | -------- | ------------ |
| `FM-101` | `title` | required, string, length 1–72 chars after trimming | error | skeleton |
| `FM-102` | `author` | required, matches `^@[A-Za-z0-9_.-]{1,38}$` | error | false |
| `FM-103` | `department` | required, one of: engineering, design, product, sales, operations, hr, client_success | error | false |
| `FM-104` | `status` | required, one of: `draft`, `ready_to_implement`, `implementing`, `ready_to_review`, `reviewing`, `ready_to_test`, `testing`, `done`, `on_hold`, `closed` (per `modules/skill/contracts/task/STATUS-REFERENCE.md` §1). Freeform tags like `[BLOCKED: ...]` / `[FAILED: ...]` are NO LONGER valid status values — failures route back to `ready_to_implement` (STATUS-REFERENCE §1.3). | error | false |
| `FM-105` | `priority` | required, one of: p0, p1, p2, p3 | error | false |
| `FM-106` | `created_at` | required, ISO 8601 with timezone | error | true |
| `FM-107` | `ai_authorship` | required, one of: none, assisted, co_authored, generated_then_reviewed | error | false |
| `FM-108` | `type` | required, one of: `feature`, `bug`, `improvement`, `chore`. Replaces the retired `feature_type` and `class` fields (decision 2026-07-14 — three overlapping axes collapsed to one). Selects the body template and the per-type rule family: `feature` → §9, `bug` → §10. | error | false |
| `FM-109` | `eu_ai_act_risk_class` | required, one of: not_ai, minimal, limited, high. `unacceptable` MUST be rejected (per Article 5) | error | false |
| `FM-110` | `target_release` | optional; if present, SemVer `^\d+\.\d+\.\d+(-[A-Za-z0-9.-]+)?$` OR quarter `^\d{4}-Q[1-4]$` | error | false |
| `FM-111` | `client_visible` | required, boolean (YAML true/false, not strings, not yes/no) | error | true |
| `FM-113` | `duplicate_of` | required **iff** `status: duplicate`; must be `TASK-<MODULE>-<NNN>` and must resolve to a task that exists. Forbidden otherwise. `duplicate` exists as a status separate from `closed` precisely *because* it carries this link — without it the two are the same terminal state and the link is the entire value: it is how you find out that six reports were one cause. A `duplicate` with a dangling or absent `duplicate_of` is worse than `closed`, because it claims a relationship that cannot be followed. | error | false |
| `FM-114` | `severity` | required **iff** `type: bug` (see `contracts/task/rubrics/bug.md` BUG-010); forbidden otherwise. Severity is how bad it is if left alone; `priority` is when we will get to it. A sev1 you have consciously deferred is a legitimate and legible state — collapsing the two axes hides that decision. | error | false |
| `FM-112` | *(any)* | **No `# UNREVIEWED` marker may survive `draft`.** The 2026-07-14 schema migration backfilled `ai_authorship` and `eu_ai_act_risk_class` onto 498 legacy specs. Those two values cannot be derived from anything on disk — one records who wrote the spec and how much of it a model wrote, the other is a regulatory classification. Auto-filling them would be *fabricating compliance metadata*, so the migration wrote a plausible value plus an explicit `# UNREVIEWED` marker. A human MUST confirm both before the task leaves `draft`. | error | false |

**FM-105 note.** `priority` is `p0..p3`. The legacy MoSCoW values (`MUST`/`SHOULD`/`COULD`) were mapped by the 2026-07-14 migration (`MUST`→`p0`, `SHOULD`→`p1`, `COULD`→`p2`). `ship_manifest._PRIORITY_RANK` and `status.css` still accept both, so a downstream repo that has not run the migration keeps sorting and rendering correctly rather than silently ranking everything last.

**Retired: `FM-108 feature_type`.** Its four values (`user_facing`, `internal_tooling`, `integration`, `infrastructure`) overlapped `class` (`product`/`improvement`) and the new `type`. Only 7 of 507 specs ever carried it. Three enums describing the same thing is how you get a taxonomy nobody fills in correctly.

## §3  Always-required sections

| rule_id | Heading | Severity |
| ------- | ------- | -------- |
| `SEC-001` | `## Summary` | error |
| `SEC-002` | `## Problem` | error |
| `SEC-003` | `## Proposed Solution` | error |
| `SEC-004` | `## Alternatives Considered` | error |
| `SEC-005` | `## Success Metrics` | error |
| `SEC-006` | `## Scope` | error |
| `SEC-007` | `## Dependencies` | error |
| `SEC-008` | Each required H2 is non-empty (≥1 non-blank line of body) | error |
| `SEC-009` | Heading hierarchy well-formed (no H2→H4 jumps; one or zero H1s) | warning |

## §4  Conditionally-required sections

| rule_id | Trigger | Required | Severity |
| ------- | ------- | -------- | -------- |
| `COND-001` | `client_visible: true` | `## Customer Quotes` with ≥1 quote in `<untrusted_content>`, attribution outside | error |
| `COND-002` | `client_visible: true` | `## Sales/CS Summary` in plain English (no jargon — see QA-009) | error |
| `COND-003` | `eu_ai_act_risk_class ∈ {limited, high}` | `## AI Risk Assessment` with H3s `### Data Sources`, `### Human Oversight`, `### Failure Modes` in that order | error |
| `COND-004` | `ai_authorship != none` | `## AI Authorship Disclosure` with three bullets labeled `Tools used:`, `Scope:`, `Human review:` | error |

## §5  Quality heuristics (anti-patterns)

| rule_id | Anti-pattern | Detection | Severity |
| ------- | ------------ | --------- | -------- |
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

## §6  Untrusted-content safety

| rule_id | Check | Severity |
| ------- | ----- | -------- |
| `SAFE-001` | `<untrusted_content>` blocks not nested | error |
| `SAFE-002` | No unclosed `<untrusted_content>` block at EOF | error |
| `SAFE-003` | Interior of `<untrusted_content>` scanned for prompt-injection markers (case-insensitive, NFC-normalised, zero-width stripped, confusables folded). Markers: `ignore previous`, `ignore all prior`, `disregard the above`, `system prompt`, `you are now`, `developer mode`, `DAN`, `jailbreak`, `<\|im_start\|>`, `<\|im_end\|>`, `[INST]`, `</s>`, `assistant:` at line start, `BEGIN SYSTEM`, `print your instructions`, `reveal your`, base64 blobs ≥80 chars with no surrounding prose | warning (error if ≥3 matches) |
| `SAFE-004` | Quote outside `<untrusted_content>` contains second-person commands targeting auditor (`do this`, `output X`) | warning |

## §7  Cross-skill rules (when chained from `task-author`)

| rule_id | Check | Severity |
| ------- | ----- | -------- |
| `XCHAIN-001` | The task's `provenance.source_path` matches the author's manifest's `source_files[].path` | warning |
| `XCHAIN-002` | The task's `provenance.source_hash` matches the author's manifest's `source_files[].hash` at write time | error |
| `XCHAIN-003` | If the task was created via task-with-subtasks chain, the linked impl-plan path resolves | warning |

## §8  Staleness

| rule_id | Trigger | Action | Severity |
| ------- | ------- | ------ | -------- |
| `STALE-001` | Source artefact hash differs from `provenance.source_hash` | Reset open + needs_human issues to open; re-evaluate. Surface diff to operator. | warning → needs_human (`stale_artefact_disposition`) |

## §9  Spec-vs-implementation traceability  *(applies to cyberos-style §1/§4/§5 task template only)*

These rules apply to tasks that use the cyberos template (numbered §1 normative clauses · §4 acceptance criteria · §5 verification/tests), per `task-audit` skill §1. Added 2026-05-18 (session 21) after the audit-fix loop on TASK-AUTH-001 + TASK-AUTH-006 surfaced 13 §1↔§4 / §4↔§5 traceability gaps in two "shipped" tasks — see memory feedback `feedback_task_author_clause_to_test_traceability.md`. The upstream fix: refuse to score 10/10 if any §1 clause lacks a downstream test, so future tasks can't ship code that passes §5 tests while missing §1 clauses.

| rule_id | Check | Severity | Auto-fixable |
| ------- | ----- | -------- | ------------ |
| `TRACE-001` | Every §1 numbered clause with a BCP-14 keyword (MUST · MUST NOT · SHOULD · SHOULD NOT · MAY) is cited by at least one §4 AC. Citation form: `§1 #N` or `§1.N` inside the AC's rationale or in the AC's `traces_to:` frontmatter field. Clauses explicitly tagged `(deferred to slice N)` in §1 are exempt. | error → needs_human (`spec_clause_without_ac`) | skeleton (insert AC stub with TODO marker linked to §1 #N) |
| `TRACE-002` | Every §4 AC cites at least one §5 verification entry — typically a named test function (e.g. `services/<crate>/tests/<file>.rs::<test_fn>`) OR a manual verification step with a rationale (manual is acceptable only for ops/UI flows that can't be automated, and must justify why). | error → needs_human (`ac_without_test`) | skeleton (insert §5 test-name placeholder) |
| `TRACE-003` | Every §5 test path is either listed in `frontmatter.new_files` (test file will be authored as part of this task's implementation) OR resolves to an existing file on disk. Dangling test references (test name with no file) → fail. | error | false |
| `TRACE-004` | If `status: done`, every §1 clause's cited test is `passed` in the most recent `implementation_audit.coverage_report` (§10.3 audit-fix log). Tests in `implementing`/`ready_to_review`/`reviewing`/`ready_to_test`/`testing`/`draft`/`ready_to_implement` tasks are exempt (coverage is enforced separately by `coverage-gate-audit` during the `testing → done` transition). | error → needs_human (`done_with_untested_clause`) | false |
| `TRACE-005` | When a task uses the deferred-slice pattern (e.g. "§1 #8 — deferred to slice 2"), §10.7 of the `.audit.md` MUST enumerate the deferred clauses with a scope estimate per the TASK-AUTH-006 slice-2 precedent. Missing §10.7 with deferred clauses → fail. | warning | false |
| `TRACE-006` | For every §1 clause that cites a test, the audit MUST name the clause's VERB (the observable it demands, per the verb→evidence table in "TRACE-006 — the cited test must exercise its clause's verb" below), name what the cited test actually ASSERTS, and fail when the assertion is weaker than the verb. A cited test that PASSES (TRACE-004) but exercises something weaker than its clause is not evidence the clause holds. Judgment family — a model-audit rule, ABSENT from `task-lint`; fires at the spec-correctness gate, never the coverage gate. | error → needs_human (`clause_verb_untested`) | false |

**Rationale:** the audit-fix loop on TASK-AUTH-001 surfaced 7 spec-vs-code gaps where §1 MUST clauses had no §4 AC or no §5 test backing them — the implementer passed all declared tests while quietly missing 7 normative clauses. TRACE-001..005 close that gap structurally: a task can't score 10/10 (and thus can't move from `draft` → `ready_to_implement` → ... → `done`) if any of its §1 clauses lacks a downstream test. **The audit becomes the source of truth for "what's actually shipped" instead of `BACKLOG.md` status alone.**

**Phase ownership.** This skill (`task-audit`) is the **spec correctness gate** — it drives the `draft → ready_to_implement` transition by verifying frontmatter, structure, traceability (TRACE-001..005), and quality heuristics on the spec itself. It does NOT enforce test coverage; that is the job of `coverage-gate-audit` during the `testing → done` transition. The two gates are deliberately separated so spec correctness can be verified before any implementation work begins (cheap early failure), and coverage can be verified independently once tests have run (expensive late failure).

**Worked example** (TASK-AUTH-001's §1 #14 — `slug == "root"` defence-in-depth reject):
- §1 #14 says: `MUST NOT create a tenant with slug "root"`
- §4 AC #11 says: `Reserved-slug validator returns 400 with structured body before DB transaction (traces_to: §1 #14)`
- §5 test entry says: `services/auth/tests/admin_tenant_create_test.rs::create_tenant_rejects_reserved_root_slug` (covers ECM-008)
- §5 test file is in `frontmatter.new_files: [services/auth/tests/admin_tenant_create_test.rs, ...]`
- Pre-G-001 the file didn't exist on disk → TRACE-003 would have failed
- Post-G-001 the file + test exist → TRACE-003 + TRACE-001 + TRACE-002 all pass

### TRACE-006 — the cited test must exercise its clause's verb (judgment)

Added 2026-07-18 (TASK-IMP-118) after external review found TASK-IMP-108 §1.7 shipped `done` through both human gates with its clause unsatisfied and its cited test green. TRACE-004 checks that a cited test PASSES; nothing checked that it tests the CLAUSE. An author who writes both the clause and its test can satisfy every existing rule while asserting something strictly weaker than the clause promised — and the weaker test is the one most likely to be written, because it is the one that passes first. TRACE-006 points a check at that gap: for every §1 clause with a cited test, compare the test's ASSERTION to the clause's PROMISE.

**How the auditor runs it (per clause).** (1) Read the clause and name its VERB — what observable evidence the verb demands. (2) Read the cited test and name what it actually asserts. (3) Compare; if the assertion is weaker than the verb demands, TRACE-006 fails and the task routes back. A clause carrying two verbs ("MUST render AND MUST NOT change status") is compared against each verb separately, and either one weaker fails. A test asserting MORE than the verb demands is never a finding — stronger is fine. A clause with no BCP-14 verb, or citing no test, is out of TRACE-006's scope (that is TRACE-001/004 territory). Record BOTH halves — the verb's demand and the test's actual assertion — in the audit body per `REPORT_FORMAT.md`, for every clause compared (PASS or FAIL), so the comparison is legible to the next reader and not just its verdict.

**Verb → evidence table.** What discharges each recurring verb, and what does NOT:

| Clause verb | Discharged by a test that asserts… | Does NOT discharge it |
| ----------- | ---------------------------------- | --------------------- |
| **render** | the value is present in the RENDERED view a reader sees — the DOM/text the code path under test produces, read after the render (data island stripped) | the value present only in a data payload / JSON blob no view reads; a substring `grep` of source or payload that no renderer consumes (TASK-IMP-108 §1.7) |
| **reject** | a negative outcome on the rejected input — a non-zero exit / error status / 4xx / raised error | a log line that says "rejected"; a happy-path assertion; the mere absence of a crash |
| **refuse** | the guarded action did NOT occur AND a refusal was signalled — asserts both the absent side effect and the error/exit | a message was logged, or the caller was told, with no check that the effect did not happen |
| **halt** | execution stopped at the guard — the process/loop returned before the guarded effect ran | a warning printed while execution continued past the guard |
| **emit** | the artefact was actually produced and is observable at its sink — the row is in the log, the event on the bus, the file on disk | the emitting function was called, or the intent was recorded, without reading the sink |
| **preserve** | the value is UNCHANGED across the operation — compares before to after (byte- or value-equal) | the value still exists afterward, or was copied, with no comparison to the original |

These six are the recurring cases, not a closed set: a clause verb not in the table is judged by the same standard — name what the verb demands as observable evidence, then check the test asserts that and nothing weaker.

**Worked anti-example — TASK-IMP-108 §1.7 (render vs present-in-payload).**
- The clause (108 §1.7, verbatim): "The status page MUST render a staleness report: drafts grouped by `draft_reason` with age. It MUST NOT change any task's status." Two verbs: **render** and **MUST NOT change**.
- The original cited test asserted `grep -q '"draft_staleness"'` against the built HTML — that the string `draft_staleness` appears somewhere in the page bytes.
- Why the assertion was weaker than the verb: the string appears only inside a JSON blob injected into the payload that no code reads — `status-app.js` has zero references to the key, so nothing renders it. `grep`-in-payload proves PRESENT-IN-PAYLOAD; **render** demands PRESENT-IN-RENDERED-VIEW. The test passes on an implementation that computes the report, ships it in the JSON, and never draws it — which is exactly what shipped `done`. Under TRACE-006 the original assertion FAILS the **render** row above. Its replacement discharges the verb: it strips the data island and asserts the report is visible markup outside the payload ("Drafts awaiting triage", a reason column, an age column) — the DOM a reader sees. (External review 2026-07-17; `tools/docs-site/tests/test_render_status_hub.sh::t11_draft_staleness_report` carries both the history and the fix.)

**TRACE-006 is judgment-family and is ABSENT from `task-lint`.** Like TRACE-004 and TRACE-005 — and unlike the structural halves TRACE-001..003, which the machine floor `task-lint.mjs` implements — TRACE-006 cannot be mechanised: deciding whether a test exercises a clause's verb means reading a test and a sentence and comparing their meaning. It therefore lives with the judgment families and MUST NOT be added to `task-lint` (TASK-IMP-118 §1.5). A structural check that appeared to enforce it — e.g. "the clause's key string appears in the cited test" — would PASS TASK-IMP-108 §1.7's original test (the string genuinely is in the file) and restore exactly the false assurance TRACE-006 exists to remove. The auditor runs it under "TRACE semantic sufficiency" (task-audit skill §3); the coverage gate does not — TRACE-004's mechanical pass/fail is a different job at a different gate. TRACE-006 fails a test that asserts less than its clause's VERB; a spec that cannot fail its own motivating case is decoration, so re-auditing 108 §1.7 against this rule MUST fail the original assertion and pass its replacement.

---

## Rule auto-fix behaviour catalogue

| auto-fixable value | Audit behaviour |
| ------------------ | --------------- |
| `true` | Minimal textual change; mark `fixed`. |
| `false` | Leave `open` or mark `needs_human` per severity. |
| `skeleton` | Insert TODO marker; mark `open` with `todo_inserted: true`. |

## Cross-references

- `cyberos/skill/docs/AUDIT_LOOP.md` — the 8-step algorithm.
- `cyberos/skill/docs/RUBRIC_FORMAT.md` — the rubric format.
- `REPORT_FORMAT.md` (sibling file) — `.audit.md` shape.
- `INVARIANTS.md` (sibling file) — invariant catalog including `deterministic_drift`.
- `cyberos/skill/contracts/task/CONTRACT.md` — the task template this rubric audits.
- `cyberos/skill/contracts/task/template.md` — the task body skeleton.

## §10  Template detection (TASK-CUO-208)

Family applicability is selected per file by detection - `template: task@1` key -> FM+SEC+COND+QA+SAFE (+TRACE per §9); `## §1 - Description`..`## §11` grammar -> engineering-spec@1 (author §12 sub-rules + TRACE+QA+SAFE). Both/neither -> needs_human (`template_ambiguous`). Profiles normative in `../task-author/references/TEMPLATE_PROFILES.md`.
