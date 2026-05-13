<!-- Canonical task@1 body template. Loaded by `fr-with-tasks` and rendered per task inside a feature_request@1. -->

### {id} — {title}

**Sizing**: {sizing}  ({estimated_hours}h human / {estimated_tokens} tokens AI) **Assignable to**: {assignable_to}{ if ai-agent: " (profile: " + agent_profile + ")" } **Status**: {status}{ if owner: " — owner: " + owner } **Dependencies**: {dependencies | "none"} **Parallelisable**: {parallelisable}

#### Description

{description}

#### Preconditions

{preconditions as bullet list, or "none"}

#### Deliverables

{deliverables as bullet list}

#### Acceptance test

```
{acceptance_test.shell or acceptance_test.assertion}
```

{ if runbook_hint: "**Runbook hint**: `" + runbook_hint + "`" } { if notes: "**Notes**: " + notes }
