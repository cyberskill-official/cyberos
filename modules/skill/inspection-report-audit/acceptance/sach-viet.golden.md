# /inspect report: cyberskill-official/sach-viet

## 1. Side-effect disclosure

None. Every command was read-only: git clone, git ls-files, git log, file reads, and text search. No dependency was installed, no build was run, no database or deployment was contacted, and nothing was written to the repository or pushed. The clone lives in a scratch directory.

## 2. Executive summary

sach-viet is a Vietnamese bookselling platform: a Next.js application with 59 route handlers over fourteen store modules, backed by Postgres, with a task trail of 59 specifications each carrying a paired audit. Its verification posture is the most interesting in either batch, because it is simultaneously the best and the most misleading.

The good part is real and unusual. The quality gate runs on every push and every pull request with no branch filter, which no other repository across ten inspections manages. It provisions an actual database, runs migrations, and then runs lint, tests, a verification step, and a build in sequence. Human acceptance is enforced mechanically: a script inspects the diff for task transitions into review or completion and fails the build when a recorded verdict is missing. And the test suite is substantial, 50 files and 942 assertions, running against the same Postgres engine production uses, isolated per test by schema rather than substituted for something lighter.

The misleading part sits directly beside it. Twenty-one scripts totalling 825 lines run as their own required step called Verify, and not one of them executes a line of application code. They assert that source files exist and that source text contains particular substrings. The identity check passes if `auth-core.mjs` contains the string HttpOnly and fails if a correct refactor stops spelling it that way.

Separately, the build that ships is not the build that CI proves. Production is produced by a shell command in `vercel.json` that copies every entry of `app/web` over the repository root, deleting each root entry first, then installs. CI never runs that command; it builds inside `app/web` directly.

Findings: 9 total, 0 Critical, 3 High, 4 Medium, 2 Low. Five strengths recorded.

The one exact next action is at section 22.

## 3. Inspection metadata and baseline

Target: https://github.com/cyberskill-official/sach-viet, default branch main, head 160f153, 88 commits, last commit 2026-07-26. Working tree 6.9M excluding .git. 1,149 tracked files, of which 899 are documentation and 227 are the application.

The file count is not the code surface. The application is `app/web`: 111 source files, 50 test files, 36 scripts, three migrations. Languages by line count across the repository: Markdown 18,482 across 620 files; ES modules 13,464 across 120 files, which is the application's store layer plus the script tree; YAML 5,073 across 198 files, almost all task artifacts; JSON 8,332; TypeScript 1,744 across 61 files; TSX 857 across 19.

Stack: Next.js 16.2.11 pinned exactly, React 19.2.4, Postgres reached through the `pg` driver behind a synckit worker that presents a synchronous interface, Tailwind 4, an internal design package. Node pinned to 24.x in both manifests. One workflow. Deployment targets both Vercel and a container platform.

Inspection was single-pass and reached saturation without continuation; see section 5.

## 4. Coverage ledger

All 69 disciplines, in stable id order. 58 applicable, 11 not applicable with a recorded reason.

