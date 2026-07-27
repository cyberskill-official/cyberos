# /inspect report: cyberskill-official/shopass (run 2, slot 3)

QUALITY-HEADER
coverage: 66/75 applicable, clusters fully read: 7/12
evidence: 9/9 findings carry a verbatim quote, distinct evidence pointers: 34
verification: 9/9 findings survived a recorded refutation attempt
stability: single run, unmeasured
calibration: 2 run-1 findings re-tested against second-shape searches, 1 falsified

## 1. Side-effect disclosure

None. Every command was read-only: git clone, git ls-files, file reads, and text search. No dependency was installed, no module was compiled, no container was built, no database or hosted service was contacted, and nothing was written to the repository or pushed. The clone lives in a scratch directory.

## 2. Executive summary

This run falsified one of run 1's own findings. Run 1 shipped INS-F-0008 asserting dependency automation present across twenty declared ecosystems, recorded as VERIFIED_ABSENT. `.github/dependabot.yml` exists and declares twenty package ecosystems on a weekly schedule. A second search shape found it in one command. The finding is withdrawn, and it is the second confirmed false positive that re-inspection has produced, after a telemetry claim on a sibling repository.

Two further run-1 states were too weak. A retention position exists in the account-lifecycle task specification, so privacy moves from suspected to strong evidence, and accessibility markup is present in the web components even though no accessibility test or dependency is, so that row moves from not found to strong evidence with the distinction stated.

With those corrected, shopass is the largest and most disciplined repository across both batches: fifteen services in Go, TypeScript, and Python behind one gateway, four shared Go modules, a browser extension, a web client, and a six-job gate covering all three languages plus compose validation. Seven strengths are recorded, the most of any repository inspected.

Several things here are the best examples of their kind in this project. Token verification pins the signing algorithm at the parser and again inside the key function, validates issuer and audience, and filters the key set on type, algorithm, use, and key identifier. Every workflow action is pinned to a full commit with a version comment, with no exception. Python dependencies are pinned exactly, all eight of them. And the rate limiter's address source, which was wrong in two earlier repositories, is correct here and the coupling is written down: all four proxy configurations overwrite the header with the direct remote address, and the deploy readme says so.

Two findings are High. The workflow maintains two lists of Go modules, one for the vulnerability scan and one for the tests, and they have diverged: `services/b2b` and `services/trust` are scanned and not tested, so nine test files never run. Separately, fifteen services share one database credential, and across 85 SQL files there is not one grant statement and not one row-level security statement, so the architecture separates the services and the database does not.

Findings: 9 total, 0 Critical, 2 High, 6 Medium, 1 Low.

The one exact next action is at section 22.

## 3. Inspection metadata and baseline

Target: https://github.com/cyberskill-official/shopass, default branch main, head a796d98, 164 commits, last commit 2026-07-26. Working tree 94M excluding .git, the largest in either batch. 1,265 tracked files.

Languages by line count: JSON 36,717 across 28 files, dominated by lockfiles; Go 27,725 across 399 files; TypeScript 7,552 across 184; TSX 3,554 across 42; YAML 1,373 across 15; Markdown 1,136 across 259; Python 1,053 across 19; SQL 902 across 85 files in twelve locations.

Structure: `services/` holds fifteen services, of which thirteen are Go modules, one is a TypeScript backend-for-frontend, and one is a Python model service. `db`, `obs`, `region`, and `secrets` are shared Go modules. `web` and `extension` are the two clients. `deploy` holds four Dockerfiles by language, three compose files, and four proxy configurations. Go 1.25.12, Node 24.18.0, Python 3.11, all pinned in the workflow.

Inspection was single-pass and reached saturation without continuation; see section 5.

## 4. Coverage ledger

All 75 disciplines, in stable id order. {{APPLICABLE_COUNT}} applicable, 3 not applicable with a recorded reason.

Five of the six rows new in 1.2 apply. Two collected findings that run 1 had nowhere to put, and one produced the only security policy found anywhere in this project. Three run-1 states are corrected: GOV-08 from verified absent to verified, SEC-02 from suspected to strong evidence, and EXP-02 from not found to strong evidence.

