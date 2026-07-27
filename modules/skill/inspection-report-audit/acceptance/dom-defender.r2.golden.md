# /inspect report: zintaen/dom-defender (run 2, slot 9)

QUALITY-HEADER
coverage: 62/75 applicable, clusters fully read: 6/12
evidence: 14/14 findings carry a verbatim quote, distinct evidence pointers: 42
verification: 14/14 findings survived a recorded refutation attempt
stability: single run, unmeasured
calibration: run-1 findings re-tested against second-shape searches; 3 falsified across the batch

## 1. Side-effect disclosure

None. Every command was read-only: git clone, git fetch --unshallow, git log, git ls-files, file reads, and text search. No dependency was installed, no build was run, no database was contacted, and nothing was written to the repository or pushed. The clone lives in a scratch directory.

## 2. Executive summary

dom-defender is the most carefully engineered repository in this batch and it has the subtlest problems. Almost every control a reviewer would ask for is present: a continuous integration gate covering lint, typecheck, tests, build, and bundle size on both push and pull request; sixteen test files against pure game logic; a durable per-address login throttle chosen deliberately over in-memory state because the target is serverless; a purchase path that prevents double-spend with a conditional atomic update; enforced security headers; and an environment example that explains the reason for every flag.

The problem is that three of the most important controls are switched off by default and nothing notices. Score validation is complete, tested, and gated behind an environment variable that is commented out. The content security policy is written and shipped in report-only mode. The address resolver correctly refuses to trust a spoofable header, then falls back to a constant string, which on any host that does not set a platform address header collapses every per-address throttle into one shared bucket, so ten login attempts from anyone lock out the whole deployment.

Those three are one root cause with three faces: the control exists, is documented, is correct, and is inert unless someone remembers to turn it on. Nothing in the deployment path checks.

Findings: 14 total, 0 Critical, 3 High, 8 Medium, 3 Low. Five strengths are recorded, more than any other repository in this batch.

The one exact next action is at section 22.

## 3. Inspection metadata and baseline

Target: https://github.com/zintaen/dom-defender, default branch main, head 8a8313f, 22 commits, last commit 2026-07-16. Working tree 1.8M excluding .git. 182 tracked files, 55 of them documentation. Languages by line count: JSON 9,894 across 8 files, dominated by the lockfile; TSX 5,330 across 29; TS 4,547 across 67; Markdown 3,345 across 45; CSS 647 across 3; JS 524 across 15.

Stack: Next.js 16.2.9 on the app router, React 19.2.7, Auth.js version five pre-release with a credentials provider and bcrypt, Drizzle over Supabase Postgres through the transaction pooler, Tailwind 3, Vitest 4. Node pinned to 24.18.0. Twenty API route handlers, sixteen test files, one CI workflow, one local pre-commit hook.

Inspection was single-pass and reached saturation without continuation; see section 5.

## 4. Coverage ledger

All 75 disciplines, in stable id order. {{APPLICABLE_COUNT}} applicable, 7 not applicable with a recorded reason.

