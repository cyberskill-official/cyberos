# /inspect report: cyberskill-official/landing-page

## 1. Side-effect disclosure

None. Every command was read-only: git clone, git ls-files, file reads, and text search. No dependency was installed, no build was run, no deployment or database was contacted, and nothing was written to the repository or pushed. The clone lives in a scratch directory.

## 2. Executive summary

landing-page has the most developed quality gate across both batches and one failure mode that gate cannot see.

The gate is three jobs. The first chains a task-specification check, a static import check, lint, typecheck, tests, a build, an asset budget, and an assertion that the performance budget file itself has not been loosened. The second runs Lighthouse against the built site with budget assertions. The third runs axe against five real routes on a served production build and fails on serious or critical violations. The workflow's own comments record the staged path each gate took from advisory to blocking, which is a policy most teams keep in their heads.

Against that: the datastore silently falls back to a per-process store on two separate paths, and the choice is memoized for the life of the instance. Leads are not lost, because the email sink fail-closes and four other sinks still receive the record. What degrades is the durable row and the weekly teardown cap, which is enforced by counting rows, so a per-week limit becomes a per-instance limit and is effectively unenforced. The scheduled check that exists specifically to prove this pipeline works asserts only that the endpoint returns 200, which the degraded path also returns.

Third, the assistant proxy budgets requests per address using the first element of a caller-supplied forwarding header, so the twenty-request ceiling in front of a metered provider resets whenever the caller changes a string.

Findings: 10 total, 0 Critical, 3 High, 5 Medium, 2 Low. Six strengths recorded, the joint highest in either batch.

The one exact next action is at section 22.

## 3. Inspection metadata and baseline

Target: https://github.com/cyberskill-official/landing-page, default branch main, head f6375e4, 299 commits, last commit 2026-07-25. Working tree 11M excluding .git. 704 tracked files, of which 356 are documentation.

Note on the target name: the repository requested was `langing-page`, which returns 404. The organisation repository is `landing-page` and that is what was inspected.

Languages by line count: TypeScript 18,198 across 139 files; TSX 11,760 across 120; Markdown 11,807 across 246; JSON 11,374; CSS 4,256 across 4; ES modules 2,474 across 19, which is the script tree.

Stack: Next.js 16.2.10 with React 19.2.7, Prisma 7 over Postgres behind an adapter interface, Zod validation, React Hook Form, Zustand, and a dynamically imported WebGL enhancement built on Three, React Three Fiber, and GSAP. Vitest for unit tests, Playwright and Puppeteer both present, axe-core for accessibility, Lighthouse CI for performance. Node pinned to 24.18.0 in two files with a matching engine range. Eight API routes including an assistant proxy to a hosted model.

Inspection was single-pass and reached saturation without continuation; see section 5.

## 4. Coverage ledger

All 69 disciplines, in stable id order. 58 applicable, 11 not applicable with a recorded reason.

