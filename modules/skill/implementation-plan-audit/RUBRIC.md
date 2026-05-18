# `impl_plan_rubric@1.0` — machine-checkable Implementation Plan rubric

> Sourced from `cyberos/skill/contracts/implementation-plan/CONTRACT.md` and `cyberos/docs/Software Development Process.md` §2(f) Implementation; DORA findings on small-batch discipline + AI-assisted code review. Rubric version `1.0` is locked.

---

## §1  Frontmatter — structural

| rule_id | Check | Severity | Auto-fixable |
| ------- | ----- | -------- | ------------ |
| `FM-001` | YAML parses; closing `---` present | error | false |
| `FM-002` | All keys are `snake_case` | error | true |
| `FM-003` | No duplicate keys | error | false |
| `FM-004` | `template` equals `implementation-plan@1` | error | true |

## §2  Frontmatter — per-field

| rule_id | Field | Rule | Severity | Auto-fixable |
| ------- | ----- | ---- | -------- | ------------ |
| `FM-101` | `title` | required, string | error | skeleton |
| `FM-102` | `plan_version` | required, SemVer | error | true |
| `FM-103` | `linked_fr` | required, resolves to an FR that passed feature-request-audit at 10/10 | error | false |
| `FM-104` | `linked_sdd` | recommended; if present, resolves to an SDD that passed software-design-document-audit | warning | false |
| `FM-105` | `target_sprint` | required, string (sprint identifier per project convention) | error | false |
| `FM-106` | `target_proj_backend` | required, one of: linear, jira, github_projects, monday, asana, none | error | false |
| `FM-107` | `provenance.source_path`, `provenance.source_hash` | required | error | false |
| `FM-108` | `total_estimate_pts` | required, integer (sum of per-task estimates) | error | false |
| `FM-109` | `author` | required, matches `^@[A-Za-z0-9_.-]{1,38}$` | error | false |
| `FM-110` | `reviewer` | required (the engineer who will code-review the implementation) | error | false |

## §3  Always-required sections

| rule_id | Heading | Severity |
| ------- | ------- | -------- |
| `SEC-001` | `## 1. Summary` (1-2 sentences — what this plan implements) | error |
| `SEC-002` | `## 2. Tasks` (one row per ticket; columns: ticket_id, title, estimate_pts, owner, blocked_by, acceptance_link) | error |
| `SEC-003` | `## 3. Branch and PR Strategy` (trunk-based / short-lived; PR size cap) | error |
| `SEC-004` | `## 4. Test Strategy` (which test types apply at impl time — unit + integration mandatory; reference test-strategy artefact for the system) | error |
| `SEC-005` | `## 5. Observability` (logs / metrics / traces hooks the impl will add) | error |
| `SEC-006` | `## 6. Rollout` (feature-flag plan, dark-launch strategy if applicable) | error |
| `SEC-007` | `## 7. Risks and Mitigations` (impl-specific risks: data migration, breaking change, perf regression) | error |
| `SEC-008` | `## 8. AI Tool Usage` (per SDP §5 — which AI tools will be used during impl + AI-assisted PR labelling commitment) | error |
| `SEC-901` | Each required section is non-empty | error |

## §4  Conditionally-required sections

