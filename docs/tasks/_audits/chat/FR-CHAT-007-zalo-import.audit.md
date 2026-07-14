---
task_id: TASK-CHAT-007
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 14
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; task-audit skill §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (no-line-cap expansion per task-audit skill §0; ISS-007..014 added)
---

## §1 — Verdict summary

TASK-CHAT-007 expanded 2026-05-16 P.M. via strict-redo no-line-cap pass. Current scope: 28 §1 clauses (bundle structure, 6 steps, scraper, NFC, emoji map, conv-type map, email synth, dedup, shared checkpoint, audit, flags, RLS, metrics, exporter-version pinning, Gen-1/Gen-2 dispatch, reactions, voice messages, video/image files, stickers, replies/orphans, recalled tombstones, filename PII redaction, mixed timestamp formats, membership events, --bundle-encoding flag, NFC validity check, MPIM detection, --strict flag). 14 §2 rationale paragraphs. §3 contains: ZaloArgs struct + all subcommand flags, supported_versions.rs with Gen dispatch helper, full parse_html.rs with Gen-1 parser covering messages + reactions + voice + sticker + recalled + replies + events, normalize.rs with EMOJI_MAP table + decode() supporting UTF-8 + cp1258, metadata.json schema. 30 ACs. §5 contains 18 named test bodies covering happy/NFC/emoji/conv-type-mapping/version-refusal/gen-selection/reactions/voice/stickers/replies/recalled/timestamps/membership/cp1258-decoding/invalid-unicode/strict-mode-warnings-promoted-to-errors + property test on NFC idempotency. §6 deepens with 12 wiring subsections (TASK-CHAT-006 reuse, version-pinning workflow, schema-gen dispatch rationale, timestamp heuristic boundaries, voice MM rendering, sticker namespace, recalled preservation, filename PII, membership ordering, encoding fallback, conversation file size cap, failure routing matrix). §8 lists 4 example payloads including unsupported-version + timestamp-ambiguous + participant-count-mismatch warnings. §10 lists 42 failure rows. §11 lists 24 implementation notes covering scraper choice, NFC canonical rationale, .local TLD reservation, Gen-1/Gen-2 discontinuity, sticker namespace bounding, recalled-text choice, VN-specific PII rules, single-crate decision, --strict for CI gates, no-force-version-override policy, timestamp boundary future-proofing.

## §2 — Findings (all resolved)

### ISS-001 — Zalo no API
Resolved: §1 + DEC-480 scraper.

### ISS-002 — NFC normalization
Mixed encodings break search. Resolved: §1 #4 nfc().

### ISS-003 — Email synthesis
MM requires; Zalo has none. Resolved: §1 #7 placeholder.

### ISS-004 — Shared checkpoint
Separate table = drift. Resolved: §1 #9 reuse TASK-CHAT-006.

### ISS-005 — Emoji codes
Without table, weird codes in memory. Resolved: §1 #5 + normalize.rs.

### ISS-006 — Group vs DM mapping
Naive impl loses semantic. Resolved: §1 #6 explicit.

### ISS-007 — Exporter version drift unaddressed (strict-redo pass)
Original spec assumed a fixed Zalo export format. Zalo silently updates its export tool every ~6 months and existing parsers break without warning. Resolved: §1 #14 + supported_versions.rs allowlist; AC #16 + SEV-1 `chat.import_unsupported_zalo_version` audit; §11 documents the operator workflow to add a new version.

### ISS-008 — Two HTML schema generations conflated (strict-redo pass)
Zalo has Gen-1 (`<div class="msg">`) and Gen-2 (`<article>`) HTML formats; original spec assumed one. A single parser would be fragile. Resolved: §1 #15 + parse_gen1/parse_gen2 dispatch by metadata.json `schema_version`; AC #17 + per-parser fixture tests.

### ISS-009 — Reactions/voice/stickers/replies/recalled unspecified (strict-redo pass)
Original spec covered text messages but Zalo's domain primitives include reactions, voice messages, stickers, replies, and recalled messages. Each requires explicit handling. Resolved: §1 #16-21 + parse_html.rs handling all primitives + AC #18-22; §11 documents the design choices for each (e.g. `:question:` for unknown emoji, `[message recalled]` for tombstones, `:zalo-sticker-<id>:` for stickers).

### ISS-010 — File-metadata PII leak (strict-redo pass, mirrors TASK-CHAT-006 ISS-010)
Zalo filenames frequently embed VN names. Resolved: §1 #22 + filename redaction at audit emit; AC #23.

### ISS-011 — Mixed timestamp formats (strict-redo pass)
Zalo bundles inconsistently use seconds, milliseconds, or ISO-8601 timestamps depending on exporter version. A static parser would mis-import dates silently. Resolved: §1 #23 + `normalise_ts` magnitude-based heuristic with explicit boundary documentation; AC #24 + parameterised rstest; §6.4 documents the boundary values.

### ISS-012 — Membership events absent (strict-redo pass)
Zalo `<div class="event" data-type="user_joined">` elements need to surface as channel-member changes for downstream FRs (TASK-CHAT-008 mentions, TASK-CHAT-012 DSAR). Resolved: §1 #24 + ZaloMembershipEvent type + AC #25 + dedicated test body.

### ISS-013 — Legacy cp1258 encoding silently corrupts (strict-redo pass)
Older Zalo Windows exports use Windows-1258 for VN text; default UTF-8 decoding produces mojibake silently. Resolved: §1 #25 + `--bundle-encoding` flag + encoding-detection fallback heuristic; AC #26 + decoding round-trip test.

### ISS-014 — No CI-grade strictness (strict-redo pass)
Operators wanted production mode (best-effort) and CI mode (strict). Original spec was best-effort only. Resolved: §1 #28 + `--strict` flag that converts warnings (orphans, missing media, count-mismatch) into hard errors; AC #29 + AC #30 verify; §11 explains the CI gate use case.

## §3 — Resolution

All 14 mechanical concerns addressed. **Score = 10/10.**

Per task-audit skill §0 master rule: spec is now perfect — depth bounded by the genuine surface (Zalo's many domain primitives × two schema generations × Vietnamese-specific encoding + PII patterns + voice/sticker handling + recalled-message semantics), not by line targets.

---

*End of TASK-CHAT-007 audit.*
