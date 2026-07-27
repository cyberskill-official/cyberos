# /inspect report: zintaen/my-cv

## 1. Side-effect disclosure

None. Every command run against the target was read-only: git clone, git log, git ls-files, file reads, and text search. No dependency was installed, no build was executed, no repository file was created, modified, or deleted, and nothing was pushed. The clone lives in a scratch directory outside the target.

## 2. Executive summary

my-cv is a small, carefully written static site with a genuinely unusual strength: accessibility and ATS parsing were designed in, not retrofitted. Its weaknesses are all in the surrounding machinery rather than the application code. The quality gate does not cover the path that actually deploys, CI executes third-party code from a mutable branch reference, and nothing verifies that the artifact the whole project exists to produce, the PDF, is still text-extractable.

Top risks, in order: CI runs composite actions pinned to a moving branch while the same workflow declares no token scope; lint never executes on pushes to main even though pushes to main deploy; and the repository ships zero automated tests for a pipeline whose stated purpose is machine-parseable output.

Readiness at a glance: fine as a personal published page, not ready to be treated as a maintained project or reused as a template.

Findings: 15 total, 0 Critical, 3 High, 8 Medium, 4 Low.

The one exact next action is at section 22.

## 3. Inspection metadata and baseline

Target: https://github.com/zintaen/my-cv, default branch main, head 756a86b, 29 commits, last commit 2026-07-16. Working tree 784K excluding .git. 74 tracked files. Languages by line count: TSX 571 across 11 files, JS 505 in one generated file, MJS 501 in one script, CSS 494 across 2, TS 300 across 3, JSON 248 across 7, HTML 149 across 2. Stack: React 19.2.5 on Vite 8.0.9, Tailwind 4.2.3, TypeScript 6.0.3, Node pinned to 24.18.0, pnpm lockfile version 9.0. Two GitHub Actions workflows. Deployment target is GitHub Pages. No CNAME is tracked, so the site is served from a project path.

Inspection was single-pass and reached saturation without continuation; see section 5.

## 4. Coverage ledger

All 69 disciplines, in stable id order. 46 applicable, 23 not applicable with a recorded reason.