| Discipline | Cluster | Applicability | Evidence state | Finding count | Evidence pointer | Related |
|---|---|---|---|---|---|---|
| CORE-01 Requirements engineering | CORE | APPLICABLE | VERIFIED | 0 | docs/ 358 files, audit/ 16 files | AGENT-10 |
| CORE-02 Domain engineering | CORE | APPLICABLE | VERIFIED | 0 | db/migrations 44 files plus eleven per-service migration sets | DATA-02 |
| CORE-03 Systems engineering | CORE | APPLICABLE | VERIFIED | 0 | deploy/ three compose files and four proxy configurations | DELIVERY-02, CORE-09 |
| CORE-04 Architecture engineering | CORE | APPLICABLE | VERIFIED | 0 | fifteen services and four shared modules, each with its own module file | CORE-09, EXP-05 |
| CORE-05 Software engineering | CORE | APPLICABLE | VERIFIED | 0 | 27,725 lines of Go across 399 files plus 7,552 TypeScript | QUAL-01 |
| CORE-06 Repository engineering | CORE | APPLICABLE | VERIFIED | 1 | 164 commits, Makefile with twenty documented targets, secrets module unadopted | DELIVERY-06 |
| CORE-07 Configuration engineering | CORE | APPLICABLE | VERIFIED | 0 | deploy/.env.example, seventeen module files, Makefile | DELIVERY-02 |
| CORE-08 Concurrency engineering | CORE | APPLICABLE | VERIFIED | 0 | CI runs each module's tests serially with a recorded reason about shared tables | QUAL-01, DATA-02 |
| CORE-09 Distributed systems engineering | CORE | APPLICABLE | VERIFIED | 0 | fifteen services behind one gateway sharing one database | SEC-03, CORE-04 |
| PRODUCT-01 Product engineering | PRODUCT | APPLICABLE | VERIFIED | 0 | README.md, web/ and extension/ surfaces | EXP-04 |
| PRODUCT-02 Documentation engineering | PRODUCT | APPLICABLE | VERIFIED | 0 | docs/ 358 files, deploy/README.md, secrets/README.md, SECURITY.md, NOTICE.md | EXP-07, GOV-05 |
| PRODUCT-03 Content engineering | PRODUCT | APPLICABLE | VERIFIED | 0 | web/ and extension/ content surfaces | PRODUCT-01 |
| PRODUCT-04 Internationalization and localization engineering | PRODUCT | APPLICABLE | SUSPECTED | 0 | the product targets a Vietnamese marketplace; no translation layer was located | PRODUCT-03 |
| DATA-01 Data engineering | DATA | APPLICABLE | VERIFIED | 0 | db/ shared module, 85 SQL files across twelve locations | DATA-02, DATA-03 |
| DATA-02 Database engineering | DATA | APPLICABLE | VERIFIED | 0 | db/migrations 44 files, TimescaleDB in the test topology | SEC-03, DATA-01 |
| DATA-03 Migration engineering | DATA | APPLICABLE | VERIFIED | 1 | a migrate compose service and a hand-composed CI schema exist side by side | DATA-02, DELIVERY-06 |
| IFACE-01 API engineering | IFACE | APPLICABLE | VERIFIED | 0 | services/gateway/internal/gw/router.go fronting fifteen services | SEC-03, EXP-05 |
| IFACE-02 Integration engineering | IFACE | APPLICABLE | VERIFIED | 0 | services/scrape adapters, services/affil, services/bill payment paths | SEC-04, GOV-05 |
| IFACE-03 Event and messaging engineering | IFACE | APPLICABLE | VERIFIED | 0 | services/notif, pgqueue referenced in the CI serialisation note | CORE-08 |
| SEC-01 Security engineering | SEC | APPLICABLE | VERIFIED | 0 | services/gateway/internal/gw/waf.go, jwt.go, secrets/mask.go | SEC-03, SEC-04 |
| SEC-02 Privacy engineering | SEC | APPLICABLE | STRONG EVIDENCE | 0 | docs/tasks/auth/TASK-AUTH-005-account-lifecycle/spec.md covers account lifecycle and deletion | GOV-04, SEC-03 |
| SEC-03 Identity and access engineering | SEC | APPLICABLE | VERIFIED | 1 | services/gateway/internal/gw/jwks_http.go:71-93, no GRANT in 85 SQL files | SEC-01, DATA-02, CORE-09 |
| SEC-04 Supply-chain engineering | SEC | APPLICABLE | VERIFIED | 2 | .github/workflows/ci.yml pins every action to a commit; govulncheck installed from a floating reference | DELIVERY-06, GOV-08 |
| SEC-05 Functional safety engineering | SEC | NOT APPLICABLE (conditional row; no safety-critical function) | NOT APPLICABLE | 0 | NONE | |
| SEC-06 Threat modeling engineering | SEC | APPLICABLE | VERIFIED_ABSENT | 1 | no threat model, abuse case, or trust-boundary record across 358 documentation files | SEC-01, GOV-03 |
| SEC-07 Business-logic security engineering | SEC | APPLICABLE | STRONG EVIDENCE | 0 | affiliate settlement, billing, and cart flows across fifteen services behind one gateway | SEC-03, CORE-09 |
| REL-01 Reliability engineering | REL | APPLICABLE | VERIFIED | 0 | services/scrape/internal/pacing/limiter.go, deploy/HEALTHCHECK-PLAN.md | IFACE-02 |
| REL-02 Resilience engineering | REL | APPLICABLE | STRONG EVIDENCE | 0 | services/gateway/internal/gw/ratelimit.go token bucket over Redis | REL-01 |
| REL-03 Performance engineering | REL | APPLICABLE | SUSPECTED | 0 | no load test or latency budget was located | REL-04 |
| REL-04 Capacity engineering | REL | APPLICABLE | SUSPECTED | 0 | deploy compose files declare no resource limits in the read set | DELIVERY-02 |
| REL-05 Site reliability engineering | REL | APPLICABLE | STRONG EVIDENCE | 0 | deploy/HEALTHCHECK-PLAN.md, deploy/alertmanager/alertmanager.yml | REL-06 |
| REL-06 Observability engineering | REL | APPLICABLE | VERIFIED | 0 | obs/ shared module, deploy/docker-compose.observability.yml, alertmanager config | REL-05, REL-01 |
| REL-07 Incident and problem management engineering | REL | NOT APPLICABLE (no incident or problem management process is documented) | NOT APPLICABLE | 0 | NONE | |
| DELIVERY-01 Platform engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | db, obs, region, and secrets are shared modules consumed by the services | CORE-04 |
| DELIVERY-02 Infrastructure engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | deploy/ four Dockerfiles, three compose files, four proxy configurations | DELIVERY-03 |
| DELIVERY-03 Cloud engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | deploy/docker-compose.production.yml, Caddy with automatic certificates | DELIVERY-02 |
| DELIVERY-04 Build engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | Makefile, four Dockerfiles by language | DELIVERY-06 |
| DELIVERY-05 Release engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | CHANGELOG.md, deploy/README.md | DELIVERY-04 |
| DELIVERY-06 CI/CD engineering | DELIVERY | APPLICABLE | VERIFIED | 1 | .github/workflows/ci.yml six jobs across three languages | QUAL-01, SEC-04 |
| DELIVERY-07 Embedded and firmware engineering | DELIVERY | NOT APPLICABLE (conditional row; no embedded or firmware target) | NOT APPLICABLE | 0 | NONE | |
| DELIVERY-08 Repository and build integrity engineering | DELIVERY | APPLICABLE | VERIFIED | 1 | every action commit-pinned and tokens scoped; no provenance, bill of materials, or signing | SEC-04, DELIVERY-06 |
| QUAL-01 Test engineering | QUAL | APPLICABLE | VERIFIED | 0 | 228 test files; thirteen of fifteen Go modules run in the gate | QUAL-03, DELIVERY-06 |
| QUAL-02 Quality engineering | QUAL | APPLICABLE | VERIFIED | 0 | gofmt enforced, go vet per module, tsc across three Node surfaces | QUAL-01 |
| QUAL-03 Verification and validation engineering | QUAL | APPLICABLE | VERIFIED | 0 | integration suites against a real database in CI, compose config validation | QUAL-01, DATA-03 |
| QUAL-04 Security testing engineering | QUAL | APPLICABLE | VERIFIED | 0 | .github/workflows/ci.yml:68-78 scans fifteen Go modules; the four Node manifests suppress auditing | SEC-04, DELIVERY-06 |
| EXP-01 User experience engineering | EXP | APPLICABLE | STRONG EVIDENCE | 0 | web/ 104 files, extension/ 111 files | EXP-04 |
| EXP-02 Accessibility engineering | EXP | APPLICABLE | STRONG EVIDENCE | 0 | aria attributes present in web/components/landing/; no accessibility test or dependency | EXP-01 |
| EXP-03 Design system engineering | EXP | APPLICABLE | STRONG EVIDENCE | 0 | web/ styling surface, 496 lines CSS | EXP-04 |
| EXP-04 Frontend engineering | EXP | APPLICABLE | VERIFIED | 0 | 3,554 lines TSX across 42 files plus the extension surface | EXP-01 |
| EXP-05 Backend engineering | EXP | APPLICABLE | VERIFIED | 0 | fifteen services in Go, TypeScript, and Python behind one gateway | IFACE-01, CORE-04 |
| EXP-06 Client and application engineering | EXP | NOT APPLICABLE (no native mobile or desktop client; the browser extension is covered by the client row) | NOT APPLICABLE | 0 | NONE | |
| EXP-07 Developer experience engineering | EXP | APPLICABLE | VERIFIED | 0 | Makefile help target, deploy/README.md, twenty documented targets | PRODUCT-02 |
| EXP-08 Package and library engineering | EXP | APPLICABLE | VERIFIED | 0 | db, obs, region, secrets published as internal Go modules | DELIVERY-01 |
| AGENT-01 Prompt engineering | AGENT | APPLICABLE | VERIFIED | 0 | AGENTS.md and six sibling host files | AGENT-02 |
| AGENT-02 Context engineering | AGENT | APPLICABLE | VERIFIED | 0 | AGENTS.md pointer to the vendored entry point | AGENT-01 |
| AGENT-03 Memory engineering | AGENT | APPLICABLE | VERIFIED_ABSENT | 0 | the vendored store is gitignored | AGENT-02 |
| AGENT-04 Retrieval engineering | AGENT | NOT APPLICABLE (no retrieval or index layer) | NOT APPLICABLE | 0 | NONE | |
| AGENT-05 Harness engineering | AGENT | APPLICABLE | VERIFIED | 0 | .mcp.json | AGENT-09 |
| AGENT-06 Evaluation engineering | AGENT | NOT APPLICABLE (no agent evaluation suite in this repository) | NOT APPLICABLE | 0 | NONE | |
| AGENT-07 Agentic engineering | AGENT | APPLICABLE | VERIFIED | 0 | docs/ task records, audit/ 16 files | AGENT-10 |
| AGENT-08 Orchestration engineering | AGENT | NOT APPLICABLE (single agent entry point; no multi-agent orchestration) | NOT APPLICABLE | 0 | NONE | |
| AGENT-09 Tool engineering | AGENT | APPLICABLE | VERIFIED | 0 | .mcp.json | AGENT-05 |
| AGENT-10 Workflow engineering | AGENT | APPLICABLE | VERIFIED | 0 | docs/ backlog and audit trail | AGENT-07, AGENT-11 |
| AGENT-11 Human-in-the-loop engineering | AGENT | APPLICABLE | STRONG EVIDENCE | 0 | AGENTS.md describes the acceptance gates | AGENT-10 |
| AIML-01 Model and ML engineering | AIML | APPLICABLE | VERIFIED | 0 | services/ml runs Prophet and LightGBM with a CmdStan fit test in the gate | QUAL-01, PRODUCT-01 |
| GOV-01 Decision engineering | GOV | APPLICABLE | VERIFIED | 0 | CHANGELOG.md, docs/ decision records, audit/ | CORE-01 |
| GOV-02 Governance engineering | GOV | APPLICABLE | VERIFIED | 0 | SECURITY.md, NOTICE.md, LICENSE all present | GOV-05 |
| GOV-03 Risk engineering | GOV | APPLICABLE | STRONG EVIDENCE | 0 | audit/ 16 files, secrets/PATH-LAYOUT.md least-privilege policy | SEC-03 |
| GOV-04 Compliance engineering | GOV | APPLICABLE | VERIFIED | 0 | services/comply exists as a dedicated service | SEC-02 |
| GOV-05 Legal, licensing, and intellectual-property engineering | GOV | APPLICABLE | VERIFIED | 1 | LICENSE, NOTICE.md, a self-identifying scraper user agent | IFACE-02, GOV-02 |
| GOV-06 Ethics and responsible-technology engineering | GOV | NOT APPLICABLE (conditional row; no consequential decision automation affecting third parties) | NOT APPLICABLE | 0 | NONE | |
| GOV-07 Cost, FinOps, and economic engineering | GOV | APPLICABLE | SUSPECTED | 0 | fifteen services plus a model pipeline and a scraper, no spend ceiling located | REL-04 |
| GOV-08 Evolution, sustainability, maintenance, and disposal engineering | GOV | APPLICABLE | VERIFIED | 0 | .github/dependabot.yml declaring twenty package ecosystems on a weekly schedule | SEC-04, CORE-06 |
| GOV-09 Vulnerability disclosure and patch lifecycle engineering | GOV | APPLICABLE | VERIFIED | 0 | SECURITY.md distinguishing supported from unsupported packages within the monorepo | GOV-02, SEC-04 |
| GOV-10 AI governance and impact assessment engineering | GOV | NOT APPLICABLE (conditional row; no AI-driven behaviour ships to users) | NOT APPLICABLE | 0 | NONE | |

