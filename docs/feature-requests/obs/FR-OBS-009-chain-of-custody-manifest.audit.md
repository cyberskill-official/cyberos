---
fr_id: FR-OBS-009
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

FR-OBS-009 expanded from 170 lines to ~810. Added 6 §1 clauses (#3 public key on CDN, #7 standalone verify_manifest binary, #8 deterministic canonical JSON, #10 PDF cover + JSON sidecar pairing, #11 metrics, expanded #1 with full field list). 7 §2 rationale paragraphs. Full Rust types + signing + verifier binary + PDF render in §3. 17 ACs. 8 full Rust test bodies. 16 failure modes. 9 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — Standalone verifier binary missing
First-pass had no offline verifier. Auditor would depend on CyberOS infra for verification — defeats independent-verification principle. Resolved: §1 #7 + `bin/verify_manifest.rs` with `--pubkey` for fully-offline use; AC #9 + #10.

### ISS-002 — Public key distribution unspecified
First-pass §1 #3 mentioned "include the public key" without distribution mechanism. Auditors couldn't fetch independently. Resolved: §1 #3 + CDN at keys.cyberos.world; verifier auto-fetches OR accepts local file; AC #15.

### ISS-003 — Canonicalisation rule unspecified
First-pass had no JSON canonicalisation rule. Same data could produce different signatures. Resolved: §1 #8 RFC 8785 JCS; serde-jcs crate; AC #2 + §5 verify-test.

### ISS-004 — PDF + JSON sidecar pairing unspecified
First-pass §1 #4 said "PDF cover + JSON sidecar" without format. Resolved: §1 #10 + zip pairing + QR code + manifest_pdf.rs render; AC #7 + #8.

### ISS-005 — Quarterly key rotation handling
First-pass §10 mentioned "Public key rotation in flight" but no spec. Resolved: §1 #2 + key version overlap; quarterly cadence; CDN serves both keys during overlap; AC #17.

### ISS-006 — `state: Incomplete` enforcement mechanism unspecified
First-pass §1 #6 said "shouldn't be trusted" without verifier failing on Incomplete. Resolved: §3 verifier `if state != Complete: exit 1`; AC #6 asserts.

## §3 — Resolution

All 6 mechanical revisions applied. **Score = 10/10.**

---

*End of FR-OBS-009 audit.*