| Discipline | Cluster | Applicability | Evidence state | Finding count | Evidence pointer | Related |
|---|---|---|---|---|---|---|
| CORE-01 Requirements engineering | CORE | APPLICABLE | VERIFIED | 0 | docs/ 356 files, scripts/check-tasks.mjs enforced in CI | AGENT-10 |
| CORE-02 Domain engineering | CORE | APPLICABLE | VERIFIED | 0 | prisma/schema.prisma, lib/db/adapter.ts | DATA-02 |
| CORE-03 Systems engineering | CORE | APPLICABLE | VERIFIED | 0 | next.config.ts, proxy.ts, vercel.json, three-job workflow | DELIVERY-03 |
| CORE-04 Architecture engineering | CORE | APPLICABLE | VERIFIED | 0 | app/, components/ 99 files, lib/ 58 files with an adapter boundary | EXP-04, DATA-01 |
| CORE-05 Software engineering | CORE | APPLICABLE | VERIFIED | 0 | 18,198 lines TypeScript across 139 files | QUAL-01 |
| CORE-06 Repository engineering | CORE | APPLICABLE | VERIFIED | 0 | 299 commits, VERSION file agreeing with the manifest | DELIVERY-06 |
| CORE-07 Configuration engineering | CORE | APPLICABLE | VERIFIED | 1 | package.json:20-21 two scripts alias one file | DELIVERY-06 |
| CORE-08 Concurrency engineering | CORE | APPLICABLE | VERIFIED_ABSENT | 0 | no concurrent execution paths beyond per-request handlers | REL-02 |
| CORE-09 Distributed systems engineering | CORE | APPLICABLE | VERIFIED_ABSENT | 0 | single deployment plus one managed database | DELIVERY-03 |
| PRODUCT-01 Product engineering | PRODUCT | APPLICABLE | VERIFIED | 0 | package.json:4 states the product intent, app/ route groups | EXP-01 |
| PRODUCT-02 Documentation engineering | PRODUCT | APPLICABLE | VERIFIED | 0 | docs/ 356 files, inline rationale throughout the workflow | EXP-07 |
| PRODUCT-03 Content engineering | PRODUCT | APPLICABLE | VERIFIED | 0 | content across English and Vietnamese route variants | PRODUCT-04 |
| PRODUCT-04 Internationalization and localization engineering | PRODUCT | APPLICABLE | VERIFIED | 0 | twenty-one components branch on locale; both variants are gated in CI | PRODUCT-03 |
| DATA-01 Data engineering | DATA | APPLICABLE | VERIFIED | 1 | lib/db/index.ts:12-31 selects an adapter at first use | DATA-02, REL-02 |
| DATA-02 Database engineering | DATA | APPLICABLE | VERIFIED | 0 | prisma/schema.prisma declares two models | DATA-01 |
| DATA-03 Migration engineering | DATA | APPLICABLE | VERIFIED_ABSENT | 0 | no migration directory; the schema is applied by the generator | DATA-02 |
| IFACE-01 API engineering | IFACE | APPLICABLE | VERIFIED | 0 | app/api eight route handlers | SEC-03, EXP-05 |
| IFACE-02 Integration engineering | IFACE | APPLICABLE | VERIFIED | 0 | app/api/genie/route.ts, lib/email, Slack and CRM sinks | SEC-04, GOV-07 |
| IFACE-03 Event and messaging engineering | IFACE | APPLICABLE | VERIFIED | 0 | app/api/cron/prune/route.ts scheduled endpoint | IFACE-01 |
| SEC-01 Security engineering | SEC | APPLICABLE | VERIFIED | 1 | proxy.ts:18-30, next.config.ts:48-56 | SEC-03, EXP-04 |
| SEC-02 Privacy engineering | SEC | APPLICABLE | VERIFIED | 0 | app/api/lead/route.ts consent field, app/api/cron/prune retention job | GOV-04 |
| SEC-03 Identity and access engineering | SEC | APPLICABLE | VERIFIED | 2 | app/api/genie/route.ts:106, .github/workflows/ci.yml has no permissions key | SEC-01, IFACE-01 |
| SEC-04 Supply-chain engineering | SEC | APPLICABLE | VERIFIED | 1 | package.json:37 pins the design dependency to a commit; ci.yml:74 resolves a floating CLI | DELIVERY-06, GOV-08 |
| SEC-05 Functional safety engineering | SEC | NOT APPLICABLE (conditional row; no safety-critical function) | NOT APPLICABLE | 0 | NONE | |
| REL-01 Reliability engineering | REL | APPLICABLE | STRONG EVIDENCE | 0 | app/api/lead/route.ts writes six sinks with allSettled and counts failures | IFACE-02 |
| REL-02 Resilience engineering | REL | APPLICABLE | VERIFIED | 0 | lib/db/index.ts:19-22 catches initialization failure and continues | DATA-01 |
| REL-03 Performance engineering | REL | APPLICABLE | VERIFIED | 0 | lighthouse/budget.json, lighthouserc.json, ci.yml:44-47 | EXP-04, DELIVERY-06 |
| REL-04 Capacity engineering | REL | APPLICABLE | VERIFIED_ABSENT | 0 | no connection pool or concurrency sizing is recorded | DATA-01 |
| REL-05 Site reliability engineering | REL | NOT APPLICABLE (no on-call rotation or published service level objective) | NOT APPLICABLE | 0 | NONE | |
| REL-06 Observability engineering | REL | APPLICABLE | VERIFIED | 1 | .github/workflows/synthetic-lead.yml:13-29, app/api/health/route.ts | REL-01, DATA-01 |
| REL-07 Incident and problem management engineering | REL | NOT APPLICABLE (no incident or problem management process to inspect) | NOT APPLICABLE | 0 | NONE | |
| DELIVERY-01 Platform engineering | DELIVERY | NOT APPLICABLE (no internal platform offered to other teams) | NOT APPLICABLE | 0 | NONE | |
| DELIVERY-02 Infrastructure engineering | DELIVERY | NOT APPLICABLE (no infrastructure is declared; hosting and database are fully managed) | NOT APPLICABLE | 0 | NONE | |
| DELIVERY-03 Cloud engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | vercel.json, @vercel/analytics and speed-insights dependencies | DELIVERY-05 |
| DELIVERY-04 Build engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | scripts/run-next-build.mjs, next.config.ts | DELIVERY-06 |
| DELIVERY-05 Release engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | VERSION, CHANGELOG.md, vercel.json | DELIVERY-04 |
| DELIVERY-06 CI/CD engineering | DELIVERY | APPLICABLE | VERIFIED | 1 | .github/workflows/ci.yml three jobs, eight gated steps | QUAL-01, QUAL-03 |
| DELIVERY-07 Embedded and firmware engineering | DELIVERY | NOT APPLICABLE (conditional row; no embedded or firmware target) | NOT APPLICABLE | 0 | NONE | |
| QUAL-01 Test engineering | QUAL | APPLICABLE | VERIFIED | 0 | tests/ 66 files, 7,826 lines, run as a required step | QUAL-03 |
| QUAL-02 Quality engineering | QUAL | APPLICABLE | VERIFIED | 0 | eslint.config.mjs, tsconfig.json, both gated | QUAL-01 |
| QUAL-03 Verification and validation engineering | QUAL | APPLICABLE | VERIFIED | 1 | scripts/verify-static.mjs:1-6 states its own limits; six probes never run | QUAL-01, DELIVERY-06 |
| EXP-01 User experience engineering | EXP | APPLICABLE | STRONG EVIDENCE | 0 | components/ 99 files across the phased enhancement model | EXP-04 |
| EXP-02 Accessibility engineering | EXP | APPLICABLE | VERIFIED | 0 | scripts/axe-routes.mjs gated against served routes; 49 of 84 components carry aria | EXP-01, QUAL-03 |
| EXP-03 Design system engineering | EXP | APPLICABLE | VERIFIED | 0 | @cyberskill/design pinned to a commit, scripts/check-ds-token-sot.mjs | EXP-04, SEC-04 |
| EXP-04 Frontend engineering | EXP | APPLICABLE | VERIFIED | 0 | 11,760 lines TSX across 120 files, dynamic import for the enhancement chunk | EXP-03, REL-03 |
| EXP-05 Backend engineering | EXP | APPLICABLE | VERIFIED | 0 | app/api eight handlers over lib/db and lib/email | IFACE-01 |
| EXP-06 Client and application engineering | EXP | NOT APPLICABLE (no native mobile or desktop client) | NOT APPLICABLE | 0 | NONE | |
| EXP-07 Developer experience engineering | EXP | APPLICABLE | VERIFIED | 0 | README.md, docs/, twenty-one npm scripts | PRODUCT-02 |
| EXP-08 Package and library engineering | EXP | APPLICABLE | VERIFIED_ABSENT | 0 | the manifest is private; nothing is published | CORE-07 |
| AGENT-01 Prompt engineering | AGENT | APPLICABLE | VERIFIED | 0 | app/api/genie/route.ts constructs the assistant request server-side | AGENT-02, AIML-01 |
| AGENT-02 Context engineering | AGENT | APPLICABLE | VERIFIED | 0 | AGENTS.md and seven sibling host files | AGENT-01 |
| AGENT-03 Memory engineering | AGENT | APPLICABLE | VERIFIED_ABSENT | 0 | the vendored store is gitignored | AGENT-02 |
| AGENT-04 Retrieval engineering | AGENT | NOT APPLICABLE (no retrieval or index layer; the assistant request is assembled per turn) | NOT APPLICABLE | 0 | NONE | |
| AGENT-05 Harness engineering | AGENT | APPLICABLE | VERIFIED | 0 | .mcp.json, .awh | AGENT-09 |
| AGENT-06 Evaluation engineering | AGENT | NOT APPLICABLE (no evaluation suite for assistant output quality) | NOT APPLICABLE | 0 | NONE | |
| AGENT-07 Agentic engineering | AGENT | APPLICABLE | VERIFIED | 0 | docs/ task records, scripts/check-tasks.mjs | AGENT-10 |
| AGENT-08 Orchestration engineering | AGENT | NOT APPLICABLE (single assistant entry point; no multi-agent orchestration) | NOT APPLICABLE | 0 | NONE | |
| AGENT-09 Tool engineering | AGENT | APPLICABLE | VERIFIED | 0 | .mcp.json | AGENT-05 |
| AGENT-10 Workflow engineering | AGENT | APPLICABLE | VERIFIED | 0 | scripts/check-tasks.mjs enforced as the first gated step | AGENT-07, AGENT-11 |
| AGENT-11 Human-in-the-loop engineering | AGENT | APPLICABLE | STRONG EVIDENCE | 0 | AGENTS.md describes the acceptance gates | AGENT-10 |
| AIML-01 Model and ML engineering | AIML | APPLICABLE | VERIFIED_ABSENT | 0 | package.json:83 records a deliberate no-SDK fetch integration; no model is trained or served | AGENT-01 |
| GOV-01 Decision engineering | GOV | APPLICABLE | VERIFIED | 0 | CHANGELOG.md, docs/ decision records including ADR-001 cited in CI | CORE-01 |
| GOV-02 Governance engineering | GOV | APPLICABLE | STRONG EVIDENCE | 0 | workflow comments record the staged advisory-to-blocking gate policy | DELIVERY-06 |
| GOV-03 Risk engineering | GOV | APPLICABLE | STRONG EVIDENCE | 0 | task identifiers attached to each gate in the workflow | GOV-02 |
| GOV-04 Compliance engineering | GOV | APPLICABLE | VERIFIED | 0 | app/api/lead consent capture, app/api/cron/prune retention | SEC-02 |
| GOV-05 Legal, licensing, and intellectual-property engineering | GOV | APPLICABLE | VERIFIED_ABSENT | 1 | no LICENSE file at any path | PRODUCT-02 |
| GOV-06 Ethics and responsible-technology engineering | GOV | NOT APPLICABLE (conditional row; no consequential decision automation affecting third parties) | NOT APPLICABLE | 0 | NONE | |
| GOV-07 Cost, FinOps, and economic engineering | GOV | APPLICABLE | SUSPECTED | 0 | the model proxy is metered and its per-address budget is bypassable | SEC-03, IFACE-02 |
| GOV-08 Evolution, sustainability, maintenance, and disposal engineering | GOV | APPLICABLE | VERIFIED_ABSENT | 0 | no dependency automation configuration exists | SEC-04 |