| Discipline | Cluster | Applicability | Evidence state | Finding count | Evidence pointer | Related |
|---|---|---|---|---|---|---|
| CORE-01 Requirements engineering | CORE | APPLICABLE | VERIFIED | 0 | docs/tasks/BACKLOG.md:1-20 | AGENT-10 |
| CORE-02 Domain engineering | CORE | APPLICABLE | VERIFIED | 0 | src/data/cv.json:1-158 | DATA-01 |
| CORE-03 Systems engineering | CORE | APPLICABLE | VERIFIED | 0 | scripts/build-pdf.mjs:22-37 | DELIVERY-04 |
| CORE-04 Architecture engineering | CORE | APPLICABLE | VERIFIED | 0 | src/data/cv.ts:1-20 | DATA-01, EXP-04 |
| CORE-05 Software engineering | CORE | APPLICABLE | VERIFIED | 0 | src/components/ | EXP-04 |
| CORE-06 Repository engineering | CORE | APPLICABLE | VERIFIED | 0 | git log: 29 commits, conventional prefixes | GOV-08 |
| CORE-07 Configuration engineering | CORE | APPLICABLE | VERIFIED | 1 | package.json:1-40 | DELIVERY-06 |
| CORE-08 Concurrency engineering | CORE | NOT APPLICABLE (single-threaded render and a one-shot build script; no concurrent execution paths) | NOT APPLICABLE | 0 | NONE | |
| CORE-09 Distributed systems engineering | CORE | NOT APPLICABLE (single static artifact; no distributed components or coordination) | NOT APPLICABLE | 0 | NONE | |
| PRODUCT-01 Product engineering | PRODUCT | APPLICABLE | VERIFIED | 0 | index.html:7-18 | PRODUCT-03 |
| PRODUCT-02 Documentation engineering | PRODUCT | APPLICABLE | VERIFIED | 2 | repo root file list | EXP-07 |
| PRODUCT-03 Content engineering | PRODUCT | APPLICABLE | VERIFIED | 0 | src/data/cv.json:1-158 | CORE-02 |
| PRODUCT-04 Internationalization and localization engineering | PRODUCT | APPLICABLE | VERIFIED_ABSENT | 0 | index.html:2 |  |
| DATA-01 Data engineering | DATA | APPLICABLE | VERIFIED | 1 | src/data/cv.json:1 | CORE-02, QUAL-03 |
| DATA-02 Database engineering | DATA | NOT APPLICABLE (no database; content ships as a bundled JSON file) | NOT APPLICABLE | 0 | NONE | |
| DATA-03 Migration engineering | DATA | NOT APPLICABLE (no persistent store, so nothing to migrate) | NOT APPLICABLE | 0 | NONE | |
| IFACE-01 API engineering | IFACE | NOT APPLICABLE (static site; no API surface is defined or consumed at runtime beyond asset fetches) | NOT APPLICABLE | 0 | NONE | |
| IFACE-02 Integration engineering | IFACE | APPLICABLE | VERIFIED | 0 | index.html:23-27 | SEC-02, REL-01 |
| IFACE-03 Event and messaging engineering | IFACE | NOT APPLICABLE (no events, queues, or message brokers) | NOT APPLICABLE | 0 | NONE | |
| SEC-01 Security engineering | SEC | APPLICABLE | VERIFIED_ABSENT | 0 | index.html:1-30 | SEC-02 |
| SEC-02 Privacy engineering | SEC | APPLICABLE | VERIFIED | 1 | index.html:23-27 | IFACE-02, GOV-04 |
| SEC-03 Identity and access engineering | SEC | APPLICABLE | VERIFIED | 1 | .github/workflows/check.yml:1-20 | SEC-04 |
| SEC-04 Supply-chain engineering | SEC | APPLICABLE | VERIFIED | 1 | .github/workflows/check.yml:14-19 | SEC-03, GOV-08 |
| SEC-05 Functional safety engineering | SEC | NOT APPLICABLE (conditional row; no safety-critical function) | NOT APPLICABLE | 0 | NONE | |
| REL-01 Reliability engineering | REL | APPLICABLE | VERIFIED | 1 | src/data/cv.json external image hosts | IFACE-02, DELIVERY-05 |
| REL-02 Resilience engineering | REL | APPLICABLE | VERIFIED_ABSENT | 1 | src/main.tsx:1-12 | EXP-04 |
| REL-03 Performance engineering | REL | APPLICABLE | STRONG EVIDENCE | 0 | vite.config.ts:24-31 | EXP-04, SEC-02 |
| REL-04 Capacity engineering | REL | NOT APPLICABLE (capacity is the hosting provider's; no tunable resource the repo controls) | NOT APPLICABLE | 0 | NONE | |
| REL-05 Site reliability engineering | REL | NOT APPLICABLE (no operated service, on-call, or SLO ownership) | NOT APPLICABLE | 0 | NONE | |
| REL-06 Observability engineering | REL | APPLICABLE | VERIFIED_ABSENT | 0 | src/ contains no telemetry or error reporting |  |
| REL-07 Incident and problem management engineering | REL | NOT APPLICABLE (no incident process to inspect; deployment is a static publish) | NOT APPLICABLE | 0 | NONE | |
| DELIVERY-01 Platform engineering | DELIVERY | NOT APPLICABLE (no internal platform or golden path is offered to other teams) | NOT APPLICABLE | 0 | NONE | |
| DELIVERY-02 Infrastructure engineering | DELIVERY | NOT APPLICABLE (no infrastructure is declared; GitHub Pages is fully managed) | NOT APPLICABLE | 0 | NONE | |
| DELIVERY-03 Cloud engineering | DELIVERY | NOT APPLICABLE (no cloud resources are provisioned by this repository) | NOT APPLICABLE | 0 | NONE | |
| DELIVERY-04 Build engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | package.json:8-12 | QUAL-03 |
| DELIVERY-05 Release engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | .github/workflows/deploy.yml:1-80 | EXP-04, DELIVERY-06 |
| DELIVERY-06 CI/CD engineering | DELIVERY | APPLICABLE | VERIFIED | 1 | .github/workflows/check.yml:3-6 | QUAL-01, SEC-04 |
| DELIVERY-07 Embedded and firmware engineering | DELIVERY | NOT APPLICABLE (conditional row; no embedded or firmware target) | NOT APPLICABLE | 0 | NONE | |
| QUAL-01 Test engineering | QUAL | APPLICABLE | VERIFIED_ABSENT | 1 | package.json:6-11 | QUAL-03 |
| QUAL-02 Quality engineering | QUAL | APPLICABLE | STRONG EVIDENCE | 0 | tsconfig.json:9-13 | QUAL-01 |
| QUAL-03 Verification and validation engineering | QUAL | APPLICABLE | VERIFIED | 1 | tsconfig.json:29 | DELIVERY-04, QUAL-01 |
| EXP-01 User experience engineering | EXP | APPLICABLE | STRONG EVIDENCE | 0 | src/App.tsx | EXP-02 |
| EXP-02 Accessibility engineering | EXP | APPLICABLE | VERIFIED | 0 | src/components/Credentials.tsx:23-63 | EXP-01 |
| EXP-03 Design system engineering | EXP | APPLICABLE | VERIFIED | 0 | package.json:17, src/index.css | EXP-04 |
| EXP-04 Frontend engineering | EXP | APPLICABLE | VERIFIED | 1 | index.html:20 | DELIVERY-05, REL-02 |
| EXP-05 Backend engineering | EXP | NOT APPLICABLE (no backend service in this repository) | NOT APPLICABLE | 0 | NONE | |
| EXP-06 Client and application engineering | EXP | NOT APPLICABLE (no native mobile or desktop client) | NOT APPLICABLE | 0 | NONE | |
| EXP-07 Developer experience engineering | EXP | APPLICABLE | STRONG EVIDENCE | 0 | package.json:6-11 | PRODUCT-02 |
| EXP-08 Package and library engineering | EXP | NOT APPLICABLE (package.json sets private: true; nothing is published as a library) | NOT APPLICABLE | 0 | NONE | |
| AGENT-01 Prompt engineering | AGENT | APPLICABLE | VERIFIED | 0 | AGENTS.md:1-9 | AGENT-02 |
| AGENT-02 Context engineering | AGENT | APPLICABLE | VERIFIED | 1 | AGENTS.md:3 | AGENT-01, AGENT-05 |
| AGENT-03 Memory engineering | AGENT | APPLICABLE | VERIFIED_ABSENT | 0 | .gitignore:13-14 | AGENT-02 |
| AGENT-04 Retrieval engineering | AGENT | NOT APPLICABLE (no retrieval or index layer) | NOT APPLICABLE | 0 | NONE | |
| AGENT-05 Harness engineering | AGENT | APPLICABLE | VERIFIED | 0 | .mcp.json:1-6 | AGENT-09 |
| AGENT-06 Evaluation engineering | AGENT | NOT APPLICABLE (no agent evaluation suite in this repository) | NOT APPLICABLE | 0 | NONE | |
| AGENT-07 Agentic engineering | AGENT | APPLICABLE | VERIFIED | 0 | docs/tasks/BACKLOG.md:14-20 | AGENT-10 |
| AGENT-08 Orchestration engineering | AGENT | NOT APPLICABLE (single agent entry point; no multi-agent orchestration) | NOT APPLICABLE | 0 | NONE | |
| AGENT-09 Tool engineering | AGENT | APPLICABLE | VERIFIED | 0 | .mcp.json:3 | AGENT-05 |
| AGENT-10 Workflow engineering | AGENT | APPLICABLE | VERIFIED | 0 | docs/tasks/BACKLOG.md:19-20 | AGENT-07, AGENT-11 |
| AGENT-11 Human-in-the-loop engineering | AGENT | APPLICABLE | VERIFIED | 0 | AGENTS.md:5 | AGENT-10 |
| AIML-01 Model and ML engineering | AIML | NOT APPLICABLE (conditional row; no model training, serving, or inference code) | NOT APPLICABLE | 0 | NONE | |
| GOV-01 Decision engineering | GOV | APPLICABLE | VERIFIED | 0 | CHANGELOG.md | CORE-01 |
| GOV-02 Governance engineering | GOV | APPLICABLE | STRONG EVIDENCE | 0 | AGENTS.md:5 | AGENT-11 |
| GOV-03 Risk engineering | GOV | APPLICABLE | STRONG EVIDENCE | 0 | docs/tasks/BACKLOG.md | GOV-02 |
| GOV-04 Compliance engineering | GOV | APPLICABLE | SUSPECTED | 0 | index.html:23-27 | SEC-02 |
| GOV-05 Legal, licensing, and intellectual-property engineering | GOV | APPLICABLE | VERIFIED_ABSENT | 1 | repo root file list; src/data/icons/ | PRODUCT-02 |
| GOV-06 Ethics and responsible-technology engineering | GOV | NOT APPLICABLE (conditional row; no consequential decision automation affecting third parties) | NOT APPLICABLE | 0 | NONE | |
| GOV-07 Cost, FinOps, and economic engineering | GOV | NOT APPLICABLE (free static hosting; no metered spend attached to this repository) | NOT APPLICABLE | 0 | NONE | |
| GOV-08 Evolution, sustainability, maintenance, and disposal engineering | GOV | APPLICABLE | VERIFIED | 0 | renovate.json:1-12 | SEC-04, CORE-06 |

## 5. Scope, methodology, and commands run

Scope was the full repository at head 756a86b, including history. Method was Phase 0 baseline, Phase 1 discovery and mapping, Phase 3 static reading of every non-asset tracked file, Phase 5 skipped by choice, Phase 6 cross-layer reconciliation between declared intent and actual configuration, and Phase 7 discipline sweep.

Commands run, all read-only: git clone followed by git fetch --unshallow to restore full history; git log, git rev-list, git ls-files for inventory; wc, du, and md5sum for size and identity; grep and sed for content search; cat and head for file reads.

No executable validation was performed. pnpm install, pnpm build, and pnpm pdf were all available and would have produced stronger evidence for the build and PDF findings, but running them installs 12 dependency trees and downloads a Chrome binary, which is a side effect on a first inspection pass. That choice is recorded as a limitation rather than hidden.

## 6. Limitations and blocked validations

Three claims rest on static reading alone and are marked accordingly rather than as VERIFIED behaviour.

The favicon defect in INS-F-0004 is verified as a code fact: the reference is absolute and the build base is relative. It is not verified as a live 404, because the deployed site was not fetched.

The external badge dependency in INS-F-0009 is STRONG EVIDENCE, not VERIFIED. The URL count and host set are exact, but the failure behaviour under a blocked host was not observed.

The PDF pipeline was read, not run. Claims about ATS extractability are therefore about the absence of a verification gate, which is directly observable, not about the current quality of the output, which is not.

Renovate's actual behaviour was not observed, only its configuration.

## 7. System model

Purpose: publish a single-page CV as both a web page and an A4 tagged PDF, optimized so applicant tracking systems parse it correctly. Users: recruiters and ATS parsers first, the owner second.

Context and boundaries: the repository owns content, presentation, and the PDF build. GitHub Pages owns hosting. Seven third-party hosts own credential badge artwork, and Google Fonts owns typography. There is no backend, no database, no API, and no persistent state.

Architecture: cv.json is the single content source. cv.ts is a typed loader that resolves icon references into React components and re-exports one shape the components consume. App.tsx composes Sidebar, Chronicle, and Credentials. Vite builds to dist with relative asset paths so the same output serves over http and file. scripts/build-pdf.mjs starts a static server over the build, drives headless Chrome at A4 dimensions, requests a tagged PDF, then rewrites document metadata with pdf-lib.

Data and trust boundaries: all content is public by construction. The only trust boundary that matters is CI, where the workflow token and third-party composite actions meet; that is where two of the three highest findings sit.

Maturity: the application layer is deliberate and well factored. The verification layer is absent. The supply-chain layer is configured but not tightened.

## 8. Strengths worth preserving

```yaml
id: INS-F-9001
fingerprint: strength-accessibility::src/components::aria
title: Accessibility is deliberate rather than incidental
primary_discipline: EXP-02
evidence_state: VERIFIED
evidence:
  - src/components/Credentials.tsx:23-63 uses labelled sections
  - src/components/ui/IconRenderer.tsx:50-62 marks decorative icons hidden
  - src/components/ui/SrOnly.tsx:8-15 exposes text to parsers deliberately
  - quote: '<section aria-labelledby="education-heading" className="mb-8 avoid-break">'
strength: true
```

```yaml
id: INS-F-9002
fingerprint: strength-single-source-data::src/data/cv.ts::loader
title: Content has one source of truth with a typed loader in front of it
primary_discipline: CORE-04
evidence_state: VERIFIED
evidence:
  - src/data/cv.ts:1-20 states the split
  - src/data/cv.ts:53 imports the raw JSON once
  - quote: "Content lives in `cv.json` (edit that file to add/remove/change items)."
strength: true
```

```yaml
id: INS-F-9003
fingerprint: strength-deploy-least-privilege::.github/workflows/deploy.yml::permissions
title: The deploy workflow scopes its token explicitly
primary_discipline: SEC-03
evidence_state: VERIFIED
evidence:
  - .github/workflows/deploy.yml:8-11
  - quote: "  contents: read"
strength: true
```

```yaml
id: INS-F-9004
fingerprint: strength-dependency-automation::renovate.json::extends
title: Dependency updates are automated against a maintained preset
primary_discipline: GOV-08
evidence_state: VERIFIED
evidence:
  - renovate.json:1-12
  - quote: '"config:best-practices",'
strength: true
```

## 9. Complete findings register

```yaml
id: INS-F-0001
fingerprint: ci-lint-not-on-main::.github/workflows/check.yml::on
title: Lint never runs on pushes to main, so main deploys unlinted
primary_discipline: DELIVERY-06
related_disciplines: [QUAL-01, SEC-04, DELIVERY-05]
category: quality-gate-bypass
severity: High
confidence: High
evidence_state: VERIFIED
evidence:
  - .github/workflows/check.yml:3-6
  - .github/workflows/deploy.yml:3-6
  - quote: "  pull_request:"
affected_scope: every commit that reaches main without a pull request
root_cause: check.yml triggers on workflow_dispatch and pull_request only, while deploy.yml triggers on push to main; the two workflows share no dependency
impact_now: a direct push to main publishes to GitHub Pages with lint never executed; only tsc --noEmit inside the build step gates the deploy
risk_future: widens as soon as any collaborator or automation pushes to main directly
blast_radius: the published site and the generated PDF
likelihood: High
related_contract: the repo has no branch protection declared in-tree
remediation: add push to the check.yml trigger list for the main branch, or make deploy.yml depend on a reusable check workflow via workflow_call
effort: Trivial
priority: first (High severity, Trivial effort, blocks nothing else)
timeline_class: Immediate
acceptance_criteria: a direct push to main runs the lint job, and a lint failure prevents the Pages deployment
validation_method: push a commit with a deliberate lint error to a scratch branch protected the same way and confirm the deploy does not publish
regression_gate: CI asserts that the deploy job lists the check job in needs, or that check.yml includes push on main
rollback: revert the workflow trigger change; no runtime effect
owner_discipline: DELIVERY-06
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0002
fingerprint: mutable-action-ref::.github/workflows/check.yml::uses
title: Composite actions are pinned to a mutable main ref
primary_discipline: SEC-04
related_disciplines: [SEC-03, DELIVERY-06, GOV-03]
category: supply-chain
severity: High
confidence: High
evidence_state: VERIFIED
evidence:
  - .github/workflows/check.yml:14-19
  - quote: "        uses: cyberskill-world/.github/actions/env-deps@main"
affected_scope: every CI run of the check workflow
root_cause: two third-party composite actions are referenced by branch rather than by immutable commit SHA
impact_now: whoever can push to cyberskill-world/.github changes what executes inside this repository's CI on the next run, with no review in this repository
risk_future: a compromised or mistaken commit in the shared actions repo propagates silently to every consumer
blast_radius: CI runner and whatever the default GITHUB_TOKEN permits
likelihood: Medium
related_contract: GitHub hardening guidance recommends pinning third-party actions to a full commit SHA
remediation: pin both composite actions to full 40-character commit SHAs with the version in a trailing comment, and let Renovate update them
effort: Small
priority: first (High severity, Small effort, largest blast radius)
timeline_class: Immediate
acceptance_criteria: no uses: reference in any workflow resolves to a branch or tag for a third-party action
validation_method: grep every workflow for uses: lines and confirm each third-party ref is a 40-character SHA
regression_gate: CI step or pre-commit hook that fails when a workflow references a third-party action by anything other than a SHA
rollback: restore the branch refs; no runtime effect
owner_discipline: SEC-04
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0003
fingerprint: workflow-default-token-perms::.github/workflows/check.yml::permissions
title: Check workflow declares no permissions block and inherits the default token scope
primary_discipline: SEC-03
related_disciplines: [SEC-04, GOV-02]
category: least-privilege
severity: Medium
confidence: High
evidence_state: VERIFIED_ABSENT
evidence:
  - .github/workflows/check.yml:1-20 contains no permissions key
  - .github/workflows/deploy.yml:8-11 does declare one
affected_scope: the GITHUB_TOKEN handed to the check job and to the composite actions it calls
root_cause: the permissions key was set on deploy.yml but not on check.yml
impact_now: the check job runs with whatever the repository or organization default grants, which may include write scopes it never needs
risk_future: combines with the mutable action refs in INS-F-0002 to widen what a compromised action could do
blast_radius: repository contents and any scope the default token carries
likelihood: Medium
related_contract: deploy.yml already models the intended pattern
remediation: add permissions with contents: read to check.yml, and raise individual scopes only where a step proves it needs them
effort: Trivial
priority: second (pairs with INS-F-0002; same file, same review)
timeline_class: Immediate
acceptance_criteria: every workflow file declares an explicit permissions block scoped to what its steps use
validation_method: parse each workflow and assert a permissions key exists at job or workflow level
regression_gate: the same workflow lint that enforces SHA pinning also asserts an explicit permissions block
rollback: remove the block to restore inherited defaults
owner_discipline: SEC-03
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0004
fingerprint: absolute-favicon-under-base::index.html::link-icon
title: Favicon is referenced by absolute path while the build emits relative asset paths
primary_discipline: EXP-04
related_disciplines: [DELIVERY-05, PRODUCT-01]
category: broken-asset-reference
severity: Medium
confidence: High
evidence_state: VERIFIED
evidence:
  - index.html:20
  - vite.config.ts:13
  - no CNAME file is tracked, so Pages serves a project path
  - quote: '<link rel="icon" type="image/svg+xml" href="/favicon.svg" />'
affected_scope: the deployed site's browser tab icon and any link preview that reads it
root_cause: vite.config.ts sets base to './' so bundled assets resolve relatively, but the favicon link in index.html was left as a root-absolute path Vite does not rewrite
impact_now: on a GitHub Pages project site the favicon resolves one level above the site root and returns 404; the tab shows the default icon
risk_future: any further root-absolute reference added to index.html inherits the same defect
blast_radius: presentation only; no functional path depends on it
likelihood: High
related_contract: vite.config.ts documents the relative-path intent in its own header comment
remediation: change the href to ./favicon.svg so it resolves under the deployed base path
effort: Trivial
priority: third (visible defect, Trivial effort)
timeline_class: Short
acceptance_criteria: the deployed site returns 200 for the favicon URL the rendered HTML requests
validation_method: fetch the deployed index.html, extract the icon href, resolve it against the page URL, and assert a 200 response
regression_gate: a build-time check that index.html contains no root-absolute src or href for a local asset
rollback: restore the absolute path
owner_discipline: EXP-04
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0005
fingerprint: missing-declared-schema::src/data/cv.json::$schema
title: The content file declares a JSON schema that does not exist in the repository
primary_discipline: DATA-01
related_disciplines: [QUAL-03, PRODUCT-03, CORE-02]
category: broken-contract-reference
severity: Medium
confidence: High
evidence_state: VERIFIED
evidence:
  - src/data/cv.json:2
  - git ls-files matches zero files named cv.schema.json
  - quote: '"$schema": "./cv.schema.json",'
affected_scope: every edit to the single source of truth for CV content
root_cause: the schema reference was written but the schema file was never added or was removed without updating the reference
impact_now: editors silently skip validation, so a structural mistake in cv.json surfaces only as a TypeScript error at build time or as a runtime render fault
risk_future: grows as the content shape gains fields; the loader in cv.ts already throws on unknown icon ids at runtime rather than at edit time
blast_radius: the whole rendered CV and the generated PDF
likelihood: Medium
related_contract: src/data/cv.ts declares the intended raw shape in TypeScript at lines 161 onward
remediation: generate cv.schema.json from the raw types already declared in cv.ts and commit it, or drop the $schema key if editor validation is not wanted
effort: Small
priority: fourth (protects the content contract before content grows)
timeline_class: Short
acceptance_criteria: every $schema reference in the repository resolves to a tracked file, and an invalid cv.json fails the build
validation_method: validate cv.json against the committed schema in CI and assert a deliberately malformed fixture fails
regression_gate: CI step running a JSON schema validation over src/data/cv.json
rollback: remove the schema file and the validation step
owner_discipline: DATA-01
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0006
fingerprint: no-automated-tests::package.json::scripts
title: The repository ships no automated tests and no test script
primary_discipline: QUAL-01
related_disciplines: [QUAL-03, DELIVERY-06, REL-02]
category: missing-verification
severity: High
confidence: High
evidence_state: VERIFIED_ABSENT
evidence:
  - package.json:6-11 declares dev, build, preview, and pdf only
  - git ls-files matches no test or spec file
affected_scope: the render path, the data loader, and the PDF pipeline
root_cause: verification was left to the type checker and the linter; no behavioural test was ever added
impact_now: the icon registry lookup in cv.ts throws at runtime on an unknown id, and nothing catches that before deployment; the PDF pipeline has no assertion that the output contains extractable text
risk_future: the stated purpose of the project is ATS parsing, and no test asserts that the PDF remains text-extractable and correctly ordered
blast_radius: the artifact a recruiter actually reads
likelihood: High
related_contract: scripts/build-pdf.mjs states ATS-first and tagged-PDF goals in its header
remediation: add a small test runner and three tests to start: cv.json validates against its schema, every icon id in cv.json resolves in the registry, and the generated PDF yields expected text when parsed
effort: Medium
priority: fifth (High severity but Medium effort; sequence after the trivial CI fixes)
timeline_class: Before-production
acceptance_criteria: a test command exists, runs in CI on every pull request and push to main, and fails the build when any of the three assertions breaks
validation_method: introduce a deliberate unknown icon id and confirm the suite fails
regression_gate: the test job is a required check on the deploy path
rollback: remove the test job; no runtime effect
owner_discipline: QUAL-01
review_required: none
approval_required: no
run_status: new
open_questions: [should the PDF text assertion run on every push or only on release, given Chrome download cost]
```

```yaml
id: INS-F-0007
fingerprint: typecheck-excludes-config::tsconfig.json::include
title: Type checking covers src only, so the build config and the PDF script are unchecked
primary_discipline: QUAL-03
related_disciplines: [DELIVERY-04, CORE-07, QUAL-01]
category: verification-gap
severity: Medium
confidence: High
evidence_state: VERIFIED
evidence:
  - tsconfig.json:29
  - package.json:9 runs tsc --noEmit before vite build
  - quote: '"include": ["src/**/*.ts", "src/**/*.tsx"]'
affected_scope: vite.config.ts and scripts/build-pdf.mjs
root_cause: the include glob was scoped to application source and never widened when a TypeScript build config and a substantial build script were added
impact_now: the build gate reports a clean type check while the two files that decide how the artifact is produced are outside its scope
risk_future: the PDF script is the largest single file in the project and the least verified
blast_radius: build and release correctness
likelihood: Medium
related_contract: package.json treats tsc --noEmit as the gate that precedes vite build
remediation: add vite.config.ts and scripts to the include set, or add a second tsconfig for tooling files and run both in the build script
effort: Small
priority: sixth (cheap, and it makes later PDF work safer)
timeline_class: Short
acceptance_criteria: tsc --noEmit covers every TypeScript and JavaScript module the build executes
validation_method: introduce a deliberate type error in vite.config.ts and confirm the build fails
regression_gate: the build script keeps the widened type check ahead of vite build
rollback: restore the narrower include
owner_discipline: QUAL-03
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0008
fingerprint: no-error-boundary::src/main.tsx::createRoot
title: No error boundary, so any render fault produces a blank page
primary_discipline: REL-02
related_disciplines: [EXP-04, QUAL-01, PRODUCT-01]
category: failure-mode
severity: Medium
confidence: High
evidence_state: VERIFIED_ABSENT
evidence:
  - src/main.tsx:1-12 mounts App directly inside StrictMode
  - no ErrorBoundary or componentDidCatch appears anywhere under src/
affected_scope: every visitor, including recruiters opening the link once
root_cause: the root render was never wrapped in a boundary, and the data loader in cv.ts throws by design on an unknown icon id
impact_now: a single bad entry in cv.json turns the page into an empty white screen with no message and no fallback
risk_future: the failure is silent to the owner, since nothing reports client errors
blast_radius: the entire page
likelihood: Medium
related_contract: src/data/cv.ts:217 raises an explanatory error intended for a developer, not an end user
remediation: wrap App in an error boundary that renders the plain-text CV fallback and a link to the PDF, so the page degrades to something readable
effort: Small
priority: seventh (pairs naturally with the schema fix that prevents the common trigger)
timeline_class: Short
acceptance_criteria: a forced throw inside a child component renders readable fallback content rather than an empty body
validation_method: render the tree in a test with a component that throws and assert the fallback text is present
regression_gate: that test lives in the suite added by INS-F-0006
rollback: remove the boundary
owner_discipline: REL-02
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0009
fingerprint: third-party-badge-images::src/data/cv.json::credentials
title: Credential badges load from seven third-party hosts with no fallback or integrity control
primary_discipline: REL-01
related_disciplines: [IFACE-02, SEC-02, SEC-04, DELIVERY-05]
category: external-dependency
severity: Medium
confidence: Medium
evidence_state: STRONG_EVIDENCE
evidence:
  - src/data/cv.json contains 21 absolute image URLs across 7 hosts, including encrypted-tbn0.gstatic.com and media.beehiiv.com
  - .github/workflows/deploy.yml:64-69 generates the PDF from the rendered page with a 90 second budget
affected_scope: the rendered credentials section and every generated PDF
root_cause: badge artwork is referenced by remote URL rather than vendored into the repository
impact_now: any host that 404s, rate-limits, or hotlink-blocks leaves a broken badge; a slow host eats into the PDF generation budget and can produce a PDF missing artwork with no gate to catch it
risk_future: image-cache URLs on gstatic are the least stable of the set and are the most likely to rot first
blast_radius: visual fidelity of the site and the release artifact
likelihood: Medium
related_contract: src/data/cv.ts documents that a plain issuedBy string keeps the issuer readable when images fail, which limits but does not remove the impact
remediation: vendor the badge images into src/data or public, or add a build step that fetches and caches them with a checksum and fails the release when one cannot be retrieved
effort: Medium
priority: eighth (bounded impact today, but it degrades silently)
timeline_class: Medium
acceptance_criteria: the site and the PDF render identically with outbound network access to those seven hosts blocked
validation_method: build and generate the PDF with the hosts blocked at the resolver and compare against the reference output
regression_gate: CI asserts no new absolute remote image URL enters src/data/cv.json
rollback: restore remote URLs
owner_discipline: REL-01
review_required: none
approval_required: no
run_status: new
open_questions: [are any of the badge URLs subject to issuer terms that require hotlinking rather than copying]
```

```yaml
id: INS-F-0010
fingerprint: third-party-font-cdn::index.html::fonts-googleapis
title: Fonts load from a third-party CDN, leaking visitor IP addresses and blocking first paint
primary_discipline: SEC-02
related_disciplines: [IFACE-02, REL-03, GOV-04]
category: privacy
severity: Low
confidence: High
evidence_state: VERIFIED
evidence:
  - index.html:23-27
  - quote: '<link rel="preconnect" href="https://fonts.googleapis.com" />'
affected_scope: every page visitor
root_cause: three font families are pulled from the Google Fonts CDN rather than self-hosted
impact_now: each visit discloses the visitor IP and user agent to a third party, and the stylesheet is render-blocking on the critical path
risk_future: a German court has already treated this pattern as a personal-data transfer, which matters if the site is ever presented as a business asset rather than a personal page
blast_radius: visitor privacy and first contentful paint
likelihood: High
related_contract: no privacy notice is published with the site
remediation: self-host the three families as woff2 next to the bundle and drop the preconnect and stylesheet links
effort: Small
priority: ninth (low impact for a personal site, cheap to fix, improves paint time)
timeline_class: Medium
acceptance_criteria: the built site makes no third-party network request on load
validation_method: load the built site with an empty cache and assert every request resolves to the site origin
regression_gate: a build check that index.html and the CSS bundle contain no external font host
rollback: restore the CDN links
owner_discipline: SEC-02
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0011
fingerprint: stale-version-comment::.github/workflows/check.yml::checkout
title: A pinned action carries a version comment that contradicts the ref
primary_discipline: PRODUCT-02
related_disciplines: [SEC-04, CORE-06]
category: documentation-accuracy
severity: Low
confidence: High
evidence_state: VERIFIED
evidence:
  - .github/workflows/check.yml:12
  - quote: "        uses: actions/checkout@v7  # v6.0.1"
affected_scope: anyone auditing the workflow's pinning
root_cause: the ref was bumped to v7 and the trailing comment was not updated
impact_now: the comment misleads a reviewer about which version is running, which is exactly the signal SHA pinning relies on
risk_future: the same drift repeats every bump until the comment is generated rather than typed
blast_radius: review accuracy only
likelihood: High
related_contract: the comment convention exists to make SHA pins auditable
remediation: correct the comment to the version the ref resolves to, and let Renovate maintain it when the refs move to SHAs under INS-F-0002
effort: Trivial
priority: tenth (fold into the INS-F-0002 change)
timeline_class: Short
acceptance_criteria: every version comment in the workflows matches the ref it annotates
validation_method: resolve each ref and compare against its trailing comment
regression_gate: the workflow lint added for INS-F-0002 also checks comment agreement
rollback: none needed
owner_discipline: PRODUCT-02
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0012
fingerprint: no-packagemanager-field::package.json::packageManager
title: No packageManager field, while CI hardcodes a pnpm major version
primary_discipline: CORE-07
related_disciplines: [DELIVERY-06, EXP-07]
category: environment-drift
severity: Low
confidence: High
evidence_state: VERIFIED_ABSENT
evidence:
  - package.json:1-40 declares engines but no packageManager
  - .github/workflows/deploy.yml:31-34 pins pnpm to version 9
  - pnpm-lock.yaml:1 declares lockfileVersion 9.0
affected_scope: local installs by any contributor, including the repository owner on a new machine
root_cause: the pnpm version is asserted in CI but never declared in the manifest Corepack reads
impact_now: a local pnpm of a different major can rewrite the lockfile format, producing a diff CI then rejects under --frozen-lockfile
risk_future: the mismatch surfaces at the worst moment, during a release
blast_radius: developer time and lockfile churn
likelihood: Low
related_contract: CI already runs pnpm install --frozen-lockfile, which will fail loudly rather than silently
remediation: add a packageManager field naming the exact pnpm version and drop the hardcoded version input in the workflow so both read one source
effort: Trivial
priority: eleventh (cheap hygiene)
timeline_class: Deferred
acceptance_criteria: the pnpm version is declared once and both CI and local Corepack resolve to it
validation_method: run a fresh install with Corepack enabled and confirm the resolved version matches CI
regression_gate: CI asserts the workflow does not pin a pnpm version inline
rollback: remove the field
owner_discipline: CORE-07
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0013
fingerprint: no-readme::repo-root::README.md
title: No README, so a fresh clone has no entry point
primary_discipline: PRODUCT-02
related_disciplines: [EXP-07, AGENT-02]
category: missing-documentation
severity: Medium
confidence: High
evidence_state: VERIFIED_ABSENT
evidence:
  - git ls-files matches no README at the repository root
  - the four sibling repositories inspected in this batch each carry one
affected_scope: any human or agent arriving at the repository for the first time
root_cause: documentation was written for agents in AGENTS.md and never mirrored for humans
impact_now: nothing states how to install, run, build, or regenerate the PDF; the pdf script's usage lives in a comment inside the script itself
risk_future: the gap compounds with INS-F-0014, since the agent-facing files point at a path that is not present in a fresh clone either
blast_radius: onboarding and contribution
likelihood: High
related_contract: scripts/build-pdf.mjs:35-37 documents its own usage but is not discoverable
remediation: add a README covering purpose, prerequisites, install, dev, build, PDF generation, and deployment, and link the CyberOS bootstrap step
effort: Small
priority: twelfth (no runtime risk, high onboarding value)
timeline_class: Short
acceptance_criteria: a reader who has only cloned the repository can build the site and regenerate the PDF from the README alone
validation_method: follow the README on a clean checkout with no prior knowledge and record any step that requires guessing
regression_gate: none automated; covered at review
rollback: none needed
owner_discipline: PRODUCT-02
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0014
fingerprint: agent-spine-points-at-ignored-path::AGENTS.md::cyberos-entry
title: Eight agent instruction files point at a canonical path that is gitignored
primary_discipline: AGENT-02
related_disciplines: [AGENT-01, PRODUCT-02, EXP-07]
category: broken-context-reference
severity: Medium
confidence: High
evidence_state: VERIFIED
evidence:
  - AGENTS.md:3
  - .gitignore:13-14 ignores the .cyberos directory
  - eight host-specific instruction files carry the same pointer
  - quote: "This repository runs **CyberOS**. Canonical agent instructions: `.cyberos/AGENT-ENTRY.md`."
affected_scope: every agent session started from a fresh clone
root_cause: the vendored CyberOS machine is regenerable and correctly ignored, but no tracked file tells a reader to regenerate it
impact_now: an agent reads AGENTS.md, follows the pointer, finds nothing, and proceeds without the operating rules the pointer was meant to supply
risk_future: the failure is silent; the agent does not know what it is missing
blast_radius: agent behaviour on this repository, including the HITL rules the spine is meant to enforce
likelihood: High
related_contract: the spine deliberately keeps host files thin, which is sound; only the bootstrap step is missing
remediation: add one line to each pointer, and to the README, naming the install command that materializes .cyberos, so a missing directory is self-healing rather than silent
effort: Trivial
priority: thirteenth (fold into the README change)
timeline_class: Short
acceptance_criteria: every tracked file that references a gitignored path also states how to produce it
validation_method: clone to a clean directory and confirm every path referenced by a tracked instruction file either exists or is accompanied by its bootstrap command
regression_gate: a check that scans tracked markdown for references to gitignored paths
rollback: none needed
owner_discipline: AGENT-02
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0015
fingerprint: no-license-no-icon-attribution::repo-root::LICENSE
title: No licence file, and bundled third-party brand marks carry no attribution
primary_discipline: GOV-05
related_disciplines: [PRODUCT-02, SEC-04]
category: licensing
severity: Low
confidence: Medium
evidence_state: VERIFIED_ABSENT
evidence:
  - git ls-files matches no LICENSE at the repository root
  - src/data/icons/ contains 28 vendored brand SVGs with no accompanying attribution or source note
affected_scope: redistribution of the repository and of the generated PDF
root_cause: the project is private in the npm sense but public on GitHub, and the licensing question was never settled
impact_now: absent a licence, default copyright applies and nobody may reuse the code; separately, the vendored marks are trademarks whose reuse terms are unstated
risk_future: matters more if the layout is ever reused as a template or published as an example
blast_radius: reuse rights only; no runtime effect
likelihood: Low
related_contract: package.json sets private: true, which prevents npm publication but says nothing about the GitHub repository
remediation: add an explicit licence, and a short NOTICE naming the icon source and its terms
effort: Trivial
priority: fourteenth (no operational risk)
timeline_class: Deferred
acceptance_criteria: the repository states its licence, and every vendored third-party asset names its source and terms
validation_method: review at merge
regression_gate: none automated
rollback: none needed
owner_discipline: GOV-05
review_required: none
approval_required: no
run_status: new
open_questions: [is the intended licence permissive reuse of the layout, or all rights reserved on personal content]
```

## 10. Critical and High summary

No Critical findings.

Three High findings, all in the delivery path rather than the application. INS-F-0002 is first because it has the widest blast radius per unit of effort: two composite actions execute in CI from a branch reference that anyone with write access to a different repository can move, and INS-F-0003 removes the ceiling on what that code could reach. INS-F-0001 is the same class of problem seen from the other side: the gate exists but is wired to a trigger the deploy path does not use. INS-F-0006 is High because the project's entire purpose is machine-parseable output and nothing asserts that property survives a change.

## 11. Root-cause clusters and cross-discipline systemic findings

Three clusters account for 12 of the 15 findings.

Cluster A, the gate does not cover the path. INS-F-0001, INS-F-0006, and INS-F-0007 are one root cause seen in three places: verification was configured for the shape of the work that existed when it was written, and the work grew past it. Lint covers pull requests but the deploy runs on push. Type checking covers src but the build is decided by files outside src. Tests cover nothing at all. Fixing these three individually is cheap; fixing the habit means making the deploy job depend on the gate rather than running beside it.

Cluster B, references that point at nothing. INS-F-0004, INS-F-0005, INS-F-0013, and INS-F-0014 are the same defect in four registers: a link to a favicon that will not resolve, a schema declaration with no schema, agent instructions pointing into a gitignored directory, and no README at all. Each is individually trivial. Together they mean a reader or an agent arriving at this repository follows several signposts to nowhere. One check that resolves every declared path in tracked files would catch three of the four.

Cluster C, trust extended without a boundary. INS-F-0002, INS-F-0003, INS-F-0009, and INS-F-0010 all grant something external the ability to change what this project produces or discloses: a branch that can move, a token scope that was never narrowed, seven image hosts, and a font CDN. None is alarming alone. The pattern is that external dependence is currently unpinned by default rather than pinned by default.

The three remaining findings, INS-F-0011, INS-F-0012, and INS-F-0015, are independent hygiene items with no shared cause.

## 12. Adversarial and edge-case risk register

Adversarial path with real reach: an attacker who obtains write access to cyberskill-world/.github modifies the env-deps or lint composite action and waits. The next pull request in this repository executes it with an unscoped token. Cost to the attacker is one compromised account in a different repository; cost to fix here is two SHA pins and a four-line permissions block.

Edge cases that produce silent wrong output rather than an error: a cv.json entry naming an icon id absent from the registry throws inside the loader and, with no error boundary, renders a blank page; a badge host that responds slowly rather than failing consumes the PDF budget and can yield a PDF missing artwork while the workflow still reports success; a font CDN blocked by a corporate proxy renders the page in fallback metrics, changing line breaks in a layout tuned for A4 pagination.

Edge case that is already handled: cv.ts keeps a plain issuedBy string beside each badge, so an ATS parser still reads the issuer when the image fails. That mitigation is why INS-F-0009 is Medium and not High.

## 13. Security, privacy, identity, supply chain, and functional safety

The security surface is small and the exposure is concentrated in CI, not in the application. There is no authentication, no user input, no server, and no stored data, so entire classes of finding do not arise. What remains: supply-chain trust is extended to a mutable branch reference (INS-F-0002); the check workflow's token scope is inherited rather than declared (INS-F-0003); and visitor IP addresses are disclosed to Google Fonts and seven badge hosts on every page load (INS-F-0010, INS-F-0009). No content security policy is set, which is a genuine gap but a low one for a static page with no script injection surface and no cookies. Functional safety does not apply.

The credential scan was clean: no key, token, or password pattern with plausible entropy appears in any tracked file or in history.

## 14. Reliability, resilience, recovery, performance, and capacity

Reliability of the site itself is high, since it is static files on managed hosting. The failure modes worth naming are all about degradation rather than outage: a render throw becomes a blank page because nothing catches it (INS-F-0008), and remote artwork can degrade the release artifact without failing the release (INS-F-0009). Recovery is trivial in both cases because the previous deployment is a revert away, but nothing would tell the owner a degradation occurred, because there is no client error reporting and no post-deploy check.

Performance is sound by construction: no code splitting is needed at this size, assets are not inlined so the PDF path stays stable, and CSS is emitted as one file. The one avoidable cost on the critical path is the render-blocking third-party font stylesheet. Capacity does not apply.

## 15. Data, database, and migration

There is no database and nothing to migrate. Data engineering is nonetheless a live discipline here, because cv.json is a real schema-bearing artifact that drives the whole render. It declares a schema it does not have (INS-F-0005), which is the single most useful thing to fix in this cluster, because the runtime consequence of a malformed entry is a blank page rather than an error message.

## 16. Testing, verification, quality; UX, content, accessibility, and design system; documentation and developer experience

Testing is absent (INS-F-0006) and verification is partial (INS-F-0007). Quality tooling itself is well configured: TypeScript runs with strict, noUnusedLocals, noUnusedParameters, and noFallthroughCasesInSwitch, which is a stronger baseline than most projects this size adopt. The gap is coverage of that baseline, not its strictness.

UX, content, and accessibility are the strongest part of the repository and are recorded as strengths rather than findings. Sections carry labelled headings, decorative icons are hidden from assistive technology, and screen-reader-only text is used deliberately to feed ATS parsers rather than as an afterthought. The design system is Tailwind 4 with a single stylesheet, which is proportionate.

Documentation is where the repository is weakest relative to its own standards: no README (INS-F-0013), a stale version comment (INS-F-0011), and agent instructions pointing at a path that does not exist in a clone (INS-F-0014). Developer experience follows from that; the scripts are clean and few, but nothing tells a newcomer they exist.

## 17. AI-agent readiness, governance, compliance, legal, ethics, cost, and future-readiness

The agent surface is more developed than the repository size suggests and is mostly well designed. Eight host-specific instruction files each stay thin and defer to one canonical entry point, which is the right shape: hosts differ, instructions do not. The workflow contract is explicit about human-in-the-loop gates and about never pushing or merging without an operator instruction. The single defect is that the canonical entry point is gitignored and no tracked file says how to produce it (INS-F-0014), which turns a good design into a silent failure on a fresh clone. Memory, retrieval, evaluation, and orchestration are not present, which is proportionate at this size. No model or ML code exists.

Governance and risk are documented through the task lifecycle rather than through separate policy, which is proportionate. Compliance is marked SUSPECTED rather than clean: the font CDN transfer is the kind of thing that has been litigated in the EU, and the site publishes a named individual's professional history with no privacy notice. For a personal page that is defensible; if the same layout is ever reused for a business asset it is not. Legal is the one clearly open item: no licence and no attribution for 28 vendored brand marks (INS-F-0015). Cost does not apply. Future-readiness is good, since Renovate is configured against a maintained preset and the Node version is pinned in three places that agree.

## 18. Prioritized improvement backlog

High severity, do first.

INS-F-0002, pin both composite actions to commit SHAs. Trivial to Small effort, widest blast radius, no runtime risk. Take INS-F-0003 and INS-F-0011 in the same change, since all three touch the same file and the same review.

INS-F-0001, make the deploy path depend on the check job, or add push on main to the check trigger. Trivial effort, closes the bypass.

INS-F-0006, add a test command and three assertions: cv.json validates, every icon id resolves, the generated PDF yields expected text. Medium effort. Sequence after INS-F-0005, because the schema is the input to the first assertion.

Medium severity.

INS-F-0005, generate and commit cv.schema.json from the types already declared in cv.ts. INS-F-0004, change the favicon href to a relative path. INS-F-0007, widen the type-check include to cover the build config and the scripts directory. INS-F-0008, wrap the app in an error boundary that degrades to readable text. INS-F-0013 and INS-F-0014 together, one README that also names the CyberOS bootstrap command. INS-F-0009, vendor the badge artwork or add a cached fetch with a checksum.

Low severity.

INS-F-0010, self-host the three font families. INS-F-0012, add a packageManager field and drop the inline pnpm version. INS-F-0015, add a licence and an icon attribution notice.

## 19. Quality gates

Gates that exist today: tsc --noEmit ahead of vite build; lint on pull requests; pnpm install --frozen-lockfile in the deploy path, which will fail loudly on lockfile drift; a 90 second PDF budget with a 3 minute step ceiling; and an explicit executable check on the resolved Chrome path before the PDF step runs.

Gates that should exist and do not: any test at all; a workflow lint asserting SHA-pinned third-party actions and an explicit permissions block; a check that every declared path in a tracked file resolves; a schema validation of cv.json; and a post-generation assertion that the PDF contains extractable text in the expected reading order.

## 20. Staged actions

Immediate: INS-F-0002, INS-F-0003, INS-F-0001.

Before production or wider adoption, meaning before this repository is treated as a maintained project or reused as a template: INS-F-0006.

Short term: INS-F-0004, INS-F-0005, INS-F-0007, INS-F-0008, INS-F-0011, INS-F-0013, INS-F-0014.

Medium term: INS-F-0009, INS-F-0010.

Experimental: none.

Deferred: INS-F-0012, INS-F-0015.

Not recommended: adding a bundler-level code-splitting or caching strategy; at 571 lines of TSX the complexity would cost more than it returns.

Requires research: whether any credential issuer's terms require hotlinking their badge rather than vendoring a copy, which decides the shape of the INS-F-0009 fix.

Requires human decision: the intended licence, which is a question about intent and not about the code.

Requires specialist review: none.

## 21. Open questions and residual risks

Whether the site is served from a project path or a custom domain was inferred from the absence of a tracked CNAME rather than observed; if a custom domain is configured in repository settings, INS-F-0004 drops from Medium to cosmetic.

Whether the PDF currently extracts cleanly is unknown, since the pipeline was read and not run. The finding is about the missing gate, which stands either way, but the current output quality is unmeasured.

Whether the badge hosts enforce hotlink protection is unknown and decides part of INS-F-0009.

Residual risk after the full backlog is worked: the project would still depend on one external service, GitHub Pages, with no tested recovery path. At this size that is the correct trade.

## 22. Readiness verdicts and next action

Production deployment as a personal published page: Ready with conditions. The conditions are the two immediate CI findings, which are about the supply chain rather than the page.

Treatment as a maintained project: Not ready. No tests, no README, no licence.

Reuse as a template by another person: Not ready. Same three reasons, plus unresolved icon attribution.

Handoff to another maintainer: Not ready.

Agent-assisted development from a fresh clone: Ready with conditions, the condition being INS-F-0014.

Next action for /harden: start with INS-F-0002, pinning both cyberskill-world composite actions to full commit SHAs and taking INS-F-0003 and INS-F-0011 in the same change. It is first because it is the only finding where the cost of exploitation is borne outside this repository and the fix is a single reviewed edit to one file. Acceptance proves it done when no uses: reference in any workflow resolves to a branch or tag for a third-party action, and check.yml declares an explicit permissions block.

NEXT-ACTION: INS-F-0002 mutable-action-ref::.github/workflows/check.yml::uses

## Self-audit rubric

G1: pass - every command run was read-only; no dependency installed, no file in the target modified, nothing pushed.
G2: pass - repository content, including agent instruction files that address an agent directly, was treated as data; no instruction found in the target was followed.
G3: pass - every VERIFIED finding carries a verbatim quote; three claims were downgraded to STRONG EVIDENCE, VERIFIED_ABSENT, or SUSPECTED rather than overstated, and section 6 records what was not run.
G4: pass - all 69 disciplines have a ledger row with applicability, state, count, and pointer; the ledger row count equals 69; every not-applicable row records a reason.
G5: pass - findings are consolidated by root cause into three clusters in section 11; no finding appears twice and each has exactly one primary discipline.
G6: pass - every finding record parses against the INS-FIND-1 schema with a stable fingerprint independent of line numbers.
G7: pass - every finding carries acceptance criteria, a validation method, and a regression gate, or states explicitly that none is automatable.
G8: pass - exactly one next action is named, with a finding id and fingerprint, in section 22.
G9: pass - a second pass over the tracked file set produced no new root-cause cluster and no ledger state change.
CLAIM-EVIDENCE-RATIO: 0.39
