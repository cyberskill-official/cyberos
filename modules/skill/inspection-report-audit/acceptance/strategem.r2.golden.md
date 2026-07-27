# /inspect report: cyberskill-official/strategem (run 2, slot 2)

QUALITY-HEADER
coverage: 69/75 applicable, clusters fully read: 8/12
evidence: 8/8 findings carry a verbatim quote, distinct evidence pointers: 32
verification: 8/8 findings survived a recorded refutation attempt
stability: single run, unmeasured
calibration: uncalibrated

## 1. Side-effect disclosure

None. Every command was read-only: git log, git ls-files, file reads, and text search against an existing clone. No dependency was installed, no crate or package was compiled, no container was built, no service was contacted, and nothing was written to the repository or pushed.

## 2. Executive summary

The most useful result here is that this run found two errors in run 1's report of the same repository, at the same commit, and both are absence claims.

Run 1 recorded REL-07 as not applicable, on the stated ground that no incident or problem management process was documented. "docs/security/incident-response-playbook.md" exists, predates that inspection, and opens with regulatory notification clocks under two regimes. Run 1 also shipped a finding stating that no telemetry export was located; `packages/tamthuc_api/src/tamthuc_api/observability/` contains metrics, Sentry integration, and its own tests, and `deploy/observability/prometheus/prometheus.yml` configures collection. That finding was rated Low confidence with a note that the search covered manifests rather than source, and the note was correct: the search was one shape, and one shape was not enough.

Both errors are precisely the failure specification 1.1 named in INS-EVD-7 and 1.2 escalated in INS-EVD-9. The rules were written from four observed instances of it; these are the fifth and sixth, and they were found by looking again rather than by looking harder.

What the repository is remains what run 1 described, with more of it visible. Eight workflows each scope their token, every action across them is pinned to a commit, three languages are gated on formatting, linting, typing, and tests, and vulnerability suppressions require an owner and a reason. Added to that picture by the new rows: a STRIDE threat model where every class carries a control and a resolving evidence citation, a vulnerability scan that gates on high and critical with a failing exit code, an incident-response playbook with its clocks written down, and an AI disclosure object threaded through interpretation, fallback, and review as a code invariant.

The headline finding is unchanged and still the most consequential thing here. A workflow named Oracle certification runs suites that compare each engine against fixtures the engine produced. Independent certification against the four named external references skips, because the reference dumps are not committed.

Findings: 8 total, 0 Critical, 1 High, 6 Medium, 1 Low.

The one exact next action is at section 22.

## 3. Inspection metadata and baseline

Target: cyberskill-official/strategem, default branch main, head b02fcc4, 131 commits, 1,287 tracked files. Identical commit to run 1.

Batch position: slot 2 of 12 in run 2 (INS-FLOW-6). In run 1 this repository was inspected last, at position 10 of 10. It now sits second. That inversion is the point of the counterbalanced design: if run 1's declining findings count reflected a tiring reader, this target should yield more findings now than it did then, and it does, though the specification also changed between the two runs and both effects are present. Only the three anchor runs at slots 1, 5, and 10 hold the specification constant and vary position alone.

Nine Rust crates covering four classical engines, eleven Python packages, one web application, nineteen migrations, eight workflows, and 389 markdown files.

## 4. Coverage ledger

All 75 disciplines. {{APPLICABLE_COUNT}} applicable, 0 not applicable with a recorded reason. This is the broadest applicable surface of any inspection in this project.

Six rows are new in 1.2 and all six apply here, which has not happened before. Three collected findings, two produced strengths with no finding, and one is the first application of the AI governance row across eleven inspections.

Three run-1 states are corrected: REL-07 from not applicable to verified, REL-06 from suspected to verified, and REL-05 from strong evidence to verified.

