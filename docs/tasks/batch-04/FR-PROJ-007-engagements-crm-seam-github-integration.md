---
title: "PROJ — Engagements (contract → project mapping), CRM seam, GitHub integration, client-visibility flag"
author: "@stephen-cheng"
department: product
status: ready_for_review
priority: p1
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: user_facing
eu_ai_act_risk_class: not_ai
target_release: "P1 / 2026-Q4"
client_visible: false
template: feature_request@1
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Wire the **Engagement** primitive (FR-PROJ-001) into the platform's other modules: **CRM seam** (Engagement → CRM Account; Engagement contacts pulled from CRM; deal stage informs Engagement status) — stub against batch-05's CRM; **GitHub integration** (per-Engagement repo allowlist; webhook receiver; commit message → issue auto-link via `Closes #ALPHA-1234`); the **client-visibility flag** at the Engagement, Project, Cycle, and Issue level (with default cascade) that determines what the Client Portal (FR-PORTAL P4) eventually surfaces; **rate-card + budget tracking** placeholders consumed by INV (P2) and RES (P2); and the **Engagement dashboard** showing health metrics across Projects + Cycles + Issues + recent Activity (read-only in this FR; full UX builds on FR-PROJ-005). This FR is the connective tissue that makes the rest of the platform's contract-aware behaviour possible.

## Problem

PROJ alone is sufficient for internal task tracking; the Engagement layer is what makes PROJ a *consultancy* tool rather than a generic kanban. Three failure modes without this FR:

- **Disconnected work.** A team Member working on `ALPHA-1234` cannot see at a glance which contract it's billed against, which client it serves, or which deal it derives from. Decisions about scope creep + invoicing + revenue-sharing fork off into spreadsheets.
- **Manual GitHub link.** Linking a PR to an issue today means the Member writes the issue ID in the commit message and someone manually closes the issue when the PR merges. The auto-transition (FR-PROJ-003) needs the receiver this FR ships.
- **No client-visibility plumbing.** The PRD's P4 Client Portal (PRD §14.5.1) needs years of structured client-visibility decisions to land before P4; this FR ships the flag at the right granularity so the future portal can render correctly.

## Proposed Solution

The shape of the answer is the Engagement-side enrichments to the schema (extending FR-PROJ-001), the GitHub webhook receiver, the CRM-stub interfaces, the client-visibility cascade, and the Engagement dashboard payload.

**CRM seam (stub).**

`proj.engagement.client_account_id` references `crm.account.id`. Today (P1, before CRM ships) the column is nullable; when CRM ships in batch-05, a backfill matches existing Engagements to CRM Accounts by name + domain.

The Engagement dashboard reads:
- `crm.account.name` for the client display name.
- `crm.account.primary_contact_id` → `crm.contact` for the primary client contact.
- `crm.deal` records associated with the account, filtered to those whose `engagement_id` matches.
- Recent CRM activities (FR-EMAIL-006) for the account.

The reverse direction: when a CRM Deal moves to `closed-won`, CUO/CRO surfaces a Notify suggesting "create an Engagement?" with pre-drafted fields.

**GitHub integration.**

```sql
CREATE TABLE proj.github_install (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  github_installation_id BIGINT NOT NULL,
  installed_by_member_id UUID NOT NULL,
  installed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);

CREATE TABLE proj.github_repo_link (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  engagement_id UUID NOT NULL REFERENCES proj.engagement(id) ON DELETE CASCADE,
  github_owner TEXT NOT NULL,
  github_repo TEXT NOT NULL,
  is_default BOOLEAN NOT NULL DEFAULT false,
  added_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (tenant_id, engagement_id, github_owner, github_repo)
);

CREATE TABLE proj.github_pr (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  engagement_id UUID,                          -- derived from the repo link
  github_owner TEXT NOT NULL,
  github_repo TEXT NOT NULL,
  pr_number INT NOT NULL,
  title TEXT,
  state TEXT,                                  -- "open" | "closed" | "merged"
  author_github_login TEXT,
  linked_issue_ids UUID[],                     -- proj.issue references derived from commit-message regex
  opened_at TIMESTAMPTZ,
  merged_at TIMESTAMPTZ,
  closed_at TIMESTAMPTZ,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  UNIQUE (tenant_id, github_owner, github_repo, pr_number)
);
```

