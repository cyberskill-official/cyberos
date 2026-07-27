# /inspect report: zintaen/kristen-calendar

## 1. Side-effect disclosure

None against the target. Every command was read-only: git clone, git fetch --unshallow, git log, git ls-files, git show against historical revisions, file reads, and text search. No dependency was installed, no container was built, no database or hosted service was contacted, and nothing was written to the repository or pushed. The clone lives in a scratch directory.

One disclosure about the method: reading historical revisions of a tracked file is what surfaced INS-F-0001. Credential values were redacted before display and only their prefixes and formats were recorded, which is enough to classify them and not enough to use them.

## 2. Executive summary

Rotate two credentials before reading further. A database service role key and a model API key are reachable in the public history of this repository. The file that held them was emptied on the current revision, with a comment saying it once held real secrets and was committed by mistake, but emptying a tracked file does nothing to the blobs behind it. Three earlier revisions still carry the values, and a service role key bypasses row-level security entirely, so the carefully written policies in migration 0017 do not constrain whoever holds it.

Past that, kristen-calendar is a substantial and thoughtfully built system: a seven-package workspace spanning a web app, an iOS widget and intents extension in Swift, a Zalo mini app, a shared component library, a lunar arithmetic package, and an API service with consent logging, data minimisation, hashed partner keys, and a payment webhook that verifies signatures properly. Five strengths are recorded and they are all in the parts that handle user data.

The recurring problem is that the verification layer covers a fraction of what exists. Continuous integration runs six of the thirty-one test files and none of the ten covering the service that holds user data. It installs without verifying the lockfile. Lint is configured, has four packages installed, and cannot fail because there is no script and the step swallows errors anyway. And the migration chain starts at number sixteen, so the schema those policies protect cannot be built from this repository at all.

Findings: 11 total, 1 Critical, 3 High, 5 Medium, 2 Low.

The one exact next action is at section 22.

## 3. Inspection metadata and baseline

Target: https://github.com/zintaen/kristen-calendar, default branch main, head bad3d42, 33 commits, last commit 2026-07-16. Working tree 8.0M excluding .git. 421 tracked files, the largest in this batch, of which 119 are documentation.

Languages by line count: YAML 10,080 across 3 files, dominated by the lockfile; TypeScript 9,147 across 121; TSX 3,236 across 46; Markdown 1,143 across 97; JSON 1,124 across 30; JS 539 across 30; plus 13 Swift files and 12 SQL migrations.

Structure: a pnpm workspace over apps/web, packages/amlich-core, packages/content, packages/ui, services/genie-api, and zalo. Node pinned to 24.18.0 in fourteen files and again in two workflows. Package manager pinned exactly in the root manifest. Two workflows, Docker Compose for development and production, a Caddy configuration for deployment, and a Supabase-backed API service.

Product: a Vietnamese lunar calendar and reminder application, with the lunar arithmetic computed on device and reimplemented in Swift for the iOS widget.

Inspection was single-pass and reached saturation without continuation; see section 5.

## 4. Coverage ledger

All 69 disciplines, in stable id order. 60 applicable, 9 not applicable with a recorded reason. This is the broadest applicable surface in the batch, and the first repository where localisation, native client, and internal package publication all apply at once.

