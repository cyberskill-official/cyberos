---
fr_id: FR-SKILL-110
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; AUTHORING.md §3.12 compliant)
---

## §1 — Verdict summary

FR-SKILL-110 authored direct-to-10/10. ~920 lines. 16 §1 clauses (request shape, MST validation, monotonic numbering, XML composition, XSD validation, ed25519 signing, GDT submission with retry semantics, audit rows on success + failure, gap detection, replay, OTel, metrics, log redaction, optional PDF render). 10 §2 rationale paragraphs. Full Rust API + XML composer + ed25519 signer + GDT submitter in §3. 24 ACs. 4 XML tests + 5 integration tests. 22 failure modes. 9 implementation notes.

## §2 — Findings (all resolved during authoring)

### ISS-001 — Buyer ID flexibility (MST vs CCCD)
B2B uses MST; B2C uses personal CCCD. Without spec, callers wouldn't know which field to populate. Resolved: §1 #1 + #2 + #4 both fields supported; one required; AC #2 #3 #4.

### ISS-002 — Number reservation vs failure recovery
Advance counter AFTER submit = race; BEFORE submit = orphan numbers on failure. Resolved: §1 #3 + #8 reserve first; on permanent failure emit `vn.invoice_submission_failed` with the orphan number; gap detection flags it; operator files manual notice.

### ISS-003 — Idempotency for crash recovery
Network blip mid-submit could create duplicate hóa đơn (illegal). Resolved: §1 #12 + §3 replay::lookup; AC #16 verifies same-key returns prior outcome.

### ISS-004 — Signature canonicalisation
XML whitespace normalisation downstream breaks verify. Resolved: §1 #6 + §11 note: xml_c14n exclusive C14N before sign; ed25519 over canonical form.

### ISS-005 — Permanent vs transient GDT errors
Without distinction, transient 5xx burns retries on permanent 4xx. Resolved: §1 #8 + §3 `submit_with_retry` distinguishes by status class; 4xx returns immediately without retry; AC #13 #14 #15.

### ISS-006 — Gap detection vs silent loss
Lost invoice numbers (crash, GDT reject without counter rollback) violate Decree 123 Art. 11. Resolved: §1 #11 + DEC-234 monotonic counter + `numbering::check_consecutive` + sev-1 alarm; AC #17 + §10 numbering-gap row.

## §3 — Resolution

All 6 mechanical concerns addressed during authoring. **Score = 10/10.** Ready to ship + transition `draft → accepted`.

---

*End of FR-SKILL-110 audit.*