The `cyberos-proj-github-receiver` service listens for webhook events:

1. **HMAC verification** against the GitHub App's secret; rejects on mismatch.
2. **Repo allowlist check** — the source `(owner, repo)` must be in `proj.github_repo_link` for at least one Engagement in the tenant; otherwise the event is dropped with an audit row.
3. **Event handling.** PR opened / synchronised / closed / merged; commit message regex extracts `(Closes|Fixes|Resolves) #([A-Z]+-\d+)` patterns; linked issues are looked up by key; the link is recorded in `proj.github_pr.linked_issue_ids`.
4. **Auto-transitions** (FR-PROJ-003): on PR merged → linked issues transition per project workflow; on PR closed-without-merge → Notify card asks the assignee to handle.
5. **Issue → PR backlink** on the issue's `external_refs[]`.

GitHub App is registered as `cyberskill-cyberos`; per-tenant installation; OAuth flow gates the install.

**Client-visibility cascade.**

The flag lives at four levels:
- `proj.engagement.client_visibility_default` — default for new Projects in this Engagement.
- `proj.project.client_visibility` — overrides Engagement default; defaults to inherit.
- `proj.cycle.client_visibility` — overrides Project; defaults to inherit.
- `proj.issue.client_visible` — overrides Cycle; defaults to inherit (false unless explicitly set true).

