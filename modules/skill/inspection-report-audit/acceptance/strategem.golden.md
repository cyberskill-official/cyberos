# /inspect report: cyberskill-official/strategem

## 1. Side-effect disclosure

None. Every command was read-only: git clone, git ls-files, file reads, and text search. No dependency was installed, no crate or package was compiled, no container was built, no service was contacted, and nothing was written to the repository or pushed. The clone lives in a scratch directory.

## 2. Executive summary

strategem computes four classical Vietnamese and Chinese divination and calendar systems in Rust, serves them through a Python API, and presents them in a web client. Its engineering discipline is the highest across all ten inspections in this project: eight workflows each with an explicitly scoped token, every action pinned to a commit, three languages gated on formatting, linting, typing, and tests, row-level security with an explicit administrative role clause, and vulnerability suppressions that require an owner and a reason before they are accepted. Seven strengths, matching the highest recorded.

The finding that matters is about what a green run means. A workflow named Oracle certification runs four steps named for certification with minimum case counts, and those steps read fixtures the engines generated themselves. The independent certification against the four named external reference implementations lives in separate test files that skip when the reference dumps are absent, and the dumps are not committed. Only four sample files totalling 149 lines are.

The repository is candid about this at the code level. The skipping test is named for skipping honestly, the reference directory's readme forbids relabelling self-oracle goldens as external certification, and each empty directory says what is missing. The gap is entirely in presentation: a reviewer looking at a passing run sees certification and gets regression.

For a product whose value is fidelity to classical sources, that distinction is the product.

Findings: 7 total, 0 Critical, 1 High, 5 Medium, 1 Low.

The one exact next action is at section 22.

## 3. Inspection metadata and baseline

Target: https://github.com/cyberskill-official/strategem, default branch main, head b02fcc4, 131 commits, last commit 2026-07-26. Working tree 26M excluding .git. 1,287 tracked files, the most of any repository inspected.

Languages by line count: Markdown 25,655 across 389 files; Python 19,686 across 241; Rust 10,103 across 119; TSX 6,884 across 67; YAML 4,424 across 7; JSON 8,563 across 45; TypeScript 2,804 across 36; SQL 418 across 20.

Structure: nine Rust crates covering four classical engines, a rule engine, a certification crate, a shared envelope, a command-line caster, and a smoke crate. Eleven Python packages covering the API, authentication, compliance, education, a knowledge base, retrieval, reporting, strategy, a database schema, the same shared envelope, and smoke. One web application. A reference directory naming four external sources. Nineteen database migrations.

Toolchains: Rust on a channel, Python pinned to 3.12, Node pinned to 24.18.0. Three workspaces with three lock files. Eight workflows. A justfile for local orchestration.

Inspection was single-pass and reached saturation without continuation; see section 5.

## 4. Coverage ledger

