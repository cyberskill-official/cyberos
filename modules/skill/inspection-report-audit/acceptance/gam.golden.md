# /inspect report: cyberskill-official/gam

## 1. Side-effect disclosure

None. Every command was read-only: git clone, git fetch --unshallow, git log, git ls-files, file reads, and text search. No dependency was installed, no Rust crate was compiled, no binary was built or run, and nothing was written to the repository or pushed. The clone lives in a scratch directory.

## 2. Executive summary

gam is a Tauri desktop application for managing git aliases, and it has the strongest engineering posture of the five repositories in this batch. Six strengths are recorded, more than any other. The web view is granted four capability sets and no blanket filesystem or shell access, so every privileged operation goes through a named Rust command. Git is invoked with an argument vector rather than a shell string, with a test specifically named for injection rejection. The content policy is restrictive with an explicit connection allowlist. Verification runs across three operating systems with an empty default permission set, clippy warnings denied, and both cargo-audit and cargo-deny gating the Rust tree. Releases are signed, carry a bill of materials, and attest build provenance.

The findings are therefore about the edges of a well-built system rather than its centre, and two of them sit in the supply chain that everything else depends on. Eleven third-party actions across the two workflows are pinned to full commit hashes; the one that installs the compiler for the signed release binaries is not. Separately, the entire JavaScript dependency graph resolves through a third-party registry mirror, which is the one place where this repository's supply-chain posture is materially weaker than the Rust half of the same application.

The third finding worth naming is a privacy default rather than a defect: the application reads zsh, bash, fish, and PowerShell history on first launch, and the disclosure explaining that lives in a settings dropdown the user has not opened yet. The disclosure text is honest and complete, the opt-out clears the cache, and nothing is transmitted. Only the ordering is wrong.

Findings: 10 total, 0 Critical, 2 High, 5 Medium, 3 Low.

The one exact next action is at section 22.

## 3. Inspection metadata and baseline

Target: https://github.com/cyberskill-official/gam, default branch main, head 699d795, 58 commits, last commit 2026-07-20. Working tree 6.9M excluding .git, of which 69 files are fonts and images. 256 tracked files.

Languages by line count: YAML 12,855 across 2 files, dominated by the lockfile; CSS 5,256 across 20, of which ten are theme files; TSX 4,454 across 47; TS 3,873 across 44; Rust 2,661 across 11; Markdown 2,599 across 25; JS 650 across 4; YML 417 across 4.

Stack: Tauri 2 with a React 19 front end on Vite 8, TypeScript 6, Tailwind 4, Vitest 4 for unit and component tests, Playwright for end to end. Nine Rust service modules behind twenty-three exposed commands. Node pinned to 24.18.0, pnpm workspace, exact version pins on all forty JavaScript dependencies. Two workflows, two composite actions, a commit-hook configuration, a Renovate configuration, and a cargo-deny policy.

Inspection was single-pass and reached saturation without continuation; see section 5.

## 4. Coverage ledger

All 69 disciplines, in stable id order. 57 applicable, 12 not applicable with a recorded reason. This is the first repository in the batch where the native client row applies.