Cascade resolution at read time: an issue is considered client-visible if its own flag is true OR (it's not explicitly false AND its cycle is visible OR (cycle inherits AND project is visible OR (project inherits AND engagement default permits))).

The flag controls:
- Whether the issue surfaces in P4's Client Portal (FR-PORTAL).
- Whether the issue surfaces in cycle-review drafts ingested to BRAIN with `audience: "client"` (so client-facing summaries can pull only client-visible issues).
- Whether automatic CRM signals (FR-EMAIL-006) reference the issue.

UI surfaces (FR-PROJ-005) show a small chip per issue: `Client-visible` (Ochre) vs. `Internal-only` (Charcoal). Toggle requires `lead` role on the project.

**Rate-card + budget tracking placeholders.**

`proj.engagement.rate_card` JSONB schema (consumed by INV in P2):
```json
{
  "currency": "USD",
  "rates": [
    { "role": "engineering_senior", "hourly": 85, "minimum_hours": 0 },
    { "role": "engineering_mid", "hourly": 65 },
    { "role": "design", "hourly": 80 },
    { "role": "pm", "hourly": 95 }
  ],
  "discount_pct": 0,
  "minimum_monthly": 0,
  "billing_cadence": "monthly",
  "net_terms_days": 30
}
```

`proj.engagement.budget_hours` and `budget_amount_minor + budget_currency` track the contract envelope. P1 ships the storage; P2 (FR-INV-001) ships the consumption + alerting + invoicing.

**Engagement dashboard payload.**

A new GraphQL field `engagementDashboard(id: ID!): ProjEngagementDashboard!` returns:

- Engagement metadata (name, contract, dates).
- Active projects + their states.
- Active cycles' progress.
- Aggregate health: at-risk-issue count, blocked-issue count, this-cycle-velocity, rolling-velocity.
- Recent activity feed (last 30 events: state transitions, comments, PR merges, CRM signals).
- Budget burn (hours used / hours budgeted; gauge).
- Client-visibility summary (count of client-visible issues this cycle).

Used by FR-PROJ-005's Engagement page and by the Founder Daily Flow (FR-GENIE-003).

**MCP tool surface (extends FR-PROJ-001..006).**

- `cyberos.proj.list_engagement_repos(engagement_id)` — read.
- `cyberos.proj.link_repo(engagement_id, owner, repo)` — `destructive: false`; idempotent.
- `cyberos.proj.unlink_repo(engagement_id, owner, repo)` — `destructive: true; requires_confirmation: true`.
- `cyberos.proj.list_prs(engagement_id?, state?)` — read.
- `cyberos.proj.set_engagement_visibility(id, default)` — `destructive: true; requires_confirmation: true`.
- `cyberos.proj.engagement_dashboard(id)` — read.

## Alternatives Considered

- **Skip the GitHub integration in P1; manual-link PRs.** Rejected: the auto-transition behaviour (FR-PROJ-003) requires it; manual-link is the workflow we're replacing.
- **Direct GitHub API polling instead of webhooks.** Rejected: rate limits + freshness; webhooks are the floor.
- **Engagement absorbed into Project (no separate primitive).** Rejected (also at FR-PROJ-001): the contract layer above Project is structurally needed.
- **Per-tenant default client-visibility = true (transparent by default).** Rejected: the prior tracker's accidental over-sharing is exactly the failure mode; conservative default + explicit opt-in is the floor.
- **GitLab + Bitbucket + Gitea support in P1.** Considered; rejected for P1 — GitHub is what the team uses; others arrive in P3 if the customer base demands.

## Success Metrics

- **Primary metric.** P1 sprint demo passes: (1) a GitHub PR with `Closes #ALPHA-1234` in the commit message merges; the issue auto-transitions to `done` within 10 s p95 of the webhook; (2) the Engagement dashboard renders for the canonical CyberSkill-Acme Engagement with all five aggregate panels populated; (3) flipping an issue to `client_visible: true` cascades correctly through the read-resolution.
- **Adoption metric.** Every active Engagement has at least one linked GitHub repo by P1 → P2 exit.
- **Latency NFR.** Webhook processing ≤ 2 s p95 from receipt to issue auto-transition.

## Scope

**In-scope.**
- `proj.github_install`, `proj.github_repo_link`, `proj.github_pr` tables.
- `cyberos-proj-github-receiver` service with HMAC + allowlist + commit-message regex.
- Auto-transition wiring back to FR-PROJ-003.
- Client-visibility cascade resolution.
- CRM stub interfaces (real in batch-05).
- `engagementDashboard` GraphQL field + payload.
- Rate-card + budget storage placeholders (no consumption logic — that's P2).
- The six new MCP tools.
- Notify card "Deal closed-won → create Engagement?" (when CRM ships).

**Out-of-scope (deferred).**
- GitLab / Bitbucket / Gitea (P3).
- Bidirectional issue ↔ PR sync (P2 — today only PR → issue auto-transition).
- Budget-burn alerting (P2 — FR-INV-001 owns it).
- Client-portal rendering (P4 FR-PORTAL).
- Multi-engagement contract-stack (one customer with multiple Engagements summed) — P3.

## Dependencies

- FR-PROJ-001..006.
- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-002 / FR-MCP-001 / FR-AI-001 / FR-OBS-001.
- FR-GENIE-001 / FR-GENIE-002 (Notify cards for create-Engagement-from-deal).
- FR-CRM-001 + FR-CRM-002 + FR-CRM-003 (batch-05) for the real CRM data; this FR ships against stubs.
- A GitHub App `cyberskill-cyberos` registered in the CyberSkill GitHub org.
- Compliance: PDPL Decree 13 (Engagement metadata can contain client personal data; the existing audit + RLS controls apply); SOC 2 CC8 (change-control on the engagement → repo allowlist).
- Locked decisions referenced: DEC-116 (Engagement → CRM Account 1:1), DEC-117 (client-visibility cascade with conservative inherit), DEC-118 (GitHub-only auto-link in P1).

## AI Risk Assessment

Not applicable. `eu_ai_act_risk_class: not_ai`. The plumbing is deterministic; the "deal closed-won → suggest Engagement" Notify card inherits FR-GENIE-001 risk classification.

## AI Authorship Disclosure

- **Tools used:** Claude Cowork (Anthropic).
- **Scope:** drafted the FR end-to-end against the PRD + SRS; founder reviews and edits before status changes from `ready_for_review`.
- **Human review:** founder (`@stephen-cheng`) — final wording is the founder's responsibility.