| Discipline | Cluster | Applicability | Evidence state | Finding count | Evidence pointer | Related |
|---|---|---|---|---|---|---|
| CORE-01 Requirements engineering | CORE | APPLICABLE | VERIFIED | 0 | docs/tasks/BACKLOG.md, 59 task specs with paired audits | AGENT-10 |
| CORE-02 Domain engineering | CORE | APPLICABLE | VERIFIED | 0 | app/web/src/lib/ store factories, app/web/migrations | DATA-02 |
| CORE-03 Systems engineering | CORE | APPLICABLE | VERIFIED | 0 | app/docker-compose.yml, app/web/Dockerfile, vercel.json | DELIVERY-05 |
| CORE-04 Architecture engineering | CORE | APPLICABLE | VERIFIED | 0 | app/web/src/lib 14 store modules behind 59 route handlers | EXP-05 |
| CORE-05 Software engineering | CORE | APPLICABLE | VERIFIED | 0 | app/web/src 111 files, 1,744 lines TypeScript plus module stores | QUAL-01 |
| CORE-06 Repository engineering | CORE | APPLICABLE | VERIFIED | 0 | 88 commits, conventional prefixes, one workflow | DELIVERY-06 |
| CORE-07 Configuration engineering | CORE | APPLICABLE | VERIFIED | 1 | package.json:7-11 duplicates app/web/package.json:33-38 | DELIVERY-05 |
| CORE-08 Concurrency engineering | CORE | APPLICABLE | VERIFIED | 0 | app/web/src/lib/db.mjs synckit worker bridges async pg to a sync API | REL-02 |
| CORE-09 Distributed systems engineering | CORE | APPLICABLE | VERIFIED_ABSENT | 0 | single deployment plus one database; the worker is in-process | CORE-08 |
| PRODUCT-01 Product engineering | PRODUCT | APPLICABLE | VERIFIED | 0 | README.md, app/web/src/app 16 route groups | PRODUCT-03 |
| PRODUCT-02 Documentation engineering | PRODUCT | APPLICABLE | VERIFIED | 1 | app/web/OPERATIONS.md, README.md, 620 markdown files | EXP-07 |
| PRODUCT-03 Content engineering | PRODUCT | APPLICABLE | VERIFIED | 0 | app/web/src/app Vietnamese-language surfaces | PRODUCT-04 |
| PRODUCT-04 Internationalization and localization engineering | PRODUCT | APPLICABLE | VERIFIED | 0 | app/web/src/app/layout.tsx declares the Vietnamese locale | PRODUCT-03 |
| DATA-01 Data engineering | DATA | APPLICABLE | VERIFIED | 0 | app/web/src/lib/db.mjs, db-worker.mjs | DATA-02 |
| DATA-02 Database engineering | DATA | APPLICABLE | VERIFIED | 0 | app/web/src/lib/db.mjs:150-178, app/web/migrations | DATA-03, QUAL-01 |
| DATA-03 Migration engineering | DATA | APPLICABLE | VERIFIED | 0 | app/web/migrations three files applied by scripts/migrate.mjs in CI | DATA-02 |
| IFACE-01 API engineering | IFACE | APPLICABLE | VERIFIED | 0 | app/web/src/app/api 59 route handlers | SEC-03, EXP-05 |
| IFACE-02 Integration engineering | IFACE | APPLICABLE | VERIFIED | 0 | app/web/src/lib email and Zalo integration modules | SEC-04 |
| IFACE-03 Event and messaging engineering | IFACE | APPLICABLE | VERIFIED | 0 | app/web/src/lib/live-notifications-core.mjs | IFACE-01 |
| SEC-01 Security engineering | SEC | APPLICABLE | VERIFIED | 1 | app/web/src/lib/auth-core.mjs cookie flags, scripts/hash-password.mjs | SEC-03 |
| SEC-02 Privacy engineering | SEC | APPLICABLE | SUSPECTED | 0 | app/web/src/lib stores customer, order, and support records | GOV-04 |
| SEC-03 Identity and access engineering | SEC | APPLICABLE | VERIFIED | 0 | app/web/src/lib/auth-core.mjs:133, src/lib/access.mjs, src/proxy.ts | SEC-01, IFACE-01 |
| SEC-04 Supply-chain engineering | SEC | APPLICABLE | VERIFIED | 1 | .github/workflows/ci.yml:40-52, app/web/package.json overrides | GOV-08, DELIVERY-06 |
| SEC-05 Functional safety engineering | SEC | NOT APPLICABLE (conditional row; no safety-critical function) | NOT APPLICABLE | 0 | NONE | |
| REL-01 Reliability engineering | REL | APPLICABLE | STRONG EVIDENCE | 0 | app/web/scripts/smoke-production.mjs, smoke-docker.mjs | DELIVERY-05 |
| REL-02 Resilience engineering | REL | APPLICABLE | VERIFIED | 1 | app/web/src/lib/db.mjs:16-28 resolves the worker by trying candidate paths | CORE-08 |
| REL-03 Performance engineering | REL | APPLICABLE | SUSPECTED | 0 | the synckit bridge serialises database calls through one worker | CORE-08 |
| REL-04 Capacity engineering | REL | APPLICABLE | VERIFIED_ABSENT | 0 | no connection or worker pool sizing is recorded | REL-03 |
| REL-05 Site reliability engineering | REL | NOT APPLICABLE (no on-call rotation or published service level objective) | NOT APPLICABLE | 0 | NONE | |
| REL-06 Observability engineering | REL | APPLICABLE | VERIFIED | 0 | app/web/scripts emit structured JSON events per verification run | REL-01 |
| REL-07 Incident and problem management engineering | REL | NOT APPLICABLE (no incident or problem management process to inspect) | NOT APPLICABLE | 0 | NONE | |
| DELIVERY-01 Platform engineering | DELIVERY | NOT APPLICABLE (no internal platform offered to other teams) | NOT APPLICABLE | 0 | NONE | |
| DELIVERY-02 Infrastructure engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | app/docker-compose.yml, app/web/Dockerfile, captain-definition | DELIVERY-03 |
| DELIVERY-03 Cloud engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | vercel.json, app/web/vercel.json | DELIVERY-05 |
| DELIVERY-04 Build engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | app/web/package.json:6-11 quality chain | DELIVERY-06 |
| DELIVERY-05 Release engineering | DELIVERY | APPLICABLE | VERIFIED | 1 | vercel.json:4 materializes app/web at the build root | CORE-07, DELIVERY-03 |
| DELIVERY-06 CI/CD engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | .github/workflows/ci.yml:1-95 | QUAL-01, GOV-02 |
| DELIVERY-07 Embedded and firmware engineering | DELIVERY | NOT APPLICABLE (conditional row; no embedded or firmware target) | NOT APPLICABLE | 0 | NONE | |
| QUAL-01 Test engineering | QUAL | APPLICABLE | VERIFIED | 1 | app/web/tests 50 files, 4,703 lines, 942 assertions | QUAL-03, DATA-02 |
| QUAL-02 Quality engineering | QUAL | APPLICABLE | VERIFIED | 0 | app/web/eslint.config.mjs run in CI | QUAL-01 |
| QUAL-03 Verification and validation engineering | QUAL | APPLICABLE | VERIFIED | 1 | app/web/scripts 21 verification scripts, 825 lines | QUAL-01, DELIVERY-06 |
| EXP-01 User experience engineering | EXP | APPLICABLE | STRONG EVIDENCE | 0 | app/web/src/app 19 TSX surfaces | EXP-04 |
| EXP-02 Accessibility engineering | EXP | APPLICABLE | NOT FOUND | 0 | four of nineteen TSX files carry any aria attribute; no accessibility test | EXP-01 |
| EXP-03 Design system engineering | EXP | APPLICABLE | VERIFIED | 0 | @cyberskill/design dependency, Tailwind 4 with a postcss override | EXP-04 |
| EXP-04 Frontend engineering | EXP | APPLICABLE | VERIFIED | 0 | app/web/src/app, 857 lines TSX | EXP-03 |
| EXP-05 Backend engineering | EXP | APPLICABLE | VERIFIED | 0 | app/web/src/app/api 59 handlers over 14 store modules | IFACE-01 |
| EXP-06 Client and application engineering | EXP | NOT APPLICABLE (no native mobile or desktop client) | NOT APPLICABLE | 0 | NONE | |
| EXP-07 Developer experience engineering | EXP | APPLICABLE | VERIFIED | 0 | app/web/OPERATIONS.md, README.md, AGENTS.md, 15 operational scripts | PRODUCT-02 |
| EXP-08 Package and library engineering | EXP | NOT APPLICABLE (both manifests are private; nothing is published as a library) | NOT APPLICABLE | 0 | NONE | |
| AGENT-01 Prompt engineering | AGENT | APPLICABLE | VERIFIED | 0 | AGENTS.md, CLAUDE.md, GEMINI.md, .cursorrules and five sibling host files | AGENT-02 |
| AGENT-02 Context engineering | AGENT | APPLICABLE | VERIFIED | 0 | AGENTS.md pointer to the vendored entry point | AGENT-01 |
| AGENT-03 Memory engineering | AGENT | APPLICABLE | VERIFIED_ABSENT | 0 | the vendored store is gitignored | AGENT-02 |
| AGENT-04 Retrieval engineering | AGENT | NOT APPLICABLE (no retrieval or index layer) | NOT APPLICABLE | 0 | NONE | |
| AGENT-05 Harness engineering | AGENT | APPLICABLE | VERIFIED | 0 | .mcp.json | AGENT-09 |
| AGENT-06 Evaluation engineering | AGENT | NOT APPLICABLE (no agent evaluation suite in this repository) | NOT APPLICABLE | 0 | NONE | |
| AGENT-07 Agentic engineering | AGENT | APPLICABLE | VERIFIED | 0 | docs/tasks 59 specs each with a paired audit | AGENT-10 |
| AGENT-08 Orchestration engineering | AGENT | NOT APPLICABLE (single agent entry point; no multi-agent orchestration) | NOT APPLICABLE | 0 | NONE | |
| AGENT-09 Tool engineering | AGENT | APPLICABLE | VERIFIED | 0 | .mcp.json | AGENT-05 |
| AGENT-10 Workflow engineering | AGENT | APPLICABLE | VERIFIED | 0 | docs/tasks/BACKLOG.md declares the eight-state lifecycle | AGENT-07, AGENT-11 |
| AGENT-11 Human-in-the-loop engineering | AGENT | APPLICABLE | VERIFIED | 0 | app/web/scripts/require-hitl-verdict.mjs enforced in CI | AGENT-10, GOV-02 |
| AIML-01 Model and ML engineering | AIML | APPLICABLE | VERIFIED_ABSENT | 0 | no model is trained, served, or called | PRODUCT-03 |
| GOV-01 Decision engineering | GOV | APPLICABLE | VERIFIED | 0 | CHANGELOG.md, docs/governance, docs/status 63 files | CORE-01 |
| GOV-02 Governance engineering | GOV | APPLICABLE | VERIFIED | 0 | app/web/scripts/require-hitl-verdict.mjs:9-13 | AGENT-11, DELIVERY-06 |
| GOV-03 Risk engineering | GOV | APPLICABLE | STRONG EVIDENCE | 0 | docs/gaps nine files, docs/ops five files | GOV-02 |
| GOV-04 Compliance engineering | GOV | APPLICABLE | SUSPECTED | 0 | commerce and support records are retained with no policy recorded | SEC-02 |
| GOV-05 Legal, licensing, and intellectual-property engineering | GOV | APPLICABLE | VERIFIED | 0 | LICENSE present at the repository root | PRODUCT-02 |
| GOV-06 Ethics and responsible-technology engineering | GOV | NOT APPLICABLE (conditional row; no consequential decision automation affecting third parties) | NOT APPLICABLE | 0 | NONE | |
| GOV-07 Cost, FinOps, and economic engineering | GOV | APPLICABLE | VERIFIED_ABSENT | 0 | no spend ceiling recorded for hosting or database | DELIVERY-03 |
| GOV-08 Evolution, sustainability, maintenance, and disposal engineering | GOV | APPLICABLE | VERIFIED_ABSENT | 1 | no dependency automation configuration exists | SEC-04 |

