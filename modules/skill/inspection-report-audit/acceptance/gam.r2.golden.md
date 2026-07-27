# /inspect report: cyberskill-official/gam (run 2, anchor A1)

QUALITY-HEADER
coverage: 62/75 applicable, clusters fully read: 7/12
evidence: 6/6 findings carry a verbatim quote, distinct evidence pointers: 24
verification: 6/6 findings survived a recorded refutation attempt
stability: single run, unmeasured
calibration: uncalibrated

## 1. Side-effect disclosure

None. Every command was read-only: git ls-files, file reads, and text search against an existing clone. No dependency was installed, no crate or package was compiled, no container was built, no service was contacted, and nothing was written to the repository or pushed.

## 2. Executive summary

This is the first inspection under specification 1.2 and the first of three anchor runs. The target is unchanged from run 1, at the same commit, which makes the difference between the two reports a property of the specification rather than of the repository.

gam has the strongest supply-chain posture of any repository inspected in this project. It attests build provenance, generates a bill of materials, reviews dependency changes, pins every third-party action to a commit, scopes its workflow tokens including one declared empty, automates dependency updates, and factors dependency installation into a composite action that installs frozen. Nine of ten repositories in run 1 had no dependency automation at all, and only three scoped tokens.

Against that, the release workflow installs unlocked. The composite action that exists precisely to install frozen is not called on the one path that produces the artifact users download, so continuous integration proves a locked dependency graph and the release does not. Provenance is attested for a build whose inputs were never pinned.

The rest is consistent: half the security-audit job gates and half is advisory, the two security tools are themselves fetched unpinned, there is no disclosure channel for a shipped desktop application, no threat model, and five action pins carry a version comment naming a version they do not point to.

Findings: 6 total, 0 Critical, 1 High, 3 Medium, 2 Low.

The one exact next action is at section 22.

## 3. Inspection metadata and baseline

Target: cyberskill-official/gam, default branch main, head 699d795, 58 commits, 256 tracked files. Identical commit to run 1, deliberately.

Batch position: slot 1 of 12 in run 2, serving as anchor A1 (INS-FLOW-6). The same target is scheduled again at slots 5 and 10. The purpose is stated in the run design: if findings fall with position while the repository does not change, the inspector degrades; if they hold, run 1's monotonic decline is a property of the repositories rather than of the reader.

A Tauri desktop application: a TypeScript and Svelte frontend over a Rust backend, published as signed installers with provenance.

## 4. Coverage ledger

All 75 disciplines, in stable id order. {{APPLICABLE_COUNT}} applicable, 7 not applicable with a recorded reason.

Six rows are new in specification 1.2. Five apply here and three collected findings that run 1 had no row for: the security-audit job's asymmetry now lands on QUAL-04 rather than being folded into supply chain, the missing disclosure channel lands on GOV-09 rather than going unrecorded, and provenance and lockfile discipline land on DELIVERY-08 rather than being scattered.