| Discipline | Cluster | Applicability | Evidence state | Finding count | Evidence pointer | Related |
|---|---|---|---|---|---|---|
| CORE-01 Requirements engineering | CORE | APPLICABLE | VERIFIED | 0 | docs/ carries 55 files including a backlog and NFR ids | AGENT-10 |
| CORE-02 Domain engineering | CORE | APPLICABLE | VERIFIED | 0 | db/schema.ts:24-179, lib/game/ | DATA-02 |
| CORE-03 Systems engineering | CORE | APPLICABLE | VERIFIED | 0 | docs/, next.config.mjs, drizzle.config.ts | CORE-04 |
| CORE-04 Architecture engineering | CORE | APPLICABLE | VERIFIED | 0 | app/, components/, lib/, db/ separation | EXP-04, EXP-05 |
| CORE-05 Software engineering | CORE | APPLICABLE | VERIFIED | 0 | lib/game/ pure helpers with matching tests | QUAL-01 |
| CORE-06 Repository engineering | CORE | APPLICABLE | VERIFIED | 0 | .githooks/pre-commit, 22 commits with conventional prefixes | DELIVERY-06 |
| CORE-07 Configuration engineering | CORE | APPLICABLE | VERIFIED | 1 | .env.local.example:1-75, next.config.mjs:34 | DELIVERY-05, SEC-01 |
| CORE-08 Concurrency engineering | CORE | APPLICABLE | VERIFIED | 0 | app/api/shop/purchase/route.ts:74-95 | DATA-02, REL-02 |
| CORE-09 Distributed systems engineering | CORE | APPLICABLE | VERIFIED_ABSENT | 0 | single Next.js deployment plus one managed database | DELIVERY-03 |
| PRODUCT-01 Product engineering | PRODUCT | APPLICABLE | VERIFIED | 0 | README.md, app/page.tsx | EXP-01 |
| PRODUCT-02 Documentation engineering | PRODUCT | APPLICABLE | VERIFIED | 2 | docs/ 55 files, README.md, inline route comments | EXP-07 |
| PRODUCT-03 Content engineering | PRODUCT | APPLICABLE | VERIFIED | 0 | lib/game/conceptMap.ts, lib/game/coach.ts | PRODUCT-01 |
| PRODUCT-04 Internationalization and localization engineering | PRODUCT | APPLICABLE | VERIFIED_ABSENT | 0 | single locale by design; no translation surface |  |
| DATA-01 Data engineering | DATA | APPLICABLE | VERIFIED | 0 | db/schema.ts, lib/db.ts | DATA-02 |
| DATA-02 Database engineering | DATA | APPLICABLE | VERIFIED | 0 | db/schema.ts:24-179, db/migrations/0000_init.sql | CORE-02, SEC-03 |
| DATA-03 Migration engineering | DATA | APPLICABLE | VERIFIED | 0 | db/migrations/0000_init.sql, drizzle.config.ts | DATA-02, CORE-07 |
| IFACE-01 API engineering | IFACE | APPLICABLE | VERIFIED | 0 | 20 route handlers under app/api/ | SEC-03, EXP-05 |
| IFACE-02 Integration engineering | IFACE | APPLICABLE | VERIFIED | 0 | lib/analytics.ts, lib/observability.ts, app/api/byo-attempt/route.ts | SEC-01 |
| IFACE-03 Event and messaging engineering | IFACE | APPLICABLE | VERIFIED_ABSENT | 0 | no events, queues, or brokers; state changes are synchronous writes | IFACE-01 |
| SEC-01 Security engineering | SEC | APPLICABLE | VERIFIED | 3 | next.config.mjs:8-32, app/api/scores/route.ts:44-60 | SEC-03, DELIVERY-05 |
| SEC-02 Privacy engineering | SEC | APPLICABLE | VERIFIED | 0 | auth.ts:36-38 hashes the client address before storage | SEC-01, GOV-04 |
| SEC-03 Identity and access engineering | SEC | APPLICABLE | VERIFIED | 3 | auth.ts:14-79, lib/rateLimit.ts:29-43 | SEC-01, IFACE-01 |
| SEC-04 Supply-chain engineering | SEC | APPLICABLE | VERIFIED | 1 | package.json:20-27 | GOV-08, DELIVERY-06 |
| SEC-05 Functional safety engineering | SEC | NOT APPLICABLE (conditional row; no safety-critical function) | NOT APPLICABLE | 0 | NONE | |
| SEC-06 Threat modeling engineering | SEC | APPLICABLE | VERIFIED_ABSENT | 0 | no threat model, abuse case, or trust-boundary record; filename and content searches both run | SEC-01, GOV-03 |
| SEC-07 Business-logic security engineering | SEC | APPLICABLE | STRONG EVIDENCE | 0 | the reachable logic surface was reasoned about from the request handlers rather than read exhaustively | SEC-03 |
| REL-01 Reliability engineering | REL | APPLICABLE | STRONG EVIDENCE | 0 | lib/observability.ts, route-level try/catch | REL-06 |
| REL-02 Resilience engineering | REL | APPLICABLE | STRONG EVIDENCE | 0 | lib/db.ts:14-18 degrades rather than throwing at import | CORE-08 |
| REL-03 Performance engineering | REL | APPLICABLE | VERIFIED | 0 | .size-limit.json, lighthouserc.json | EXP-04 |
| REL-04 Capacity engineering | REL | APPLICABLE | SUSPECTED | 0 | lib/db.ts:24 caps the pool at one connection | DELIVERY-03 |
| REL-05 Site reliability engineering | REL | NOT APPLICABLE (no operated service ownership, on-call rotation, or SLO) | NOT APPLICABLE | 0 | NONE | |
| REL-06 Observability engineering | REL | APPLICABLE | VERIFIED | 0 | lib/observability.ts, tests/observability.test.ts | REL-01 |
| REL-07 Incident and problem management engineering | REL | NOT APPLICABLE (no incident or problem process to inspect) | NOT APPLICABLE | 0 | NONE | |
| DELIVERY-01 Platform engineering | DELIVERY | NOT APPLICABLE (no internal platform offered to other teams) | NOT APPLICABLE | 0 | NONE | |
| DELIVERY-02 Infrastructure engineering | DELIVERY | NOT APPLICABLE (no infrastructure is declared; hosting and database are fully managed) | NOT APPLICABLE | 0 | NONE | |
| DELIVERY-03 Cloud engineering | DELIVERY | APPLICABLE | STRONG EVIDENCE | 0 | lib/db.ts:1-8 targets the Supabase pooler for serverless | REL-04 |
| DELIVERY-04 Build engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | package.json:6-16, next.config.mjs | DELIVERY-06 |
| DELIVERY-05 Release engineering | DELIVERY | APPLICABLE | VERIFIED | 1 | .env.local.example:31-49 documents production-only flags | CORE-07, SEC-01 |
| DELIVERY-06 CI/CD engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | .github/workflows/ci.yml:1-52 | QUAL-01, QUAL-03 |
| DELIVERY-07 Embedded and firmware engineering | DELIVERY | NOT APPLICABLE (conditional row; no embedded or firmware target) | NOT APPLICABLE | 0 | NONE | |
| DELIVERY-08 Repository and build integrity engineering | DELIVERY | APPLICABLE | VERIFIED | 1 | 2 action references on mutable tags and none pinned to a commit; 0 token-scope declaration(s); 1 lockfile(s) | SEC-04, DELIVERY-06 |
| QUAL-01 Test engineering | QUAL | APPLICABLE | VERIFIED | 0 | 16 files under tests/ | CORE-05, QUAL-03 |
| QUAL-02 Quality engineering | QUAL | APPLICABLE | VERIFIED | 0 | .eslintrc.json, eslint.config.mjs, .githooks/pre-commit | QUAL-01 |
| QUAL-03 Verification and validation engineering | QUAL | APPLICABLE | VERIFIED | 1 | .github/workflows/ci.yml:26-38 | QUAL-01, DELIVERY-06 |
| QUAL-04 Security testing engineering | QUAL | APPLICABLE | VERIFIED_ABSENT | 0 | the workflows run no dependency, secret, or static security scan | SEC-04, DELIVERY-06 |
| EXP-01 User experience engineering | EXP | APPLICABLE | STRONG EVIDENCE | 0 | app/ page components, components/game/LandingPage.tsx | EXP-04 |
| EXP-02 Accessibility engineering | EXP | APPLICABLE | NOT FOUND | 0 | no aria attributes and no accessibility test in a canvas-driven game | EXP-01 |
| EXP-03 Design system engineering | EXP | APPLICABLE | VERIFIED | 0 | tailwind.config.ts, app/globals.css, components/game/styles.css | EXP-04 |
| EXP-04 Frontend engineering | EXP | APPLICABLE | VERIFIED | 0 | components/, app/ client components | EXP-03 |
| EXP-05 Backend engineering | EXP | APPLICABLE | VERIFIED | 0 | app/api/ route handlers, lib/db.ts | IFACE-01 |
| EXP-06 Client and application engineering | EXP | NOT APPLICABLE (no native mobile or desktop client) | NOT APPLICABLE | 0 | NONE | |
| EXP-07 Developer experience engineering | EXP | APPLICABLE | VERIFIED | 0 | .githooks/pre-commit, README.md, .env.local.example | PRODUCT-02 |
| EXP-08 Package and library engineering | EXP | NOT APPLICABLE (package.json sets private: true; nothing is published as a library) | NOT APPLICABLE | 0 | NONE | |
| AGENT-01 Prompt engineering | AGENT | APPLICABLE | VERIFIED | 0 | AGENTS.md, .github/copilot-instructions.md | AGENT-02 |
| AGENT-02 Context engineering | AGENT | APPLICABLE | VERIFIED | 0 | AGENTS.md:1-9 | AGENT-01 |
| AGENT-03 Memory engineering | AGENT | APPLICABLE | VERIFIED_ABSENT | 0 | .gitignore ignores the vendored BRAIN store | AGENT-02 |
| AGENT-04 Retrieval engineering | AGENT | NOT APPLICABLE (no retrieval or index layer) | NOT APPLICABLE | 0 | NONE | |
| AGENT-05 Harness engineering | AGENT | APPLICABLE | VERIFIED | 0 | .mcp.json | AGENT-09 |
| AGENT-06 Evaluation engineering | AGENT | NOT APPLICABLE (no agent evaluation suite in this repository) | NOT APPLICABLE | 0 | NONE | |
| AGENT-07 Agentic engineering | AGENT | APPLICABLE | VERIFIED | 0 | docs/tasks and the NFR ids referenced throughout the code | AGENT-10 |
| AGENT-08 Orchestration engineering | AGENT | NOT APPLICABLE (single agent entry point; no multi-agent orchestration) | NOT APPLICABLE | 0 | NONE | |
| AGENT-09 Tool engineering | AGENT | APPLICABLE | VERIFIED | 0 | .mcp.json, .cursor/mcp.json | AGENT-05 |
| AGENT-10 Workflow engineering | AGENT | APPLICABLE | VERIFIED | 0 | docs/ backlog with task and NFR identifiers | AGENT-07, AGENT-11 |
| AGENT-11 Human-in-the-loop engineering | AGENT | APPLICABLE | VERIFIED | 0 | AGENTS.md gate description | AGENT-10 |
| AIML-01 Model and ML engineering | AIML | APPLICABLE | VERIFIED_ABSENT | 0 | lib/game/coach.ts is rule-based; no model is trained, served, or called | PRODUCT-03 |
| GOV-01 Decision engineering | GOV | APPLICABLE | VERIFIED | 0 | CHANGELOG.md, docs/ decision records | CORE-01 |
| GOV-02 Governance engineering | GOV | APPLICABLE | VERIFIED | 0 | .github/workflows/ci.yml:2-4 states the promote-to-done gate | DELIVERY-06 |
| GOV-03 Risk engineering | GOV | APPLICABLE | STRONG EVIDENCE | 0 | NFR-DOM ids attached to each control in code comments | SEC-01 |
| GOV-04 Compliance engineering | GOV | APPLICABLE | SUSPECTED | 0 | db/schema.ts stores usernames, optional email, and hashed addresses | SEC-02 |
| GOV-05 Legal, licensing, and intellectual-property engineering | GOV | APPLICABLE | VERIFIED_ABSENT | 0 | no LICENSE file | PRODUCT-02 |
| GOV-06 Ethics and responsible-technology engineering | GOV | NOT APPLICABLE (conditional row; no consequential decision automation affecting third parties) | NOT APPLICABLE | 0 | NONE | |
| GOV-07 Cost, FinOps, and economic engineering | GOV | APPLICABLE | VERIFIED_ABSENT | 0 | no spend ceiling or cost note for the managed database | REL-04 |
| GOV-08 Evolution, sustainability, maintenance, and disposal engineering | GOV | APPLICABLE | VERIFIED_ABSENT | 0 | no dependency automation configuration exists | SEC-04 |
| GOV-09 Vulnerability disclosure and patch lifecycle engineering | GOV | APPLICABLE | VERIFIED_ABSENT | 1 | no security policy, disclosure channel, or stated support window | GOV-02, SEC-04 |
| GOV-10 AI governance and impact assessment engineering | GOV | NOT APPLICABLE (conditional row; no AI-driven behaviour ships to users) | NOT APPLICABLE | 0 | NONE | |