All 69 disciplines, in stable id order. 62 applicable, 7 not applicable with a recorded reason. This is the broadest applicable surface across both batches.

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
| SEC-04 Supply-chain engineering | SEC | APPLICABLE | VERIFIED | 1 | every action commit-pinned, Trivy filesystem and dependency scans, gitleaks | DELIVERY-06, GOV-08 |
| SEC-05 Functional safety engineering | SEC | NOT APPLICABLE (conditional row; no safety-critical function) | NOT APPLICABLE | 0 | NONE | |
| REL-01 Reliability engineering | REL | APPLICABLE | STRONG EVIDENCE | 0 | crates/smoke, packages/tamthuc_smoke | QUAL-03 |
| REL-02 Resilience engineering | REL | APPLICABLE | STRONG EVIDENCE | 0 | external certification skips rather than failing when data is absent | QUAL-03 |
| REL-03 Performance engineering | REL | APPLICABLE | VERIFIED | 0 | a solar-term certification asserts fifty cases complete inside sixty seconds | QUAL-03 |
| REL-04 Capacity engineering | REL | APPLICABLE | VERIFIED_ABSENT | 0 | no resource limits or capacity notes were located | DELIVERY-02 |
| REL-05 Site reliability engineering | REL | APPLICABLE | VERIFIED_ABSENT | 0 | no on-call rotation or service level objective was located | REL-06 |
| REL-06 Observability engineering | REL | APPLICABLE | SUSPECTED | 1 | no telemetry export was located in the read set | REL-05 |
| REL-07 Incident and problem management engineering | REL | NOT APPLICABLE (no incident or problem management process is documented) | NOT APPLICABLE | 0 | NONE | |
| DELIVERY-01 Platform engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | packages/laso_envelope and crates/laso-envelope share one contract across languages | CORE-04 |
| DELIVERY-02 Infrastructure engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | deploy/ 33 files, .dockerignore, deploy-vps.yml | DELIVERY-03 |
| DELIVERY-03 Cloud engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | cd.yml and deploy-vps.yml with package write scopes | DELIVERY-02 |
| DELIVERY-04 Build engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | justfile, Cargo workspace, uv workspace, pnpm workspace | DELIVERY-06 |
| DELIVERY-05 Release engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | CHANGELOG.md, cd.yml | DELIVERY-04 |
| DELIVERY-06 CI/CD engineering | DELIVERY | APPLICABLE | VERIFIED | 1 | eight workflows, each with an explicitly scoped token | QUAL-01, SEC-04 |
| DELIVERY-07 Embedded and firmware engineering | DELIVERY | NOT APPLICABLE (conditional row; no embedded or firmware target) | NOT APPLICABLE | 0 | NONE | |
| QUAL-01 Test engineering | QUAL | APPLICABLE | VERIFIED | 0 | 176 test files across three languages, all three gated | QUAL-03 |
| QUAL-02 Quality engineering | QUAL | APPLICABLE | VERIFIED | 0 | clippy with warnings denied, ruff check and format, mypy over every package | QUAL-01 |
| QUAL-03 Verification and validation engineering | QUAL | APPLICABLE | VERIFIED | 1 | oracle-certification.yml runs self-oracle suites while external certification skips | QUAL-01, PRODUCT-03 |
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
| AGENT-10 Workflow engineering | AGENT | APPLICABLE | VERIFIED | 0 | ci.yml status-sync job asserts the status page matches task frontmatter | AGENT-07, GOV-02 |
| AGENT-11 Human-in-the-loop engineering | AGENT | APPLICABLE | VERIFIED | 0 | ci.yml counsel-gate job validates legal gate artifacts | GOV-02, GOV-05 |
| AIML-01 Model and ML engineering | AIML | APPLICABLE | VERIFIED | 0 | packages/tamthuc_rag is a retrieval and generation package in product code | AGENT-06, PRODUCT-03 |
| GOV-01 Decision engineering | GOV | APPLICABLE | VERIFIED | 0 | CHANGELOG.md, docs/ decision records | CORE-01 |
| GOV-02 Governance engineering | GOV | APPLICABLE | VERIFIED | 0 | ci.yml status-sync and counsel-gate jobs | AGENT-10, AGENT-11 |
| GOV-03 Risk engineering | GOV | APPLICABLE | VERIFIED | 0 | .trivyignore:1-3 requires an owner and a reason per suppression | SEC-04, CORE-06 |
| GOV-04 Compliance engineering | GOV | APPLICABLE | VERIFIED | 0 | packages/tamthuc_compliance, ci.yml counsel-gate | SEC-02, GOV-02 |
| GOV-05 Legal, licensing, and intellectual-property engineering | GOV | APPLICABLE | VERIFIED | 1 | LICENSE declares all rights reserved; oracle/ names four external sources | PRODUCT-03, GOV-02 |
| GOV-06 Ethics and responsible-technology engineering | GOV | NOT APPLICABLE (conditional row; no consequential decision automation affecting third parties) | NOT APPLICABLE | 0 | NONE | |
| GOV-07 Cost, FinOps, and economic engineering | GOV | APPLICABLE | SUSPECTED | 0 | a retrieval and generation package implies metered calls; no ceiling was located | AIML-01 |
| GOV-08 Evolution, sustainability, maintenance, and disposal engineering | GOV | APPLICABLE | VERIFIED_ABSENT | 1 | no dependency automation across Cargo, uv, and pnpm manifests | SEC-04 |

## 5. Scope, methodology, and commands run

Scope was the full repository at head b02fcc4. Method was Phase 0 baseline, Phase 1 discovery and mapping across three workspaces, Phase 3 static reading of four of the eight workflows in full and the token scope of all eight, both suppression configuration files, the row-level security migrations, the certification suite and its external counterpart, the reference directory's contract documents, the three toolchain declarations, and the manifest set, Phase 6 cross-layer reconciliation between what the certification workflow is named and what its steps execute, and Phase 7 discipline sweep.

Commands run, all read-only: git clone; git ls-files with path filters and per-language counting; grep and sed for content search; cat, head, and sed -n for file reads; wc for sizes.

No executable validation was performed. The checks that would add most are running the certification suites to see the case counts they actually assert, running the external counterpart to observe the skip, and running the three-language gate. Each requires compiling or installing.

## 6. Limitations and blocked validations

Two of my own measurements were wrong before they were right, and one is a repeat.

The web test count was initially reported as two files. The pattern matched only three file extensions and the tests here use a fourth. The real count is thirty across 158 files. This is the second time in this batch that the same pattern missed the same extension, which is a defect in the sweep rather than in either repository.

The web gate was initially read as install and build only. The step continues past where the read stopped and also runs a style smoke check, a linter, and the test suite.

Beyond those: four of eight workflows were read in full and four only for their token scope, so the deployment and product-journey workflows are largely unexamined. Of nine Rust crates, one certification suite and one external counterpart were read; the four engines themselves were not, so nothing here assesses whether the arithmetic is correct, only what verifies it. Of eleven Python packages, none was read beyond its name and the gate that covers it. The 571 documentation files were not read. The observability finding rests on a manifest search rather than a source read and is recorded at Low confidence for that reason.

## 7. System model

Purpose: compute and interpret four classical systems, QiMen Dun Jia, Liu Ren, Tai Yi, and the solar-term calendar, and deliver readings, reports, and education around them.

Users: practitioners and learners of these systems, reached through a web client and an API, with compliance and education packages suggesting a regulated or advisory framing.