| Discipline | Cluster | Applicability | Evidence state | Finding count | Evidence pointer | Related |
|---|---|---|---|---|---|---|
| CORE-01 Requirements engineering | CORE | APPLICABLE | VERIFIED | 0 | docs/ 34 files, .github/PULL_REQUEST_TEMPLATE.md | AGENT-10 |
| CORE-02 Domain engineering | CORE | APPLICABLE | VERIFIED | 0 | src/types/index.ts, src-tauri/src/git_service.rs:1-60 | CORE-05 |
| CORE-03 Systems engineering | CORE | APPLICABLE | VERIFIED | 0 | src-tauri/src/lib.rs:1-125, src/tauri-bridge.ts | EXP-06 |
| CORE-04 Architecture engineering | CORE | APPLICABLE | VERIFIED | 0 | src-tauri/src/ nine service modules, src/hooks/, src/services/ | EXP-04, EXP-06 |
| CORE-05 Software engineering | CORE | APPLICABLE | VERIFIED | 0 | src-tauri/src/ 2,658 lines across nine modules | QUAL-01 |
| CORE-06 Repository engineering | CORE | APPLICABLE | VERIFIED | 0 | .simple-git-hooks.json, commitlint, 58 commits | DELIVERY-06 |
| CORE-07 Configuration engineering | CORE | APPLICABLE | VERIFIED | 1 | package.json:3, src-tauri/tauri.conf.json:3, src-tauri/Cargo.toml | DELIVERY-04, SEC-04 |
| CORE-08 Concurrency engineering | CORE | APPLICABLE | VERIFIED_ABSENT | 0 | single-window desktop app; state is serialised through the command layer | EXP-06 |
| CORE-09 Distributed systems engineering | CORE | APPLICABLE | VERIFIED_ABSENT | 0 | no distributed components; the only remote call is the update check | DELIVERY-05 |
| PRODUCT-01 Product engineering | PRODUCT | APPLICABLE | VERIFIED | 0 | package.json:4, src/components/ | EXP-01 |
| PRODUCT-02 Documentation engineering | PRODUCT | APPLICABLE | VERIFIED | 1 | docs/ 34 files, inline module docstrings across the Rust services | EXP-07 |
| PRODUCT-03 Content engineering | PRODUCT | APPLICABLE | VERIFIED | 0 | build/gitalias.txt, src/services/gitalias-data.ts | PRODUCT-01 |
| PRODUCT-04 Internationalization and localization engineering | PRODUCT | APPLICABLE | VERIFIED_ABSENT | 0 | single locale by design; no translation surface |  |
| DATA-01 Data engineering | DATA | APPLICABLE | VERIFIED | 0 | src-tauri/src/settings_service.rs, group_service.rs, known_repos_service.rs | DATA-02 |
| DATA-02 Database engineering | DATA | APPLICABLE | VERIFIED_ABSENT | 0 | state persists as local JSON files and gitconfig; no database | DATA-01 |
| DATA-03 Migration engineering | DATA | APPLICABLE | VERIFIED_ABSENT | 0 | no schema versioning for the persisted settings and group files | DATA-01 |
| IFACE-01 API engineering | IFACE | APPLICABLE | VERIFIED | 0 | src-tauri/src/commands.rs 23 exposed commands | SEC-03, EXP-06 |
| IFACE-02 Integration engineering | IFACE | APPLICABLE | VERIFIED | 0 | src-tauri/src/git_service.rs:78-110 invokes the git binary | SEC-01 |
| IFACE-03 Event and messaging engineering | IFACE | APPLICABLE | VERIFIED_ABSENT | 0 | no events, queues, or brokers | IFACE-01 |
| SEC-01 Security engineering | SEC | APPLICABLE | VERIFIED | 1 | src-tauri/src/git_service.rs:78-81, src-tauri/tauri.conf.json:30-32 | SEC-03, SEC-04 |
| SEC-02 Privacy engineering | SEC | APPLICABLE | VERIFIED | 1 | src-tauri/src/ranking_service.rs:38-90 | GOV-04, SEC-01 |
| SEC-03 Identity and access engineering | SEC | APPLICABLE | VERIFIED | 0 | src-tauri/capabilities/default.json:1-14 | SEC-01, IFACE-01 |
| SEC-04 Supply-chain engineering | SEC | APPLICABLE | VERIFIED | 3 | .npmrc:1, src-tauri/deny.toml, .github/workflows/check.yml:57-80 | DELIVERY-05, GOV-08 |
| SEC-05 Functional safety engineering | SEC | NOT APPLICABLE (conditional row; no safety-critical function) | NOT APPLICABLE | 0 | NONE | |
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
| DELIVERY-04 Build engineering | DELIVERY | APPLICABLE | VERIFIED | 1 | package.json:31, vite.config.ts, src-tauri/build.rs | DELIVERY-05, EXP-06 |
| DELIVERY-05 Release engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | .github/workflows/release.yml:105-210, scripts/release.js | SEC-04, DELIVERY-04 |
| DELIVERY-06 CI/CD engineering | DELIVERY | APPLICABLE | VERIFIED | 1 | .github/workflows/check.yml:1-125 | QUAL-01, SEC-04 |
| DELIVERY-07 Embedded and firmware engineering | DELIVERY | NOT APPLICABLE (conditional row; a desktop application is not an embedded or firmware target) | NOT APPLICABLE | 0 | NONE | |
| QUAL-01 Test engineering | QUAL | APPLICABLE | VERIFIED | 1 | 35 test files under tests/, e2e/alias-crud.spec.ts, Rust unit tests in-module | QUAL-03 |
| QUAL-02 Quality engineering | QUAL | APPLICABLE | VERIFIED | 0 | eslint.config.js, clippy with warnings denied, lint-staged | QUAL-01 |
| QUAL-03 Verification and validation engineering | QUAL | APPLICABLE | VERIFIED | 0 | .github/workflows/check.yml:40-56 | QUAL-01, DELIVERY-06 |
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

## 5. Scope, methodology, and commands run

Scope was the full repository at head 699d795 including history. Method was Phase 0 baseline, Phase 1 discovery and mapping, Phase 3 static reading of the capability manifest, the application configuration, both workflows, both composite action definitions, the dependency policy, the package manager configuration, the three version manifests, the command surface in full, and the git, ranking, and settings services in the parts that execute or read outside the application, Phase 6 cross-layer reconciliation between the declared security posture and the implemented one, and Phase 7 discipline sweep.

Commands run, all read-only: git clone and git fetch --unshallow; git ls-files; grep and sed for content search and for building the action-pinning inventory; cat, head, and sed -n for file reads; wc for module sizes; node -p to read three manifest versions without hand-parsing.

No executable validation was performed. The three checks that would add most are running the Rust test suite, running cargo-deny to see what the policy currently permits, and launching the application with a fresh settings directory to observe the first-run history read directly. None was run because each requires compiling the Rust tree or installing the JavaScript one, which is a side effect on a first pass.

## 6. Limitations and blocked validations

INS-F-0003 is verified as a code fact rather than as observed behaviour. The default is explicit at ranking_service.rs:67 and confirmed by a comment at lib.rs:69, and no consent flow exists in the source, but the actual sequence of file reads on a fresh launch was not observed.

INS-F-0002 rests on a judgement about what a registry mirror does and does not protect. The lockfile carries integrity hashes, so the claim is deliberately narrow: already-locked versions are protected, and the exposure is at add-time and in graph disclosure. Confidence is recorded as Medium for that reason.

The Rust services were read selectively. commands.rs was read in full, git_service.rs and ranking_service.rs in the parts that execute processes or read user files, and settings, group, known-repos, file, and error services only at their interfaces. file_service.rs in particular handles import and export paths and deserves a full read in a later pass.

Whether branch protection is configured on main is not knowable from the repository, and it determines whether INS-F-0004 is active or latent.

The ten theme files and 5,256 lines of CSS were not read. Nothing in the finding set depends on them.

## 7. System model

Purpose: a desktop application for creating, editing, grouping, and ranking git aliases across global and repository-local scopes, with import and export, a curated alias library, and ten visual themes.

Users: a single local developer. There is no account, no server, and no multi-user surface.

Context and boundaries: the application edits the user's git configuration, which is the highest-consequence thing it touches, and optionally reads the user's shell history for usage ranking. Its only network dependency is the update endpoint. The trust boundary that matters is the twenty-three command surface between the web view and the Rust backend, and the capability manifest is deliberately minimal so that boundary carries the weight rather than a blanket permission.