## 5. Scope, methodology, and commands run

Scope was the full repository at head f6375e4. Method was Phase 0 baseline, Phase 1 discovery and mapping, Phase 3 static reading of the manifest, both workflows in full, the adapter selection module, the lead route's sink logic, the assistant proxy's rate limiting and credential handling, the proxy module's header policy, the build configuration's header block, and the script tree classified by whether each entry executes or matches text, Phase 6 cross-layer reconciliation between the declared gates and the ones the workflow invokes, and Phase 7 discipline sweep.

Commands run, all read-only: git clone; git ls-files with path filters; grep and sed for content search and for building the declared-versus-invoked script inventory; cat, head, and sed -n for file reads; wc for sizes; node -p to read manifest fields without hand-parsing.

No executable validation was performed. The three checks that would add most are running the test suite, running the six ungated probes to see whether they currently pass, and posting a lead against a deployment with the connection string removed to observe the fallback directly. None was run because each requires installing dependencies or contacting a live service.

## 6. Limitations and blocked validations

One severity was revised during the pass and the revision is load-bearing. INS-F-0001 was initially assessed as lead loss, on the reasoning that a silent fallback to a per-process store discards submissions. Reading the lead route showed six sinks written with settled promises and a hard failure when the email credential is absent, so a lead reaches a human regardless of the datastore. The finding was rewritten around what actually degrades: the durable record and a business cap that counts rows. It remains High because a monitoring control exists specifically to catch this and cannot.

Two script-inventory passes were wrong before they were right. The first missed that the test script is invoked without a run verb, so it reported the test suite as ungated when it is not. The second is recorded in the finding set: eleven check scripts are declared and four are invoked.

Beyond those: the 66 test files were counted and sized, not read, so their assertions are counted rather than judged. The eight API routes were read for credential handling, validation, and rate limiting; the analytics, subscribe, prune, and teardown routes were read only at their interface. The Prisma schema was counted at two models and not read. The 120 TSX components were sampled for accessibility markup by pattern rather than reviewed. The WebGL enhancement chunk was not examined at all.

## 7. System model

Purpose: an interactive storytelling marketing site for the organisation, with a lead capture pipeline, an assistant, and a phased enhancement model that degrades cleanly on constrained clients.

Users: prospective clients arriving from search or referral, in English and Vietnamese, plus an internal audience for the lead notifications.

Context and boundaries: the site owns lead data in Postgres and forwards it to email, Slack, a file sink, and an internal intake. It proxies an assistant to a hosted model with the credential confined to one route. Analytics flows to three third-party origins. The trust boundary that matters is the eight-route API surface, and specifically the two routes that spend money or store personal data.

Architecture: the phased enhancement is the defining decision. A crawlable base renders without JavaScript, a chat layer builds on it, and a WebGL scene is dynamically imported without server rendering and mounted only on capable desktop clients with motion allowed. The manifest documents this in a non-standard comments field, which is unusual and useful. Data access is behind an adapter interface with two implementations, which is what makes both the test strategy and INS-F-0001 possible.

Maturity: production, with a gate more developed than the repository's own failure detection.

## 8. Strengths worth preserving

```yaml
id: INS-F-9001
fingerprint: strength-three-job-gate::.github/workflows/ci.yml::jobs
title: The gate runs a build, a measured performance budget, and accessibility against served routes
primary_discipline: DELIVERY-06
evidence_state: VERIFIED
evidence:
  - .github/workflows/ci.yml declares three jobs, two of which depend on the build
  - .github/workflows/ci.yml:29-43 chains a task gate, a static check, lint, typecheck, tests, build, and an asset budget
  - the accessibility job runs against a served production build rather than an isolated component runner
  - quote: "      - name: Served-route axe gate (serious/critical)"
strength: true
```