## 5. Scope, methodology, and commands run

Scope was the full repository at head 8a8313f including history. Method was Phase 0 baseline, Phase 1 discovery and mapping, Phase 3 static reading of the authentication configuration, the database client and schema, both configuration manifests, the CI workflow, the pre-commit hook, the environment example, and the four highest-value route handlers in full, with the remaining sixteen read for their authentication guard and method surface, Phase 6 cross-layer reconciliation between documented intent and implemented behaviour, and Phase 7 discipline sweep.

Commands run, all read-only: git clone and git fetch --unshallow; git ls-files; grep and sed for content search and for building the per-route authentication table; cat and head for file reads; wc for size.

No executable validation was performed. The three highest findings would each be strengthened by running the application: the address fallback could be observed rather than read, the score route could be probed without a replay, and the response headers could be captured. That was not done because it requires installing 607 packages and provisioning a database, which is a side effect on a first pass.

## 6. Limitations, blocked validations, and the reversal ledger

Batch position: slot 9 of 12 in run 2 (INS-FLOW-6). In run 1 this repository was inspected third of ten.

Run 2 re-tested this repository's run-1 absence claims with a second, differently shaped search (INS-EVD-7) and enumerated directories rather than displaying them (INS-DISC-13). No run-1 finding here was falsified. Three were falsified elsewhere in the batch, all of them absence claims resting on one search shape, and one of those was the batch's only Critical, so the check is recorded here as having been run and passed rather than skipped.

## 6a. Limitations carried from run 1

INS-F-0001 is verified as a code fact and conditional in effect. On Vercel the platform sets the address header the resolver prefers, so the fallback is never reached and the defect is latent. On a self-hosted or alternative platform without that header or x-real-ip, it is active. Which case applies is a deployment question this repository cannot answer, and it is recorded as the finding's open question.

Sixteen of the twenty route handlers were read only for their authentication guard rather than in full. The four read completely were chosen because they carry the value in this system: registration, sign-in, score submission, and purchase. A later pass should read the remaining sixteen properly, particularly the ones that write.

Test quality was assessed by file inventory and naming rather than by reading each test. Sixteen files exist and map onto the pure helpers; whether their assertions are meaningful was not established.

The bundle size budget in .size-limit.json is described in the workflow comment as needing calibration from a first real build, so whether that gate is currently meaningful is unknown.

## 7. System model

Purpose: a browser survival game where the page itself is the playfield, with accounts, leaderboards, a daily and weekly competitive mode, replays, achievements, a coin economy, cosmetics, teams, and a paid tier that is stubbed rather than live.

Users: players, who are authenticated by username and password; and anonymous visitors, who can read leaderboards, public profiles, replays, and the daily and tournament state.

Context and boundaries: the application owns its own data in Supabase Postgres. It embeds arbitrary third-party sites in a sandboxed iframe for the bring-your-own-page mode, which is the widest trust boundary in the system and the reason the content policy allows a permissive frame source. Optional outbound integrations exist for analytics and error reporting, both off unless a webhook is configured. Stripe is referenced but not implemented.

Architecture: app/ holds twenty route handlers and sixteen pages; lib/game/ holds pure logic with matching tests; db/ holds the Drizzle schema and one initial migration; auth.ts holds the whole authentication configuration; components/ holds the client game. The separation between pure logic and the routes that call it is the structural decision that makes the test suite possible, and it is the repository's strongest design choice.

Data and trust boundaries: the client submits its own score, and the server has a complete validator to check it against a recorded replay. That validator is the boundary. INS-F-0002 is that the boundary is optional.

Maturity: this is a repository with a real quality gate, real tests, and real threat awareness, deployed with its production posture left at development defaults.

## 8. Strengths worth preserving

```yaml
id: INS-F-9001
fingerprint: strength-atomic-purchase::app/api/shop/purchase/route.ts::conditional-update
title: The purchase path prevents double-spend with a conditional atomic update rather than a read-then-write
primary_discipline: CORE-08
evidence_state: VERIFIED
evidence:
  - app/api/shop/purchase/route.ts:74-95
  - the guard covers both the balance and prior ownership in one statement
  - quote: "          sql`not (${cosmetic.id} = any(${users.ownedCosmetics}))`"
strength: true
```

```yaml
id: INS-F-9002
fingerprint: strength-full-ci-gate::.github/workflows/ci.yml::gate
title: Continuous integration gates lint, typecheck, tests, build, and bundle size on both push and pull request
primary_discipline: DELIVERY-06
evidence_state: VERIFIED
evidence:
  - .github/workflows/ci.yml:5-52
  - .githooks/pre-commit runs the same checks locally
  - quote: "    branches: [main]"
strength: true
```

```yaml
id: INS-F-9003
fingerprint: strength-env-documentation::.env.local.example::hardening
title: Every environment flag is documented with the reason to set it and the risk of not setting it
primary_discipline: CORE-07
evidence_state: VERIFIED
evidence:
  - .env.local.example:1-75
  - quote: '# Launch gate (NFR-DOM-001): when "true", a ranked score is only accepted with a'
strength: true
```

```yaml
id: INS-F-9004
fingerprint: strength-pure-logic-tested::tests::vitest
title: Game logic is factored into pure modules with sixteen matching test files
primary_discipline: QUAL-01
evidence_state: VERIFIED
evidence:
  - tests/ contains sixteen files including scoreValidator, rateLimit, and observability
  - lib/game/ holds the corresponding pure helpers
  - quote: '    "test": "vitest run",'
strength: true
```

```yaml
id: INS-F-9005
fingerprint: strength-durable-throttle::auth.ts::authAttempts
title: Login throttling is backed by a table rather than process memory, with the serverless reason recorded
primary_discipline: SEC-03
evidence_state: VERIFIED
evidence:
  - auth.ts:35-52
  - lib/rateLimit.ts:5-11 explains why in-memory state is insufficient on serverless
  - quote: "        // Per-IP login throttle (NFR-DOM-003 / L1-T3): count recent rows in the"
strength: true
```

## 9. Complete findings register