## 5. Scope, methodology, and commands run

Scope was the full repository at head 160f153. Method was Phase 0 baseline, Phase 1 discovery and mapping, Phase 3 static reading of both manifests, both deployment configurations, the workflow, the governance gate script, the database bridge, the authentication store's entry point, one verification script in full and all 21 by assertion style, and the test suite by import pattern and assertion count, Phase 6 cross-layer reconciliation between the declared verification story and what each layer actually checks, and Phase 7 discipline sweep.

Commands run, all read-only: git clone; git ls-files with path filters; grep and sed for content search and for building the assertion-style inventory across the script tree; cat, head, and sed -n for file reads; wc for sizes and counts.

No executable validation was performed. The three checks that would add most are running the test suite to confirm the assertion count corresponds to passing tests, running the coverage script to measure the suite's reach, and running the root install command to observe what the production build actually produces. None was run because each requires installing dependencies or provisioning a database, which is a side effect on a first pass.

## 6. Limitations and blocked validations

Two hypotheses were formed during this inspection and both were wrong. They are recorded because the corrections are load-bearing.

The first was that the tests run against SQLite while production runs Postgres, formed from test fixtures constructing stores with temporary file paths. Reading `db.mjs` showed the opposite: a path argument is converted into a schema name and the handle returned is always Postgres. What looked like a substituted backend is per-test schema isolation against the real one, and it is recorded as a strength.

The second was that the task tree was an unmanaged backlog, formed from aggregating `status:` frontmatter across all 566 markdown files and finding 345 instances of a value absent from the declared lifecycle. Scoping the aggregation to task specifications showed 59 tasks, all in terminal states; the 345 were audit-finding statuses in `audit.md` files, a different field entirely. That is also recorded as a strength.

Beyond those: the 59 route handlers were inventoried and not read individually, so authorisation coverage per route is unassessed and no finding is raised about it. The three migrations were counted, not read. Twenty of the 21 verification scripts were classified by assertion style rather than read in full; the one read completely is representative of the pattern the classification found, but the sample is one. The 50 test files were measured by import pattern and assertion count rather than read, so their assertions are counted, not judged.

## 7. System model

Purpose: a Vietnamese-language bookselling platform spanning consumer retail, business-to-business quoting and ordering, institutional buyers, vendor and publisher and author portals, employee retail, support, and notifications, with a WordPress import path for existing catalogue data.

Users: consumers, vendors, publishers, authors, institutional buyers, employees, and administrators, distinguished by role in a single identity store.

Context and boundaries: the application owns its data in Postgres. Outbound integrations are email and Zalo notification delivery. The trust boundary is the 59-handler API surface plus a proxy module that enforces access before routes are reached.

Architecture: fourteen store modules under `src/lib` expose synchronous factories; `db.mjs` bridges those to the asynchronous Postgres driver through a synckit worker so the factories can stay synchronous inside server components. That bridge is the architectural decision this repository turns on, and it is the source of INS-F-0003. Route handlers are thin over the stores. Tests construct the same store factories with isolated schemas.

Maturity: production-shaped, with an unusually complete process trail and a verification layer whose strongest and weakest components sit in the same CI step.

## 8. Strengths worth preserving

```yaml
id: INS-F-9001
fingerprint: strength-hitl-enforced-in-ci::app/web/scripts/require-hitl-verdict.mjs::gates
title: Human acceptance is enforced mechanically in continuous integration, not by convention
primary_discipline: GOV-02
evidence_state: VERIFIED
evidence:
  - app/web/scripts/require-hitl-verdict.mjs:9-13 names the two lifecycle transitions it gates
  - .github/workflows/ci.yml:60-78 runs it before installing anything, against the diff base
  - the script reconstructs a sensible base commit for a first push rather than failing
  - quote: '  ready_to_test: { name: "review", from: "reviewing", to: "ready_to_test" },'
strength: true
```

