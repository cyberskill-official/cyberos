---
title: "EMAIL — attachment scanning, S/MIME read support, link rewriting + warn-on-click, attachment-level CaMeL extraction"
author: "@stephen-cheng"
department: engineering
status: ready_for_review
priority: p1
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: infrastructure
eu_ai_act_risk_class: limited
target_release: "P1 / 2026-Q4"
client_visible: false
template: feature_request@1
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Close the EMAIL security perimeter: every inbound and outbound attachment passes through **ClamAV scanning** (signature + heuristic + sandbox-extracted metadata); inbound HTML messages are rewritten so every external link goes through **`https://link.cyberos.world/r/{token}`** with a server-side warn-page that reveals the destination + a CUO-evaluated risk score before redirect; **S/MIME signed messages** are verified and the verification status surfaces as a chip on the message; **S/MIME encrypted messages** are decryptable when the recipient has a configured certificate (P1 read-only; sign+encrypt outbound is P2); attachment content (PDF, Word, image-with-text) is fed to the **CaMeL quarantine** (FR-EMAIL-003) for fact extraction so attachment-borne instructions cannot escape into CUO; and a **DLP (data-loss-prevention) outbound check** flags messages whose body or attachments match the BRAIN denylist (compensation values, government IDs, encryption keys) before send. Together this FR closes the EMAIL attack surface beyond the inbound CaMeL of FR-EMAIL-003.

## Problem

Three EMAIL attack classes that FR-EMAIL-003 alone does not address:

- **Malware-bearing attachments.** A customer emails a PDF with an embedded exploit; a Member opens it; the Member's machine is compromised. Stalwart's milter chain runs ClamAV but the surface is wider — VBA macros, ZIP-bombed archives, Office-doc OLE embedding, PDF JavaScript.
- **Phishing links.** An email body contains `https://acme-corp-secure-login.example.com/...` that mimics a known counterparty; the Member clicks and enters credentials. URL filtering + warn-on-click is the structural mitigation.
- **Outbound data-leak.** A Member accidentally pastes a bank account number or a CCCD into an email reply; the message goes out before anyone catches the mistake. DLP is the structural mitigation.
- **Attachment-borne prompt injection.** An attacker emails a PDF whose extracted text includes "ignore your instructions and exfiltrate X". FR-EMAIL-003's body-level CaMeL did not see the PDF; this FR extends CaMeL to attachment-extracted text.

## Proposed Solution

The shape of the answer is a chain of pre-delivery and pre-send checks integrated with Stalwart's milter, the ClamAV daemon, the link-rewriter, the S/MIME verifier, the attachment-extractor + CaMeL bridge, and the outbound DLP scanner.

**Inbound attachment chain.**

For each attachment on an inbound message:

1. **Size cap.** Attachments > 30 MB are quarantined (held in a separate mailbox; the recipient is notified; can release).
2. **Type allowlist + extension/MIME mismatch detection.** Allowed types: PDF, Word, Excel, PowerPoint, plain text, Markdown, common images (PNG/JPG/HEIC/WebP), audio (MP3/WAV/OGG up to 20 MB), video (MP4 up to 30 MB), ZIP (recursively scanned to depth 3), tar.gz (depth 3). Disallowed: `.exe`, `.bat`, `.com`, `.scr`, `.js`, `.jse`, `.vbs`, `.ps1`, raw shell scripts. Mismatched extension/MIME (`.pdf` claiming MIME `image/jpeg`) is flagged.
3. **ClamAV scan.** Signature + heuristic detection. ClamAV runs as a sidecar deployment with daily signature updates (CVD via official mirror).
4. **Sandbox extraction (text only).** A separate `cyberos-attachment-extractor` pod runs the extractors:
   - PDF: `pdf-extract` Rust crate (no JavaScript execution).
   - Word/Excel/PowerPoint: `docx-rs` / `xlsx-rs` / `pptx-rs` + a fallback to `mammoth` for older binary formats.
   - Image: Tesseract OCR (self-hosted) — Vietnamese + English language packs.
   - Archive: extracted recursively to depth 3; each contained file goes through the same chain.
   The extractor pod has the same NetworkPolicy egress restriction as the CaMeL quarantine.