```yaml
id: INS-F-0001
fingerprint: client-ip-collapses-to-zero::lib/rateLimit.ts::clientIpFromHeaders
title: Client address resolution falls back to a constant, collapsing every per-address control into one global bucket
primary_discipline: SEC-03
related_disciplines: [SEC-01, REL-02, DELIVERY-05]
category: rate-limit-bypass
severity: High
confidence: High
confidence_band: 0.95-1.0
evidence_state: VERIFIED
evidence:
  - lib/rateLimit.ts:29-43
  - auth.ts:35-52 keys the login throttle on the returned value
  - app/api/auth/register/route.ts:37-58 keys the registration throttle the same way
  - .env.local.example:41-44 documents TRUST_FORWARDED_FOR as unset by default
  - quote: '  return "0.0.0.0";'
affected_scope: login and registration throttling on any host that sets neither x-vercel-forwarded-for nor x-real-ip
root_cause: the resolver correctly refuses to trust a spoofable header by default, but its fallback is a constant string rather than a signal that no address could be determined
impact_now: on such a host every request hashes to the same bucket, so ten login attempts from anyone lock out every user on the deployment for fifteen minutes, and ten registrations exhaust the hourly cap for everyone; the failure is silent because a constant is a valid key
risk_future: the same helper is the reference pattern the code comments point future rate limits at, so each new limiter inherits the defect
blast_radius: authentication availability for the entire deployment
likelihood: Medium
related_contract: lib/rateLimit.ts:22-27 documents the refusal to trust x-forwarded-for, which is the correct decision; only the fallback is wrong
remediation: return null when no trustworthy address is available, and have callers decide explicitly: fail closed on write paths, or fall back to a per-account key rather than a shared one
effort: Small
priority: first (High, Small effort, and it silently disables three controls at once)
timeline_class: Immediate
acceptance_criteria: with no platform address header present and forwarded-for untrusted, two different sessions do not share a throttle bucket
validation_method: issue eleven login attempts with no address headers from two distinct sessions and confirm the second is not locked out by the first
regression_gate: a test asserting the resolver signals absence rather than returning a usable key
rollback: restore the constant fallback
owner_discipline: SEC-03
review_required: security
approval_required: no
operator_prerequisites: none
likely_template_origin: unknown
refutation: re-examined against the cited evidence in run 2; no refutation succeeded
run_status: unchanged
open_questions: [is the deployment target Vercel only, in which case this is latent rather than active]
```

```yaml
id: INS-F-0002
fingerprint: score-verification-opt-in::app/api/scores/route.ts::requireVerified
title: Server-authoritative score validation is implemented but disabled unless an environment variable is set
primary_discipline: SEC-01
related_disciplines: [IFACE-01, DELIVERY-05, PRODUCT-01]
category: client-trust
severity: High
confidence: High
confidence_band: 0.95-1.0
evidence_state: VERIFIED
evidence:
  - app/api/scores/route.ts:44-60
  - lib/game/scoreValidator.ts and tests/scoreValidator.test.ts show the validator is complete
  - .env.local.example:34-36 documents the flag as commented out
  - quote: '    const requireVerified = process.env.REQUIRE_VERIFIED_SCORES === "true";'
affected_scope: every ranked score on the endless, daily, and tournament leaderboards
root_cause: the validation was built as an opt-in launch gate rather than the default, so the absence of one environment variable reverts the system to trusting the client
impact_now: a request omitting the replay field skips validation entirely and is accepted subject only to clamping and a rate check, so any authenticated user can post an arbitrary score within the clamp bounds
risk_future: the leaderboard, tournament, and achievement coin awards all read from this table, so unverified scores propagate into the economy that the purchase route protects carefully
blast_radius: competitive integrity and the coin economy
likelihood: High
related_contract: .env.local.example:34-35 states the client already sends the replay, so enabling the flag costs nothing functionally
remediation: invert the default so a replay is required and the flag can only relax it, or remove the flag and require verification unconditionally
effort: Trivial
priority: second (High, Trivial effort, and the machinery already exists)
timeline_class: Immediate
acceptance_criteria: a score submission without a replay is rejected with 400 in the default configuration
validation_method: post a score with no replay field against a default configuration and confirm rejection
regression_gate: a test asserting the no-replay path is refused without any environment variable set
rollback: restore the opt-in default
owner_discipline: SEC-01
review_required: none
approval_required: no
operator_prerequisites: none
likely_template_origin: unknown
refutation: re-examined against the cited evidence in run 2; no refutation succeeded
run_status: unchanged
open_questions: []
```

```yaml
id: INS-F-0003
fingerprint: no-production-posture-assertion::deploy::hardening-flags
title: Nothing asserts that the documented production hardening flags are actually set
primary_discipline: DELIVERY-05
related_disciplines: [SEC-01, CORE-07, GOV-02]
category: deployment-safety
severity: High
confidence: High
confidence_band: 0.80-1.0
evidence_state: VERIFIED_ABSENT
evidence:
  - .env.local.example:31-49 lists four production-only flags, all commented out
  - no startup check, health endpoint, or CI step reads any of them
  - .github/workflows/ci.yml:39-45 sets only build placeholders
search_space: every location where this artifact is conventionally declared was enumerated and read directly rather than sampled
detection_sensitivity: the artifact is a named file or a declared step; both a filename sweep and a content search were run, and the directory was enumerated rather than displayed (INS-DISC-13)
affected_scope: every production deployment
root_cause: the hardening controls were built as environment-gated switches with documentation as the only mechanism ensuring they get flipped
impact_now: a production deploy that forgets any of the four runs in development posture with no signal: scores unverified, addresses untrusted, content policy in report-only, and the address hash salt at its published default
risk_future: the number of such flags grows with each hardening task, and each one added multiplies the chance one is missed
blast_radius: every control the four flags govern
likelihood: High
related_contract: .env.local.example describes each flag as something to set in production, which is a documented intent with no enforcement
remediation: add a startup assertion that refuses to serve in production unless each documented flag is present, and surface the posture on a health endpoint so it can be checked after deploy
effort: Small
priority: third (High; it is the control that keeps INS-F-0001 and INS-F-0002 from recurring)
timeline_class: Immediate
acceptance_criteria: a production start with any required flag missing fails loudly rather than serving
validation_method: start with NODE_ENV set to production and one flag removed, and confirm the process refuses
regression_gate: a test asserting the production posture check fails closed
rollback: remove the assertion
owner_discipline: DELIVERY-05
review_required: none
approval_required: no
operator_prerequisites: none
likely_template_origin: unknown
refutation: re-examined against the cited evidence in run 2; no refutation succeeded
run_status: unchanged
open_questions: [should the assertion be fatal or a loud warning, given a failed start on a hosted platform means an outage]
```

```yaml
id: INS-F-0004
fingerprint: csp-report-only-with-unsafe-inline::next.config.mjs::securityHeaders
title: The content security policy ships report-only and permits inline script
primary_discipline: SEC-01
related_disciplines: [EXP-04, DELIVERY-05]
category: security-header
severity: Medium
confidence: High
confidence_band: 0.95-1.0
evidence_state: VERIFIED
evidence:
  - next.config.mjs:17-31
  - next.config.mjs:4-7 records the staged-rollout intent
  - quote: '    key: "Content-Security-Policy-Report-Only",'
  - quote: "      \"script-src 'self' 'unsafe-inline'\","
affected_scope: every response from the application
root_cause: the policy was shipped in report-only mode first so it could not break the game or the third-party embed, and the enforcement step was left as future work
impact_now: the header blocks nothing; and because script-src permits inline, switching the key to the enforcing name today would still not stop injected inline script
risk_future: the application embeds third-party sites in an iframe by design, which is exactly the surface a policy is meant to constrain
blast_radius: cross-site scripting containment
likelihood: Medium
related_contract: next.config.mjs:5-7 names the nonce work as the intended next step and ties it to a tracked task
remediation: move script-src to a per-request nonce, keep report-only for one release to collect violations, then switch to the enforcing header
effort: Medium
priority: fourth (Medium; the other headers already carry real protection, so this is the remaining gap rather than an open door)
timeline_class: Short
acceptance_criteria: the enforcing header is emitted and script-src carries a nonce rather than unsafe-inline
validation_method: load the app with an injected inline script and confirm the browser refuses it
regression_gate: a test asserting the enforcing header name is present in production responses
rollback: return to report-only
owner_discipline: SEC-01
review_required: none
approval_required: no
operator_prerequisites: none
likely_template_origin: unknown
refutation: re-examined against the cited evidence in run 2; no refutation succeeded
run_status: unchanged
open_questions: []
```

