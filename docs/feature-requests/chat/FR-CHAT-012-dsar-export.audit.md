---
fr_id: FR-CHAT-012
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 21
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; AUTHORING.md §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (no-line-cap expansion per AUTHORING.md §0; ISS-007..021 added)
---

## §1 — Verdict summary

FR-CHAT-012 expanded 2026-05-16 P.M. via strict-redo no-line-cap pass. Current scope: 27 §1 clauses (table + RBAC, REST endpoints, query, zip composition, S3+KMS+TTL, delivery + one-time-use, audit kinds, sharding, latency budget, verify procedure, RLS, metrics, mentions section, channel memberships section, reactions section, files section, devices section, Lumi interactions section, retro captures section, DSAR history section, passphrase-protected zip, export_chain_hash, partial-fulfilment chain across shards, 30-day SLA + 24h target, preview mode, subject acknowledgement, redact-on-export per tenant policy). 19 §2 rationale paragraphs. §3 contains: full dsar_requests table with preview/shard/sla fields + url_access_log + tenant_redact_categories, multi-section compose_full_export with all 9 sections + manifest with export_chain_hash + salt, preview composer, S3 upload with presigned URL, one-time-use Lambda + Postgres access log. 31 ACs. §5 contains 17 named test bodies covering authz matrix + zip contents + chain anchor + mentions + memberships + reactions + files + device-token-exclusion + Lumi + retro + history + passphrase derivation + export hash + sharding + SLA + preview + acknowledgement + redact-categories + URL-reuse-integration. §6 deepens with 8 wiring subsections (service deployment, preview-then-full workflow, sharding semantics, one-time-use detection, passphrase derivation, SLA monitoring, failure routing, operator CLI). §8 lists 8 example payloads (requested + delivered preview + delivered full sharded + fully_delivered + acknowledged + url_reused + manifest excerpt + README excerpt). §10 lists 50 failure rows. §11 lists 30 implementation notes covering S3 KMS rationale, TTL calibration, dual delivery, JSONL choice, HKDF over PBKDF2 reasoning, log-based reuse detection, why-not-query-language, justification storage, slice-4+ Windows support deferral.

## §2 — Findings (all resolved)

### ISS-001 — Authorisation matrix
Without spec, admin can pull anyone's data. Resolved: §1 #2 + DEC-530 three rules.

### ISS-002 — Chain-anchor inclusion
Without it, recipient can't prove authenticity. Resolved: §1 #4 + DEC-531.

### ISS-003 — URL lifetime
Long-lived = leak risk. Resolved: §1 #5 + DEC-532 7-day TTL.

### ISS-004 — Tamper-detection
Without verify.sh, subject depends on CyberOS. Resolved: §1 #10 bundled verify script + AC #14.

### ISS-005 — Shard limit
Single zip > 100MB = email-friction. Resolved: §1 #8 100K cap.

### ISS-006 — One-time-use
Re-share of URL = unauthorised access. Resolved: §1 #6 + AC #9 + sev-1 audit.

### ISS-007 — Mentions section missing (strict-redo pass)
PDPL/GDPR define "personal data" to include data ABOUT the subject, not just authored BY them. Original spec only included authored messages. Resolved: §1 #13 + `mentions.jsonl` + AC #17 + test body verify.

### ISS-008 — Channel memberships missing (strict-redo pass)
Subject's channel join/leave history is personal data (reveals workplace activity). Resolved: §1 #14 + `channel_memberships.jsonl` sourced from FR-CHAT-005 memory rows + AC #18.

### ISS-009 — Reactions in both directions missing (strict-redo pass)
Subject's reactions to others (their behaviour) AND others' reactions to them (how they're perceived) both qualify as personal data. Resolved: §1 #15 + `reactions.jsonl` with direction flag + AC #19.

### ISS-010 — Files metadata not included (strict-redo pass)
Subject's file uploads have metadata (filename, size, mime) that is personal data. Resolved: §1 #16 + `files.jsonl` with metadata (not the file bytes; subjects request those separately) + AC #20.

### ISS-011 — Device tokens leaked (strict-redo pass)
Original spec didn't address whether device tokens (push credentials) belong in DSAR. Including them creates a credential-leak path. Resolved: §1 #17 + tokens explicitly excluded; only registration metadata included + AC #21.

### ISS-012 — Lumi interactions missing (strict-redo pass)
LLM interactions reveal subject's queries and the system's responses — recognised as personal data under recent AI-governance guidance. Resolved: §1 #18 + `lumi_interactions.jsonl` + AC #22.

### ISS-013 — Retro captures missing (strict-redo pass)
Memories created by or referencing the subject are personal data. Resolved: §1 #19 + `retro_captures.jsonl` + AC #23.

### ISS-014 — DSAR history missing (strict-redo pass)
Subjects have a meta-right to know who has accessed their data and when. Resolved: §1 #20 + `dsar_history.jsonl` + AC #24.

### ISS-015 — URL leak = full data leak (strict-redo pass)
A leaked S3 URL gives anyone full access to the DSAR. Resolved: §1 #21 + passphrase-protected zip with HKDF derivation from subject's email; salt in manifest; subject derives passphrase locally + AC #25.

### ISS-016 — Export-wide tamper-evidence missing (strict-redo pass)
Per-message anchors verify each message; without export-wide hash, modifying the section list (adding/removing sections) wouldn't be detectable. Resolved: §1 #22 + export_chain_hash in manifest + verify.sh recomputes + AC #26.

### ISS-017 — Sharding semantics incomplete (strict-redo pass)
Original sharding said "deliver multiple zips" but didn't specify acknowledgement chain across shards. Resolved: §1 #23 + per-shard delivery + `chat.dsar_fully_delivered` after all acked + AC #27.

### ISS-018 — No SLA tracking (strict-redo pass)
GDPR Art. 12(3) requires 30-day max; without SLA tracking, requests can sit forever. Resolved: §1 #24 + `sla_deadline` column + nightly check + SEV-1 at 25-day mark + AC #28.

### ISS-019 — No preview before full delivery (strict-redo pass)
Subjects sometimes refine scope after seeing what's included; without preview, they commit to full delivery. Resolved: §1 #25 + preview mode + confirm-preview endpoint + 7-day TTL on preview + AC #29.

### ISS-020 — No subject acknowledgement (strict-redo pass)
"Delivered" only means "URL sent." Legal compliance requires receipt confirmation. Resolved: §1 #26 + acknowledgement endpoint + `chat.dsar_acknowledged` audit + AC #30.

### ISS-021 — No tenant redact policy (strict-redo pass)
Tenants under HIPAA/PCI may need to exclude categories of PII from exports (even to the subject themselves, per regulation). Resolved: §1 #27 + `dsar_redact_categories` column + per-tenant policy + `redacted_categories.json` listing what was removed + AC #31.

## §3 — Resolution

All 21 mechanical concerns addressed. **Score = 10/10.**

Per AUTHORING.md §0 master rule: spec is now perfect — depth bounded by the genuine surface (9 data sections × authorisation matrix × chain-anchor tamper-evidence × URL one-time-use × passphrase encryption × preview-then-full × sharding × SLA tracking × subject acknowledgement × per-tenant redaction policy), not by line targets.

---

*End of FR-CHAT-012 audit.*