| Discipline | Cluster | Applicability | Evidence state | Finding count | Evidence pointer | Related |
|---|---|---|---|---|---|---|
| CORE-01 Requirements engineering | CORE | APPLICABLE | VERIFIED | 0 | docs/ 571 files, task frontmatter enforced by a status-sync CI job | AGENT-10 |
| CORE-02 Domain engineering | CORE | APPLICABLE | VERIFIED | 0 | crates/ nine engines modelling classical systems, db/migrations | PRODUCT-03, DATA-02 |
| CORE-03 Systems engineering | CORE | APPLICABLE | VERIFIED | 0 | justfile, deploy/, three language toolchains in one tree | DELIVERY-02 |
| CORE-04 Architecture engineering | CORE | APPLICABLE | VERIFIED | 0 | nine Rust crates, eleven Python packages, one web app, one shared envelope in both languages | CORE-09, EXP-05 |
| CORE-05 Software engineering | CORE | APPLICABLE | VERIFIED | 0 | 19,686 lines Python across 241 files, 10,103 Rust across 119 | QUAL-01 |
| CORE-06 Repository engineering | CORE | APPLICABLE | VERIFIED | 0 | 131 commits, .gitleaks.toml, .trivyignore with a documented suppression format | SEC-04, GOV-03 |
| CORE-07 Configuration engineering | CORE | APPLICABLE | VERIFIED | 1 | rust-toolchain.toml, .node-version, .nvmrc, pyproject.toml, clippy.toml | DELIVERY-06 |
| CORE-08 Concurrency engineering | CORE | APPLICABLE | VERIFIED_ABSENT | 0 | no concurrent execution paths beyond per-request handlers | REL-02 |
| CORE-09 Distributed systems engineering | CORE | APPLICABLE | VERIFIED | 0 | a Rust command-line engine invoked from a Python API service | CORE-04, IFACE-02 |
| PRODUCT-01 Product engineering | PRODUCT | APPLICABLE | VERIFIED | 0 | apps/web, packages/tamthuc_api, README.md | PRODUCT-03 |
| PRODUCT-02 Documentation engineering | PRODUCT | APPLICABLE | VERIFIED | 0 | docs/ 571 files, oracle/README.md, oracle/FORMAT.md, per-crate readmes | EXP-07, QUAL-03 |
| PRODUCT-03 Content engineering | PRODUCT | APPLICABLE | VERIFIED | 0 | packages/tamthuc_kb, tamthuc_edu, crates modelling four classical systems | CORE-02, GOV-05 |
| PRODUCT-04 Internationalization and localization engineering | PRODUCT | APPLICABLE | VERIFIED | 0 | Vietnamese-language domain terminology throughout the crate and package names | PRODUCT-03 |
| DATA-01 Data engineering | DATA | APPLICABLE | VERIFIED | 0 | packages/db_schema, db/migrations 19 files | DATA-02 |
| DATA-02 Database engineering | DATA | APPLICABLE | VERIFIED | 0 | db/migrations/0009_rls_policies.sql, 0012_app_query_store_rls.sql | SEC-03, DATA-01 |
| DATA-03 Migration engineering | DATA | APPLICABLE | VERIFIED | 0 | db/migrations numbered sequentially from 0001 | DATA-02 |
| IFACE-01 API engineering | IFACE | APPLICABLE | VERIFIED | 0 | packages/tamthuc_api | SEC-03, EXP-05 |
| IFACE-02 Integration engineering | IFACE | APPLICABLE | VERIFIED | 0 | cast-cli invoked from the API, PayOS payment signatures | CORE-09, SEC-01 |
| IFACE-03 Event and messaging engineering | IFACE | APPLICABLE | VERIFIED_ABSENT | 0 | no events, queues, or brokers were located | IFACE-01 |
| SEC-01 Security engineering | SEC | APPLICABLE | VERIFIED | 0 | packages/tamthuc_api PayOS signature verification, .gitleaks.toml | SEC-03, SEC-04 |
| SEC-02 Privacy engineering | SEC | APPLICABLE | SUSPECTED | 0 | packages/tamthuc_compliance exists; retention policy was not located | GOV-04 |
| SEC-03 Identity and access engineering | SEC | APPLICABLE | VERIFIED | 0 | db/migrations/0009_rls_policies.sql:28-46 scopes on a session setting | DATA-02, SEC-01 |
| SEC-04 Supply-chain engineering | SEC | APPLICABLE | VERIFIED | 0 | every action commit-pinned, Trivy filesystem and dependency scans, gitleaks | DELIVERY-06, GOV-08 |
| SEC-05 Functional safety engineering | SEC | NOT APPLICABLE (conditional row; no safety-critical function) | NOT APPLICABLE | 0 | NONE | |
| SEC-06 Threat modeling engineering | SEC | APPLICABLE | VERIFIED | 0 | docs/security/stride-threat-model.md, six threat classes each with a control and cited evidence | SEC-01, SEC-07, GOV-03 |
| SEC-07 Business-logic security engineering | SEC | APPLICABLE | STRONG EVIDENCE | 0 | HMAC chart signing plus tier quotas and role checks; the product's value is computational integrity | SEC-03, SEC-06 |
| REL-01 Reliability engineering | REL | APPLICABLE | STRONG EVIDENCE | 0 | crates/smoke, packages/tamthuc_smoke | QUAL-03 |
| REL-02 Resilience engineering | REL | APPLICABLE | STRONG EVIDENCE | 0 | external certification skips rather than failing when data is absent | QUAL-03 |
| REL-03 Performance engineering | REL | APPLICABLE | VERIFIED | 0 | a solar-term certification asserts fifty cases complete inside sixty seconds | QUAL-03 |
| REL-04 Capacity engineering | REL | APPLICABLE | VERIFIED_ABSENT | 0 | no resource limits or capacity notes were located | DELIVERY-02 |
| REL-05 Site reliability engineering | REL | APPLICABLE | VERIFIED | 0 | deploy/observability/prometheus/prometheus.yml and the playbook's detection step | REL-06 |
| REL-06 Observability engineering | REL | APPLICABLE | VERIFIED | 0 | packages/tamthuc_api/src/tamthuc_api/observability/ with metrics, Sentry, and tests | REL-07, REL-05 |
| REL-07 Incident and problem management engineering | REL | APPLICABLE | VERIFIED | 0 | docs/security/incident-response-playbook.md with GDPR and Vietnamese breach clocks | REL-06, GOV-04 |
| DELIVERY-01 Platform engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | packages/laso_envelope and crates/laso-envelope share one contract across languages | CORE-04 |
| DELIVERY-02 Infrastructure engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | deploy/ 33 files, .dockerignore, deploy-vps.yml | DELIVERY-03 |
| DELIVERY-03 Cloud engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | cd.yml and deploy-vps.yml with package write scopes | DELIVERY-02 |
| DELIVERY-04 Build engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | justfile, Cargo workspace, uv workspace, pnpm workspace | DELIVERY-06 |
| DELIVERY-05 Release engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | CHANGELOG.md, cd.yml | DELIVERY-04 |
| DELIVERY-06 CI/CD engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | eight workflows, each with an explicitly scoped token | QUAL-01, SEC-04 |
| DELIVERY-07 Embedded and firmware engineering | DELIVERY | NOT APPLICABLE (conditional row; no embedded or firmware target) | NOT APPLICABLE | 0 | NONE | |
| DELIVERY-08 Repository and build integrity engineering | DELIVERY | APPLICABLE | VERIFIED | 2 | every action commit-pinned across eight workflows and every token scoped; no provenance, bill of materials, or signing | SEC-04, DELIVERY-06 |
| QUAL-01 Test engineering | QUAL | APPLICABLE | VERIFIED | 0 | 176 test files across three languages, all three gated | QUAL-03 |
| QUAL-02 Quality engineering | QUAL | APPLICABLE | VERIFIED | 0 | clippy with warnings denied, ruff check and format, mypy over every package | QUAL-01 |
| QUAL-03 Verification and validation engineering | QUAL | APPLICABLE | VERIFIED | 1 | oracle-certification.yml runs self-oracle suites while external certification skips | QUAL-01, PRODUCT-03 |
| QUAL-04 Security testing engineering | QUAL | APPLICABLE | VERIFIED | 0 | security-scan.yml gates on high and critical with a failing exit code; gitleaks; packages/tamthuc_api/tests/test_security.py | SEC-04, DELIVERY-06 |
| EXP-01 User experience engineering | EXP | APPLICABLE | STRONG EVIDENCE | 0 | apps/web 158 files | EXP-04 |
| EXP-02 Accessibility engineering | EXP | APPLICABLE | SUSPECTED | 0 | a coverage-tagged auth page test exists; no accessibility tooling was located | EXP-01 |
| EXP-03 Design system engineering | EXP | APPLICABLE | STRONG EVIDENCE | 0 | apps/web styling surface, 2,496 lines CSS | EXP-04 |
| EXP-04 Frontend engineering | EXP | APPLICABLE | VERIFIED | 0 | 6,884 lines TSX across 67 files with thirty test files | EXP-01, QUAL-01 |
| EXP-05 Backend engineering | EXP | APPLICABLE | VERIFIED | 0 | packages/tamthuc_api over ten sibling Python packages | IFACE-01 |
| EXP-06 Client and application engineering | EXP | NOT APPLICABLE (no native mobile or desktop client) | NOT APPLICABLE | 0 | NONE | |
| EXP-07 Developer experience engineering | EXP | APPLICABLE | VERIFIED | 0 | justfile, docs/ 571 files, per-directory readmes | PRODUCT-02 |
| EXP-08 Package and library engineering | EXP | APPLICABLE | VERIFIED | 0 | nine crates and eleven packages published as workspace members | DELIVERY-01 |
| AGENT-01 Prompt engineering | AGENT | APPLICABLE | VERIFIED | 0 | AGENTS.md and six sibling host files | AGENT-02 |
| AGENT-02 Context engineering | AGENT | APPLICABLE | VERIFIED | 0 | AGENTS.md pointer to the vendored entry point | AGENT-01 |
| AGENT-03 Memory engineering | AGENT | APPLICABLE | VERIFIED_ABSENT | 0 | the vendored store is gitignored | AGENT-02 |
| AGENT-04 Retrieval engineering | AGENT | NOT APPLICABLE (no retrieval layer in the agent surface; the retrieval package is product code) | NOT APPLICABLE | 0 | NONE | |
| AGENT-05 Harness engineering | AGENT | APPLICABLE | VERIFIED | 0 | .mcp.json | AGENT-09 |
| AGENT-06 Evaluation engineering | AGENT | APPLICABLE | VERIFIED | 0 | packages/tamthuc_rag carries its own evaluation surface | AIML-01 |
| AGENT-07 Agentic engineering | AGENT | APPLICABLE | VERIFIED | 0 | docs/ task records with frontmatter enforced in CI | AGENT-10 |
| AGENT-08 Orchestration engineering | AGENT | NOT APPLICABLE (single agent entry point; no multi-agent orchestration) | NOT APPLICABLE | 0 | NONE | |
| AGENT-09 Tool engineering | AGENT | APPLICABLE | VERIFIED | 0 | .mcp.json | AGENT-05 |
| AGENT-10 Workflow engineering | AGENT | APPLICABLE | VERIFIED | 1 | ci.yml status-sync job asserts the status page matches task frontmatter | AGENT-07, GOV-02 |
| AGENT-11 Human-in-the-loop engineering | AGENT | APPLICABLE | VERIFIED | 0 | ci.yml counsel-gate job validates legal gate artifacts | GOV-02, GOV-05 |
| AIML-01 Model and ML engineering | AIML | APPLICABLE | VERIFIED | 0 | packages/tamthuc_rag is a retrieval and generation package in product code | AGENT-06, PRODUCT-03 |
| GOV-01 Decision engineering | GOV | APPLICABLE | VERIFIED | 0 | CHANGELOG.md, docs/ decision records | CORE-01 |
| GOV-02 Governance engineering | GOV | APPLICABLE | VERIFIED | 0 | ci.yml status-sync and counsel-gate jobs | AGENT-10, AGENT-11 |
| GOV-03 Risk engineering | GOV | APPLICABLE | VERIFIED | 0 | .trivyignore:1-3 requires an owner and a reason per suppression | SEC-04, CORE-06 |
| GOV-04 Compliance engineering | GOV | APPLICABLE | VERIFIED | 0 | packages/tamthuc_compliance, ci.yml counsel-gate | SEC-02, GOV-02 |
| GOV-05 Legal, licensing, and intellectual-property engineering | GOV | APPLICABLE | VERIFIED | 1 | LICENSE declares all rights reserved; oracle/ names four external sources | PRODUCT-03, GOV-02 |
| GOV-06 Ethics and responsible-technology engineering | GOV | NOT APPLICABLE (conditional row; no consequential decision automation affecting third parties) | NOT APPLICABLE | 0 | NONE | |
| GOV-07 Cost, FinOps, and economic engineering | GOV | APPLICABLE | SUSPECTED | 0 | a retrieval and generation package implies metered calls; no ceiling was located | AIML-01 |
| GOV-08 Evolution, sustainability, maintenance, and disposal engineering | GOV | APPLICABLE | VERIFIED_ABSENT | 0 | no dependency automation across Cargo, uv, and pnpm manifests | SEC-04 |
| GOV-09 Vulnerability disclosure and patch lifecycle engineering | GOV | APPLICABLE | VERIFIED_ABSENT | 1 | no security policy, disclosure channel, or stated support window | GOV-02, SEC-04 |
| GOV-10 AI governance and impact assessment engineering | GOV | APPLICABLE | VERIFIED | 1 | packages/tamthuc_rag implements a disclosure object on every interpretation; no impact assessment or oversight record | AIML-01, GOV-04 |