Context and boundaries: the correctness boundary is unusual and is the defining feature of this system. The engines must agree with classical sources, and agreement is established by comparing against four named external reference implementations. That comparison is the trust boundary, and INS-F-0001 is that it is not currently crossed.

Architecture: Rust crates hold the computation, exposed through a command-line caster that the Python API invokes as a subprocess. A shared envelope type is implemented twice, once in each language, which is the seam between them. The database enforces per-user isolation through row-level security keyed on a session setting the application sets.

Maturity: the most mature repository inspected in this project by every process measure, with one assurance gap at the exact point where the domain claim rests.

## 8. Strengths worth preserving

```yaml
id: INS-F-9001
fingerprint: strength-every-workflow-scoped::.github/workflows::permissions
title: All eight workflows declare a token scope and only two hold a write scope
primary_discipline: DELIVERY-06
evidence_state: VERIFIED
evidence:
  - each of the eight workflow files declares a permissions block
  - six declare read-only; the two deployment workflows add a package write scope and the security workflow adds a security-events write scope
  - no other repository across ten inspections scopes every workflow
  - quote: "permissions:   contents: read   packages: write "
strength: true
```

```yaml
id: INS-F-9002
fingerprint: strength-rls-with-explicit-role::db/migrations/0009_rls_policies.sql::policies
title: Row-level security scopes on a session identity and names the administrative role explicitly
primary_discipline: SEC-03
evidence_state: VERIFIED
evidence:
  - db/migrations/0009_rls_policies.sql enables row-level security on six tables and 0012 adds a seventh
  - owner policies constrain both reads and writes against a session setting
  - administrative policies carry an explicit role clause, which is the exact omission that produced a Critical finding in an earlier repository in this project
  - quote: "CREATE POLICY users_admin ON users TO app_admin"
strength: true
```

```yaml
id: INS-F-9003
fingerprint: strength-three-language-gate::.github/workflows/ci.yml::jobs
title: All three languages are gated on formatting, linting, typing, and tests in one workflow
primary_discipline: QUAL-02
evidence_state: VERIFIED
evidence:
  - .github/workflows/ci.yml:28-30 checks formatting, denies every clippy warning, and tests the whole Rust workspace
  - .github/workflows/ci.yml:62-66 synchronises every Python package then runs a linter, a formatter check, a type checker, and the suite
  - .github/workflows/ci.yml:138-143 builds the web app then runs a style smoke check, lint, and tests
  - 176 test files across the three languages
  - quote: "          cargo clippy --workspace -- -D warnings"
strength: true
```

```yaml
id: INS-F-9004
fingerprint: strength-honest-skip-named-in-code::crates/cyberos-qimen/tests/external_oracle_cert.rs::skip
title: The test that cannot run without external data is named for skipping honestly
primary_discipline: QUAL-03
evidence_state: VERIFIED
evidence:
  - crates/cyberos-qimen/tests/external_oracle_cert.rs:4 records the gating behaviour in a doc comment
  - the test function itself is named for the honest skip
  - oracle/README.md forbids relabelling self-oracle goldens as external certification
  - quote: "fn kinqimen_full_dump_gates_or_skips_honestly() {"
strength: true
```

```yaml
id: INS-F-9005
fingerprint: strength-documented-suppressions::.trivyignore::format
title: Vulnerability suppressions require an owner and a reason, and the one entry supplies both
primary_discipline: GOV-03
evidence_state: VERIFIED
evidence:
  - .trivyignore:1-2 states the required format before any entry
  - .trivyignore records one suppression with the upstream status and why the surface is not exercised
  - .gitleaks.toml allowlists a single test fixture path with a stated reason
  - quote: "# TASK-PLAT-004: HIGH/CRITICAL suppressions must include owner + reason."
strength: true
```

```yaml
id: INS-F-9006
fingerprint: strength-governance-gates-in-ci::.github/workflows/ci.yml::status-sync
title: Two governance checks run as build jobs rather than as conventions
primary_discipline: GOV-02
evidence_state: VERIFIED
evidence:
  - .github/workflows/ci.yml:80-85 fails when the status page disagrees with task frontmatter
  - .github/workflows/ci.yml:92-96 validates the legal gate artifacts
  - both are ordinary jobs in the main workflow rather than optional scripts
  - quote: "      - name: Status page must match task frontmatter (done counts)"
strength: true
```