| rule_id | Trigger | Required | Severity |
| ------- | ------- | -------- | -------- |
| `COND-001` | `total_estimate_pts > 13` (i.e. larger than a single sprint item) | `## 9. Decomposition Rationale` explaining why the work isn't split into multiple impl-plans | warning → needs_human (`scope_decomposition`) |
| `COND-002` | Plan involves a database schema change | `## 10. Migration Strategy` covering forward + backward compatibility window + rollback | error |
| `COND-003` | Plan touches public API surface | `## 11. API Versioning Impact` (per SDD's `api_versioning_policy`) | error |
| `COND-004` | Plan touches security boundary | `## 12. Security Review Checklist` (links to relevant threat-model entries) | error |
| `COND-005` | Plan is AI-implementation-heavy (≥30% expected AI authorship) | `## 13. AI-Generated Code Review Plan` per SDP §5.5 — mandatory human review + SAST/SCA in PR + SBOM check + AI-assisted PR label | error |

## §5  Quality heuristics

| rule_id | Anti-pattern | Detection | Severity |
| ------- | ------------ | --------- | -------- |
| `QA-CITE-001` | Claim without `source_ref` | non-boilerplate paragraph lacks `source_ref` | error |
| `QA-AUTH-001` | Paragraph without `authority` marker | non-boilerplate paragraph lacks `authority:` | error |
| `QA-TASK-001` | Task without acceptance link | A row in §2 lacks `acceptance_link:` referencing a specific FR section or SDD interface | error |
| `QA-TASK-002` | Task without owner | error |
| `QA-TASK-003` | Task estimate is null or zero | error |
| `QA-TASK-004` | Cyclic blocked_by | warning → needs_human |
| `QA-BATCH-001` | DORA small-batch violation | Sum of task estimates >40 pts (heuristic — humans may override with rationale in §9) | warning (per DORA 2024 finding that AI-assisted work inflates batch sizes) |
| `QA-TEST-001` | Test strategy lacks unit-test mention | §4 lacks any reference to unit tests | error |
| `QA-OBS-001` | Observability section empty of concrete hooks | §5 has no concrete log/metric/trace name | warning |
| `QA-FLAG-001` | Rollout without feature-flag plan when behavior changes user-facing | §6 lacks `feature_flag:` field for user-facing changes | warning |
| `QA-AI-001` | AI tool usage missing concrete tools | §8 says "AI may be used" without naming specific tools (Claude Code / Cursor / Copilot / etc.) | error |
| `QA-AI-002` | COND-005 fires but AI-assisted PR label commitment missing | §13 missing the PR-label commitment | error → needs_human |
| `QA-TODO` | Skeleton TODO marker remaining | warning |
| `QA-QUOTE-001` | Quote outside `<untrusted_content>` | warning |

## §6  Untrusted-content safety

| rule_id | Check | Severity |
| ------- | ----- | -------- |
| `SAFE-001` | Nested `<untrusted_content>` | error |
| `SAFE-002` | Unclosed `<untrusted_content>` at EOF | error |
| `SAFE-003` | Injection-marker scan | warning (error if ≥3) |
| `SAFE-004` | Second-person commands outside `<untrusted_content>` | warning |

## §7  Cross-skill rules

| rule_id | Check | Severity |
| ------- | ----- | -------- |
| `XCHAIN-001` | `provenance.source_path` matches author manifest | warning |
| `XCHAIN-002` | `provenance.source_hash` matches at write time | error |
| `XCHAIN-003` | `linked_fr` resolves to an FR that passed feature-request-audit at 10/10 (else block plan) | error |
| `XCHAIN-004` | `linked_sdd` (if present) resolves to an SDD that passed software-design-document-audit at 10/10 | warning |
| `XCHAIN-005` | Every `acceptance_link` in §2 tasks resolves to a specific FR section or SDD interface | warning |

## §8  Staleness

| rule_id | Trigger | Action | Severity |
| ------- | ------- | ------ | -------- |
| `STALE-001` | Source FR / SDD hash differs | Reset open + needs_human to open | warning → needs_human |
| `STALE-002` | `target_sprint` has passed and plan status not yet `shipped` or `wontfix` | Surface as overdue | warning |

---

## Cross-references

- `cyberos/skill/docs/AUDIT_LOOP.md`, `cyberos/skill/docs/RUBRIC_FORMAT.md`
- `cyberos/skill/contracts/implementation-plan/CONTRACT.md`
- `cyberos/docs/Software Development Process.md` §2(f) — Implementation stage source
- `cyberos/docs/Software Development Process.md` §5 — AI integration source (drives §8 and COND-005)
- DORA 2024 — small-batch discipline rationale for QA-BATCH-001
