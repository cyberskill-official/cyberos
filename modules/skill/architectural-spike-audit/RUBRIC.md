# architectural_spike_rubric@1.0

constants: OVERRUN_FACTOR=1.5 | HIGH_CONFIDENCE_MIN_EVIDENCE=2 (per surviving option)
verdict: pass requires 10/10; any rule failure -> fail; ambiguity -> needs_human.

## SPK-STRUCT - structural completeness
- SPK-STRUCT-001 frontmatter carries every architectural-spike@1 field with the typed
  shape (spike_id, task_id, question, timebox_hours, actual_hours, halted, options[],
  recommendation, confidence, discarded[], created).
- SPK-STRUCT-002 the five body sections present, in order: Question, Options probed,
  Evidence log, Recommendation, Discard log.
- SPK-STRUCT-003 recommendation names EXACTLY ONE option that appears in options[].
- SPK-STRUCT-004 spike_id matches `SPIKE-<FR-ID>-<n>`.

## SPK-EVID - evidence quality
- SPK-EVID-001 >= 2 options probed (a one-option spike is not a fork - route to the
  ADR lean fallback instead).
- SPK-EVID-002 every option carries >= 1 CHECKABLE evidence entry (repo file path that
  resolves at audit time, command plus observed output, or URL). Uncited assertions
  count as zero evidence.
- SPK-EVID-003 the recommendation cites >= 1 evidence entry from its own option.
- SPK-EVID-004 confidence cross-check: `high` requires >= HIGH_CONFIDENCE_MIN_EVIDENCE
  evidence entries per surviving option.

## SPK-BOX - timebox discipline
- SPK-BOX-001 timebox_hours recorded (integer >= 1).
- SPK-BOX-002 actual_hours recorded at close.
- SPK-BOX-003 actual <= OVERRUN_FACTOR x timebox, OR halted=true with the operator
  verdict recorded in the artefact.

## SPK-DISC - discard honesty
- SPK-DISC-001 discard log non-empty whenever any option was rejected.
- SPK-DISC-002 every discard entry names a reason (not just the option name).

## Prose -> rule mapping (TASK-SKILL-118 discipline)
Every rule above encodes a clause of architectural-spike-author/SKILL.md §2-§4 or
TASK-SKILL-117 §1; no rule is stricter than its prose source.
