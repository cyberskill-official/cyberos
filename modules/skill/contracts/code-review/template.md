---
template: code-review@1
title: PR #<N> — <description>
pr_url: https://github.com/<org>/<repo>/pull/<N>
pr_number: <N>
pr_size_loc: <integer>
reviewer: @<reviewer>
reviewed_at: 2026-MM-DDTHH:MM:SS+07:00
linked_impl_plan: ./impl-plan.md
ai_assisted: false    # set true if any portion of the PR is AI-generated
provenance: { source_path: <pr diff path or sha>, source_hash: sha256:<hash> }
verdict: approved    # approved | request_changes | approved_with_conditions | blocked
---

# PR #<N> — <description>

## 1. Correctness vs Ticket
Does the diff implement the linked task? <Y/N + notes>.

## 2. Readability
<naming, comments, structure>.

## 3. Test Coverage
New code covered? Coverage % vs DoD threshold? <numbers>.

## 4. Secrets / Credentials
| secret_scan_tool: <e.g. gitleaks> | result: <clean | findings + status> |

## 5. Injection Surfaces
<SQL / command / template injection paths introduced?>

## 6. Input Validation
<boundary conditions, type checks, allowlist over denylist>.

## 7. Error Handling
<failure paths, no swallowed exceptions>.

## 8. Logging
<no PII; consistent log levels; trace correlation>.

## 9. Performance Considerations
<N+1, large allocations, hot paths>.

## 10. Backwards Compatibility
<API contract preserved; migration path if breaking>.

## 11. SAST / SCA Results
| tool | findings (high) | status |
|---|---|---|

## 12. SBOM Impact
<dependency additions / removals / version changes>.

<!-- ── AI-specific sections (REQUIRED when ai_assisted: true) ── -->
<!-- ## 13. AI-Generated Code Review     — tools used, scope, human-verification done -->
<!-- ## 14. Hallucinated-API Check       — every imported symbol verified in declared dependency -->
<!-- ## 15. Oversized-Diff Check         — diffs >500 LOC require explicit rationale -->
<!-- ## 16. Dependency-Addition Provenance — per OWASP A03 -->
<!-- ## 17. PR Label Verification        — ai-assisted: yes label applied -->

<!-- ── Other conditional sections ── -->
<!-- ## 18. Migration Review            — when diff touches DB migration -->
<!-- ## 19. Security Review             — when diff touches auth/crypto -->
<!-- ## 20. API Contract Diff           — when diff touches public API surface -->
<!-- ## 21. Privacy Review              — when diff touches personal data -->
<!-- ## 22. Conditions to Resolve Pre-Merge — when verdict: approved_with_conditions -->