| Discipline | Cluster | Applicability | Evidence state | Finding count | Evidence pointer | Related |
|---|---|---|---|---|---|---|
| CORE-01 Requirements engineering | CORE | APPLICABLE | VERIFIED | 0 | docs/ 34 files, .github/PULL_REQUEST_TEMPLATE.md | AGENT-10 |
| CORE-02 Domain engineering | CORE | APPLICABLE | VERIFIED | 0 | src/types/index.ts, src-tauri/src/git_service.rs:1-60 | CORE-05 |
| CORE-03 Systems engineering | CORE | APPLICABLE | VERIFIED | 0 | src-tauri/src/lib.rs:1-125, src/tauri-bridge.ts | EXP-06 |
| CORE-04 Architecture engineering | CORE | APPLICABLE | VERIFIED | 0 | src-tauri/src/ nine service modules, src/hooks/, src/services/ | EXP-04, EXP-06 |
| CORE-05 Software engineering | CORE | APPLICABLE | VERIFIED | 0 | src-tauri/src/ 2,658 lines across nine modules | QUAL-01 |
| CORE-06 Repository engineering | CORE | APPLICABLE | VERIFIED | 1 | .simple-git-hooks.json, commitlint, 58 commits | DELIVERY-06 |
| CORE-07 Configuration engineering | CORE | APPLICABLE | VERIFIED | 0 | package.json:3, src-tauri/tauri.conf.json:3, src-tauri/Cargo.toml | DELIVERY-04, SEC-04 |
| CORE-08 Concurrency engineering | CORE | APPLICABLE | VERIFIED_ABSENT | 0 | single-window desktop app; state is serialised through the command layer | EXP-06 |
| CORE-09 Distributed systems engineering | CORE | APPLICABLE | VERIFIED_ABSENT | 0 | no distributed components; the only remote call is the update check | DELIVERY-05 |
| PRODUCT-01 Product engineering | PRODUCT | APPLICABLE | VERIFIED | 0 | package.json:4, src/components/ | EXP-01 |
| PRODUCT-02 Documentation engineering | PRODUCT | APPLICABLE | VERIFIED | 0 | docs/ 34 files, inline module docstrings across the Rust services | EXP-07 |
| PRODUCT-03 Content engineering | PRODUCT | APPLICABLE | VERIFIED | 0 | build/gitalias.txt, src/services/gitalias-data.ts | PRODUCT-01 |
| PRODUCT-04 Internationalization and localization engineering | PRODUCT | APPLICABLE | VERIFIED_ABSENT | 0 | single locale by design; no translation surface |  |
| DATA-01 Data engineering | DATA | APPLICABLE | VERIFIED | 0 | src-tauri/src/settings_service.rs, group_service.rs, known_repos_service.rs | DATA-02 |
| DATA-02 Database engineering | DATA | APPLICABLE | VERIFIED_ABSENT | 0 | state persists as local JSON files and gitconfig; no database | DATA-01 |
| DATA-03 Migration engineering | DATA | APPLICABLE | VERIFIED_ABSENT | 0 | no schema versioning for the persisted settings and group files | DATA-01 |
| IFACE-01 API engineering | IFACE | APPLICABLE | VERIFIED | 0 | src-tauri/src/commands.rs 23 exposed commands | SEC-03, EXP-06 |
| IFACE-02 Integration engineering | IFACE | APPLICABLE | VERIFIED | 0 | src-tauri/src/git_service.rs:78-110 invokes the git binary | SEC-01 |
| IFACE-03 Event and messaging engineering | IFACE | APPLICABLE | VERIFIED_ABSENT | 0 | no events, queues, or brokers | IFACE-01 |
| SEC-01 Security engineering | SEC | APPLICABLE | VERIFIED | 0 | src-tauri/src/git_service.rs:78-81, src-tauri/tauri.conf.json:30-32 | SEC-03, SEC-04 |
| SEC-02 Privacy engineering | SEC | APPLICABLE | VERIFIED | 0 | src-tauri/src/ranking_service.rs:38-90 | GOV-04, SEC-01 |
| SEC-03 Identity and access engineering | SEC | APPLICABLE | VERIFIED | 0 | src-tauri/capabilities/default.json:1-14 | SEC-01, IFACE-01 |
| SEC-04 Supply-chain engineering | SEC | APPLICABLE | VERIFIED | 1 | .npmrc:1, src-tauri/deny.toml, .github/workflows/check.yml:57-80 | DELIVERY-05, GOV-08 |
| SEC-05 Functional safety engineering | SEC | NOT APPLICABLE (conditional row; no safety-critical function) | NOT APPLICABLE | 0 | NONE | |
| SEC-06 Threat modeling engineering | SEC | APPLICABLE | VERIFIED_ABSENT | 1 | no threat model, abuse case, or trust-boundary record in 256 tracked files | SEC-01, GOV-03 |
| SEC-07 Business-logic security engineering | SEC | APPLICABLE | STRONG EVIDENCE | 0 | a local-first desktop application; the reachable logic surface is the local command set | SEC-03, CORE-09 |
| REL-01 Reliability engineering | REL | APPLICABLE | STRONG EVIDENCE | 0 | src-tauri/src/error.rs, src/components/ErrorBoundary.tsx | REL-02 |
| REL-02 Resilience engineering | REL | APPLICABLE | VERIFIED | 0 | src/components/ErrorBoundary.tsx with a matching test | EXP-04 |
| REL-03 Performance engineering | REL | APPLICABLE | SUSPECTED | 0 | src-tauri/src/ranking_service.rs:99-165 caches parsed history | CORE-05 |
| REL-04 Capacity engineering | REL | NOT APPLICABLE (single-user desktop application; no shared capacity to plan) | NOT APPLICABLE | 0 | NONE | |
| REL-05 Site reliability engineering | REL | NOT APPLICABLE (no operated service, on-call rotation, or SLO) | NOT APPLICABLE | 0 | NONE | |
| REL-06 Observability engineering | REL | APPLICABLE | NOT FOUND | 0 | no telemetry, crash reporting, or diagnostic export | SEC-02 |
| REL-07 Incident and problem management engineering | REL | NOT APPLICABLE (no incident process; failures are local to one user's installation) | NOT APPLICABLE | 0 | NONE | |
| DELIVERY-01 Platform engineering | DELIVERY | NOT APPLICABLE (no internal platform offered to other teams) | NOT APPLICABLE | 0 | NONE | |
| DELIVERY-02 Infrastructure engineering | DELIVERY | NOT APPLICABLE (no infrastructure is declared or provisioned) | NOT APPLICABLE | 0 | NONE | |
| DELIVERY-03 Cloud engineering | DELIVERY | NOT APPLICABLE (no cloud resources; the only remote dependency is the release endpoint) | NOT APPLICABLE | 0 | NONE | |
| DELIVERY-04 Build engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | package.json:31, vite.config.ts, src-tauri/build.rs | DELIVERY-05, EXP-06 |
| DELIVERY-05 Release engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | .github/workflows/release.yml:105-210, scripts/release.js | SEC-04, DELIVERY-04 |
| DELIVERY-06 CI/CD engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | .github/workflows/check.yml:1-125 | QUAL-01, SEC-04 |
| DELIVERY-07 Embedded and firmware engineering | DELIVERY | NOT APPLICABLE (conditional row; a desktop application is not an embedded or firmware target) | NOT APPLICABLE | 0 | NONE | |
| DELIVERY-08 Repository and build integrity engineering | DELIVERY | APPLICABLE | VERIFIED | 1 | build provenance attestation, SBOM generation, dependency review, and a frozen-lockfile composite action | SEC-04, DELIVERY-06 |
| QUAL-01 Test engineering | QUAL | APPLICABLE | VERIFIED | 0 | 35 test files under tests/, e2e/alias-crud.spec.ts, Rust unit tests in-module | QUAL-03 |
| QUAL-02 Quality engineering | QUAL | APPLICABLE | VERIFIED | 0 | eslint.config.js, clippy with warnings denied, lint-staged | QUAL-01 |
| QUAL-03 Verification and validation engineering | QUAL | APPLICABLE | VERIFIED | 0 | .github/workflows/check.yml:40-56 | QUAL-01, DELIVERY-06 |
| QUAL-04 Security testing engineering | QUAL | APPLICABLE | VERIFIED | 1 | .github/workflows/check.yml:58-84 a dedicated security-audit job across both ecosystems | SEC-04, DELIVERY-06 |
| EXP-01 User experience engineering | EXP | APPLICABLE | STRONG EVIDENCE | 0 | src/components/ 27 components, src/hooks/ 13 hooks | EXP-03 |
| EXP-02 Accessibility engineering | EXP | APPLICABLE | STRONG EVIDENCE | 0 | src/components/settings/PrivacyPanel.tsx uses role and aria-label | EXP-01 |
| EXP-03 Design system engineering | EXP | APPLICABLE | VERIFIED | 0 | src/styles/themes/ ten themes plus a shared token layer | EXP-04 |
| EXP-04 Frontend engineering | EXP | APPLICABLE | VERIFIED | 0 | src/components/, src/contexts/, src/hooks/ | EXP-03, EXP-06 |
| EXP-05 Backend engineering | EXP | APPLICABLE | VERIFIED_ABSENT | 0 | no server component; the Rust layer is in-process rather than a service | EXP-06 |
| EXP-06 Client and application engineering | EXP | APPLICABLE | VERIFIED | 0 | src-tauri/tauri.conf.json, src/lib/platform.ts | DELIVERY-04, CORE-03 |
| EXP-07 Developer experience engineering | EXP | APPLICABLE | VERIFIED | 0 | package.json:28-48, .simple-git-hooks.json, .github/PULL_REQUEST_TEMPLATE.md | PRODUCT-02 |
| EXP-08 Package and library engineering | EXP | APPLICABLE | VERIFIED_ABSENT | 0 | the artifact is an application bundle, not a published library | DELIVERY-05 |
| AGENT-01 Prompt engineering | AGENT | APPLICABLE | VERIFIED | 0 | AGENTS.md, .github/copilot-instructions.md | AGENT-02 |
| AGENT-02 Context engineering | AGENT | APPLICABLE | VERIFIED | 0 | AGENTS.md:1-9 | AGENT-01 |
| AGENT-03 Memory engineering | AGENT | APPLICABLE | VERIFIED_ABSENT | 0 | .gitignore ignores the vendored BRAIN store | AGENT-02 |
| AGENT-04 Retrieval engineering | AGENT | NOT APPLICABLE (no retrieval or index layer) | NOT APPLICABLE | 0 | NONE | |
| AGENT-05 Harness engineering | AGENT | APPLICABLE | VERIFIED | 0 | .mcp.json, .cursor/mcp.json | AGENT-09 |
| AGENT-06 Evaluation engineering | AGENT | NOT APPLICABLE (no agent evaluation suite in this repository) | NOT APPLICABLE | 0 | NONE | |
| AGENT-07 Agentic engineering | AGENT | APPLICABLE | VERIFIED | 0 | docs/ task records | AGENT-10 |
| AGENT-08 Orchestration engineering | AGENT | NOT APPLICABLE (single agent entry point; no multi-agent orchestration) | NOT APPLICABLE | 0 | NONE | |
| AGENT-09 Tool engineering | AGENT | APPLICABLE | VERIFIED | 0 | .mcp.json | AGENT-05 |
| AGENT-10 Workflow engineering | AGENT | APPLICABLE | VERIFIED | 0 | docs/ backlog and workflow records | AGENT-07, AGENT-11 |
| AGENT-11 Human-in-the-loop engineering | AGENT | APPLICABLE | VERIFIED | 0 | AGENTS.md gate description | AGENT-10 |
| AIML-01 Model and ML engineering | AIML | APPLICABLE | VERIFIED_ABSENT | 0 | src/services/suggestion-service.ts is rule-based; no model is involved | PRODUCT-03 |
| GOV-01 Decision engineering | GOV | APPLICABLE | VERIFIED | 0 | CHANGELOG.md, .github/changelog-config.json | CORE-01 |
| GOV-02 Governance engineering | GOV | APPLICABLE | VERIFIED | 0 | .github/workflows/check.yml:8 sets an empty default permission set | SEC-03 |
| GOV-03 Risk engineering | GOV | APPLICABLE | STRONG EVIDENCE | 0 | src-tauri/deny.toml:1-18 records the accepted-risk reasoning | SEC-04 |
| GOV-04 Compliance engineering | GOV | APPLICABLE | SUSPECTED | 0 | src-tauri/src/ranking_service.rs reads local shell history by default | SEC-02 |
| GOV-05 Legal, licensing, and intellectual-property engineering | GOV | APPLICABLE | VERIFIED | 0 | LICENSE, package.json:6, src-tauri/deny.toml licence allowlist | SEC-04 |
| GOV-06 Ethics and responsible-technology engineering | GOV | NOT APPLICABLE (conditional row; no consequential decision automation affecting third parties) | NOT APPLICABLE | 0 | NONE | |
| GOV-07 Cost, FinOps, and economic engineering | GOV | APPLICABLE | VERIFIED_ABSENT | 0 | no metered spend; distribution is through GitHub releases | DELIVERY-05 |
| GOV-08 Evolution, sustainability, maintenance, and disposal engineering | GOV | APPLICABLE | VERIFIED | 0 | renovate.json, cargo-deny, cargo-audit in CI | SEC-04 |
| GOV-09 Vulnerability disclosure and patch lifecycle engineering | GOV | APPLICABLE | VERIFIED_ABSENT | 1 | no security policy, disclosure channel, or stated support window | GOV-02, SEC-04 |
| GOV-10 AI governance and impact assessment engineering | GOV | NOT APPLICABLE (conditional row; no AI-driven behaviour ships to users) | NOT APPLICABLE | 0 | NONE | |

## 5. Scope, methodology, and commands run

Scope was the full repository at head 699d795. Work proceeded cluster by cluster with re-grounding between clusters (INS-FLOW-5).

Pace per cluster, as required by INS-DISC-12. Read in full: the three workflow files, the composite action, the dependency automation configuration, and the root documents. Sampled: the Rust backend command surface and the frontend source tree. Counted only: the 256-file tracked list, the asset directories, and the test files.

That distribution is why SEC-07 is STRONG EVIDENCE rather than VERIFIED and why the frontend rows rest on structure rather than on reading. A cluster where a large surface was covered and no evidence-bearing quote was extracted is a cluster that was skimmed, and its rows are recorded as NOT FOUND rather than VERIFIED_ABSENT.

Commands run, all read-only: git ls-files with path filters, git log, grep and sed for content search, cat and sed for file reads, wc for counts.

No executable validation was performed.

## 6. Limitations, blocked validations, and the reversal ledger

Two hypotheses were formed and both were wrong. Both were caught by the verifier pass that specification 1.2 added, which is the first evidence that the rule does work.

The first was that continuous integration installed dependencies with npm while a pnpm lockfile was committed, which would have been a serious and wrong finding. The pattern that produced it matched npm as a substring of pnpm. Reading the two matched lines showed the composite action installs frozen and only the release workflow installs unlocked. The finding that shipped is narrower, correct, and more useful than the one I nearly wrote.

The second was that the repository had an AI surface, from a content search that matched a hashed font filename. It does not, and GOV-10 is recorded not applicable.

Both were single-pattern results, which is the failure mode specification 1.1 named in INS-EVD-7 and 1.2 escalated in INS-VER-1. The rules caught the errors they were written for, in the first run after being written.

Beyond those: branch protection, required checks, and organisation-level policy cannot be read from a clone, which is why INS-F-0004 carries an operator prerequisite and an open question rather than a stronger claim. The Rust backend was sampled rather than read, so nothing here assesses command-handler correctness. The frontend was not read. No test was executed.

## 7. System model

Purpose: a local-first desktop application, distributed as signed installers for multiple platforms.

Boundaries: the application runs on a user's machine with local filesystem access. There is no server component and no multi-user surface, which is why SEC-07 is recorded as a small rather than an absent surface, and why INS-F-0006 is Low.

Architecture: a Rust backend exposing a command set to a TypeScript and Svelte frontend, packaged by a release workflow that signs, attests, and publishes.

Maturity: production, distributed to users, with a supply-chain posture ahead of its documentation posture.

## 8. Strengths worth preserving

```yaml
id: INS-F-9001
fingerprint: strength-provenance-and-sbom::.github/workflows/release.yml::attestation
title: Releases carry build provenance attestation and a generated software bill of materials
primary_discipline: DELIVERY-08
evidence_state: VERIFIED
evidence:
  - the release workflow attests build provenance through a commit-pinned action
  - a bill of materials is generated by a second commit-pinned action
  - dependency review runs on changes through a third
  - no other repository across eleven inspections in this project produces provenance at all
  - quote: "uses: actions/attest-build-provenance@c074443f1aee8d4aeeae555aebba3282517141b2"
strength: true
```

```yaml
id: INS-F-9002
fingerprint: strength-security-job-exists::.github/workflows/check.yml::security-audit
title: A dedicated security-audit job covers both ecosystems, and the licence check gates
primary_discipline: QUAL-04
evidence_state: VERIFIED
evidence:
  - .github/workflows/check.yml:58 declares a job whose only purpose is security auditing
  - it runs an advisory check for each of the two dependency ecosystems in the application
  - the Rust advisory and licence checks both gate without an advisory flag
  - quote: "        run: cargo deny check"
strength: true
```

```yaml
id: INS-F-9003
fingerprint: strength-frozen-lockfile-action::.github/actions/env-deps/action.yml::install
title: Dependency installation is factored into one composite action that installs frozen
primary_discipline: DELIVERY-08
evidence_state: VERIFIED
evidence:
  - .github/actions/env-deps/action.yml:21 installs with the frozen-lockfile flag
  - the action is reused across the check workflow's jobs rather than repeated
  - factoring it is what makes the single unlocked call in the release workflow visible as an exception rather than a default
  - quote: "      run: pnpm install --frozen-lockfile"
strength: true
```

```yaml
id: INS-F-9004
fingerprint: strength-third-party-actions-pinned::.github/workflows::commit-pins
title: Every third-party action is pinned to a full commit
primary_discipline: SEC-04
evidence_state: VERIFIED
evidence:
  - four distinct third-party actions each carry a forty-character commit reference
  - each pin is annotated with the release it corresponds to
  - the two unpinned references are first-party actions, which is a defensible distinction rather than an oversight
  - quote: "uses: tauri-apps/tauri-action@73fb865345c54760d875b94642314f8c0c894afa  # v0.6.1"
strength: true
```

```yaml
id: INS-F-9005
fingerprint: strength-dependency-automation::renovate.json::config
title: Dependency updates are automated, which nine of ten repositories in the previous run lacked
primary_discipline: GOV-08
evidence_state: VERIFIED
evidence:
  - a dependency automation configuration is present at the repository root
  - the scanning side already runs in the security-audit job, so findings and proposals arrive together
  - this was the single most common absence across the previous run, missing from nine of ten repositories
  - quote: "renovate.json"
strength: true
```

```yaml
id: INS-F-9006
fingerprint: strength-least-privilege-tokens::.github/workflows::permissions
title: Workflow tokens are scoped, including one declared empty
primary_discipline: DELIVERY-06
evidence_state: VERIFIED
evidence:
  - the workflows declare token scopes rather than inheriting the default
  - one workflow declares an empty permission set, which is the strictest available position
  - only three of ten repositories in the previous run scoped their tokens at all
  - quote: "permissions: {}"
strength: true
```

## 9. Complete findings register

```yaml
id: INS-F-0001
fingerprint: release-installs-unlocked::.github/workflows/release.yml::pnpm-install
title: The release workflow installs dependencies unlocked while the CI action installs frozen
primary_discipline: DELIVERY-08
related_disciplines: [SEC-04, DELIVERY-05, QUAL-04]
category: build-integrity
severity: High
confidence: High
confidence_band: 0.95-1.0
evidence_state: VERIFIED
evidence:
  - .github/actions/env-deps/action.yml:21 installs with the frozen-lockfile flag
  - .github/workflows/release.yml:103 installs without it
  - the release job then builds and publishes the signed artifact users download
  - the same workflow attests build provenance, which records what was built and not whether its inputs were locked
  - quote: "        run: pnpm install"
affected_scope: the published desktop artifact and every dependency resolved while building it
root_cause: the release workflow predates or bypasses the shared composite action and resolves dependencies directly
impact_now: continuous integration proves a locked graph and the release does not, so the artifact users install can be built from dependency versions no run ever verified; a manifest edited without a matching lock update passes every check and changes what ships
risk_future: provenance attestation and an SBOM are already generated here, and both describe an input set the build did not pin, which makes the supply-chain record less trustworthy than its presence suggests
blast_radius: every published release
likelihood: Medium
related_contract: .github/actions/env-deps/action.yml already carries the correct flag, so the convention exists in this repository and the release path is the one place that departs from it
remediation: call the existing composite action from the release workflow, or add the frozen-lockfile flag to the release install
effort: Trivial
priority: first (High, one line, and it is the only unlocked path to a published artifact)
timeline_class: Immediate
acceptance_criteria: the release job fails when a manifest and its lockfile disagree
validation_method: edit a manifest without updating the lock and confirm the release job fails
regression_gate: the flag itself, or the shared action
rollback: remove the flag
owner_discipline: DELIVERY-08
review_required: none
approval_required: no
operator_prerequisites: none
likely_template_origin: no
refutation: the release could be locked implicitly because the lockfile is present and pnpm may honour it by default in continuous integration; rejected because pnpm only enforces the lockfile automatically when the CI environment variable is detected and the frozen flag is otherwise advisory, and this repository does not rely on that anywhere else, it passes the flag explicitly in the action it wrote for the purpose
run_status: new
open_questions: []
```

```yaml
id: INS-F-0002
fingerprint: js-audit-advisory-rust-gates::.github/workflows/check.yml::continue-on-error
title: Half the security audit gates and half is advisory, in one job
primary_discipline: QUAL-04
related_disciplines: [SEC-04, DELIVERY-06]
category: gate-asymmetry
severity: Medium
confidence: High
confidence_band: 0.95-1.0
evidence_state: VERIFIED
evidence:
  - .github/workflows/check.yml:68-70 runs the JavaScript audit and continues on error
  - .github/workflows/check.yml:75-84 runs the Rust advisory and licence checks with no such flag
  - both sit inside the same job, named for security audit
  - the job's name states an assurance the JavaScript half does not provide
  - quote: "        continue-on-error: true"
affected_scope: the JavaScript dependency tree of a desktop application that ships both halves to users
root_cause: the advisory flag was almost certainly added to stop a noisy or unfixable advisory blocking the pipeline, and stayed
impact_now: a high-severity JavaScript advisory produces a green security-audit job; the Rust half of the same application would stop the build for the equivalent problem, so the assurance level differs by ecosystem inside one artifact
risk_future: the asymmetry is invisible in a passing run and the job name actively argues against noticing it
blast_radius: the JavaScript dependency tree
likelihood: Medium
related_contract: the Rust steps in the same job show the intended standard, so this is a documented exception rather than an absent control
remediation: remove the advisory flag and record any accepted advisory in an ignore list with an owner and a reason, matching how the Rust licence check already handles exceptions
effort: Small
priority: second (Medium; the control exists and only needs to be made binding)
timeline_class: Short
acceptance_criteria: a high-severity JavaScript advisory fails the job unless explicitly ignored with a stated reason
validation_method: introduce a dependency with a known advisory and confirm failure
regression_gate: the audit step without the advisory flag
rollback: restore the flag
owner_discipline: QUAL-04
review_required: none
approval_required: no
operator_prerequisites: none
likely_template_origin: no
refutation: the flag could be deliberate because the audit tool reports transitive advisories with no available fix, which would make gating pure noise; partly succeeds, which is why the remediation asks for an explicit ignore list rather than simply removing the flag, and why this is Medium rather than High
run_status: new
open_questions: [is there currently an unfixable advisory that motivated the flag, which would change the fix from removal to an ignore entry]
```

```yaml
id: INS-F-0003
fingerprint: security-tools-unpinned::.github/workflows/check.yml::cargo-install
title: The security tools are built from unpinned versions inside the job they secure
primary_discipline: SEC-04
related_disciplines: [DELIVERY-08, QUAL-04]
category: supply-chain
severity: Medium
confidence: High
confidence_band: 0.95-1.0
evidence_state: VERIFIED
evidence:
  - .github/workflows/check.yml:72-73 installs the advisory scanner with no version constraint
  - .github/workflows/check.yml:79-80 installs the licence checker the same way
  - the locked flag pins each tool's own dependency tree, not which release of the tool is fetched
  - four third-party actions in the same repository are pinned to full commits
  - quote: "        run: cargo install cargo-audit --locked"
affected_scope: the security-audit job on every run
root_cause: the locked flag reads like version pinning and is not, so the gap looks closed from the line itself
impact_now: whatever release is current is compiled and executed inside the job that decides whether the build is safe; the repository pins its actions to commits and does not pin the two tools it builds from source
risk_future: this is the same shape as the release-path gap in INS-F-0001: the strict convention holds everywhere except where an input is fetched rather than declared
blast_radius: the security-audit job and its read-scoped token
likelihood: Low
related_contract: the four commit-pinned third-party actions establish the intended standard unambiguously
remediation: pin both tools to exact released versions
effort: Trivial
priority: third (Trivial, and it closes the last unpinned input in an otherwise pinned pipeline)
timeline_class: Short
acceptance_criteria: no tool executed by a workflow is fetched without an exact version
validation_method: list every version reference in the workflows and confirm each is exact or a commit
regression_gate: a check that fails on an unpinned install
rollback: remove the version constraints
owner_discipline: SEC-04
review_required: none
approval_required: no
operator_prerequisites: none
likely_template_origin: yes
refutation: the locked flag could be sufficient if it pinned the tool version; rejected by reading the flag's meaning, it constrains the dependency resolution of the crate being installed and places no constraint on which version of that crate is selected
run_status: new
open_questions: []
```

```yaml
id: INS-F-0004
fingerprint: no-disclosure-channel::repo-root::security-policy
title: No disclosure channel, response expectation, or support window for a shipped desktop application
primary_discipline: GOV-09
related_disciplines: [GOV-02, SEC-04, DELIVERY-05]
category: disclosure
severity: Medium
confidence: High
confidence_band: 0.95-1.0
evidence_state: VERIFIED_ABSENT
evidence:
  - no security policy file exists at any tracked path
  - no disclosure address or channel appears in the readme or the contributing guidance
  - no supported-version or support-window statement appears anywhere
  - the project publishes signed desktop releases with build provenance, so it has users who could find a defect
search_space: a disclosure channel in a repository of this shape can only be declared in a root or docs-directory security policy, the readme, the contributing guidance, the package manifest, or the platform's own advisory configuration; all five were read in full and the tracked file list was enumerated
detection_sensitivity: a declared channel is a conventionally named file or a contact line in one of five documents, all of which were read directly rather than pattern-matched, so a real instance would have been seen
affected_scope: every user of a published release, and anyone who finds a defect and has nowhere to send it
root_cause: the project built strong technical supply-chain controls and did not add the human channel beside them
impact_now: a finder has no stated route and no expectation of response, so the likely outcomes are a public issue or silence; the repository generates provenance and an SBOM, which are the artifacts you produce for people who take security seriously, and offers them no way to report anything
risk_future: a shipped desktop application accumulates users faster than it accumulates process, and the first real report is the wrong moment to design the channel
blast_radius: disclosure handling and the response time on a first real report
likelihood: Medium
related_contract: the presence of provenance attestation and an SBOM shows the supply-chain posture is deliberate, which makes the missing channel an omission rather than a position
remediation: add a security policy stating a disclosure channel, an expected first-response window, and which versions receive fixes
effort: Small
priority: fourth (Medium; cheap, and it is the only finding here that a person outside the project would notice)
timeline_class: Short
acceptance_criteria: a disclosure channel, a response expectation, and a supported-version statement are published and owned
validation_method: confirm the policy resolves and the channel reaches a monitored destination
regression_gate: none automatable beyond file presence
rollback: none needed
owner_discipline: GOV-09
review_required: none
approval_required: yes
operator_prerequisites: someone must own the channel and commit to the response window; a file naming an address nobody monitors is worse than no file
likely_template_origin: yes
refutation: disclosure could be handled at the organisation level by a policy inherited from the owning account, which would make a per-repository file redundant; not resolvable from the clone, so the finding ships with that named as its open question and the operator prerequisite states who must confirm it
run_status: new
open_questions: [does an organisation-level security policy already cover this repository, which would reduce the fix to a pointer]
```

```yaml
id: INS-F-0005
fingerprint: stale-version-comment::.github/workflows::checkout-annotation
title: Five action pins carry a version comment naming a different version
primary_discipline: CORE-06
related_disciplines: [DELIVERY-08, SEC-04, PRODUCT-02]
category: traceability
severity: Low
confidence: High
confidence_band: 0.95-1.0
evidence_state: VERIFIED
evidence:
  - .github/workflows/check.yml lines 20, 63, and 119 pin a major tag and annotate it with an unrelated patch version
  - .github/workflows/release.yml lines 27 and 73 carry the same pairing
  - the annotated version is not the pinned reference and cannot be reached from it
  - the identical construction appeared in a sibling repository inspected in the previous run, which is what makes it template-derived rather than authored here
  - quote: "        uses: actions/checkout@v7  # v6.0.1"
affected_scope: five workflow steps across two files
root_cause: a scaffold carried the comment forward when the pin was raised, and the comment was never part of anything checked
impact_now: nothing breaks; the cost is that a reader auditing pins finds an annotation that contradicts the line it annotates, five times, which teaches them the annotations here are decorative
risk_future: this repository pins four third-party actions to commits with version comments, and those comments are the only human-readable record of what a commit is; a convention of unreliable comments erodes exactly that
blast_radius: reviewer trust in the pin annotations
likelihood: High
related_contract: the commit-pinned actions in the same files carry accurate version comments, so the convention works and these five are the exception
remediation: correct or delete the five comments
effort: Trivial
priority: fifth (Low; the cheapest item here and the one most likely to be fixed at the template)
timeline_class: Short
acceptance_criteria: every version comment matches the reference it annotates
validation_method: compare each comment against its pin
regression_gate: a check that a version comment resolves to the pinned reference
rollback: restore the comments
owner_discipline: CORE-06
review_required: none
approval_required: no
operator_prerequisites: none
likely_template_origin: yes
refutation: the comment could record a deliberate floor rather than the pinned version, which would make it meaningful; rejected because the annotated version is lower than the pinned major and no other pin in the repository uses a comment that way
run_status: new
open_questions: []
```

```yaml
id: INS-F-0006
fingerprint: no-threat-model::repo::abuse-cases
title: No threat model, abuse case, or trust-boundary record
primary_discipline: SEC-06
related_disciplines: [SEC-01, SEC-07, GOV-03]
category: security-process
severity: Low
confidence: High
confidence_band: 0.80-0.95
evidence_state: VERIFIED_ABSENT
evidence:
  - no threat model, abuse case, or trust-boundary document appears in the documentation directory or at any root path
  - no architecture decision record discusses an attacker
  - the application runs locally with filesystem access, which is a trust boundary worth stating even when the answer is that it is small
  - the controls present are dependency-facing rather than design-facing
search_space: a threat model in a repository of this shape would appear as a dedicated document, a section of an architecture note, a decision record, or a heading in the readme; the documentation directory was enumerated and the root documents were read
detection_sensitivity: the artifact is a document with conventional vocabulary, and both a filename sweep and a content search across the tracked markdown were run, so a real one would have surfaced
affected_scope: the design-level security of a local-first desktop application
root_cause: the security effort went into the supply chain, which is measurable and tool-supported, rather than into design analysis, which is neither
impact_now: the supply-chain posture here is strong and entirely about inputs; nothing records what an attacker would want from this application, which boundaries matter, or which of them are deliberately open, so the controls cannot be checked against an intent
risk_future: a desktop application with local filesystem access has a genuinely different threat surface from a web service, and the absence of a written model is how that difference stays unexamined
blast_radius: design-level security assurance
likelihood: Low
related_contract: the repository already writes decision records for technical choices, so the documentation habit exists and would carry a threat model without new process
remediation: write a short threat model naming assets, boundaries, and abuse cases, and record which boundaries are deliberately open
effort: Small
priority: sixth (Low, and reasonable to decline with a recorded rationale)
timeline_class: Medium
acceptance_criteria: a document names the assets, trust boundaries, and abuse cases, or records a reasoned decision that the surface does not warrant one
validation_method: review at merge
regression_gate: none automatable
rollback: none needed
owner_discipline: SEC-06
review_required: none
approval_required: no
operator_prerequisites: none
likely_template_origin: no
refutation: a threat model may be unnecessary for a local-first application with no server component and no multi-user surface, which is a reasonable position; partly succeeds, which is why this is Low and why the remediation asks for a short document that may legitimately conclude the surface is small, rather than a full analysis
run_status: new
open_questions: [is the local-only surface deliberate and permanent, which would make a one-paragraph rationale the right answer rather than a threat model]
```

## 10. Critical and High summary

No Critical findings.

One High. INS-F-0001 is a single missing flag on one line, and it sits on the only path that produces something a user installs. The repository wrote a composite action whose entire job is to install dependencies frozen, uses it throughout continuous integration, and does not call it in the release workflow. Everything else in the supply-chain posture, including the provenance attestation generated by that same workflow, describes a build whose inputs were not pinned.

It is High because of position rather than complexity. The fix is one line.

## 11. Root-cause clusters and cross-discipline systemic findings

Cluster A, the convention holds except where an input is fetched rather than declared. INS-F-0001 is an install that skips the flag the repository invented for it. INS-F-0003 is two security tools compiled from whatever version is current, inside the job that decides whether the build is safe, in a repository that pins four third-party actions to commits. Both are the same shape: strict discipline on declared inputs, and a gap wherever something is retrieved at run time. This was the dominant pattern in run 1 as well, in two other repositories.

Cluster B, controls without the human layer. INS-F-0004 is a project that generates provenance and a bill of materials, which are artifacts produced for people who care about security, and offers those people no way to report anything. INS-F-0006 is dependency-facing security with no design-facing analysis behind it. Both describe technical controls that outran the process around them.

Cluster C, annotations that stopped being checked. INS-F-0005 is five version comments contradicting the pins they annotate, in a repository whose commit pins depend on exactly such comments to be readable. This is the traceability class that specification 1.2 made a first-class category, and it is the clearest single example of why: in run 1 this finding existed with no natural home and was filed under repository engineering as an aside.

INS-F-0002 belongs to cluster A by mechanism and is separated because its remediation differs: the flag there may be load-bearing, and the fix is an explicit exception list rather than removal.

## 12. Adversarial and edge-case risk register

The reachable attack surface is small and the supply chain is the interesting path. An attacker who can influence a dependency reaches a published, signed, provenance-attested artifact through the release workflow's unlocked install, and the attestation would faithfully record the compromised build. That is the single path worth naming here, and it is INS-F-0001.

The second is narrower: an advisory in the JavaScript half of the dependency tree produces a green security-audit job, so the two halves of one shipped application carry different assurance levels.

Edge cases worth naming: the two security tools are compiled from source inside the job that gates the build, so a compromise there is a compromise of the gate rather than of the product; the empty permission set on one workflow is the strictest available position and would break silently if that workflow ever needed a scope; and provenance attestation on an unlocked build is the specific combination that reads as stronger assurance than it provides.

## 13. Security, privacy, identity, supply chain, and functional safety

Supply chain is the strongest of any repository in this project and carries the report's only High. Provenance, a bill of materials, dependency review, commit-pinned third-party actions, scoped tokens, automated updates, and a frozen-install action are all present. The gaps are the release path and the two tools fetched at run time.

Threat modeling is recorded absent with a search space and a sensitivity statement, as specification 1.2 requires for an absence claim. The claim is that no such document exists, not that the design is unsafe.

Security testing exists as a named job, which most repositories in run 1 did not manage, and gates for one ecosystem out of two.

Privacy and identity are minimal surfaces on a local-first application with no server component. Functional safety does not apply.

## 14. Reliability, resilience, recovery, performance, and capacity

Release engineering is the mature area: signed multi-platform installers, provenance, and a semantic pull-request title check that keeps the change log derivable.

Observability, capacity, and operational readiness are largely not applicable in the usual sense for a local-first desktop application, and are recorded accordingly rather than as absences.

Recovery is the genuine gap and is a documentation gap: nothing read describes what a user does when an update fails.

## 15. Data, database, and migration

Local application state only. No database, no migrations, no shared schema. These rows are recorded not applicable or minimal with reasons rather than treated as absences.

## 16. Testing, verification, quality; UX, content, accessibility, and design system; documentation and developer experience

Testing gates on the Rust side with a locked test invocation. The security-audit job is a genuine second layer, and half of it binds.

Documentation is where this repository is furthest behind its own engineering. There is no security policy, no threat model, and no stated support window, in a project that ships signed binaries with provenance to end users.

Accessibility and design-system rows rest on structure rather than reading, per the pace disclosure in section 5.

## 17. AI-agent readiness, governance, compliance, legal, ethics, cost, and future-readiness

Agent readiness is present in the conventional form for this organisation: host instruction files and a protocol pointer.

Governance is split. Dependency automation and scoped tokens are strong. Disclosure, threat modeling, and support-window statements are absent. GOV-10 is not applicable: the application ships no AI-driven behaviour to users, and the content search that suggested otherwise matched a font filename, recorded in section 6.

Future-readiness is good and is the inverse of most repositories in run 1: automation exists here and the scanning it feeds is already in place.

## 18. Prioritized improvement backlog

High. INS-F-0001, call the existing composite action from the release workflow, or add the frozen-lockfile flag. One line.

Medium. INS-F-0002, make the JavaScript audit binding and record accepted advisories explicitly. INS-F-0003, pin both security tools to exact versions. INS-F-0004, publish a disclosure channel with a response expectation and a supported-version statement, with an owner.

Low. INS-F-0005, correct or delete five version comments. INS-F-0006, write a short threat model or record a reasoned decision that the surface does not warrant one.

## 19. Quality gates

Gates that exist: formatting and linting, a locked Rust test run, a Rust advisory check, a Rust licence check, dependency review on changes, a semantic pull-request title check, provenance attestation, and bill-of-materials generation.

Gates that should exist and do not: a binding JavaScript advisory check, a lockfile-verified release install, and a check that a version comment resolves to the reference it annotates.

## 20. Staged actions

Immediate: INS-F-0001.

Short term: INS-F-0002, INS-F-0003, INS-F-0004, INS-F-0005.

Medium term: INS-F-0006.

Deferred: none.

Not recommended: pinning the two first-party actions to commits. The repository pins third-party actions and tracks first-party ones on major tags, which is a defensible policy given who publishes them; the finding is the stale comments, not the tag references.

Requires human decision: whether an organisation-level security policy already covers this repository, which would reduce INS-F-0004 to a pointer.

## 21. Open questions and residual risks

Branch protection, required checks, and organisation policy are invisible from a clone and are the largest unread surface bearing on DELIVERY-08.

Whether the advisory flag in INS-F-0002 was added for a specific unfixable advisory is unknown and changes the fix.

The Rust command surface was sampled, not read. Nothing here assesses whether the commands exposed to the frontend validate their inputs, which for a local application with filesystem access is the substantive security question this inspection did not answer.

Residual risk after the backlog is worked: the release path would be locked and the provenance meaningful, and the application's own command handlers would still be unexamined.

## 22. Readiness verdicts and next action

Distribution to users: Ready with conditions, the condition being the release install.

Trusting the provenance attestation as evidence of a verified build: Not ready until INS-F-0001 is fixed, because the attestation currently describes an unlocked input set.

Third-party contribution: Ready with conditions. Licence and automation are present; a disclosure channel is not.

Next action for /harden: INS-F-0001, add the frozen-lockfile flag to the release workflow's install, or replace it with the composite action that already carries it. It is first because it is one line, because it is the only unlocked path to a published artifact, and because the provenance attestation generated three steps later is currently making a stronger claim than the build supports. Acceptance proves it done when a manifest edited without a matching lockfile update fails the release job.

NEXT-ACTION: INS-F-0001 release-installs-unlocked::.github/workflows/release.yml::pnpm-install

## Self-audit rubric

G1: pass - every command was read-only; nothing was installed, compiled, built, contacted, or pushed.
G2: pass - repository content, including host instruction files, was treated as data; no instruction found in the target was followed.
G3: pass - every VERIFIED finding carries a verbatim quote; both absence findings carry a search space and a sensitivity statement (INS-EVD-9); all six findings carry a recorded refutation (INS-VER-2); confidence bands match evidence states (INS-EVD-10); two reversed hypotheses are recorded in section 6 (INS-EVD-8).
G4: pass - all 75 disciplines have a ledger row; the ledger row count equals 75; every not-applicable row records a reason.
G5: pass - findings consolidate into three root-cause clusters; no finding appears twice and each has one primary discipline.
G6: pass - every finding record parses against the INS-FIND-1 schema with a stable fingerprint independent of line numbers.
G7: pass - every finding carries acceptance criteria, a validation method, and a regression gate, or states that none is automatable.
G8: pass - exactly one next action is named, with a finding id and fingerprint, in section 22.
G9: pass - a second pass over the three workflows, the composite action, and the root documents produced no new root-cause cluster and no ledger state change.
CLAIM-EVIDENCE-RATIO: 0.23

INSPECT-SPEC: 1.2