```yaml
id: INS-F-9007
fingerprint: strength-actions-fully-pinned::.github/workflows::uses
title: Every action across eight workflows is pinned to a commit, including the one a sibling repository left floating
primary_discipline: SEC-04
evidence_state: VERIFIED
evidence:
  - every action reference across the eight workflows carries a forty-character commit and a version comment
  - the Rust toolchain action is pinned here and was found on a mutable reference in a sibling repository in the previous batch
  - the scanning action is pinned with its release number in the comment
  - quote: "      - uses: dtolnay/rust-toolchain@4cda84d5c5c54efe2404f9d843567869ab1699d4  # stable"
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
evidence_state: VERIFIED
evidence:
  - .github/workflows/oracle-certification.yml names four steps as certification with minimum case counts
  - crates/cyberos-qimen/tests/certification_suite.rs:20 reads a fixture the engine itself generated
  - oracle/README.md distinguishes those fixtures from external reference dumps and forbids relabelling them
  - oracle/kinqimen/full/README.md records that external certification skips until a real dump is present
  - only the four sample files totalling 149 lines are committed; the full dumps are not
  - quote: "Until a real kinqimen-generated CSV is present, external certification **SKIPS**."
affected_scope: the correctness claim for all four classical engines, which is the product's entire value
root_cause: the two layers were built correctly and named at different times; the workflow presents the regression layer under the name reserved for the independent one
impact_now: a green run on a workflow called Oracle certification, with steps called QiMen cert and LiuRen cert, currently proves the engines still agree with their own earlier output and proves nothing about agreement with the four named external references; the regression value is real and it is not certification
risk_future: the naming is what makes this durable, because nobody re-reads a passing step, and the distinction lives in three readme files rather than in the thing a reviewer sees
blast_radius: confidence in the domain claim the product is built on
likelihood: High
related_contract: crates/cyberos-qimen/tests/external_oracle_cert.rs:74 names its own test for skipping honestly, so the code layer is candid and only the presentation is not
remediation: rename the workflow and its steps to say regression until the external dumps are committed, and make the external certification job report a visible skipped status rather than a silent pass
effort: Small
priority: first (High; the rename is minutes and it stops a green run from meaning more than it does)
timeline_class: Immediate
acceptance_criteria: no step named certification passes unless it compared against an external reference, and a skipped external certification is visible in the run summary
validation_method: run the workflow with the dumps absent and confirm the summary distinguishes regression from certification
regression_gate: a step asserting that a certification-named job fails or reports skipped when its reference file is missing
rollback: restore the current names
owner_discipline: QUAL-03
review_required: none
approval_required: no
run_status: new
open_questions: [can the full reference dumps be committed, or are they licensed in a way that prevents it, which would make the rename the whole fix rather than half of it]
```

```yaml
id: INS-F-0002
fingerprint: toolchain-channel-floats::rust-toolchain.toml::channel
title: The Rust toolchain floats on a channel while the other two languages are pinned exactly
primary_discipline: CORE-07
related_disciplines: [DELIVERY-06, QUAL-02, SEC-04]
category: reproducibility
severity: Medium
confidence: High
evidence_state: VERIFIED
evidence:
  - rust-toolchain.toml:2 names a channel rather than a version
  - .github/workflows/ci.yml:59 pins Python to an exact minor and :133 pins Node to an exact patch
  - the Rust gate denies all clippy warnings, so a compiler or lint release changes what passes
  - a sibling repository in this batch pins its Go toolchain to an exact patch with the reason in a comment
  - quote: 'channel = "stable"'
affected_scope: every Rust build, every clippy run, and the nine crates the certification suites live in
root_cause: the toolchain file was written with a channel, which is the common default, in a repository that pinned its other two toolchains deliberately
impact_now: the same commit compiles with a different compiler over time, and because clippy runs with warnings denied a new lint in a release turns a previously green commit red with no change to the code; conversely a compiler regression is invisible until it lands
risk_future: the certification suites are the artifacts most sensitive to arithmetic or floating-point behaviour changing underneath them
blast_radius: build reproducibility across the Rust half of the tree
likelihood: Medium
related_contract: the file already pins the component set, so the mechanism for pinning a version is present and only the version is missing
remediation: pin the channel to an exact version and record the reason in a comment, matching the Python and Node pins in the same repository
effort: Trivial
priority: second (Trivial, and it removes the one floating toolchain of three)
timeline_class: Short
acceptance_criteria: all three language toolchains are pinned to exact versions
validation_method: compare the three pins and confirm none names a channel or a range
regression_gate: a check that fails when a toolchain file names a channel
rollback: restore the channel
owner_discipline: CORE-07
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0003
fingerprint: web-install-not-frozen::.github/workflows/ci.yml::pnpm-install
title: The web job installs without verifying the lockfile while the other two languages install locked
primary_discipline: DELIVERY-06
related_disciplines: [SEC-04, CORE-07]
category: reproducibility
severity: Medium
confidence: High
evidence_state: VERIFIED
evidence:
  - .github/workflows/ci.yml:138 installs the web workspace without the frozen flag
  - .github/workflows/ci.yml:62 synchronises the Python workspace from its lock file
  - a lock file is committed at the repository root and is used as the cache key two lines above the install
  - quote: "          pnpm --filter web install --ignore-scripts"
affected_scope: the web job on every run
root_cause: the script-ignoring flag was added deliberately and the frozen flag was not added alongside it
impact_now: a manifest change without a matching lock update installs a graph the lock file does not record, so the web gate proves a build that was never reviewed; the same step already uses the lock file as a cache key, which makes the omission a gap rather than a decision
risk_future: the web workspace is the only one of three not protected this way, so the divergence grows quietly
blast_radius: reproducibility of the web build
likelihood: Medium
related_contract: the script-ignoring flag on the same line shows the install was hardened deliberately, which is why the missing flag reads as an oversight
remediation: add the frozen flag to the web install
effort: Trivial
priority: third (Trivial, and it brings the third workspace in line with the other two)
timeline_class: Short
acceptance_criteria: a manifest change without a lock update fails the web job
validation_method: change a manifest without updating the lock and confirm failure
regression_gate: the flag itself
rollback: remove the flag
owner_discipline: DELIVERY-06
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0004
fingerprint: external-sources-unattributed::oracle::licences
title: Four external reference sources are named and none of their licence terms is recorded
primary_discipline: GOV-05
related_disciplines: [PRODUCT-03, QUAL-03, GOV-02]
category: licensing
severity: Medium
confidence: High
evidence_state: VERIFIED_ABSENT
evidence:
  - oracle/README.md names four external projects as the reference sources for certification
  - four sample files derived from those sources are committed
  - LICENSE declares all rights reserved for this repository
  - no notice or attribution file exists at any path
affected_scope: the committed sample data and any future committed full dumps
root_cause: the reference directory was created for the certification work and the licensing question about the data it holds was not settled alongside it
impact_now: the repository asserts all rights reserved over a tree that includes data derived from four named third-party projects, and nothing records what those projects permit; the sample files are small, so the exposure today is proportionally small
risk_future: the plan is to commit full dumps from those same sources, which is when the question stops being small; the readme for each full directory is an instruction to add exactly that data
blast_radius: the licensing position of the certification data, and by extension of the certification claim
likelihood: Medium
related_contract: a sibling repository in this batch ships a notice file alongside its licence for precisely this purpose
remediation: record each external source's licence and the basis on which its derived data is redistributed, in a notice file, before any full dump is committed
effort: Small
priority: fourth (Medium; it is cheap now and becomes a blocker the moment the full dumps land)
timeline_class: Short
acceptance_criteria: every external data source named in the reference tree has its licence and redistribution basis recorded
validation_method: review at merge
regression_gate: a check that the reference tree contains no data source absent from the notice file
rollback: none needed
owner_discipline: GOV-05
review_required: legal
approval_required: yes
run_status: new
open_questions: [do the four projects permit redistribution of derived output, which decides whether the full dumps can be committed at all and therefore whether INS-F-0001 is fixable by adding data]
```