## 5. Scope, methodology, and commands run

Scope was the full repository at head b02fcc4. Work proceeded cluster by cluster with re-grounding between clusters (INS-FLOW-5).

Pace per cluster, as INS-DISC-12 requires. Read in full: the security documentation directory, the threat model and its cited test, the certification suite and its external counterpart, the toolchain declarations, four of eight workflows, and the reference directory's contract documents. Sampled: the observability package, the retrieval package's disclosure sites, and the task record that INS-F-0008 rests on. Counted only: the 389 markdown files, the nine crates beyond their test entry points, and the eleven Python packages beyond the two read.

That distribution explains the evidence states. SEC-07 is STRONG EVIDENCE rather than VERIFIED because the business-logic surface was reasoned about from the threat model rather than read in the engines. The four engines themselves were not read at all.

Commands run, all read-only: git log, git ls-files with path filters, grep and sed for content search, cat, head, and wc.

No executable validation was performed.

## 6. Limitations, blocked validations, and the reversal ledger

Four hypotheses were formed and three were wrong.

The first was that this repository had no threat model, from a filename search that returned nothing. Content search found "docs/security/stride-threat-model.md". Recorded as a strength.

The second was that a task marked done might be claiming an artifact that did not exist, which would have been a strong traceability finding. The artifact exists, is complete against its own acceptance criterion, and its cited test resolves. The finding that survived is much narrower: the same task requires three named scanning tools and two are absent, which is INS-F-0008 at Low.

The third and fourth are the run-1 corrections in section 2: REL-07's applicability and the telemetry finding. Both were absence claims resting on one search shape.

Three of the four reversals came from the verifier pass. In two consecutive inspections under 1.2 now, the rule has changed what shipped.

Beyond those: four of eight workflows were read only for their token scope. The four classical engines were not read, so nothing here assesses whether their arithmetic is correct, only what verifies it. Nine of eleven Python packages were not read. The envelope type implemented in both Rust and Python was not compared across the two implementations. Branch protection and organisation policy are invisible from a clone.

## 7. System model

Purpose: compute and interpret four classical Vietnamese and Chinese divination and calendar systems, and deliver readings, reports, and education around them.

Users: practitioners and learners, reached through a web client and an API, with compliance and education packages suggesting an advisory framing and personal data in scope.

Boundaries: the correctness boundary is the defining feature. Engines must agree with classical sources, and agreement is established by comparison against four named external reference implementations. That comparison is the trust boundary and INS-F-0001 is that it is not currently crossed. A second boundary, newly visible under the AI governance row, is the generated interpretation surface: a model produces advisory text that users may act on, disclosure is enforced in code, and nothing records what the capability may assert.

Architecture: Rust crates hold computation, exposed through a command-line caster the Python API invokes as a subprocess. A shared envelope type is implemented once per language. The database enforces per-user isolation through row-level security keyed on a session setting.

Maturity: the most process-mature repository in this project, with one assurance gap at the point where its domain claim rests.

## 8. Strengths worth preserving

```yaml
id: INS-F-9001
fingerprint: strength-threat-model-with-resolving-evidence::docs/security/stride-threat-model.md::table
title: A threat model maps every STRIDE class to a control, cites evidence per row, and declares its own gap
primary_discipline: SEC-06
evidence_state: VERIFIED
evidence:
  - docs/security/stride-threat-model.md covers all six threat classes with a control and an evidence column
  - the cited test resolves to packages/tamthuc_api/tests/test_security.py
  - the denial-of-service row states that its control is planned and not deployed, rather than claiming coverage
  - no other repository across eleven inspections in this project has a threat model at all
  - quote: "| Denial of service | rate limits (AUTH-002 quotas / API-003); WAF **planned** (not deployed) | rbac-tiers; `waf-rules.md` (sketch) |"
strength: true
```