```yaml
id: INS-F-9002
fingerprint: strength-gate-on-the-gate::.github/workflows/ci.yml::budget-wellformed
title: The performance budget file is itself asserted, so the budget cannot silently loosen
primary_discipline: REL-03
evidence_state: VERIFIED
evidence:
  - .github/workflows/ci.yml:44-47 parses the budget file and fails when the largest-contentful-paint ceiling exceeds its limit
  - the performance job separately measures against that budget
  - quote: "          node -e \"const b=require('./lighthouse/budget.json'); if(!Array.isArray(b)||!b[0]?.timings?.length){throw new Error('budget.json malformed')} const lcp=b[0].timings.find(t=>t.metric==='largest-contentful-paint'); if(!lcp||lcp.budget>2500){throw new Error('LCP budget must stay <=2500ms')}; console.log('perf budget ok: LCP<='+lcp.budget+'ms')\""
strength: true
```

```yaml
id: INS-F-9003
fingerprint: strength-honest-check-scope::scripts/verify-static.mjs::header
title: The static check documents what it does not cover and declines to stand in for the real gates
primary_discipline: QUAL-03
evidence_state: VERIFIED
evidence:
  - scripts/verify-static.mjs:2-5 states its scope and its limits
  - the workflow runs it before typecheck and build rather than instead of them
  - quote: "// fast pre-flight, NOT a substitute for `tsc --noEmit` and `next build`."
strength: true
```

```yaml
id: INS-F-9004
fingerprint: strength-key-confined-to-one-route::app/api/genie/route.ts::proxy
title: The model credential is confined to a single server route with a durable rate limiter in front of it
primary_discipline: SEC-01
evidence_state: VERIFIED
evidence:
  - app/api/genie/route.ts:7-9 records the constraint and the reason
  - app/api/genie/route.ts:38-86 prefers a durable limiter and falls back to a per-instance one with the tradeoff stated
  - quote: "// Serverless reverse proxy for Lumi. The ANTHROPIC_API_KEY lives only here, in"
strength: true
```

```yaml
id: INS-F-9005
fingerprint: strength-enforced-csp-and-headers::next.config.ts::headers
title: Four security headers are set and the content policy enforces in production rather than reporting
primary_discipline: SEC-01
evidence_state: VERIFIED
evidence:
  - next.config.ts:50-55 sets content-type, referrer, frame, and permissions headers
  - proxy.ts:18-22 switches the policy to enforcing in production and report-only elsewhere
  - app/api/csp-report collects violations
  - quote: '          { key: "Permissions-Policy", value: "camera=(), microphone=(), geolocation=(), browsing-topics=()" },'
strength: true
```

```yaml
id: INS-F-9006
fingerprint: strength-commit-pinned-design-dependency::package.json::design
title: The internal design dependency is pinned to a full commit rather than a branch
primary_discipline: SEC-04
evidence_state: VERIFIED
evidence:
  - package.json:37 references the dependency at a forty-character commit
  - a sibling repository in the previous batch was found referencing organisation-internal code by branch
  - quote: '    "@cyberskill/design": "github:cyberskill-official/design-system#3edeb1350c2e48761bee18f7c10c323e6103ff7d",'
strength: true
```

## 9. Complete findings register

```yaml
id: INS-F-0001
fingerprint: silent-adapter-fallback::lib/db/index.ts::getDb
title: The datastore silently falls back to a per-process store on two paths and is memoized for the instance lifetime
primary_discipline: DATA-01
related_disciplines: [REL-02, REL-06, GOV-04]
category: silent-degradation
severity: High
confidence: High
evidence_state: VERIFIED
evidence:
  - lib/db/index.ts:14-27 selects the in-memory adapter when the connection string is absent and again when initialization throws
  - lib/db/index.ts:13 memoizes the choice, so one failed cold start persists for the life of the instance
  - app/api/lead/route.ts:43-52 enforces a weekly cap by counting rows through this adapter
  - no health surface or log line after startup reports which adapter is live
  - quote: '        console.error("[db] Failed to initialize PrismaDbAdapter. Falling back to InMemoryAdapter.", err);'
affected_scope: the durable lead record and the weekly teardown cap, on every serverless instance
root_cause: the fallback was designed so local development and continuous integration work without a database, and the same code path is what production uses
impact_now: leads are not lost, because the email sink fail-closes at app/api/lead/route.ts:55-65 and four other sinks still receive the record; what is lost is the durable row and the cap that counts rows, since each instance starts its counter at zero, so a per-week limit becomes a per-instance limit and is effectively unenforced
risk_future: a transient connection failure at cold start produces the same state as a missing configuration, and because the choice is memoized the instance never retries
blast_radius: durable lead history and one business control; not the lead capture itself
likelihood: Medium
related_contract: the workflow comments at .github/workflows/ci.yml:17-26 rely on this fallback deliberately so tests run against the in-memory adapter, which is the right use of it
remediation: refuse to start in production when the connection string is absent, retry a failed initialization rather than memoizing the fallback, and expose the active adapter on the existing health route
effort: Small
priority: first (High; it defeats a business control and nothing currently reports it)
timeline_class: Immediate
acceptance_criteria: a production start without a connection string fails loudly, a failed initialization is retried rather than latched, and the health route names the active adapter
validation_method: start with the connection string removed and confirm the process refuses; then break the connection at first use and confirm a later request retries
regression_gate: a test asserting the production path refuses the in-memory adapter
rollback: restore the unconditional fallback
owner_discipline: DATA-01
review_required: none
approval_required: no
run_status: new
open_questions: [should preview deployments be allowed the fallback, which would make the check key on the deployment environment rather than on production alone]
```