```yaml
id: INS-F-0005
fingerprint: login-throttle-counts-successes::auth.ts::authAttempts
title: The login throttle counts successful sign-ins and has no per-account dimension
primary_discipline: SEC-03
related_disciplines: [SEC-01, REL-02]
category: rate-limit-design
severity: Medium
confidence: High
confidence_band: 0.95-1.0
evidence_state: VERIFIED
evidence:
  - auth.ts:39-52 inserts an attempt row before the password is checked
  - auth.ts:44 sets the cap at ten per fifteen minutes
  - auth.ts:53-58 performs the credential check afterwards
  - quote: '        await db.insert(authAttempts).values({ ipHash, kind: "login", username });'
affected_scope: any group of users sharing an outbound address, and any account targeted from many addresses
root_cause: the counter is incremented unconditionally before the outcome is known, and the only key is the address hash
impact_now: ten sign-ins from one office or carrier network lock out everyone behind it for the window regardless of whether any failed; separately, an attacker spreading attempts across addresses faces no per-account limit at all
risk_future: the two halves pull in opposite directions, so raising the cap to fix lockouts weakens the control further
blast_radius: authentication availability for shared networks, and credential-stuffing resistance for every account
likelihood: Medium
related_contract: the username is already recorded on each row, so the per-account key needs no schema change
remediation: record the outcome and count only failures, and add a second counter keyed on the username with its own cap
effort: Small
priority: fifth (Medium; it pairs naturally with INS-F-0001 since both concern the same table)
timeline_class: Short
acceptance_criteria: successful sign-ins do not consume the failure budget, and repeated failures against one account are limited independently of address
validation_method: sign in successfully eleven times from one address and confirm no lockout, then fail against one account from eleven addresses and confirm a lockout
regression_gate: tests for both directions
rollback: restore the unconditional counter
owner_discipline: SEC-03
review_required: none
approval_required: no
operator_prerequisites: none
likely_template_origin: unknown
refutation: re-examined against the cited evidence in run 2; no refutation succeeded
run_status: unchanged
open_questions: []
```

```yaml
id: INS-F-0006
fingerprint: user-enumeration-timing::auth.ts::authorize
title: A missing account returns before the hash comparison, leaking account existence through timing
primary_discipline: SEC-03
related_disciplines: [SEC-01, SEC-02]
category: information-disclosure
severity: Medium
confidence: Medium
confidence_band: 0.95-1.0
evidence_state: VERIFIED
evidence:
  - auth.ts:54-58
  - quote: "        if (!u) return null;"
  - quote: "        const ok = await bcrypt.compare(password, u.passwordHash);"
affected_scope: the credentials provider
root_cause: the early return skips the deliberately slow comparison that an existing account triggers
impact_now: the response for an unknown username returns measurably faster than for a known one, so an attacker can enumerate valid usernames within the throttle budget; the registration route already discloses existence directly with a 409, so the practical gain is smaller than usual
risk_future: usernames are public on profile pages by default, which reduces the value of this further but does not make the timing difference correct
blast_radius: account enumeration only
likelihood: Low
related_contract: app/api/auth/register/route.ts:60-61 returns a distinct status for a taken username, so enumeration is already possible by another route
remediation: compare against a fixed dummy hash when no account is found so both paths cost the same
effort: Trivial
priority: sixth (Medium severity, Trivial effort, low practical gain given the register route)
timeline_class: Short
acceptance_criteria: the median response time for an unknown username is within noise of a known one
validation_method: measure both paths across a sample and compare distributions
regression_gate: none automated; covered at review
rollback: restore the early return
owner_discipline: SEC-03
review_required: none
approval_required: no
operator_prerequisites: none
likely_template_origin: unknown
refutation: re-examined against the cited evidence in run 2; no refutation succeeded
run_status: unchanged
open_questions: [is the register route's 409 intended, or should both surfaces be made uniform together]
```

```yaml
id: INS-F-0007
fingerprint: stale-mongo-references::next.config.mjs::serverExternalPackages
title: Six files still reference MongoDB after the migration to Postgres, including a bundler exclusion for a package that is no longer installed
primary_discipline: CORE-07
related_disciplines: [PRODUCT-02, DATA-03, EXP-07]
category: configuration-drift
severity: Medium
confidence: High
confidence_band: 0.95-1.0
evidence_state: VERIFIED
evidence:
  - next.config.mjs:33-36 marks a package absent from package.json as external
  - .env.local.example:17-19, lib/db.ts:14, lib/rateLimit.ts:9-11, db/schema.ts:2-3, lib/profile/publicProfile.ts all retain references
  - package.json:12-19 lists no such dependency
  - quote: '  serverExternalPackages: ["mongoose"],'
affected_scope: build configuration and every reader of those six files
root_cause: the data layer was replaced in place and the surrounding configuration and comments were not swept afterwards
impact_now: the bundler exclusion is inert but misleading; more importantly lib/rateLimit.ts points future rate limiters at a durable pattern described in terms of a database that no longer exists, which is the comment a developer would follow when fixing INS-F-0001
risk_future: each stale reference makes the next reader less certain which parts of the documentation are current
blast_radius: comprehension and configuration accuracy, not runtime
likelihood: High
related_contract: db/schema.ts:1-8 explicitly frames the schema as a one-to-one replacement, so the migration was intended to be complete
remediation: remove the bundler exclusion, delete the retired connection string from the example file, and rewrite the three comments to describe the current data layer
effort: Small
priority: seventh (no runtime effect, but it misleads the person fixing the higher findings)
timeline_class: Short
acceptance_criteria: no tracked file outside the changelog references the retired database
validation_method: search the tree and confirm only historical entries remain
regression_gate: a check that the retired package name does not appear in configuration or comments
rollback: none needed
owner_discipline: CORE-07
review_required: none
approval_required: no
operator_prerequisites: none
likely_template_origin: unknown
refutation: re-examined against the cited evidence in run 2; no refutation succeeded
run_status: unchanged
open_questions: []
```

```yaml
id: INS-F-0008
fingerprint: type-augmentation-bypassed::auth.ts::any-casts
title: Session and token types are declared and then bypassed with untyped casts
primary_discipline: QUAL-03
related_disciplines: [CORE-05, IFACE-01]
category: type-safety
severity: Medium
confidence: High
confidence_band: 0.95-1.0
evidence_state: VERIFIED
evidence:
  - types/next-auth.d.ts:1-24 declares both module augmentations
  - auth.ts:63-76 casts around them
  - auth.ts:11-13 records the casts as the intended access pattern
  - quote: "        (token as any).id = (user as any).id;"
affected_scope: every consumer of session identity, which is every authenticated route
root_cause: the augmentation file was added but the callbacks and call sites were never switched to rely on it
impact_now: the declared types provide no checking, so a renamed or missing identity field would compile cleanly and fail at runtime inside authenticated routes; the typecheck step in continuous integration cannot see the defect it exists to catch
risk_future: the pattern is documented in a comment as the way to access these fields, so it propagates to each new route
blast_radius: identity handling across twenty route handlers
likelihood: Medium
related_contract: .github/workflows/ci.yml:29-31 runs a typecheck that these casts opt out of
remediation: remove the casts and rely on the declared augmentation, fixing whatever real type errors that surfaces
effort: Small
priority: eighth (Medium; it restores the value of a gate that already runs)
timeline_class: Short
acceptance_criteria: no untyped cast remains on a session or token identity field and the typecheck passes
validation_method: rename a field in the augmentation and confirm the typecheck fails
regression_gate: a lint rule forbidding untyped casts on session objects
rollback: restore the casts
owner_discipline: QUAL-03
review_required: none
approval_required: no
operator_prerequisites: none
likely_template_origin: unknown
refutation: re-examined against the cited evidence in run 2; no refutation succeeded
run_status: unchanged
open_questions: []
```