```yaml
id: INS-F-9002
fingerprint: strength-no-gate-trigger-gap::.github/workflows/ci.yml::on
title: The quality gate runs on every push and every pull request with no branch filter
primary_discipline: DELIVERY-06
evidence_state: VERIFIED
evidence:
  - .github/workflows/ci.yml:3-5
  - .github/workflows/ci.yml:6-7 sets an explicit read-only permission
  - .github/workflows/ci.yml:80-95 runs install, lint, migrate, test, verify, and build in sequence
  - quote: "  push:"
strength: true
```

```yaml
id: INS-F-9003
fingerprint: strength-tests-share-production-backend::app/web/src/lib/db.mjs::schema-isolation
title: Tests run against the same database engine as production, isolated by schema rather than substituted
primary_discipline: DATA-02
evidence_state: VERIFIED
evidence:
  - app/web/src/lib/db.mjs:168-177 derives a schema from a path argument and returns a Postgres handle either way
  - .github/workflows/ci.yml:20-33 provisions a real database service for the run
  - app/web/tests construct stores with temporary paths that become schema names
  - quote: "  const db = new PgDatabase(id);"
strength: true
```

```yaml
id: INS-F-9004
fingerprint: strength-real-test-suite::app/web/tests::assertions
title: The test suite exercises application code directly with 942 assertions
primary_discipline: QUAL-01
evidence_state: VERIFIED
evidence:
  - app/web/tests holds 50 files totalling 4,703 lines
  - 28 of the 50 import modules from the source tree
  - fixtures create and tear down isolated stores per test
  - quote: 'import { createAdminCommerceStore, getAdminCommerceDashboard, listVendorApplications, resolveVendorApplication, submitVendorApplication } from "../src/lib/admin-commerce-core.mjs";'
strength: true
```

```yaml
id: INS-F-9005
fingerprint: strength-complete-task-trail::docs/tasks::specs-and-audits
title: Every task carries a paired audit and a terminal state
primary_discipline: AGENT-10
evidence_state: VERIFIED
evidence:
  - docs/tasks holds 59 task specifications and 59 matching audits
  - every specification's status is one of done, on_hold, or closed, with none in flight
  - docs/tasks/BACKLOG.md declares the eight-state lifecycle and names the two human gates
  - quote: "Lifecycle: draft -> ready_to_implement -> implementing -> ready_to_review -> reviewing -> ready_to_test -> testing -> done. Off-ramps: on_hold, closed."
strength: true
```

## 9. Complete findings register

```yaml
id: INS-F-0001
fingerprint: verification-script set-never-executes-code::app/web/scripts::verify
title: Twenty-one verification scripts gate every build and none of them executes application code
primary_discipline: QUAL-03
related_disciplines: [QUAL-01, DELIVERY-06, SEC-01]
category: false-confidence
severity: High
confidence: High
evidence_state: VERIFIED
evidence:
  - app/web/scripts holds 21 files matching verify-*.mjs, 825 lines in total
  - none of the 21 imports from ../src or performs a dynamic import of application code
  - app/web/package.json:9 chains all 21 into one script, run as its own CI step
  - app/web/scripts/verify-identity.mjs:19-24 asserts on substrings in source text
  - quote: '  if (!authCore.includes("HttpOnly") || !authCore.includes("SameSite=Lax")) throw new Error("Identity cookies must be httpOnly and same-site.");'
affected_scope: every pull request and every push, as a required step alongside lint, test, and build
root_cause: the script set was written to assert that each feature slice had landed, and file existence plus substring presence was the cheapest available proxy for that
impact_now: the checks pass on text rather than behaviour, so a comment reading no HttpOnly here satisfies the identity check while a correct refactor to a cookie helper that sets the flag through an options object fails it; the step reads in CI as a verification pass beside the real test suite
risk_future: the substring checks are coupled to the current spelling of the implementation, so they penalise refactoring in exactly the modules they claim to protect
blast_radius: confidence in the CI result, not the running system
likelihood: High
related_contract: app/web/tests holds 50 files with 942 assertions that do execute the code, so the behaviour is covered; the script set adds a second signal that looks equivalent and is not
remediation: convert each verification script into an assertion against the module it names, importing it as the test suite already does, or demote the script set to a scaffolding checklist that runs outside the quality chain and is labelled as such
effort: Medium
priority: first (High; it is the largest single block of verification in the repository and it verifies text)
timeline_class: Before-production
acceptance_criteria: every script in the quality chain either executes the code it makes a claim about, or states in its own output that it is a structural check and not a behavioural one
validation_method: insert a comment containing the asserted substring into a module whose implementation has been removed, and confirm the check now fails rather than passes
regression_gate: a CI step asserting no file under scripts matching verify-*.mjs makes a claim about behaviour without importing the module
rollback: restore the text checks
owner_discipline: QUAL-03
review_required: none
approval_required: no
run_status: new
open_questions: [were the scripts intended as per-task completion gates rather than as verification, which would make relabelling the right fix rather than rewriting]
```

```yaml
id: INS-F-0002
fingerprint: production-build-path-untested::vercel.json::installCommand
title: Production is built by a destructive file-copy that continuous integration never exercises
primary_discipline: DELIVERY-05
related_disciplines: [CORE-07, DELIVERY-06, DELIVERY-03]
category: untested-build-path
severity: High
confidence: High
evidence_state: VERIFIED
evidence:
  - vercel.json:4 copies every entry of app/web over the repository root, deleting the root entry first, then installs
  - package.json:3 records that the root manifest exists only so the platform detects the framework
  - no lockfile exists at the repository root; the one that governs the build is copied in by that command
  - .github/workflows/ci.yml:44-46 sets the working directory to app/web and installs there, so the copy never runs
  - quote: "    \"installCommand\": \"sh -c 'set -e; for item in app/web/* app/web/.[!.]*; do [ -e \\\"$item\\\" ] || continue; base=$(basename \\\"$item\\\"); case \\\"$base\\\" in .|..|node_modules|.next) continue;; esac; rm -rf \\\"./$base\\\"; cp -a \\\"$item\\\" \\\"./$base\\\"; done; npm ci'\","
affected_scope: every production and preview deployment
root_cause: the platform's root-directory setting was left unset, so the repository works around it by materialising the application at the build root during install
impact_now: the build that CI proves green is a different build from the one that ships; CI builds inside app/web with its own manifest and lockfile, while production first rewrites the root tree and then builds there, and nothing tests that rewrite
risk_future: the command deletes any root entry whose name matches an app/web entry, so adding a root-level file that also exists in app/web silently changes what deploys
blast_radius: every deployment
likelihood: Medium
related_contract: app/web/vercel.json is the straightforward configuration this repository would use if the root directory were set
remediation: set the platform's root directory to app/web and delete the root stub manifest and the copy command, or add a CI job that runs the same install command from the repository root and builds from the result
effort: Small
priority: second (High; the fix is a platform setting, and until then the deployed build is unverified)
timeline_class: Immediate
acceptance_criteria: the command that produces a production build is executed and its output built in continuous integration
validation_method: run the root install command in a clean checkout, then build, and compare the result with the CI build from app/web
regression_gate: a CI job that builds through the production path on every pull request
rollback: restore the root stub and the copy command
owner_discipline: DELIVERY-05
review_required: none
approval_required: no
run_status: new
open_questions: [is the root directory setting unavailable on the current plan, which would make the CI job the answer rather than the platform setting]
```