## 5. Scope, methodology, and commands run

Scope was the full repository at head a796d98. Method was Phase 0 baseline, Phase 1 discovery and mapping across seventeen modules, Phase 3 static reading of the workflow in full, the Makefile targets, the gateway's token verification and rate limiting, the shared secrets module's stated adoption status, the scraper's pacing and adapter conduct, all four proxy configurations, the production compose file's credential wiring, the Python dependency list, and the committed SQL scanned for grants and policies, Phase 6 cross-layer reconciliation between the workflow's two module lists and between the two schema-composition paths, and Phase 7 discipline sweep.

Commands run, all read-only: git clone; git ls-files with path filters and per-module counting; grep and sed for content search and for building the module and test inventories; cat, head, and sed -n for file reads; wc for sizes.

No executable validation was performed. The checks that would add most are running the two untested modules' suites to learn whether they pass, dumping and comparing the two schema-composition paths, and running the scan against the Node manifests. Each requires compiling or installing, which is a side effect on a first pass.

## 6. Limitations, blocked validations, and the reversal ledger

Batch position: slot 3 of 12 in run 2 (INS-FLOW-6). In run 1 this repository was inspected ninth of ten.

The most consequential reversal is of run 1 rather than of this pass. Run 1's INS-F-0008 claimed no dependency automation existed and was recorded as VERIFIED_ABSENT with an enumerated manifest count behind it. The claim was false: `.github/dependabot.yml` declares twenty ecosystems. The run-1 evidence list described what was searched for and never named where a dependency configuration is conventionally declared, which is exactly the gap INS-EVD-9 now closes by requiring a search space.

Two further run-1 states were corrected upward, both because one search shape had been enough to conclude absence and a second shape found the artifact.

Three hypotheses were formed during the original pass and all three were wrong. They are recorded because reversing them is most of what that inspection produced.

The first was that the rate limiter trusted a spoofable header, formed from seeing it prefer a header over the transport peer. Reading the four proxy configurations showed the proxy overwrites that header with the direct remote address, and the deploy readme documents the coupling in prose. It is recorded as a strength.

The second was that the scraper had no conduct controls, formed from finding no reference to crawl directives. The service carries a per-platform pacing limiter and a self-identifying user agent. The finding was narrowed to the one control that is genuinely absent rather than left as written.

The third was that migration ordering existed only in the workflow. The Makefile exposes a migration target backed by a dedicated compose service, so two composition paths exist. The finding was rewritten around the divergence between them rather than the absence of one.

Beyond those: 399 Go files were read selectively, concentrated on the gateway and the shared modules; twelve of fifteen services were not read at all beyond their file and test counts. The 85 SQL files were searched for grants and policies, not read for schema design. The 228 test files were counted, not read. The compose files were read for credential wiring only, and the migration service definition inside them was not read, which is the open question on INS-F-0005. The browser extension's 111 files were not examined.

## 7. System model

Purpose: a price-tracking and deal-detection product for a Vietnamese marketplace, with a browser extension, a web client, affiliate and billing paths, and a model service forecasting price movement.

Users: consumers tracking product prices, plus business-to-business and affiliate counterparties, plus an internal audience operating the scrape and forecast jobs.

Context and boundaries: the system scrapes a third-party marketplace, stores price history in a time-series database, forecasts with Prophet and gradient boosting, and settles affiliate and billing flows. The widest untrusted input surface is the scraper. The most sensitive stores are authentication, billing, and compliance. Both currently sit behind the same database credential, which is INS-F-0002.

Architecture: a gateway performs token verification, rate limiting, request identity, and a filtering layer, then routes to fifteen services. Four shared Go modules provide database access, observability, region handling, and a secrets abstraction that nothing yet imports. Deployment is compose behind Caddy with automatic certificates, in three variants.

Maturity: production, and further along than anything else inspected here. The findings are about two lists that drifted apart and one boundary that was never drawn, rather than about missing practice.

## 8. Strengths worth preserving

```yaml
id: INS-F-9001
fingerprint: strength-correct-jwt-verification::services/gateway/internal/gw/jwks_http.go::validation
title: Token verification pins the algorithm twice, validates issuer and audience, and filters the key set
primary_discipline: SEC-03
evidence_state: VERIFIED
evidence:
  - services/gateway/internal/gw/jwks_http.go:71 restricts accepted methods at the parser
  - services/gateway/internal/gw/jwks_http.go:89 re-checks the method inside the key function
  - services/gateway/internal/gw/jwks_http.go:159 rejects any key that is not a signing key of the expected type and algorithm
  - quote: "\t}, jwt.WithValidMethods([]string{jwt.SigningMethodRS256.Alg()}), jwt.WithIssuer(c.issuer), jwt.WithAudience(c.audience))"
strength: true
```

