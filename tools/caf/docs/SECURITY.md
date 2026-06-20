# Security Policy

## Supported versions

Only the latest released version of the protocol (`AUDIT.md` on `main`) and its
validator (`core/evals/validate.py`) are supported. Historical snapshots in
`core/improve/versions/` are immutable records, not supported artifacts.

## Reporting a vulnerability

Email **info@cyberskill.world** with subject `SECURITY: code-audit-framework`.
Please do not open a public issue for anything exploitable. We aim to
acknowledge within 3 business days.

In scope:

- The validator (`core/evals/validate.py`) mis-parsing crafted run artifacts in a
  way that hides planted violations (validator bypasses).
- Secret-redaction gaps: a credential format that R8/`SECRET_PATTERNS` should
  plausibly catch but does not (include a sanitized example, never a live key).
- CI workflow weaknesses that would let a protocol change land unguarded.

Out of scope (by design, documented in `core/improve/BLINDSPOTS.md`):

- "The validator can't prove the agent actually ran the command" — execution
  authenticity is BS-01, an accepted limit; hard guarantees belong in the
  target repo's own CI.
- Prompt-injection resistance of any particular LLM running the protocol.

## Offline by design

The validator is stdlib-only Python with **no network access and no
telemetry**. Validating a client codebase sends nothing anywhere — suitable
for air-gapped and regulated environments. Any future feature that would
require network access must be opt-in and documented here first.

## Handling of secrets in this repo

This repository's own artifacts are held to the protocol's R8: credentials
never appear unredacted in any committed file. Eval fixtures that exercise the
secret-leak tripwire (e.g. `B07`, `B16`) use synthetic, non-functional values
with valid formats only.
