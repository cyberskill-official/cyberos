---
title: "PROJ — migration from Asana, Linear, Jira, Trello (history import, label mapping, attachment streaming, contributor matching)"
author: "@stephen-cheng"
department: operations
status: ready_for_review
priority: p1
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: internal_tooling
eu_ai_act_risk_class: not_ai
target_release: "P1 / 2026-Q4"
client_visible: false
template: feature_request@1
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Migrate the team's existing project-tracker history into CyberOS PROJ so the team can fully replace the prior tracker at P1 → P2 exit (PRD §14.2.3). Migration covers four sources — **Asana** (the team's current primary), **Linear**, **Jira**, **Trello** — with per-source extractors that share a common normaliser. The migration produces: per-source Engagements + Projects + Cycles + Issues; preserved comments + attachments + state-history; mapped contributors (external user IDs → CyberOS Member IDs); preserved external IDs as `metadata.imported_from.{source}.{external_id}` so links from emails / docs / chat scrollback remain resolvable. Outcome: every active Member sees their work in CyberOS PROJ on day one with byte-identical comment text + attachment integrity, including the team's full Asana history (the prior primary) plus optional spillover from any auxiliary trackers.

## Problem

The team's institutional memory in PM tooling lives in Asana with secondary records in Trello (older internal initiatives) and Jira (one client engagement that mandated Jira). Replacing the tracker without migrating that history strands the memory; a Member who needs to find "what we decided about the Acme proposal flow back in February" has to keep Asana open. The PRD's P1 → P2 exit gate is unattainable.

Three failure modes the migration must avoid:

- **Lost issue context.** A migrated issue without its comment thread + attachments is a stub the Member ignores.
- **Broken external links.** Notion docs, Slack messages, and emails reference Asana URLs (`https://app.asana.com/0/12345/67890`); without preserved external IDs and a redirect surface, those links bit-rot.
- **Contributor identity drift.** A Member who appears as `khoa@cyberskill.world` in Asana, `@khoa-le` in GitHub, and `Khoa L.` in a Jira-instance must collapse to one CyberOS Member ID.

## Proposed Solution

The shape of the answer is a `cyberos-proj-import` CLI + service with per-source extractors, a common normalisation layer, the migration UI in `/projects/import`, and the redirect-surface for legacy external links.

**Per-source extractors.**

Each extractor implements the `ProjImportExtractor` trait:

```rust
trait ProjImportExtractor {
    async fn authenticate(&self, credentials: &Credentials) -> Result<Session>;
    async fn list_workspaces(&self, sess: &Session) -> Result<Vec<Workspace>>;
    async fn list_projects(&self, sess: &Session, ws: &Workspace) -> Result<Vec<RawProject>>;
    async fn list_issues(&self, sess: &Session, project: &RawProject) -> Result<Vec<RawIssue>>;
    async fn fetch_attachments(&self, sess: &Session, issue: &RawIssue) -> Result<Vec<RawAttachment>>;
    async fn list_users(&self, sess: &Session, ws: &Workspace) -> Result<Vec<RawUser>>;
}
```

Source implementations:

