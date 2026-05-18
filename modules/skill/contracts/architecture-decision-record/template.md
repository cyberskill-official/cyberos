---
template: architecture-decision-record@1
title: <Short imperative — e.g. "Use CockroachDB for multi-region OLTP">
adr_id: ADR-0001
status: proposed    # proposed | accepted | deprecated | superseded
# supersedes:    [ADR-0007]
# superseded_by: ADR-0042
decision_date: 2026-MM-DD
decided_by:
  - { handle: "@<ar-handle>", role: "Architect" }
  - { handle: "@<tl-handle>", role: "TL" }
provenance: { source_path: ./srs.md, source_hash: sha256:<hash> }
linked_srs_reqs: [REQ-AUTH-001, REQ-PERSIST-003]
iso_25010_impacted_chars: [performance_efficiency, maintainability, security]
---

# ADR-0001: <Short imperative>

## 1. Context

<!-- authority: human-edited --> <Forces in tension; problem to solve. Cite source spec lines in surrounding HTML comments.>

## 2. Options Considered

### Option A: <Name>
- **Pros:** ...
- **Cons:** ...

### Option B: <Name>
- **Pros:** ...
- **Cons:** ...

### Option C: Do nothing
- **Pros:** ...
- **Cons:** ...

## 3. Decision

We will <Option X>.

## 4. Consequences

- **Positive:** ...
- **Negative:** ...
- **Neutral:** ...

## 5. Compliance / Quality Impact

Maps to `iso_25010_impacted_chars`:
- **performance_efficiency:** <impact>
- **maintainability:** <impact>
- **security:** <impact>

## 6. Notes / References

- <Link to RFC / prior ADR / benchmark / spike-output>

<!-- ── Conditionally-required sections (uncomment per COND-001..004) ── -->
<!-- ## 7. Security Impact          — when decision touches auth / crypto / data flow / attack surface -->
<!-- ## 8. Data Residency Impact    — when decision touches personal data / cross-border flow -->
<!-- ## 9. Reversal Cost Estimate   — when decision is one-way-door -->
<!-- ## 10. Why Superseded          — when status: superseded -->