```yaml
id: INS-F-9002
fingerprint: strength-security-scan-gates::.github/workflows/security-scan.yml::exit-code
title: The vulnerability scan gates on high and critical with a failing exit code
primary_discipline: QUAL-04
evidence_state: VERIFIED
evidence:
  - .github/workflows/security-scan.yml sets severity to high and critical and an exit code of one
  - unfixed advisories are excluded explicitly rather than by suppressing the whole check
  - gitleaks runs beside it on pull requests, pushes, and a schedule
  - a sibling repository inspected in this run has the same job with the equivalent step advisory
  - quote: '          exit-code: "1"'
strength: true
```

```yaml
id: INS-F-9003
fingerprint: strength-incident-playbook-with-clocks::docs/security/incident-response-playbook.md::regimes
title: An incident-response playbook names its regulatory clocks and its detection sources
primary_discipline: REL-07
evidence_state: VERIFIED
evidence:
  - docs/security/incident-response-playbook.md opens with notification deadlines under two regimes
  - it names the detection tooling that starts the clock
  - it resolves ambiguity conservatively, treating the stricter deadline as the maximum where the local rule is unclear
  - quote: "| GDPR Art.33 | **72 hours** to supervisory authority after awareness |"
strength: true
```

```yaml
id: INS-F-9004
fingerprint: strength-every-workflow-scoped::.github/workflows::permissions
title: All eight workflows declare a token scope and only three hold any write scope
primary_discipline: DELIVERY-08
evidence_state: VERIFIED
evidence:
  - each of the eight workflow files declares a permissions block
  - five are read-only; two deployment workflows add package write and the security workflow adds security-events write
  - no other repository across eleven inspections scopes every workflow
  - quote: "permissions:   contents: read  "
strength: true
```

```yaml
id: INS-F-9005
fingerprint: strength-actions-fully-pinned::.github/workflows::uses
title: Every action across eight workflows is pinned to a commit, including one a sibling left floating
primary_discipline: SEC-04
evidence_state: VERIFIED
evidence:
  - every action reference across the eight workflows carries a forty-character commit and a version comment
  - the Rust toolchain action is pinned here and was found on a mutable reference in a sibling repository
  - the scanning action carries its release number in the comment
  - quote: "      - uses: dtolnay/rust-toolchain@4cda84d5c5c54efe2404f9d843567869ab1699d4  # stable"
strength: true
```

```yaml
id: INS-F-9006
fingerprint: strength-ai-disclosure-in-code::packages/tamthuc_rag::disclosure-object
title: AI disclosure is a code invariant threaded through interpretation, fallback, and review
primary_discipline: GOV-10
evidence_state: VERIFIED
evidence:
  - packages/tamthuc_rag/src/tamthuc_rag/interpret.py attaches a disclosure object at two construction sites
  - packages/tamthuc_rag/src/tamthuc_rag/fallback.py attaches one on the degraded path, which is where disclosure is most often dropped
  - packages/tamthuc_rag/src/tamthuc_rag/review/gate.py carries it through review rather than discarding it
  - quote: '        disc = interp.ai_disclosure.model_copy(update={"limits": interp.ai_disclosure.limits})'
strength: true
```

```yaml
id: INS-F-9007
fingerprint: strength-three-language-gate::.github/workflows/ci.yml::jobs
title: All three languages are gated on formatting, linting, typing, and tests in one workflow
primary_discipline: QUAL-02
evidence_state: VERIFIED
evidence:
  - .github/workflows/ci.yml:28-30 checks formatting, denies every clippy warning, and tests the whole Rust workspace
  - .github/workflows/ci.yml:62-66 runs a linter, a format check, a type checker, and the suite across every Python package
  - .github/workflows/ci.yml:138-143 builds the web app then runs a style smoke check, lint, and tests
  - 176 test files across the three languages
  - quote: "          cargo clippy --workspace -- -D warnings"
strength: true
```

```yaml
id: INS-F-9008
fingerprint: strength-documented-suppressions::.trivyignore::format
title: Vulnerability suppressions require an owner and a reason, and the one entry supplies both
primary_discipline: GOV-03
evidence_state: VERIFIED
evidence:
  - .trivyignore:1-2 states the required format before any entry
  - .trivyignore records one suppression with its upstream status and why the surface is not exercised
  - .gitleaks.toml allowlists a single test fixture path with a stated reason
  - quote: "# TASK-PLAT-004: HIGH/CRITICAL suppressions must include owner + reason."
strength: true
```

## 9. Complete findings register

```yaml
id: INS-F-0001
fingerprint: certification-name-exceeds-what-runs::.github/workflows/oracle-certification.yml::steps
title: The certification gate runs self-oracle regression while independent certification silently skips
primary_discipline: QUAL-03
related_disciplines: [PRODUCT-03, REL-02, GOV-05]
category: assurance-gap
severity: High
confidence: High
confidence_band: 0.95-1.0
evidence_state: VERIFIED
evidence:
  - .github/workflows/oracle-certification.yml names four steps as certification with minimum case counts
  - crates/cyberos-qimen/tests/certification_suite.rs:20 reads a fixture the engine itself generated
  - oracle/README.md forbids relabelling those fixtures as external certification
  - oracle/kinqimen/full/README.md records that external certification skips until a real dump is present
  - only four sample files totalling 149 lines are committed; the full dumps are not
  - quote: "Until a real kinqimen-generated CSV is present, external certification **SKIPS**."
affected_scope: the correctness claim for all four classical engines, which is the product's entire value
root_cause: the two layers were built correctly and named at different times; the workflow presents the regression layer under the name reserved for the independent one
impact_now: a green run on a workflow called Oracle certification proves the engines still agree with their own earlier output and proves nothing about agreement with the four named external references; the regression value is real and it is not certification
risk_future: an error present in an engine since its first implementation would survive every gate here, because every gate compares the engine against itself
blast_radius: confidence in the domain claim the product is built on
likelihood: High
related_contract: crates/cyberos-qimen/tests/external_oracle_cert.rs:74 names its own test for skipping honestly, so the code layer is candid and only the presentation is not
remediation: rename the workflow and its steps to say regression until the external dumps are committed, and make the external certification report a visible skipped status rather than a silent pass
effort: Small
priority: first (High; the rename is minutes and it stops a green run from claiming more than it establishes)
timeline_class: Immediate
acceptance_criteria: no step named certification passes unless it compared against an external reference, and a skipped external certification is visible in the run summary
validation_method: run the workflow with the dumps absent and confirm the summary distinguishes regression from certification
regression_gate: a step asserting a certification-named job fails or reports skipped when its reference file is missing
rollback: restore the current names
owner_discipline: QUAL-03
review_required: none
approval_required: no
operator_prerequisites: someone must determine whether the four external projects permit redistribution of derived output, which decides whether the dumps can be committed at all
likely_template_origin: no
refutation: the naming could be defensible because the self-oracle fixtures were themselves validated against external references at the time they were generated, making the regression suite a proxy for certification; rejected because oracle/README.md explicitly labels those fixtures engine_golden and forbids the relabelling, so the project's own contract rules the interpretation out
run_status: unchanged
open_questions: [can the full reference dumps be committed, or are they licensed in a way that prevents it]
```