- **Asana.** Uses Asana's REST API (`https://app.asana.com/api/1.0/`) with personal-access-token or OAuth. Workspace → Asana workspace; Projects → Asana projects; Issues → Asana tasks; Comments → Asana stories; Custom Fields → preserved in `metadata.imported_custom_fields`. Sections become Cycles when they're date-bounded; otherwise become labels.
- **Linear.** Uses Linear's GraphQL API. Workspace → Linear workspace; Projects → Linear projects (with Engagement created if there's a 1:1 client mapping); Issues → Linear issues; Cycles → Linear cycles. Linear's data model is closest to ours; mapping is near-1:1.
- **Jira.** Uses Jira REST API v3. Workspace → Jira project (Jira inverts the term — an entire Jira "project" is one of our Projects). Issues → Jira issues; Sprints → Cycles; Components → labels; Epics → issues with `parent_issue_id`.
- **Trello.** Uses Trello REST API. Board → Project; List → workflow state seed (the team's lists become custom states); Card → Issue; Checklist items → sub-issues.

**Common normaliser.**

Each extractor produces normalised records that the importer applies to PROJ:

```rust
struct NormalisedIssue {
    external_id: String,
    external_url: String,
    title: String,
    description_md: String,
    state_in_source: String,           // raw source state
    state_mapped: String,              // mapped to default catalogue or per-project workflow
    priority: String,
    assignee_external_id: Option<String>,
    reporter_external_id: String,
    labels: Vec<String>,
    cycle_external_id: Option<String>,
    parent_external_id: Option<String>,
    blocked_by_external_ids: Vec<String>,
    blocks_external_ids: Vec<String>,
    comments: Vec<NormalisedComment>,
    attachments: Vec<NormalisedAttachment>,
    state_transitions: Vec<NormalisedStateTransition>,
    created_at: DateTime,
    updated_at: DateTime,
    completed_at: Option<DateTime>,
    metadata: serde_json::Value,
}
```

Two-pass import:

1. **First pass:** create the Engagement (1 per import default) → Projects → Cycles → Issues. External IDs are stored in `proj.issue.metadata.imported_from`. Issue numbering preserves source ordering — issues imported from Asana in order get sequential CyberOS numbers.
2. **Second pass:** resolve cross-references. `parent_external_id` → `parent_issue_id`; `blocked_by_external_ids` → `blocked_by_issue_ids`; `blocks_external_ids` → `blocks_issue_ids`. Cross-references that fail (the parent didn't import) are logged and surface as "review needed" items.

**Contributor matching.**

The contributor-matching service `cyberos-proj-import-matcher`:

1. Extracts source-side users + their email addresses + display names.
2. Matches by email against `auth.member.email`. Exact match → auto-assigned. Same-domain + similar-name → suggested match for human confirmation.
3. Unmatched users (e.g. former employees or external contractors) become "imported user" records that the HR/Ops Lead can later resolve (link to a Member or mark as external).
4. The matching UI surfaces suggested + manual-confirmed matches before the import proceeds.

**Attachment streaming.**

Attachments are downloaded from the source, hashed (SHA-256), stored in Stalwart's content-addressed blob store (the same one EMAIL uses; FR-EMAIL-001) with a `proj/` namespace. Duplicates dedup. Large attachments (> 30 MB) prompt the Member to confirm before download (the same threshold as EMAIL).

**Redirect surface for legacy links.**

A small Cloudflare Workers route at `https://link.cyberos.world/legacy/{source}/{external_id}` redirects to the corresponding CyberOS issue:

```
https://link.cyberos.world/legacy/asana/12345/67890
  → https://app.cyberos.world/projects/ALPHA/issue/ALPHA-1234

https://link.cyberos.world/legacy/jira/PROJ-456
  → https://app.cyberos.world/projects/BETA/issue/BETA-789
```

The redirect table is populated during import; the route reads from `proj.legacy_link_redirect{tenant_id, source, external_id, target_issue_id}`. Members are advised to update old documents but the redirect ensures legacy links don't 404.

**Migration UI (`/projects/import`).**

Per-import view shows: source, status (authenticating / extracting / matching / importing / validating / done / error), counts (workspaces, projects, issues, attachments, contributors), errors with retry buttons, and a "rollback this import" button (within 7 days of completion).

The HR/Ops Lead orchestrates the team's Asana migration; per-Member side-imports (e.g. a Member's personal Trello board) are self-service.

**Validation phase.**

Post-import:
- Sample 10 random issues across the import; render side-by-side with the source; the Member confirms the migration.
- Run a synthetic query ("find issues labelled `auth` from Q3") in both PROJ and the source; compare counts.
- Check that every external link in the redirect table resolves correctly.

**Rollback.**

Within 7 days of import completion, the importer can be rolled back: every record imported with `metadata.imported_from.{source}.import_id == <this-import>` is deleted; the rollback writes audit rows; legacy redirects are removed. After 7 days, rollback is no longer available; cleanup is per-record.

**Audit + observability.**

- `proj.import.{tenant}` audit scope.
- Prometheus counters: `cyberos_proj_import_total{source, status}`, `cyberos_proj_import_duration_seconds_bucket{source}`.
- OBS dashboard: "Migration progress" panel.
- Alerts: import-failure spike, contributor-match-rejection spike.

**MCP tool surface.**

- `cyberos.proj.import_status(import_id?)` (read).
- `cyberos.proj.import_start(source, credentials_ref, target_engagement_id?)` (`destructive: true; requires_confirmation: true; sensitivity: high; step_up_required: true`).
- `cyberos.proj.import_retry_failures(import_id)` (`destructive: false; idempotent: true`).
- `cyberos.proj.import_rollback(import_id)` (`destructive: true; requires_confirmation: true; sensitivity: high; step_up_required: true`).

## Alternatives Considered

- **Manual recreation in PROJ.** Rejected: the team's history is too large; migration is mandatory for the gate.
- **Source-only access via embedded iframe inside CyberOS.** Rejected: the architecture violates ownership; no MCP / RBAC / audit alignment.
- **Skip Trello + Jira; only Asana.** Considered; we ship Asana + Linear in P1 (Linear is similar enough that the cost is small), defer Trello + Jira to P2 unless a customer engagement demands earlier.
- **Rollback the entire schema rather than per-import marker.** Rejected: per-import marker makes selective rollback safe and forensic.

## Success Metrics

- **Primary metric.** P1 → P2 exit-gate: every active Member's Asana history fully imported into CyberOS PROJ; ≥ 99% of issues + comments + attachments preserved; the team uses CyberOS PROJ exclusively for ≥ 21 consecutive days.
- **Migration completeness.** ≥ 99% of source issues land with title + description + comments preserved; remaining ≤ 1% catalogued in `proj.import_failure`.
- **Latency / reliability.** A 10K-issue Asana workspace migrates in ≤ 4 hours; redirect lookups ≤ 100 ms p99.
- **Contributor matching.** ≥ 95% of source contributors auto-matched or accepted by HR/Ops Lead within 2 business days.

## Scope

**In-scope (P1 sprint cluster S1-5 to S1-6).**
- Per-source extractors for Asana + Linear in P1.
- Common normaliser + two-pass import.
- Contributor matcher.
- Attachment streamer.
- Redirect surface at `link.cyberos.world/legacy/...`.
- Migration UI at `/projects/import`.
- Validation playbook + 10-issue sample-render comparison.
- 7-day rollback window.
- Audit + dashboards + alerts.
- The four MCP tools.

**Out-of-scope (deferred).**
- Jira + Trello extractors (P2 unless customer demand).
- ClickUp / Notion-projects / monday.com extractors (P3).
- Live two-way sync with the source (we migrate, then cut over; no ongoing sync).
- Time-tracking import (deferred to FR-TIME-MIGRATE-001 in P2 with the TIME module; the time-entries from Asana / Jira sync there).

## Dependencies

- FR-PROJ-001..008.
- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-002 / FR-AUTH-003 / FR-MCP-001.
- FR-OBS-001 / FR-OBS-002.
- FR-EMAIL-001's content-addressed blob store (shared with PROJ).
- Cloudflare Workers for the legacy-link redirect.
- Source API credentials per Member (Asana PAT or OAuth).
- Compliance: PDPL Decree 13 (the imported issues + comments + attachments contain personal data; the import is processing under the existing tenant ToS plus the per-Member consent for Asana access). SOC 2 CC8 (change management — the migration is a change of record).
- Locked decisions referenced: DEC-121 (P1 covers Asana + Linear; Jira + Trello P2), DEC-122 (legacy-link redirect lives at `link.cyberos.world/legacy/`), DEC-123 (7-day rollback window).

## AI Risk Assessment

Not applicable. `eu_ai_act_risk_class: not_ai`. The migration is deterministic ETL; no AI inference. (CaMeL re-ingests the imported comments + descriptions through BRAIN, where the AI risk classification already lives.)

## AI Authorship Disclosure

- **Tools used:** Claude Cowork (Anthropic).
- **Scope:** drafted the FR end-to-end against the PRD + SRS; founder reviews and edits before status changes from `ready_for_review`.
- **Human review:** founder (`@stephen-cheng`) — final wording is the founder's responsibility.
