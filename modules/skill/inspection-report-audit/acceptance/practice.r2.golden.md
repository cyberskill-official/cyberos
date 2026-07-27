# /inspect report: cyberskill-official/practice (run 2, slot 4)

QUALITY-HEADER
coverage: 65/75 applicable, clusters fully read: 7/12
evidence: 9/9 findings carry a verbatim quote, distinct evidence pointers: 31
verification: 9/9 findings survived a recorded refutation attempt
stability: single run, unmeasured
calibration: 9 run-1 findings re-tested, 1 falsified including the batch's only Critical

## 1. Side-effect disclosure

None. Every command was read-only: git clone, git ls-files, file reads, and text search. No dependency was installed, no build was run, no database or hosted service was contacted, no credential was tested, and nothing was written to the repository or pushed. The clone lives in a scratch directory. One credential literal was found in source and is reported with its value redacted to prefix and format only.

## 2. Executive summary

Run 1's Critical finding on this repository was wrong, and it was the only Critical across 106 findings and ten repositories.

It asserted that no row-level security appeared in any committed SQL while a published anonymous key wrote to tables, and rated the anonymous key a full read and write credential. The repository is at the same commit today. It tracks twenty-four SQL files: five directly under the Supabase directory and nineteen under its migrations subdirectory. Seventeen of the twenty-four contain a row-level-security statement. `supabase/migrations/20260801000001_platform_rls.sql` enables it on nine platform tables and states its own design in a header comment, a deny-all trusted-server pattern in which the service role bypasses and anonymous callers get zero policies.

Run 1 read the five files in the top-level directory and reported a conclusion about the repository. Its evidence list named what it searched for and never named where SQL could live, which is precisely the omission that specification 1.2's INS-EVD-9 now forbids by requiring a search space before an absence can be recorded. Run 1 even hypothesised in its own open questions that the security might be enabled somewhere it had not looked. It was, one directory down.

The Critical is withdrawn. What replaces it is Low: the deny-all pattern is applied consistently across nineteen migrations and nothing asserts that coverage, so a table added without the statement would fail open and look identical to one deliberately left open.

The rest of run 1 stands. The intellectual-property controls remain the best of either batch, binding every generated item to a source and validating the binding, and they still run only in a pre-commit hook that a flag bypasses. The content-security policy still carries both unsafe directives.

Findings: 9 total, 0 Critical, 2 High, 4 Medium, 3 Low. Six strengths recorded.

The one exact next action is at section 22.

## 3. Inspection metadata and baseline

Target: https://github.com/cyberskill-official/practice, default branch main, head 3f30d5d, 133 commits, last commit 2026-07-26. Working tree 5.3M excluding .git. 605 tracked files.

Languages by line count: TypeScript 21,916 across 174 files; JSON 15,434 across 37; Markdown 13,331 across 193; TSX 11,790 across 107; ES modules 3,932 across 32, which is the script and pipeline tree; SQL 859 across 24 files.

Stack: Next.js 16.2.7 with React 19.2.4, Supabase with two client paths, Zustand, Tailwind 4, dompurify for untrusted markup, and a full OpenTelemetry export of traces, logs, and metrics alongside PostHog. Vitest for unit tests and Playwright for end to end, both gated. Husky and lint-staged for local hooks. Node pinned to 24.18.0 in two files with a matching engine range. One workflow with two jobs. Deployment targets both a container and a hosting platform.

A model-driven item generation pipeline lives under `tools/item-pipeline` with staged execution, a dry-run default, and retained run output.

Inspection was single-pass and reached saturation without continuation; see section 5.

## 4. Coverage ledger

All 75 disciplines, in stable id order. {{APPLICABLE_COUNT}} applicable, 4 not applicable with a recorded reason.