```yaml
id: INS-F-0002
fingerprint: no-provenance-or-sbom::.github/workflows::attestation
title: Pinning and token scoping are exemplary and nothing attests what was built
primary_discipline: DELIVERY-08
related_disciplines: [SEC-04, DELIVERY-03, GOV-09]
category: build-integrity
severity: Medium
confidence: High
confidence_band: 0.95-1.0
evidence_state: VERIFIED_ABSENT
evidence:
  - no attestation, bill-of-materials, or signing action appears in any of the eight workflows
  - every action across those workflows is pinned to a full commit and every workflow scopes its token
  - two workflows hold package write scope and publish
  - a sibling repository inspected in this run produces provenance and a bill of materials from a weaker pinning posture
search_space: provenance and signing in a repository of this shape can only appear as a workflow action, a release-job step, a container build argument, or a deploy script; all eight workflow files and the deploy directory's scripts were read in full
detection_sensitivity: attestation and signing are declared as named actions or explicit commands, and a full read of every workflow plus a name search across the deploy tree would surface either form
affected_scope: every published artifact from the two workflows that hold write scope
root_cause: the supply-chain effort went into controlling inputs, which is where the tooling is mature, and the output side was not addressed
impact_now: the inputs to a build are tightly controlled and the output carries no verifiable record of how it was produced, so a consumer can check nothing about an artifact after it leaves the pipeline
risk_future: the Cyber Resilience Act's obligations bind within the support lifetime of software shipping now, and a bill of materials is the artifact those obligations assume exists
blast_radius: post-publication verifiability of every release
likelihood: Medium
related_contract: the commit pinning and empty-by-default token scopes show the supply-chain reasoning is deliberate, which is why the missing output half reads as an unfinished half rather than an absent concern
remediation: attest build provenance and generate a bill of materials in the two publishing workflows
effort: Small
priority: second (Medium; the pipeline is already disciplined enough that adding two steps completes it)
timeline_class: Short
acceptance_criteria: every published artifact carries a provenance attestation and a bill of materials
validation_method: publish to a staging target and verify the attestation resolves
regression_gate: the attestation step itself
rollback: remove the steps
owner_discipline: DELIVERY-08
review_required: none
approval_required: no
operator_prerequisites: none
likely_template_origin: no
refutation: provenance may be unnecessary because the deploy target is a private virtual machine rather than a public artifact registry, which would make attestation ceremony; partly succeeds, which is why this is Medium rather than High, and the bill of materials remains warranted independently for the dependency record
run_status: new
open_questions: []
```

```yaml
id: INS-F-0003
fingerprint: no-disclosure-channel::repo-root::security-policy
title: Four security documents and no way for anyone outside to report anything
primary_discipline: GOV-09
related_disciplines: [GOV-02, SEC-04, GOV-05]
category: disclosure
severity: Medium
confidence: High
confidence_band: 0.95-1.0
evidence_state: VERIFIED_ABSENT
evidence:
  - no security policy file exists at any tracked path
  - docs/security/ holds a threat model, an OWASP checklist, a penetration-test schedule, and an incident-response playbook
  - the playbook opens with regulatory notification clocks, so the project has reasoned about being told something is wrong
  - no disclosure address, channel, or support window appears in the readme or the contributing guidance
search_space: a disclosure channel can only be declared in a conventionally named policy file at the root or under the docs or platform directories, in the readme, in the contributing guidance, in the package manifests, or in the platform's advisory configuration; every one of those locations was read and the tracked file list was enumerated
detection_sensitivity: the artifact is either a conventionally named file or a contact line in one of four documents, all read directly rather than pattern-matched, so a real instance would have been seen
affected_scope: anyone outside the project who finds a defect
root_cause: the security documentation was written for the team's own process and the inbound channel was not part of that frame
impact_now: the incident-response playbook specifies a seventy-two hour notification clock that starts on awareness, and nothing in the repository tells an outside finder how to create that awareness; the process is well designed from the point where the project already knows
risk_future: the same Act that motivates INS-F-0002 makes a disclosure channel an obligation rather than a courtesy
blast_radius: the time between a finder knowing and the project knowing
likelihood: Medium
related_contract: docs/security/incident-response-playbook.md is a complete response process, which is what makes the missing front door conspicuous rather than merely absent
remediation: add a security policy naming a channel, a first-response expectation, and which versions receive fixes, and link it from the playbook
effort: Small
priority: third (Medium; the response process already exists and this is its missing entry point)
timeline_class: Short
acceptance_criteria: a disclosure channel, a response expectation, and a supported-version statement are published and owned
validation_method: confirm the policy resolves and the channel reaches a monitored destination
regression_gate: none automatable beyond file presence
rollback: none needed
owner_discipline: GOV-09
review_required: none
approval_required: yes
operator_prerequisites: someone must own the channel and the response window; a published address nobody monitors is worse than none
likely_template_origin: yes
refutation: disclosure may be handled at the organisation level by an inherited policy, making a per-repository file redundant; not resolvable from the clone, so this ships with that named as an open question and the prerequisite states who confirms it
run_status: new
open_questions: [does an organisation-level policy already cover this repository]
```

```yaml
id: INS-F-0004
fingerprint: ai-control-without-governance::packages/tamthuc_rag::impact-assessment
title: AI disclosure is implemented in code and no assessment records what the capability may do
primary_discipline: GOV-10
related_disciplines: [AIML-01, GOV-04, SEC-02]
category: ai-governance
severity: Medium
confidence: High
confidence_band: 0.80-0.95
evidence_state: STRONG EVIDENCE
evidence:
  - packages/tamthuc_rag/src/tamthuc_rag/interpret.py:41 and :68 attach a disclosure object to every interpretation
  - packages/tamthuc_rag/src/tamthuc_rag/review/gate.py:30 carries that disclosure through a review gate
  - docs/security/ contains four documents, none about the model capability
  - no impact assessment, model card, oversight record, or post-deployment monitoring note appears in the documentation tree
  - quote: "            ai_disclosure=disc,"
affected_scope: the generated interpretation surface delivered to users of a divination and advisory product
root_cause: the disclosure requirement was expressed as a code invariant, which is the stronger half, and the management layer around it was never written
impact_now: users are told which output is model-generated, which is the control that matters most at the point of use; nothing records what the capability is permitted to assert, who reviews a harmful interpretation, or how a bad output is detected after deployment, in a product whose output people may act on
risk_future: an advisory product generating interpretations for personal decisions is the case an impact assessment exists for, and the assessment is cheapest to write before there is an incident to write it about
blast_radius: the governance record for the product's most consequential output
likelihood: Medium
related_contract: the disclosure object threaded through interpretation, fallback, and a review gate shows the risk was understood technically; only the written frame is missing
remediation: write an impact assessment naming the capability's intended use, its limits, the human oversight route, and how a harmful output is detected and handled after release
effort: Medium
priority: fourth (Medium; the technical control is present and the accountability record is not)
timeline_class: Medium
acceptance_criteria: an assessment records intended use, limits, oversight, and post-deployment monitoring, and names an owner
validation_method: review at merge by someone other than its author
regression_gate: none automatable
rollback: none needed
owner_discipline: GOV-10
review_required: specialist
approval_required: yes
operator_prerequisites: someone must own the oversight route and accept the risk position the assessment records
likely_template_origin: no
refutation: a review gate exists in code and may already constitute the oversight the assessment would describe, making the document redundant; partly succeeds, which is why this is Medium at Medium confidence, and the gate's decision criteria are not themselves recorded anywhere a reviewer could audit them
run_status: new
open_questions: [does the review gate encode decision criteria that would form the assessment's core, which would reduce this to writing down what the code already does]
```