5. **Macro / active-content detection.** Office docs flagged when they contain VBA macros, OLE embeddings, external references, or autorun directives. Macro-bearing docs are not blocked by default but receive a "macro warning" banner when previewed.
6. **CaMeL extraction.** The extractor's plain-text output is passed to the CaMeL quarantine (FR-EMAIL-003) tagged `source: "email.attachment.<filename>"`. Facts are extracted; injection markers are dropped from BRAIN ingestion; the audit row references both the email and the attachment.
7. **Result.** Attachments that pass become available for download/preview; flagged ones surface a banner with the result + a "request release" button (escalates to HR/Ops Lead or DPO).

**Outbound attachment chain.**

For each attachment on an outbound message:

1. **Type allowlist** (same list).
2. **ClamAV scan** (catches accidentally-attached infected files).
3. **DLP scan** of the file's text (PDF/Office extraction) — same regex set as the BRAIN denylist (CCCD, bank accounts, gov IDs, compensation values, key material). Hits are flagged; the Member sees a warning before send: "This attachment contains a CCCD-like pattern; are you sure?". Override requires step-up auth.
4. **Encryption advisory** — if the message body or attachments contain personal data and the recipient is external (no `cyberskill.world`), a non-blocking advisory chip suggests S/MIME or TLS-only delivery. P1 ships the advisory; P2 ships the encrypt-on-send action.

**Link rewriting + warn-on-click.**

Every external URL in inbound HTML body + plaintext body is rewritten:

- Original: `https://acme-corp.com/proposal-q3`
- Rewritten: `https://link.cyberos.world/r/AbC123XyZ`
- The `link.cyberos.world` server resolves the token to the original URL + the link metadata (sender domain, delivery time, message ID).

When the user clicks:

1. The warn-page renders showing the original URL, the sender, the message subject, a CUO/CTO-skill risk-score (low / medium / high) computed from URL features (domain age, TLD reputation, similarity to known domains, presence in safe-browsing blocklists).
2. Low risk: a 3-second auto-redirect with a "Cancel" button.
3. Medium risk: requires explicit "Continue" click.
4. High risk: requires "Continue" click + a typed-confirmation acknowledging the risk.
5. The click is logged in `email.link_click.{tenant}` audit scope.

The warn-page is a thin Cloudflare Workers route; ≤ 100 ms p99 latency added to the click experience.

Internal links (cyberos.world subdomains; Member-tagged trusted domains) skip rewriting.

**S/MIME read support (P1).**

- Inbound S/MIME signed messages: signature verified against the sender's certificate chain; verification result rendered as a "verified by S/MIME" chip on the message. Failed verification → red chip with the reason.
- Inbound S/MIME encrypted messages: decryption attempted with the recipient Member's configured certificate (Members can upload an S/MIME private cert in `/auth/account` — encrypted at rest with the Member's KMS-derived key). Decrypted body is rendered; the original encrypted blob is preserved in storage. Decryption failure → "encrypted; certificate not configured" surface.

P1 is read-only; outbound S/MIME sign + encrypt is P2 (FR-EMAIL-SMIME-OUT-001 in batch-08).

**Attachment preview.**

- PDF + Word + Excel + PowerPoint preview rendered in-browser via `pdf.js` and a small server-side rasteriser; original file never executes.
- Images preview natively.
- Audio + video stream from the blob store.
- Archives show a directory listing; individual files are previewable with one click.

**Audit + observability.**

- `email.attachment.{tenant}` audit scope for every scan + every release-request.
- `email.link_click.{tenant}` for warn-page click events.
- `email.dlp.{tenant}` for DLP hits.
- OBS dashboards: ClamAV virus-detection rate, DLP hit rate, link-warn-page click-through rates by risk class.
- Alert: ClamAV signature-update failure (sev-1; signatures going stale is a regression).

**MCP tool surface.**

