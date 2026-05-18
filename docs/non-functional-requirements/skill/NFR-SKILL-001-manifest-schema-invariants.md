---
id: NFR-SKILL-001
title: "SKILL manifest schema invariants — required-field gate + semver discipline"
module: SKILL
category: maintainability
priority: MUST
verification: T
phase: P0
slo: "100% of published manifests pass schema validation; 0 unknown-field tolerance"
owner: CTO
created: 2026-05-18
related_frs: [FR-SKILL-103, FR-SKILL-101]
---

## §1 — Statement (BCP-14 normative)

1. Every SKILL bundle published to the OCI registry **MUST** carry a top-level `SKILL.md` whose YAML frontmatter parses cleanly and contains the closed set of required keys: `name`, `version`, `description`, `inputs`, `outputs`, `capabilities`, `audit`.
2. `version` **MUST** be valid SemVer 2.0.0; any author bump that lowers `version` against the registry's latest **MUST** be rejected at publish time.
3. Manifest schema validation **MUST** reject **unknown top-level fields** (strict mode) — typos silently shipping would mask broken capability requests or audit toggles.
4. The `capabilities` array **MUST** be a subset of the broker's published capability catalog (FR-SKILL-104); unknown capability strings cause the publish to fail with `E_CAPABILITY_UNKNOWN`.
5. Validation **MUST** run both at `cyberos-skill publish` time (client-side gate) **and** at registry-side ingress (server gate). The two gates **MUST** share the same schema artifact (`modules/skill/schema/manifest.schema.json`).

## §2 — Why this constraint

The SKILL bundle is a contract between the author and the runtime. If a manifest can carry typos or unknown fields the runtime silently ignores, the deployed skill behaviour diverges from the author's intent — a class of bug that surfaces months later as audit gaps or capability denials. Strict schema enforcement + dual gates make manifests **fail loud**. The SemVer-monotonicity rule prevents the catalog from accidentally shipping a regression bump.

## §3 — Measurement

- Counter `skill_publish_validation_failure_total{stage=client|server, error_code}` per failed publish.
- CI metric: `skill_schema_drift_count` — number of bundles in the OCI registry whose manifest fails the latest schema (must always be 0 post-migration).
- Histogram `skill_manifest_field_count` — surfaces drift toward overly-rich manifests (proxy for capability sprawl).

## §4 — Verification

- Unit test `modules/skill/tests/manifest_schema_test.py` (T) — fixtures for valid + 12 invalid manifests; asserts pass/fail outcome and error code.
- CI gate `skill-validate-all` (T) — walks every bundle in `skill/public/` + `skill/private/` and runs the validator; fails the CI run on any failure.
- Property test (T) — generates random valid frontmatter; asserts client+server gates always agree.

## §5 — Failure handling

- Unknown field detected at publish → `E_SCHEMA_UNKNOWN_FIELD`, publish blocked, author guided to whitelist.
- SemVer regression → `E_VERSION_REGRESSION`, publish blocked.
- Registry-side validation finds a corrupted manifest (post-publish drift) → sev-2 alert; bundle quarantined; CTO investigates how it bypassed publish gate.

---

*End of NFR-SKILL-001.*