```yaml
id: INS-F-0003
fingerprint: worker-resolved-by-path-guessing::app/web/src/lib/db.mjs::resolveWorker
title: The database worker is located at runtime by trying a list of candidate paths
primary_discipline: REL-02
related_disciplines: [CORE-08, DATA-01, DELIVERY-04]
category: fragile-resolution
severity: High
confidence: Medium
evidence_state: VERIFIED
evidence:
  - app/web/src/lib/db.mjs:16-28 builds a candidate list and probes the filesystem
  - the candidates include a path inside the framework's build output directory
  - the comment records that the direct form breaks during page collection
  - every database call in the application goes through this bridge
  - quote: '    join(process.cwd(), ".next/server/src/lib/db-worker.mjs"),'
affected_scope: every request that touches the database, which is every authenticated page and all 59 route handlers
root_cause: the module-relative form of worker resolution broke during the framework's build-time page collection, and the workaround became a runtime search across build layouts
impact_now: the resolution depends on the framework's internal output layout, which is not a published contract; a change to it fails at request time rather than at build time, and the failure surfaces as a database error rather than as a missing file
risk_future: the framework is pinned to an exact version in both manifests, which contains the risk today and turns every upgrade into a deployment-time gamble
blast_radius: all data access
likelihood: Medium
related_contract: app/web/package.json pins the framework exactly, which is the only thing currently holding the layout stable
remediation: resolve the worker through a build-time constant the bundler can see, or fail loudly at startup when no candidate resolves rather than at first query
effort: Medium
priority: third (High; it is the single point every request passes through)
timeline_class: Before-production
acceptance_criteria: a missing worker is detected at startup with a clear error, and resolution does not depend on the framework's output layout
validation_method: build, remove the worker from every candidate location, and confirm the failure occurs at startup with a named error
regression_gate: a smoke check that the worker resolves in the built artifact, run after build in CI
rollback: restore the candidate list
owner_discipline: REL-02
review_required: none
approval_required: no
run_status: new
open_questions: [does the framework offer a supported way to ship a worker file alongside server output, which would remove the search entirely]
```