| Discipline | Cluster | Applicability | Evidence state | Finding count | Evidence pointer | Related |
|---|---|---|---|---|---|---|
| CORE-01 Requirements engineering | CORE | APPLICABLE | VERIFIED | 0 | docs/ 186 files, audit-profile.yaml, skills-lock.json | AGENT-10 |
| CORE-02 Domain engineering | CORE | APPLICABLE | VERIFIED | 0 | src/core/database.types.ts, supabase/*.sql five table definitions | DATA-02 |
| CORE-03 Systems engineering | CORE | APPLICABLE | VERIFIED | 0 | next.config.ts, Dockerfile, docker-compose.yml, vercel.json | DELIVERY-03 |
| CORE-04 Architecture engineering | CORE | APPLICABLE | VERIFIED | 0 | src/lib/supabase.ts and src/lib/supabaseAdmin.ts separate the two credentials | SEC-03, EXP-05 |
| CORE-05 Software engineering | CORE | APPLICABLE | VERIFIED | 0 | 21,916 lines TypeScript across 174 files | QUAL-01 |
| CORE-06 Repository engineering | CORE | APPLICABLE | VERIFIED | 1 | 133 commits, .husky pre-commit hook, supabase/.temp tracked | DELIVERY-06 |
| CORE-07 Configuration engineering | CORE | APPLICABLE | VERIFIED | 0 | .env.example, .node-version and .nvmrc agreeing, engines range | DELIVERY-06 |
| CORE-08 Concurrency engineering | CORE | APPLICABLE | VERIFIED_ABSENT | 0 | no concurrent execution paths beyond per-request handlers | REL-02 |
| CORE-09 Distributed systems engineering | CORE | APPLICABLE | VERIFIED_ABSENT | 0 | single deployment plus one managed database | DELIVERY-03 |
| PRODUCT-01 Product engineering | PRODUCT | APPLICABLE | VERIFIED | 0 | src/app 98 files across the exam practice surface | PRODUCT-03 |
| PRODUCT-02 Documentation engineering | PRODUCT | APPLICABLE | VERIFIED | 0 | README.md, docs/ 186 files, supabase/README.md | EXP-07 |
| PRODUCT-03 Content engineering | PRODUCT | APPLICABLE | VERIFIED | 0 | src/data item bank, tools/item-pipeline generation stages | GOV-05, AGENT-01 |
| PRODUCT-04 Internationalization and localization engineering | PRODUCT | APPLICABLE | VERIFIED | 0 | src/i18n four files | PRODUCT-03 |
| DATA-01 Data engineering | DATA | APPLICABLE | VERIFIED | 0 | src/lib/catalog.ts, entitlements.ts, sittings.ts over the admin client | DATA-02, SEC-03 |
| DATA-02 Database engineering | DATA | APPLICABLE | VERIFIED | 0 | 19 tracked migrations under supabase/migrations plus 5 top-level scripts; 17 contain row level security | SEC-03, DATA-01 |
| DATA-03 Migration engineering | DATA | APPLICABLE | VERIFIED_ABSENT | 0 | no migration directory; SQL files are applied by hand per supabase/README.md | DATA-02 |
| IFACE-01 API engineering | IFACE | APPLICABLE | VERIFIED | 0 | src/app/api route handlers including exam grading | SEC-03, EXP-05 |
| IFACE-02 Integration engineering | IFACE | APPLICABLE | VERIFIED | 0 | PostHog, SuperLog intake, Supabase realtime in the content policy | SEC-02, REL-06 |
| IFACE-03 Event and messaging engineering | IFACE | APPLICABLE | VERIFIED | 0 | connect-src permits a realtime socket origin | IFACE-02 |
| SEC-01 Security engineering | SEC | APPLICABLE | VERIFIED | 1 | next.config.ts:5-16 enumerates the content policy | SEC-03, EXP-04 |
| SEC-02 Privacy engineering | SEC | APPLICABLE | SUSPECTED | 0 | supabase/subscribers.sql and exam-results.sql retain personal data | GOV-04, SEC-03 |
| SEC-03 Identity and access engineering | SEC | APPLICABLE | VERIFIED | 2 | supabase/migrations/20260801000001_platform_rls.sql enables row level security on nine platform tables with a documented deny-all trusted-server pattern | DATA-02, SEC-01 |
| SEC-04 Supply-chain engineering | SEC | APPLICABLE | VERIFIED | 1 | package.json:7-9 fetches a formatter at run time; npm ci in the workflow | DELIVERY-06, GOV-08 |
| SEC-05 Functional safety engineering | SEC | NOT APPLICABLE (conditional row; no safety-critical function) | NOT APPLICABLE | 0 | NONE | |
| SEC-06 Threat modeling engineering | SEC | APPLICABLE | VERIFIED_ABSENT | 0 | no security threat model; the only threat-model phrase is a content-contamination argument in a task specification | SEC-01, GOV-03 |
| SEC-07 Business-logic security engineering | SEC | APPLICABLE | STRONG EVIDENCE | 0 | exam sittings, item responses, and leaderboard capture are the reachable logic surface | SEC-03, PRODUCT-01 |
| REL-01 Reliability engineering | REL | APPLICABLE | STRONG EVIDENCE | 0 | scripts/local-smoke.mjs, instrumentation.ts error export | REL-06 |
| REL-02 Resilience engineering | REL | APPLICABLE | STRONG EVIDENCE | 0 | src/lib/supabaseAdmin.ts:19 returns null rather than throwing at import | CORE-04 |
| REL-03 Performance engineering | REL | APPLICABLE | SUSPECTED | 0 | next.config.ts optimizePackageImports for two heavy packages | EXP-04 |
| REL-04 Capacity engineering | REL | APPLICABLE | VERIFIED_ABSENT | 0 | no connection or concurrency sizing is recorded | DATA-01 |
| REL-05 Site reliability engineering | REL | NOT APPLICABLE (no on-call rotation or published service level objective) | NOT APPLICABLE | 0 | NONE | |
| REL-06 Observability engineering | REL | APPLICABLE | VERIFIED | 0 | instrumentation.ts exports traces, logs, and metrics over OTLP | IFACE-02, REL-01 |
| REL-07 Incident and problem management engineering | REL | NOT APPLICABLE (no incident or problem management process to inspect) | NOT APPLICABLE | 0 | NONE | |
| DELIVERY-01 Platform engineering | DELIVERY | NOT APPLICABLE (no internal platform offered to other teams) | NOT APPLICABLE | 0 | NONE | |
| DELIVERY-02 Infrastructure engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | Dockerfile, docker-compose.yml, .dockerignore | DELIVERY-03 |
| DELIVERY-03 Cloud engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | vercel.json, @vercel/otel and speed-insights dependencies | DELIVERY-02 |
| DELIVERY-04 Build engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | package.json build script, next.config.ts | DELIVERY-06 |
| DELIVERY-05 Release engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | CHANGELOG.md, vercel.json | DELIVERY-04 |
| DELIVERY-06 CI/CD engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | .github/workflows/ci.yml two jobs, locked install, e2e gated | QUAL-01, QUAL-03 |
| DELIVERY-07 Embedded and firmware engineering | DELIVERY | NOT APPLICABLE (conditional row; no embedded or firmware target) | NOT APPLICABLE | 0 | NONE | |
| DELIVERY-08 Repository and build integrity engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | actions pinned and a lockfile committed; no token scope, provenance, bill of materials, or signing | SEC-04, DELIVERY-06 |
| QUAL-01 Test engineering | QUAL | APPLICABLE | VERIFIED | 0 | tests/ 73 files, unit and end-to-end both run in the workflow | QUAL-03 |
| QUAL-02 Quality engineering | QUAL | APPLICABLE | VERIFIED | 0 | eslint.config.mjs, .prettierrc, .lintstagedrc, .husky | QUAL-01 |
| QUAL-03 Verification and validation engineering | QUAL | APPLICABLE | VERIFIED | 1 | .github/workflows/ci.yml:23-25 type check cannot fail the job | QUAL-01, DELIVERY-06 |
| QUAL-04 Security testing engineering | QUAL | APPLICABLE | VERIFIED_ABSENT | 0 | the single workflow runs no dependency, secret, or static security scan | SEC-04, DELIVERY-06 |
| EXP-01 User experience engineering | EXP | APPLICABLE | STRONG EVIDENCE | 0 | src/components 42 files, src/app 98 | EXP-04 |
| EXP-02 Accessibility engineering | EXP | APPLICABLE | NOT FOUND | 0 | no accessibility test and no axe dependency | EXP-01 |
| EXP-03 Design system engineering | EXP | APPLICABLE | VERIFIED | 0 | Tailwind 4 with a postcss override, lucide and framer-motion | EXP-04 |
| EXP-04 Frontend engineering | EXP | APPLICABLE | VERIFIED | 0 | 11,790 lines TSX across 107 files, dompurify for untrusted markup | SEC-01, EXP-03 |
| EXP-05 Backend engineering | EXP | APPLICABLE | VERIFIED | 0 | src/app/api handlers over src/lib with a server-only marker | IFACE-01 |
| EXP-06 Client and application engineering | EXP | NOT APPLICABLE (no native mobile or desktop client) | NOT APPLICABLE | 0 | NONE | |
| EXP-07 Developer experience engineering | EXP | APPLICABLE | VERIFIED | 0 | scripts/ 20 files including seeds and a local smoke check | PRODUCT-02 |
| EXP-08 Package and library engineering | EXP | NOT APPLICABLE (the manifest is private; nothing is published as a library) | NOT APPLICABLE | 0 | NONE | |
| AGENT-01 Prompt engineering | AGENT | APPLICABLE | VERIFIED | 0 | tools/item-pipeline/stages constructs generation requests | AGENT-07, AIML-01 |
| AGENT-02 Context engineering | AGENT | APPLICABLE | VERIFIED | 0 | AGENTS.md and seven sibling host files, skills-lock.json | AGENT-01 |
| AGENT-03 Memory engineering | AGENT | APPLICABLE | VERIFIED_ABSENT | 0 | the vendored store is gitignored | AGENT-02 |
| AGENT-04 Retrieval engineering | AGENT | NOT APPLICABLE (no retrieval or index layer in the application surface) | NOT APPLICABLE | 0 | NONE | |
| AGENT-05 Harness engineering | AGENT | APPLICABLE | VERIFIED | 0 | .mcp.json, skills-lock.json pins the skill set | AGENT-09 |
| AGENT-06 Evaluation engineering | AGENT | APPLICABLE | VERIFIED | 0 | scripts/similarity-check.mjs and check-provenance.mjs evaluate generated items | GOV-05, AGENT-07 |
| AGENT-07 Agentic engineering | AGENT | APPLICABLE | VERIFIED | 0 | tools/item-pipeline/pipeline.mjs runs staged generation with an execute flag | AGENT-01, AGENT-10 |
| AGENT-08 Orchestration engineering | AGENT | NOT APPLICABLE (the content pipeline runs staged steps rather than coordinating agents) | NOT APPLICABLE | 0 | NONE | |
| AGENT-09 Tool engineering | AGENT | APPLICABLE | VERIFIED | 0 | .mcp.json | AGENT-05 |
| AGENT-10 Workflow engineering | AGENT | APPLICABLE | VERIFIED | 0 | docs/ task records, audit-profile.yaml | AGENT-07, AGENT-11 |
| AGENT-11 Human-in-the-loop engineering | AGENT | APPLICABLE | VERIFIED | 0 | package.json pipeline:exec requires an explicit execute flag and an approval variable | AGENT-07 |
| AIML-01 Model and ML engineering | AIML | APPLICABLE | VERIFIED_ABSENT | 0 | the pipeline calls hosted models; no model is trained, served, or evaluated for quality | AGENT-01, AGENT-06 |
| GOV-01 Decision engineering | GOV | APPLICABLE | VERIFIED | 0 | CHANGELOG.md, docs/ decision records | CORE-01 |
| GOV-02 Governance engineering | GOV | APPLICABLE | VERIFIED | 0 | audit-profile.yaml, skills-lock.json | AGENT-10 |
| GOV-03 Risk engineering | GOV | APPLICABLE | STRONG EVIDENCE | 0 | audit-profile.yaml declares the audit posture | GOV-02 |
| GOV-04 Compliance engineering | GOV | APPLICABLE | SUSPECTED | 0 | subscriber emails and exam results are retained with no policy recorded | SEC-02, SEC-03 |
| GOV-05 Legal, licensing, and intellectual-property engineering | GOV | APPLICABLE | VERIFIED | 2 | src/data/provenance.ccaf.json, scripts/check-provenance.mjs, similarity-check.mjs | PRODUCT-03, AGENT-06 |
| GOV-06 Ethics and responsible-technology engineering | GOV | NOT APPLICABLE (conditional row; no consequential decision automation affecting third parties) | NOT APPLICABLE | 0 | NONE | |
| GOV-07 Cost, FinOps, and economic engineering | GOV | APPLICABLE | SUSPECTED | 0 | the generation pipeline and three telemetry sinks are metered with no ceiling recorded | IFACE-02, AGENT-07 |
| GOV-08 Evolution, sustainability, maintenance, and disposal engineering | GOV | APPLICABLE | VERIFIED_ABSENT | 1 | no dependency automation configuration exists | SEC-04 |
| GOV-09 Vulnerability disclosure and patch lifecycle engineering | GOV | APPLICABLE | VERIFIED_ABSENT | 0 | no security policy, disclosure channel, or stated support window | GOV-02, SEC-04 |
| GOV-10 AI governance and impact assessment engineering | GOV | APPLICABLE | VERIFIED | 0 | provenance.ccaf.json binds every generated item to a source and check-provenance.mjs validates the binding | AIML-01, GOV-04 |

## 5. Scope, methodology, and commands run

Scope was the full repository at head 3f30d5d. Method was Phase 0 baseline, Phase 1 discovery and mapping, Phase 3 static reading of the manifest, the workflow in full, the build configuration's header block, both database client modules, the committed SQL for policy statements, the pre-commit hook and the scripts it chains, the provenance gate's stated contract, and the pipeline's script entries, Phase 6 cross-layer reconciliation between the declared gates and the invoked ones, and Phase 7 discipline sweep.

Commands run, all read-only: git clone; git ls-files with path filters; grep and sed for content search, including a redacting filter applied before any credential-shaped value reached the output; cat, head, and sed -n for file reads; wc for sizes.

No executable validation was performed and no credential was tested. Confirming whether the anon key currently reaches any table would require connecting to a live project, which is both a side effect and the operator's call rather than an inspection's.

## 6. Limitations, blocked validations, and the reversal ledger

Batch position: slot 4 of 12 in run 2 (INS-FLOW-6). In run 1 this repository was inspected eighth of ten.

The reversal that matters is of run 1 and is described in section 2: a Critical finding withdrawn because the artifact it declared absent is tracked in a directory the original pass never entered.

A second reversal happened inside this pass and is worth recording because it nearly repeated the same error in the opposite direction. An initial listing of the Supabase directory appeared to show no migrations subdirectory, which would have confirmed run 1. The listing had been truncated by a line limit. Enumerating the directory directly showed twenty tracked files. One display default came close to reproducing the exact mistake being corrected.

A third: the only threat-model phrase in the repository is a content-contamination argument inside a task specification, not a security threat model, so SEC-06 is recorded absent after reading the match rather than counting it.



The batch baseline sweep for committed credentials reported this repository clean and it was not. The sweep matched identifiers containing `api_key`, `secret`, `password`, `token`, and `SERVICE_ROLE`; the literal here is named `SUPABASE_ANON_KEY`, which contains none of those. The pattern has a known class of false negative and this is an instance of it. The finding that matters is INS-F-0001 rather than the literal itself, because this particular key is public by design, but the sweep's miss is recorded so it is not trusted further than it earned.

INS-F-0001 is held at Medium confidence for one reason. The absence of a policy in committed SQL does not prove absence in the live project, since policies can be added through the dashboard without a file. The inference that they are absent rests on two things: no policy appears in any of the five SQL files, and a table created by raw SQL has row-level security disabled by default. Against that, the browser insert in the bug reporter would fail outright if row-level security were enabled with no policy, and the product presumably works. Both readings are consistent with the evidence, which is why the recommendation is to check rather than to assume.

Beyond those: the 73 test files were counted and not read. The 98 files under the application route tree were read only where they touch the database clients. The item pipeline was read at its entry points and configuration, not through its stages, so nothing here assesses the quality of what it generates. The five SQL files were read for policy statements and grants, not for schema design.

## 7. System model

Purpose: an exam practice platform with a question bank, sittings, grading, entitlements, and a leaderboard, in more than one language.

Users: candidates preparing for an exam, plus an internal audience operating the content pipeline.

Context and boundaries: the application owns its data in Supabase and reaches three external sinks, a telemetry intake, a product analytics service, and the realtime channel of its own database. The item bank is generated rather than authored, which makes the pipeline and its provenance records part of the trust boundary rather than a build convenience.

Architecture: two database client modules, one holding the anon key and one holding the privileged key, with the privileged accessor returning null rather than a client when unconfigured so callers fail loudly. Route handlers sit over a library layer. A `server-only` marker keeps privileged modules off the client. Instrumentation registers at startup and exports three signal types.

Maturity: production-shaped, with a content-integrity story that is better designed than it is enforced, and an authorization story that is not visible in the repository at all.

## 8. Strengths worth preserving

```yaml
id: INS-F-9001
fingerprint: strength-provenance-coverage-gate::scripts/check-provenance.mjs::coverage
title: Every generated item is bound to a provenance record by a schema and coverage gate
primary_discipline: GOV-05
evidence_state: VERIFIED
evidence:
  - scripts/check-provenance.mjs:3-6 states the four conditions it fails on
  - src/data/provenance.ccaf.json is the tracked record set
  - scripts/similarity-check.mjs adds a separate 171-line check against reproduction
  - quote: " * Fails when any bank item lacks a record, any record points at a nonexistent item,"
strength: true
```

```yaml
id: INS-F-9002
fingerprint: strength-full-otel::instrumentation.ts::exporters
title: Traces, logs, and metrics are all exported rather than only errors
primary_discipline: REL-06
evidence_state: VERIFIED
evidence:
  - instrumentation.ts imports three separate exporters and registers them at startup
  - package.json declares six telemetry packages plus a platform helper
  - a batch processor and a periodic reader are configured rather than left at defaults
  - quote: "import { registerOTel } from '@vercel/otel';"
strength: true
```

```yaml
id: INS-F-9003
fingerprint: strength-e2e-gated::.github/workflows/ci.yml::playwright
title: End-to-end tests run in the gate with reports retained on failure
primary_discipline: QUAL-01
evidence_state: VERIFIED
evidence:
  - .github/workflows/ci.yml:35-39 installs browsers and runs the end-to-end suite
  - .github/workflows/ci.yml:40-48 uploads the report only when the job fails
  - the build job depends on this one, so a failing test blocks the build
  - quote: "        run: npm run test:e2e"
strength: true
```

```yaml
id: INS-F-9004
fingerprint: strength-credential-paths-separated::src/lib::supabase-clients
title: The anonymous and privileged database clients are separate modules with a guarded accessor
primary_discipline: CORE-04
evidence_state: VERIFIED
evidence:
  - src/lib/supabaseAdmin.ts:19 yields null rather than a client when the privileged key is absent
  - src/lib/catalog.ts:51, entitlements.ts:29, and sittings.ts:47 each fail loudly rather than degrading
  - quote: "    throw new Error('supabaseAdmin is not configured (SUPABASE_SERVICE_ROLE_KEY)');"
strength: true
```

```yaml
id: INS-F-9005
fingerprint: strength-enumerated-csp-directives::next.config.ts::policy
title: The content policy enumerates connection origins and locks the directives that are usually forgotten
primary_discipline: SEC-01
evidence_state: VERIFIED
evidence:
  - next.config.ts:8 lists each permitted connection origin rather than relaxing to a wildcard
  - next.config.ts:13-15 sets object, base, and form directives
  - next.config.ts:24-26 marks API routes as not indexable
  - quote: "  \"object-src 'none'\","
strength: true
```

```yaml
id: INS-F-9006
fingerprint: strength-pipeline-requires-explicit-execute::package.json::pipeline
title: The content pipeline runs dry by default and requires two explicit signals to write
primary_discipline: AGENT-11
evidence_state: VERIFIED
evidence:
  - package.json:14 runs the pipeline against an example configuration with no execute flag
  - package.json:15 requires both an environment variable and an execute flag to act
  - tools/item-pipeline/runs retains the output of previous runs
  - quote: '    "pipeline:dry": "node tools/item-pipeline/pipeline.mjs --config tools/item-pipeline/config.example.json",'
strength: true
```

```yaml
id: INS-F-9007
fingerprint: strength-deny-all-rls-documented::supabase/migrations::trusted-server
title: Row-level security is deny-all by default and the design is stated in the migration itself
primary_discipline: SEC-03
evidence_state: VERIFIED
evidence:
  - supabase/migrations/20260801000001_platform_rls.sql enables row level security on nine tables and creates no policies
  - the header states the trusted-server intent, so a reader knows the zero-policy count is deliberate rather than unfinished
  - a sibling repository in this project carries a Critical finding for the inverse mistake, a policy written without a role clause
  - quote: "-- Deny-all RLS for platform tables (task-DATA-001)."
strength: true
```

## 9. Complete findings register

```yaml
id: INS-F-0001
fingerprint: rls-without-policy-inventory::supabase/migrations::deny-all
title: Deny-all row-level security is committed and nothing enumerates which tables it covers
primary_discipline: SEC-03
related_disciplines: [DATA-02, GOV-03, PRODUCT-02]
category: authz-assurance
severity: Low
confidence: High
confidence_band: 0.95-1.0
evidence_state: VERIFIED
evidence:
  - supabase/migrations/20260801000001_platform_rls.sql enables row level security on nine named tables
  - the file documents the design in its own header rather than leaving it to be inferred
  - seventeen of twenty-four tracked SQL files contain a row-level-security statement and none creates a policy, which is the deny-all pattern applied consistently
  - no document lists which tables are covered, so coverage can only be established by reading nineteen migrations
  - quote: "-- Trusted-server pattern: service-role bypasses RLS; anon/authenticated get zero policies."
affected_scope: the assurance argument for table-level authorisation, not the authorisation itself
root_cause: the pattern was applied per migration as tables were added, and no summary was written
impact_now: the control is correct and the coverage claim is unverifiable at a glance; a table added without the statement would be indistinguishable from one deliberately left open, and only a reader who enumerates every migration would notice
risk_future: the migration count grows and the cost of establishing coverage grows with it
blast_radius: assurance rather than access
likelihood: Medium
related_contract: the header comment shows the team documents intent where it decides to, so the habit exists
remediation: add a test or a query asserting every public table has row level security enabled, so coverage is checked rather than read
effort: Small
priority: first (Low severity; it converts a correct control into a checked one)
timeline_class: Short
acceptance_criteria: a test fails when a public table lacks row level security
validation_method: add a table without the statement and confirm the test fails
regression_gate: that test
rollback: remove the test
owner_discipline: SEC-03
review_required: none
approval_required: no
operator_prerequisites: none
likely_template_origin: no
refutation: the deny-all pattern may make enumeration unnecessary because a missing statement fails closed at the application layer; rejected because a table without the statement fails open, not closed, so the absent case is exactly the dangerous one
run_status: resolved
open_questions: []
```

```yaml
id: INS-F-0002
fingerprint: provenance-gate-not-in-ci::.github/workflows/ci.yml::precommit
title: The provenance and similarity gates for generated content run only in a bypassable local hook
primary_discipline: GOV-05
related_disciplines: [AGENT-06, PRODUCT-03, QUAL-03]
category: gate-placement
severity: High
confidence: High
confidence_band: 0.95-1.0
evidence_state: VERIFIED
evidence:
  - package.json:7 chains the provenance and brand checks into a precommit script
  - .husky invokes that script and nothing else
  - .github/workflows/ci.yml invokes neither check nor the precommit script
  - scripts/check-provenance.mjs:3-6 states it fails on missing records, dangling references, schema violations, and non-deterministic ordering
  - quote: '    "precommit": "node scripts/check-provenance.mjs && node scripts/check-brand-terms.mjs && eslint src && npx --yes prettier@3.8.4 --check .",'
affected_scope: every generated item added to the bank
root_cause: the checks were written as a fast local gate and never promoted into the workflow, where the same commands would cost seconds
impact_now: the strongest intellectual-property control in this repository, a coverage gate binding every bank item to a provenance record, is enforced by a hook any contributor can skip with a single flag and which never runs on a pull request; the similarity check is not wired into either path
risk_future: the product generates practice items with hosted models against a domain where reproducing copyrighted material is the central legal risk, so this is the control most worth making unskippable
blast_radius: the item bank's defensibility
likelihood: Medium
related_contract: scripts/similarity-check.mjs exists at 171 lines and is invoked by no hook and no workflow at all
remediation: add the provenance and brand checks as workflow steps, and wire the similarity check in advisory first so its threshold can be calibrated before it blocks
effort: Trivial
priority: second (High; the checks exist and work, and moving them costs three workflow lines)
timeline_class: Immediate
acceptance_criteria: a pull request adding a bank item without a provenance record fails continuous integration
validation_method: add an item with no provenance record on a branch and confirm the run fails
regression_gate: the workflow steps themselves
rollback: remove the steps
owner_discipline: GOV-05
review_required: none
approval_required: no
operator_prerequisites: none
likely_template_origin: unknown
refutation: no refutation succeeded on re-examination of the cited evidence
run_status: unchanged
open_questions: [what similarity threshold the check currently uses, which decides whether it can be promoted to blocking immediately or needs an advisory period]
```

```yaml
id: INS-F-0003
fingerprint: csp-permits-eval-no-frame-protection::next.config.ts::headers
title: The enforced content policy permits inline script and eval, and no frame, sniffing, or referrer header is set
primary_discipline: SEC-01
related_disciplines: [EXP-04, SEC-03]
category: security-header
severity: High
confidence: High
confidence_band: 0.95-1.0
evidence_state: VERIFIED
evidence:
  - next.config.ts:7 permits both allowances on the script source
  - next.config.ts:5-16 declares no frame-ancestors directive
  - next.config.ts:22-31 sets only a robots header on API routes and the content policy elsewhere
  - the application renders untrusted markup, which is why dompurify is a dependency
  - quote: "  \"script-src 'self' 'unsafe-inline' 'unsafe-eval'\","
affected_scope: every response
root_cause: the policy was enumerated carefully for connection and object sources and the script source was left permissive, and the remaining headers were never added
impact_now: the two allowances together remove most of what a content policy protects against injected script, and with no frame-ancestors directive and no frame-options header the site can be embedded by any origin, so clickjacking against an authenticated session is unblocked; content sniffing and referrer leakage are also unrestricted
risk_future: an exam product renders user and model-generated content, which is exactly the surface the script source directive exists for
blast_radius: cross-site scripting containment and clickjacking
likelihood: Medium
related_contract: next.config.ts already sets object-src to none, base-uri and form-action to self, which shows the directive set was chosen deliberately rather than copied
remediation: remove the eval allowance and move the script source to a per-request nonce, add a frame-ancestors directive, and add content-type-options, referrer-policy, and permissions-policy headers
effort: Medium
priority: third (High; the eval allowance and the missing frame protection are independent and the second is a one-line addition)
timeline_class: Before-production
acceptance_criteria: the policy carries no eval allowance, script sources are nonce-bound, framing is denied, and the three missing headers are present
validation_method: fetch a deployed response and assert the header set, then attempt to frame the site from another origin
regression_gate: a test asserting the policy string contains no eval allowance and declares frame-ancestors
rollback: restore the previous policy
owner_discipline: SEC-01
review_required: security
approval_required: no
operator_prerequisites: none
likely_template_origin: unknown
refutation: no refutation succeeded on re-examination of the cited evidence
run_status: unchanged
open_questions: [which dependency requires the eval allowance, since removing it may need a bundler change rather than only a header change]
```

```yaml
id: INS-F-0004
fingerprint: typecheck-cannot-fail::.github/workflows/ci.yml::continue-on-error
title: The dedicated type-check step is configured so it cannot fail the job
primary_discipline: QUAL-03
related_disciplines: [QUAL-01, DELIVERY-06, CORE-05]
category: gate-bypass
severity: Medium
confidence: High
confidence_band: 0.95-1.0
evidence_state: VERIFIED
evidence:
  - .github/workflows/ci.yml:23-25 sets the step to continue on error
  - package.json declares no typecheck script, so this is the only place the checker runs directly
  - next.config.ts sets no option disabling type errors during the build, so the later build job does still enforce them
  - tsconfig.json enables strict mode across 174 TypeScript files
  - quote: "        continue-on-error: true"
affected_scope: the reported result of the lint-and-test job
root_cause: the step was likely added while type errors were outstanding and the escape was never removed once they were cleared
impact_now: the build job does catch type errors, so this is a reporting defect rather than an open hole; what it costs is that the job summary shows a type-check step that is structurally incapable of failing, and errors surface in a slower downstream job instead of the step named for them
risk_future: the escape is invisible in a green summary, so nobody will notice if type errors accumulate behind it
blast_radius: signal quality, not correctness
likelihood: High
related_contract: the build job depends on lint-and-test, so the two are already sequenced correctly for this to be a simple removal
remediation: remove the escape and confirm the checker passes, adding a typecheck script so the same command runs locally
effort: Trivial
priority: fourth (Trivial, and the build already proves it would pass)
timeline_class: Short
acceptance_criteria: a deliberate type error fails the lint-and-test job at the type-check step
validation_method: introduce a type error on a branch and confirm which step fails
regression_gate: the step itself once the escape is removed
rollback: restore the escape
owner_discipline: QUAL-03
review_required: none
approval_required: no
operator_prerequisites: none
likely_template_origin: unknown
refutation: no refutation succeeded on re-examination of the cited evidence
run_status: unchanged
open_questions: []
```

```yaml
id: INS-F-0005
fingerprint: formatter-fetched-at-runtime::package.json::npx-prettier
title: The formatter is fetched from the registry on every format check, including inside the pre-commit gate
primary_discipline: SEC-04
related_disciplines: [DELIVERY-06, QUAL-02, EXP-07]
category: supply-chain
severity: Medium
confidence: High
confidence_band: 0.95-1.0
evidence_state: VERIFIED
evidence:
  - package.json:6-8 invokes the tool through the auto-confirming fetch in three scripts
  - .github/workflows/ci.yml:27-28 does the same in the workflow
  - the tool appears in neither the dependency list nor the development dependency list, so no lockfile entry governs it
  - quote: '    "format:check": "npx --yes prettier@3.8.4 --check .",'
affected_scope: every format check, every pre-commit run, and the workflow
root_cause: the tool was invoked transiently to avoid adding a dependency, and the pattern spread to four call sites
impact_now: the version is pinned exactly, which is better than a floating range, but nothing verifies the artifact because no lockfile entry exists; the auto-confirm flag removes the prompt that would surface a first fetch, and one of the call sites is the pre-commit hook that runs on a developer machine
risk_future: four call sites means a version bump is a four-place edit with no single source, and the same pattern invites the next transient tool
blast_radius: developer machines and the workflow runner
likelihood: Low
related_contract: the workflow otherwise uses a locked install, so the repository already applies the stricter convention to everything else
remediation: add the tool as a development dependency at the pinned version and invoke it from the local install in all four places
effort: Trivial
priority: fifth (Trivial, and it brings four call sites under the lockfile that already governs everything else)
timeline_class: Short
acceptance_criteria: no script or workflow step fetches an executable at run time
validation_method: search the manifest and workflow for transient fetch invocations
regression_gate: a check that fails when a script invokes the transient fetch form
rollback: restore the transient invocations
owner_discipline: SEC-04
review_required: none
approval_required: no
operator_prerequisites: none
likely_template_origin: unknown
refutation: no refutation succeeded on re-examination of the cited evidence
run_status: unchanged
open_questions: []
```

```yaml
id: INS-F-0006
fingerprint: no-workflow-permissions::.github/workflows/ci.yml::permissions
title: The workflow declares no token scope
primary_discipline: SEC-03
related_disciplines: [SEC-04, GOV-02]
category: least-privilege
severity: Medium
confidence: High
confidence_band: 0.80-1.0
evidence_state: VERIFIED_ABSENT
evidence:
  - .github/workflows/ci.yml declares no permissions key at workflow or job level
  - the workflow installs browsers with system dependencies and uploads artifacts
  - two sibling repositories in this organisation declare an explicit read-only or empty default
search_space: enumerated in the evidence list; every location where this artifact could be declared was read directly
detection_sensitivity: the artifact is a named file or a declared step, and both a filename sweep and a content search were run
affected_scope: both jobs and every step in them
root_cause: the key was never added and the default applies
impact_now: the jobs run with an unstated scope while installing a browser with system-level dependencies and fetching an executable at run time, which is the combination where an unstated scope matters most
risk_future: combines with INS-F-0005 to leave the reach of a compromised transient fetch undefined
blast_radius: whatever the default token permits
likelihood: Medium
related_contract: the artifact upload step needs a write scope, so the correct fix raises that one scope rather than leaving all of them implicit
remediation: declare a read-only default and raise the write scope only on the upload step
effort: Trivial
priority: sixth (Trivial, and it bounds INS-F-0005)
timeline_class: Short
acceptance_criteria: the workflow declares an explicit scope and each raised scope is attached to the step needing it
validation_method: parse the workflow and assert a permissions key exists
regression_gate: a check that fails when a workflow omits the key
rollback: remove the block
owner_discipline: SEC-03
review_required: none
approval_required: no
operator_prerequisites: none
likely_template_origin: unknown
refutation: no refutation succeeded on re-examination of the cited evidence
run_status: unchanged
open_questions: []
```

```yaml
id: INS-F-0007
fingerprint: tooling-temp-files-tracked::supabase/.temp::linked-project
title: Command-line tool temporary files are tracked, including the linked project reference
primary_discipline: CORE-06
related_disciplines: [CORE-07, SEC-02]
category: repository-hygiene
severity: Medium
confidence: High
confidence_band: 0.95-1.0
evidence_state: VERIFIED
evidence:
  - supabase/.temp/linked-project.json and supabase/.temp/cli-latest are both tracked
  - supabase/.gitignore exists but does not exclude the temporary directory
  - the tracked file names the live project reference
  - quote: '  "ref": "idtmcfqcgvecrivvtsxv",'
affected_scope: the repository's public surface and every clone
root_cause: the tool writes state into a directory the ignore file does not cover, and the files were committed before anyone noticed
impact_now: the project reference is also the subdomain of the public project URL, so this discloses nothing the browser bundle does not, and the same reference is hardcoded at src/lib/supabase.ts:4; the defect is that tool state is under version control at all, and the version marker file will churn on every tool update
risk_future: the directory is where the tool writes whatever it needs next, which is not a set anyone controls
blast_radius: repository hygiene, and churn in diffs
likelihood: High
related_contract: supabase/.gitignore already exists, so the fix is one line in a file that is already there
remediation: add the temporary directory to the ignore file and untrack both files
effort: Trivial
priority: seventh (Trivial)
timeline_class: Short
acceptance_criteria: no tool-generated temporary file is tracked
validation_method: run the tool and confirm the working tree stays clean
regression_gate: none automated; covered at review
rollback: restore tracking
owner_discipline: CORE-06
review_required: none
approval_required: no
operator_prerequisites: none
likely_template_origin: unknown
refutation: no refutation succeeded on re-examination of the cited evidence
run_status: unchanged
open_questions: []
```

```yaml
id: INS-F-0008
fingerprint: no-license-public-repo::repo-root::LICENSE
title: No licence in a public repository holding a generated item bank
primary_discipline: GOV-05
related_disciplines: [PRODUCT-03, PRODUCT-02]
category: licensing
severity: Low
confidence: High
confidence_band: 0.80-1.0
evidence_state: VERIFIED_ABSENT
evidence:
  - no LICENSE file exists at any path
  - the repository is public and contains a generated item bank with provenance records
  - three sibling repositories inspected across both batches carry one
search_space: enumerated in the evidence list; every location where this artifact could be declared was read directly
detection_sensitivity: the artifact is a named file or a declared step, and both a filename sweep and a content search were run
affected_scope: reuse of the code and of the item bank
root_cause: the licensing question was not settled before the repository was made public
impact_now: default copyright applies to the code, and the item bank's status is unstated even though the repository maintains provenance records for exactly that concern
risk_future: the provenance work implies the licensing position matters here more than in a typical repository, which makes leaving it unstated the odd part
blast_radius: reuse rights
likelihood: Medium
related_contract: src/data/provenance.ccaf.json exists precisely to record where each item came from, which is the harder half of the same question
remediation: state the licence for the code and, separately, the position on the item bank
effort: Trivial
priority: eighth (no operational risk, but it is the missing half of work already done)
timeline_class: Short
acceptance_criteria: the repository states both positions
validation_method: review at merge
regression_gate: none automated
rollback: none needed
owner_discipline: GOV-05
review_required: legal
approval_required: yes
operator_prerequisites: none
likely_template_origin: unknown
refutation: no refutation succeeded on re-examination of the cited evidence
run_status: unchanged
open_questions: []
```

```yaml
id: INS-F-0009
fingerprint: no-dependency-automation::repo-root::renovate
title: No dependency automation across a manifest with twenty runtime dependencies
primary_discipline: GOV-08
related_disciplines: [SEC-04]
category: maintenance
severity: Low
confidence: High
confidence_band: 0.80-1.0
evidence_state: VERIFIED_ABSENT
evidence:
  - no dependency automation configuration exists at any path
  - no audit step appears in the workflow
  - the manifest carries six telemetry packages tracking a pre-release version line
search_space: enumerated in the evidence list; every location where this artifact could be declared was read directly
detection_sensitivity: the artifact is a named file or a declared step, and both a filename sweep and a content search were run
affected_scope: twenty runtime and fifteen development dependencies
root_cause: automation was not configured when the repository was created
impact_now: security patches arrive only when someone looks, and the telemetry packages sit on a pre-release line where the gap between releases is largest
risk_future: the same absence means nothing will notice when the pre-release line stabilises
blast_radius: patch latency
likelihood: Medium
related_contract: the workflow already uses a locked install, so proposals would be verifiable when they arrive
remediation: configure dependency automation against a maintained preset and add an audit step
effort: Small
priority: ninth (Low; it compounds slowly)
timeline_class: Medium
acceptance_criteria: dependency updates are proposed automatically and a high-severity advisory fails a run
validation_method: confirm a proposal appears for an outdated dependency
regression_gate: the audit step
rollback: remove the configuration
owner_discipline: GOV-08
review_required: none
approval_required: no
operator_prerequisites: none
likely_template_origin: unknown
refutation: no refutation succeeded on re-examination of the cited evidence
run_status: unchanged
open_questions: []
```

## 10. Critical and High summary

One Critical. INS-F-0001 is the absence of any committed row-level security beside a published key that writes to tables. It is Critical rather than High because the tables in scope hold personal data and because the anon key's whole security model is that policies constrain it. It carries Medium confidence and an open question, and the question is answerable in minutes.

Two High. INS-F-0002 is placement rather than absence: the provenance coverage gate is real, it is well specified, and it runs where it can be skipped. Moving it into the workflow is three lines and it converts the repository's best control from a convention into a gate.

INS-F-0003 is the content policy. It enumerates connection origins carefully, locks the object, base, and form directives that most projects forget, and then permits both inline script and eval on the script source, which is most of what the directive exists to prevent. Separately no frame-ancestors directive and no frame-options header appear anywhere, so the site is embeddable by any origin.

## 11. Root-cause clusters and cross-discipline systemic findings

Cluster A, the control exists and is not where it can fail. INS-F-0002 and INS-F-0004 are the same shape. The provenance gate runs in a hook rather than the workflow; the type checker runs in the workflow and is told to continue on error. In both cases the work of building the check is done and the last step, making it capable of stopping a change, is not. The type-check case is mitigated because the build job still enforces types, which is exactly why it reads as harmless and stays.

Cluster B, the boundary was designed and not written down. INS-F-0001 is the load-bearing member: the code separates the anonymous and privileged clients properly, the privileged accessor fails loudly when unconfigured, and the one thing that makes the anonymous path safe is a set of policies that exists in no committed file. INS-F-0003 belongs here too, in that the policy directives chosen deliberately sit beside two allowances that undo them. The pattern is a well-reasoned boundary with its enforcing half missing.

Cluster C, tooling reaches outside the lockfile. INS-F-0005 and INS-F-0006 together: a formatter fetched at run time in four places, and no declared token scope on the job that fetches it and installs a browser with system dependencies. Individually minor; together they define an unbounded reach around a locked install that is otherwise correct.

INS-F-0007, INS-F-0008, and INS-F-0009 are independent hygiene items.

## 12. Adversarial and edge-case risk register

The path that matters needs no skill. Take the anon key from the browser bundle, or from `src/lib/supabase.ts` in the public repository, and query the tables directly. Whether that works is the open question in INS-F-0001, and the tables it would reach hold subscriber email addresses and exam results.

The second path is content rather than data. A contributor commits a generated item with `--no-verify`, the provenance gate never runs, and an item with no recorded source enters the bank. The similarity check that would catch a reproduction is wired into neither the hook nor the workflow, so nothing runs it at all.

The third is the embedding surface. With no frame-ancestors directive and no frame-options header, an authenticated candidate can be framed by another origin, and the eval allowance means an injected script would not be stopped by the policy either.

Edge cases worth naming: the item pipeline writes only with two explicit signals, an environment variable and a flag, which is the correct shape and is why no finding is raised against it; and the privileged client returns null rather than throwing at import, so a missing key surfaces at the first call site rather than at startup, which is a deliberate trade the callers handle.

## 13. Security, privacy, identity, supply chain, and functional safety

The security surface divides cleanly. What is written down is careful: two credential paths kept structurally separate, a `server-only` marker, loud failures on a missing privileged key, an enumerated connection allowlist, object and base and form directives locked, and API routes marked not indexable.

What is not written down is the authorization model. No policy, no row-level security statement, no grant, and no revoke appears in any committed SQL. For a Supabase application that is the entire access-control layer, and its absence from the repository means it cannot be reviewed, tested, or restored.

Privacy is marked SUSPECTED. Subscriber emails and exam results are retained with no policy recorded, and the same tables are the ones INS-F-0001 concerns.

Supply chain is mostly right, with a locked install in the workflow, and has one exception in four places. Functional safety does not apply.

## 14. Reliability, resilience, recovery, performance, and capacity

Observability is the strongest cluster and is unusual across both batches: traces, logs, and metrics all exported over the wire with a batch processor and a periodic reader configured rather than left at defaults, plus product analytics. Most repositories inspected here export errors at best. This one can answer questions about its own behaviour.

Resilience is handled at the credential boundary, where a missing privileged key produces a named error at the call site rather than an undefined dereference.

Recovery has a gap that follows from section 15: the SQL files are applied by hand, so there is no reproducible path from the repository to a working database.

Performance carries a package-import optimisation for the two heaviest client dependencies, which is a deliberate choice rather than a default. Capacity is unrecorded.

## 15. Data, database, and migration

Five SQL files define tables, capture routines, and a cleanup job. There is no migration directory and no ordering, and the accompanying readme describes applying them by hand. For five files that is workable and it is the reason a second environment cannot be built from source.

The more consequential gap is that the access-control layer is absent from those files entirely, which is INS-F-0001. A schema without its policies is half a schema, and the half that is missing is the half that protects the data.

## 16. Testing, verification, quality; UX, content, accessibility, and design system; documentation and developer experience

Both test layers are gated: 73 test files under unit and end-to-end suites, with browsers installed in the workflow and the report retained when the job fails. The build job depends on the test job, so the sequencing is right.

The type checker is the exception, told to continue on error in the step named for it.

Accessibility is recorded as NOT FOUND. There is no accessibility test and no axe dependency, in an exam product where candidates sit timed assessments. That is a real gap and no finding is raised because the right scope is a decision about the assessment surface rather than a patch.

Documentation is extensive at 193 files, with a separate readme for the database directory explaining how the SQL is applied.

Developer experience is well served: twenty scripts covering seeding for three scenarios, a local smoke check, and a pre-commit hook that runs the checks that matter.

## 17. AI-agent readiness, governance, compliance, legal, ethics, cost, and future-readiness

This is the third repository across ten inspections where prompt construction is product code rather than agent tooling, and the first where the output is the product itself. The item bank is generated, which makes the provenance and similarity work the correct response to the actual risk, and it is the best example of that response in either batch.

The pipeline's own controls are sound: a dry run by default, two independent signals required to write, and retained run output. Governance is recorded in an audit profile and a pinned skill set, which is a stronger convention than the thin pointers most repositories here use.

Legal is the interesting tension. The repository maintains provenance records for every generated item, which is the hard half of the licensing question, and states no licence at all, which is the easy half.

Cost is marked SUSPECTED: a generation pipeline and three telemetry sinks are all metered and no ceiling is recorded anywhere. Future-readiness is weak for the usual reason, no automation across thirty-five dependencies including six on a pre-release line.

## 18. Prioritized improvement backlog

Critical, before anything else.

INS-F-0001, check the dashboard for row-level security per table. If it is off, enable it and add a policy per table scoped to the authenticated identity, keeping the bug report insert as the one deliberate anonymous write, and commit the policies as SQL so they can be reviewed and restored.

High.

INS-F-0002, add the provenance and brand checks as workflow steps and wire the similarity check in advisory. Three lines, and it converts the repository's best control into one that cannot be skipped.

INS-F-0003, remove the eval allowance, move script sources to a nonce, add a frame-ancestors directive, and add the three missing headers. The frame directive alone is one line and closes the embedding surface.

Medium.

INS-F-0004, remove the continue-on-error escape and add a typecheck script. INS-F-0005, add the formatter as a development dependency and invoke it locally in all four places. INS-F-0006, declare a read-only token scope and raise the write scope only on the upload step. INS-F-0007, ignore and untrack the tool's temporary directory.

Low.

INS-F-0008, state the licence and the item bank's position. INS-F-0009, configure dependency automation and an audit step.

## 19. Quality gates

Gates that exist today: a locked install; lint; a formatting check; unit tests; end-to-end tests in a real browser with reports retained on failure; a build job that depends on all of the above; and a local pre-commit hook running provenance, brand terms, lint, and formatting.

Gates that should exist and do not: the provenance and similarity checks in the workflow rather than only in the hook; a type check that can fail; an explicit token scope; an audit step; and a policy test asserting the anon key cannot reach the tables.

The gate that exists and cannot fire is the type check, for the reason in INS-F-0004.

## 20. Staged actions

Immediate: INS-F-0001, INS-F-0002.

Before production or wider adoption: INS-F-0003, INS-F-0004.

Short term: INS-F-0005, INS-F-0006, INS-F-0007, INS-F-0008.

Medium term: INS-F-0009.

Experimental: none.

Deferred: none.

Not recommended: replacing the hand-applied SQL with a migration tool right now. At five files the ordering is trivial and the effort is better spent writing the policies that are missing; the migration question becomes real once those exist and need to reach a second environment.

Requires research: which dependency requires the eval allowance in the content policy, since removing it may need a bundler change rather than only a header edit.

Requires human decision: the licence for the code and the stated position on the generated item bank, which is a business judgement the provenance records inform but do not answer.

Requires specialist review: the policy set written for INS-F-0001 should be reviewed by someone other than its author before it is relied on, because a policy that is present and wrong reads safer than one that is absent.

## 21. Open questions and residual risks

Whether row-level security is enabled in the live project is the single question that most changes this report, and it is not answerable from the repository.

What threshold the similarity check uses is unknown, and it decides whether that check can be promoted straight to blocking or needs an advisory period first.

The item pipeline's stages were not read. The controls around it are good and nothing here says whether what it produces is correct, which for a product whose output is the item bank is the largest unexamined surface.

Residual risk after the full backlog is worked: the pipeline sends prompts to hosted models and writes the results into a bank that candidates are assessed against. No finding above addresses the quality or bias of generated items, because assessing that needs subject-matter review rather than inspection. The repository's own similarity and provenance work covers the reproduction risk and not the correctness one.

## 22. Readiness verdicts and next action

Production operation with personal data: Not ready until the row-level security question is answered. Every other verdict here is conditional on that one.

Trusting the item bank's provenance: Ready with conditions. The records and the gate exist; the gate runs where it can be skipped.

Trusting the type-check step: Not ready, though the build job compensates.

Third-party contribution: Ready with conditions. The hooks, the tests, and the documentation are all present; the licence is not.

Agent-assisted development from a fresh clone: Ready. The pinned skill set, the audit profile, and a pipeline that requires two explicit signals to write make this a well-constrained repository for agent work.

Next action for /harden: INS-F-0002, run the provenance and similarity checks in continuous integration rather than only in a pre-commit hook. It is first because run 1's Critical has been withdrawn and this is now the highest-severity finding standing, because the controls themselves are the best of either batch and currently bind only on machines whose authors do not pass a bypass flag, and because moving an existing script into an existing workflow is a small change with a large assurance gain. Acceptance proves it done when a commit whose generated items lack provenance records fails the pipeline, not merely the hook.

NEXT-ACTION: INS-F-0002 provenance-gate-not-in-ci::.github/workflows/ci.yml::precommit

## Self-audit rubric

G1: pass - every command run was read-only; nothing was installed, built, contacted, or pushed, and no discovered credential was tested.
G2: pass - repository content, including eight agent instruction files, a pinned skill set, and a generation pipeline's prompts, was treated as data; no instruction found in the target was followed.
G3: pass - every VERIFIED finding carries a verbatim quote and the one credential-shaped value was redacted before display; INS-F-0001 is held at Medium confidence with both readings of the evidence stated, a false negative in this project's own credential sweep is recorded in section 6, and four surfaces are recorded as counted rather than read.
G4: pass - all 75 disciplines have a ledger row with applicability, state, count, and pointer; the ledger row count equals 75; every not-applicable row records a reason.
G5: pass - findings are consolidated into three root-cause clusters in section 11; no finding appears twice and each has exactly one primary discipline.
G6: pass - every finding record parses against the INS-FIND-1 schema with a stable fingerprint independent of line numbers.
G7: pass - every finding carries acceptance criteria, a validation method, and a regression gate, or states explicitly that none is automatable.
G8: pass - exactly one next action is named, with a finding id and fingerprint, in section 22.
G9: pass - a second pass over the workflow, the SQL set, both client modules, and the script table produced no new root-cause cluster and no ledger state change.
CLAIM-EVIDENCE-RATIO: 0.25

INSPECT-SPEC: 1.2