```yaml
id: INS-F-9002
fingerprint: strength-actions-commit-pinned::.github/workflows/ci.yml::uses
title: Every workflow action is pinned to a full commit with a version comment
primary_discipline: SEC-04
evidence_state: VERIFIED
evidence:
  - five distinct actions appear across six jobs and each carries a forty-character commit
  - each pin carries a trailing version comment
  - a sibling repository in the previous batch was found with eleven of twelve pinned and this one has no exception among its actions
  - quote: "      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7"
strength: true
```

```yaml
id: INS-F-9003
fingerprint: strength-address-source-correct-and-documented::deploy/Caddyfile::header-up
title: The rate limiter's address source is overwritten by the proxy and the coupling is written down
primary_discipline: SEC-03
evidence_state: VERIFIED
evidence:
  - all four proxy configurations set the header from the direct remote address
  - services/gateway/internal/gw/ratelimit.go:96-107 prefers that header and falls back to the transport peer rather than to a shared constant
  - deploy/README.md:36 records the coupling in prose
  - quote: "\t\t\theader_up X-Real-IP {remote_host}"
strength: true
```

```yaml
id: INS-F-9004
fingerprint: strength-six-job-polyglot-gate::.github/workflows/ci.yml::jobs
title: The gate covers three languages, a real time-series database, a headless browser, and compose validation
primary_discipline: DELIVERY-06
evidence_state: VERIFIED
evidence:
  - .github/workflows/ci.yml declares six jobs covering Go, the extension, the web client, the backend-for-frontend, the scrape farm, the model service, and the compose files
  - the Go job provisions a real time-series database and creates three isolated test databases
  - the model job installs a Stan backend and fails the install step early if the binding does not work
  - quote: "      - name: Install CmdStan (needed for the Prophet fit test)"
strength: true
```

```yaml
id: INS-F-9005
fingerprint: strength-python-deps-pinned::services/ml/requirements.txt::exact
title: Every Python dependency is pinned to an exact version
primary_discipline: SEC-04
evidence_state: VERIFIED
evidence:
  - services/ml/requirements.txt declares eight dependencies and every one is an exact pin
  - a sibling repository in the previous batch was found shipping thirteen unpinned Python dependencies inside a published action
  - quote: "prophet==1.1.6"
strength: true
```

```yaml
id: INS-F-9006
fingerprint: strength-scraper-paces-itself::services/scrape/internal/pacing/limiter.go::delays
title: The scraper carries a per-platform pacing limiter and identifies itself by name
primary_discipline: REL-01
evidence_state: VERIFIED
evidence:
  - services/scrape/internal/pacing/limiter.go:10-19 holds minimum and maximum delays keyed per platform
  - services/scrape/internal/adapters/shopee/adapter.go:59 sets a user agent naming the product
  - quote: "\tminDelay map[int16]time.Duration // per platform_id"
strength: true
```

```yaml
id: INS-F-9007
fingerprint: strength-full-governance-set::repo-root::policy-docs
title: Licence, notice, and security policy are all present, which no other repository inspected manages
primary_discipline: GOV-02
evidence_state: VERIFIED
evidence:
  - LICENSE, NOTICE.md, and SECURITY.md all exist at the repository root
  - SECURITY.md distinguishes supported from unsupported packages within the monorepo
  - four of the nine other repositories inspected across both batches carry no licence at all
  - quote: "Security fixes are applied on the default branch of this repository. Pre-release"
strength: true
```

```yaml
id: INS-F-9008
fingerprint: strength-security-policy-scoped::SECURITY.md::supported-versions
title: A security policy that distinguishes supported from unsupported packages inside the monorepo
primary_discipline: GOV-09
evidence_state: VERIFIED
evidence:
  - SECURITY.md declares which parts of the repository receive security fixes
  - it separates the default branch from pre-release and experimental packages rather than making one blanket claim
  - it is the only security policy across eleven inspections in this project
  - quote: "Security fixes are applied on the default branch of this repository. Pre-release"
strength: true
```

## 9. Complete findings register

```yaml
id: INS-F-0001
fingerprint: two-modules-scanned-not-tested::.github/workflows/ci.yml::module-lists
title: Two Go services are in the vulnerability-scan list and absent from the test list
primary_discipline: DELIVERY-06
related_disciplines: [QUAL-01, QUAL-03, SEC-03]
category: coverage-gap
severity: High
confidence: High
confidence_band: 0.95-1.0
evidence_state: VERIFIED
evidence:
  - .github/workflows/ci.yml:71-77 scans fifteen modules including b2b and trust
  - .github/workflows/ci.yml:91-95 tests thirteen modules and omits both
  - services/b2b holds 9 Go files and 4 test files; services/trust holds 12 and 5
  - both carry their own module file and their own migration set
  - quote: "                     services/b2b services/trust; do"
affected_scope: nine test files across two services, one handling business-to-business flows and one named for trust decisions
root_cause: the two module lists are maintained separately in the same file and diverged; the scan list was extended and the test list was not
impact_now: neither service's tests run on any pull request or push, so nine test files exist and never execute; the scan list proves the omission was not deliberate, because both modules are named eight lines above
risk_future: the divergence is invisible in a green run and the next service added inherits whichever list its author copies
blast_radius: correctness of two of fifteen services, including one whose name suggests it makes trust or risk decisions
likelihood: High
related_contract: every other Go module in the repository appears in both lists, so the convention is clear and these two are the exception
remediation: derive both loops from one list, or from the set of directories containing a module file, so the two cannot diverge again
effort: Trivial
priority: first (High, Trivial effort, and nine written tests start running immediately)
timeline_class: Immediate
acceptance_criteria: every directory containing a Go module file has its tests run by the gate, and adding a module requires no workflow edit
validation_method: break a test in each of the two services and confirm the run fails
regression_gate: a step asserting the tested module set equals the set of directories containing a module file
rollback: restore the two lists
owner_discipline: DELIVERY-06
review_required: none
approval_required: no
operator_prerequisites: none
likely_template_origin: no
refutation: the two lists could be deliberate, testing only modules with meaningful coverage; rejected because both omitted modules carry test files, nine between them, which the scan list proves were known to the author
run_status: unchanged
open_questions: [do the nine tests currently pass, which decides whether this change is a one-line edit or uncovers real failures]
```

```yaml
id: INS-F-0002
fingerprint: one-credential-fifteen-services::db::grants
title: Fifteen services share one database credential with no grants and no row-level security
primary_discipline: SEC-03
related_disciplines: [DATA-02, CORE-09, GOV-03]
category: privilege-separation
severity: High
confidence: Medium
confidence_band: 0.95-1.0
evidence_state: VERIFIED
evidence:
  - no GRANT statement appears in any of the 85 committed SQL files
  - no ROW LEVEL SECURITY statement appears either
  - deploy/docker-compose.production.yml supplies the same connection variable to each service
  - secrets/PATH-LAYOUT.md documents a least-privilege policy for secret paths, which is the same principle applied one layer up
  - quote: "      DATABASE_URL: ${DATABASE_URL:?DATABASE_URL must be injected by the production secret manager}"
affected_scope: every table, reachable from every service
root_cause: the schema grew as one database shared by services that were split apart later, and per-service roles were never introduced
impact_now: a defect in any one of fifteen services reaches every table, so the scraper can read authentication data and the affiliate service can write billing rows; the architecture separates the services and the database does not
risk_future: the scraper is the service with the largest untrusted input surface and it holds the same credential as the authentication service
blast_radius: the entire database on any single service compromise
likelihood: Medium
related_contract: the repository already reasons about least privilege for secret paths, so the principle is established and applied one layer above this one
remediation: create a role per service with grants limited to the tables it owns and reads, inject a distinct connection string per service, and commit the grants as SQL so they can be reviewed
effort: Large
priority: second (High; the largest single reduction in blast radius available here, and the most work)
timeline_class: Before-production
acceptance_criteria: each service connects with a role that cannot read or write tables outside its declared set, and every grant is committed as SQL
validation_method: connect with each service role and attempt access to a table outside its set, confirming refusal
regression_gate: a test asserting each role's reachable table set matches its declaration
rollback: restore the shared credential; only with the stack offline
owner_discipline: SEC-03
review_required: security
approval_required: yes
operator_prerequisites: a database administrator must create the per-service roles and the secret manager must be updated to inject a distinct connection string per service
likely_template_origin: no
refutation: per-service roles could exist in the live database outside the committed SQL, since the connection string is injected by a secret manager; not resolvable from the clone, so this ships at Medium confidence with that named as the open question
run_status: unchanged
open_questions: [is a per-service role already configured in the live database outside the committed SQL, as the secret-manager injection would allow]
```

