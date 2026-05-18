# `sdd_rubric@1.0` — machine-checkable Software Design Description rubric

> Sourced from `cyberos/docs/Software Development Process.md` §2(e) Detailed design; IEEE 1016-2009 (Recommended Practice for Software Design Descriptions); arc42 §5-§10. Rubric version `1.0` is locked.

---

## §1  Frontmatter — structural

| rule_id | Check | Severity | Auto-fixable |
| ------- | ----- | -------- | ------------ |
| `FM-001` | YAML parses; closing `---` present | error | false |
| `FM-002` | All keys are `snake_case` | error | true |
| `FM-003` | No duplicate keys | error | false |
| `FM-004` | `template` equals `software-design-document@1` | error | true |

## §2  Frontmatter — per-field

| rule_id | Field | Rule | Severity | Auto-fixable |
| ------- | ----- | ---- | -------- | ------------ |
| `FM-101` | `title` | required, string | error | skeleton |
| `FM-102` | `component_or_system` | required, string | error | false |
| `FM-103` | `sdd_version` | required, SemVer | error | true |
| `FM-104` | `linked_srs` | required, resolves to an SRS that passed software-requirements-specification-audit at 10/10 | error | false |
| `FM-105` | `linked_adrs` | required, ≥1; each resolves to an ADR with `status: accepted` | error | false |
| `FM-106` | `provenance.source_path`, `provenance.source_hash` | required | error | false |
| `FM-107` | `created_at` | required, ISO 8601 | error | true |
| `FM-108` | `author` | required, matches `^@[A-Za-z0-9_.-]{1,38}$` | error | false |
| `FM-109` | `api_versioning_policy` | required, one of: url_path, header, content_negotiation, none_applicable | error | false |

## §3  Always-required sections (IEEE 1016 viewpoints)

| rule_id | Heading | Severity |
| ------- | ------- | -------- |
| `SEC-001` | `## 1. Introduction` (purpose, scope, design overview) | error |
| `SEC-002` | `## 2. Context Viewpoint` (system boundary, external interfaces) | error |
| `SEC-003` | `## 3. Composition Viewpoint` (component decomposition, modules) | error |
| `SEC-004` | `## 4. Logical Viewpoint` (class / object model, key abstractions) | error |
| `SEC-005` | `## 5. Information Viewpoint` (data model, persistence schema) | error |
| `SEC-006` | `## 6. Interface Viewpoint` (API specs, message formats, OpenAPI link) | error |
| `SEC-007` | `## 7. Patterns Viewpoint` (design patterns applied, rationale) | error |
| `SEC-008` | `## 8. Interaction Viewpoint` (sequence diagrams for primary flows) | error |
| `SEC-009` | `## 9. State Dynamics Viewpoint` (state machines for stateful components) | warning (required when components have non-trivial state) |
| `SEC-010` | `## 10. Algorithm Viewpoint` (algorithms with complexity analysis where non-obvious) | warning |
| `SEC-011` | `## 11. Resource Viewpoint` (memory / CPU / storage / network expectations) | error |
| `SEC-901` | Each required section is non-empty | error |

## §4  Conditionally-required sections

| rule_id | Trigger | Required | Severity |
| ------- | ------- | -------- | -------- |
| `COND-001` | Component exposes HTTP/gRPC/GraphQL API | `## 12. API Specification` referencing an OpenAPI/AsyncAPI/proto file with hash | error |
| `COND-002` | Component persists data | `## 13. Persistence Design` covering schema, indexes, migration strategy | error |
| `COND-003` | Component has UI | `## 14. UI Design` referencing Figma / wireframe / mock-up assets with `hash:` | error |
| `COND-004` | Component is performance-critical (NFR target <p99 <100ms or >1k req/s) | `## 15. Performance Design` with budget allocation + cache/index strategy | error |
| `COND-005` | Component is part of a public-facing surface | `## 16. Backwards-Compatibility Strategy` (per `api_versioning_policy`) | error |

## §5  Quality heuristics

| rule_id | Anti-pattern | Detection | Severity |
| ------- | ------------ | --------- | -------- |
| `QA-CITE-001` | Claim without `source_ref` | non-boilerplate paragraph lacks `source_ref` | error |
| `QA-AUTH-001` | Paragraph without `authority` marker | non-boilerplate paragraph lacks `authority:` | error |
| `QA-TRACE-001` | Design element without SRS traceability | A component / API / data entity lacks `traces_to:` referencing ≥1 REQ-ID | error → needs_human (`scope_decomposition`) |
| `QA-ADR-001` | Design contradicts an accepted ADR | A design choice differs from the decision captured in a `linked_adrs` ADR without an explicit override note | error → needs_human (`legal_compliance` if security ADR) |
| `QA-OPENAPI-001` | API section references OpenAPI but the file doesn't resolve | error |
| `QA-OPENAPI-002` | OpenAPI version field missing or older than 3.1 | warning |
| `QA-SCHEMA-001` | Persistence design lacks index strategy for tables expected >1M rows | warning |
| `QA-PATTERN-001` | Pattern named in §7 without rationale | "uses Strategy pattern" without `why:` | warning |
| `QA-VERSION-001` | `api_versioning_policy: none_applicable` for a component with COND-001 firing | error → needs_human |
| `QA-COMPLEX-001` | Algorithm in §10 without Big-O analysis | warning |
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
| `XCHAIN-003` | Every `linked_adrs` ADR has `status: accepted` | error |
| `XCHAIN-004` | If a threat-model exists for this system, every security-relevant component in §3-§4 is enumerated in that threat model's STRIDE analysis | warning |

## §8  Staleness

| rule_id | Trigger | Action | Severity |
| ------- | ------- | ------ | -------- |
| `STALE-001` | Source SRS hash differs | Reset open + needs_human to open | warning → needs_human |
| `STALE-002` | A `linked_adrs` ADR moved to `superseded` | Suggest SDD review | warning |

---

## Cross-references

- `cyberos/skill/docs/AUDIT_LOOP.md`, `cyberos/skill/docs/RUBRIC_FORMAT.md`
- IEEE 1016-2009 — SDD viewpoints source
- arc42 §5-§10
- `cyberos/docs/Software Development Process.md` §2(e) — Detailed design stage source
