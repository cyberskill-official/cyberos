# Standalone-mode architectural-review interview (5-7 questions)

> The PRD covered product intent; these questions cover system design.

## Q1 — Architecture style

> "Which architecture style fits this PRD? (a) extend an existing service, (b) new microservice, (c) frontend-only, (d) data pipeline / batch job, (e) hybrid (specify which combo)."

## Q2 — Datastore decision

> "Which datastore(s)? (a) existing Postgres + new tables, (b) existing Postgres + existing tables (extend), (c) new datastore (which? why?), (d) cache-only / no persistence."

For (c), route to CTO via `chat.review_request` if not already CTO who's authoring.

## Q3 — Performance target

> "P95 latency target for the primary user story? Numeric value with units (e.g., '300ms at p95 for 1k req/s'). Required for Quality Bars."

## Q4 — Scaling envelope

> "What's the current load (requests / day or active users)? Expected load 12 weeks post-launch? 12 months?"

## Q5 — Security review trigger

> "Does this introduce: (a) new authentication surface, (b) new data flow leaving our infrastructure, (c) new secret-store usage, (d) new encryption-at-rest decision, (e) cross-tenant data access? Any 'yes' here triggers CSecO sign-off."

## Q6 — Telemetry budget

> "What metric / event budget can this carry? Roughly: how many events / sec at full load? Affects `genie.action_log` retention sizing + observability cost."

## Q7 — Rollback complexity (skip if PRD's rollout is full-on)

> "If we ship this and need to roll back in week 2, what's the cost? (a) toggle a flag, (b) re-deploy previous version, (c) data migration to reverse, (d) contractual penalty."

## After the interview

Authority-elevation pass on Architecture claims (mirrors prd-author's). Then synthesise SRS.

## Citations

- Pattern source — sibling `cuo/cpo/prd-author/STANDALONE_INTERVIEW.md`.
- INV-002 — no llm-implicit on Architecture; Q5 + Q6 + authority-elevation enforce.