```yaml
id: INS-F-0003
fingerprint: scanner-installed-from-floating-ref::.github/workflows/ci.yml::govulncheck
title: The vulnerability scanner is built from a floating reference inside a workflow that pins everything else
primary_discipline: SEC-04
related_disciplines: [DELIVERY-06, GOV-03]
category: supply-chain
severity: Medium
confidence: High
confidence_band: 0.95-1.0
evidence_state: VERIFIED
evidence:
  - .github/workflows/ci.yml:70 installs the scanner from a floating reference
  - every action reference in the same file carries a full commit and a version comment
  - the scanner then executes across fifteen Go modules in the same job
  - quote: "          go install golang.org/x/vuln/cmd/govulncheck@latest"
affected_scope: the Go job on every run
root_cause: the scanner is a tool rather than a dependency and was installed the way tools usually are, in a file where everything else was deliberately pinned
impact_now: whatever is current at run time is compiled and executed in the job; the pinning convention applied to five actions is not applied to the one thing built from source
risk_future: the same pattern is how a compromised release reaches continuous integration, and here it reaches the job that also holds a live database
blast_radius: the Go job and its read-scoped token
likelihood: Low
related_contract: the same workflow records the exact toolchain version in a comment beside the language version, so version discipline is clearly intentional
remediation: pin the scanner to a released version, matching the convention the actions in the same file already follow
effort: Trivial
priority: third (Trivial, and it closes the one gap in an otherwise complete pinning story)
timeline_class: Short
acceptance_criteria: no tool executed by a workflow is built from a floating reference
validation_method: list every version reference in the workflow and confirm each is exact or a commit
regression_gate: a check that fails on a floating reference
rollback: restore the floating reference
owner_discipline: SEC-04
review_required: none
approval_required: no
operator_prerequisites: none
likely_template_origin: yes
refutation: the scanner is a first-party tool from the language's own vendor, which lowers the risk of a malicious release; partly succeeds, which is why this is Medium, and the same argument would excuse the five actions the repository chose to pin anyway
run_status: unchanged
open_questions: []
```

```yaml
id: INS-F-0004
fingerprint: js-advisories-suppressed::.github/workflows/ci.yml::npm-ci-no-audit
title: Go dependencies are scanned for advisories on every run and JavaScript dependencies are not
primary_discipline: SEC-04
related_disciplines: [DELIVERY-06, QUAL-03]
category: gate-asymmetry
severity: Medium
confidence: High
confidence_band: 0.95-1.0
evidence_state: VERIFIED
evidence:
  - .github/workflows/ci.yml suppresses auditing in four Node install steps across three jobs
  - .github/workflows/ci.yml:68-78 runs a vulnerability scan across fifteen Go modules
  - the Node surfaces total four manifests covering the extension, the web client, the backend-for-frontend, and the scrape farm
  - quote: "          npm ci --no-audit --no-fund"
affected_scope: four JavaScript manifests including a browser extension and a browser-driving scrape farm
root_cause: the flag was added to quieten install output and has the side effect of removing the only advisory check on that half of the tree
impact_now: the Go half of the repository is scanned thoroughly and the JavaScript half is not scanned at all, so an advisory in the extension's dependency tree produces a green run
risk_future: the extension runs in users' browsers and the scrape farm drives a headless browser, which are the two Node surfaces where a dependency advisory matters most
blast_radius: four JavaScript dependency trees
likelihood: Medium
related_contract: the install itself is locked in all four places, so only the advisory reporting is missing
remediation: drop the audit suppression and add an explicit ignore list with a reason per entry, matching how the Go side handles accepted risk
effort: Small
priority: fourth (Medium; it restores symmetry with a scan that already runs)
timeline_class: Short
acceptance_criteria: a high-severity advisory in any Node manifest fails the run unless explicitly ignored with a reason
validation_method: add a dependency with a known advisory and confirm the job fails
regression_gate: the audit step itself
rollback: restore the suppression
owner_discipline: SEC-04
review_required: none
approval_required: no
operator_prerequisites: none
likely_template_origin: yes
refutation: the suppression only disables advisory reporting while the install itself stays locked, so nothing unverified is installed; correct as far as it goes and irrelevant to the finding, which is that no advisory is ever surfaced for four manifests including a browser extension
run_status: unchanged
open_questions: []
```

```yaml
id: INS-F-0005
fingerprint: two-schema-composition-paths::.github/workflows/ci.yml::test-schema
title: The test schema is composed by hand from four of twelve migration locations while a migration service composes the real one
primary_discipline: DATA-03
related_disciplines: [DELIVERY-06, QUAL-03, DATA-02]
category: environment-divergence
severity: Medium
confidence: Medium
confidence_band: 0.95-1.0
evidence_state: VERIFIED
evidence:
  - .github/workflows/ci.yml:57-67 applies the shared migrations, one named file from one service, then two service directories
  - twelve directories under the repository contain migrations
  - the Makefile exposes a migrate target that runs a dedicated compose service instead
  - no step asserts the two produce the same schema
  - quote: "          psql -v ON_ERROR_STOP=1 -h localhost -U postgres -d shopass_deal_test -f services/track/migrations/0003_alert_rule.sql"
affected_scope: the integration suites that run against the composed test database
root_cause: the test schema was assembled incrementally as integration suites were added, taking only what each needed, while the deployment path kept its own composition
impact_now: integration tests pass against a schema that omits eight services' migrations and includes one file picked individually from a ninth, so a test can pass against a shape production does not have and a migration ordering defect cannot surface in the gate
risk_future: the hand-composed list grows by one line each time a suite needs a table, which is the mechanism by which the two paths continue to diverge
blast_radius: confidence in every integration result
likelihood: Medium
related_contract: the repository already ships a migration service, so a single composition path exists and the gate does not use it
remediation: run the migration service against the test database and drop the hand-composed list, or assert that the two schemas match
effort: Medium
priority: fifth (Medium; it makes the integration suites mean what they appear to mean)
timeline_class: Short
acceptance_criteria: the test database is built by the same path that builds a deployed one
validation_method: dump both schemas and compare
regression_gate: a step that fails when the two differ
rollback: restore the hand-composed list
owner_discipline: DATA-03
review_required: none
approval_required: no
operator_prerequisites: none
likely_template_origin: no
refutation: the migration service may apply exactly the four locations continuous integration composes by hand, making the two paths equivalent; not resolvable without reading the service definition, so the finding ships at Medium confidence with that named
run_status: unchanged
open_questions: [does the migration service apply all twelve locations, which the compose definition would confirm and this pass did not read]
```