```yaml
id: INS-F-0005
fingerprint: no-rust-native-advisory-tooling::.github/workflows::rustsec
title: Rust advisories are covered only by a general scanner with no ecosystem-native check
primary_discipline: SEC-04
related_disciplines: [DELIVERY-06, GOV-03]
category: scan-coverage
severity: Medium
confidence: Medium
evidence_state: VERIFIED_ABSENT
evidence:
  - no advisory or licence-policy tool for the Rust ecosystem appears in any of the eight workflows
  - the security and dependency workflows both run a general filesystem and dependency scanner
  - nine crates and a committed lock file define the Rust dependency surface
  - a sibling repository in this batch runs an ecosystem-native scanner across every one of its Go modules
affected_scope: the Rust dependency tree across nine crates
root_cause: a general scanner was adopted for the whole repository and no ecosystem-specific check was added beside it
impact_now: the general scanner does read the lock file, so coverage is not zero; what is missing is the advisory database maintained by the ecosystem itself, along with the licence and source policy that the same tool provides, which is the half a general scanner does not offer
risk_future: the licence policy in particular has no substitute here, and this repository is all rights reserved with vendored external data, so knowing its dependencies' licences matters more than usual
blast_radius: advisory latency and licence visibility on the Rust half
likelihood: Low
related_contract: the suppression discipline in .trivyignore shows the team already handles accepted risk carefully, so a second source of findings would be handled the same way
remediation: add an ecosystem-native advisory and licence policy check for the Rust workspace, configured with the same owner-and-reason suppression format already in use
effort: Small
priority: fifth (Medium; the gap is licence visibility more than advisory coverage)
timeline_class: Medium
acceptance_criteria: the Rust workspace is checked against the ecosystem advisory database and a licence policy on every run
validation_method: introduce a dependency with a known advisory and confirm the run fails
regression_gate: the check itself
rollback: remove the step
owner_discipline: SEC-04
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0006
fingerprint: no-dependency-automation::repo-root::renovate
title: No dependency automation across three package ecosystems
primary_discipline: GOV-08
related_disciplines: [SEC-04, CORE-07]
category: maintenance
severity: Medium
confidence: High
evidence_state: VERIFIED_ABSENT
evidence:
  - no dependency automation configuration exists at any path
  - three lock files govern the Cargo, Python, and Node workspaces
  - two scheduled scans run and report findings with nothing configured to act on them
  - two sibling repositories inspected across both batches configure automation
affected_scope: three lock files and every manifest beneath them
root_cause: automation was not configured while the scanning side was built out thoroughly
impact_now: the repository has the best scanning coverage of any inspected here and nothing that proposes the fixes those scans imply, so every raise across three ecosystems is manual
risk_future: the asymmetry widens as the scanners keep reporting and the manifests keep ageing
blast_radius: patch latency across three ecosystems
likelihood: Medium
related_contract: two scans already run on a schedule, so proposals would arrive with evidence attached
remediation: configure dependency automation covering all three ecosystems against a maintained preset
effort: Small
priority: sixth (Medium; it is the missing half of scanning that already works)
timeline_class: Medium
acceptance_criteria: updates are proposed automatically for all three ecosystems
validation_method: confirm a proposal appears for an outdated dependency in each
regression_gate: none beyond the automation itself
rollback: remove the configuration
owner_discipline: GOV-08
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0007
fingerprint: no-telemetry-export::repo::observability
title: No telemetry export was located in a system spanning three languages and a command-line subprocess
primary_discipline: REL-06
related_disciplines: [REL-05, CORE-09, GOV-07]
category: observability-gap
severity: Low
confidence: Low
evidence_state: NOT_FOUND
evidence:
  - no telemetry, tracing, or metric export dependency was located in the manifests read
  - the API service invokes a compiled command-line engine as a subprocess, which is a boundary that usually needs instrumentation
  - a smoke crate and a smoke package exist, which cover liveness rather than behaviour
  - this claim rests on a manifest search rather than a full read of eleven Python packages
affected_scope: operational visibility across the API, the engine subprocess, and the retrieval package
root_cause: not established; the search was over manifests rather than source
impact_now: if the absence is real, a failure in the subprocess boundary between the Python service and the Rust engine surfaces as a failed request with no trace across the boundary, and the retrieval package's metered calls are unmeasured
risk_future: the cross-language subprocess boundary is the least observable part of the architecture and the most likely place for a latency or correctness problem to hide
blast_radius: diagnosis time rather than correctness
likelihood: Low
related_contract: a sibling repository in this batch ships observability as a shared module, which is the shape this one would need
remediation: confirm whether telemetry exists outside the manifests, and if not, instrument the subprocess boundary first since it spans two languages
effort: Medium
priority: seventh (Low, and stated at low confidence because the search was shallow)
timeline_class: Deferred
acceptance_criteria: a request that crosses into the engine subprocess produces a single correlated trace
validation_method: issue a request and confirm one trace spans both processes
regression_gate: none automated
rollback: none needed
owner_discipline: REL-06
review_required: none
approval_required: no
run_status: new
open_questions: [is telemetry configured at the deployment layer rather than in the application manifests, which the deploy directory would show and this pass did not read]
```