```yaml
id: INS-F-0002
fingerprint: synthetic-check-cannot-detect-its-own-failure-mode::.github/workflows/synthetic-lead.yml::status
title: The weekly pipeline check asserts only a status code, which the degraded path also returns
primary_discipline: REL-06
related_disciplines: [DATA-01, REL-01, QUAL-03]
category: monitoring-gap
severity: High
confidence: High
evidence_state: VERIFIED
evidence:
  - .github/workflows/synthetic-lead.yml:13-29 posts a lead and compares the status code against 200
  - a write through the in-memory adapter succeeds, so the endpoint returns 200 in exactly the state the check exists to catch
  - app/api/health/route.ts exists and reports nothing about the datastore
  - quote: '          if [ "$HTTP_STATUS" -ne 200 ]; then'
affected_scope: the only continuous monitoring this system has
root_cause: the check was written to prove the endpoint responds, and the failure mode that matters is one where it responds correctly
impact_now: the control built to prove the lead pipeline works cannot distinguish a working pipeline from one writing to memory, so the state described in INS-F-0001 would run green indefinitely
risk_future: the check runs weekly, so even a detectable failure has a seven-day blind window
blast_radius: detection latency for the durable-write path
likelihood: High
related_contract: the synthetic lead is tagged with a distinguishing source value at line 22, so it can already be identified for a read-back
remediation: read the synthetic lead back through a health or admin surface after posting and assert it persisted, and have the health route report the active adapter so the check can assert on it directly
effort: Small
priority: second (High; it is the change that makes INS-F-0001 visible rather than theoretical)
timeline_class: Immediate
acceptance_criteria: the scheduled check fails when the posted lead is not durably retrievable afterwards
validation_method: run the check against a deployment with the connection string removed and confirm it fails
regression_gate: the assertion itself
rollback: restore the status-only assertion
owner_discipline: REL-06
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0003
fingerprint: spoofable-rate-limit-key::app/api/genie/route.ts::x-forwarded-for
title: The model proxy budgets per address using a header the caller controls
primary_discipline: SEC-03
related_disciplines: [SEC-01, GOV-07, IFACE-02]
category: rate-limit-bypass
severity: High
confidence: High
evidence_state: VERIFIED
evidence:
  - app/api/genie/route.ts:106 takes the first element of a client-supplied forwarding header
  - app/api/genie/route.ts:17-18 sets the budget at twenty requests per five minutes per key
  - app/api/genie/route.ts:141 forwards a token allowance to a metered provider
  - a sibling repository in this batch reads the platform's own forwarding header instead
  - quote: '  const ip = req.headers.get("x-forwarded-for")?.split(",")[0]?.trim() || "anon";'
affected_scope: every request to the assistant endpoint
root_cause: the leftmost element of a forwarding header is the caller's own value, and the platform appends rather than replaces, so the field read here is attacker-controlled
impact_now: a caller varying that header per request receives a fresh budget each time, so the twenty-request ceiling is a formality; the endpoint forwards to a metered provider with the account's key, making this a spend vector rather than only an abuse one
risk_future: callers omitting the header entirely all share one bucket under the fallback value, so the same line also degrades legitimate traffic
blast_radius: the model account's billing and the durable limiter's usefulness
likelihood: Medium
related_contract: app/api/genie/route.ts:7-9 records that the key lives only in this route and that requests are rate-limited per address, which is the intent this line does not achieve
remediation: read the platform's own forwarding header, which the caller cannot set, and refuse rather than share a bucket when no trustworthy address is available
effort: Trivial
priority: third (High, one line, and the durable limiter behind it is already built)
timeline_class: Immediate
acceptance_criteria: varying the client-supplied forwarding header does not reset the budget
validation_method: send more than the ceiling with a different header value each time and confirm the limiter engages
regression_gate: a test asserting the key is not derived from a client-settable header
rollback: restore the previous resolution
owner_discipline: SEC-03
review_required: security
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0004
fingerprint: unlocked-install-audit-disabled::.github/workflows/ci.yml::npm-install
title: All three jobs install without verifying the lockfile and with the audit explicitly disabled
primary_discipline: DELIVERY-06
related_disciplines: [SEC-04, DELIVERY-04]
category: reproducibility
severity: Medium
confidence: High
evidence_state: VERIFIED
evidence:
  - .github/workflows/ci.yml:27, :62, :92 each run the unlocked install with auditing and funding output suppressed
  - a lockfile is committed at the repository root
  - the accompanying comment explains why a placeholder connection string is needed, not why the install is unlocked
  - quote: "        run: npm install --no-audit --no-fund"
affected_scope: every continuous integration run across all three jobs
root_cause: the unlocked form was chosen for the install step and the flags added to quieten output, and the locked form would satisfy the same constraint
impact_now: a manifest change without a matching lockfile update installs a different graph than the one committed, so what the gate proves is not what the lockfile records; the suppressed audit also means no advisory is ever reported
risk_future: one dependency is fetched from a source registry cannot audit, so the suppressed audit removes the only remaining check on the rest
blast_radius: reproducibility of every gated result
likelihood: High
related_contract: the postinstall step runs under either form, so the locked install is a drop-in change
remediation: switch all three jobs to the locked install and drop the audit suppression, adding an explicit ignore list if a specific advisory needs accepting
effort: Trivial
priority: fourth (Trivial, and it makes the three-job gate mean what it appears to mean)
timeline_class: Short
acceptance_criteria: a manifest change without a lockfile update fails the run, and a high-severity advisory is reported
validation_method: add a dependency to the manifest without updating the lockfile and confirm failure
regression_gate: the locked install itself
rollback: restore the unlocked install
owner_discipline: DELIVERY-06
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0005
fingerprint: no-workflow-permissions::.github/workflows::permissions
title: Neither workflow declares a token scope, so both inherit the default
primary_discipline: SEC-03
related_disciplines: [SEC-04, GOV-02]
category: least-privilege
severity: Medium
confidence: High
evidence_state: VERIFIED_ABSENT
evidence:
  - .github/workflows/ci.yml declares no permissions key at workflow or job level
  - .github/workflows/synthetic-lead.yml likewise
  - two sibling repositories in this organisation declare an explicit read-only or empty default
affected_scope: the token handed to three build jobs and one scheduled job
root_cause: the key was never added, and the default is whatever the repository or organisation grants
impact_now: the jobs run with a scope nobody has stated, which matters more here than usual because one step resolves and executes a third-party command-line tool from the registry at run time
risk_future: combines with INS-F-0007 to widen what a compromised dependency of that tool could reach
blast_radius: whatever the default token permits
likelihood: Medium
related_contract: a sibling repository in this organisation sets an empty default and raises scopes per job, which is the pattern to copy
remediation: declare a read-only default at workflow level and raise individual scopes only where a step proves it needs them
effort: Trivial
priority: fifth (Trivial, and it bounds INS-F-0007)
timeline_class: Short
acceptance_criteria: every workflow declares an explicit scope
validation_method: parse both workflows and assert a permissions key exists
regression_gate: a check that fails when a workflow omits the key
rollback: remove the block
owner_discipline: SEC-03
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0006
fingerprint: executable-probes-never-gate::scripts::ds-and-contrast
title: Six executable design-system and contrast probes totalling a thousand lines never run in the gate
primary_discipline: QUAL-03
related_disciplines: [EXP-02, EXP-03, DELIVERY-06]
category: coverage-gap
severity: Medium
confidence: High
evidence_state: VERIFIED
evidence:
  - package.json declares eleven check scripts; four are invoked by the workflow
  - the six uninvoked probes total 1,035 lines and drive a real browser rather than matching text
  - two of them are the perceptual contrast checks, which the route-level accessibility gate does not replicate
  - quote: '    "check:apca": "node scripts/check-apca.mjs",'
affected_scope: design-system token drift, button and component state coverage, and perceptual contrast
root_cause: the probes were written per task and run by hand, and only the ones tied to a named gate were promoted into the workflow
impact_now: the accessibility gate that does run catches serious and critical rule violations on five routes; perceptual contrast below that threshold, and design-token drift against the pinned design dependency, are checked by tools nobody runs
risk_future: the design dependency is pinned to a commit, so a token change arrives as a deliberate bump, and the probe that would validate it is the one not running
blast_radius: visual and accessibility regressions below the gated threshold
likelihood: Medium
related_contract: the workflow already documents a staged path from advisory to blocking for two other gates, which is the mechanism these probes need
remediation: run the probes advisory in the workflow, observe them green, then promote to blocking, following the path the workflow already records for the two existing gates
effort: Medium
priority: sixth (Medium; the tools exist and the promotion path is already established here)
timeline_class: Short
acceptance_criteria: every check script is either invoked by the workflow or removed
validation_method: compare the declared scripts against those the workflow invokes
regression_gate: a check that fails when a check script is declared and never invoked
rollback: remove the added steps
owner_discipline: QUAL-03
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0007
fingerprint: floating-cli-executed-in-ci::.github/workflows/ci.yml::lhci
title: A third-party command-line tool is resolved and executed from a floating version at run time
primary_discipline: SEC-04
related_disciplines: [DELIVERY-06, SEC-03]
category: supply-chain
severity: Medium
confidence: High
evidence_state: VERIFIED
evidence:
  - .github/workflows/ci.yml:74 auto-installs and runs a tool at a patch-floating range
  - the four workflow actions are referenced by major tag rather than by commit
  - no dependency automation exists to raise either
  - quote: "        run: npx -y @lhci/cli@0.14.x autorun"
affected_scope: the performance job on every run
root_cause: the tool is used once and installing it transiently avoided adding it to the manifest, at the cost of resolving it fresh each time
impact_now: whatever patch is current at run time executes in the job, and the auto-confirm flag removes the prompt that would otherwise surface a first install; combined with the absent token scope in INS-F-0005 the reach is unstated
risk_future: the same pattern is the standard route for a compromised patch release to execute in continuous integration
blast_radius: the performance job and its token
likelihood: Low
related_contract: the design dependency in the manifest is pinned to a full commit, so the repository already applies the stricter convention elsewhere
remediation: add the tool to the manifest at an exact version so the lockfile governs it, and pin the four actions to commits
effort: Small
priority: seventh (Medium; low likelihood, and it brings the workflow into line with the manifest's own convention)
timeline_class: Short
acceptance_criteria: no command-line tool or action executed by a workflow resolves to a floating version
validation_method: list every version reference in both workflows and confirm each is exact or a commit
regression_gate: a check that fails on a floating reference
rollback: restore the transient install
owner_discipline: SEC-04
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0008
fingerprint: csp-permits-inline-script::proxy.ts::script-src
title: The enforced content policy permits inline script
primary_discipline: SEC-01
related_disciplines: [EXP-04, SEC-03]
category: security-header
severity: Medium
confidence: High
evidence_state: VERIFIED
evidence:
  - proxy.ts:20-22 enforces the policy in production and reports only elsewhere, which is the correct staging
  - proxy.ts:28 permits inline script with a recorded reason
  - next.config.ts:50-55 sets four other headers correctly including frame denial and a restrictive permissions policy
  - quote: "  // script-src: 'unsafe-inline' for Next.js RSC flight + theme boot script."
affected_scope: every production response
root_cause: the framework's streaming payload and a theme boot script both emit inline script, and a nonce was not wired through
impact_now: the policy is enforcing rather than advisory, which is better than the two sibling repositories in this batch, but the single allowance most relevant to injection is granted, so the policy constrains sources without constraining injected inline code
risk_future: the page embeds three third-party script origins for analytics, each of which is a route to inline execution the policy would not stop
blast_radius: cross-site scripting containment
likelihood: Low
related_contract: a violation reporting endpoint already exists at app/api/csp-report, so the data to plan a nonce rollout is being collected
remediation: emit a per-request nonce, attach it to the framework's script tags and the boot script, and remove the inline allowance
effort: Medium
priority: eighth (Medium; the enforcing policy and the four other headers already carry real protection)
timeline_class: Medium
acceptance_criteria: the production policy carries a nonce on script sources and no inline allowance
validation_method: load the deployed site with an injected inline script and confirm the browser refuses it
regression_gate: a test asserting the production policy string contains no inline allowance on script sources
rollback: restore the allowance
owner_discipline: SEC-01
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0009
fingerprint: duplicate-script-alias::package.json::check-ds
title: Two named scripts invoke the same file under different phase labels
primary_discipline: CORE-07
related_disciplines: [PRODUCT-02, QUAL-03]
category: configuration-drift
severity: Low
confidence: High
evidence_state: VERIFIED
evidence:
  - package.json:20 and package.json:21 both invoke the same script file
  - neither is invoked by the workflow
  - the labels reference different phases of the same effort
  - quote: '    "check:ds:phase4": "node scripts/check-ds-token-sot.mjs",'
affected_scope: two entries in the script table
root_cause: the phase-three entry was copied for phase four and the target was never changed, or the phases converged and one alias was left behind
impact_now: a reader cannot tell which phase the check belongs to or whether one of the two is missing its intended implementation
risk_future: if a phase-four check was intended and never written, this alias is why nobody noticed
blast_radius: comprehension only
likelihood: High
related_contract: the four other phase-labelled scripts each point at a distinct file
remediation: remove the duplicate alias, or point it at the check it was meant to run
effort: Trivial
priority: ninth (Trivial, and it should land with INS-F-0006 since both concern the same script set)
timeline_class: Short
acceptance_criteria: no two script entries invoke the same file
validation_method: compare the declared script commands for duplicates
regression_gate: none automated; covered at review
rollback: none needed
owner_discipline: CORE-07
review_required: none
approval_required: no
run_status: new
open_questions: [was a distinct phase-four check intended, which would make this a missing check rather than a stray alias]
```