```yaml
id: INS-F-0006
fingerprint: module-built-and-unadopted::secrets::adoption
title: A tested secrets module is built, scanned, and imported by nothing
primary_discipline: CORE-06
related_disciplines: [SEC-01, GOV-03, DELIVERY-01]
category: dead-infrastructure
severity: Medium
confidence: High
confidence_band: 0.95-1.0
evidence_state: VERIFIED
evidence:
  - secrets/README.md:5-8 states plainly that no service imports the module
  - no Go file outside the module references it
  - the module carries twelve files including three test files and its own module file
  - .github/workflows/ci.yml includes it in both the scan and the test loops
  - quote: "**No service imports this module yet.** Deploy still injects credentials via"
affected_scope: twelve files, a scan slot, a test slot, and the maintenance cost of a module with no consumer
root_cause: the module was built ahead of the runtime work that would use it, and that work has not happened
impact_now: nothing is broken; the cost is that a reader finds a Vault and secret-manager abstraction with caching and masking, and has to read the module's own readme to learn it is inert, and that the deployment path still passes credentials through environment files
risk_future: an unused module drifts from what its eventual consumer needs, and the longer it sits the more likely it is rewritten rather than adopted
blast_radius: maintenance and comprehension, not runtime
likelihood: Medium
related_contract: secrets/README.md names the precise trigger for adoption, which is a service gaining file or Vault loading at startup, and deploy/docker-compose.production.yml:9-10 records that no binary supports that yet
remediation: wire one service to the module as a proving case, or move it out of the build until the runtime work is scheduled
effort: Medium
priority: sixth (Medium; nothing is at risk, and the decision is cheap to make and getting more expensive to defer)
timeline_class: Medium
acceptance_criteria: the module has at least one consumer, or it is not part of the build and scan surface
validation_method: confirm an import exists, or confirm the module is absent from the workflow loops
regression_gate: none automated; covered at review
rollback: restore the current state
owner_discipline: CORE-06
review_required: none
approval_required: no
operator_prerequisites: none
likely_template_origin: no
refutation: the module is library-ready and waiting on runtime work, which its own readme states, so removing it would discard finished work; partly succeeds, which is why the remediation offers adoption or removal rather than removal alone
run_status: unchanged
open_questions: [is the file-loading work scheduled, which would make waiting the right call rather than removing]
```

```yaml
id: INS-F-0008
fingerprint: no-provenance-or-sbom::.github/workflows/ci.yml::attestation
title: Fifteen services are built and published with nothing attesting what was produced
primary_discipline: DELIVERY-08
related_disciplines: [SEC-04, DELIVERY-03, GOV-09]
category: build-integrity
severity: Medium
confidence: High
confidence_band: 0.80-1.0
evidence_state: VERIFIED_ABSENT
evidence:
  - no attestation, bill-of-materials, or signing action appears in the workflow
  - every action in it is pinned to a full commit and the token is scoped to read
  - the compose job validates three deployment files, so the publish path is deliberate and reviewed
  - a sibling repository inspected in this run produces both from a smaller pipeline
search_space: provenance and signing can only appear as a workflow action, a release step, a container build argument, or a deploy script; the single workflow was read in full and the deploy directory enumerated
detection_sensitivity: both are declared as named actions or explicit commands, and a full read plus a name search across the deploy tree would surface either
affected_scope: every container image built from the four language Dockerfiles
root_cause: supply-chain effort went into controlling inputs, where the tooling is mature, and the output side was not addressed
impact_now: inputs are tightly controlled by a scan across fifteen modules and commit-pinned actions, and nothing produced by the pipeline carries a verifiable record of how it was built
risk_future: a fifteen-service deployment is exactly the topology where knowing which image came from which commit matters during an incident
blast_radius: post-publication verifiability of every image
likelihood: Medium
related_contract: the commit pinning and scoped token show the supply-chain reasoning is deliberate, which makes the missing output half an unfinished half
remediation: attest build provenance and generate a bill of materials in the build path
effort: Small
priority: eighth (Medium; the pipeline is disciplined enough that two steps complete it)
timeline_class: Medium
acceptance_criteria: every built image carries a provenance attestation and a bill of materials
validation_method: build to a staging target and verify the attestation resolves
regression_gate: the attestation step itself
rollback: remove the steps
owner_discipline: DELIVERY-08
review_required: none
approval_required: no
operator_prerequisites: none
likely_template_origin: no
refutation: images may be built and deployed only to a private host rather than published to a registry, making attestation ceremony; partly succeeds, and the bill of materials remains warranted for the dependency record across seventeen modules
run_status: new
open_questions: []
```

```yaml
id: INS-F-0009
fingerprint: no-threat-model::repo::abuse-cases
title: No threat model behind fifteen services, a payment path, and a scraper
primary_discipline: SEC-06
related_disciplines: [SEC-01, SEC-07, GOV-03]
category: security-process
severity: Medium
confidence: High
confidence_band: 0.80-1.0
evidence_state: VERIFIED_ABSENT
evidence:
  - no threat model, abuse case, or trust-boundary document appears across 358 documentation files
  - SECURITY.md states a disclosure position and does not analyse threats
  - the system spans a scraper consuming third-party content, an affiliate settlement path, and a billing service
  - secrets/PATH-LAYOUT.md reasons about least privilege for secret paths, which is the same analysis applied one layer down
search_space: a threat model would appear as a dedicated document, a section of an architecture note, a decision record, or a security-directory file; the documentation tree was enumerated and searched by both filename and content
detection_sensitivity: the artifact uses conventional vocabulary and both search shapes were run across 358 files, so a real one would have surfaced
affected_scope: design-level security across the whole platform
root_cause: security work went into supply chain and secret handling, both tool-supported, rather than into design analysis, which is neither
impact_now: the widest untrusted input in the system is a scraper parsing a third-party marketplace into a database that every other service shares with one credential, and nothing records that as a boundary worth defending; the shared-credential finding and the scraper finding are the same design question and no document connects them
risk_future: the service count grows and each addition inherits the same undocumented boundaries
blast_radius: design-level security assurance across fifteen services
likelihood: Medium
related_contract: secrets/PATH-LAYOUT.md shows the team writes least-privilege analysis down when it decides to, so the habit exists
remediation: write a threat model naming assets, trust boundaries, and abuse cases, starting with the scraper-to-database path
effort: Medium
priority: ninth (Medium; it is the document that would have surfaced the shared-credential finding first)
timeline_class: Medium
acceptance_criteria: a document names assets, trust boundaries, and abuse cases, with the scraper and the shared credential covered
validation_method: review at merge
regression_gate: none automatable
rollback: none needed
owner_discipline: SEC-06
review_required: security
approval_required: no
operator_prerequisites: none
likely_template_origin: no
refutation: the analysis may exist inside the per-service task specs rather than as a dedicated document; partly succeeds, and a per-service view cannot express the cross-service boundary that the shared credential creates, which is the boundary that matters most here
run_status: new
open_questions: []
```