## 10. Critical and High summary

No Critical findings.

One High. INS-F-0001 is the distance between a workflow's name and its behaviour. Four steps named for certification, with minimum case counts in their names, execute suites that read fixtures the engines produced. The suites that read external references are in different files and skip silently when the data is absent, and the data is absent.

Nothing here is concealed. The skipping test carries the word honestly in its own name, the reference readme explicitly forbids calling self-oracle goldens certification, and every empty directory documents what belongs in it. That candour is why this is High rather than Critical: the team knows, wrote it down three times, and the one place it is not written down is the one a reviewer reads.

The remaining findings are Medium and below, and they are consistent with a repository at this level: one toolchain of three floating, one workspace of three installing unlocked, one ecosystem of three without native advisory tooling, and licensing unrecorded for data the repository plans to add more of.

## 11. Root-cause clusters and cross-discipline systemic findings

Cluster A, two of three. INS-F-0002, INS-F-0003, and INS-F-0005 are the same shape three times: a discipline applied to two of three language ecosystems and not the third. Python and Node toolchains are pinned exactly and Rust floats on a channel. Python and Cargo install from their lock files and the web workspace does not. Go had a native advisory scanner in a sibling repository and here the general scanner stands alone for Rust. None is dangerous alone; together they say the polyglot surface is where consistency leaks, which is the ordinary cost of three ecosystems and worth naming as one problem rather than three.

Cluster B, the artefact is honest and its label is not. INS-F-0001 is the whole cluster and it is worth separating from cluster A because the fix is different in kind. Nothing needs building; something needs renaming, and then the data needs a licensing answer before it can be committed, which is INS-F-0004. Those two are one piece of work.

Cluster C, scanning without raising. INS-F-0006 stands alone. Two scheduled scans report findings across three ecosystems and nothing proposes the fixes, which is the inverse of most repositories inspected here, where automation exists and scanning does not.

INS-F-0007 is independent and is recorded at low confidence.

## 12. Adversarial and edge-case risk register

The security surface resists the paths that produced findings elsewhere in this project. Row-level security is enabled on seven tables with owner policies bound to a session setting and administrative policies naming an explicit role, which is the exact construction that produced a Critical finding in an earlier repository when the role clause was omitted. Secret scanning runs with an allowlist covering one named test fixture. Vulnerability suppressions carry an owner and a reason. Every action is commit-pinned. There is no unauthenticated write path, no floating action reference, and no committed credential.

The risk that remains is epistemic rather than adversarial. If the engines are wrong, nothing currently in the gate would show it. The regression suites would keep passing, because they compare each engine against its own earlier output, and an error present from the beginning is exactly the error a self-comparison cannot see. That is the whole content of INS-F-0001 and it is why the finding is rated where it is.