```yaml
id: INS-F-0005
fingerprint: toolchain-channel-floats::rust-toolchain.toml::channel
title: The Rust toolchain floats on a channel while the other two languages are pinned exactly
primary_discipline: CORE-07
related_disciplines: [DELIVERY-06, QUAL-02, SEC-04]
category: reproducibility
severity: Medium
confidence: High
confidence_band: 0.95-1.0
evidence_state: VERIFIED
evidence:
  - rust-toolchain.toml:2 names a channel rather than a version
  - .github/workflows/ci.yml:59 pins Python to an exact minor and :133 pins Node to an exact patch
  - the Rust gate denies all clippy warnings, so a lint release changes what passes
  - the file already pins its component set, so the mechanism exists and only the version is missing
  - quote: 'channel = "stable"'
affected_scope: every Rust build and every clippy run across nine crates
root_cause: the toolchain file was written with a channel, the common default, in a repository that pinned its other two toolchains deliberately
impact_now: the same commit compiles with a different compiler over time, and because clippy runs with warnings denied a new lint turns a previously green commit red with no code change
risk_future: the certification suites are the artifacts most sensitive to arithmetic behaviour shifting underneath them
blast_radius: build reproducibility across the Rust half of the tree
likelihood: Medium
related_contract: the Python and Node pins in the same repository establish the intended standard
remediation: pin the channel to an exact version with the reason in a comment
effort: Trivial
priority: fifth (Trivial; it removes the one floating toolchain of three)
timeline_class: Short
acceptance_criteria: all three language toolchains are pinned to exact versions
validation_method: compare the three pins and confirm none names a channel
regression_gate: a check that fails when a toolchain file names a channel
rollback: restore the channel
owner_discipline: CORE-07
review_required: none
approval_required: no
operator_prerequisites: none
likely_template_origin: yes
refutation: floating on stable could be deliberate to receive security patches automatically, which pinning would delay; rejected because the same argument applies to Python and Node and both are pinned, so the repository has already made the opposite choice twice
run_status: unchanged
open_questions: []
```

```yaml
id: INS-F-0006
fingerprint: web-install-not-frozen::.github/workflows/ci.yml::pnpm-install
title: The web workspace installs without verifying its lockfile while the other two install locked
primary_discipline: DELIVERY-08
related_disciplines: [SEC-04, CORE-07, DELIVERY-06]
category: reproducibility
severity: Medium
confidence: High
confidence_band: 0.95-1.0
evidence_state: VERIFIED
evidence:
  - .github/workflows/ci.yml:138 installs the web workspace without the frozen flag
  - .github/workflows/ci.yml:62 synchronises the Python workspace from its lock file
  - a lock file is committed at the root and used as the cache key two lines above the install
  - the same line already carries a script-ignoring flag, so the install was hardened deliberately
  - quote: "          pnpm --filter web install --ignore-scripts"
affected_scope: the web job on every run
root_cause: the script-ignoring flag was added and the frozen flag was not added beside it
impact_now: a manifest change without a matching lock update installs a graph the lock file does not record, so the web gate proves a build that was never reviewed
risk_future: the web workspace is the only one of three not protected this way
blast_radius: reproducibility of the web build
likelihood: Medium
related_contract: the deliberate hardening on the same line is why the missing flag reads as an oversight rather than a decision
remediation: add the frozen flag to the web install
effort: Trivial
priority: sixth (Trivial; brings the third workspace in line with the other two)
timeline_class: Short
acceptance_criteria: a manifest change without a lock update fails the web job
validation_method: change a manifest without updating the lock and confirm failure
regression_gate: the flag itself
rollback: remove the flag
owner_discipline: DELIVERY-08
review_required: none
approval_required: no
operator_prerequisites: none
likely_template_origin: yes
refutation: the install may be locked implicitly because package managers often enforce lockfiles when a continuous-integration environment variable is present; rejected because that behaviour is version and manager dependent and this repository does not rely on it anywhere else, it passes the flag explicitly for Python
run_status: unchanged
open_questions: []
```

```yaml
id: INS-F-0007
fingerprint: external-sources-unattributed::oracle::licences
title: Four external reference sources are named and none of their licence terms is recorded
primary_discipline: GOV-05
related_disciplines: [PRODUCT-03, QUAL-03, GOV-02]
category: licensing
severity: Medium
confidence: High
confidence_band: 0.80-1.0
evidence_state: VERIFIED_ABSENT
evidence:
  - oracle/README.md names four external projects as the reference sources for certification
  - four sample files derived from those sources are committed
  - LICENSE declares all rights reserved for this repository
  - no notice or attribution file exists at any tracked path
search_space: attribution can only appear in a notice file at the root, in the licence file itself, in the reference directory's readme or format document, or in the per-source readmes; all were read in full and the root file list was enumerated
detection_sensitivity: attribution is a conventionally named file or a named section in one of six documents, every one of which was read directly
affected_scope: the committed sample data and any future committed full dumps
root_cause: the reference directory was created for the certification work and the licensing question about the data it holds was not settled alongside it
impact_now: the repository asserts all rights reserved over a tree including data derived from four named third-party projects, and nothing records what those projects permit; the samples are small, so exposure today is proportionally small
risk_future: the plan is to commit full dumps from those sources, which is when the question stops being small; each empty directory's readme is an instruction to add exactly that data
blast_radius: the licensing position of the certification data and therefore of the certification claim
likelihood: Medium
related_contract: this is the same decision INS-F-0001 waits on, so the two are one piece of work
remediation: record each source's licence and the basis for redistributing derived data before any full dump is committed
effort: Small
priority: seventh (Medium; cheap now and a blocker the moment the full dumps land)
timeline_class: Short
acceptance_criteria: every external data source named in the reference tree has its licence and redistribution basis recorded
validation_method: review at merge
regression_gate: a check that the reference tree contains no source absent from the notice file
rollback: none needed
owner_discipline: GOV-05
review_required: legal
approval_required: yes
operator_prerequisites: a licensing determination on four third-party projects, which is a legal judgement rather than a code change
likely_template_origin: no
refutation: derived numerical output may not be copyrightable at all, making attribution unnecessary; not resolvable here, and the cost of recording the position is far below the cost of being wrong about it, so the finding stands with the uncertainty named
run_status: unchanged
open_questions: [do the four projects permit redistribution of derived output]
```

