# Anti-fabrication rules (audit-side)

> Same contract as `cuo/cpo/fr-create/references/ANTI_FABRICATION.md`. The audit MUST NOT invent the very things `fr-create` is forbidden to invent — otherwise an audit-side hallucination would silently pass an FR that should fail. Sourced from `feature-request/FR_CREATE_AND_AUDIT.md` v2.0.0 §9.

The audit MUST NEVER:

- Invent customer quotes, attributions, or dates "to satisfy COND-001" (the audit reports the absence; it does not synthesise a quote).
- Invent metric baselines or numeric targets "to satisfy SEC-005" (it surfaces the absence via QA-007).
- Invent named entities (people, vendors, regulations) — the audit citation may only reference what's literally in the FR or in BRAIN.
- Assert an external team has agreed to a dependency without an in-FR reference (QA-008 fires).
- Change `eu_ai_act_risk_class` or `ai_authorship` autonomously — these are HITL-only fields per QA-001/002/003 and the audit's no-auto-fix list in `AUDIT_LOOP.md` §16.5.
- Set `ai_authorship: none` on output the audit itself produced (the audit does not produce FR content; it produces audit reports — but this rule is included for clarity in case future audit features generate FR-edit suggestions).

## Why this matters in the audit

An audit that invents a fix to make a rule pass is worse than no audit at all — it creates the appearance of conformance without the substance. The auditor therefore prefers to mark `status = needs_human` over producing a plausible-looking auto-fix when the underlying signal is ambiguous.

The Levenshtein-≤2 ambiguous-fix rule in `AUDIT_LOOP.md` §16.5 carries two carve-outs precisely for this reason: it MUST NOT auto-correct `eu_ai_act_risk_class` or `ai_authorship`, no matter how close the incorrect value looks to a valid enum.

## Cross-reference

The same rules are enforced on `fr-create`'s output before the FR reaches the audit. Symmetric enforcement means a bug in either skill is caught by the other on the first chained run — the contracts are defensive complements, not redundant.