```yaml
id: INS-F-0004
fingerprint: duplicated-framework-versions::package.json::dependencies
title: Three framework versions are declared in two manifests and kept in agreement by hand
primary_discipline: CORE-07
related_disciplines: [DELIVERY-05, SEC-04]
category: duplication
severity: Medium
confidence: High
evidence_state: VERIFIED
evidence:
  - package.json:7-11 declares three dependencies at exact versions
  - app/web/package.json:33-38 declares the same three at the same versions
  - package.json:3 records that the root manifest exists only for framework detection
  - the root manifest has no lockfile, so its declarations are never resolved
  - quote: '  "description": "Repo-root stub so Vercel Git (rootDirectory unset) detects Next.js. Real app lives in app/web; vercel.json installCommand materializes it at the build root.",'
affected_scope: framework, runtime, and renderer versions
root_cause: the detection stub needs plausible dependencies to be recognised, and the simplest way to make it plausible was to copy the real ones
impact_now: nothing today, because all three agree; the cost is that an upgrade in app/web leaves a stale root manifest that is never installed and never checked, so the two drift silently
risk_future: the drift is invisible because the root declarations are overwritten during the production install before they are ever resolved
blast_radius: reader comprehension, and framework detection if the stub ever becomes implausible
likelihood: Medium
related_contract: the same root-directory setting that fixes INS-F-0002 removes the need for this stub entirely
remediation: remove the stub as part of the INS-F-0002 fix, or add a check that the three declarations match app/web
effort: Trivial
priority: fourth (fold into INS-F-0002)
timeline_class: Short
acceptance_criteria: no dependency version is declared in two places without an automated check that they agree
validation_method: change one version in app/web and confirm the check fails
regression_gate: a CI assertion comparing the two manifests, until the stub is removed
rollback: restore the duplicated declarations
owner_discipline: CORE-07
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0005
fingerprint: actions-tag-pinned-and-behind::.github/workflows/ci.yml::uses
title: Workflow actions are pinned to tags and two majors behind the sibling repositories
primary_discipline: SEC-04
related_disciplines: [DELIVERY-06, GOV-08]
category: supply-chain
severity: Medium
confidence: High
evidence_state: VERIFIED
evidence:
  - .github/workflows/ci.yml:40 and :44 reference two actions by major tag
  - sibling repositories in this organisation reference the same two actions at v7
  - no dependency automation exists to raise them
  - quote: "        uses: actions/checkout@v4"
affected_scope: every continuous integration run
root_cause: the workflow was written once and no mechanism exists to notice that its action versions have aged
impact_now: a tag can be moved, so what executes is not fixed; and two majors of accumulated fixes are not being picked up, including changes to how the checkout action handles credentials
risk_future: the gap widens passively, and this repository's workflow is the one that runs migrations against a database service
blast_radius: the CI runner and the token it holds, which is scoped to read
likelihood: Low
related_contract: .github/workflows/ci.yml:6-7 already sets an explicit read-only permission, which limits what a compromised action could reach
remediation: pin both actions to a commit with a version comment and configure dependency automation to raise them
effort: Trivial
priority: fifth (Medium; the read-only token limits the blast radius, so this is hygiene rather than exposure)
timeline_class: Short
acceptance_criteria: no third-party action reference resolves to a branch or a tag
validation_method: list every reference and confirm each is a commit
regression_gate: a check that fails on a tag reference
rollback: restore the tags
owner_discipline: SEC-04
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0006
fingerprint: coverage-available-not-gated::app/web/package.json::test-coverage
title: Coverage tooling is wired up and never runs, so the suite's reach is unmeasured
primary_discipline: QUAL-01
related_disciplines: [QUAL-03, DELIVERY-06]
category: measurement-gap
severity: Medium
confidence: High
evidence_state: VERIFIED
evidence:
  - app/web/package.json:8 declares a coverage script
  - .github/workflows/ci.yml:56-57 runs the plain test script instead
  - no coverage threshold or report is configured anywhere
  - quote: '    "test:coverage": "node --test --experimental-test-coverage tests/*.test.mjs",'
affected_scope: the 50-file test suite and the 111 source files it covers
root_cause: the coverage script was added for local use and never promoted into the pipeline
impact_now: the suite is substantial at 942 assertions, and how much of the source it reaches is unknown, so the gap between it and the text-based script set in INS-F-0001 cannot be measured either
risk_future: without a floor, coverage drifts down silently as routes are added faster than tests
blast_radius: knowledge of what is tested, not the tests themselves
likelihood: High
related_contract: the runtime's built-in coverage support means this costs one word in the CI step
remediation: run the coverage script in CI, record the current number as the floor, and fail below it
effort: Trivial
priority: sixth (Trivial, and it is the measurement that tells you how bad INS-F-0001 actually is)
timeline_class: Short
acceptance_criteria: CI reports coverage and fails when it drops below the recorded floor
validation_method: delete a test and confirm the run fails on the threshold
regression_gate: the threshold itself
rollback: restore the plain test step
owner_discipline: QUAL-01
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0007
fingerprint: no-dependency-automation::repo-root::renovate
title: No dependency automation across two manifests with a pinned override block
primary_discipline: GOV-08
related_disciplines: [SEC-04, CORE-07]
category: maintenance
severity: Medium
confidence: High
evidence_state: VERIFIED_ABSENT
evidence:
  - no dependency automation configuration exists at any path
  - app/web/package.json declares an override block pinning two transitive dependencies
  - no audit step exists in the workflow
  - two sibling repositories in this organisation configure automation
affected_scope: both manifests, the override block, and the workflow's action references
root_cause: automation was not configured when the repository was created
impact_now: security patches arrive only when someone looks, and the override block in particular pins two transitive dependencies that will need manual review to release
risk_future: the overrides exist because a specific version was needed; without automation nothing will notice when the constraint that required them is gone
blast_radius: patch latency across both manifests
likelihood: Medium
related_contract: the exact pinning throughout both manifests shows dependency control is intentional; only the raising mechanism is missing
remediation: configure dependency automation against a maintained preset and add an audit step to the workflow
effort: Small
priority: seventh (Medium; it compounds slowly)
timeline_class: Medium
acceptance_criteria: dependency updates are proposed automatically and a high-severity advisory fails a run
validation_method: confirm an update proposal appears for an outdated dependency
regression_gate: the audit step
rollback: remove the configuration
owner_discipline: GOV-08
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0008
fingerprint: stale-sqlite-type-annotation::app/web/src/lib/live-notifications-core.mjs::jsdoc
title: A type annotation still names the database the application migrated away from
primary_discipline: PRODUCT-02
related_disciplines: [DATA-01, CORE-05]
category: documentation-accuracy
severity: Low
confidence: High
evidence_state: VERIFIED
evidence:
  - app/web/src/lib/live-notifications-core.mjs:127 annotates the store type as the retired driver
  - app/web/src/lib/db.mjs:150-178 returns a Postgres handle regardless of the argument shape
  - no other source file references the retired driver
  - quote: ' *   store: { db: import("node:sqlite").DatabaseSync, close?: () => void, log?: Function },'
affected_scope: one annotation, and any editor or reader that trusts it
root_cause: the data layer was replaced and this annotation was not swept
impact_now: an editor resolves the wrong type for the store handle in that module, and a reader concludes the application has two backends when it has one
risk_future: it is the only remaining reference, so the sweep is one line
blast_radius: comprehension only
likelihood: High
related_contract: app/web/src/lib/db.mjs is explicit that a path argument becomes a schema name rather than a file
remediation: correct the annotation to the Postgres handle type the bridge returns
effort: Trivial
priority: eighth (Trivial)
timeline_class: Short
acceptance_criteria: no source file references the retired driver
validation_method: search the tree
regression_gate: none automated; covered at review
rollback: none needed
owner_discipline: PRODUCT-02
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0009
fingerprint: no-vulnerability-reporting-path::repo-root::SECURITY.md
title: A public commerce application publishes no way to report a vulnerability
primary_discipline: SEC-01
related_disciplines: [PRODUCT-02, GOV-03]
category: disclosure-process
severity: Low
confidence: High
evidence_state: VERIFIED_ABSENT
evidence:
  - no security policy file exists at any path
  - the repository is public and carries authentication, ordering, and payment-adjacent surfaces across 59 route handlers
  - a sibling repository in this organisation publishes one
affected_scope: anyone who finds a defect and wants to report it responsibly
root_cause: the file was never added
impact_now: a finder has no stated channel, no expected response time, and no assurance of safe harbour, so the likely outcomes are a public issue or silence
risk_future: matters more as the commerce surfaces go live
blast_radius: disclosure handling only
likelihood: Medium
related_contract: the repository already carries a licence and a changelog, so the governance surface exists to attach this to
remediation: add a security policy naming a contact, an expected response window, and what is in scope
effort: Trivial
priority: ninth (no operational risk, and it is the cheapest governance gap to close)
timeline_class: Short
acceptance_criteria: a security policy is published and names a working contact
validation_method: review at merge
regression_gate: none automated
rollback: none needed
owner_discipline: SEC-01
review_required: none
approval_required: no
run_status: new
open_questions: []
```

## 10. Critical and High summary

No Critical findings.

Three High findings, and the first two are the same problem at different layers: something that reads as verification does not verify the thing it names.

