---
title: "EMAIL — Gmail migration: history import, label preservation, attachment streaming, contact dedup, undelivered-mail handling"
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

Migrate the team's Gmail history into the EMAIL module (FR-EMAIL-001) so the team can fully replace Gmail at P1 → P2 exit (PRD §14.2.3 — "EMAIL has fully replaced Gmail for at least 21 consecutive days"). Migration covers: full per-Member mailbox history (mbox import via Google Takeout or live IMAP); per-thread label → folder mapping; attachment streaming with content-addressed dedup; contact extraction + dedup against any pre-existing CRM contacts (FR-EMAIL-006); MX cutover plan with `cyberskill.world` and `cyberos.world` redirected from Google Workspace to Stalwart; undelivered-mail handling during cutover (Google Workspace's catch-all + 7-day delayed-mail retry); and a migration drill on a synthetic Member's mailbox before the founder's. Outcome: each of the 10 employees has their full Gmail history searchable in CyberOS EMAIL with byte-identical message bodies.

## Problem

The team's institutional memory lives in Gmail (5+ years for the founder; 1–4 years for Members). Replacing Gmail without migrating that history strands the memory; the team will not switch. The PRD's P1 → P2 exit gate ("EMAIL has fully replaced Gmail for at least 21 consecutive days") is unattainable if the team needs to keep both clients open to access old threads.

Three failure modes the migration must avoid:

- **Lost mail during cutover.** The MX-record swap window is the single most fragile moment; mail in flight can land in the old mailbox or bounce. The plan must catch every message.
- **Label semantics drift.** Gmail labels are many-to-many; CyberOS EMAIL folders are one-to-many. Naïve mapping loses information.
- **Attachment storage explosion.** A team's 5 years of Gmail can include 50+ GB of redundant attachments (the same PDF emailed 10 times across threads). Without content-addressed dedup, storage costs spike disproportionate to message count.

## Proposed Solution

The shape of the answer is a `cyberos-email-import` CLI + service that runs the migration in three phases (history → cutover → validate), a per-Member migration UI in `/email/import`, and the MX cutover plan + catch-all retry strategy.

**Phase 1 — History import (per Member).**

Two paths:

1. **Google Takeout mbox import** (preferred for speed). The Member exports their Gmail data via Google Takeout (`https://takeout.google.com/`) selecting "Mail (mbox format)". The resulting `.mbox` file (typically 1–20 GB per Member) is uploaded to a one-time signed URL.
2. **Live IMAP import** (preferred for fresh state). The Member authorises an OAuth scope (`gmail.readonly`) one-time; the importer connects via IMAP and walks every label.

Both paths feed the same parser:

- Each message is parsed via `mail-parser` Rust crate.
- Attachments are extracted, hashed (SHA-256), stored in Stalwart's content-addressed blob store; duplicates dedupe.
- The message envelope + headers + body + attachment refs are converted to a Stalwart-native message.
- Gmail labels map to CyberOS folders: a primary folder is chosen (`INBOX` by default; or the most-prominent label after INBOX); other labels become `flags` on the message (`label:Work`, `label:Acme`).
- The message is inserted via Stalwart's bulk-import API; the Postgres mirror (FR-EMAIL-001) is populated by the same NATS event-replay path.
- Threading is preserved via `Message-ID` + `In-Reply-To` + `References` headers; orphan messages whose parents are out-of-range fall into a "Imported · Orphaned threads" virtual folder.

A migration of 100K messages + 5 GB attachments completes in ≤ 4 hours on a single import worker; parallel workers across Members can run concurrently.

**Phase 2 — MX cutover.**

The MX cutover plan, scripted in `cyberos-email-import cutover --domain cyberskill.world`:

1. **T-7 days.** SPF/DKIM/DMARC records for Stalwart published *alongside* the Google Workspace records; both sets pass for outbound.
2. **T-2 days.** Send a "we're switching mail systems on <date>" announcement from each Member's account to active recipients (CRM-linked contacts especially).
3. **T-0.** MX record swapped: `cyberskill.world MX 10 mx.cyberos.world`. TTL on the MX is dropped to 5 minutes 24 hours before, restored to 1 hour after.
4. **T+0 to T+72 hours.** Google Workspace catch-all forwards any mail still arriving there to the Member's CyberOS mailbox via SMTP forward; this catches recipients whose DNS resolvers cache the old MX.
5. **T+7 days.** Google Workspace MX record removed; catch-all retained for emergency.
6. **T+30 days.** Google Workspace plan downgraded to a "vault" tier for read-only history access; mail-receive disabled.

The cutover is per-domain (we run `cyberskill.world` and `cyberos.world` on separate cutovers; `cyberos.world` was platform-only so cuts before `cyberskill.world`).

**Phase 3 — Validation.**

Post-cutover validation:

- Send a test email from each external account the team has registered (Stephen's personal Gmail, partner Gmails) to each Member's `@cyberskill.world` address; verify it lands in CyberOS EMAIL within 5 minutes.
- Send a test email from each `@cyberskill.world` to each external account; verify SPF/DKIM/DMARC `pass`.
- Run a 100-message synthetic load through each Member's address; verify no loss.
- Member-side: search a known-old query ("Acme proposal 2024-Q3") in CyberOS EMAIL; verify the expected thread appears.

**Migration UI (`/email/import`).**

Each Member sees their migration progress: total messages, imported, attachments deduped, errors. Errors are categorised (parser failure / oversized message / encoding error) with a "retry" or "skip" action. The HR/Ops Lead sees an aggregate view across the team.

**Contact extraction + CRM dedup.**

During import, every unique sender + recipient email is extracted into `email.imported_contact{tenant_id, email, display_name, first_seen, last_seen, message_count}`. Post-migration, a CRM dedup pass (FR-EMAIL-006 + batch-05) matches imported contacts to existing CRM contacts; high-volume unmatched contacts (≥ 10 messages with the team) are surfaced as "should this be a CRM contact?" suggestions to the Account Manager.

**Undelivered-mail handling.**

During cutover, mail that hits Google Workspace after the MX swap (DNS cache lag) is forwarded to CyberOS via SMTP from the catch-all. The forwarder marks these messages with `X-CyberOS-Imported: true` so they are not double-archived in BRAIN; a 7-day window is the floor — after 7 days, residual delivery to Google is rare enough that we accept the residual loss (and the catch-all stays up another 23 days as belt-and-braces).

**Rollback plan.**

If validation fails post-cutover (say, > 5% of test messages do not arrive within 5 minutes), the rollback:

1. Restore the Google Workspace MX record (TTL is short, DNS converges in ≤ 1 hour).
2. The CyberOS Stalwart cluster continues to receive mail at the catch-all (MX 20 record retained as backup throughout).
3. Re-run validation; identify the issue; reschedule the cutover.

The rollback path is documented as `obs/runbooks/email-cutover-rollback.md`.

**Audit + observability.**

Every imported message is recorded with `provenance: "gmail-import"` in `email.message_index.metadata`; every cutover step writes an audit row in `email.cutover.{tenant}` scope; the OBS Compliance Cockpit shows a "Gmail migration" panel during the cutover window with progress + validation status.

**MCP tool surface.**

- `cyberos.email.import_status(member_id?)` (read).
- `cyberos.email.import_start(member_id, source: "takeout"|"imap", credentials_ref)` (`destructive: true; requires_confirmation: true`).
- `cyberos.email.import_retry_failures(member_id)` (`destructive: false; idempotent: true`).
- `cyberos.email.cutover_status(domain)` (read).

## Alternatives Considered

- **Run Gmail and CyberOS EMAIL in parallel forever.** Rejected: the P1 → P2 exit gate requires full replacement; parallel-mode is a transitional state, not an end state.
- **Skip historical migration; only migrate going forward.** Rejected: the team will keep Gmail open to access old threads; the gate is unattainable.
- **Use Google's bulk-export API.** Considered as a tertiary path; the Takeout mbox export is the canonical bulk path, IMAP is the realtime path. Both are sufficient.
- **One-shot MX cutover with no catch-all.** Rejected: DNS caching is not optional; the catch-all is the only way to catch tail-end deliveries.

## Success Metrics

- **Primary metric.** P1 → P2 exit-gate: 14+21 consecutive days where the team uses CyberOS EMAIL exclusively (no Gmail open in any Member's browser; verified by absence of Gmail tabs in the Cowork browser-session telemetry).
- **Migration completeness.** ≥ 99% of pre-cutover Gmail messages searchable in CyberOS EMAIL; remaining ≤ 1% (parse failures, edge encodings) catalogued in `email.import_failure` with the per-message reason.
- **Cutover loss.** ≤ 0.1% of mail sent during the T-0 to T+72h window experiences > 5-minute delivery delay.
- **Storage efficiency.** Attachment dedup ratio ≥ 1.5× (i.e. raw attachment bytes / deduped storage ≥ 1.5).

## Scope

**In-scope.**
- `cyberos-email-import` CLI + service supporting both Takeout-mbox and live-IMAP paths.
- Parser + threading + label mapping + attachment dedup.
- Per-Member migration UI at `/email/import`.
- HR/Ops Lead aggregate view.
- MX cutover scripts per domain.
- Catch-all forwarder + 7-day window.
- Validation playbook + scripted tests.
- Rollback runbook.
- Contact extraction + CRM dedup hooks.
- Audit + OBS panels.
- The four MCP tools.

**Out-of-scope (deferred).**
- Migration from non-Gmail providers (Outlook, Zoho, custom IMAP) — P3.
- Migration of calendar (covered by TIME / a future calendar FR; not in EMAIL scope).
- Migration of Drive attachments referenced by email links (P2 if a KB module migration is planned).
- Cross-team-shared-mailbox migration semantics beyond the per-Member case (handled per-Member; aggregate shared inboxes are constructed post-migration).

## Dependencies

- FR-EMAIL-001 / FR-EMAIL-002.
- FR-INFRA-001 (storage + DNS).
- FR-AUTH-001 (OAuth flow for IMAP path).
- FR-AUTH-002 (audit log).
- FR-MCP-001 (destructive-confirmation on import_start + cutover).
- FR-OBS-001 / FR-OBS-002 (cutover dashboard + alert routes).
- FR-CP-002 (the import is a major personal-data ingestion event; the DPIA for EMAIL-001 covers it but the import-specific consent is Member-level).
- DNS authority for `cyberskill.world` and `cyberos.world` at Cloudflare.
- Compliance: PDPL Decree 13 (mail content is personal data; import is processing under the existing tenant ToS); SOC 2 CC7.
- Locked decisions referenced: DEC-094 (catch-all + 7-day window is the cutover floor), DEC-095 (history migration is mandatory for P1 → P2 gate).

## AI Risk Assessment

Not applicable. `eu_ai_act_risk_class: not_ai`. The migration pipeline is deterministic ETL; no AI inference. CaMeL re-runs on the imported messages as they enter BRAIN ingestion (FR-EMAIL-003), which keeps the AI risk classification at the BRAIN ingestion layer where it already lives.

## AI Authorship Disclosure

- **Tools used:** Claude Cowork (Anthropic).
- **Scope:** drafted the FR end-to-end against the PRD + SRS; founder reviews and edits before status changes from `ready_for_review`.
- **Human review:** founder (`@stephen-cheng`) — final wording is the founder's responsibility.
