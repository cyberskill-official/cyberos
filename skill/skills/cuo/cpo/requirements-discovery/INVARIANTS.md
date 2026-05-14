# `requirements-discovery` self-audit invariants (scaffold)

> Truths the chain entry point MUST enforce about its own behaviour at runtime. Scaffold-only at v0.1.0; runtime engine in v0.3.0.

## Invariants

### INV-001 — BRAIN must be reachable

**Statement.** The skill MUST refuse to proceed past `CONTRACT_ECHO` if BRAIN is unreachable. Discovery without BRAIN is worse than no discovery — the resulting brief would invent context the company has no record of.

**Check.** Before phase 2 (triage), execute one canary `brain.search` against `company:locked-decisions` with a known query. If no response within timeout (5s) OR error response → halt with `BOOT-005`.

**Severity.** `error` (sev-0).

**Refinement template.**
```
trigger: INV-001 breach: BRAIN unreachable for trace_id {trace_id}
observation: canary query returned {error_code}; discovery proceeding without BRAIN data is forbidden
proposed_amendment_target: cyberos/docs/skills/cuo/cpo/requirements-discovery/SKILL.md
proposed_amendment_section: §"Failure modes" BOOT-005
proposed_diff: |
  +  Add a BRAIN-degraded mode: with explicit user consent, allow discovery
  +  to proceed citing "BRAIN unavailable; proceed with chat-only data" as
  +  a recorded provenance note. ONLY for non-client_visible projects.
minimum_viable: "Document the degradation path; never silent-fall-back."
```

### INV-002 — interview-completion threshold

**Statement.** The brief MUST NOT be written if fewer than 12 of the 20 interview questions have answers. Writing a partial brief makes downstream skills consume incomplete intake; better to halt and surface.

**Check.** Before phase 5 (synthesise), count answered questions. If <12, set `triage_verdict: revise`, write `## Open Questions` listing every unanswered question, halt with HITL.

**Severity.** `error`.

### INV-003 — triage-rejected briefs flagged downstream

**Statement.** A brief with `triage_verdict ∈ {revise, reject}` MUST carry an explicit `## Triage Reasoning` section AND the output envelope's `next_skill_recommendation` MUST be null (no chain to prd-author).

**Check.** Before write, validate frontmatter `triage_verdict` field; if `revise` or `reject`, validate body has `## Triage Reasoning` H2 with non-empty content; validate envelope payload's `next_skill_recommendation` is null.

**Severity.** `error`.

### INV-004 — authority markers on Goals

**Statement.** Every numbered item in `## Goals` MUST carry an inline `<!-- authority: ... -->` marker. No bare goals.

**Check.** Regex against the brief body — every line matching `^\d+\. ` MUST be preceded by `<!-- authority: (human-edited|human-confirmed|llm-explicit|llm-implicit) -->`.

**Severity.** `error`.

### INV-005 — scope discipline

**Statement.** No `write_file` lands outside `output_dir`. The brief is a sibling of any other briefs in `output_dir/`; no nested folders, no writes to BRAIN scopes outside the declared `allowed_brain_scopes.write` list.

**Check.** Walk audit rows of `op:create` or `op:str_replace` written by this skill; every `path` is under `output_dir` OR matches `^\.cyberos-memory/(project|memories/projects)/.*$`.

**Severity.** `error`.

### INV-006 — BRAIN read budget enforcement

**Statement.** Phase 4 (BRAIN-targeted reads) MUST NOT exceed 10 queries OR 50 returned memories. Unbounded reads turn discovery into noise + cost-blow.

**Check.** Track query count + cumulative returned memory count during phase 4. If either threshold is crossed, halt phase 4 + flag in audit row + proceed to phase 5 with whatever was collected (do NOT block discovery on a budget breach — just record it).

**Severity.** `warning` (advisory; soft cap, hard cap is the supervisor's MCP rate-limit).

## Adding a new invariant

Same procedure as fr-author + fr-audit + fr-to-tech-spec. The author + persona steward (cpo) propose; registry maintainer reviews; an acceptance test is added; the skill MINOR-bumps.