INS-F-0001 is the verification script set. Twenty-one scripts, 825 lines, a named CI step, and no execution of application code. It sits beside a real test suite, which is what makes it costly rather than merely useless: a reader seeing Verify pass alongside Test reasonably infers a second independent signal, and there is not one.

INS-F-0002 is the build. CI proves that `app/web` installs, migrates, tests, verifies, and builds. Production deletes and overwrites the repository root with `app/web`, then installs and builds there. Those are different operations and only one of them is tested.

INS-F-0003 is narrower but sits under everything: every database call resolves a worker file by probing a list of candidate paths, one of which is inside the framework's build output directory. The framework version is pinned exactly in both manifests, which is currently the only thing holding that layout stable.

## 11. Root-cause clusters and cross-discipline systemic findings

Cluster A, the proxy stood in for the thing. INS-F-0001 and INS-F-0006 are two sides of one habit. Where measuring the real property was expensive, a cheaper proxy was substituted and then treated as equivalent: substring presence stands in for behaviour, and the absence of coverage measurement means nobody can see the gap between the proxy and the real suite. The two belong together because running coverage is what would quantify how much INS-F-0001 actually costs.

Cluster B, the workaround became the architecture. INS-F-0002, INS-F-0003, and INS-F-0004 are all consequences of two platform constraints, an unset root directory and a bundler that breaks module-relative worker resolution. Each workaround is individually reasonable. Together they mean the production build path is untested, the runtime depends on an unpublished output layout, and three dependency versions are duplicated across manifests to keep a detection stub plausible. Setting the root directory removes two of the three.

Cluster C, no mechanism to notice ageing. INS-F-0005 and INS-F-0007 are one absence: nothing raises action versions, dependency versions, or the two transitive overrides. The pinning throughout is deliberate and exact, which makes the missing raising mechanism the only gap rather than a general looseness.

INS-F-0008 and INS-F-0009 are independent hygiene items.

## 12. Adversarial and edge-case risk register

There is no unauthenticated write path of the kind found elsewhere in this batch, and the credential sweep across HEAD and full history was clean.

The realistic failure here is not an attacker but a deployment. A framework upgrade changes the server output layout, the worker resolution in INS-F-0003 finds nothing, and every database-backed page fails at request time with a driver error rather than at build time with a missing file. CI would not catch it, because CI builds a different tree than production does.

The second is a silent behaviour change. Someone adds a file at the repository root whose name also exists in `app/web`. The production install deletes it before copying, so the deployed tree differs from the reviewed tree and nothing reports the difference.

The third is confidence rather than failure. A module is refactored, the substring the verification script greps for disappears, and the fix is to restore the substring rather than to check the behaviour. That path makes the script set actively harmful, and it is the most likely of the three.

Edge cases worth naming: the synckit bridge serialises all database access through one worker, so throughput per instance is bounded by that worker rather than by the connection pool, and no pool or worker sizing is recorded anywhere.

## 13. Security, privacy, identity, supply chain, and functional safety

Identity handling is sound where it was read. The session cookie carries the three flags that matter, the store includes login attempt throttling and a session secret read from the environment, and a proxy module enforces access ahead of the route handlers. Passwords are hashed through a dedicated script rather than inline. The one gap worth naming is coverage of the claim rather than the claim itself: the verification script that asserts these properties does so by grepping for their names, so the assurance in CI is weaker than the implementation.

Supply chain carries two findings, both about ageing rather than exposure: actions pinned to tags two majors behind, and no automation to raise anything. The workflow's read-only token limits what either could reach.

Privacy is marked SUSPECTED. The platform stores customer, order, and support records across a commerce surface with no retention policy recorded and no data subject path visible. For a live Vietnamese consumer platform that is a real question, and it is one this inspection cannot answer from the repository.

Functional safety does not apply. The credential scan was clean on HEAD and in full history, which across this batch is the norm and across the previous batch was not.

## 14. Reliability, resilience, recovery, performance, and capacity

Recovery is better provided for than in most repositories of this size: there are backup and restore scripts for the database, a migration runner, and two smoke scripts, one for the container path and one for production. Those are operational tools rather than gates, and none runs in CI.

Reliability is dominated by INS-F-0003. Every request that touches data passes through the worker bridge, and its resolution is the least certain thing in the system.

Performance and capacity are both marked with reservations. The bridge exists to keep store factories synchronous, which is a deliberate design choice with a throughput consequence nobody has measured or bounded. No pool size, no worker count, and no concurrency limit appears anywhere.

Observability is real but narrow: the scripts emit structured JSON events per verification run, which is useful in CI logs and is not application telemetry. There is no request tracing, error reporting, or metric export.

## 15. Data, database, and migration

The data layer is the strongest architectural decision here and the source of the report's most interesting correction. `openDatabase` accepts either a connection string or a path, and returns a Postgres handle in both cases, converting a path into a schema name. That gives the test suite genuine isolation against the same engine production runs, which is what most projects give up when they reach for a lighter database in tests.

Three migrations exist and are applied by a script that CI runs against a real service before the tests. The chain is complete and starts at the beginning, which after the previous batch is worth stating explicitly.

The one stale artifact is a type annotation still naming the retired driver, recorded as INS-F-0008.

## 16. Testing, verification, quality; UX, content, accessibility, and design system; documentation and developer experience

The test suite is real: 50 files, 4,703 lines, 942 assertions, 28 of them importing application modules directly, with fixtures that create and destroy isolated stores. Its reach is unmeasured, which is INS-F-0006.

The verification script set is not a test suite, and the report's central recommendation is to stop presenting it as one.

Accessibility is recorded as NOT FOUND rather than absent by design. Four of nineteen TSX files carry any aria attribute, and there is no accessibility test. For a consumer commerce platform with institutional and employee portals, that is a genuine gap; no finding is raised because the right scope is a design decision across sixteen route groups rather than a patch.

Localisation is verified and correct: the document locale is declared and the interface is Vietnamese throughout, which is the product's premise rather than an afterthought.

Documentation is extensive and operationally focused. An operations document, a per-application readme, and 620 markdown files including a complete task trail. Developer experience follows: fifteen operational scripts covering seeding, backup, restore, migration, and both smoke paths.

## 17. AI-agent readiness, governance, compliance, legal, ethics, cost, and future-readiness