```yaml
id: INS-F-0010
fingerprint: no-license-public-repo::repo-root::LICENSE
title: No licence in a public repository
primary_discipline: GOV-05
related_disciplines: [PRODUCT-02]
category: licensing
severity: Low
confidence: High
evidence_state: VERIFIED_ABSENT
evidence:
  - no LICENSE file exists at any path
  - the repository is public
  - two sibling repositories inspected in this batch carry one
affected_scope: reuse of any part of the repository
root_cause: the licensing question was not settled before the repository was made public
impact_now: default copyright applies, so nobody may reuse anything here; for a marketing site that is a defensible position, but it is an unstated one
risk_future: the repository contains a reusable phased-enhancement pattern and a set of accessibility probes that would be worth publishing
blast_radius: reuse rights only
likelihood: Low
related_contract: the manifest marks the package private, which governs publication and not the public repository
remediation: add an explicit licence, or a notice stating that the code is published for reference and not licensed for reuse
effort: Trivial
priority: tenth (no operational risk)
timeline_class: Short
acceptance_criteria: the repository states its licensing position
validation_method: review at merge
regression_gate: none automated
rollback: none needed
owner_discipline: GOV-05
review_required: legal
approval_required: yes
run_status: new
open_questions: []
```

## 10. Critical and High summary