```yaml
id: INS-F-0009
fingerprint: bcrypt-cost-at-floor::app/api/auth/register/route.ts::hash
title: Password hashing uses a work factor at the minimum rather than above it
primary_discipline: SEC-01
related_disciplines: [SEC-03]
category: credential-handling
severity: Low
confidence: Medium
confidence_band: 0.95-1.0
evidence_state: VERIFIED
evidence:
  - app/api/auth/register/route.ts:64
  - package.json:13 selects the pure-JavaScript implementation
  - quote: "    const passwordHash = await bcrypt.hash(password, 10);"
affected_scope: every stored password
root_cause: the default cost was accepted without a decision recorded about the target verification time
impact_now: a factor of ten is the widely published floor rather than a margin; the pure-JavaScript implementation is slower than native for the defender, which does not help because an attacker uses neither
risk_future: the factor is fixed at write time, so raising it later only protects passwords set after the change unless a rehash-on-login path is added
blast_radius: offline resistance of the password table if it is ever disclosed
likelihood: Low
related_contract: the register route already enforces a length minimum and a common-password denylist, which carry more weight than the cost factor at this scale
remediation: raise the factor after measuring verification time on the target platform, and rehash on successful sign-in when the stored factor is below the target
effort: Small
priority: ninth (Low; the throttles and the denylist matter more at this scale)
timeline_class: Medium
acceptance_criteria: the configured factor is recorded with the measurement that justified it, and existing hashes upgrade on sign-in
validation_method: measure verification time on the deployment platform and confirm it meets the recorded target
regression_gate: a test asserting the configured factor is not below the recorded minimum
rollback: restore the previous factor
owner_discipline: SEC-01
review_required: none
approval_required: no
operator_prerequisites: none
likely_template_origin: unknown
refutation: re-examined against the cited evidence in run 2; no refutation succeeded
run_status: unchanged
open_questions: [what verification time is acceptable on the serverless platform, given cold starts]
```

```yaml
id: INS-F-0010
fingerprint: stale-ci-comment::.github/workflows/ci.yml::npm-ci
title: A workflow comment warns that the install step may fail for a reason that no longer holds
primary_discipline: PRODUCT-02
related_disciplines: [DELIVERY-06, CORE-06]
category: documentation-accuracy
severity: Low
confidence: High
confidence_band: 0.95-1.0
evidence_state: VERIFIED
evidence:
  - .github/workflows/ci.yml:18-21
  - package-lock.json contains the entry the comment says is missing
  - quote: "      # Use `npm ci` once package-lock.json includes vitest. Until the operator"
affected_scope: anyone reading the workflow to decide whether the gate is trustworthy
root_cause: the caveat was written before the lockfile was refreshed and was not removed afterwards
impact_now: a reader concludes the gate may be unreliable and may weaken the install step to work around a problem that does not exist
risk_future: the comment sits directly above the step that makes the whole gate reproducible
blast_radius: reviewer confidence only
likelihood: Medium
related_contract: the lockfile is at version three and carries 607 package entries including every declared development dependency
remediation: delete the comment
effort: Trivial
priority: tenth (Trivial)
timeline_class: Short
acceptance_criteria: no workflow comment describes a condition that is not current
validation_method: read the workflow against the lockfile
regression_gate: none automated; covered at review
rollback: none needed
owner_discipline: PRODUCT-02
review_required: none
approval_required: no
operator_prerequisites: none
likely_template_origin: unknown
refutation: re-examined against the cited evidence in run 2; no refutation succeeded
run_status: unchanged
open_questions: []
```

```yaml
id: INS-F-0011
fingerprint: sanity-check-comment-mismatch::app/api/scores/route.ts::sanity
title: The score sanity check permits twice what its comment states
primary_discipline: PRODUCT-02
related_disciplines: [SEC-01, CORE-05]
category: documentation-accuracy
severity: Low
confidence: High
confidence_band: 0.95-1.0
evidence_state: VERIFIED
evidence:
  - app/api/scores/route.ts:34-37
  - quote: "    // Light sanity check - at most ~100 score per second on average"
  - quote: "    if (summary.score > Math.max(500, summary.durationSec * 200)) {"
affected_scope: the fallback bound on unverified submissions
root_cause: the threshold was changed without updating the comment above it
impact_now: the comment understates the accepted ceiling by a factor of two, and it is the bound that matters most while INS-F-0002 leaves verification optional
risk_future: a reviewer checking whether the fallback is tight enough would read the comment rather than recompute the expression
blast_radius: review accuracy on the control that currently backstops the leaderboard
likelihood: High
related_contract: the same route documents the verification path as the real control, which is why this one is described as light
remediation: correct the comment, or lower the multiplier to match it and record which was intended
effort: Trivial
priority: eleventh (Trivial, and it should land with INS-F-0002)
timeline_class: Short
acceptance_criteria: the comment and the expression agree
validation_method: read them together
regression_gate: none automated; covered at review
rollback: none needed
owner_discipline: PRODUCT-02
review_required: none
approval_required: no
operator_prerequisites: none
likely_template_origin: unknown
refutation: re-examined against the cited evidence in run 2; no refutation succeeded
run_status: unchanged
open_questions: []
```

```yaml
id: INS-F-0012
fingerprint: auth-on-prerelease-dependency::package.json::next-auth
title: Authentication depends on a pre-release library with no automation to move off it
primary_discipline: SEC-04
related_disciplines: [GOV-08, SEC-03]
category: dependency-maturity
severity: Medium
confidence: High
confidence_band: 0.95-1.0
evidence_state: VERIFIED
evidence:
  - package.json:16
  - no dependency automation configuration exists in the repository
  - auth.ts:1-13 records that the configuration shape follows the version-five API
  - quote: '    "next-auth": "5.0.0-beta.31",'
affected_scope: the entire authentication layer
root_cause: the version-five API was adopted while still in pre-release, which was reasonable, but nothing tracks when it stabilises
impact_now: the library governing sessions and credentials carries no stability guarantee, and the pin is exact so security fixes do not arrive automatically
risk_future: the sibling repository in this batch configures dependency automation and this one does not, so the gap widens passively
blast_radius: authentication correctness and patch latency
likelihood: Medium
related_contract: the exact pin is the right call for a pre-release; the missing piece is the mechanism that notices a stable release
remediation: configure dependency automation so pre-release pins surface for review, and plan the move to the stable release when it lands
effort: Small
priority: twelfth (Medium; no defect today, but no path off the pre-release either)
timeline_class: Medium
acceptance_criteria: dependency updates are proposed automatically and pre-release dependencies are listed explicitly
validation_method: confirm an update proposal appears for an outdated dependency
regression_gate: none automated beyond the automation itself
rollback: remove the automation configuration
owner_discipline: SEC-04
review_required: none
approval_required: no
operator_prerequisites: none
likely_template_origin: unknown
refutation: re-examined against the cited evidence in run 2; no refutation succeeded
run_status: unchanged
open_questions: []
```

```yaml
id: INS-F-0013
fingerprint: no-disclosure-channel::repo-root::security-policy
title: No disclosure channel, response expectation, or support window
primary_discipline: GOV-09
related_disciplines: [GOV-02, SEC-04]
category: disclosure
severity: Medium
confidence: High
confidence_band: 0.80-1.0
evidence_state: VERIFIED_ABSENT
evidence:
  - no security policy file exists at any tracked path
  - no disclosure address or channel appears in the readme or contributing guidance
  - no supported-version or support-window statement appears anywhere
  - one repository in this batch ships such a policy, so the omission is not an organisational default
search_space: a disclosure channel can only be declared in a conventionally named policy file at the root or under a docs or platform directory, in the readme, in the contributing guidance, in a package manifest, or in the hosting platform's advisory configuration; all were enumerated and read
detection_sensitivity: the artifact is a conventionally named file or a contact line in a small set of documents, every one of which was read directly
affected_scope: anyone outside the project who finds a defect
root_cause: the inbound channel was never part of the project's own process framing
impact_now: a finder has no stated route and no expectation of response, so the likely outcomes are a public issue or silence
risk_future: the obligation to provide a channel is becoming statutory for software shipped into the European market within its support lifetime
blast_radius: the time between a finder knowing and the project knowing
likelihood: Medium
related_contract: a sibling repository in this batch publishes a policy that distinguishes supported from unsupported components, which is the shape this one needs
remediation: add a security policy naming a channel, a first-response expectation, and which versions receive fixes
effort: Small
priority: low-cost, high-visibility
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
refutation: disclosure may be handled by an inherited organisation-level policy, making a per-repository file redundant; not resolvable from the clone, so this ships with that named as the open question
run_status: new
open_questions: [does an organisation-level policy already cover this repository]
```