Governance is where this repository is distinctive across both batches. Every other repository inspected treats human-in-the-loop acceptance as a documented convention. This one enforces it in CI: a script reads the diff, finds task status transitions into review acceptance or completion, and fails the build when the corresponding verdict is not recorded. It also handles the awkward case of a first push on a new branch by reconstructing a sensible base rather than demanding verdicts for all history. That is a governance control with teeth, and it is recorded as a strength.

The agent surface is otherwise the organisation's standard thin-pointer pattern across eight host files. The task trail is complete: 59 specifications, 59 audits, all terminal.

Legal is settled. A licence is present, which four of ten repositories across both batches still lack.

Compliance is SUSPECTED for the privacy reasons in section 13. Cost governance is absent, which matters little at this size. Future-readiness is the weakest cluster, for the reasons in cluster C.

## 18. Prioritized improvement backlog

High.

INS-F-0002, set the platform's root directory to the application path and delete the stub manifest and the copy command, taking INS-F-0004 in the same change. If the setting is unavailable, add a CI job that runs the production install command from the repository root and builds the result. Small effort either way, and until it is done the deployed build is unverified.

INS-F-0001, convert each verification script into an assertion against the module it names, or demote the script set out of the quality chain and label it as a structural checklist. Medium effort. Run INS-F-0006 first so the decision is informed by a coverage number rather than an impression.

INS-F-0003, resolve the worker through a build-time constant, and fail at startup rather than at first query when it cannot be found.

Medium.

INS-F-0006, run the coverage script in CI and record the current number as a floor. Trivial, and it is the measurement that sizes INS-F-0001. INS-F-0005, pin the two actions to commits. INS-F-0007, configure dependency automation and add an audit step.

Low.

INS-F-0008, correct the stale type annotation. INS-F-0009, publish a security policy.

## 19. Quality gates

Gates that exist today, and the list is longer than most: the workflow runs on every push and every pull request with no branch filter; the token is explicitly read-only; concurrent runs on a reference are cancelled; a real database service is provisioned; the human-verdict governance check runs before anything is installed; the install is locked; and lint, migration, tests, verification, and build run in sequence with a thirty-minute ceiling.

Gates that should exist and do not: a coverage floor; a build through the production install path; a startup check that the database worker resolves; an audit step for dependency advisories; and a check that no third-party action is referenced by tag.

The gate that exists and should be reclassified rather than removed is Verify, for the reasons in INS-F-0001.

## 20. Staged actions

Immediate: INS-F-0002, INS-F-0004.

Before production or wider adoption: INS-F-0001, INS-F-0003, INS-F-0006.

Short term: INS-F-0005, INS-F-0008, INS-F-0009.

Medium term: INS-F-0007.

Experimental: none.

Deferred: none.

Not recommended: replacing the synckit bridge with an asynchronous store interface. It would touch all fourteen store modules, all 59 handlers, and all 50 test files to remove a constraint that a build-time constant resolves. The bridge is a reasonable answer to a real framework limitation.

Requires research: whether the framework offers a supported way to ship a worker file alongside server output, which decides the shape of the INS-F-0003 fix.

Requires human decision: whether the verification script set was intended as a per-task completion checklist rather than as verification. If it was, relabelling it is the whole fix and INS-F-0001 drops from High to Low.

Requires specialist review: none.

## 21. Open questions and residual risks

Whether the platform's root directory setting is available on the current plan determines whether INS-F-0002 is fixed by configuration or by an additional CI job.

Whether the 21 verification scripts were meant as completion checklists rather than verification is the single question that most changes this report. The evidence is ambiguous: each emits a task identifier in its output, which suggests per-task provenance, and each is chained into a script named quality, which suggests verification.

The 59 route handlers were inventoried rather than read, so per-route authorisation coverage is unknown. The proxy module enforces access ahead of them, which is the right shape, but whether every handler is behind it was not established and is the most valuable thing a second pass would settle.

Residual risk after the full backlog is worked: throughput through the single database worker remains unbounded and unmeasured. No finding above addresses it because sizing it requires load testing rather than reading, and it is the largest unquantified risk in this repository.

## 22. Readiness verdicts and next action

Production deployment: Ready with conditions. The conditions are INS-F-0002 and INS-F-0003, and the first is a platform setting.

Trusting the CI result as a proxy for production correctness: Not ready, because the build CI proves is not the build that ships.

Trusting the Verify step as verification: Not ready, and this is a labelling decision as much as an engineering one.

Third-party contribution: Ready with conditions. The licence, the gate, the task trail, and the operations documentation are all present; the missing piece is a disclosure path.

Agent-assisted development from a fresh clone: Ready. The mechanically enforced human gate makes this the safest repository in either batch for agent work, because an agent cannot advance a task past review or completion without a recorded verdict.

Next action for /harden: start with INS-F-0002, setting the platform root directory to the application path and removing the root stub manifest and the copy command, taking INS-F-0004 in the same change since the stub exists only to satisfy the detection the setting replaces. It is first because every other finding in this report concerns code that CI does verify, while this one concerns the fact that the artifact reaching production is built by a command no gate has ever executed. Acceptance proves it done when the command that produces a production build is executed in continuous integration and its output built, and when no dependency version is declared in two manifests without a check that they agree.

NEXT-ACTION: INS-F-0002 production-build-path-untested::vercel.json::installCommand

## Self-audit rubric

G1: pass - every command run was read-only; nothing was installed, built, contacted, or pushed.
G2: pass - repository content, including eight agent instruction files and 620 markdown documents, was treated as data; no instruction found in the target was followed.
G3: pass - every VERIFIED finding carries a verbatim quote; two hypotheses were falsified during the pass and both corrections are recorded in section 6 rather than discarded, INS-F-0003 is held at Medium confidence, and four surfaces are recorded as inventoried rather than read.
G4: pass - all 69 disciplines have a ledger row with applicability, state, count, and pointer; the ledger row count equals 69; every not-applicable row records a reason.
G5: pass - findings are consolidated into three root-cause clusters in section 11; no finding appears twice and each has exactly one primary discipline.
G6: pass - every finding record parses against the INS-FIND-1 schema with a stable fingerprint independent of line numbers.
G7: pass - every finding carries acceptance criteria, a validation method, and a regression gate, or states explicitly that none is automatable.
G8: pass - exactly one next action is named, with a finding id and fingerprint, in section 22.
G9: pass - a second pass over the workflow, both deployment configurations, the script tree, and the manifest pair produced no new root-cause cluster and no ledger state change.
CLAIM-EVIDENCE-RATIO: 0.24