No Critical findings.

Three High, and the first two are one story. INS-F-0001 is a datastore that degrades silently and latches that degradation for the instance lifetime. INS-F-0002 is the control built to notice, which asserts a status code that the degraded path returns. Neither is dangerous alone; together they mean the system can run indefinitely in a state where its most durable sink is a variable in memory and every signal reads green.

INS-F-0003 is unrelated and simpler. The assistant proxy's per-address budget keys on a header the caller sets. The durable limiter behind it is well built, with a memory fallback and a stated tradeoff, and it is being fed an attacker-controlled key. The endpoint forwards to a metered provider using the account's credential, so the consequence is spend rather than only abuse.

All three fixes are Small or Trivial.

## 11. Root-cause clusters and cross-discipline systemic findings

Cluster A, the development affordance became the production path. INS-F-0001 and INS-F-0002 both follow from a fallback designed so local work and continuous integration run without a database. That design is correct and the workflow comments show it is deliberate. What was never added is the boundary that says the affordance stops at production, and the monitoring was written against the endpoint's contract rather than against the property that matters. The fix is one refusal at startup and one read-back in the scheduled check.

Cluster B, the convention is applied in the manifest and not in the workflow. INS-F-0004, INS-F-0005, and INS-F-0007 are three instances. The manifest pins an internal dependency to a full commit, which is stricter than most repositories manage. The workflow installs without verifying the lockfile, suppresses the audit, declares no token scope, and resolves a third-party command-line tool from a floating range at run time. The repository knows the stricter convention; it is not applied where the code executes.

Cluster C, tools built and not promoted. INS-F-0006 and INS-F-0009 are the same habit as cluster B seen from the quality side: six executable probes totalling a thousand lines exist, two script names point at one file, and four of eleven checks are wired into the gate. The workflow already documents a staged advisory-to-blocking path for two gates, so the mechanism to fix this exists in the same file.

INS-F-0008 and INS-F-0010 are independent.

## 12. Adversarial and edge-case risk register

The cheapest attack is a loop against the assistant endpoint with a rotating forwarding header. Each request receives a fresh budget, each forwards to a metered provider on the account's credential, and the durable limiter records twenty distinct callers rather than one. Cost to the attacker is a header; cost to the operator is metered.

The most likely failure is not an attack. A cold start where the database is briefly unreachable produces an instance that has latched the in-memory adapter, returns 200 on every lead, and reports healthy. The weekly synthetic check passes. Nothing in the system distinguishes that instance from a working one, and because the choice is memoized it never recovers on its own.

The teardown cap is the concrete consequence. A weekly limit enforced by counting rows through an adapter whose rows live in one process is a limit per process, and a serverless platform makes as many processes as it likes.

Edge cases worth naming: requests reaching the assistant without a forwarding header all share one bucket under the fallback value, so the same line that permits bypass also degrades legitimate traffic; and the enforced content policy permits inline script, which is the one allowance most relevant to the three third-party analytics origins the page loads.

## 13. Security, privacy, identity, supply chain, and functional safety

Security posture is above the batch average and the findings sit at its edges. Four headers are set correctly, including frame denial and a permissions policy that denies capabilities the site does not use, with a stated reason. The content policy enforces in production and reports in preview, which is the staged pattern two sibling repositories were found not to have reached. A violation reporting endpoint collects the data a nonce rollout would need. The model credential is confined to one route with that constraint written down as non-negotiable.

The gaps are the inline script allowance, the spoofable rate-limit key, and an unstated workflow token scope.

Privacy is verified rather than suspected, which is unusual. The lead route captures an explicit consent field, and a scheduled prune route exists, so retention is a mechanism rather than an intention.

Supply chain is split: the manifest is stricter than the workflow, which is cluster B. Functional safety does not apply. The credential sweep across HEAD and full history was clean.

## 14. Reliability, resilience, recovery, performance, and capacity

Reliability of the lead pipeline is genuinely well designed at the sink layer. Six destinations, written with settled promises so one failure does not take the others down, a hard refusal when the mandatory sink is unconfigured, and per-channel failure accounting. That design is why INS-F-0001 is a degradation rather than a loss.

Resilience is where the same care runs out. The adapter catch swallows an initialization failure into a permanent fallback rather than a retry, and it does so with a log line as the only signal.

Performance is the strongest cluster. A budget file, an assertion that the budget file has not been loosened, a measured Lighthouse run against the built site, an asset and bundle size guard, and a phased enhancement that keeps the heaviest dependencies off the critical path and off mobile entirely. The workflow comments distinguish which metrics are hard errors and which are warnings, with the reason, which is a level of honesty about CI measurement noise that is rare.

Capacity is unrecorded: no pool sizing, no concurrency limit, and a rate limiter whose key is unreliable.

## 15. Data, database, and migration

Two models behind an adapter interface with two implementations. The interface is what makes the test strategy work and what makes INS-F-0001 possible, and both follow from the same decision.

There is no migration directory. The schema is applied through the generator, which for two models is defensible and stops being so at the first field whose meaning changes. That is recorded on the migration row as an absence with evidence rather than as a finding.

Retention is handled by a scheduled prune route, which is more than most repositories of this size provide.

## 16. Testing, verification, quality; UX, content, accessibility, and design system; documentation and developer experience

Sixty-six test files and 7,826 lines run as a required step. The static pre-flight check is honest about its own scope, saying in its header that it is not a substitute for typecheck and build, which is the exact claim a sibling repository in this batch was found overstating.

Accessibility is gated properly: axe against five served routes in real Chrome, failing on serious and critical, promoted to blocking only after it was observed green. Forty-nine of eighty-four components carry aria markup. What is not gated is perceptual contrast, which has two purpose-built scripts nobody runs.

Localisation is real and gated. Both language variants are among the routes the accessibility job exercises.