Architecture: src-tauri holds nine service modules behind a command layer; src holds a React front end organised into components, hooks, contexts, and services, with a thin bridge module wrapping the command calls. Tests mirror that structure with 18 component files, 12 hook files, and 3 service files, plus in-module Rust tests and one end-to-end spec.

Data and state: settings, groups, and known repositories persist as local files; aliases live in git configuration and are read back through the git binary. No database, no schema versioning for the persisted files, which is recorded under DATA-03 as an absence rather than a finding.

Maturity: production. This is the only repository in the batch with a signed, attested, multi-platform release pipeline and an automatic updater serving real installations.

## 8. Strengths worth preserving

```yaml
id: INS-F-9001
fingerprint: strength-minimal-capabilities::src-tauri/capabilities/default.json::permissions
title: The web view is granted four capability sets and no blanket filesystem or shell access
primary_discipline: SEC-03
evidence_state: VERIFIED
evidence:
  - src-tauri/capabilities/default.json:7-13
  - every filesystem and git operation goes through a named command in src-tauri/src/commands.rs instead
  - quote: '        "core:default",'
strength: true
```

```yaml
id: INS-F-9002
fingerprint: strength-argv-not-shell::src-tauri/src/git_service.rs::exec_git
title: Git is invoked with an argument vector rather than a shell string, with a test that rejects injection
primary_discipline: SEC-01
evidence_state: VERIFIED
evidence:
  - src-tauri/src/git_service.rs:78-81
  - src-tauri/src/git_service.rs:347-363 constrains alias names to a strict pattern
  - src-tauri/src/git_service.rs:516 is a test named for injection rejection
  - quote: "        cmd.args(args);"
strength: true
```

```yaml
id: INS-F-9003
fingerprint: strength-tight-csp::src-tauri/tauri.conf.json::security
title: The content policy is restrictive with an explicit connection allowlist and no inline script
primary_discipline: SEC-01
evidence_state: VERIFIED
evidence:
  - src-tauri/tauri.conf.json:30-32
  - quote: "            \"csp\": \"default-src 'self'; script-src 'self'; style-src 'self'; font-src 'self'; img-src 'self' data:; connect-src ipc: http://ipc.localhost https://github.com/cyberskill-official/gam/releases/ https://objects.githubusercontent.com\""
strength: true
```

```yaml
id: INS-F-9004
fingerprint: strength-three-os-gate::.github/workflows/check.yml::matrix
title: Verification runs across three operating systems with an empty default permission set
primary_discipline: DELIVERY-06
evidence_state: VERIFIED
evidence:
  - .github/workflows/check.yml:8 sets no permissions by default and raises them per job
  - .github/workflows/check.yml:14-17 runs the matrix on three operating systems
  - .github/workflows/check.yml:50-56 denies clippy warnings and runs the Rust tests against the lockfile
  - quote: "permissions: {}"
strength: true
```

```yaml
id: INS-F-9005
fingerprint: strength-signed-release-chain::.github/workflows/release.yml::provenance
title: Releases are signed, accompanied by a bill of materials, and carry build provenance attestation
primary_discipline: DELIVERY-05
evidence_state: VERIFIED
evidence:
  - .github/workflows/release.yml:105-110 signs with a key held as a secret
  - .github/workflows/release.yml:163-171 generates and uploads a bill of materials
  - .github/workflows/release.yml:202 attests build provenance
  - src-tauri/tauri.conf.json:47-52 pins the update endpoint and the verification key
  - quote: "          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}"
strength: true
```

```yaml
id: INS-F-9006
fingerprint: strength-version-agreement::manifests::version
title: The three manifests that must agree on a version do agree
primary_discipline: CORE-07
evidence_state: VERIFIED
evidence:
  - package.json, src-tauri/tauri.conf.json, and src-tauri/Cargo.toml all declare the same version
  - scripts/release.js drives the bump
  - quote: 'version = "1.0.12"'
strength: true
```

## 9. Complete findings register

```yaml
id: INS-F-0001
fingerprint: unpinned-toolchain-in-signed-release::.github/workflows/release.yml::rust-toolchain
title: The action that installs the compiler for signed release binaries is the one action not pinned to a commit
primary_discipline: SEC-04
related_disciplines: [DELIVERY-05, GOV-03, CORE-07]
category: supply-chain
severity: High
confidence: High
evidence_state: VERIFIED
evidence:
  - .github/workflows/release.yml:81-84
  - .github/workflows/check.yml:24-27
  - eleven other third-party actions across both workflows carry full commit pins with version comments
  - .github/workflows/release.yml:105-110 signs the artifacts produced by that toolchain
  - quote: "        uses: dtolnay/rust-toolchain@stable"
affected_scope: every released desktop binary on macOS, Windows, and Linux
root_cause: the repository adopted commit pinning as its convention and this action was left on a mutable branch reference
impact_now: the release job signs its output with a private key and attests build provenance, but the compiler that produced the binary comes from a reference that can change between runs, so the integrity chain has one link nobody can reproduce
risk_future: the updater ships these binaries to installed users automatically, so a compromised build reaches machines without any further human step
blast_radius: every installation that accepts an automatic update
likelihood: Low
related_contract: src-tauri/deny.toml gates the Rust dependency tree on four axes, which makes the unpinned toolchain the weakest remaining link in the same chain
remediation: pin the toolchain action to a full commit SHA with a version comment, matching the convention the other eleven actions already follow, and let the dependency bot raise it
effort: Trivial
priority: first (High, Trivial effort, and it is the only gap in an otherwise complete chain)
timeline_class: Immediate
acceptance_criteria: no third-party action reference in any workflow resolves to a branch or tag
validation_method: list every uses: reference across both workflows and both composite actions and confirm each third-party entry is a 40-character SHA
regression_gate: a CI step that fails when a workflow references a third-party action by anything other than a commit
rollback: restore the branch reference
owner_discipline: SEC-04
review_required: none
approval_required: no
run_status: new
open_questions: [should the two remaining tag-pinned first-party actions move to commits as well, or is the convention deliberately third-party only]
```