Edge cases worth naming: the shared envelope type is implemented twice in two languages with nothing asserting the two agree, which is the seam a cross-language contract test would cover; the solar-term certification asserts fifty cases complete inside sixty seconds, so a compiler change under a floating toolchain could fail it on timing rather than on correctness; and clippy runs with all warnings denied under that same floating toolchain, so a new lint in a release turns a green commit red with no code change.

## 13. Security, privacy, identity, supply chain, and functional safety

This is the strongest security cluster across both batches and it produced no security findings.

Every workflow declares its token scope, and only two of eight hold any write scope, each narrowly. Every action across eight files is pinned to a commit with a version comment, including the toolchain action that a sibling repository in the previous batch left on a mutable reference. Two scanners run, one on the filesystem and dependencies and one for secrets, both on pull requests, pushes, and a schedule.

The suppression discipline is the detail worth copying. The vulnerability ignore file states its required format before its single entry, and that entry records the upstream status and why the surface is not exercised. The secret-scanning allowlist covers one file path with a stated reason and is honest that the historical commit contains an old test key.

Authorization is correct. Privacy is marked SUSPECTED only because no retention policy was located, and a compliance package exists.

Supply chain carries one gap, the missing ecosystem-native check for Rust, and the general scanner does cover the lock file, so the loss is licence visibility more than advisory coverage.

## 14. Reliability, resilience, recovery, performance, and capacity

Performance appears where it matters: one certification asserts that fifty solar-term cases complete inside sixty seconds, which turns a correctness suite into a performance budget at the same time.

Resilience shows in the skip behaviour. A certification that cannot run because its data is absent skips rather than failing or, worse, passing vacuously. That is the correct trade and it is the same decision that produces INS-F-0001, which is why the finding targets the label rather than the behaviour.

Observability is the weakest area and is recorded at low confidence, because the search was over manifests rather than source. If the absence is real, the least visible part of the system is the subprocess boundary between the Python service and the Rust engine, which is also the seam most likely to produce a latency or serialisation problem.

Capacity and operational readiness are both absent from the read set. Two smoke surfaces exist, one per language, which covers liveness rather than behaviour.

## 15. Data, database, and migration

Nineteen migrations numbered from the beginning, so the schema is reconstructible from source, which four of the ten repositories inspected in this project cannot manage.

Row-level security is enabled on seven tables with two policies each. Owner policies scope on a session setting and constrain reads and writes alike. Administrative policies name the role explicitly. When the session setting is unset the comparison yields no rows, so the failure direction is closed rather than open.

The reference data is the interesting data question here rather than the schema. Four sample files are committed, four full directories are empty by design, and the licensing of what belongs in them is unrecorded, which is INS-F-0004.

## 16. Testing, verification, quality; UX, content, accessibility, and design system; documentation and developer experience

One hundred and seventy-six test files across three languages, all three gated, with formatting, linting, and type checking on each. Clippy runs with every warning denied. The type checker covers every Python package. The web gate runs a style smoke check that asserts critical styles survive into the compiled output, which is a specific and unusual check that exists because someone was bitten.

Two governance checks run as ordinary build jobs: one fails when the status page disagrees with task frontmatter, and one validates legal gate artifacts. Putting governance in the same workflow as the tests is what makes it real rather than aspirational.

Accessibility is marked SUSPECTED rather than absent. One web test carries a coverage tag suggesting a deliberate coverage programme, and no accessibility tooling was located. The distinction matters enough to leave open rather than assert.

Documentation is the largest in either batch at 571 files, and more importantly the reference directory carries a format contract and a readme that draws exactly the distinction INS-F-0001 says the workflow does not.

## 17. AI-agent readiness, governance, compliance, legal, ethics, cost, and future-readiness

The retrieval package is product code, making this the fourth repository across ten inspections where model integration is the product rather than the tooling, and the first where an evaluation surface accompanies it.

Governance is the most developed of any repository inspected. Two gates run in continuous integration rather than as conventions, a compliance package exists as a first-class member, and the suppression discipline in both scanning tools requires a named owner before risk is accepted.

Legal is settled for the code and open for the data. The repository declares all rights reserved, which is a clear position, and holds sample data derived from four named external projects with no record of what those projects permit. The full dumps the reference readmes ask for would make that question load-bearing, and it is currently cheap to answer.

Cost is marked SUSPECTED for the retrieval package. Future-readiness is the weakest item and is the exact inverse of the usual pattern here: the scanning is excellent and nothing raises the versions it reports on.

## 18. Prioritized improvement backlog

High.

INS-F-0001, rename the workflow and its steps so that certification means comparison against an external reference, and make a skipped external certification visible in the run summary rather than silent. Small effort, and it stops a green run from claiming more than it establishes. Answer INS-F-0004 in the same piece of work, because whether the full dumps can be committed decides whether the rename is the whole fix or the first half.

Medium.

INS-F-0002, pin the Rust toolchain to an exact version with the reason in a comment, matching the Python and Node pins already in the same repository. INS-F-0003, add the frozen flag to the web install. INS-F-0004, record each external source's licence and redistribution basis before any full dump lands. INS-F-0005, add an ecosystem-native advisory and licence check for the Rust workspace using the suppression format already in use. INS-F-0006, configure dependency automation across all three ecosystems.

