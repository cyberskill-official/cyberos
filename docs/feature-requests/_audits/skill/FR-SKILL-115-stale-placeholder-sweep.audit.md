---
fr_id: FR-SKILL-115
audited: 2026-05-19
verdict: PASS (after revision)
score_pre_revision: 7.0/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 7
template: engineering-spec@1
authoring_md_compliance: 2026-05-19 (per feature-request-audit skill §3.12 — 7 canonical ISSes verified; placeholder substitution discipline + persona-batch commit rationale explicitly captured)
---

## §1 — Verdict summary

FR-SKILL-115 authored direct-to-10/10 with one mid-loop expansion (persona-batch commit discipline added after recognising 134-file commits are unreviewable). ~720 lines. 15 §1 normative clauses (detect + suggest + report + auditor rule + Python validator + persona batches + body XML preservation + audit-chain compat + commit attestation + CI gate + _template exemption + verify script + report committed + registry bump). 11 §2 rationale paragraphs. Full Python detect + suggest skeletons + auditor rule + 5 example payloads in §3. 20 numbered ACs. 4 pytest functions + manual verification cases. 13 failure modes. 7 implementation notes.

## §2 — Findings (all resolved during authoring)

### ISS-001 — Conflation risk with FR-SKILL-113
First draft of §1 #1 said "stale XML brackets" — could be confused with FR-SKILL-113's now-fixed `wrap_in:` field. **Resolved:** §1 #1 explicitly excludes `wrap_in_marker:` from detection scope; §2 first paragraph documents the split between 113 (one mechanical pattern) and 115 (dozens of distinct fields, operator-attested substitution); detect.py skips fields already in the new string-form.

### ISS-002 — Auto-substitution risk
First draft hinted at automated batch substitution. Risk: pattern-matching gets 30-50% wrong because the right value depends on body context. **Resolved:** §1 #4 explicitly forbids auto-substitution; §2 second paragraph quantifies the error rate; the suggest.py output is advisory only — operator review + edit required.

### ISS-003 — _template/ exemption was implicit
Draft didn't explicitly exempt `_template/author/SKILL.md` and `_template/audit/SKILL.md`. Risk: detector flags scaffolds as production-broken; operator wastes time substituting. **Resolved:** §1 #12 + §2 paragraph 7 + detect.py EXEMPT_PATHS prefix list + AC #2 verifies. Templates are contract artefacts, not stale leakage.

### ISS-004 — 134-file mega-commit unreviewability
Original sweep timing said "one atomic commit batch". Risk: 134-file diff is unreviewable; PR review delay > sweep value. **Resolved:** §1 #7 mandates persona-grouped batches (P0 cpo+cto first; then P1 personas; then P2+); §2 third paragraph documents reasons (reviewability + risk isolation + schedule fit); commit-message format prescribed.

### ISS-005 — Commit message rationale discipline
Draft didn't require per-substitution rationale in commit messages. Risk: 6 months later, operators can't tell WHY a substitution was chosen — re-derivation cost. **Resolved:** §1 #10 mandates operator-attested commit message with one-line rationale per field-type; §2 paragraph 5 documents the chain-of-custody value; example commit format in §3.

### ISS-006 — Body XML preservation invariant
Draft §1 #8 was added on second loop after re-reading FR-SKILL-113. Risk: sweep accidentally touches body markdown `<untrusted_content>...</untrusted_content>` blocks. **Resolved:** §1 #8 explicit prohibition; §10 failure mode "Sweep accidentally touches body XML" + verify.py SHA256 invariant. AC #10 + AC #14 verify byte-identity of body bytes.

### ISS-007 — Multi-stage skill ambiguity
Draft suggest.py picked the "most frequent" stage letter. Risk: cross-cutting skills (e.g. retrospective-author spans b/d/e) get a wrong single-stage tag. **Resolved:** §10 failure mode "Multi-stage skill ambiguity" + suggest.py logic: when 2+ stages tied or close, recommend "cross"; operator overrides as needed.

## §3 — Resolution

All 7 mechanical concerns addressed during authoring. **Score = 10/10.** Ready to ship + transition `draft → accepted`.

Cross-FR sanity check:
- `depends_on: [FR-SKILL-113]` — reciprocal; FR-SKILL-113's `blocks:` list updated in the housekeeping commit that batches 111/112/113/114/115 reciprocity.
- `related_frs:` enumerates FR-SKILL-111 / 112 / 113 / 114 — all four shipped this session.
- `priority: SHOULD` reflects portability concern but non-blocking for CyberOS internal use (only hits when Phase-B transpilers ship).
- Registry v0.2.6 increment is documented in §1 #15 + §11 last bullet.

---

*End of FR-SKILL-115 audit.*