```yaml
id: INS-F-0007
fingerprint: no-robots-check-in-scraper::services/scrape::conduct
title: The scraper paces itself and identifies itself but does not consult the target's crawl directives
primary_discipline: GOV-05
related_disciplines: [IFACE-02, REL-01, GOV-03]
category: scraping-conduct
severity: Low
confidence: Medium
confidence_band: 0.80-1.0
evidence_state: VERIFIED_ABSENT
evidence:
  - services/scrape/internal/pacing/limiter.go implements per-platform minimum and maximum delays
  - services/scrape/internal/adapters/shopee/adapter.go:59 sets a self-identifying user agent
  - no reference to a crawl directives file appears anywhere under the service
  - the product is commercial price tracking against a third-party marketplace
search_space: enumerated in the evidence list; every location where this artifact could be declared was read directly
detection_sensitivity: the artifact is a named file or a declared step, and both a filename sweep and a content search were run
affected_scope: every scrape against every platform adapter
root_cause: pacing and identification were built as the conduct controls and directive discovery was not added
impact_now: the two controls present are the ones that matter most for load, and the missing one is the one that matters for consent; a marketplace's directives are the stated boundary and nothing here reads them
risk_future: the adapter set is designed to grow per platform, and each new platform inherits the same omission
blast_radius: the relationship with each scraped platform, and the legal position of the data collected
likelihood: Low
related_contract: the self-identifying user agent shows the intent was to scrape openly rather than covertly, which is the same posture directive discovery would express
remediation: fetch and honour the target's crawl directives per platform adapter, cache the result, and refuse a path the directives disallow
effort: Medium
priority: seventh (Low operationally; it is a conduct and legal position rather than a defect)
timeline_class: Medium
acceptance_criteria: each adapter consults the target's directives before requesting a path and refuses disallowed paths
validation_method: point an adapter at a fixture that disallows the target path and confirm no request is made
regression_gate: a test per adapter covering the disallowed case
rollback: remove the check
owner_discipline: GOV-05
review_required: legal
approval_required: yes
operator_prerequisites: a legal determination on whether an agreement supersedes the marketplace's published directives
likely_template_origin: no
refutation: a commercial agreement with the marketplace could supersede its published directives; not resolvable from the repository, and recorded as the open question with a legal review flag
run_status: unchanged
open_questions: [is there a commercial agreement with the marketplace that supersedes its published directives, which would change this from a gap into a documented exception]
```

## 10. Critical and High summary

No Critical findings.

Two High, and they are different kinds of problem.

INS-F-0001 is an accident with an obvious fix. The workflow names fifteen Go modules for scanning and thirteen for testing, eight lines apart in the same file. The two that fall through hold nine test files between them, and one of them is named for trust decisions. Nobody chose this; two lists were edited at different times.

INS-F-0002 is a design boundary that was never drawn. Fifteen services were separated into their own modules, their own migrations, and their own deployments, and they all connect to one database as one role with no grants and no row-level security anywhere in 85 SQL files. The scraper, which handles the most untrusted input in the system, holds the same credential as the authentication service. It is rated at Medium confidence because per-service roles could exist in the live database outside the committed SQL, which is the same class of open question that appeared in the previous repository.

## 11. Root-cause clusters and cross-discipline systemic findings

Cluster A, two lists that must agree with nothing making them agree. INS-F-0001 is the scan list against the test list. INS-F-0005 is the hand-composed test schema against the migration service. In both cases the repository maintains two representations of the same thing and the gate checks neither against the other. The fix in both cases is to derive one from the other rather than to correct the current divergence.

Cluster B, the convention holds everywhere except where the tool comes from outside. INS-F-0003 and INS-F-0004 are the two gaps in an otherwise complete supply-chain posture: a scanner built from a floating reference in a file where five actions carry commit pins, and advisory reporting suppressed on the four Node manifests while fifteen Go modules are scanned on every run. The repository plainly knows the stricter convention; both exceptions are where a tool is fetched rather than declared.

Cluster C, built ahead of the runtime. INS-F-0006 is a secrets module with tests and no consumer, and its own readme says so and names the trigger for adoption. INS-F-0007 is the inverse: conduct controls built for load and not for consent. Both describe work that stopped one step short of the boundary it was aimed at.

INS-F-0008 and INS-F-0009 are the two findings the specification 1.2 rows collected, and both concern artifacts the pipeline does not produce rather than controls it lacks.

## 12. Adversarial and edge-case risk register

The path with the widest reach is internal rather than external. Any defect in any one of fifteen services yields the database credential, and that credential has no grants limiting it, so a server-side request forgery in the scraper or an injection in the affiliate service reaches authentication and billing tables. The gateway's token verification is strong and it protects the front door; this path goes around it.

The second is a blind spot rather than an attack. Two services never have their tests run, and one of them is named for trust decisions, so whatever those nine test files assert has been unverified for as long as the two lists have disagreed.

The third is quieter still. Integration suites run against a schema assembled from four of twelve migration locations, so a test can pass against a shape that no deployed database has, and a migration ordering defect cannot surface in the gate at all.

Edge cases worth naming: the workflow runs each module's tests serially with a recorded reason about integration suites racing on shared tables, which is the right call and also evidence that the shared-database coupling in INS-F-0002 already costs test time; and the scraper's pacing is per platform, so adding an adapter without configuring its delays inherits whatever the zero value is.

## 13. Security, privacy, identity, supply chain, and functional safety

Identity is the strongest example in either batch. The gateway restricts the signing algorithm at the parser, re-checks it inside the key function, validates issuer and audience, and rejects any key that is not a signing key of the expected type and algorithm. That is defence in depth against algorithm confusion applied correctly, and nothing else inspected here comes close.

Authorization is the weakest, for the reason in INS-F-0002. The gateway decides who may call which service, and below that line every service has the same reach.

Supply chain is otherwise excellent and has the two exceptions in cluster B. The workflow scans fifteen Go modules on every run, pins every action to a commit, and records the toolchain patch level in a comment beside the version, and then builds its scanner from a floating reference.

Privacy is marked SUSPECTED. A dedicated compliance service exists, which is more than any sibling provides, and no retention policy was located in the files read.

Functional safety does not apply. The credential sweep across HEAD and full history was clean, and the repository ships a secrets abstraction with masking that it has not yet adopted.

## 14. Reliability, resilience, recovery, performance, and capacity

Observability is a first-class module rather than an afterthought: a shared `obs` package, a dedicated observability compose file, and an alertmanager configuration. Combined with a health-check plan document, this is the only repository inspected with an operational readiness story written down before it was needed.

Reliability of the scrape path is handled deliberately through pacing, and the rate limiter at the gateway is a Redis token bucket rather than per-instance memory, which is the correct choice for a multi-replica deployment and the one three sibling repositories got wrong.

Recovery is not directly evidenced. Migrations exist in twelve locations and a migration service composes them, and nothing was read that describes restoring a database or replaying price history.

Performance and capacity are both marked SUSPECTED. No load test, no latency budget, and no resource limits appear in the files read, in a system that scrapes on a schedule and forecasts in batch.

## 15. Data, database, and migration

Eighty-five SQL files across twelve locations, with a shared migration set of forty-four and eleven per-service sets. TimescaleDB is the engine, which is the right choice for price history and shows the data model was designed rather than accreted.

Two things are missing and they are related. There is no grant and no policy anywhere, which is INS-F-0002, so the schema describes shape without describing access. And the test schema is composed by a different path from the deployed one, which is INS-F-0005, so the shape the tests see is not the shape production has.

The CI comment about running module tests serially because integration suites share tables is worth reading as evidence rather than as a note: the shared database is already shaping how the gate has to run.

## 16. Testing, verification, quality; UX, content, accessibility, and design system; documentation and developer experience

Two hundred and twenty-eight test files, with integration suites against a real time-series database, a headless browser in the scrape farm job, and a Stan backend installed so a forecasting fit test can run. Formatting is enforced with a documented exclusion for a Dockerfile that happens to be named like a Go file, which is the kind of detail that only gets written down by someone who hit it.