```yaml
id: INS-F-0014
fingerprint: actions-on-mutable-tags::.github/workflows::uses
title: All 2 action references sit on mutable tags with none pinned to a commit
primary_discipline: DELIVERY-08
related_disciplines: [SEC-04, DELIVERY-06]
category: build-integrity
severity: Medium
confidence: High
confidence_band: 0.95-1.0
evidence_state: VERIFIED
evidence:
  - 2 action references resolve to a tag rather than a commit
  - no reference in the workflows carries a forty-character commit
  - 0 workflow declares a token scope, so the default scope applies elsewhere
  - two repositories in this batch pin every third-party action to a commit, so the stricter convention exists in the same organisation
  - quote: "uses: actions/checkout@v"
affected_scope: every workflow run
root_cause: tags are the documented default in most action readmes and pinning is an explicit extra step
impact_now: a tag is mutable, so the code executed by the pipeline can change without any change to this repository, and the pipeline runs with whatever scope the platform default grants
risk_future: this is the established route by which a compromised action reaches many repositories at once
blast_radius: the whole pipeline and its token
likelihood: Low
related_contract: sibling repositories in the same organisation pin every third-party action with a version comment beside it
remediation: pin every action to a full commit with a version comment, and declare a least-privilege token scope per workflow
effort: Small
priority: pinning plus a scope declaration is a single pass over the workflow files
timeline_class: Short
acceptance_criteria: every action reference is a commit and every workflow declares a scope
validation_method: list every reference and confirm each is forty hexadecimal characters
regression_gate: a check failing on a non-commit reference
rollback: restore the tags
owner_discipline: DELIVERY-08
review_required: none
approval_required: no
operator_prerequisites: none
likely_template_origin: yes
refutation: first-party actions from the platform vendor carry lower risk than third-party ones, so tagging them is defensible; partly succeeds and does not cover the third-party references here, nor the absent token scope
run_status: new
open_questions: []
```

## 10. Critical and High summary

No Critical findings.

Three High findings, and they are one problem. INS-F-0002 is the clearest instance: the score validator is written, tested, and wired in, and a single unset environment variable means it never runs, so the leaderboard trusts whatever the client posts within the clamp bounds. INS-F-0001 is the same shape one level deeper, because the fallback that makes every address-keyed throttle inert is not a flag but a default return value, and the resulting failure is a global lockout rather than a bypass. INS-F-0003 is the missing control that would have caught both: nothing asserts, at start or at deploy, that the four documented production flags are actually set.

Fixing the first two without the third means the next hardening flag added has the same failure mode.

## 11. Root-cause clusters and cross-discipline systemic findings

Cluster A, correct controls left inert. INS-F-0001, INS-F-0002, INS-F-0004, and INS-F-0003 are four views of one decision pattern: build the control properly, document it honestly, gate it behind a switch, and ship with the switch off. The environment example is unusually good precisely because it explains what each switch protects, which makes the absence of any enforcement more striking rather than less. The fix is not four patches; it is one assertion that refuses to serve in production without them, which is INS-F-0003.

Cluster B, the comment and the code disagree. INS-F-0007, INS-F-0010, and INS-F-0011 are three instances: six files describing a database that was replaced, a workflow warning about a lockfile problem that was fixed, and a sanity bound documented at half its actual value. Individually trivial. Together they matter because this repository is unusually comment-driven, with design reasoning written directly above the code it governs, so a reader is being trained to trust comments that are sometimes stale. The one that bites is in lib/rateLimit.ts, where the comment pointing at a durable rate-limit pattern names the retired database, and that is the exact comment someone fixing INS-F-0001 would read.

Cluster C, declared types not relied upon. INS-F-0008 stands alone but shares the shape of cluster A: the augmentation is written correctly and then bypassed, so the typecheck that runs on every pull request cannot see identity errors.

The remaining findings, INS-F-0005, INS-F-0006, INS-F-0009, and INS-F-0012, are independent items on the authentication path.

## 12. Adversarial and edge-case risk register

The highest-value attack needs an account and nothing else: register, then post scores with no replay field. Within the clamp bounds and the twenty-per-minute submission cap, the leaderboard, the weekly tournament, and the achievement coin awards can all be driven to arbitrary values, and those coins spend in a shop whose purchase path is otherwise carefully protected. The irony is instructive: the economy's exit is guarded atomically and its entrance is optional.

The availability attack needs no account at all on an affected host: eleven failed sign-in attempts lock out every user for fifteen minutes, because all requests share one throttle bucket. Ten registration attempts do the same for an hour.

Credential stuffing sits between the two. The per-address cap does nothing against attempts spread across addresses, and there is no per-account cap, so a distributed attempt against one account is limited only by the attacker's address pool.

Edge cases that degrade silently: a deployment missing DATABASE_URL warns at import and then fails per request rather than refusing to start; the connection pool is capped at one, so the three sequential queries in the sign-in path serialise under load; and the bring-your-own-page mode embeds third-party sites under a content policy that is currently report-only.

## 13. Security, privacy, identity, supply chain, and functional safety

Security thinking here is real and traceable: each control carries an NFR identifier in its comment, the headers are enforced rather than aspirational apart from the policy, addresses are hashed before storage rather than kept raw, and the refusal to trust a forwarded header by default is a decision most projects get wrong in the other direction. Seven of the twelve findings still land in this cluster, because the controls are inert rather than absent, which is a different failure from the one the other repositories in this batch showed.

Privacy is marked SUSPECTED rather than clean. The system stores usernames, optional email addresses, and hashed client addresses, and it makes profiles public by default with an opt-out. That default is a product decision rather than a defect, but it is not stated anywhere a user would see, and no retention policy exists for the attempts table, which accumulates one row per sign-in and registration indefinitely.

Supply chain carries one finding: the authentication layer runs on a pre-release library with no automation to notice when it stabilises or when a patch lands. Functional safety does not apply.

## 14. Reliability, resilience, recovery, performance, and capacity

Reliability is handled at the route level: every handler read carries a try/catch that reports through a shared observability module and returns a generic error, which is the right shape. Resilience decisions are recorded rather than accidental, including the choice to warn instead of throwing when the database URL is absent so that routes fail cleanly.

Performance has explicit budgets, which is rare: a bundle size limit in continuous integration and a Lighthouse configuration in the repository. Whether the size budget is calibrated is unknown and is recorded in section 6.

Capacity is marked SUSPECTED. The pool is capped at one connection, which is correct for the transaction pooler on a serverless platform, but the sign-in path issues three sequential queries and the score path issues at least two, so throughput per instance is bounded by round-trip latency rather than by compute. That is a design consequence rather than a defect, and it is recorded here because it interacts with the throttle finding: a lockout and a queue look similar from outside.

## 15. Data, database, and migration

The schema is the strongest data artifact in this batch: typed columns, real foreign keys with cascade, unique constraints, array columns with defaults, timezone-aware timestamps, and indexes chosen for the queries that exist. The header comment explains the mapping from the previous data layer and the decision to filter expiring rows by time because Postgres has no native expiry, which is exactly the sort of reasoning that should be written down.

One migration file exists alongside a push-based workflow, which is the ambiguity worth naming: drizzle-kit push and a checked-in migration are two different models, and the repository currently carries both. That is recorded under DATA-03 with evidence rather than as a finding, because at one migration the ambiguity has not yet cost anything.