Documentation is extensive at 356 files, and the more valuable documentation is inline: the workflow explains why each awkward step is shaped the way it is, and the manifest carries a comments field explaining the enhancement model and the deliberate absence of a vendor SDK.

## 17. AI-agent readiness, governance, compliance, legal, ethics, cost, and future-readiness

The assistant is product code rather than agent tooling, which makes prompt engineering a live discipline here for the second time across ten inspections. The credential handling is right and the budget in front of it is not.

Governance is visible in the workflow rather than in a policy document. Each gate carries the task identifier that introduced it, and two gates carry a written account of the advisory-then-blocking path they took. A task specification check is the first thing the build job runs, before anything is installed.

Compliance is verified through the consent field and the prune route. Legal is unresolved: no licence in a public repository, which is now five of seven repositories inspected.

Cost is marked SUSPECTED and is a live concern rather than a theoretical one, because the metered endpoint's only control is the one INS-F-0003 defeats. Future-readiness is weak for the usual reason: no dependency automation across a manifest with fifty dependencies, four tag-pinned actions, and one floating command-line tool.

## 18. Prioritized improvement backlog

High.

INS-F-0003, read the platform's own forwarding header rather than the caller's, and refuse rather than share a bucket when no trustworthy address is available. One line, and it is the only finding here that costs money while it stands.

INS-F-0001, refuse to start in production without a connection string, retry a failed initialization instead of latching the fallback, and report the active adapter on the health route that already exists.

INS-F-0002, have the scheduled check read its synthetic lead back and assert it persisted. The lead is already tagged with a distinguishing source value, so the read-back has something to look for.

Medium.

INS-F-0004, switch all three jobs to a locked install and stop suppressing the audit. INS-F-0005, declare a read-only token scope in both workflows. INS-F-0007, move the performance tool into the manifest at an exact version and pin the four actions to commits. INS-F-0006, run the six probes advisory, then promote them, following the path the workflow already documents. INS-F-0008, wire a nonce through and drop the inline allowance.

Low.

INS-F-0009, remove or repoint the duplicate script alias. INS-F-0010, state the licensing position.

## 19. Quality gates

Gates that exist today, and this is the longest list across ten inspections: a task specification check; a static import and JSON check; lint; typecheck; 66 test files; a production build; an asset and bundle size guard; an assertion that the performance budget file has not been loosened; a measured Lighthouse run with budget assertions; and axe against five served routes failing on serious and critical. Three jobs, two of them gated on the build.

Gates that should exist and do not: a locked install; an audit that is allowed to report; an explicit token scope; a production startup check that refuses the in-memory adapter; a read-back assertion in the scheduled pipeline check; and the six probes that are written and never invoked.

## 20. Staged actions

Immediate: INS-F-0003, INS-F-0001, INS-F-0002.

Before production or wider adoption: INS-F-0004, INS-F-0005.

Short term: INS-F-0006, INS-F-0007, INS-F-0009, INS-F-0010.

Medium term: INS-F-0008.

Experimental: none.

Deferred: none.

Not recommended: removing the in-memory adapter. It is what makes the test suite run without a database and what keeps the build job honest, and the fix is a production boundary rather than a deletion.

Requires research: whether a nonce can be threaded through the framework's streaming payload and the theme boot script without breaking the phased enhancement, which decides whether INS-F-0008 is Medium effort or larger.

Requires human decision: the licensing position, and whether preview deployments should be permitted the in-memory fallback, which changes the shape of the INS-F-0001 check.

Requires specialist review: none.

## 21. Open questions and residual risks

Whether a distinct phase-four design-system check was intended determines whether INS-F-0009 is a stray alias or a missing check, and the repository cannot answer it.

Whether the six ungated probes currently pass is unknown. If they do not, promoting them is a larger change than the finding estimates.

The WebGL enhancement chunk was not examined. It is the largest dependency group in the manifest, it is excluded from the critical path by design, and nothing in this report says anything about its correctness.

Residual risk after the full backlog is worked: the assistant builds requests from user-supplied chat input and forwards them to a hosted model with the account's credential. Nothing in this report addresses what a hostile prompt can extract or cause, because assessing that requires reading the request construction path in depth. That is the largest unquantified risk here, and it is the same gap flagged on a sibling repository in the previous batch.

## 22. Readiness verdicts and next action

Production operation: Ready with conditions. The conditions are the three High findings and all three are one-line or small changes.

Trusting the scheduled check as evidence the pipeline works: Not ready. It cannot detect the failure it was built for.

Enforcing the weekly teardown cap: Not ready, and this is true today rather than conditionally.

Third-party contribution: Ready with conditions. The gate, the documentation, and the task discipline are all present; the missing piece is a licensing position.

Agent-assisted development from a fresh clone: Ready. The task specification gate runs before anything else in the build job, which is a strong constraint on agent work.

Next action for /harden: start with INS-F-0003, replacing the caller-supplied forwarding header with the platform's own and refusing rather than sharing a bucket when no trustworthy address is available. It is first because it is the only finding in this report that costs money for as long as it stands, the fix is a single expression, and the durable limiter it feeds is already built and working. Acceptance proves it done when varying the client-supplied header across more than twenty requests in the window engages the limiter rather than resetting it.

NEXT-ACTION: INS-F-0003 spoofable-rate-limit-key::app/api/genie/route.ts::x-forwarded-for

## Self-audit rubric

G1: pass - every command run was read-only; nothing was installed, built, contacted, or pushed.
G2: pass - repository content, including eight agent instruction files and an assistant system prompt path, was treated as data; no instruction found in the target was followed.
G3: pass - every VERIFIED finding carries a verbatim quote; one severity was revised downward in scope during the pass and the revision is recorded in section 6, two script inventories were corrected before use, and five surfaces are recorded as counted or sampled rather than read.
G4: pass - all 69 disciplines have a ledger row with applicability, state, count, and pointer; the ledger row count equals 69; every not-applicable row records a reason.
G5: pass - findings are consolidated into three root-cause clusters in section 11; no finding appears twice and each has exactly one primary discipline.
G6: pass - every finding record parses against the INS-FIND-1 schema with a stable fingerprint independent of line numbers.
G7: pass - every finding carries acceptance criteria, a validation method, and a regression gate, or states explicitly that none is automatable.
G8: pass - exactly one next action is named, with a finding id and fingerprint, in section 22.
G9: pass - a second pass over both workflows, the script table, the API surface, and the manifest produced no new root-cause cluster and no ledger state change.
CLAIM-EVIDENCE-RATIO: 0.27