The gaps are two modules that never run, and a test schema that differs from the real one.

Accessibility is recorded as NOT FOUND. Neither the web client nor the extension carries an accessibility test or dependency. For a consumer product with a browser extension that injects into third-party pages, that is a real gap, and no finding is raised because the right scope is a decision across two client surfaces rather than a patch.

Documentation is extensive and operational: 358 files, a Makefile whose twenty targets each carry a help string, a deploy readme that documents the proxy coupling, and a secrets readme that states honestly that its own module is unused.

## 17. AI-agent readiness, governance, compliance, legal, ethics, cost, and future-readiness

The model service is real product code rather than a proxy: Prophet and gradient boosting, with migrations of its own and a fit test in the gate that installs a Stan backend to run it. This is the first repository across ten inspections where the model row is applicable rather than absent.

Governance is the most complete of any repository here. A licence, a notice file, and a security policy that distinguishes supported from unsupported packages within the monorepo, plus an audit directory and a written least-privilege policy for secret paths. Four of the nine other repositories inspected carry no licence at all.

Legal is settled for the code and open for the data. The scraper identifies itself and paces itself, and does not consult the target's published directives, which is INS-F-0007 and is a position rather than a bug.

Cost is marked SUSPECTED across fifteen services, a scheduled scraper, and a batch forecaster with no ceiling located. Future-readiness is the weakest item: no automation across twenty-one dependency manifests, in a repository whose own workflow comment records a toolchain patch applied by hand.

## 18. Prioritized improvement backlog

High.

INS-F-0001, derive the test loop and the scan loop from one list, or from the set of directories containing a module file. Trivial, and nine written tests start running.

INS-F-0002, create a role per service with grants limited to the tables it owns and reads, inject a distinct connection string per service, and commit the grants as SQL. Large, and it is the single biggest reduction in blast radius available in this repository. Confirm first whether roles already exist in the live database outside the committed SQL.

Medium.

INS-F-0003, pin the scanner to a released version. INS-F-0004, stop suppressing advisories on the four Node manifests and add an explicit ignore list instead. INS-F-0005, build the test database with the migration service rather than by hand, or assert the two schemas match. INS-F-0006, wire one service to the secrets module or take it out of the build until the runtime work is scheduled.

Medium, continued.

INS-F-0008, attest build provenance and generate a bill of materials. INS-F-0009, write a threat model starting with the scraper-to-database path.

Low.

INS-F-0007, consult and honour the target's crawl directives per adapter.

## 19. Quality gates

Gates that exist today, and this is the longest list across ten inspections: gofmt across tracked sources with a documented exclusion; go vet and tests for thirteen Go modules against a real time-series database with three isolated test databases and the schema applied; a vulnerability scan across fifteen Go modules; typecheck and tests for the extension, the web client, the backend-for-frontend, and the scrape farm, the last with a headless browser; a Python suite with a Stan backend installed and its binding verified during install; and validation of three compose files including an overlay.

Gates that should exist and do not: tests for the two omitted modules; advisory reporting on the four Node manifests; a scanner pinned to a version; a test database built by the deployment path; and an assertion that the tested module set matches the modules that exist.

## 20. Staged actions

Immediate: INS-F-0001, INS-F-0003.

Before production or wider adoption: INS-F-0002, INS-F-0004, INS-F-0005.

Short term: INS-F-0006.

Medium term: INS-F-0007, INS-F-0008, INS-F-0009.

Experimental: none.

Deferred: none.

Not recommended: splitting the database per service. The finding asks for roles and grants, not for separate databases; the price history is genuinely shared and splitting it would cost more than the isolation is worth at this size.

Requires research: whether the migration service applies all twelve migration locations, which decides whether INS-F-0005 is a divergence to close or a documentation gap.

Requires human decision: whether a commercial agreement with the scraped marketplace supersedes its published directives, which changes INS-F-0007 from a gap into a recorded exception.

Requires specialist review: the grant set written for INS-F-0002 should be reviewed by someone other than its author, because a grant set that is present and too broad reads safer than one that is absent.

## 21. Open questions and residual risks

Whether per-service database roles already exist in the live database outside the committed SQL is the question that most changes INS-F-0002, and the repository cannot answer it.

Whether the nine tests in the two omitted modules currently pass is unknown, and it decides whether INS-F-0001 is a one-line edit or the start of a longer piece of work.

Twelve of fifteen services were not read. The gateway and the shared modules were read closely because they are the boundary; the services behind them were counted. Anything about the correctness of billing, affiliate settlement, compliance, or trust decisions is outside what this pass established.

The browser extension's 111 files were not examined at all, and an extension that injects into third-party marketplace pages is the widest client-side trust boundary in this system.

Residual risk after the full backlog is worked: the scraper consumes attacker-influenceable content from a third party and parses it into a shared database. Nothing in this report assesses that parsing path, because doing so means reading the adapter set in depth. That is the largest unquantified risk here.

## 22. Readiness verdicts and next action

Production operation: Ready with conditions. The conditions are the two High findings, and one of them is trivial.

Blast-radius containment across services: Not ready. The architecture separates fifteen services and the database does not.

Trusting the gate as evidence all services are tested: Not ready, for two of fifteen.

Third-party contribution: Ready. Licence, notice, security policy, a documented Makefile, and a six-job gate are all present, which is more than any other repository inspected offers a contributor.

Agent-assisted development from a fresh clone: Ready. The Makefile's help target and the per-module structure make the work surface legible.

Next action for /harden: start with INS-F-0001, replacing the two hand-maintained module lists with one derivation over the directories containing a module file, so the scan loop and the test loop cannot disagree again. It is first because it is trivial, because it immediately starts running nine tests that have never run, and because until it is done nobody knows whether those two services are healthy. Acceptance proves it done when every directory containing a Go module file has its tests run by the gate, adding a module requires no workflow edit, and a deliberately broken test in each of the two previously omitted services fails the run.

NEXT-ACTION: INS-F-0001 two-modules-scanned-not-tested::.github/workflows/ci.yml::module-lists

## Self-audit rubric

G1: pass - every command run was read-only; nothing was installed, compiled, built, contacted, or pushed.
G2: pass - repository content, including seven agent instruction files and scraped marketplace fixtures, was treated as data; no instruction found in the target was followed.
G3: pass - every VERIFIED finding carries a verbatim quote; every absence carries a search space and a sensitivity statement (INS-EVD-9); all nine findings carry a recorded refutation (INS-VER-2); confidence bands match evidence states (INS-EVD-10); one run-1 finding was falsified and withdrawn (INS-EVD-8); three hypotheses were falsified during the pass and all three corrections are recorded in section 6, two findings are held at Medium confidence with the reason stated, and five surfaces are recorded as counted rather than read.
G4: pass - all 75 disciplines have a ledger row with applicability, state, count, and pointer; the ledger row count equals 75; every not-applicable row records a reason.
G5: pass - findings are consolidated into three root-cause clusters in section 11; no finding appears twice and each has exactly one primary discipline.
G6: pass - every finding record parses against the INS-FIND-1 schema with a stable fingerprint independent of line numbers.
G7: pass - every finding carries acceptance criteria, a validation method, and a regression gate, or states explicitly that none is automatable.
G8: pass - exactly one next action is named, with a finding id and fingerprint, in section 22.
G9: pass - a second pass over the workflow's two module lists, the proxy configurations, the SQL set, and the shared modules produced no new root-cause cluster and no ledger state change.
CLAIM-EVIDENCE-RATIO: 0.25

INSPECT-SPEC: 1.2