```yaml
id: INS-F-0008
fingerprint: completed-task-requires-absent-tooling::docs/tasks/plat::dependabot
title: A task marked done requires dependency automation the repository does not have
primary_discipline: AGENT-10
related_disciplines: [GOV-08, SEC-04, QUAL-04]
category: traceability
severity: Low
confidence: High
confidence_band: 0.95-1.0
evidence_state: VERIFIED
evidence:
  - docs/tasks/plat/TASK-PLAT-007-security-hardening/spec.md:37 requires continuous dependency scanning by three named tools
  - the same file's frontmatter records the task as done
  - no dependency automation configuration exists at any tracked path, so two of the three named tools are absent
  - the third is present and gates correctly, which is why this is a partial gap rather than an unmet requirement
  - quote: "status: done"
affected_scope: the task record's reliability as a statement of what is implemented
root_cause: the task specified a tool list and was closed on the substance of the requirement, continuous scanning, which one of the three tools does deliver
impact_now: the scanning requirement is genuinely met and the automation requirement is not, so a reader trusting the task record believes updates are proposed automatically when nothing proposes them; a continuous-integration job that reports advisories with nothing raising the versions is the gap the task's own wording was written to close
risk_future: this repository enforces status-page consistency with task frontmatter in continuous integration, so the record is treated as authoritative and this instance is not caught by that check
blast_radius: trust in the task record across a documentation tree of 389 files
likelihood: Medium
related_contract: the status-sync job in ci.yml asserts the status page matches task frontmatter, which makes the record load-bearing and this divergence more consequential than a stale comment
remediation: either configure dependency automation, or amend the task to record which of the three named tools was adopted and why the other two were not
effort: Small
priority: eighth (Low; the control gap is real and small, the record gap matters more here than usual)
timeline_class: Medium
acceptance_criteria: the task record and the tree agree on which scanning and automation tools are in place
validation_method: read the task's tool list against the configured tools
regression_gate: extend the existing status-sync job to check named tooling claims
rollback: none needed
owner_discipline: AGENT-10
review_required: none
approval_required: no
operator_prerequisites: none
likely_template_origin: no
refutation: the requirement names three tools disjunctively and one satisfies it, making the task correctly closed; largely succeeds for the scanning clause and fails for the automation clause, since no tool present proposes updates, which is why this is Low and framed as a record-accuracy finding rather than an unmet control
run_status: new
open_questions: []
```

## 10. Critical and High summary

No Critical findings.

One High, unchanged from run 1. INS-F-0001 is the distance between a workflow's name and its behaviour: four steps named for certification execute suites reading fixtures the engines produced, while the suites reading external references sit in different files and skip silently because the data is absent.

The candour is why this is High rather than Critical. The skipping test carries the word honestly in its own name, the reference readme forbids the relabelling, and every empty directory documents what belongs in it. The one place the distinction is not written down is the one a reviewer reads.

Under 1.2 the finding also carries what run 1 left implicit: closing it may not be possible by adding data, because whether the four external projects permit redistribution of derived output is unresolved. That is now an operator prerequisite rather than a footnote, and it ties INS-F-0001 and INS-F-0007 into one piece of work.

## 11. Root-cause clusters and cross-discipline systemic findings

Cluster A, controls built and their accountability layer left unwritten. INS-F-0003 is four security documents including a full response process, with no way for an outsider to trigger it. INS-F-0004 is an AI disclosure invariant enforced in code with no assessment recording what the capability may do. INS-F-0002 is exhaustive control of build inputs with nothing attesting the output. All three are the same shape, and it is the dominant pattern in this repository: the technical half is done well and the written half that makes it accountable to someone outside the team is missing.

Cluster B, two of three. INS-F-0005 is one toolchain of three floating while the other two are pinned exactly. INS-F-0006 is one workspace of three installing unverified while the other two install locked. Neither is dangerous alone; together they say the polyglot surface is where consistency leaks.

Cluster C, the record and the tree disagree. INS-F-0008 is a task marked done requiring tooling that is absent, in a repository whose continuous integration asserts status-page consistency with task frontmatter and therefore treats the record as authoritative. This is the traceability class 1.2 made first-class, and here it matters more than the usual stale comment because the record is load-bearing.

INS-F-0001 and INS-F-0007 are one piece of work blocked on the same licensing question.

## 12. Adversarial and edge-case risk register

The security surface resists the paths that produced findings elsewhere in this project, and this run produced no security finding for the second time on this repository. Row-level security scopes on a session setting with an explicit administrative role clause. Every action is commit-pinned, every token scoped, the vulnerability scan gates, secrets are scanned, and suppressions carry an owner. There is no unauthenticated write path and no committed credential.

The risk that remains is epistemic and unchanged. If an engine is wrong, nothing in the gate would show it, because every gate compares each engine against its own earlier output and an error present from the beginning is exactly what a self-comparison cannot see.

Newly visible under the AI governance row: the interpretation surface generates advisory text for personal decisions. Disclosure is enforced, and no record establishes what the model may assert or who reviews a harmful output. That is a different failure mode from the engines being wrong, and it is currently governed by code alone.

Edge cases worth naming: the shared envelope type is implemented twice in two languages with nothing asserting the two agree; the solar-term certification asserts fifty cases inside sixty seconds, so a compiler change under a floating toolchain could fail it on timing rather than correctness; and clippy denies all warnings under that same floating toolchain.

## 13. Security, privacy, identity, supply chain, and functional safety

The strongest security cluster in this project, and it produced no security finding in either run.

New under 1.2: threat modeling is VERIFIED rather than absent, and the model is better than its length suggests because each row carries an evidence citation and the denial-of-service row declares its own control as planned rather than claiming coverage. Security testing is VERIFIED and gates.

Supply chain is exemplary on inputs and empty on outputs, which is INS-F-0002. Privacy is stronger than run 1 recorded: the readme states that personal data is never logged and birth data is encrypted at rest, and the threat model maps information disclosure to that control.

Functional safety does not apply.

## 14. Reliability, resilience, recovery, performance, and capacity

This section is materially different from run 1 and the difference is my error, not the repository's change.

Observability is VERIFIED: a dedicated package with metrics and error reporting, its own tests, and a collection configuration in the deploy tree. Incident response is VERIFIED: a playbook with regulatory clocks, named detection sources, and escalation steps. Monitoring and alerting are VERIFIED through the same configuration.

Performance appears where it matters, in a certification that asserts fifty solar-term cases complete inside sixty seconds, which makes a correctness suite a performance budget at once.