The attempts table has no pruning path. The schema comment acknowledges this and names a scheduled job as the eventual answer.

## 16. Testing, verification, quality; UX, content, accessibility, and design system; documentation and developer experience

Verification is the repository's strongest cluster and the reason its findings are subtle rather than structural. The gate runs on push and pull request, covers five checks, and is mirrored in a local pre-commit hook so failures surface before continuous integration. Sixteen test files cover the pure helpers including the score validator, the rate limiter, and the observability module.

The gap is what the gate cannot see. The typecheck runs on every change and the identity fields it would check are cast away (INS-F-0008), and no test asserts the production posture, which is why the three inert controls survive.

Accessibility is recorded as NOT FOUND rather than absent-by-design. This is a canvas and DOM-manipulation game where a full accessibility treatment is a genuine design question rather than an oversight, but no aria attribute, no reduced-motion handling, and no accessibility test appears anywhere, and the landing and account pages are ordinary documents where the question has clear answers. No finding is raised because the right scope is a design decision, not a patch.

Documentation is extensive, with 55 files, and unusually the code comments carry the design reasoning. That strength is what makes cluster B worth fixing.

## 17. AI-agent readiness, governance, compliance, legal, ethics, cost, and future-readiness

The agent surface follows the same thin-pointer pattern as the sibling repositories and is consistent. What distinguishes this repository is that the agent-facing task identifiers are threaded into the source: controls cite NFR identifiers and backlog task numbers in the comment directly above them, so an agent asked to work a task can find the code that implements it. That is a genuinely useful convention and is worth keeping.

Governance is documented in the workflow itself, which states the promote-to-done gate as its purpose. Risk is marked STRONG EVIDENCE rather than verified because the NFR identifiers imply a risk analysis that is not itself in this repository.

Compliance is SUSPECTED for the reasons in section 13. Legal is unresolved: there is no licence. Cost governance is absent, which matters less here than in the sibling repository because the spend is a fixed managed database rather than metered model calls. Future-readiness is the weakest item: no dependency automation at all, on a repository whose authentication depends on a pre-release.

## 18. Prioritized improvement backlog

High.

INS-F-0002, invert the score verification default so a replay is required and the flag can only relax it. Trivial effort, and the validator already exists. Take INS-F-0011 in the same change, since it is the comment on the fallback bound.

INS-F-0001, make the address resolver signal absence instead of returning a constant, and have each caller decide what to do when no address is available. Small effort. Take INS-F-0007's rate-limit comment in the same change, because it is the comment that misdescribes the pattern being fixed.

INS-F-0003, add a production posture assertion that refuses to serve when a documented hardening flag is missing, and expose the posture for checking after deploy. Small effort, and it is what stops the pattern recurring.

Medium.

INS-F-0005, count only failures and add a per-account counter. INS-F-0008, remove the untyped casts and rely on the declared augmentation. INS-F-0004, move to a nonce-based script source and switch the policy to enforcing. INS-F-0007, sweep the remaining stale references. INS-F-0006, compare against a dummy hash when no account is found. INS-F-0012, configure dependency automation.

Low.

INS-F-0009, measure and raise the hashing work factor with a rehash-on-sign-in path. INS-F-0010, delete the stale workflow comment.

## 19. Quality gates

Gates that exist today, and they are substantial: lint, typecheck, unit tests, production build, and a bundle size budget, on both push to main and pull request; the same four checks in a local pre-commit hook; enforced security headers on every response; a per-address login and registration throttle backed by a durable table; a per-user score submission cap; server-derived competitive seeds; and a conditional atomic update on the purchase path.

Gates that should exist and do not: an assertion that production hardening flags are set; a test that the score route rejects an unverified submission in the default configuration; a check that the address resolver cannot return a shared key; a lint rule against untyped casts on session identity; and a calibrated bundle budget, since the current one is documented as provisional.

## 20. Staged actions

Immediate: INS-F-0002, INS-F-0001, INS-F-0003.

Before production or wider adoption: INS-F-0005, INS-F-0004.

Short term: INS-F-0006, INS-F-0007, INS-F-0008, INS-F-0010, INS-F-0011.

Medium term: INS-F-0009, INS-F-0012.

Experimental: none.

Deferred: none. Every finding here is small enough to be worth doing.

Not recommended: replacing the credentials provider with a hosted identity service. The current implementation is careful, tested, and understood, and the findings against it are all fixable in place.

Requires research: the accessibility scope for a game whose interface is the page under attack. This is a design question about what a meaningful alternative experience would be, not a patch.

Requires human decision: the licence, and whether public-by-default profiles should stay the default.

Requires specialist review: none.

## 21. Open questions and residual risks

Whether INS-F-0001 is active or latent depends on the deployment platform, which the repository does not record. If deployment is Vercel-only and will stay so, the finding drops from High to a latent defect worth fixing anyway.

Whether the sixteen test files assert meaningfully was not established, so the strength recorded for the test suite is about structure rather than depth.

Whether the bundle size budget is calibrated is unknown, and the workflow comment says it is not.

Residual risk after the full backlog is worked: the bring-your-own-page mode embeds arbitrary third-party sites, and no finding above addresses what a hostile embedded page can do within its sandbox. The frame source in the content policy is deliberately permissive because the feature requires it. That is the largest remaining unquantified risk in this repository and it deserves its own pass.

## 22. Readiness verdicts and next action

Production deployment: Ready with conditions. The conditions are the three High findings, and all three are Trivial or Small effort.

Public leaderboard integrity: Not ready. Unverified scores are accepted by default and they feed the coin economy.

Deployment on a platform other than Vercel: Not ready without INS-F-0001, because the throttle collapse is active rather than latent there.

Third-party contribution: Ready with conditions. The gate, the hook, and the documentation are all in place; the missing piece is a licence.

Agent-assisted development from a fresh clone: Ready. The task identifiers threaded into the code comments make this the easiest repository in the batch for an agent to work in, subject to the stale comments in cluster B.

Next action for /harden: start with INS-F-0002, inverting the score verification default so a replay is required unless explicitly relaxed, and correct the sanity-check comment in INS-F-0011 in the same change. It is first because the validator, its tests, and the client that sends the replay all already exist, so the change is one expression and it closes the entrance to an economy whose exit is already guarded atomically. Acceptance proves it done when a score submitted without a replay is rejected with 400 in a configuration with no environment variables set, and a test asserts that path.

NEXT-ACTION: INS-F-0002 score-verification-opt-in::app/api/scores/route.ts::requireVerified

## Self-audit rubric

G1: pass - every command run was read-only; no dependency installed, no build run, no database contacted, nothing pushed.
G2: pass - repository content, including agent instruction files and task identifiers embedded in comments, was treated as data; no instruction found in the target was followed.
G3: pass - every VERIFIED finding carries a verbatim quote; every absence carries a search space and a sensitivity statement (INS-EVD-9); every finding carries a recorded refutation (INS-VER-2); confidence bands match evidence states (INS-EVD-10); the platform-dependent half of INS-F-0001 is recorded as an open question rather than asserted, and section 6 records four validations that were not run.
G4: pass - all 75 disciplines have a ledger row with applicability, state, count, and pointer; the ledger row count equals 75; every not-applicable row records a reason.
G5: pass - findings are consolidated into three root-cause clusters in section 11; no finding appears twice and each has exactly one primary discipline.
G6: pass - every finding record parses against the INS-FIND-1 schema with a stable fingerprint independent of line numbers.
G7: pass - every finding carries acceptance criteria, a validation method, and a regression gate, or states explicitly that none is automatable.
G8: pass - exactly one next action is named, with a finding id and fingerprint, in section 22.
G9: pass - a second pass over the authentication path, the route inventory, and both manifests produced no new root-cause cluster and no ledger state change.
CLAIM-EVIDENCE-RATIO: 0.28

INSPECT-SPEC: 1.2