```yaml
id: INS-F-0002
fingerprint: third-party-registry-mirror::.npmrc::registry
title: The whole JavaScript dependency graph resolves through a third-party registry mirror
primary_discipline: SEC-04
related_disciplines: [DELIVERY-04, GOV-03, CORE-07]
category: supply-chain
severity: High
confidence: Medium
evidence_state: VERIFIED
evidence:
  - .npmrc:1
  - src-tauri/deny.toml gates the Rust tree on advisories, licences, bans, and sources
  - .github/workflows/check.yml:63-65 runs the JavaScript audit non-blocking
  - quote: "registry=https://registry.npmmirror.com"
affected_scope: every install of the 40 declared JavaScript dependencies, locally and in continuous integration
root_cause: a mirror was configured for download speed and became the resolution source of record for the project
impact_now: the lockfile carries integrity hashes, so tampering with an already-locked version would be caught; what is not protected is the moment a new dependency is added, because the hash recorded at that point comes from whatever the mirror served, and the dependency graph is disclosed to a third party on every install
risk_future: this is the one place where the repository's supply-chain posture is materially weaker than its Rust equivalent, which gates on four axes and pins its sources explicitly
blast_radius: the frontend bundle that ships inside every signed desktop binary
likelihood: Low
related_contract: src-tauri/deny.toml has a sources section precisely to constrain where Rust crates may come from; the JavaScript side has no equivalent
remediation: point the registry at the canonical source and keep the mirror as a fallback only, or record an explicit accepted-risk decision alongside the cargo-deny reasoning so the asymmetry is deliberate rather than incidental
effort: Trivial
priority: second (High; the fix is one line, the decision behind it deserves a written rationale)
timeline_class: Immediate
acceptance_criteria: the resolution source is either the canonical registry or a mirror with a recorded accepted-risk rationale
validation_method: install from a clean store and confirm the resolved tarball origins
regression_gate: a check that .npmrc's registry matches the recorded decision
rollback: restore the mirror
owner_discipline: SEC-04
review_required: security
approval_required: yes
run_status: new
open_questions: [was the mirror chosen for network reliability from a specific region, which would make a documented fallback the right shape rather than removal]
```

```yaml
id: INS-F-0003
fingerprint: history-reading-defaults-on::src-tauri/src/ranking_service.rs::enabled
title: Shell history is read on first launch, before the user has seen the disclosure
primary_discipline: SEC-02
related_disciplines: [GOV-04, EXP-01, SEC-01]
category: privacy-default
severity: Medium
confidence: High
evidence_state: VERIFIED
evidence:
  - src-tauri/src/ranking_service.rs:64-68
  - src-tauri/src/lib.rs:69 records the default in a comment
  - src/components/settings/PrivacyPanel.tsx:32-37 states which files are read
  - no first-run consent flow exists anywhere in src/ or src-tauri/src/
  - quote: "            enabled: true,"
affected_scope: every user on first launch, before any settings panel is opened
root_cause: the ranking feature was built with an opt-out rather than an opt-in, and the disclosure lives in a settings dropdown rather than in the first-run path
impact_now: the application reads zsh, bash, fish, and PowerShell history on this machine before the user has been told; those files routinely contain arguments a person would not choose to expose, including tokens pasted into commands
risk_future: the disclosure is honest and the opt-out clears the cache, so the implementation is sound; only the ordering is wrong, and ordering is the part that a privacy review would fail
blast_radius: local reads only, with nothing transmitted, which is why this is Medium rather than High
likelihood: High
related_contract: src/components/settings/PrivacyPanel.tsx already contains the exact disclosure text a first-run prompt would use
remediation: default the setting off and present the existing disclosure once on first launch, or present it before the first read and record the answer
effort: Small
priority: third (Medium; the copy already exists, so this is a flow change rather than new work)
timeline_class: Short
acceptance_criteria: no history file is opened before the user has answered the disclosure once
validation_method: launch with a fresh settings directory and confirm no history file is read until the prompt is answered
regression_gate: a Rust test asserting the service reads nothing while consent is unset
rollback: restore the enabled default
owner_discipline: SEC-02
review_required: none
approval_required: no
run_status: new
open_questions: [should an unanswered prompt behave as off, which is the safer default but changes ranking on first use]
```

```yaml
id: INS-F-0004
fingerprint: check-not-on-push::.github/workflows/check.yml::on
title: The check workflow runs only on pull requests, so a direct push followed by a tag reaches release ungated
primary_discipline: DELIVERY-06
related_disciplines: [QUAL-03, DELIVERY-05, GOV-02]
category: quality-gate-bypass
severity: Medium
confidence: High
evidence_state: VERIFIED
evidence:
  - .github/workflows/check.yml:3-6
  - .github/workflows/release.yml:4-6 triggers on a tag push
  - no branch protection is declared in-tree
  - quote: "  pull_request:"
affected_scope: any commit that reaches main without a pull request
root_cause: the gate was wired to the review path rather than to the branch, and the release path keys off tags rather than off a green check
impact_now: a commit pushed straight to main and tagged produces a signed release that the three-platform matrix, clippy, the test suite, and both audit jobs never saw
risk_future: the same repository already carries the strongest verification suite in this batch, so the gap is entirely in when it runs rather than in what it checks
blast_radius: the released binary and everyone who auto-updates to it
likelihood: Medium
related_contract: the release workflow signs and attests whatever it is given; it does not verify that the checks passed
remediation: add push on main to the check trigger, and make the release job depend on a successful check run for the same commit
effort: Small
priority: fourth (Medium; it protects everything the suite already tests)
timeline_class: Short
acceptance_criteria: a tag on a commit whose checks did not pass does not produce a release
validation_method: tag a commit with a deliberately failing test on a scratch branch and confirm the release job does not run
regression_gate: the dependency between the two workflows is itself the gate
rollback: remove the trigger and the dependency
owner_discipline: DELIVERY-06
review_required: none
approval_required: no
run_status: new
open_questions: [is branch protection already configured in repository settings, which would make this latent rather than active]
```