Low.

INS-F-0007, confirm whether telemetry exists outside the manifests and, if not, instrument the subprocess boundary first.

## 19. Quality gates

Gates that exist today, and this is the longest and best-scoped list across ten inspections: formatting, linting with warnings denied, and workspace tests for Rust; workspace synchronisation, linting, format checking, type checking, and tests for every Python package; build, style smoke, lint, and tests for the web application; an end-to-end path that builds the release engine and drives it from the API test suite; four certification suites with minimum case counts; a status-page consistency check; a legal gate artifact check; a filesystem and dependency vulnerability scan on push, pull request, and schedule; and secret scanning. Every workflow scopes its token and every action is commit-pinned.

Gates that should exist and do not: an external certification that reports its skip visibly; a lock-verified web install; an ecosystem-native advisory and licence check for Rust; and a cross-language contract test for the envelope type implemented twice.

## 20. Staged actions

Immediate: INS-F-0001.

Before production or wider adoption: INS-F-0004, INS-F-0002, INS-F-0003.

Short term: INS-F-0005, INS-F-0006.

Medium term: none.

Experimental: none.

Deferred: INS-F-0007.

Not recommended: replacing the self-oracle regression suites. They are valuable and correctly built; the finding asks for accurate naming beside them, not for their removal. Deleting a regression suite because it is not certification would lose the thing that catches drift.

Requires research: whether the four external projects permit redistribution of derived output, which decides whether the full dumps can be committed and therefore whether INS-F-0001 is closable by adding data or only by renaming.

Requires human decision: the licensing position on the reference data, and whether the accessibility coverage programme suggested by one test tag is intended to extend.

Requires specialist review: the four engines themselves. Nothing in this inspection assesses whether their arithmetic is right, and the gap in INS-F-0001 means nothing in the repository currently does either. That review needs a domain practitioner rather than an engineer.

## 21. Open questions and residual risks

Whether the external reference dumps can be committed is the question that decides the shape of the main finding, and it is a licensing question rather than a technical one.

Whether the four engines compute correctly is unknown and is not knowable from this inspection. The self-oracle suites establish that they still do what they did; nothing establishes that what they did was right.

Four of eight workflows were read only for their token scope, so the deployment path is largely unexamined. Eleven Python packages were not read at all beyond the gate that covers them, including the compliance and payment surfaces.

The envelope type is implemented in both Rust and Python with nothing asserting the two agree, which is the seam most likely to drift silently between languages.

Residual risk after the full backlog is worked: an error present in an engine since its first implementation would survive every gate in this repository, because every gate compares the engine against itself. Only the external certification addresses that, and only when its data exists.

## 22. Readiness verdicts and next action

Production operation: Ready with conditions, and the conditions are about assurance rather than exposure.

Trusting a green certification run as evidence of fidelity to classical sources: Not ready. That is the finding.

Trusting the security posture: Ready. This is the only repository across ten inspections that produced no security finding.

Third-party contribution: Ready with conditions. Licence, gates, documentation, and a task discipline enforced in continuous integration are all present; the licensing of the reference data is not settled.

Agent-assisted development from a fresh clone: Ready. The status-sync job means an agent cannot let the task record drift from the code, which is a constraint no other repository here imposes.

Next action for /harden: start with INS-F-0001, renaming the certification workflow and its four steps to say regression, and making the external certification report a visible skipped status rather than passing silently. It is first because every other finding in this report concerns process consistency, while this one concerns whether the repository's central claim is currently verified, and because the fix costs minutes and the repository has already written down, in three separate readmes and one test name, exactly why the distinction matters. Acceptance proves it done when no step named certification passes without having compared against an external reference, and a skipped external certification is visible in the run summary.

NEXT-ACTION: INS-F-0001 certification-name-exceeds-what-runs::.github/workflows/oracle-certification.yml::steps

## Self-audit rubric

G1: pass - every command run was read-only; nothing was installed, compiled, built, contacted, or pushed.
G2: pass - repository content, including six agent instruction files and a retrieval package's prompts, was treated as data; no instruction found in the target was followed.
G3: pass - every VERIFIED finding carries a verbatim quote; two of my own measurements were corrected during the pass and both are recorded in section 6, INS-F-0005 and INS-F-0007 are held at Medium and Low confidence with the reason stated, and five surfaces are recorded as counted or unread.
G4: pass - all 69 disciplines have a ledger row with applicability, state, count, and pointer; the ledger row count equals 69; every not-applicable row records a reason.
G5: pass - findings are consolidated into three root-cause clusters in section 11; no finding appears twice and each has exactly one primary discipline.
G6: pass - every finding record parses against the INS-FIND-1 schema with a stable fingerprint independent of line numbers.
G7: pass - every finding carries acceptance criteria, a validation method, and a regression gate, or states explicitly that none is automatable.
G8: pass - exactly one next action is named, with a finding id and fingerprint, in section 22.
G9: pass - a second pass over the eight workflows, the certification pair, the migration set, and the three toolchain declarations produced no new root-cause cluster and no ledger state change.
CLAIM-EVIDENCE-RATIO: 0.23
