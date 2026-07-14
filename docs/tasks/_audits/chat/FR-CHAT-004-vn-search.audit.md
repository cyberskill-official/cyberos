---
task_id: TASK-CHAT-004
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 6.0/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 6
revision_history:
  - 2026-05-16 first pass: shipped in "compact form" (243 lines) with §6-§11 collapsed to one paragraph; explicit truncation marker "abridged for brevity due to space budget"
  - 2026-05-16 second pass: expanded to canonical 11-section form (≈900 lines); compact-form marker removed; full §6-§11 substance restored per task-audit skill master rule
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; task-audit skill §3.12 compliant)
---

## §1 — Verdict summary

TASK-CHAT-004 originally shipped in compact form during a context-budget pressure point; the audit at that time scored it 10/10 with an explicit caveat ("compact form due to context budget"). On creation of the `task-audit` skill master rule (2026-05-16), this FR was identified as the catalogue's only canonical-template violation. Re-authored to full form on the same date.

**Current state:** ~900 lines. 15 §1 clauses (PGroonga setup, 2 PL/pgSQL functions, 2 indexes, plugin route interception, hybrid routing logic, RLS filter, pagination, ordering, latency budget, fixture corpus, recall CI gate, audit row, OTel metrics, rate limit, debug CLI). 9 §2 rationale paragraphs. Full SQL migrations + Go plugin + Python recall script + CI workflow + fixture format in §3. 27 ACs. 11 Go unit + integration tests + 1 bash test for CI gate. 18 failure modes. 10 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — Truncated original (violates task-audit skill §0 master rule)
First-pass FR was 243 lines with §6-§11 collapsed to one paragraph and explicit "abridged for brevity due to space budget" caveat. This is the canonical anti-pattern the task-audit skill master rule was written to prevent. Resolved: full re-authoring; all 11 sections restored; ≈900 lines.

### ISS-002 — §1 was a summary paragraph, not numbered MUST clauses
Original §1 was prose ("PostgreSQL extension + custom Vietnamese bigram tokeniser..."). Without numbered clauses, callers can't cite specific contract surface; reviewers can't grep for "MUST". Resolved: 15 numbered §1 clauses each carrying single-sentence MUST or SHOULD.

### ISS-003 — Missing dependency on PGroonga extension state
Without explicit "PGroonga MUST be installed via the TASK-CHAT-003 Terraform module" handshake, an operator deploying CHAT to a vanilla RDS instance would hit `relation "pgroonga" does not exist` errors. Resolved: §7 names TASK-CHAT-003 + §10 first failure-mode row + AC #1.

### ISS-004 — Audit row exposed raw query (PII leak)
Original §3 audit payload had `query: "<raw text>"`. Operators searching for sensitive content (customer names, financial terms) would have it persisted in chain. Resolved: §1 #12 + §3 + §8 + AC #20 changed to `query_hash` (SHA-256); raw query never persisted.

### ISS-005 — `IMMUTABLE PARALLEL SAFE` markers absent on PL/pgSQL functions
Without these markers, the partial index `WHERE detect_vn(message)` would be REJECTED by PostgreSQL planner (predicate functions in partial indexes MUST be IMMUTABLE). The original SQL omitted markers. Resolved: §3 + §11 first note; AC #3 #5 implicitly cover.

### ISS-006 — `bigram_split` byte vs rune handling unclear
PL/pgSQL `substr(input, i, 2)` is char-aware (good), but the Go mirror function MUST match. Without specifying rune-awareness explicitly, an implementer could ship a Go bytewise function that diverges on multi-byte chars ("đ" is 2 bytes UTF-8). Resolved: §3 Go function uses `[]rune(strings.ToLower(s))`; AC #5 + #6 + #7 + dedicated TestBigramSplit_MultiByte; §11 note 2 documents the sync requirement.

## §3 — Resolution

All 6 mechanical concerns addressed via full canonical-form re-authoring. **Score = 10/10.** The lesson learned (no truncation) is now codified in `task-audit` skill §0 (Master Rule); this FR's revision history is the case study for future authors.

---

*End of TASK-CHAT-004 audit.*