```yaml
id: INS-F-0005
fingerprint: js-audit-non-blocking::.github/workflows/check.yml::pnpm-audit
title: JavaScript advisories are reported and ignored while Rust advisories block
primary_discipline: SEC-04
related_disciplines: [DELIVERY-06, GOV-03]
category: gate-asymmetry
severity: Medium
confidence: High
evidence_state: VERIFIED
evidence:
  - .github/workflows/check.yml:63-65
  - .github/workflows/check.yml:67-80 runs the Rust audit and deny checks with no such escape
  - quote: "        continue-on-error: true"
affected_scope: the 40 JavaScript dependencies that ship inside the application bundle
root_cause: the escape hatch was added so a noisy advisory feed could not block merges, and it was left permanently rather than scoped to specific accepted advisories
impact_now: a high-severity advisory in the frontend tree produces a green build; the Rust half of the same application refuses to build under the equivalent condition
risk_future: the asymmetry is invisible in the job summary, so the security-audit job reads as passing when half of it cannot fail
blast_radius: the frontend bundle inside every release
likelihood: Medium
related_contract: src-tauri/deny.toml models the right pattern with an explicit ignore list that requires a comment per entry
remediation: remove the escape and add an explicit ignore list with a recorded reason per advisory, matching the Rust configuration
effort: Small
priority: fifth (Medium; it restores a gate that already runs)
timeline_class: Short
acceptance_criteria: a high-severity advisory not on the ignore list fails the build
validation_method: add a dependency with a known advisory and confirm the job fails
regression_gate: the audit job itself once the escape is removed
rollback: restore the escape
owner_discipline: SEC-04
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0006
fingerprint: build-script-macos-only::package.json::build
title: The build script targets macOS only while the bundle configuration and the test matrix cover three platforms
primary_discipline: DELIVERY-04
related_disciplines: [EXP-06, EXP-07, DELIVERY-05]
category: portability
severity: Medium
confidence: High
evidence_state: VERIFIED
evidence:
  - package.json:31
  - src-tauri/tauri.conf.json:39 sets the bundle targets to all
  - .github/workflows/check.yml:14-17 runs the matrix on three operating systems
  - quote: '    "build": "dotenv -- tauri build --target universal-apple-darwin",'
affected_scope: any contributor on Windows or Linux following the documented scripts
root_cause: the maintainer's own platform was encoded into the shared script rather than into a platform-specific variant
impact_now: a contributor on Linux or Windows who runs the build script gets a target-triple error; the release workflow is unaffected because it drives the Tauri action directly
risk_future: the three-platform test matrix invites contributions from those platforms that the build script then blocks
blast_radius: contributor experience, not the released artifact
likelihood: Medium
related_contract: the check workflow already proves the project builds on all three platforms
remediation: drop the explicit target so the host platform is used, and add a separate script for the macOS universal build
effort: Trivial
priority: sixth (Trivial, and it unblocks the platforms the matrix already tests)
timeline_class: Short
acceptance_criteria: the build script succeeds on all three platforms the matrix covers
validation_method: run the script on a Linux runner and confirm it completes
regression_gate: a matrix job that runs the documented build script rather than the workflow's own commands
rollback: restore the fixed target
owner_discipline: DELIVERY-04
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0007
fingerprint: unscoped-open-commands::src-tauri/src/commands.rs::open
title: Two commands launch external programs with only shape checks and no allowlist
primary_discipline: SEC-01
related_disciplines: [SEC-03, IFACE-01, EXP-06]
category: input-validation
severity: Medium
confidence: Medium
evidence_state: VERIFIED
evidence:
  - src-tauri/src/commands.rs:231-242 accepts any existing directory
  - src-tauri/src/commands.rs:244-256 accepts any address beginning with the secure scheme
  - src-tauri/capabilities/default.json:7-13 grants no blanket filesystem or shell permission, so these commands are the surface
  - quote: '    if !url.starts_with("https://") {'
affected_scope: the twenty-three command surface exposed to the web view
root_cause: both commands validate the shape of their argument rather than membership in a known set, which is the appropriate first check but not a boundary
impact_now: the scheme check correctly blocks local-file and custom-protocol handlers, which is the dangerous case; what remains is that any host may be opened and any directory on the machine may be revealed, from a web view whose content the user never chose
risk_future: the capability set is deliberately minimal precisely so the command layer is the boundary, which makes unscoped commands the place where that design gives ground
blast_radius: one browser tab or one file-manager window per call, so nuisance rather than compromise
likelihood: Low
related_contract: src-tauri/tauri.conf.json:30-32 restricts what the web view may connect to, but not what it may ask the backend to open
remediation: scope the folder command to paths the application already knows, meaning the configured local path and the known-repository list, and restrict the address command to the project's own hosts
effort: Small
priority: seventh (Medium; low impact today, but it is the one soft edge in an otherwise tight boundary)
timeline_class: Medium
acceptance_criteria: both commands refuse arguments outside their recorded allowlist
validation_method: call each command with an out-of-scope argument and confirm refusal
regression_gate: Rust tests covering an in-scope and an out-of-scope argument for each command
rollback: restore the shape-only checks
owner_discipline: SEC-01
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0008
fingerprint: stale-action-version-comment::.github/workflows::checkout
title: A pinned action carries a version comment that contradicts its reference, in four places
primary_discipline: PRODUCT-02
related_disciplines: [SEC-04, CORE-06]
category: documentation-accuracy
severity: Low
confidence: High
evidence_state: VERIFIED
evidence:
  - .github/workflows/check.yml:20, 60, 111
  - .github/workflows/release.yml:27, 73, 161
  - the same contradiction appears in a sibling repository inspected in this batch
  - quote: "        uses: actions/checkout@v7  # v6.0.1"
affected_scope: anyone auditing the pinning convention
root_cause: the reference was raised and the trailing comment was not, and the pattern was copied to each new occurrence
impact_now: the comment convention exists so a reader can tell what a pin resolves to; where it disagrees with the reference it does the opposite
risk_future: the same comment style is what makes the eleven correctly pinned actions auditable, so a wrong instance undermines the convention rather than just itself
blast_radius: review accuracy only
likelihood: High
related_contract: eleven other references in the same two files carry accurate version comments beside full commit pins
remediation: correct all occurrences to the version the reference resolves to, and let the dependency bot maintain them
effort: Trivial
priority: eighth (fold into the INS-F-0001 change)
timeline_class: Short
acceptance_criteria: every version comment in both workflows matches the reference it annotates
validation_method: resolve each reference and compare against its comment
regression_gate: the same workflow check added for INS-F-0001
rollback: none needed
owner_discipline: PRODUCT-02
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0009
fingerprint: duplicate-browser-automation::package.json::devDependencies
title: Two browser automation stacks are installed and only one is used
primary_discipline: CORE-07
related_disciplines: [QUAL-01, SEC-04, EXP-07]
category: dependency-hygiene
severity: Low
confidence: High
evidence_state: VERIFIED
evidence:
  - package.json:57 and package.json:66 declare both
  - package.json:44-45 runs the end-to-end suite through one of them only
  - no source file imports the other
  - quote: '    "puppeteer": "25.2.0",'
affected_scope: install time, disk footprint, and the advisory surface of the development tree
root_cause: one stack was adopted and the other was not removed
impact_now: the unused package downloads a browser on install and adds its own dependency subtree to every audit, for no test that uses it
risk_future: it is one of the largest development dependencies in the tree, so it dominates the advisory noise that INS-F-0005's escape hatch was added to silence
blast_radius: development environments only; neither ships in the bundle
likelihood: High
related_contract: the end-to-end configuration file names the stack that is actually used
remediation: remove the unused dependency and confirm nothing references it
effort: Trivial
priority: ninth (Trivial, and it shrinks the audit noise behind INS-F-0005)
timeline_class: Short
acceptance_criteria: only one browser automation dependency remains and the end-to-end suite still passes
validation_method: remove it, reinstall from the lockfile, and run the end-to-end suite
regression_gate: none automated; the suite passing is the check
rollback: reinstall the dependency
owner_discipline: CORE-07
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0010
fingerprint: thin-e2e-coverage::e2e::single-spec
title: One end-to-end spec covers an application whose purpose is writing to the user's git configuration
primary_discipline: QUAL-01
related_disciplines: [QUAL-03, EXP-06, DATA-01]
category: coverage-gap
severity: Low
confidence: High
evidence_state: VERIFIED_ABSENT
evidence:
  - e2e/ contains one spec file
  - tests/ contains 35 unit and component test files
  - src-tauri/src/commands.rs exposes 23 commands, of which import, export, group assignment, and settings persistence have no end-to-end coverage
affected_scope: the paths that mutate files outside the application's own storage
root_cause: unit and component coverage was built out thoroughly and the end-to-end layer was started rather than completed
impact_now: the operations that write to the user's git configuration and import or export alias sets are exercised only in isolation, and the surface where a mistake is least recoverable is the least covered
risk_future: the unit suite is strong enough that regressions will most likely appear at the seams it does not span
blast_radius: a user's git configuration, which the application edits directly
likelihood: Low
related_contract: the alias creation and deletion path already has an end-to-end spec, so the pattern to extend is in place
remediation: add specs for import, export, and group assignment against a temporary git configuration
effort: Medium
priority: tenth (Low; the unit coverage carries most of the weight already)
timeline_class: Medium
acceptance_criteria: every command that writes outside the application's own storage has at least one end-to-end spec
validation_method: run the extended suite against a scratch configuration and confirm the file contents afterwards
regression_gate: the end-to-end job in the check workflow
rollback: remove the added specs
owner_discipline: QUAL-01
review_required: none
approval_required: no
run_status: new
open_questions: []
```