| Discipline | Cluster | Applicability | Evidence state | Finding count | Evidence pointer | Related |
|---|---|---|---|---|---|---|
| CORE-01 Requirements engineering | CORE | APPLICABLE | VERIFIED | 0 | docs/ 119 files including PRD, SRS, and readiness reviews | AGENT-10 |
| CORE-02 Domain engineering | CORE | APPLICABLE | VERIFIED | 0 | packages/amlich-core, services/genie-api/supabase/migrations/ | DATA-02 |
| CORE-03 Systems engineering | CORE | APPLICABLE | VERIFIED | 0 | pnpm-workspace.yaml, docker-compose.yml, deploy/Caddyfile | DELIVERY-02 |
| CORE-04 Architecture engineering | CORE | APPLICABLE | VERIFIED | 0 | apps/web, packages/*, services/genie-api, zalo | EXP-04, EXP-05 |
| CORE-05 Software engineering | CORE | APPLICABLE | VERIFIED | 0 | 9,147 lines of TypeScript across 121 files | QUAL-01 |
| CORE-06 Repository engineering | CORE | APPLICABLE | VERIFIED | 0 | 33 commits, conventional prefixes, docs/tasks | DELIVERY-06 |
| CORE-07 Configuration engineering | CORE | APPLICABLE | VERIFIED | 1 | fourteen Node version files across seven packages | DELIVERY-06, EXP-07 |
| CORE-08 Concurrency engineering | CORE | APPLICABLE | SUSPECTED | 0 | services/genie-api/lib/zns-scheduler.ts, rate-limiter.ts | REL-02 |
| CORE-09 Distributed systems engineering | CORE | APPLICABLE | VERIFIED | 0 | apps/web, services/genie-api, zalo, and iOS widgets share one data layer | IFACE-02, EXP-06 |
| PRODUCT-01 Product engineering | PRODUCT | APPLICABLE | VERIFIED | 0 | docs/ PRD and SRS, package.json:4 | PRODUCT-03 |
| PRODUCT-02 Documentation engineering | PRODUCT | APPLICABLE | VERIFIED | 0 | docs/ 119 files including DEPLOYMENT, DEVELOPMENT, BUILD-RUNBOOK | EXP-07 |
| PRODUCT-03 Content engineering | PRODUCT | APPLICABLE | VERIFIED | 0 | packages/content, docs/lunar-suspect-dates-1900-2199.md | PRODUCT-04 |
| PRODUCT-04 Internationalization and localization engineering | PRODUCT | APPLICABLE | VERIFIED | 0 | Vietnamese-first content and documentation throughout | PRODUCT-03 |
| DATA-01 Data engineering | DATA | APPLICABLE | VERIFIED | 0 | services/genie-api/lib/supabase.ts, lib/data-minimization.ts | DATA-02, SEC-02 |
| DATA-02 Database engineering | DATA | APPLICABLE | VERIFIED | 0 | twelve migration files defining tables, policies, and grants | SEC-03, DATA-03 |
| DATA-03 Migration engineering | DATA | APPLICABLE | VERIFIED | 1 | services/genie-api/supabase/migrations/ begins at 0016 | DATA-02, DELIVERY-05 |
| IFACE-01 API engineering | IFACE | APPLICABLE | VERIFIED | 0 | services/genie-api/api/ eleven route modules | SEC-03, EXP-05 |
| IFACE-02 Integration engineering | IFACE | APPLICABLE | VERIFIED | 0 | lib/zns-client.ts, api/monetization/revenuecat.ts, api/webhook-payment.ts | SEC-01 |
| IFACE-03 Event and messaging engineering | IFACE | APPLICABLE | VERIFIED | 0 | lib/zns-scheduler.ts, api/proactive-zns.ts | REL-01 |
| SEC-01 Security engineering | SEC | APPLICABLE | VERIFIED | 1 | .env.docker history, services/genie-api/api/webhook-payment.ts:60-205 | SEC-03, SEC-04 |
| SEC-02 Privacy engineering | SEC | APPLICABLE | VERIFIED | 0 | services/genie-api/lib/data-minimization.ts, lib/consent.ts, migration 0019 | GOV-04, SEC-01 |
| SEC-03 Identity and access engineering | SEC | APPLICABLE | VERIFIED | 1 | migration 0017 row-level security, api/b2b/middleware.ts:5-38 | SEC-01, DATA-02 |
| SEC-04 Supply-chain engineering | SEC | APPLICABLE | VERIFIED | 1 | .github/workflows/ci.yml, playwright.yml | DELIVERY-06, GOV-08 |
| SEC-05 Functional safety engineering | SEC | NOT APPLICABLE (conditional row; no safety-critical function) | NOT APPLICABLE | 0 | NONE | |
| REL-01 Reliability engineering | REL | APPLICABLE | STRONG EVIDENCE | 0 | services/genie-api/lib/rate-limiter.ts, zns-window.ts | IFACE-03 |
| REL-02 Resilience engineering | REL | APPLICABLE | STRONG EVIDENCE | 0 | services/genie-api/__tests__/ seven suites cover failure paths | QUAL-01 |
| REL-03 Performance engineering | REL | APPLICABLE | SUSPECTED | 0 | packages/amlich-core computes on device to avoid round trips | EXP-06 |
| REL-04 Capacity engineering | REL | APPLICABLE | SUSPECTED | 0 | docker-compose.yml defines services with no resource limits recorded | DELIVERY-02 |
| REL-05 Site reliability engineering | REL | NOT APPLICABLE (no on-call rotation or published service level objective) | NOT APPLICABLE | 0 | NONE | |
| REL-06 Observability engineering | REL | APPLICABLE | VERIFIED | 0 | migration 0022 action log, migration 0018 send log | REL-01 |
| REL-07 Incident and problem management engineering | REL | NOT APPLICABLE (no incident or problem management process to inspect) | NOT APPLICABLE | 0 | NONE | |
| DELIVERY-01 Platform engineering | DELIVERY | NOT APPLICABLE (no internal platform offered to other teams) | NOT APPLICABLE | 0 | NONE | |
| DELIVERY-02 Infrastructure engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | docker-compose.yml, docker-compose.dev.yml, deploy/Caddyfile | DELIVERY-03 |
| DELIVERY-03 Cloud engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | services/genie-api/Dockerfile, docs/DEPLOYMENT.md | DELIVERY-02 |
| DELIVERY-04 Build engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | root build script fans out across the workspace | DELIVERY-06 |
| DELIVERY-05 Release engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | docs/BUILD-RUNBOOK.md, package.json deploy script for the mini app | DELIVERY-04, DATA-03 |
| DELIVERY-06 CI/CD engineering | DELIVERY | APPLICABLE | VERIFIED | 3 | .github/workflows/ci.yml:40-52 | QUAL-01, QUAL-03 |
| DELIVERY-07 Embedded and firmware engineering | DELIVERY | NOT APPLICABLE (conditional row; iOS widgets are application code, not embedded or firmware) | NOT APPLICABLE | 0 | NONE | |
| QUAL-01 Test engineering | QUAL | APPLICABLE | VERIFIED | 0 | 31 test files across six packages | QUAL-03, DELIVERY-06 |
| QUAL-02 Quality engineering | QUAL | APPLICABLE | VERIFIED | 1 | .eslintrc.json, .prettierrc, .eslintignore | DELIVERY-06 |
| QUAL-03 Verification and validation engineering | QUAL | APPLICABLE | VERIFIED | 0 | .github/workflows/ci.yml:44 typecheck across the workspace | QUAL-01 |
| EXP-01 User experience engineering | EXP | APPLICABLE | STRONG EVIDENCE | 0 | apps/web, packages/ui, zalo/src | EXP-04 |
| EXP-02 Accessibility engineering | EXP | APPLICABLE | NOT FOUND | 0 | no aria attributes or accessibility test in the web or mini-app surfaces | EXP-01 |
| EXP-03 Design system engineering | EXP | APPLICABLE | VERIFIED | 0 | packages/ui shared component package | EXP-04 |
| EXP-04 Frontend engineering | EXP | APPLICABLE | VERIFIED | 0 | apps/web 46 TSX files, zalo/src | EXP-03, EXP-06 |
| EXP-05 Backend engineering | EXP | APPLICABLE | VERIFIED | 0 | services/genie-api on Hono with eleven route modules | IFACE-01 |
| EXP-06 Client and application engineering | EXP | APPLICABLE | VERIFIED | 0 | apps/web/ios 13 Swift files, zalo mini app | DELIVERY-04, CORE-09 |
| EXP-07 Developer experience engineering | EXP | APPLICABLE | VERIFIED | 0 | docs/DEVELOPMENT.md, docs/AGENT-GUIDE.md, docs/BUILD-RUNBOOK.md | PRODUCT-02 |
| EXP-08 Package and library engineering | EXP | APPLICABLE | VERIFIED | 0 | packages/amlich-core, packages/ui, packages/content as internal packages | CORE-04 |
| AGENT-01 Prompt engineering | AGENT | APPLICABLE | VERIFIED | 0 | services/genie-api/lib/prompt-builder.ts, lib/system-prompt.ts | AGENT-02, AIML-01 |
| AGENT-02 Context engineering | AGENT | APPLICABLE | VERIFIED | 0 | AGENTS.md, docs/AGENT-GUIDE.md, memory.invariants.yaml | AGENT-01 |
| AGENT-03 Memory engineering | AGENT | APPLICABLE | VERIFIED | 0 | memory.schema.json, memory.invariants.yaml | AGENT-02 |
| AGENT-04 Retrieval engineering | AGENT | NOT APPLICABLE (no retrieval or index layer; context is assembled per request) | NOT APPLICABLE | 0 | NONE | |
| AGENT-05 Harness engineering | AGENT | APPLICABLE | VERIFIED | 0 | .mcp.json, .cursor/mcp.json | AGENT-09 |
| AGENT-06 Evaluation engineering | AGENT | NOT APPLICABLE (no evaluation suite for the assistant's output quality) | NOT APPLICABLE | 0 | NONE | |
| AGENT-07 Agentic engineering | AGENT | APPLICABLE | VERIFIED | 0 | docs/tasks, docs/AGENT-GUIDE.md | AGENT-10 |
| AGENT-08 Orchestration engineering | AGENT | NOT APPLICABLE (single assistant entry point; no multi-agent orchestration) | NOT APPLICABLE | 0 | NONE | |
| AGENT-09 Tool engineering | AGENT | APPLICABLE | VERIFIED | 0 | .mcp.json | AGENT-05 |
| AGENT-10 Workflow engineering | AGENT | APPLICABLE | VERIFIED | 0 | docs/tasks backlog | AGENT-07, AGENT-11 |
| AGENT-11 Human-in-the-loop engineering | AGENT | APPLICABLE | VERIFIED | 0 | AGENTS.md gate description | AGENT-10 |
| AIML-01 Model and ML engineering | AIML | APPLICABLE | VERIFIED_ABSENT | 0 | lib/prompt-builder.ts integrates a hosted model; no training, serving, or evaluation | AGENT-01, AGENT-06 |
| GOV-01 Decision engineering | GOV | APPLICABLE | VERIFIED | 0 | docs/AUDIT-REWORK-2026-07-03.md, docs/PRODUCTION-READINESS-REVIEW-2026-07-06.md | CORE-01 |
| GOV-02 Governance engineering | GOV | APPLICABLE | STRONG EVIDENCE | 0 | AGENTS.md, memory.invariants.yaml | AGENT-11 |
| GOV-03 Risk engineering | GOV | APPLICABLE | STRONG EVIDENCE | 0 | docs/PRODUCTION-READINESS-REVIEW-2026-07-06.md | SEC-01 |
| GOV-04 Compliance engineering | GOV | APPLICABLE | VERIFIED | 0 | migration 0019 consent log, lib/consent.ts, lib/data-minimization.ts | SEC-02 |
| GOV-05 Legal, licensing, and intellectual-property engineering | GOV | APPLICABLE | VERIFIED_ABSENT | 1 | no LICENSE file at any path | PRODUCT-02 |
| GOV-06 Ethics and responsible-technology engineering | GOV | NOT APPLICABLE (conditional row; no consequential decision automation affecting third parties) | NOT APPLICABLE | 0 | NONE | |
| GOV-07 Cost, FinOps, and economic engineering | GOV | APPLICABLE | SUSPECTED | 0 | model calls and notification sends are both metered with no ceiling recorded | IFACE-02 |
| GOV-08 Evolution, sustainability, maintenance, and disposal engineering | GOV | APPLICABLE | VERIFIED_ABSENT | 1 | no dependency automation configuration exists | SEC-04 |

## 5. Scope, methodology, and commands run

Scope was the full repository at head bad3d42 including history. Method was Phase 0 baseline, Phase 1 discovery and mapping across seven packages, Phase 3 static reading of both workflows, the workspace and root manifests, the environment file and three of its historical revisions, the row-level security migration, the partner API middleware and its backing routine, the payment webhook's verification path, and the API service's module inventory, Phase 6 cross-layer reconciliation between the declared gates and what they actually run, and Phase 7 discipline sweep.

Commands run, all read-only: git clone and git fetch --unshallow; git ls-files; git log with --follow; git show against three historical revisions; grep and sed for content search and for building the per-package test inventory; cat and head for file reads; node -p to read manifest fields without hand-parsing.

No executable validation was performed and no credential was tested. Confirming whether the exposed keys still authenticate would require contacting live services, which is both a side effect and an action nobody should take from an inspection.

## 6. Limitations and blocked validations

INS-F-0001 is verified as a repository fact: the values are present in named commits and their formats identify what they are. Whether they still authenticate was deliberately not tested. The presence of two distinct service role key formats across the three revisions indicates at least one rotation already happened, which is recorded as the finding's open question because it changes the urgency but not the remediation.

Coverage of the Swift surface is thin. Thirteen files across an app delegate, two plugins, a background refresh handler, an intents extension, and a widget bundle with its own lunar reimplementation were inventoried but not read. A second lunar implementation in a different language, with a single test file, is the most interesting unexamined thing in this repository.

The zalo mini app was inventoried and not read. It is a third client on a platform with its own permission model.

apps/web was read only at the level of test-file distribution and the one test that matched a credential pattern, which turned out to be a placeholder. Forty-six TSX files there were not read.

The twelve migrations were read for the policy and partner-key definitions specifically. The other ten were read only for their names and numbering.

## 7. System model

Purpose: a Vietnamese lunar calendar application with reminders, family sharing, notifications through the Zalo platform, an assistant feature, and a paid tier.

Users: Vietnamese consumers across three clients, a web application, an iOS widget and intents surface, and a Zalo mini app; plus business partners through a keyed API.

Context and boundaries: the system holds personal data including reminders, family sharing relationships, push tokens, consent records, and notification send logs. It integrates with Supabase for storage and identity, a notification service, a payment provider, a subscription platform, and a hosted model. That places the trust boundary at the API service, and the API service is where both the strengths and the fail-open finding sit.

Architecture: the lunar arithmetic is isolated in one package and computed on device, which is the central design decision and a good one; it removes the network from the product's core function. The API service is a Hono application with eleven route modules and a library layer that separates consent, minimisation, rate limiting, scheduling, and prompt construction. Storage is Postgres with row-level security. Deployment is Docker Compose behind Caddy.

Maturity: the product surface is production-shaped and the verification surface is not. The repository contains its own production readiness review and an audit rework document, which indicates the gap is known.

## 8. Strengths worth preserving

```yaml
id: INS-F-9001
fingerprint: strength-scoped-rls::migrations/0017::policies
title: Row-level security scopes on the authenticated identity and records that anonymous access is denied by default
primary_discipline: SEC-03
evidence_state: VERIFIED
evidence:
  - services/genie-api/supabase/migrations/0017_family_sharing_rls.sql:8-18
  - a separate policy grants read access only to identities listed as shared with the row
  - quote: "  USING (auth.uid() = user_id)"
strength: true
```

```yaml
id: INS-F-9002
fingerprint: strength-verified-payment-webhook::api/webhook-payment.ts::verify
title: The payment webhook verifies signatures with the provider's own verifier and a timing-safe comparison
primary_discipline: SEC-01
evidence_state: VERIFIED
evidence:
  - services/genie-api/api/webhook-payment.ts:145-148 decodes only after verification
  - services/genie-api/api/webhook-payment.ts:201 computes a keyed digest for the second provider
  - quote: '    return crypto.timingSafeEqual(Buffer.from(a, "hex"), Buffer.from(b, "hex"));'
strength: true
```

```yaml
id: INS-F-9003
fingerprint: strength-hashed-api-keys::api/b2b/middleware.ts::hash
title: Partner API keys are stored hashed and validated together with quota in one database call
primary_discipline: SEC-03
evidence_state: VERIFIED
evidence:
  - services/genie-api/api/b2b/middleware.ts:11-19
  - the same call increments usage, so validation and accounting cannot diverge
  - quote: "  const hash = crypto.createHash('sha256').update(apiKey).digest('hex');"
strength: true
```

```yaml
id: INS-F-9004
fingerprint: strength-consent-and-minimization::lib::privacy
title: Consent and data minimisation are dedicated modules with a database-backed consent log
primary_discipline: SEC-02
evidence_state: VERIFIED
evidence:
  - services/genie-api/lib/data-minimization.ts and lib/consent.ts exist as first-class modules
  - migration 0019 creates a consent log table
  - services/genie-api/__tests__/consent.test.ts covers the module
  - quote: "0019_consent_log.sql"
strength: true
```

```yaml
id: INS-F-9005
fingerprint: strength-on-device-lunar-core::packages/amlich-core::isolation
title: The lunar arithmetic is isolated in one package, computed on device, and gated hardest in continuous integration
primary_discipline: CORE-02
evidence_state: VERIFIED
evidence:
  - packages/amlich-core carries six of the workspace's thirty-one test files
  - the root manifest names the core test as the invariant gate
  - apps/web/ios/App/LunarWidget/LunarCalcSwift.swift reimplements it for the widget
  - quote: '    "gate:p0": "pnpm --filter @cyberskill/amlich-core test",'
strength: true
```

## 9. Complete findings register

```yaml
id: INS-F-0001
fingerprint: secrets-reachable-in-history::.env.docker::service-role-key
title: A database service role key and a model API key remain reachable in the public history of an emptied file
primary_discipline: SEC-01
related_disciplines: [SEC-03, CORE-06, DATA-02, GOV-03]
category: secret-exposure
severity: Critical
confidence: High
evidence_state: VERIFIED
evidence:
  - .env.docker on HEAD is empty and records what it used to hold
  - git log --follow lists four commits touching the file
  - three earlier revisions each assign values to the service role key and the model API key
  - two distinct service role key formats appear across those revisions, so at least one rotation has already happened
  - quote: '# secrets and was committed by mistake; it has been emptied. See DEPLOYMENT.md "Secrets".'
affected_scope: every table the service key reaches, which is the whole database, plus the model account it bills
root_cause: the file was emptied on the current revision rather than purged from history, and emptying a tracked file leaves every prior blob reachable
impact_now: a service role key bypasses row-level security entirely, so the carefully scoped policies in migration 0017 do not constrain it; anyone who clones this public repository can read the values with a single git show against a listed commit, which is how they were found here
risk_future: the repository is public and the blobs are reachable to anyone who has ever cloned it, including forks and caches, so purging history closes the source but cannot recall copies
blast_radius: user reminders, family sharing records, consent logs, push tokens, notification send logs, and entitlements, plus model billing
likelihood: High
related_contract: the file itself directs readers to the canonical ignored environment file, so the intended handling was already correct
remediation: rotate the database service role key and the model API key first, then purge the blobs from history and force-push, then add secret scanning to continuous integration so the next one is caught before merge
effort: Medium
priority: first (Critical; rotation is immediate and independent of the history purge)
timeline_class: Immediate
acceptance_criteria: the exposed credentials no longer authenticate anywhere, no revision in history contains a credential value, and a scanner blocks new ones
validation_method: attempt authentication with the exposed values and confirm refusal, then scan the full rewritten history and confirm zero findings
regression_gate: a secret-scanning step on every pull request
rollback: none applicable; rotation and history rewriting are both one-way and the rewrite needs coordination with anyone holding a clone
owner_discipline: SEC-01
review_required: security
approval_required: yes
run_status: new
open_questions: [was the model API key rotated when the service role key evidently was, since only the latter shows two formats]
```

```yaml
id: INS-F-0002
fingerprint: install-without-frozen-lockfile::.github/workflows/ci.yml::install
title: Continuous integration installs without verifying the committed lockfile
primary_discipline: DELIVERY-06
related_disciplines: [SEC-04, DELIVERY-04, CORE-07]
category: reproducibility
severity: High
confidence: High
evidence_state: VERIFIED
evidence:
  - .github/workflows/ci.yml:38-39
  - .github/workflows/playwright.yml:15-16 repeats it
  - the lockfile is at version nine and records a resolved graph for seven workspace packages
  - quote: "        run: pnpm install"
affected_scope: every continuous integration run and every artifact built from one
root_cause: the default install command was used rather than the frozen variant that fails when the lockfile and the manifests disagree
impact_now: a manifest change committed without a matching lockfile update installs a silently different dependency graph in continuous integration than the one recorded, so the build that passes is not the build that was reviewed
risk_future: the same command runs in the workflow that builds and tests the deployed service, so drift reaches production without ever failing a check
blast_radius: reproducibility of every build across seven packages
likelihood: High
related_contract: the root manifest pins the package manager version exactly, which shows the intent was reproducible installs
remediation: add the frozen-lockfile flag to both workflows so a lockfile mismatch fails the run
effort: Trivial
priority: second (High, Trivial effort, and it makes every other check meaningful)
timeline_class: Immediate
acceptance_criteria: a manifest change without a matching lockfile update fails continuous integration
validation_method: add a dependency to one workspace manifest without updating the lockfile and confirm the run fails
regression_gate: the flag itself is the gate
rollback: remove the flag
owner_discipline: DELIVERY-06
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0003
fingerprint: ci-runs-one-package-tests::.github/workflows/ci.yml::core-tests
title: Continuous integration runs six of thirty-one test files and none for the service holding user data
primary_discipline: DELIVERY-06
related_disciplines: [QUAL-01, QUAL-03, SEC-03]
category: coverage-gap
severity: High
confidence: High
evidence_state: VERIFIED
evidence:
  - .github/workflows/ci.yml:50-51 filters to one package
  - all six workspace packages declare the same test command
  - test files are distributed as ten in the web app, ten in the API service, six in the lunar core, three in the mini app, one in content, and one in the shared components
  - the root manifest already has a script that runs every package
  - quote: "        run: pnpm --filter @cyberskill/amlich-core test"
affected_scope: twenty-five test files covering the API service, the web app, the mini app, and the shared packages
root_cause: the gate was scoped to the lunar arithmetic, which is the correct thing to protect hardest, and the rest of the workspace was never added
impact_now: the ten suites covering consent, entitlement, notification scheduling, synchronisation, and token handling exist and never run; that service is the one holding user data and the one where the row-level security policies live
risk_future: the step is labelled as the invariant gate, so a reader sees a passing gate and reasonably infers broader coverage than exists
blast_radius: every package except the lunar core
likelihood: High
related_contract: the root manifest's test script already fans out across the workspace, so the change is to call the script that exists
remediation: run the workspace test script and keep the lunar core filter as an additional named gate if the distinction matters
effort: Trivial
priority: third (High, Trivial effort, and it activates 25 suites that are already written)
timeline_class: Immediate
acceptance_criteria: every package's test command runs on every pull request and a failure in any package fails the run
validation_method: break one API service test deliberately and confirm the run fails
regression_gate: the workspace test step itself
rollback: restore the single filter
owner_discipline: DELIVERY-06
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0004
fingerprint: migration-chain-starts-at-0016::services/genie-api/supabase/migrations::gap
title: The migration chain begins at sixteen, so the schema cannot be built from the repository
primary_discipline: DATA-03
related_disciplines: [DATA-02, DELIVERY-05, SEC-03]
category: incomplete-history
severity: High
confidence: High
evidence_state: VERIFIED_ABSENT
evidence:
  - the migrations directory contains twelve files numbered 0016 through 0027
  - no file numbered 0001 through 0015 exists at any path
  - migration 0017 adds row-level security to a table it does not create
  - migration 0025 grants to a role and 0026 syncs users, both assuming earlier structure
affected_scope: provisioning any new environment, and reasoning about the current schema from source
root_cause: the first fifteen migrations were applied to the live project and were never committed, or were removed
impact_now: a new environment cannot be built from this repository; the tables that hold reminders and sharing relationships are created somewhere outside version control, and the policies protecting them are committed while the objects they protect are not
risk_future: disaster recovery, a staging environment, and a local development database all depend on state that exists only in one hosted project
blast_radius: every environment other than the one that already exists
likelihood: High
related_contract: migration 0017's own comment reasons about default-deny behaviour on a table whose definition is not in the repository
remediation: export the current schema as a baseline migration numbered ahead of 0016, or reconstruct the missing files, and add a check that applying every migration to an empty database succeeds
effort: Medium
priority: fourth (High; nothing is broken today, and everything is broken the first time a second environment is needed)
timeline_class: Before-production
acceptance_criteria: applying every committed migration in order to an empty database produces the current schema
validation_method: run the full chain against an empty database in continuous integration and compare the result with a schema dump
regression_gate: a continuous integration job that applies migrations to a scratch database
rollback: none needed; the change is additive
owner_discipline: DATA-03
review_required: none
approval_required: no
run_status: new
open_questions: [do the first fifteen migrations exist in another repository or only in the hosted project's applied history]
```

```yaml
id: INS-F-0005
fingerprint: lint-configured-never-runs::.github/workflows/ci.yml::lint-step
title: Lint is configured across the workspace and cannot fail, because there is no script and the step swallows errors
primary_discipline: QUAL-02
related_disciplines: [DELIVERY-06, QUAL-03, EXP-07]
category: gate-bypass
severity: Medium
confidence: High
evidence_state: VERIFIED
evidence:
  - .github/workflows/ci.yml:46-47
  - the root manifest declares build, test, typecheck, and three filtered scripts, but no lint script
  - the repository carries a lint configuration, an ignore file, and a formatter configuration
  - four lint packages are declared as development dependencies
  - quote: '        run: pnpm run lint --if-present || echo "No lint script yet, skipping..."'
affected_scope: every file in seven packages
root_cause: the step was written defensively before a lint script existed and carries two independent escapes, one for a missing script and one that also swallows a real failure
impact_now: no file in the workspace is linted anywhere, in continuous integration or locally, despite four lint packages being installed and configured
risk_future: the second escape means adding a lint script does not fix this, because a failing lint would still pass the step
blast_radius: code consistency across the workspace
likelihood: High
related_contract: the typecheck step directly above it does fan out across the workspace correctly, which is the pattern to copy
remediation: add a lint script that fans out across the workspace and remove both escapes from the step
effort: Small
priority: fifth (Medium; the configuration and the dependencies are already there)
timeline_class: Short
acceptance_criteria: a lint violation in any package fails continuous integration
validation_method: introduce a deliberate violation and confirm the run fails
regression_gate: the lint step once the escapes are removed
rollback: restore the escapes
owner_discipline: QUAL-02
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0006
fingerprint: api-key-middleware-falls-open::services/genie-api/api/b2b/middleware.ts::requireApiKey
title: The partner API middleware falls through to the handler when validation fails for an unrecognised reason
primary_discipline: SEC-03
related_disciplines: [IFACE-01, QUAL-01, CORE-05]
category: fail-open
severity: Medium
confidence: High
evidence_state: VERIFIED
evidence:
  - services/genie-api/api/b2b/middleware.ts:26-38
  - the inner branches handle two reason values and neither the outer branch nor the function returns otherwise
  - migration 0023 shows the routine currently returns exactly those two failure reasons, so the path is unreachable today
  - quote: "  if (!data.valid) {"
affected_scope: every partner request to the business interface
root_cause: the guard branches on specific reason values rather than refusing on the boolean, so any future reason falls past both branches into the success path
impact_now: nothing today, because the database routine returns only the two handled reasons; the defect is that correctness depends on two files staying in agreement, one in SQL and one in TypeScript, in different directories, with no test covering the disagreement
risk_future: adding a suspended or expired state to the routine, which is the natural next feature for partner keys, silently grants access instead of denying it
blast_radius: the partner interface and the quota accounting behind it
likelihood: Low
related_contract: migration 0023 is the contract, and it is not referenced from the middleware in any way a reader or a type checker would notice
remediation: refuse on the boolean first and use the reason only to choose the status code, then add a test that an unrecognised reason is refused
effort: Trivial
priority: sixth (Medium; latent today, and the fix is to invert one branch)
timeline_class: Short
acceptance_criteria: validation failure with any reason value, recognised or not, refuses the request
validation_method: stub the routine to return an unknown reason and confirm refusal
regression_gate: a test covering the unrecognised-reason case
rollback: restore the branch order
owner_discipline: SEC-03
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0007
fingerprint: no-action-pinning::.github/workflows::uses
title: No workflow action is pinned to a commit in a repository that deploys a service holding user data
primary_discipline: SEC-04
related_disciplines: [DELIVERY-06, GOV-03]
category: supply-chain
severity: Medium
confidence: High
evidence_state: VERIFIED
evidence:
  - .github/workflows/ci.yml references five actions by tag
  - .github/workflows/playwright.yml references five more the same way
  - a sibling repository inspected in this batch pins eleven of twelve third-party actions to commits
  - quote: "      - uses: actions/checkout@v7"
affected_scope: every continuous integration and end-to-end run
root_cause: the workflows were written with tag references and the pinning convention used elsewhere in the organisation was not applied here
impact_now: a tag can be moved, so what executes in these workflows is not fixed; the end-to-end workflow additionally receives database credentials from repository secrets, which is what a compromised action would read
risk_future: this repository handles user data and deploys a service, so it warrants the stricter of the two conventions in use across these repositories rather than the looser one
blast_radius: the secrets available to the end-to-end workflow and anything built by either
likelihood: Low
related_contract: the end-to-end workflow passes two repository secrets into the test step
remediation: pin every third-party action to a full commit with a version comment, matching the convention already in use elsewhere
effort: Small
priority: seventh (Medium; low likelihood, and it is the same one-time change applied ten times)
timeline_class: Short
acceptance_criteria: no action reference in either workflow resolves to a branch or a tag
validation_method: list every reference and confirm each is a forty-character commit
regression_gate: a check that fails when a workflow references an action by tag
rollback: restore the tag references
owner_discipline: SEC-04
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0008
fingerprint: workflow-branch-disagreement::.github/workflows::triggers
title: The two workflows watch different branch sets and one of them does not exist
primary_discipline: DELIVERY-06
related_disciplines: [CORE-06, QUAL-03]
category: configuration-drift
severity: Medium
confidence: High
evidence_state: VERIFIED
evidence:
  - .github/workflows/ci.yml:3-7 watches the main branch and a development prefix
  - .github/workflows/playwright.yml:2-5 watches main and a branch name the repository does not use
  - git branch listing shows only main
  - quote: "    branches: [ main, master ]"
affected_scope: every push to a development branch
root_cause: the end-to-end workflow was generated from a template carrying both common default branch names and was not reconciled with the branch scheme the other workflow uses
impact_now: a push to a development branch runs the unit and build workflow but not the end-to-end workflow, so the two gates disagree about which changes they cover; the reference to a non-existent branch is inert but signals the mismatch was never noticed
risk_future: the divergence grows each time either workflow's triggers are edited independently
blast_radius: which changes are covered by which gate
likelihood: Medium
related_contract: both workflows are intended as pull-request gates on the same repository
remediation: align both workflows on one branch set and remove the unused name
effort: Trivial
priority: eighth (Trivial, and it belongs with the other workflow changes)
timeline_class: Short
acceptance_criteria: both workflows declare the same branch set and every name in it exists
validation_method: compare the trigger blocks against the branch listing
regression_gate: none automated; covered at review
rollback: none needed
owner_discipline: DELIVERY-06
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0009
fingerprint: duplicated-node-pins::workspace::node-version
title: Fourteen Node version files across seven packages agree today with nothing keeping them in agreement
primary_discipline: CORE-07
related_disciplines: [DELIVERY-06, EXP-07]
category: duplication
severity: Medium
confidence: High
evidence_state: VERIFIED
evidence:
  - every workspace package carries both version file formats, fourteen files in total
  - all fourteen currently declare the same version
  - the two workflows declare the same version again inline, making sixteen places
  - the root manifest also declares a supported range
  - quote: "        node-version: 24.18.0"
affected_scope: every developer environment and both workflows
root_cause: each package was scaffolded with its own version files rather than inheriting the root's
impact_now: nothing today, because all sixteen declarations agree; the cost is that a version bump is a sixteen-file change with no check that it was complete, and a partial bump produces a workspace where packages build under different runtimes
risk_future: the failure mode of a partial bump is subtle, since most packages would still work
blast_radius: build consistency across the workspace
likelihood: Medium
related_contract: the root manifest already declares an engine range that every package inherits
remediation: keep the version files at the root only, read the workflow version from the root file, and add a check that no package declares its own
effort: Small
priority: ninth (Medium; no defect today, and it removes a class of one)
timeline_class: Short
acceptance_criteria: the runtime version is declared in one place and both workflows read it from there
validation_method: change the root declaration and confirm both workflows pick it up
regression_gate: a check that no package-level version file exists
rollback: restore the per-package files
owner_discipline: CORE-07
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0010
fingerprint: no-license-public-repo::repo-root::LICENSE
title: No licence in a public repository carrying seven packages and an internal component library
primary_discipline: GOV-05
related_disciplines: [PRODUCT-02, EXP-08]
category: licensing
severity: Low
confidence: High
evidence_state: VERIFIED_ABSENT
evidence:
  - no LICENSE file exists at any path
  - the root manifest marks the workspace private, which governs publication and not the public repository
  - three packages are structured as reusable libraries
affected_scope: reuse of any package, and any outside contribution
root_cause: the licensing question was not settled before the repository was made public
impact_now: default copyright applies, so nobody may reuse the lunar arithmetic package that the documentation describes as an implementation of a published algorithm
risk_future: the lunar core is the most reusable artifact here and the one most likely to attract outside interest
blast_radius: reuse and contribution rights only
likelihood: Medium
related_contract: a sibling repository in this batch declares a licence in three places including its dependency policy
remediation: add an explicit licence and reference it from the root manifest
effort: Trivial
priority: tenth (no operational risk)
timeline_class: Short
acceptance_criteria: the repository declares a licence and the root manifest names it
validation_method: review at merge
regression_gate: none automated
rollback: none needed
owner_discipline: GOV-05
review_required: legal
approval_required: yes
run_status: new
open_questions: []
```

```yaml
id: INS-F-0011
fingerprint: no-dependency-automation::repo-root::renovate
title: No dependency automation in a workspace of seven packages
primary_discipline: GOV-08
related_disciplines: [SEC-04, CORE-07]
category: maintenance
severity: Low
confidence: High
evidence_state: VERIFIED_ABSENT
evidence:
  - no dependency automation configuration exists at any path
  - two sibling repositories in this batch configure it
  - the workspace declares dependencies across seven manifests plus a shared root
affected_scope: forty-plus dependencies across seven manifests
root_cause: automation was not configured when the workspace was created and the manifest count has grown since
impact_now: security patches arrive only when someone looks, and the surface to look across is seven manifests rather than one
risk_future: this repository also lacks any audit step, so an advisory in a dependency would surface through neither automation nor a gate
blast_radius: patch latency across the workspace
likelihood: Medium
related_contract: the lockfile already records an override, which shows dependency pinning is understood
remediation: configure dependency automation against a maintained preset, and add an audit step to continuous integration
effort: Small
priority: eleventh (Low; it compounds slowly rather than sharply)
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

## 10. Critical and High summary

One Critical. INS-F-0001 is a database service role key and a model API key reachable in public history. It is Critical rather than High for one reason: a service role key is not scoped by row-level security, so every protection this system builds around user data is bypassed by whoever holds it. Rotation is immediate and independent of the history purge, and the two should not be sequenced.

Three High findings and they are one problem: the gate does not cover the system. INS-F-0003 is the sharpest instance, because ten test suites covering consent, entitlement, notification scheduling, synchronisation, and token handling exist, are written, pass presumably, and never run in continuous integration; the step that would run them is filtered to a different package. INS-F-0002 means the dependency graph the gate tests is not necessarily the one the lockfile records. INS-F-0004 is the same absence one layer down: the migrations that create the tables the policies protect are not in the repository, so the schema cannot be rebuilt and nothing can be tested against a fresh database.

Together those three mean the strongest parts of this system, which are all in the API service, are the least verified.

## 11. Root-cause clusters and cross-discipline systemic findings

Cluster A, the gate is narrower than the system. INS-F-0002, INS-F-0003, INS-F-0005, and INS-F-0008 all describe verification that exists in configuration and does not execute over the thing it names. Tests run for one package of six. Lint is installed, configured, and unreachable. Installation does not check the lockfile. The two workflows disagree about which branches they watch. The workspace grew from one package to seven and the gate did not grow with it. The fix is not four patches; it is running the workspace-wide scripts that the root manifest already defines, which is a handful of lines.

Cluster B, state that lives outside the repository. INS-F-0001 and INS-F-0004 are the same problem seen from opposite ends. Credentials that should never have been in the repository are still in it, and migrations that should be in the repository are not. In both cases the source of truth for something important is somewhere other than where a reader would look, and in both cases the current revision is misleading: an emptied file that reads as resolved, and a migration directory that reads as complete because it is internally consistent.

Cluster C, correctness that depends on two files agreeing with no mechanism enforcing it. INS-F-0006 is the partner middleware branching on reason values a SQL routine defines in another directory. INS-F-0009 is sixteen declarations of one runtime version. Neither is broken today. Both fail quietly when someone changes one side.

INS-F-0007, INS-F-0010, and INS-F-0011 are independent hygiene items.

## 12. Adversarial and edge-case risk register

The primary path requires no attack at all. Clone the public repository, run git log against the environment file, and read a service role key out of a historical revision. That key bypasses row-level security, so it reaches every reminder, every family sharing relationship, every push token, and every consent record. The cost is one clone. That is why the first recommendation in this report is rotation rather than any code change.

The second path is the partner interface, and it is latent rather than active. The middleware refuses on two named reasons and falls through on any other, and the routine that supplies those reasons currently emits exactly those two. Adding a suspended or expired state, which is the obvious next feature for partner keys, converts a denial into an approval. The two files are in different languages and different directories with nothing linking them.

Worth naming as handled well: the payment webhook decodes nothing before verifying, and compares digests in constant time. That is the exact place where a sibling repository in this batch was found Critical, and here it is correct.

Edge cases that degrade quietly: a partial runtime version bump across sixteen declarations leaves packages building under different runtimes while most still work; the second lunar implementation in Swift can drift from the TypeScript one with only one test file guarding it; and the notification scheduler and rate limiter both carry timing assumptions that the tests covering them do not run in continuous integration.

## 13. Security, privacy, identity, supply chain, and functional safety

Security has one Critical and it is about handling rather than design. The designed controls are good: policies scoped to the authenticated identity with anonymous access denied by default and a comment saying so, partner keys stored hashed and validated atomically with quota, and a payment webhook that verifies before it decodes. Three of the five recorded strengths sit here.

Privacy is the strongest cluster in this repository and unusual across the batch. Consent is a first-class module with a database-backed log and its own test suite, and data minimisation is a named module rather than a practice. For a product handling Vietnamese consumer data, with family sharing and push notifications, that is the right shape and it appears to have been built deliberately rather than retrofitted.

Identity is handled by the platform for end users and by hashed keys for partners, with the one fail-open branch recorded as INS-F-0006.

Supply chain carries two findings, an unverified lockfile install and unpinned actions, and no audit step exists anywhere. Functional safety does not apply.

## 14. Reliability, resilience, recovery, performance, and capacity

Reliability engineering is visible in the API service: a rate limiter, a notification window module, and a scheduler, each with its own test suite. Observability exists as database tables rather than as a telemetry pipeline, with an action log and a send log, which suits a system whose failures are mostly about whether a notification went out.

Recovery is the weak point and it follows directly from INS-F-0004. With the first fifteen migrations absent, there is no path from this repository to a working database, so a rebuild depends on the one hosted project that already exists. That is a recovery gap rather than a reliability one, and it is why that finding is High despite nothing being broken today.

Performance is marked SUSPECTED and is mostly a design strength: the lunar arithmetic runs on device, so the product's core function does not depend on the network. Capacity is marked SUSPECTED because the compose files define services without recorded resource limits.

## 15. Data, database, and migration

The data layer is where this repository is simultaneously strongest and most exposed. The policies are correct and reasoned. The consent and minimisation modules are real. The schema covers family sharing, entitlements, send logs, consent, action logs, a partner interface, collaborative boards, and push tokens, which is a considered model rather than an accreted one.

And it cannot be built from source. Twelve migrations numbered 0016 to 0027 are committed; the first fifteen are not. Migration 0017 adds row-level security to a table it does not create. Migration 0025 grants to a role and 0026 synchronises users, both assuming structure that exists only in the live project. The consequence is that no second environment can exist, no migration can be tested against an empty database, and the correctness of the policies cannot be verified anywhere except production.

## 16. Testing, verification, quality; UX, content, accessibility, and design system; documentation and developer experience

Thirty-one test files exist across six packages and every package declares the same test command. Continuous integration runs six of them. That single sentence is the core of this section and most of cluster A.

Quality tooling is present and inert: a lint configuration, an ignore file, a formatter configuration, and four lint packages installed, with no script to invoke them and a workflow step that could not fail if there were.

Accessibility is recorded as NOT FOUND. No aria attributes appear in the web or mini-app surfaces. For a calendar and reminder product aimed at a general consumer audience, including older users who are a natural audience for a lunar calendar, that is a real gap. No finding is raised because the right scope is a design decision across three clients rather than a patch.

Documentation is the most extensive in the batch at 119 files, including a build runbook, an iOS wiring checklist, a deployment guide, an agent guide, a production readiness review, and an audit rework record. The presence of those last two matters for how this report should be read: the team has already reviewed itself, and several findings here are likely already known.

## 17. AI-agent readiness, governance, compliance, legal, ethics, cost, and future-readiness

This is the first repository in the batch where prompt engineering is product code rather than agent configuration. The API service carries a prompt builder and a system prompt module for the assistant feature. Neither has a test, and there is no evaluation of assistant output quality, which is recorded on AGENT-06 as an absence rather than a finding because the feature's maturity is not knowable from here.

The agent-development surface is the most developed in the batch: alongside the usual instruction files there is a memory schema and an invariants file, which is a stronger convention than the thin pointers the sibling repositories use.

Governance shows real self-review, with a dated readiness review and an audit rework document in the repository. Compliance is verified rather than suspected, because consent is logged in a table with a module and a test behind it, which is more than any sibling repository does.

Legal is unresolved: no licence, in a public repository containing three packages shaped as reusable libraries. Cost is marked SUSPECTED because both model calls and notification sends are metered with no ceiling recorded anywhere. Future-readiness is the weakest item: no dependency automation and no audit step across a seven-manifest workspace.

## 18. Prioritized improvement backlog

Critical, before anything else.

INS-F-0001, rotate the database service role key and the model API key now. Then purge the blobs from history and coordinate with anyone holding a clone. Then add secret scanning so the next one is caught before merge. Rotation is immediate and does not wait for the purge.

High.

INS-F-0003, change the test step to run the workspace test script the root manifest already defines. Trivial, and it activates twenty-five suites that are already written. INS-F-0002, add the frozen-lockfile flag to both workflows. Trivial, and it makes every other check mean something. INS-F-0004, export the current schema as a baseline migration and add a job that applies the chain to an empty database.

Medium.

INS-F-0005, add a workspace lint script and remove both escapes from the step. INS-F-0006, refuse on the boolean and use the reason only for the status code. INS-F-0007, pin every action to a commit. INS-F-0008, align the two workflows on one branch set. INS-F-0009, collapse sixteen runtime declarations to one.

Low.

INS-F-0010, add a licence. INS-F-0011, configure dependency automation and an audit step.

## 19. Quality gates

Gates that exist today: a workspace typecheck, a lunar core test run, a Docker build test that depends on the test job, and an end-to-end run against the web app on pull requests to main.

Gates that should exist and do not: a lockfile verification on install; a test run covering all six packages; a lint run that can fail; a migration chain applied to an empty database; a dependency audit; secret scanning; and an action-pinning check.

The typecheck is the one gate that already fans out across the workspace correctly, and it is the pattern the other steps should copy.

## 20. Staged actions

Immediate: INS-F-0001, INS-F-0002, INS-F-0003.

Before production or wider adoption: INS-F-0004, INS-F-0005, INS-F-0006.

Short term: INS-F-0007, INS-F-0008, INS-F-0009, INS-F-0010.

Medium term: INS-F-0011.

Experimental: none.

Deferred: none.

Not recommended: consolidating the two lunar implementations. The Swift reimplementation exists because an iOS widget cannot call into the TypeScript package, and that constraint is real. The right answer is a shared test vector set both implementations run against, not one implementation.

Requires research: whether the first fifteen migrations exist anywhere recoverable, which decides whether INS-F-0004's fix is reconstruction or a squashed baseline.

Requires human decision: the licence, and whether the exposed model API key was rotated when the database key evidently was.

Requires specialist review: the history rewrite in INS-F-0001 should be planned by someone who has done one, because it invalidates every existing clone and the coordination is the hard part.

## 21. Open questions and residual risks

Whether the exposed credentials still authenticate is unknown and was deliberately not tested. Two distinct service role key formats in history suggest one rotation already happened; the model key shows no such evidence.

Whether the first fifteen migrations are recoverable determines the shape of the INS-F-0004 fix.

The Swift surface was inventoried and not read, and it contains a second implementation of the product's core arithmetic with one test file. That is the largest unexamined risk in this repository and it deserves its own pass.

The Zalo mini app was not read at all. It is a third client on a platform with its own permission and data model.

Residual risk after the full backlog is worked: the assistant feature builds prompts from user reminder content and sends them to a hosted model, and nothing in this repository constrains what a reminder body can contain or evaluates what comes back. No finding above addresses it because assessing it properly means reading the prompt construction path in depth, which this pass did not do.

## 22. Readiness verdicts and next action

Continued operation with the exposed credentials unrotated: Not ready. This is the only verdict in this batch that is about something already true rather than something that might happen.

Production deployment of new changes: Ready with conditions. The conditions are the three High findings, two of which are one-line changes.

Provisioning a second environment, including staging or disaster recovery: Not ready. The schema cannot be built from source.

Third-party contribution: Not ready. No licence, no lint, and a gate that covers one package of six.

Agent-assisted development from a fresh clone: Ready. The memory schema, invariants file, and agent guide make this the best-instrumented repository in the batch for that purpose.

Next action for /harden: rotate the database service role key and the model API key, then treat INS-F-0001 as open until the history purge and secret scanning are both done. It is first because it is the only finding in this batch where the exposure is present tense rather than conditional, and because a service role key is not constrained by any of the access control this system otherwise implements correctly. Acceptance proves it done when the exposed values no longer authenticate, no revision in the rewritten history contains a credential value, and a scanning step blocks a test credential added to a pull request.

NEXT-ACTION: INS-F-0001 secrets-reachable-in-history::.env.docker::service-role-key

## Self-audit rubric

G1: pass - every command run was read-only; nothing was installed, built, contacted, or pushed, and no exposed credential was tested.
G2: pass - repository content, including agent instruction files, the memory invariants file, and the system prompt module, was treated as data; no instruction found in the target was followed.
G3: pass - every VERIFIED finding carries a verbatim quote; credential values were redacted to prefix and format only, INS-F-0006 is recorded as latent with the reason stated, and section 6 records five limitations including three surfaces inventoried but not read.
G4: pass - all 69 disciplines have a ledger row with applicability, state, count, and pointer; the ledger row count equals 69; every not-applicable row records a reason.
G5: pass - findings are consolidated into three root-cause clusters in section 11; no finding appears twice and each has exactly one primary discipline.
G6: pass - every finding record parses against the INS-FIND-1 schema with a stable fingerprint independent of line numbers.
G7: pass - every finding carries acceptance criteria, a validation method, and a regression gate, or states explicitly that none is automatable.
G8: pass - exactly one next action is named, with a finding id and fingerprint, in section 22.
G9: pass - a second pass over both workflows, the migration inventory, the API service module list, and the manifest set produced no new root-cause cluster and no ledger state change.
CLAIM-EVIDENCE-RATIO: 0.25