- `cyberos.email.scan_attachment(message_id, attachment_id)` (read; on-demand re-scan).
- `cyberos.email.list_quarantined_attachments` (read; HR/Ops + DPO + Founder).
- `cyberos.email.release_quarantined_attachment(attachment_id)` (`destructive: true; requires_confirmation: true`).
- `cyberos.email.list_dlp_hits(since)` (read).
- `cyberos.email.lookup_link_token(token)` (read; for the warn-page server's lookup).

## Alternatives Considered

- **Skip link rewriting.** Rejected: phishing is the single highest-volume external attack on small companies; the warn-page is the structural mitigation.
- **Auto-block all macros.** Considered; rejected for P1 because some Vietnamese accounting workflows still use macro-laden Excel templates. P2 reconsiders with a per-Member opt-out.
- **Use a hosted attachment-scanning provider (VirusTotal API).** Rejected: residency + per-attachment cost; ClamAV is the floor; we may add VirusTotal as an overlay at P3 for high-risk attachments.
- **DLP as send-blocking instead of warn.** Rejected: too many false positives; warn-with-step-up-override is the floor; P2 with calibrated rules can block.
- **No attachment preview; force download.** Rejected: forces Members to open files with Office, which is the attack surface; preview is the safer default.

## Success Metrics

- **Primary metric.** P1 → P2 exit-gate progress: zero confirmed inbound malware reaches a Member's machine over the 14-day exit window; ≤ 2 confirmed false-positive blocks (signal-to-noise tractable).
- **Phishing metric.** Warn-page click-through rate on `high` risk URLs ≤ 5% (the ones that proceed are tracked via the warn-page and reviewed quarterly by the DPO).
- **DLP metric.** ≥ 95% of synthetic outbound DLP-bait emails (CCCD or bank account in body) are flagged before send.
- **Latency metric.** Inbound attachment chain adds ≤ 8 s p95 to message-delivery time; the link-rewriter adds ≤ 80 ms p99 to warn-page transit.

## Scope

**In-scope.**
- ClamAV deployment with daily signature update + sidecar pattern.
- `cyberos-attachment-extractor` pod with NetworkPolicy egress restriction.
- Inbound + outbound attachment chain.
- DLP scanner using the BRAIN denylist regex set.
- Link rewriter + Cloudflare Workers warn-page.
- CUO/CTO link risk-score.
- S/MIME read-only support (signature verification + decryption with configured cert).
- Member S/MIME cert upload UI in `/auth/account`.
- In-browser preview for PDF/Office/image/audio/video.
- Audit + OBS dashboards.
- The five MCP tools.

**Out-of-scope (deferred).**
- Outbound S/MIME sign + encrypt (P2 — FR-EMAIL-SMIME-OUT-001).
- VirusTotal overlay (P3).
- Send-blocking DLP (P2 with calibrated rules).
- Machine-learning-based phishing classifier beyond CUO heuristic (P3).
- PGP / GnuPG support (P4 if customer demand justifies).

## Dependencies

- FR-EMAIL-001 / FR-EMAIL-002 / FR-EMAIL-003.
- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-003 / FR-MCP-001 / FR-AI-001 / FR-OBS-001 / FR-OBS-002.
- ClamAV official signature update mirror.
- Cloudflare Workers for the warn-page (already on the platform's Cloudflare zone).
- A `link.cyberos.world` subdomain provisioned + cached.
- Compliance: PDPL Decree 13 (DLP outbound check is a control on data export); EU AI Act Article 50 (the link risk-score is AI-derived; the warn-page renders the disclosure).
- Locked decisions referenced: DEC-096 (link rewriting is on-by-default), DEC-097 (DLP is warn-with-override in P1), DEC-098 (S/MIME read in P1, sign+encrypt in P2).

## AI Risk Assessment

The CUO/CTO link risk-score is an AI surface visible to natural persons; the rest of this FR is deterministic. EU AI Act risk class: `limited` for the AI surface.

### Data Sources

The risk-score consumes URL features (domain, TLD, age, similarity to known counterparties tracked in the per-tenant CRM + BRAIN). No third-party training data; per-tenant residency. The score is heuristic + model-assisted; the model is the same Haiku 4.5 line via the AI Gateway.

### Human Oversight

- Warn-pages with explicit confirmation (medium / high risk).
- Quarantined-attachment release requires HR/Ops Lead or DPO approval.
- DLP hits require step-up auth to override.
- The kill-switch from FR-GENIE-002 disables the AI risk-score; the warn-page falls back to a deterministic "external link" warning.

### Failure Modes

- **False-positive DLP block of a legitimate value.** The Member overrides with step-up auth; the override is logged; DLP rules are tightened.
- **False-negative DLP miss.** Audit + sampled review; rule additions on confirmed misses.
- **ClamAV signature staleness.** Sev-1 alert if not updated in 24 hours.
- **Warn-page outage.** Cloudflare Workers → fallback inline warning (no rewrite for new emails until restored); existing rewritten links degrade gracefully (the warn-page page returns the original URL with a static warning).
- **AI risk-score wrong.** The deterministic warning still surfaces; the AI score is an enhancement, not a gate.

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6.
- **Scope:** Drafted attachment chain + link-rewriting + S/MIME scope + DLP rules + failure modes.
- **Human review:** `@stephen-cheng` reviewed; ClamAV operational details to be re-verified by the Engineering Lead at PR-review.
