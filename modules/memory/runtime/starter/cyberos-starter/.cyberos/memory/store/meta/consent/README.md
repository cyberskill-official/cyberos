# Consent records (Layer-1 BRAIN Phase 0)

Personnel-gated memories (`classification: personnel`, `kind: person`, or
`scope` under `people`) MUST reference a consent event id that resolves to a
file in this directory: `meta/consent/<consent_event>.md`.

Use the `CONSENT` starter template:

`modules/memory/runtime/starter/templates/CONSENT.md`

These records are agent-BRAIN disclosures enforced by walker invariant
`personnel-requires-consent` (AGENTS.md §19). They are NOT the product EVAL
acknowledgment ledger — see `docs/deploy/brain-capture-activation.md`.