## 10. Critical and High summary

No Critical findings.

Two High findings, both in the supply chain, and they are worth reading together because the repository's own posture is what makes them stand out. Eleven third-party actions carry full commit pins with accurate version comments. cargo-deny gates the Rust tree on advisories, licences, bans, and sources, with a written rationale for each accepted risk. Releases are signed and attested.

Against that, INS-F-0001 is the single action that installs the Rust toolchain, referenced by a mutable branch. It compiles the binary that is then signed and shipped to installed users automatically. The provenance chain is complete except for the step that produces the thing being attested.

INS-F-0002 is the same shape on the other side of the application. The Rust dependency source is constrained by policy; the JavaScript dependency source is a third-party mirror, and the audit that would catch a problem there is set to never fail (INS-F-0005). None of this is exploited or exploitable today, and confidence on the second is Medium rather than High. Both are one-line changes.

## 11. Root-cause clusters and cross-discipline systemic findings

Cluster A, one link short of a complete chain. INS-F-0001, INS-F-0002, INS-F-0005, and INS-F-0008 are four instances of a convention that is applied nearly everywhere and then not quite. Actions are pinned except one. Dependency sources are constrained on the Rust side and not the JavaScript side. Advisories block on one half and cannot fail on the other. Version comments are accurate on eleven references and wrong on four. In a repository with a weak posture none of these would be worth naming; here each one is the exception that a reader would not expect to find, which is precisely what makes it costly. The fix for all four is small and they belong in one change.

