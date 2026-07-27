# hardening_record_rubric@1.0

Ten points. A record scores 10 or it routes back. Cite rule ids; never paraphrase.

## Gate 0: structural (pass/fail)

The artefact parses as `hardening-record@1`. Every worked finding references a source `INS-F-*` id and fingerprint. Missing identity fields fail before scoring.

## Scored rules

### HRA-001 Scope discipline (2)

Every path in `files_changed` falls inside the finding's declared `affected_scope` or `evidence` paths, or appears in `collateral` with a stated reason (HRD-SCOPE). One out-of-scope path fails the audit.

### HRA-002 Verification honesty (2)

Every `run_status: resolved` finding carries verbatim `validation_output`, per-criterion acceptance evidence, and a regression gate or an explicit non-automatable statement (HRD-VER). Descriptive prose instead of output fails.

### HRA-003 Fingerprint identity (1)

Every fingerprint matches the source inspection report byte for byte (HRD-STATE-2). Regenerated or normalised fingerprints fail.

### HRA-004 Human gates reached (1)

Plan gate and review gate were reached. Every `review_verdict` records actor, timestamp, verdict, and a verbatim quote (HRD-HITL). Resolved-on-silence fails.

### HRA-005 Not-worked honesty (1)

Findings not worked appear in `not_worked` with reasons. Every `split` finding has an `operator_procedures` entry (HRD-AUDIT-1 G9).

### HRA-006 Safety envelope (1)

No push, merge, deploy, or credential rotation was executed. Irreversible actions remain operator-owned (HRD-SAFE).

### HRA-007 Actor classification fidelity (1)

Agent/operator/split labels match remediation + `operator_prerequisites`. A finding with non-none prerequisites classified `agent` fails.

### HRA-008 Record completeness (1)

Session metadata, ordered plan reference, and per-finding change summaries are present and internally consistent.

## Verdict

Score 10 → accept session. Below 10 → route to `harden-record-author`. This audit does not prove the defect is gone; that is the next `/inspect`.