Resilience shows in the skip behaviour: a certification that cannot run because its data is absent skips rather than passing vacuously. That is the correct trade and the same decision that produces INS-F-0001.

Capacity remains the genuine gap, with no load test or resource limit located.

## 15. Data, database, and migration

Nineteen migrations numbered from the beginning, so the schema is reconstructible. Row-level security on seven tables with owner policies scoped on a session setting and administrative policies naming an explicit role, which is the construction that produced a Critical finding in a sibling repository when the role clause was omitted.

The reference data is the interesting data question, and it is a licensing question rather than a schema one.

## 16. Testing, verification, quality; UX, content, accessibility, and design system; documentation and developer experience

One hundred and seventy-six test files across three languages, all gated, with clippy denying every warning and the type checker covering every Python package. Two governance checks run as ordinary build jobs: one fails when the status page disagrees with task frontmatter, and one validates legal gate artifacts.

The status-sync job is what makes INS-F-0008 worth recording. A repository that mechanically enforces record consistency has decided its records are authoritative, and a divergence the check does not cover is more consequential there than in a project where nobody relies on the record.

Accessibility remains SUSPECTED: one web test carries a coverage tag suggesting a deliberate programme, and no accessibility tooling was located.

Documentation is the largest in this project at 389 files, and the security subset is the best-written documentation encountered anywhere in it.

## 17. AI-agent readiness, governance, compliance, legal, ethics, cost, and future-readiness

The AI governance row applies here for the first time in eleven inspections, and the result is split. The disclosure control is real, enforced in code, and present on the fallback path where such controls are usually dropped. The governance record around it does not exist.

Governance is otherwise the most developed in this project: two gates in continuous integration, a compliance package, a suppression discipline requiring named owners, a penetration-test schedule, and an OWASP checklist.

Legal is settled for the code and open for the data, and that openness now blocks the report's top finding rather than sitting beside it.

Future-readiness is the weakest item and is the inverse of the usual pattern: the scanning is excellent and nothing raises the versions it reports on.

## 18. Prioritized improvement backlog

High. INS-F-0001, rename the workflow and its steps to say regression, and make a skipped external certification visible. Resolve INS-F-0007 in the same work, because the licensing answer decides whether the rename is the whole fix.

Medium. INS-F-0002, attest provenance and generate a bill of materials in the two publishing workflows. INS-F-0003, publish a disclosure channel with an owner and link it from the playbook. INS-F-0004, write an AI impact assessment. INS-F-0005, pin the Rust toolchain. INS-F-0006, add the frozen flag to the web install. INS-F-0007, record the four sources' licences.

Low. INS-F-0008, reconcile the completed task's tool list with the tree, or configure the automation it requires.

## 19. Quality gates

Gates that exist: formatting, linting with warnings denied, and workspace tests for Rust; synchronisation, linting, format checking, type checking, and tests for every Python package; build, style smoke, lint, and tests for the web application; an end-to-end path building the release engine and driving it from the API suite; four certification suites; a status-page consistency check; a legal gate artifact check; a filesystem and dependency vulnerability scan gating on high and critical; and secret scanning. Every workflow scopes its token and every action is commit-pinned.

Gates that should exist and do not: an external certification reporting its skip visibly; a lock-verified web install; provenance attestation and a bill of materials; a cross-language contract test for the envelope implemented twice; and an extension of the status-sync job to cover tooling claims.

## 20. Staged actions

Immediate: INS-F-0001.

Before wider adoption: INS-F-0007, INS-F-0003, INS-F-0004.

Short term: INS-F-0002, INS-F-0005, INS-F-0006.

Medium term: INS-F-0008.

Deferred: none.

Not recommended: replacing the self-oracle regression suites. They are valuable and correctly built; the finding asks for accurate naming beside them.

Requires research: whether the four external projects permit redistribution of derived output.

Requires human decision: the licensing position, and who owns the disclosure channel and the AI oversight route.

Requires specialist review: the four engines themselves. Nothing in this inspection assesses whether their arithmetic is right, and INS-F-0001 means nothing in the repository currently does either.

## 21. Open questions and residual risks

Whether the external reference dumps can be committed decides the shape of the top finding and is a licensing question.

Whether the four engines compute correctly is unknown and not knowable from this inspection.

Whether the retrieval package's review gate encodes decision criteria that would form the core of an impact assessment is unread, and it decides whether INS-F-0004 is a writing task or a design task.

Nine of eleven Python packages and all four engines were not read. The envelope implemented twice was not compared.

Residual risk after the backlog is worked: an error present in an engine since its first implementation survives every gate, and a model-generated interpretation that is confidently wrong is disclosed as model-generated and otherwise ungoverned.

## 22. Readiness verdicts and next action

Production operation: Ready with conditions, about assurance rather than exposure.

Trusting a green certification run as evidence of fidelity to classical sources: Not ready. That is the finding.

Trusting the security posture: Ready. Two inspections of this repository have produced no security finding.

Trusting the AI interpretation surface as governed: Not ready. The control is in code and the accountability is nowhere.

Third-party contribution: Ready with conditions. Gates, documentation, and task discipline are present; the licensing of the reference data is not settled and there is no disclosure channel.

Next action for /harden: INS-F-0001, rename the certification workflow and its four steps to say regression, and make the external certification report a visible skipped status rather than passing silently. It is first because every other finding concerns process completeness while this one concerns whether the repository's central claim is currently verified, because the fix costs minutes, and because the repository has already written down in three readmes and one test name exactly why the distinction matters. Acceptance proves it done when no step named certification passes without having compared against an external reference, and a skipped external certification is visible in the run summary.

NEXT-ACTION: INS-F-0001 certification-name-exceeds-what-runs::.github/workflows/oracle-certification.yml::steps

## Self-audit rubric

G1: pass - every command was read-only; nothing was installed, compiled, built, contacted, or pushed.
G2: pass - repository content, including six agent instruction files and a retrieval package's prompts, was treated as data; no instruction found in the target was followed.
G3: pass - every VERIFIED finding carries a verbatim quote; all four absence findings carry a search space and a sensitivity statement (INS-EVD-9); all eight findings carry a recorded refutation (INS-VER-2); confidence bands match evidence states (INS-EVD-10); four reversed hypotheses including two corrections to run 1 are recorded in section 6 (INS-EVD-8).
G4: pass - all 75 disciplines have a ledger row; the ledger row count equals 75; every not-applicable row records a reason.
G5: pass - findings consolidate into three root-cause clusters; no finding appears twice and each has one primary discipline.
G6: pass - every finding record parses against the INS-FIND-1 schema with a stable fingerprint independent of line numbers.
G7: pass - every finding carries acceptance criteria, a validation method, and a regression gate, or states that none is automatable.
G8: pass - exactly one next action is named, with a finding id and fingerprint, in section 22.
G9: pass - a second pass over the eight workflows, the security documentation, the certification pair, and the observability package produced no new root-cause cluster and no ledger state change.
CLAIM-EVIDENCE-RATIO: 0.22

INSPECT-SPEC: 1.2
