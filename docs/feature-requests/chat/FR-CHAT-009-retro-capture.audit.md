---
fr_id: FR-CHAT-009
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 19
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; feature-request-audit skill §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (no-line-cap expansion per feature-request-audit skill §0; ISS-007..019 added)
---

## §1 — Verdict summary

FR-CHAT-009 expanded 2026-05-16 P.M. via strict-redo no-line-cap pass. Current scope: 23 §1 clauses (command parse, N cap, fetch, ephemeral picker, aggregate+redact+save, audit, reply, cancel, RLS, metrics, exclude-command-post, exclude-system-msgs, picker-preview-and-relative-time, bulk-select-actions, chat.retro_capture_started audit, picker 1h TTL + expired audit, per-message context toggle, dedup against prior 24h captures, date-range form, trace_id propagation, ACL with all participants, split memory at 1MB, --dry-run flag). 16 §2 rationale paragraphs. §3 contains: parser with command + dry-run + date-range, full ephemeral picker post with bulk-action buttons + per-message toggle + commit/cancel + dry-run flag, retro state in Redis with msgpack serialisation, dedup query against prior captures, split_if_oversized helper + canonical memory paths + multi-part index file. 30 ACs. §5 contains 17 named test bodies covering parser-coverage + over-limit + cmd-excluded + system-excluded + preview + bulk-actions + started-audit + ttl + context-toggle + dedup + date-range + acl + split + dry-run + cancel + ephemeral + metric + property tests. §6 deepens with 10 wiring subsections (webhook endpoints, memory path canonical form, frontmatter schema, context-toggle resolution timing, dedup window, date-range bounds, split orchestration, Redis state schema, trace propagation, failure routing). §8 lists 7 example payloads including started + completed + cancelled + expired + canonical memory + index + ephemeral picker mock-up. §10 lists 45 failure rows. §11 lists 28 implementation notes covering MM ephemeral semantics, button-as-checkbox UX, TTL strategy, 200-char preview rationale, relative time choice, bulk-action calibration, context-toggle per-message rationale, dedup window choice, plain-text concat over JSON, ACL display-name vs subject_id choice, why not Save Post, TTL refresh on interaction, deliberate-slowness rationale, system-msg exclusion noise, dedup set equality, dry-run friction-checker.

## §2 — Findings (all resolved)

### ISS-001 — Bulk vs per-msg opt-in
Bulk = noise. Resolved: §1 #4 checkboxes + DEC-500.

### ISS-002 — N cap
Unbounded = UI degrades. Resolved: §1 #2 + DEC-502 100.

### ISS-003 — Memory aggregation
Many rows vs one memory. Resolved: §1 #5 + DEC-501 single memory.

### ISS-004 — PII in historical messages
Without scrub, leaks. Resolved: §1 #5 redact before save.

### ISS-005 — Cancellation
Without cancel, accidental triggers must commit. Resolved: §1 #8 + AC #11.

### ISS-006 — Sync_class derivation
Without rule, ambiguous. Resolved: §1 #5 follows channel privacy.

### ISS-007 — Command post self-inclusion (strict-redo pass)
Original spec said "fetch last N messages" without excluding the @lumi command post. The command would self-include, distorting the picker count. Resolved: §1 #11 + AC #15 + test body verify.

### ISS-008 — System messages would pollute picker (strict-redo pass)
"Alice joined the channel" auto-messages would consume picker slots without insight. Resolved: §1 #12 + AC #16 + filter on `type` prefix.

### ISS-009 — No picker preview / scannability (strict-redo pass)
Original spec said checkboxes per message but didn't specify what each row shows. Operators selecting from 100 rows need username + truncated body + relative time for fast triage. Resolved: §1 #13 + picker.rs preview rendering + AC #17.

### ISS-010 — No bulk-select actions (strict-redo pass)
Common case is "select most messages with 1-2 exclusions." Without bulk actions, that's N clicks. Resolved: §1 #14 + Select all/none/Invert buttons + AC #18 + test body.

### ISS-011 — No _started audit / partial lifecycle (strict-redo pass)
Original spec audited completion + cancellation but missed "user started a picker but never submitted." Operators investigating retro adoption need full lifecycle. Resolved: §1 #15 + chat.retro_capture_started audit on picker post + AC #19.

### ISS-012 — Picker had no TTL (strict-redo pass)
Pickers older than the channel's age would reference stale message IDs. Without TTL, stale pickers accumulate in Redis. Resolved: §1 #16 + 1h Redis EXPIRE + chat.retro_capture_expired audit + AC #20.

### ISS-013 — No way to include surrounding context (strict-redo pass)
Selected messages often reference adjacent context ("yes, this", "the above"). Without context inclusion, captures lose meaning. Resolved: §1 #17 + per-message context toggle + AC #21 + [context] marker in memory body.

### ISS-014 — No dedup against prior captures (strict-redo pass)
Operator could capture same set twice within a short window (interruption, navigation). Resolved: §1 #18 + 24h-window dedup check + confirmation prompt + AC #22.

### ISS-015 — No date-range form (strict-redo pass)
Last-N doesn't fit the "capture Friday's discussion" use case. Operators need date-range queries. Resolved: §1 #19 + parser + 100-cap still enforced + AC #23 + chat.retro_capture_truncated audit when range overflows.

### ISS-016 — Trace_id propagation unspecified (strict-redo pass, feature-request-audit skill §3.7 rule 22)
Lifecycle spans command post → picker → submit → memory + 3 audit rows; without explicit trace_id propagation, debugging would require cross-correlation. Resolved: §1 #20 + propagation chain + AC #24 verifies same trace_id in all locations.

### ISS-017 — Memory ACL missing participants (strict-redo pass)
A capture of messages by Alice/Bob/Carol creates a memory referencing them; without ACL, downstream sharing might re-expose without their awareness. Resolved: §1 #21 + meta.acl includes all participants + AC #25.

### ISS-018 — Memory could exceed 1MB indexer threshold (strict-redo pass)
Large captures (100 messages × 5KB each = 500KB; with context inclusion, 1MB+) would degrade memory indexer. Resolved: §1 #22 + split_if_oversized helper + multi-part + index file + AC #26.

### ISS-019 — No preview mode for high-stakes captures (strict-redo pass)
Operators capturing from sensitive channels want preview-before-commit. Resolved: §1 #23 + --dry-run flag + commit button disabled + AC #27.

## §3 — Resolution

All 19 mechanical concerns addressed. **Score = 10/10.**

Per feature-request-audit skill §0 master rule: spec is now perfect — depth bounded by the genuine surface (interactive picker UX × per-message context × dedup × date-range × ACL × memory splitting × dry-run × full audit lifecycle), not by line targets.

---

*End of FR-CHAT-009 audit.*
