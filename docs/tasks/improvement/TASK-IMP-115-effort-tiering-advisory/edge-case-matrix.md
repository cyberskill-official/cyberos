---
artefact: edge-case-matrix@1
task_id: TASK-IMP-115
workflow: chief-technology-officer/ship-tasks
step: 5-6
categories_covered: [null_empty, bounds, malformed, concurrency, security, degradation]
security_entries: 2
security_entries_paired: 2
vacuous: false
---

# Edge-case matrix — TASK-IMP-115

The deliverable is one enum key per `skill_chain` line plus a doctrine section. The attack surface is not execution — nothing runs this field — it is **drift** (the label stops matching the work) and **false information** (a host routes on a lie).

| # | Category | Case | Behaviour required | Covered by |
|---|---|---|---|---|
| E1 | null/empty | A step carries NO `judgment` key | §1.1 violated — every step MUST carry one. A host that reads a missing key gets no annotation, i.e. the field is decoration | `test_every_step_has_judgment` — enumerates every chain row and fails on the first without the key |
| E2 | null/empty | `judgment:` present with an empty value (`judgment: ,`) | Not a member of the enum → fail | same arm: the value is matched against the enum, not against "is present" |
| E3 | null/empty | The chain block parses to ZERO steps (a broken parser) | The arm must NOT pass vacuously | same arm: asserts the parsed count equals an independently counted row count AND that it is ≥ 30 |
| E4 | bounds | A value outside the 3-value enum (`judgment: critical`, `judgment: High`) | Fail — the enum is closed and case-sensitive | `test_every_step_has_judgment` compares against the frozen set `{high, medium, mechanical}` |
| E5 | bounds | A step is ADDED to the chain later without the key | Fail — the arm enumerates rows, so a new row is a new obligation | `test_every_step_has_judgment` (no hard-coded step list) |
| E6 | bounds | A step is REMOVED | No false failure — the arm counts what it finds | same arm |
| E7 | malformed | The flow mapping breaks (unquoted `:` in a value) so the frontmatter no longer parses | Loud failure, never a silent skip | `test_every_step_has_judgment` parses the frontmatter with PyYAML; a parse error raises |
| E8 | malformed | `judgment` written as a nested block instead of a bare scalar | Fail — the value is not a string in the enum | same arm |
| E9 | malformed | The doc's helper table names a helper that does not exist on disk | Fail | `test_mechanical_steps_are_helper_backed` — `Path.is_file()` on the named helper |
| E10 | concurrency | This edit lands while IMP-106 is in flight on the same branch | No shared file: IMP-106's cone is `tools/install/**`, this task's is `modules/cuo/**`. Verified disjoint | context-map §7 |
| E11 | concurrency | The edit invalidates an in-flight ship-manifest (this task's own, or IMP-106's) | Must not. `workflow_version` stays 2.8.0, so Resume rule 1 (version mismatch → needs_human) does not fire; no step index moves, so no recorded step goes stale | context-map §3, §7 |
| E12 | concurrency | Two authors annotate different steps of the same file | Out of scope for this batch (single writer, one view — §11a). Recorded, not defended | — |
| E13 | **security** | A model string / price / host effort name enters the payload via this field (`judgment: high # use claude-fable-5`) | MUST fail. This is §1.4's actual prohibition and the whole point of the task | `test_no_host_specific_literals` — scans the chain block AND the new section for model families, currency literals, and host effort-setting names |
| E14 | **security** | The field becomes an INSTRUCTION — something in the payload starts reading it to decide | MUST NOT happen (§1.3). Today nothing reads it, by construction: no chain consumer validates or reads unknown keys | context-map §2 (table of all 9 consumers) + AC 4's recorded grep in the gate log |
| E15 | degradation | A host ignores the field entirely | Correct and the default. Advisory means ignorable; the workflow runs identically | Documented in the new §11e; no code path depends on the key |
| E16 | degradation (drift) | A step marked `mechanical` whose helper is later replaced by a model | The label is wrong until someone updates it. AC 2's arm is the detector: the mechanical claim must stay anchored to a helper that exists AND is named in the payload's own record of that skill | `test_mechanical_steps_are_helper_backed` |
| E17 | degradation | A judgment-heavy step is marked `mechanical` to make it cheap | Fail — the skill must be in the declared helper-backed set with an anchored, on-disk helper | same arm (proven by breaking it: see the code-review's load-bearing proofs) |
| E18 | degradation | Every step marked `high` "to be safe" | §1.5's failure mode — the expensive default returning. Not suite-detectable (no test can decide whether a level was guessed); it is AC 5's reviewer walk, and the new §11e carries a named reason for every `high` so the walk has something to check | AC 5 (`verify:`), gate-log |

## SECURITY-class pairing (audit requirement: every SECURITY entry has a test or an ADR)

- **E13** → paired to a test: `test_no_host_specific_literals`.
- **E14** → paired to AC 4's recorded grep (a negative structural claim; the spec itself routes this to `verify:` with the TASK-IMP-090 rationale) plus the consumer table in context-map §2, which enumerates all nine `skill_chain` readers and shows none reads an unknown key.

No ADR required: neither entry proposes an architectural deviation.