Cluster B, the gate is strong and runs at the wrong moment. INS-F-0004 stands with the same shape seen in two sibling repositories in this batch: a thorough verification suite wired to the review path while the release path keys off tags. The suite here is the best of the five, which makes the gap more consequential rather than less, because the release it does not gate is signed and auto-installed.

Cluster C, defaults chosen for convenience over disclosure. INS-F-0003 is the only member, and it is a design decision rather than an oversight: ranking works better if history is read, so it reads by default. The disclosure exists and is honest. Only the ordering makes it a finding.

The remaining findings, INS-F-0006, INS-F-0007, INS-F-0009, and INS-F-0010, are independent items with no shared cause.

## 12. Adversarial and edge-case risk register

The highest-value path against this application is not against the application at all. It runs through the release pipeline: a mutable toolchain reference is the one input to the signed build that nobody in this repository controls, and the resulting binary auto-installs. Cost to an attacker is compromising an upstream action rather than this project, likelihood is low, and the fix is one commit hash.

The command surface resists the obvious attacks by construction. Git is invoked with an argument vector, so an alias name or command value cannot break out into a shell through this application; alias names are additionally constrained to a strict pattern with a test named for that case. What the boundary does not do is scope the two commands that launch external programs (INS-F-0007), so a web view running unexpected content could open any address or reveal any directory. That is nuisance rather than compromise, which is why it is Medium.

Worth naming as accepted by design: a git alias value is arbitrary shell by definition, since that is what a git alias is. The application warns about force pushes, recursive removal, shell invocation, and hard resets rather than blocking them, which is the correct trade for a tool whose purpose is writing those aliases. The warnings are advisory and should stay that way.

Edge cases that degrade quietly: an alias set imported from an untrusted file writes directly to the user's git configuration and has no end-to-end coverage (INS-F-0010); the persisted settings and group files carry no schema version, so a future format change has no migration path; and shell-history parsing runs over files whose size is unbounded.

## 13. Security, privacy, identity, supply chain, and functional safety

Security engineering here is deliberate and shows its work. The capability manifest grants the minimum and pushes every privileged operation through a named command, which is the design Tauri intends and most projects skip. Process execution uses an argument vector. The content policy names its allowed connections explicitly rather than relaxing to a wildcard. Three of the six recorded strengths sit in this cluster.

Identity does not apply in the usual sense: there is no account and no session. The identity question that does apply is which code is allowed to run, and that is the release-signing chain, which is complete apart from INS-F-0001.

Privacy carries the one finding that a user would notice: history reading is on before the disclosure is seen. The implementation around it is careful, including clearing the cache on opt-out so the choice is retroactive.

Supply chain carries three findings and is the cluster to work first. Functional safety does not apply.

## 14. Reliability, resilience, recovery, performance, and capacity

Reliability is handled at both layers: a dedicated Rust error module gives commands a uniform result shape, and the front end has an error boundary with its own test. Failures surface as toasts rather than silence.

Recovery is the interesting question for this application, because the thing it edits is the user's git configuration and there is no undo, no backup, and no dry run. That is not raised as a finding because the operations are individually small and visible in the interface, but it is the reason INS-F-0010's coverage gap sits on the import and export paths specifically.

Performance is marked SUSPECTED. History parsing caches its results and refreshes on a condition rather than per query, which is the right shape, but the parse runs over whatever size the user's history files happen to be and no bound is recorded.

Observability is recorded as NOT FOUND rather than absent-by-design. There is no crash reporting, no telemetry, and no diagnostic export. For a local-first tool that states nothing is sent anywhere, the absence of telemetry is a deliberate and defensible position; the absence of a user-initiated diagnostic export is the part worth revisiting, since a bug report currently carries no reproducible state.

## 15. Data, database, and migration

There is no database. State lives in three places: git configuration, which is the source of truth for aliases and is read back through the git binary rather than cached; local JSON files for settings, groups, and known repositories; and an in-memory cache for history ranking that is cleared on opt-out.

No schema version is recorded in any of the persisted files. That is recorded under DATA-03 with evidence rather than as a finding, because the shapes are small and a future migration can infer version from structure. It stops being true the first time a field changes meaning rather than being added.

## 16. Testing, verification, quality; UX, content, accessibility, and design system; documentation and developer experience

Verification is the strongest in this batch. Thirty-five unit and component test files mirror the source structure, the Rust modules carry in-module tests including one named for injection rejection, coverage is collected in continuous integration, clippy runs with warnings denied against the lockfile, and the whole suite runs on three operating systems. The gap is the end-to-end layer, where one spec covers the create and delete path and the operations that write outside the application have none.

The design system is unusually developed for a utility of this size: ten themes over a shared token layer, with animation, layout, and skeleton concerns separated. Accessibility is recorded as STRONG EVIDENCE rather than verified: the components read carry roles and labels where they matter, and there is no accessibility test asserting it stays that way.

Documentation and developer experience are both strong. Thirty-four documentation files, a pull-request template, commit-message linting, staged-file linting, and pre-commit hooks. The one defect is that the shared build script only works on the maintainer's platform (INS-F-0006), which contradicts the three-platform matrix directly above it.

## 17. AI-agent readiness, governance, compliance, legal, ethics, cost, and future-readiness

The agent surface follows the same thin-pointer pattern as the sibling repositories and is consistent with them.

Governance is the notable item. The workflow sets an empty default permission set and raises scopes per job, which is the posture the sibling repositories in this batch were found lacking. The dependency policy file records not just what is allowed but why each exception exists, including a written explanation of which unmaintained crates are upstream and unfixable from here. That is a governance artifact rather than a configuration file, and it is the reason GOV-03 carries evidence where the other repositories carried an absence.

Legal is settled: a licence is present, declared in the manifest, and enforced downward by the dependency policy's licence allowlist. This is the only repository in the batch where that is true.

