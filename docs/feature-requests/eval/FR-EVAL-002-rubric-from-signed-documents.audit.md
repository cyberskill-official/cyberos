---
fr_id: FR-EVAL-002
audited: 2026-06-29
verdict: PASS
score: 10/10
template: engineering-spec@1
authoring_md_compliance: 2026-06-29 (≥6 canonical ISSes verified; feature-request-audit skill §3.12 compliant)
strict_redo_pass: 2026-06-29 (no-line-cap expansion per feature-request-audit skill §0; ISS-001..012 resolved)
---

## §1 — Verdict summary

FR-EVAL-002 turns the three signed employment documents (Labor Contract, NDA/non-compete/IP, Total Rewards & Career Path Appendix — bilingual VN/EN, dated 2026-01-01, under Labor Code 45/2019/QH14 + Decree 145/2020) into a structured, versioned, clause-cited rubric that FR-EVAL-003 evaluates evidence against, with a human approving every item before it is effective.

Scope: 15 §1 clauses (three tables `rubric`/`rubric_version`/`rubric_item`; per-item clause citation `source_doc`+`clause_ref`+verbatim bilingual quotes; `item_kind`/`obligation_kind` classification; closed `check_type` + typed `check_params` + `weight`; full VN/EN bilingual with `_vi` required; versioned + immutable published cuts; `resolve_effective(at)`; the HITL human-approver gate; the GENIE draft-only path with anti-fabrication `needs_clause_ref`; FR-EVAL-001 access-gating; per-mutation hash-chained audit rows; the authoring + read API; publish-time coherence checks; standard-vs-judgement separation; OTel metrics). 11 §2 rationale paragraphs. §3 carries the migration (three tables + RLS + REVOKE-on-published), the Rust model with the five closed enums and `validate_item`, `resolve_effective` + `publish_version` (HITL), and the GENIE `draft_from_documents` (proposes-only). 18 ACs. §10 lists 22 failure rows. §11 lists 13 implementation notes. The `## AI Risk Assessment` section is present (high-risk class) with risk-classification, data-sources, human-oversight, traceability, and failure-modes subsections, naming EU AI Act Article 14 for the human-approval gate.

Frontmatter conforms to engineering-spec@1 (the FR-PROJ-008 shape): id FR-EVAL-002; module EVAL; priority MUST; status draft (per STATUS-REFERENCE.md — author has started the spec, not yet through the build queue); verify T; phase P3; milestone/slice present; depends_on [FR-EVAL-001]; blocks [FR-EVAL-003]; eu_ai_act_risk_class high; language rust 1.81; service cyberos/services/eval/; new_files/modified_files lists; allowed_tools/disallowed_tools encode the four DEC guardrails (clause-cited items, HITL approver, immutable published versions, governance-first); source_decisions DEC-2600..2604 capture Stephen's 2026-06-29 directions; risk_if_skipped present.

## §2 — Findings (all resolved)

### ISS-001 — Clause grounding could be optional
A rubric item without a citable clause is an opinion, not a measurable standard. Resolved: §1 #2 makes `source_doc`+`clause_ref` NOT NULL with a 422 on absence; AC #2 #3; `validate_item` enforces it; the `source_doc` CHECK closes the set to the three documents.

### ISS-002 — Model could set the operative standard
If GENIE's reading became effective with no human sign-off, the model would set the bar it later scores people against. Resolved: §1 #8 #9 — GENIE writes `state='draft'` only and has no publish path; `publish_version` rejects a non-human approver (403); AC #10 #12; EU AI Act Article 14 cited in the AI Risk Assessment.

### ISS-003 — Model could fabricate a citation
A model asked to cite will invent a plausible clause that may not exist (the obs-triage local-model fabrication precedent). Resolved: §1 #9 `needs_clause_ref` flag + §1 #13 publish coherence refuses an uncited item; AC #13; §11 anti-fabrication note.

### ISS-004 — Amending a contract would rewrite the past
Mutable config would silently change the standard every past assessment was measured against. Resolved: §1 #6 immutable published versions + re-curation makes `version_no+1` + REVOKE UPDATE/DELETE; §1 #7 `resolve_effective(at)`; AC #7 #8 #9.

### ISS-005 — Standard and judgement could blur
Storing per-person scores here would make the rubric person-specific and unfair. Resolved: §1 #14 forbids any per-employee/score/evidence column; AC #17; per-person scoring is FR-EVAL-003 against `rubric_version_id`.

### ISS-006 — Authoring before governance
The rubric is the most sensitive artifact; authoring it before consent/access/retention exists is unsafe. Resolved: hard `depends_on [FR-EVAL-001]` (DEC-2601); §1 #10 access-gates authoring + reads by FR-EVAL-001 grants; AC #14; §10 dependency-gate row.

### ISS-007 — Bilingual obligation under VN law
The documents are bilingual and Vietnamese is the operative language; an English-only rubric would not be readable in the language signed. Resolved: §1 #5 requires `_vi`/`_en` pairs with `_vi NOT NULL`; AC #6; §11 primary-language note.

### ISS-008 — Evaluation engine needs a machine-usable check
A free-form check would push per-item interpretation into FR-EVAL-003. Resolved: §1 #4 closed `check_type` enum + typed `check_params` validated by shape; AC #5; §11 bounded-logic note.

### ISS-009 — Curation history must be tamper-evident
The rubric's provenance (who proposed, who approved, when published/superseded) is itself evidence. Resolved: §1 #11 emits `eval.rubric_{drafted,edited,approved,published,superseded}` hash-chained into `l1_audit_log` (FR-PROJ-008 / FR-MEMORY-123 pattern); AC #15; §8 example payload.

### ISS-010 — A half-finished version could become operative
Publishing an incoherent version corrupts every assessment that runs against it. Resolved: §1 #13 publish-time coherence (every item cited + bilingual + valid check shape, non-empty, no effective-date overlap); AC #16; `assert_version_coherent` + `assert_no_effective_overlap`.

### ISS-011 — Obligation families must be explicit
The NDA's three obligation families (confidentiality, non-compete, IP-assignment) need to be first-class so they can be measured distinctly. Resolved: §1 #3 `obligation_kind` required for obligation items; AC #4.

### ISS-012 — Effective-date gaps/overlaps
Adjacent versions could leave a day uncovered or doubly-covered. Resolved: §11 half-open `[effective_from, effective_to)` intervals + the publish-time overlap guard; AC #16 (409 on overlap); §10 overlap rows.

## §3 — Resolution

All twelve concerns resolved. The spec's depth is bounded by the genuine surface — clause-grounding × versioned-immutability × the HITL approval gate × the GENIE draft-only-with-anti-fabrication path × FR-EVAL-001 governance-gating × tamper-evident curation audit — not by a line target. The four Stephen-2026-06-29 decisions (governance first; GENIE drafts and a human approves; versioned + effective-dated; clause-cited) are each encoded in a normative clause, a decision record, an acceptance criterion, and a disallowed-tools guardrail. **Score = 10/10.**

---

*End of FR-EVAL-002 audit.*
