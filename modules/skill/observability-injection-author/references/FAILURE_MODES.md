# `observability-injection-author` - failure modes

1. Log spam instead of signal - one point per transition, not per line; audit counts transitions from the diff.
2. Spans opened but never closed on error paths - error-branch counters double as the check.
3. PII leaks via interpolated messages - redaction policy + audit grep for known PII fields.
4. Estimate inflated - audit recomputes from the artefact's own tables; mismatch is a finding.
5. Instrumentation added to unshipped dead code - points must sit inside the FR's touched files.