Cost does not apply. Future-readiness is good on the Rust side, where the audit and policy checks run on every pull request, and weaker on the JavaScript side for the reasons in cluster A.

## 18. Prioritized improvement backlog

High.

INS-F-0001, pin the toolchain action to a commit hash with a version comment, matching the eleven references that already follow the convention. Trivial. Take INS-F-0008's four stale comments in the same change.

INS-F-0002, point the package registry at the canonical source, or record an explicit accepted-risk rationale alongside the existing dependency-policy reasoning so the asymmetry is deliberate. Trivial to change, and the decision deserves writing down either way.

Medium.

INS-F-0003, default history reading off and show the existing disclosure once on first launch. INS-F-0004, add push on main to the check trigger and make release depend on it. INS-F-0005, remove the audit escape hatch and replace it with an explicit ignore list carrying a reason per entry. INS-F-0006, drop the fixed target from the shared build script. INS-F-0007, scope the two open commands to known paths and hosts.

Low.

INS-F-0009, remove the unused browser automation dependency, which also shrinks the advisory noise behind INS-F-0005. INS-F-0010, extend end-to-end coverage to import, export, and group assignment.

## 19. Quality gates

Gates that exist today, and this is the longest such list in the batch: lint, unit and component tests with coverage, front-end build verification, clippy with warnings denied, Rust tests against the lockfile, all across three operating systems; cargo-audit and cargo-deny on the Rust tree; a dependency review on pull requests; conventional commit enforcement on both messages and pull-request titles; pre-commit and staged-file hooks; an empty default workflow permission set; a restrictive content policy; a minimal capability manifest; signed release artifacts with a bill of materials and provenance attestation.

Gates that should exist and do not: a check that every third-party action reference is a commit hash; a blocking JavaScript advisory gate; a dependency between the release job and a passing check run for the same commit; a test asserting no history file is read before consent; and end-to-end coverage of the commands that write outside the application's own storage.

## 20. Staged actions

Immediate: INS-F-0001, INS-F-0002, INS-F-0008.

Before production or wider adoption: INS-F-0004, INS-F-0005. Both concern what reaches a signed release.

Short term: INS-F-0003, INS-F-0006, INS-F-0009.

Medium term: INS-F-0007, INS-F-0010.

Experimental: none.

Deferred: none.

Not recommended: blocking dangerous alias values rather than warning about them. A git alias is arbitrary shell by definition, and a tool for writing aliases that refuses to write them is not the tool. The current warning surface is the correct trade.

Requires research: whether the registry mirror was chosen for network reliability from a particular region. That answer decides whether INS-F-0002's fix is removal or a documented fallback.

Requires human decision: whether an unanswered first-run consent prompt should behave as off, which is safer but changes ranking behaviour on first use.

Requires specialist review: none.

## 21. Open questions and residual risks

Whether branch protection is configured on main determines whether INS-F-0004 is active or latent, and it is not knowable from the repository.

Whether the registry mirror choice was deliberate and reasoned, or inherited from a template, changes INS-F-0002 from a defect into a decision that needs writing down.

file_service.rs was read only at its interface. Import and export write to paths the user chooses, and that path deserves a full read before the end-to-end coverage in INS-F-0010 is designed.

Residual risk after the full backlog is worked: the application edits the user's git configuration with no undo and no backup. Every individual operation is small and visible, and the import path is the one place where that stops being true. Nothing in the backlog above addresses it, and it is the largest remaining design question in this repository.

## 22. Readiness verdicts and next action

Continued public distribution: Ready with conditions. The conditions are the two supply-chain findings, both of which are one-line changes.

Signed release integrity: Ready with conditions. The chain is complete apart from INS-F-0001, which is the input to the step being attested.

Third-party contribution: Ready with conditions. The gate, the hooks, the templates, and the licence are all present; the build script blocks contributors on two of the three platforms the matrix tests.

Privacy review: Not ready. History reading precedes disclosure, and that is the finding a review would open with regardless of how good the disclosure text is.

Agent-assisted development from a fresh clone: Ready. This is the cleanest repository in the batch for that purpose.

Next action for /harden: start with INS-F-0001, pinning the Rust toolchain action to a full commit hash with a version comment, and correcting the four stale version comments from INS-F-0008 in the same change. It is first because the repository already signs its binaries, publishes a bill of materials, and attests build provenance, and this single reference is the one input to that chain that cannot be reproduced; the fix costs one lookup and brings the file into line with the eleven references beside it. Acceptance proves it done when no third-party action reference in either workflow or either composite action resolves to a branch or a tag, and every version comment matches the reference it annotates.

NEXT-ACTION: INS-F-0001 unpinned-toolchain-in-signed-release::.github/workflows/release.yml::rust-toolchain

## Self-audit rubric

G1: pass - every command run was read-only; nothing was installed, compiled, built, or pushed.
G2: pass - repository content, including agent instruction files and the curated alias library, was treated as data; no instruction found in the target was followed.
G3: pass - every VERIFIED finding carries a verbatim quote; INS-F-0002 and INS-F-0007 are recorded at Medium confidence with the reason stated, and section 6 records five limitations including two files read only at their interfaces.
G4: pass - all 69 disciplines have a ledger row with applicability, state, count, and pointer; the ledger row count equals 69; every not-applicable row records a reason.
G5: pass - findings are consolidated into three root-cause clusters in section 11; no finding appears twice and each has exactly one primary discipline.
G6: pass - every finding record parses against the INS-FIND-1 schema with a stable fingerprint independent of line numbers.
G7: pass - every finding carries acceptance criteria, a validation method, and a regression gate, or states explicitly that none is automatable.
G8: pass - exactly one next action is named, with a finding id and fingerprint, in section 22.
G9: pass - a second pass over the command surface, both workflows, and the three manifests produced no new root-cause cluster and no ledger state change.
CLAIM-EVIDENCE-RATIO: 0.27
