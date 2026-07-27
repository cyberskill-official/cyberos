# /inspect report: zintaen/issue-hunter

## 1. Side-effect disclosure

None against the target. Every command was read-only: git clone, git fetch --unshallow, git log, git ls-files, file reads, and text search. No dependency was installed, no service was contacted, no code from the repository was executed, and nothing was written to the repository or pushed. The clone lives in a scratch directory.

One disclosure about a prior turn: during the batch baseline sweep, a credential-shaped literal in this repository was matched by a search pattern. Its provider endpoint and length were reported to the operator with the value redacted, and the operator has since rotated it. That is recorded here because it changed the severity of INS-F-0012 from an open exposure to a recurrence-prevention finding.

## 2. Executive summary

issue-hunter is an autonomous agent that clones third-party repositories, runs generated code, and opens pull requests using a GitHub token. It is published as a GitHub Action with Marketplace branding. The application design is sound, particularly the separation into named agent roles and the decision to execute generated code in a cloud sandbox rather than on the host. The controls around it are not.

Three findings are Critical and they compound. The GitHub webhook verifies nothing, so an unauthenticated caller can run the full agent, with the deployment's own credentials, against a repository the caller names. The row-level security policies are named for the service role but grant every operation to every role, while the anon key they were supposed to constrain is distributed to every consumer of the Action and shipped to the browser by design. And the Action installs thirteen unpinned Python dependencies at runtime in a step that already holds four secrets.

Six further findings are High: a default admin password published in the source, a bearer token that is that same password, an unauthenticated debug endpoint, a model key and a GitHub personal access token persisted in browser storage, the model key passed on the command line, and no continuous integration of any kind for a repository that other repositories execute.

Readiness at a glance: not ready for public use, and not ready to remain published in its current form. The three Critical findings are all Small or Trivial effort.

Findings: 17 total, 3 Critical, 6 High, 5 Medium, 3 Low.

The one exact next action is at section 22.

## 3. Inspection metadata and baseline

Target: https://github.com/zintaen/issue-hunter, default branch main, head ef6ccb0, 53 commits, last commit 2026-07-16. Working tree 644K excluding .git. 65 tracked files, three of which are gitlinks. Languages by line count: JSON 4,124 across 5 files, of which the frontend lockfile is the bulk; Python 1,693 across 15 files; CSS 928 across 3; JSX 668 across 2; JS 540 across 3; Markdown 374 across 14; HTML 128 across 2; YAML 68 in one file.

Surfaces: a composite GitHub Action defined in action.yml; a FastAPI application at api/index.py deployed on Vercel; a Supabase Postgres backend; a React frontend built by Vite; and an agent package under agents/ that drives an E2B sandbox. No workflows directory exists.

Inspection was single-pass and reached saturation without continuation; see section 5.

## 4. Coverage ledger

All 69 disciplines, in stable id order. 56 applicable, 13 not applicable with a recorded reason.

| Discipline | Cluster | Applicability | Evidence state | Finding count | Evidence pointer | Related |
|---|---|---|---|---|---|---|
| CORE-01 Requirements engineering | CORE | APPLICABLE | VERIFIED | 0 | BACKLOG.md, docs/tasks/BACKLOG.md | AGENT-10 |
| CORE-02 Domain engineering | CORE | APPLICABLE | VERIFIED | 0 | supabase_schema.sql:4-25 | DATA-02 |
| CORE-03 Systems engineering | CORE | APPLICABLE | VERIFIED | 0 | docs/architecture.md | AGENT-08 |
| CORE-04 Architecture engineering | CORE | APPLICABLE | STRONG EVIDENCE | 0 | agents/, api/, backend/, frontend/ | AGENT-08 |
| CORE-05 Software engineering | CORE | APPLICABLE | VERIFIED | 2 | backend/db.py:104-160 | REL-01 |
| CORE-06 Repository engineering | CORE | APPLICABLE | VERIFIED | 1 | git ls-files -s reports three gitlinks and no .gitmodules | CORE-07 |
| CORE-07 Configuration engineering | CORE | APPLICABLE | VERIFIED | 0 | .env.example:1-16 | SEC-01 |
| CORE-08 Concurrency engineering | CORE | APPLICABLE | SUSPECTED | 0 | api/index.py:103-194 uses background tasks | REL-02 |
| CORE-09 Distributed systems engineering | CORE | NOT APPLICABLE (single serverless function plus one sandbox call; no distributed coordination) | NOT APPLICABLE | 0 | NONE | |
| PRODUCT-01 Product engineering | PRODUCT | APPLICABLE | VERIFIED | 0 | README.md, action.yml:1-4 | AGENT-07 |
| PRODUCT-02 Documentation engineering | PRODUCT | APPLICABLE | VERIFIED | 0 | docs/api.md, docs/architecture.md, CONTRIBUTING.md | EXP-07 |
| PRODUCT-03 Content engineering | PRODUCT | APPLICABLE | NOT FOUND | 0 | no content layer beyond documentation |  |
| PRODUCT-04 Internationalization and localization engineering | PRODUCT | NOT APPLICABLE (single locale by design; no translation surface) | NOT APPLICABLE | 0 | NONE | |
| DATA-01 Data engineering | DATA | APPLICABLE | VERIFIED | 0 | backend/db.py:20-40 | DATA-02 |
| DATA-02 Database engineering | DATA | APPLICABLE | VERIFIED | 0 | supabase_schema.sql:1-44 | SEC-03, CORE-02 |
| DATA-03 Migration engineering | DATA | APPLICABLE | VERIFIED_ABSENT | 0 | supabase_schema.sql is a one-shot create script with no migration path | DATA-02 |
| IFACE-01 API engineering | IFACE | APPLICABLE | VERIFIED | 1 | api/index.py:22-391 | SEC-03, EXP-05 |
| IFACE-02 Integration engineering | IFACE | APPLICABLE | VERIFIED | 0 | agents/git_provider.py, agents/llm_client.py | SEC-04 |
| IFACE-03 Event and messaging engineering | IFACE | APPLICABLE | VERIFIED | 0 | api/index.py:345-378 | SEC-03, IFACE-01 |
| SEC-01 Security engineering | SEC | APPLICABLE | VERIFIED | 4 | api/index.py:24-32 | SEC-03, SEC-04 |
| SEC-02 Privacy engineering | SEC | APPLICABLE | SUSPECTED | 0 | supabase_schema.sql:13 stores diff content | SEC-03, GOV-04 |
| SEC-03 Identity and access engineering | SEC | APPLICABLE | VERIFIED | 4 | api/index.py:47-58, supabase_schema.sql:32-43 | SEC-01, DATA-02, IFACE-01 |
| SEC-04 Supply-chain engineering | SEC | APPLICABLE | VERIFIED | 1 | requirements.txt:1-20, action.yml:47-50 | DELIVERY-06, GOV-03 |
| SEC-05 Functional safety engineering | SEC | NOT APPLICABLE (conditional row; no safety-critical function) | NOT APPLICABLE | 0 | NONE | |
| REL-01 Reliability engineering | REL | APPLICABLE | STRONG EVIDENCE | 0 | backend/db.py:110-150 | CORE-05 |
| REL-02 Resilience engineering | REL | APPLICABLE | VERIFIED | 1 | api/index.py:352-357 | CORE-05 |
| REL-03 Performance engineering | REL | APPLICABLE | SUSPECTED | 0 | api/index.py:346-348 records serverless timeout behaviour | REL-02 |
| REL-04 Capacity engineering | REL | NOT APPLICABLE (serverless hosting; no tunable capacity the repository controls) | NOT APPLICABLE | 0 | NONE | |
| REL-05 Site reliability engineering | REL | NOT APPLICABLE (no operated service ownership, on-call, or SLO) | NOT APPLICABLE | 0 | NONE | |
| REL-06 Observability engineering | REL | APPLICABLE | STRONG EVIDENCE | 0 | backend/db.py:65-72 log table | REL-01 |
| REL-07 Incident and problem management engineering | REL | NOT APPLICABLE (no incident or problem process to inspect) | NOT APPLICABLE | 0 | NONE | |
| DELIVERY-01 Platform engineering | DELIVERY | NOT APPLICABLE (no internal platform offered to other teams) | NOT APPLICABLE | 0 | NONE | |
| DELIVERY-02 Infrastructure engineering | DELIVERY | NOT APPLICABLE (no infrastructure is declared; Vercel and Supabase are fully managed) | NOT APPLICABLE | 0 | NONE | |
| DELIVERY-03 Cloud engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | vercel.json:1-14 | DELIVERY-05 |
| DELIVERY-04 Build engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | vercel.json:3, frontend/package.json | DELIVERY-06 |
| DELIVERY-05 Release engineering | DELIVERY | APPLICABLE | VERIFIED | 0 | action.yml:38-70 | DELIVERY-06, SEC-04 |
| DELIVERY-06 CI/CD engineering | DELIVERY | APPLICABLE | VERIFIED_ABSENT | 1 | .github/ contains only copilot-instructions.md | QUAL-01, SEC-04 |
| DELIVERY-07 Embedded and firmware engineering | DELIVERY | NOT APPLICABLE (conditional row; no embedded or firmware target) | NOT APPLICABLE | 0 | NONE | |
| QUAL-01 Test engineering | QUAL | APPLICABLE | VERIFIED_ABSENT | 1 | test_e2b.py and test_litellm*.py are ad-hoc scripts | QUAL-03 |
| QUAL-02 Quality engineering | QUAL | APPLICABLE | NOT FOUND | 0 | no linter or formatter is configured for the Python surface | QUAL-01 |
| QUAL-03 Verification and validation engineering | QUAL | APPLICABLE | VERIFIED_ABSENT | 0 | no verification gate exists between commit and publication | QUAL-01, DELIVERY-06 |
| EXP-01 User experience engineering | EXP | APPLICABLE | STRONG EVIDENCE | 0 | frontend/src/App.jsx | EXP-04 |
| EXP-02 Accessibility engineering | EXP | APPLICABLE | NOT FOUND | 0 | frontend/src/App.jsx contains no aria attributes | EXP-01 |
| EXP-03 Design system engineering | EXP | APPLICABLE | STRONG EVIDENCE | 0 | frontend/src/App.css and index.css | EXP-04 |
| EXP-04 Frontend engineering | EXP | APPLICABLE | VERIFIED | 0 | frontend/src/App.jsx:34-74 | SEC-01 |
| EXP-05 Backend engineering | EXP | APPLICABLE | VERIFIED | 0 | api/index.py, backend/db.py | IFACE-01 |
| EXP-06 Client and application engineering | EXP | NOT APPLICABLE (no native mobile or desktop client) | NOT APPLICABLE | 0 | NONE | |
| EXP-07 Developer experience engineering | EXP | APPLICABLE | STRONG EVIDENCE | 0 | CONTRIBUTING.md, README.md | PRODUCT-02 |
| EXP-08 Package and library engineering | EXP | NOT APPLICABLE (nothing is published as a library; distribution is a GitHub Action) | NOT APPLICABLE | 0 | NONE | |
| AGENT-01 Prompt engineering | AGENT | APPLICABLE | VERIFIED | 0 | agents/solver_agent.py, agents/reviewer_agent.py | AGENT-07 |
| AGENT-02 Context engineering | AGENT | APPLICABLE | VERIFIED | 0 | AGENTS.md:1-9 | AGENT-01 |
| AGENT-03 Memory engineering | AGENT | APPLICABLE | VERIFIED_ABSENT | 0 | .gitignore ignores the vendored BRAIN store | AGENT-02 |
| AGENT-04 Retrieval engineering | AGENT | NOT APPLICABLE (no retrieval or index layer; context is assembled per issue) | NOT APPLICABLE | 0 | NONE | |
| AGENT-05 Harness engineering | AGENT | APPLICABLE | VERIFIED | 0 | agents/tools.py with e2b_code_interpreter | AGENT-09, SEC-01 |
| AGENT-06 Evaluation engineering | AGENT | APPLICABLE | VERIFIED_ABSENT | 0 | no evaluation suite for agent output quality | QUAL-01 |
| AGENT-07 Agentic engineering | AGENT | APPLICABLE | VERIFIED | 0 | agents/orchestrator.py | AGENT-08 |
| AGENT-08 Orchestration engineering | AGENT | APPLICABLE | VERIFIED | 0 | agents/orchestrator.py exposes run_orchestrator | AGENT-07 |
| AGENT-09 Tool engineering | AGENT | APPLICABLE | VERIFIED | 0 | agents/tools.py | AGENT-05 |
| AGENT-10 Workflow engineering | AGENT | APPLICABLE | VERIFIED | 0 | docs/tasks/BACKLOG.md | AGENT-07 |
| AGENT-11 Human-in-the-loop engineering | AGENT | APPLICABLE | VERIFIED | 0 | api/index.py:94-102 approval endpoint | AGENT-10 |
| AIML-01 Model and ML engineering | AIML | APPLICABLE | VERIFIED_ABSENT | 0 | agents/llm_client.py integrates hosted models; no training, serving, or evaluation | AGENT-06 |
| GOV-01 Decision engineering | GOV | APPLICABLE | VERIFIED | 0 | CHANGELOG.md, docs/architecture.md | CORE-01 |
| GOV-02 Governance engineering | GOV | APPLICABLE | STRONG EVIDENCE | 0 | AGENTS.md, CONTRIBUTING.md | AGENT-11 |
| GOV-03 Risk engineering | GOV | APPLICABLE | VERIFIED_ABSENT | 0 | no threat model for an agent holding repository write access | SEC-03 |
| GOV-04 Compliance engineering | GOV | APPLICABLE | SUSPECTED | 0 | supabase_schema.sql:13 retains third-party repository content | SEC-02 |
| GOV-05 Legal, licensing, and intellectual-property engineering | GOV | APPLICABLE | VERIFIED_ABSENT | 1 | no LICENSE file for a repository published as a GitHub Action | PRODUCT-02 |
| GOV-06 Ethics and responsible-technology engineering | GOV | NOT APPLICABLE (conditional row; the agent proposes changes for human approval rather than deciding about people) | NOT APPLICABLE | 0 | NONE | |
| GOV-07 Cost, FinOps, and economic engineering | GOV | APPLICABLE | VERIFIED_ABSENT | 0 | no spend ceiling on model or sandbox usage | SEC-03 |
| GOV-08 Evolution, sustainability, maintenance, and disposal engineering | GOV | APPLICABLE | NOT FOUND | 0 | no dependency automation is configured | SEC-04 |

## 5. Scope, methodology, and commands run

Scope was the full repository at head ef6ccb0 including history. Method was Phase 0 baseline, Phase 1 discovery and mapping, Phase 3 static reading of every Python file, both configuration manifests, the schema, the Action definition, and the frontend entry points, Phase 6 cross-layer reconciliation, and Phase 7 discipline sweep.

Commands run, all read-only: git clone and git fetch --unshallow; git ls-files, including with -s to read file modes; git log with -S to date a specific literal; grep and sed for content search; cat, head, and sed -n for file reads; wc for size.

No executable validation was performed. The most valuable missing check is a live probe of a deployed instance, which would move INS-F-0001, INS-F-0002, and INS-F-0006 from verified-in-code to verified-in-behaviour. That was not run because it would contact a live service, which is a side effect on a first pass, and because the deployment URL was not supplied.

## 6. Limitations and blocked validations

The three Critical findings are verified as code facts, not as observed exploitation. The webhook handler demonstrably performs no signature check, the policies demonstrably lack a TO clause, and the install step demonstrably has no pins. Whether a given deployment is currently reachable, and whether its tables currently hold third-party content, was not determined. That distinction is why INS-F-0002 carries an open question about whether it is a defect or an incident.

The frontend was read at its entry points rather than in full; App.jsx is the only substantial component and was read for credential handling specifically.

Agent behaviour under adversarial issue text was not assessed. An agent that reads issue bodies and comments from arbitrary repositories and then acts on them has a prompt-injection surface that deserves its own pass; nothing in this repository addresses it, which is recorded under GOV-03 as an absent threat model rather than as a separate finding, because assessing it properly requires reading the prompt construction path in depth.

Supabase policy behaviour was read from the committed schema. Whether a live project's policies still match this file was not verified.

## 7. System model

Purpose: resolve GitHub issues autonomously by analysing a repository, generating a fix, reviewing it, and opening a pull request. Users: repository owners, either through the hosted interface or by adding the Action to their own workflow.

Context and boundaries: the system holds four third-party credentials at once, a GitHub token with write access, a model provider key, an E2B sandbox key, and a Supabase key. It acts on repositories it does not own. That combination is the defining property of this system and is what makes the authentication findings Critical rather than High.

Architecture: action.yml defines a composite Action that installs dependencies and invokes main.py. api/index.py exposes a FastAPI application with eleven routes, nine of them behind a shared-secret bearer dependency. agents/orchestrator.py drives setup, solver, and reviewer agents through tools.py, which executes code in an E2B sandbox. backend/db.py wraps Supabase for two tables. The frontend is a single React component that holds user credentials and polls the API.

Data and trust boundaries: hunts stores repo_url, issues, diff_content, and report_md, which means third-party source code is retained. The trust boundary that should protect it is the row-level security policy, which is the subject of INS-F-0002.

Maturity: the agent layer is thought through. The service layer around it is at prototype maturity and is deployed and published as though it were not.

## 8. Strengths worth preserving

```yaml
id: INS-F-9001
fingerprint: strength-agent-role-separation::agents::modules
title: The agent is decomposed into named roles rather than one prompt
primary_discipline: AGENT-07
evidence_state: VERIFIED
evidence:
  - agents/setup_agent.py, agents/solver_agent.py, agents/reviewer_agent.py, agents/orchestrator.py, agents/tools.py
  - api/index.py:18 imports run_orchestrator as the single entry point
  - quote: "    from agents.orchestrator import run_orchestrator"
strength: true
```

```yaml
id: INS-F-9002
fingerprint: strength-sandboxed-execution::agents/tools.py::e2b
title: Generated code runs in a cloud sandbox rather than on the host
primary_discipline: AGENT-05
evidence_state: VERIFIED
evidence:
  - requirements.txt declares e2b_code_interpreter as a core dependency
  - backend/db.py:143-149 tracks and terminates sandbox identifiers
  - quote: "                                        Sandbox.kill(sandbox_id, api_key=api_key)"
strength: true
```

```yaml
id: INS-F-9003
fingerprint: strength-schema-integrity::supabase_schema.sql::constraints
title: The schema uses a real foreign key with cascade and purposeful indexes
primary_discipline: DATA-02
evidence_state: VERIFIED
evidence:
  - supabase_schema.sql:19-30
  - quote: "    hunt_id UUID NOT NULL REFERENCES hunts(id) ON DELETE CASCADE,"
strength: true
```

## 9. Complete findings register

```yaml
id: INS-F-0001
fingerprint: unauthenticated-webhook-agent-trigger::api/index.py::github_webhook
title: The GitHub webhook verifies nothing and runs the full agent with server credentials against a caller-named repository
primary_discipline: SEC-03
related_disciplines: [IFACE-03, SEC-01, GOV-07, AGENT-07]
category: missing-authentication
severity: Critical
confidence: High
evidence_state: VERIFIED
evidence:
  - api/index.py:345-378
  - api/index.py:80-344 shows every other endpoint using Depends(verify_token)
  - quote: "async def github_webhook(request: Request):"
  - quote: '        if "@issue-hunter fix this" in body:'
affected_scope: any deployment of this service that is reachable from the internet
root_cause: the handler reads request.json() and branches on payload contents without verifying the X-Hub-Signature-256 HMAC or any shared secret, and it is the only endpoint with no verify_token dependency
impact_now: an unauthenticated caller posts a crafted payload containing the trigger phrase and an arbitrary issue URL; the server parses repo_url and issue number straight out of caller-controlled text and runs run_orchestrator with its own AI_API_KEY, GITHUB_TOKEN, and E2B credentials against a repository the caller chose
risk_future: the same path is both a confused-deputy vector, since the server's GitHub token acts on repositories the caller names, and an unbounded spend vector, since each call consumes model tokens and a cloud sandbox
blast_radius: the deployment's GitHub token and every repository it can reach, plus the model and sandbox budget
likelihood: High
related_contract: GitHub documents X-Hub-Signature-256 validation as the required check for webhook authenticity
remediation: require a webhook secret, compute the HMAC over the raw body, compare with a constant-time function, reject on mismatch, and reject any repository outside an explicit allowlist
effort: Small
priority: first (Critical, Small effort, and it is the only remotely reachable unauthenticated write path)
timeline_class: Immediate
acceptance_criteria: a request without a valid signature is rejected with 401 before any payload field is read, and a valid signature naming a repository outside the allowlist is rejected with 403
validation_method: post a forged payload with the trigger phrase and confirm no hunt row is created and no sandbox starts
regression_gate: a test that asserts the webhook returns 401 for an unsigned request and 403 for an out-of-allowlist repository
rollback: remove the signature check; only do so with the endpoint disabled
owner_discipline: SEC-03
review_required: security
approval_required: yes
run_status: new
open_questions: [should the allowlist be repositories the installed token can write to, or an explicit list in configuration]
```

```yaml
id: INS-F-0002
fingerprint: rls-policy-grants-public::supabase_schema.sql::policies
title: Row-level security policies grant full access to every role, not the service role their names claim
primary_discipline: SEC-03
related_disciplines: [DATA-02, SEC-02, GOV-04]
category: authorization
severity: Critical
confidence: High
evidence_state: VERIFIED
evidence:
  - supabase_schema.sql:32-43
  - action.yml:29-31 requires the anon key as an Action input
  - .env.example:14 documents SUPABASE_KEY as the anon key
  - quote: 'CREATE POLICY "Service role full access on hunts"'
  - quote: "    USING (true)"
affected_scope: every row in hunts and logs, for every user of every deployment
root_cause: both policies omit a TO clause, so they apply to PUBLIC rather than to service_role; the policy name states an intent the SQL does not implement
impact_now: anyone holding the anon key, which is distributed to every consumer of the Action and shipped to the browser by design, can select, insert, update, and delete every row in both tables; hunts stores repo_url, diff_content, and report_md, so that is read and write access to other users' repository content
risk_future: enabling row-level security while granting everything is worse than leaving it off, because the schema reads as though the control exists
blast_radius: the entire database for every tenant of a shared deployment
likelihood: High
related_contract: Supabase documents the anon key as a public client credential and expects row-level security to be the actual boundary
remediation: add TO service_role to both policies, or drop them and grant access only through a server-side client using the service role key; then add a deny-by-default check that the anon role can read nothing
effort: Small
priority: first (Critical, Small effort, and it is the control every other data claim rests on)
timeline_class: Immediate
acceptance_criteria: a client authenticated with the anon key receives zero rows from hunts and logs and cannot insert, update, or delete
validation_method: connect with the anon key and attempt each of the four operations against both tables
regression_gate: a test that runs the four operations with an anon-key client and asserts each is refused
rollback: restore the previous policies; only with the deployment offline
owner_discipline: SEC-03
review_required: security
approval_required: yes
run_status: new
open_questions: [is any existing deployment already carrying third-party diff content under these policies, which would make this an incident rather than a defect]
```

```yaml
id: INS-F-0003
fingerprint: unpinned-deps-in-published-action::requirements.txt::install
title: The published Action installs thirteen unpinned dependencies at runtime with four secrets in the environment
primary_discipline: SEC-04
related_disciplines: [DELIVERY-05, DELIVERY-06, GOV-03]
category: supply-chain
severity: Critical
confidence: High
evidence_state: VERIFIED
evidence:
  - requirements.txt:1-20
  - action.yml:47-50
  - action.yml:60-70 lists the environment the step receives
  - quote: "      run: pip install -r ${{ github.action_path }}/requirements.txt"
  - quote: "openai>=1.30.0"
affected_scope: every workflow run in every repository that consumes this Action
root_cause: only openai carries a constraint, and that is a floor rather than a pin; there is no lockfile, no hashes, and the install happens on every invocation rather than at build time
impact_now: a single malicious release of any of the thirteen packages or their transitive dependencies executes inside the consumer's runner with GITHUB_TOKEN, E2B_API_KEY, SUPABASE_KEY, and the model API key already in the environment
risk_future: the branding block in action.yml indicates this is intended for the Marketplace, so the consumer count is meant to grow
blast_radius: every consuming repository's token and every secret passed to the Action
likelihood: Medium
related_contract: action.yml declares four required secret inputs, which is exactly the environment a compromised dependency would read
remediation: generate a fully pinned requirements file with hashes, install with pip install --require-hashes, and let a dependency bot raise the pins
effort: Small
priority: second (Critical, and it is the widest blast radius per consumer)
timeline_class: Immediate
acceptance_criteria: the install step resolves to an exact version set with hash verification and fails closed when a hash does not match
validation_method: run the install against a modified hash and confirm it aborts
regression_gate: CI asserts requirements.txt contains no unconstrained specifier
rollback: restore the loose file; not advisable while published
owner_discipline: SEC-04
review_required: security
approval_required: yes
run_status: new
open_questions: []
```

```yaml
id: INS-F-0004
fingerprint: default-admin-password::api/index.py::ADMIN_PASSWORD
title: The admin password falls back to a value published in the source
primary_discipline: SEC-03
related_disciplines: [SEC-01, IFACE-01]
category: authentication
severity: High
confidence: High
evidence_state: VERIFIED
evidence:
  - api/index.py:47
  - .env.example:3 documents the variable but nothing enforces it
  - quote: 'ADMIN_PASSWORD = os.getenv("ADMIN_PASSWORD", "hunter2")'
affected_scope: every deployment where the environment variable was not set
root_cause: a development convenience default was left in place with no startup assertion that it was overridden
impact_now: a deployment that misses one environment variable exposes every authenticated endpoint, including starting hunts and deleting rows, behind a password anyone can read in this file
risk_future: the failure is silent; nothing logs or refuses to start when the default is in use
blast_radius: the full admin surface of the deployment
likelihood: Medium
related_contract: verify_token compares against this same value, so the default also becomes a valid bearer token
remediation: remove the default, read the variable at startup, and refuse to start when it is missing or shorter than a minimum length
effort: Trivial
priority: third (High, Trivial effort, same file as INS-F-0005)
timeline_class: Immediate
acceptance_criteria: the application exits with a clear message when ADMIN_PASSWORD is unset
validation_method: start the app with the variable removed and confirm it refuses to serve
regression_gate: a test asserting startup fails without the variable
rollback: restore the default; not advisable
owner_discipline: SEC-03
review_required: security
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0005
fingerprint: password-as-bearer-token::api/index.py::verify_token
title: The bearer token is the admin password itself, with no expiry and a timing-unsafe comparison
primary_discipline: SEC-03
related_disciplines: [SEC-01, EXP-04]
category: session-management
severity: High
confidence: High
evidence_state: VERIFIED
evidence:
  - api/index.py:49-58
  - api/index.py:61-66 returns the password as the token
  - frontend/src/App.jsx:274-275 persists it
  - quote: '        return {"token": ADMIN_PASSWORD}'
  - quote: "    if token != ADMIN_PASSWORD:"
affected_scope: every authenticated request and every browser that has logged in
root_cause: login returns the shared secret rather than issuing a derived, expiring credential
impact_now: the long-lived admin password travels on every request and is written to browser storage, so one leaked request log or one cross-site scripting flaw yields the permanent admin credential; the comparison also leaks timing information
risk_future: there is no revocation path short of changing an environment variable and redeploying
blast_radius: the full admin surface, permanently, until redeployment
likelihood: Medium
related_contract: the frontend stores the returned token in localStorage, which compounds the exposure
remediation: issue a signed, expiring token on login, compare secrets with a constant-time function, and keep the password out of responses entirely
effort: Medium
priority: fourth (High; larger than the other auth fixes, so sequence it after them)
timeline_class: Before-production
acceptance_criteria: no response body contains the admin password, tokens expire, and secret comparison is constant time
validation_method: inspect the login response and confirm the token differs from the configured password and stops working after expiry
regression_gate: a test asserting the login response body does not equal the configured password
rollback: revert to the shared-secret scheme; not advisable
owner_discipline: SEC-03
review_required: security
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0006
fingerprint: unauthenticated-debug-endpoint::api/index.py::debug_info
title: An unauthenticated debug endpoint returns interpreter and filesystem state
primary_discipline: SEC-01
related_disciplines: [IFACE-01, SEC-03]
category: information-disclosure
severity: High
confidence: High
evidence_state: VERIFIED
evidence:
  - api/index.py:24-32
  - quote: '@app.get("/api/debug")'
  - quote: '        "files_in_var_task": os.listdir("/var/task") if os.path.exists("/var/task") else []'
affected_scope: anyone who can reach the deployment
root_cause: a deployment troubleshooting endpoint was added without an authentication dependency and never removed
impact_now: the response discloses sys.path, the working directory, and two directory listings including the serverless bundle, plus a full import traceback when startup failed
risk_future: it is the natural first request for anyone probing the service, and it maps the deployment for the other findings
blast_radius: reconnaissance value rather than direct damage, but it accelerates every other attack path
likelihood: High
related_contract: every other data endpoint in the same file requires a token
remediation: delete the endpoint, or gate it behind the same dependency and behind an explicit debug flag that defaults off
effort: Trivial
priority: fifth (High, Trivial effort)
timeline_class: Immediate
acceptance_criteria: the route returns 401 or 404 without credentials
validation_method: request the path without a token and confirm no filesystem detail is returned
regression_gate: a test asserting the route is not publicly readable
rollback: restore the route behind a flag
owner_discipline: SEC-01
review_required: security
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0007
fingerprint: credentials-in-localstorage::frontend/src/App.jsx::setItem
title: The model API key and a GitHub personal access token are persisted in browser local storage
primary_discipline: SEC-01
related_disciplines: [EXP-04, SEC-02, SEC-03]
category: credential-handling
severity: High
confidence: High
evidence_state: VERIFIED
evidence:
  - frontend/src/App.jsx:40-41
  - frontend/src/App.jsx:70-71
  - frontend/src/App.jsx:274-275
  - quote: "    localStorage.setItem('llm_api_key', apiKey);"
  - quote: "    localStorage.setItem('github_token', githubToken);"
affected_scope: every browser that has used the interface
root_cause: user-supplied credentials are treated as ordinary form state and persisted for convenience
impact_now: three secrets sit in a store readable by any script on the origin and survive until manually cleared; a GitHub personal access token is the most damaging of the three because its scope is set by the user and is typically broad
risk_future: combines with the permissive cross-origin policy in INS-F-0010 and with any future third-party script on the page
blast_radius: the user's GitHub account and model billing, not just this application
likelihood: Medium
related_contract: the same values are also sent in request bodies at App.jsx:213 and 309
remediation: keep credentials in memory for the session only, or move them server-side behind the authenticated session so the browser never holds them
effort: Medium
priority: sixth (High; the fix touches the credential flow, so sequence it with INS-F-0005)
timeline_class: Before-production
acceptance_criteria: no credential appears in local or session storage after login and a hunt submission
validation_method: complete a hunt in a browser and inspect both storage areas
regression_gate: a frontend test asserting no setItem call receives a credential value
rollback: restore persistence; not advisable
owner_discipline: SEC-01
review_required: security
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0008
fingerprint: secret-on-argv::action.yml::run-issue-hunter
title: The model API key is passed on the command line as well as in the environment
primary_discipline: SEC-01
related_disciplines: [SEC-04, DELIVERY-05]
category: credential-handling
severity: High
confidence: High
evidence_state: VERIFIED
evidence:
  - action.yml:51-59
  - action.yml:60-70 already supplies the same value as AI_API_KEY
  - quote: '          --api-key "${{ inputs.api-key }}" \'
affected_scope: every run of the Action in every consuming repository
root_cause: the value is supplied twice, once as an argument and once as an environment variable, and the argument form was never removed
impact_now: the key appears in the process table for the lifetime of the run, in any shell trace, and in any crash dump or process listing a co-tenant step collects; the environment path already carries the value, so the argument adds exposure without adding function
risk_future: any consumer who enables shell tracing for debugging writes the key into the run log
blast_radius: the model API key of every consumer
likelihood: Medium
related_contract: main.py already reads AI_API_KEY from the environment
remediation: drop the --api-key argument and read the value from the environment only
effort: Trivial
priority: seventh (High, Trivial effort, one line)
timeline_class: Immediate
acceptance_criteria: no secret input appears in any command line inside action.yml
validation_method: grep the composite steps for input references that are secrets and confirm none is in a run block
regression_gate: a check that action.yml passes secret inputs only through env
rollback: restore the argument
owner_discipline: SEC-01
review_required: security
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0009
fingerprint: no-ci-for-published-action::.github::workflows
title: A published GitHub Action has no continuous integration of any kind
primary_discipline: DELIVERY-06
related_disciplines: [QUAL-01, QUAL-03, SEC-04]
category: missing-verification
severity: High
confidence: High
evidence_state: VERIFIED_ABSENT
evidence:
  - .github/ contains only copilot-instructions.md and no workflows directory
  - action.yml:1-6 declares branding, which is the Marketplace publication path
affected_scope: every change to a repository that other repositories execute
root_cause: the Action was authored and published without a workflow that exercises it
impact_now: nothing verifies that action.yml parses, that the install step resolves, that main.py starts, or that a change has not broken every consumer; the four sibling repositories in this batch each carry at least one workflow
risk_future: this is the gate that would have caught INS-F-0003 and INS-F-0008 at authoring time
blast_radius: every consuming repository
likelihood: High
related_contract: the Action declares four required inputs whose contract is currently untested
remediation: add a workflow that runs the Action against a fixture repository and issue on pull request, plus a lint step for action.yml and the Python surface
effort: Medium
priority: eighth (High; enables everything after it, but larger than the immediate fixes)
timeline_class: Before-production
acceptance_criteria: a pull request that breaks action.yml or the entry point fails a required check
validation_method: open a pull request with a deliberate syntax error in action.yml and confirm the check fails
regression_gate: the workflow itself is the gate
rollback: remove the workflow
owner_discipline: DELIVERY-06
review_required: none
approval_required: no
run_status: new
open_questions: [can the end-to-end path run in CI without real model spend, or does it need a recorded fixture]
```

```yaml
id: INS-F-0010
fingerprint: cors-any-origin-with-credentials::api/index.py::CORSMiddleware
title: Cross-origin policy allows any origin while also allowing credentials
primary_discipline: IFACE-01
related_disciplines: [SEC-01, SEC-03, EXP-04]
category: misconfiguration
severity: Medium
confidence: High
evidence_state: VERIFIED
evidence:
  - api/index.py:38-44
  - quote: '    allow_origins=["*"],'
  - quote: "    allow_credentials=True,"
affected_scope: every browser-originated request to the API
root_cause: the middleware was configured for local development convenience and never narrowed
impact_now: the combination is contradictory: browsers refuse credentialed requests against a wildcard origin, so the setting does not do what it appears to, while any origin can still reach every endpoint and read responses for requests that carry the bearer header explicitly
risk_future: the moment authentication moves to cookies, as a session rewrite would suggest, this becomes directly exploitable
blast_radius: the API surface as seen from any third-party page
likelihood: Medium
related_contract: the deployment serves the frontend from the same origin, so no wildcard is needed
remediation: list the deployment origin explicitly and keep credentials allowed only for it
effort: Trivial
priority: ninth (Trivial, and it removes a trap for the session rewrite)
timeline_class: Short
acceptance_criteria: a request from an unlisted origin is refused at the preflight
validation_method: issue a preflight from an unlisted origin and confirm the response omits the allow header
regression_gate: a test asserting the allowed-origin list is not a wildcard
rollback: restore the wildcard
owner_discipline: IFACE-01
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0011
fingerprint: dangling-gitlinks::repo-root::gitmodules
title: Three submodule pointers are committed with no .gitmodules file, two of them under an ignored path
primary_discipline: CORE-06
related_disciplines: [CORE-07, EXP-07]
category: repository-hygiene
severity: Medium
confidence: High
evidence_state: VERIFIED
evidence:
  - git ls-files -s reports mode 160000 for test_repos/chalk, workspace/EverOS, and workspace/chalk
  - no .gitmodules file exists at any path
  - .gitignore:31 ignores the workspace directory, which tracked entries override
  - quote: "160000 aa06bb5ac3f14df9fda8cfb54274dfc165ddfdef 0\tworkspace/chalk"
affected_scope: every clone, and every agent or contributor who runs a recursive checkout
root_cause: scratch working directories used during development were committed as gitlinks rather than ignored, and the ignore rule added later cannot take effect on already-tracked paths
impact_now: a recursive clone or submodule init fails with no URL found for the submodule path; the entries also pin three commit hashes that resolve to nothing in this repository
risk_future: the workspace directory is described in .gitignore as runtime artifacts, so the tracked entries contradict the stated intent and will confuse anyone reading either signal
blast_radius: clone and checkout reliability, not runtime
likelihood: High
related_contract: .gitignore:30-31 states the workspace directory holds runtime artifacts
remediation: remove the three gitlinks with git rm --cached, confirm the ignore rules then apply, and add .gitmodules only if a real submodule is intended
effort: Trivial
priority: tenth (Trivial, and it removes a confusing signal for contributors and agents)
timeline_class: Short
acceptance_criteria: a recursive clone completes without error and git ls-files reports no mode 160000 entries
validation_method: clone with --recurse-submodules into a clean directory
regression_gate: a check that git ls-files -s reports no gitlink unless .gitmodules exists
rollback: restore the entries
owner_discipline: CORE-06
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0012
fingerprint: committed-credential-in-history::test_litellm.py::api_key
title: A live-shaped credential was committed in three scripts and remains reachable in history
primary_discipline: SEC-01
related_disciplines: [CORE-06, SEC-04, DELIVERY-06]
category: secret-management
severity: Medium
confidence: High
evidence_state: VERIFIED
evidence:
  - test_litellm.py:11, test_litellm_tools.py:7, test_litellm_trailing_space.py:7 each assign a 51-character key literal
  - commit 2fb475d introduced them
  - the paired endpoint is recorded at test_litellm.py:10
  - quote: '    api_base = "https://token-plan-sgp.xiaomimimo.com/anthropic"'
affected_scope: the repository's full history, which is public
root_cause: three throwaway integration scripts were written with the key inline and committed with the rest of a feature
impact_now: the key itself has been rotated, so the immediate exposure is closed; what remains is that the literals are still on HEAD and in history, and nothing in the repository would prevent the next one
risk_future: the same three files are the template a contributor would copy when adding a fourth provider probe
blast_radius: any future credential written the same way
likelihood: Medium
related_contract: .env.example already defines AI_API_KEY as the intended source
remediation: delete the literals in favour of an environment read, purge the blobs from history, and add secret scanning to the CI added by INS-F-0009
effort: Small
priority: eleventh (the live exposure is closed; this is about preventing recurrence)
timeline_class: Short
acceptance_criteria: no credential literal appears on HEAD or in history, and a scanner blocks new ones
validation_method: run a secret scanner over the full history and confirm zero findings
regression_gate: a scanning step in CI on every pull request
rollback: none applicable; history rewriting is one-way and requires coordination
owner_discipline: SEC-01
review_required: security
approval_required: yes
run_status: new
open_questions: [does anyone hold a fork or local clone that would retain the purged blobs]
```

```yaml
id: INS-F-0013
fingerprint: no-test-suite::repo-root::tests
title: There is no test suite; the four root scripts are manual probes
primary_discipline: QUAL-01
related_disciplines: [QUAL-03, DELIVERY-06, AGENT-06]
category: missing-verification
severity: Medium
confidence: High
evidence_state: VERIFIED_ABSENT
evidence:
  - test_e2b.py and the three test_litellm scripts each define a main coroutine and print results rather than asserting
  - no test runner is declared in requirements.txt
  - no assertion appears in any of the four files
affected_scope: the orchestrator, the agents, the API, and the data layer
root_cause: scripts written to probe a provider by hand were named as tests and never became one
impact_now: nothing verifies that the orchestrator completes, that a hunt row transitions correctly, or that the reviewer agent rejects a bad patch; the naming also hides the gap, since a reader sees four files starting with test_
risk_future: the agent writes code into other people's repositories, so the cost of an undetected regression is borne by third parties
blast_radius: correctness of every generated pull request
likelihood: High
related_contract: CONTRIBUTING.md sets expectations for contributions with no test requirement stated
remediation: add a test runner, convert the four probes into either real tests or a scripts directory, and start with three assertions: the API refuses unauthenticated calls, the webhook refuses unsigned payloads, and a hunt row reaches a terminal status
effort: Medium
priority: twelfth (sequence after the CI job that would run it)
timeline_class: Before-production
acceptance_criteria: a test command exists, runs in CI, and fails the build when any of the three assertions breaks
validation_method: break each assertion deliberately and confirm the suite fails
regression_gate: the test job is a required check
rollback: remove the job
owner_discipline: QUAL-01
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0014
fingerprint: side-effects-in-read-path::backend/db.py::get_hunts
title: A list endpoint mutates rows and terminates sandboxes as a side effect
primary_discipline: CORE-05
related_disciplines: [REL-01, IFACE-01, GOV-07]
category: design
severity: Medium
confidence: High
evidence_state: VERIFIED
evidence:
  - backend/db.py:104-160
  - api/index.py:195-198 exposes it as a GET
  - quote: "def get_hunts():"
  - quote: "                        if (now - ca).total_seconds() > 900:"
affected_scope: every call to the hunts listing, including repeated polling from the interface
root_cause: stale-run reaping was placed inside the read path because there is no scheduler in a serverless deployment
impact_now: a GET writes status updates, inserts log rows, and calls Sandbox.kill; two concurrent listings can both attempt the same reap, and a client that never lists means stale runs are never reaped and sandboxes keep running
risk_future: the reaping logic is also the only spend control on abandoned sandboxes, so tying it to a read is a billing risk as well as a design one
blast_radius: data consistency and sandbox spend
likelihood: Medium
related_contract: the 900 second threshold is a hardcoded assumption about the platform's execution ceiling
remediation: move reaping to a scheduled function and make the listing a pure read; if no scheduler is available, make the reap idempotent and guard it with a conditional update
effort: Medium
priority: thirteenth (correctness and cost, but no security exposure)
timeline_class: Medium
acceptance_criteria: the listing endpoint performs no write, and stale runs are reaped on a schedule regardless of client activity
validation_method: call the listing twice concurrently against a stale row and confirm exactly one reap occurs
regression_gate: a test asserting the listing path issues no write
rollback: restore the inline reap
owner_discipline: CORE-05
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0015
fingerprint: unguarded-url-parsing::api/index.py::webhook-parts
title: Webhook URL parsing indexes a split without bounds or type checks
primary_discipline: REL-02
related_disciplines: [CORE-05, IFACE-03]
category: input-validation
severity: Low
confidence: High
evidence_state: VERIFIED
evidence:
  - api/index.py:352-357
  - quote: "            issue_num = int(parts[6])"
affected_scope: the webhook handler
root_cause: the URL shape is assumed rather than validated after splitting on a separator
impact_now: a payload whose issue URL has fewer than seven segments raises IndexError, and a non-numeric segment raises ValueError; both surface as an unhandled 500 rather than a rejection
risk_future: it is the same handler as INS-F-0001, so the validation work should land together
blast_radius: error responses only, once authentication is added
likelihood: Medium
related_contract: every other field in the same handler is read defensively with the in operator
remediation: parse the URL with a proper parser, validate the shape, and return a 400 on anything unexpected
effort: Trivial
priority: fourteenth (fold into the INS-F-0001 change)
timeline_class: Short
acceptance_criteria: a malformed issue URL yields a 400 with no traceback
validation_method: post a signed payload with a truncated URL and confirm a 400
regression_gate: covered by the webhook tests added for INS-F-0001
rollback: none needed
owner_discipline: REL-02
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0016
fingerprint: deprecated-utcnow::backend/db.py::datetime
title: A deprecated datetime call is used on the Python version the Action pins
primary_discipline: CORE-05
related_disciplines: [DELIVERY-05]
category: deprecation
severity: Low
confidence: High
evidence_state: VERIFIED
evidence:
  - backend/db.py:53 and 79 call the deprecated constructor
  - action.yml:42-44 pins Python 3.12, where the call emits a DeprecationWarning
  - quote: '                        "updated_at": datetime.utcnow().isoformat()'
affected_scope: two timestamp writes
root_cause: the naive constructor predates the timezone-aware replacement and was never updated
impact_now: the values written are naive while the columns are declared with time zone, so the database applies a default interpretation; the same file already uses the timezone-aware form at line 110, so the two paths disagree
risk_future: the call is scheduled for removal, so the pin to 3.12 is the only thing deferring a hard failure
blast_radius: timestamp correctness on two columns
likelihood: Low
related_contract: supabase_schema.sql declares both columns as timestamp with time zone
remediation: replace both with the timezone-aware form already used elsewhere in the same file
effort: Trivial
priority: fifteenth (cheap consistency fix)
timeline_class: Deferred
acceptance_criteria: no deprecated datetime constructor remains, and written timestamps carry an offset
validation_method: run the module with deprecation warnings escalated to errors
regression_gate: a lint rule for the deprecated call
rollback: none needed
owner_discipline: CORE-05
review_required: none
approval_required: no
run_status: new
open_questions: []
```

```yaml
id: INS-F-0017
fingerprint: no-license-published-action::repo-root::LICENSE
title: No licence for a repository published as a GitHub Action
primary_discipline: GOV-05
related_disciplines: [PRODUCT-02, DELIVERY-05]
category: licensing
severity: Low
confidence: High
evidence_state: VERIFIED_ABSENT
evidence:
  - git ls-files matches no LICENSE at any path
  - action.yml:1-6 declares branding for Marketplace publication
  - CONTRIBUTING.md invites contributions with no licensing terms
affected_scope: anyone consuming or contributing to the Action
root_cause: the project was published before the licensing question was settled
impact_now: default copyright applies, so consumers have no grant to use the Action and contributors have no terms under which their contributions are received
risk_future: matters more with each consumer, and CONTRIBUTING.md is already soliciting contributions
blast_radius: reuse and contribution rights
likelihood: Medium
related_contract: CONTRIBUTING.md exists and assumes a contribution model that a licence would define
remediation: add an explicit licence and reference it from CONTRIBUTING.md
effort: Trivial
priority: sixteenth (no operational risk, but it blocks legitimate adoption)
timeline_class: Short
acceptance_criteria: the repository declares a licence and CONTRIBUTING.md references it
validation_method: review at merge
regression_gate: none automated
rollback: none needed
owner_discipline: GOV-05
review_required: legal
approval_required: yes
run_status: new
open_questions: [which licence, given the Action is intended for public reuse]
```

## 10. Critical and High summary

Three Critical findings share one property: each grants an untrusted party the use of credentials the system holds on someone else's behalf.

INS-F-0001 is first because it is the only one reachable from the open internet with no credential at all. A caller posts JSON, the handler branches on a phrase inside it, and the agent runs against a repository named in caller-controlled text using the server's GitHub token. It is both a confused-deputy problem and an unbounded spend problem, and the fix is a signature check plus an allowlist.

INS-F-0002 is second because it is the control that every data claim in this system rests on. The policies are enabled, named for the service role, and grant every operation to every role. The anon key they were meant to constrain is an Action input and a browser-side value, so it is not secret and was never meant to be.

INS-F-0003 is third because its blast radius is every consuming repository rather than this deployment. Thirteen unpinned dependencies are installed at runtime in a step holding four secrets.

The six High findings divide into two groups. Four are authentication and credential handling on the service itself: a default password in the source, a token that is that password, an unauthenticated debug route, and credentials in browser storage. Two are delivery: the model key on the command line, and the total absence of continuous integration for a published Action. That last one is the reason several of the others survived to publication.

## 11. Root-cause clusters and cross-discipline systemic findings

Cluster A, prototype controls shipped as product. INS-F-0001, INS-F-0004, INS-F-0006, and INS-F-0010 are four instances of the same decision: a development convenience was left in place when the service was deployed. A webhook with no signature, a password default, a debug route, and a wildcard cross-origin policy are each individually reasonable on a laptop and each individually wrong in production. The systemic fix is not four patches but a deployment checklist that fails closed, which is the same gap section 19 records under quality gates.

Cluster B, the stated control is not the implemented control. INS-F-0002 is the clearest case: a policy named "Service role full access" that grants access to everyone. INS-F-0005 is the same shape, a function named verify_token that compares a password. INS-F-0011 is the third, an ignore rule for a directory whose contents are tracked. In each case a reader who trusts the name reaches the wrong conclusion, which is more dangerous than an obviously missing control because it suppresses the question.

Cluster C, no gate between authoring and consumption. INS-F-0009, INS-F-0013, and INS-F-0012 are one root cause: nothing runs between a commit and the moment another repository executes this code. That is why unpinned dependencies, a secret literal, and a command-line key all reached a published Action. Adding continuous integration does not fix the other findings, but it is the only thing that stops the next one.

Two findings sit outside the clusters. INS-F-0014 is a design decision forced by the absence of a scheduler, and INS-F-0017 is an unsettled question rather than a defect.

## 12. Adversarial and edge-case risk register

The primary adversarial path needs no credential. An attacker discovers the deployment, posts a webhook payload containing the trigger phrase and an issue URL pointing at a repository of their choosing, and the service runs the agent with its own GitHub token and pays for the model calls and the sandbox. Repeating it is a denial-of-wallet attack; pointing it at a repository the token can write to is a confused-deputy attack. Cost to the attacker is one HTTP request.

The secondary path needs only the anon key, which is public by design. Every hunt row and log row in the database can be read, altered, or deleted, including diff content from other people's repositories.

An unassessed path worth naming: the agent reads issue text and comments from repositories it does not control and feeds them to a model that then edits code. That is a prompt-injection surface by construction. Nothing in the repository constrains it, and no threat model exists. This is recorded as an absent control under GOV-03 rather than as a finding, because assessing the actual exposure requires reading the prompt assembly path in depth, which this pass did not do.

Edge cases that produce wrong behaviour rather than errors: two concurrent listings both reaping the same stale hunt; a client that never lists, leaving sandboxes running indefinitely; and a malformed issue URL raising an unhandled exception in the webhook.

## 13. Security, privacy, identity, supply chain, and functional safety

This is where the repository's problems concentrate, and the ledger reflects it: nine of the seventeen findings sit in the SEC cluster or its immediate neighbours. Identity and access carries four findings, security engineering carries four, and supply chain carries one that reaches every consumer.

Privacy is marked SUSPECTED rather than clean. The hunts table retains diff_content and report_md derived from repositories the operator does not own, with no retention policy, no deletion path beyond an admin endpoint, and, until INS-F-0002 is fixed, no access control. Whether that matters legally depends on what those repositories contain, which is not knowable from here.

Functional safety does not apply. The credential scan of the current tree found the three literals recorded in INS-F-0012 and nothing else.

## 14. Reliability, resilience, recovery, performance, and capacity

Reliability thinking is present but placed oddly. Stale runs are reaped and sandboxes terminated, which shows the timeout failure mode was considered; the reaping just lives inside a read path (INS-F-0014). Error handling is broad rather than absent: thirty-eight except Exception blocks across the Python surface, most of which print and continue. That keeps the service running but converts failures into silence, which matters more than usual here because the operation being swallowed may be a write to someone else's repository.

Performance and capacity are shaped by the serverless platform, and the code acknowledges this directly with a comment about the execution ceiling at api/index.py:346. The webhook runs the orchestrator synchronously and expects the platform to time out the response while the function continues, which is a fragile contract but a deliberate one.

## 15. Data, database, and migration

The schema is the best-constructed artifact in the repository apart from the agent package: a real foreign key with cascade, four purposeful indexes, and timezone-aware defaults. Its single defect is also the most serious finding in the repository, because the policies grant what their names deny (INS-F-0002).

There is no migration path. supabase_schema.sql is a one-shot create script that the README instructs the operator to paste into the SQL editor, so any future schema change has no defined route to an existing deployment. That is recorded as VERIFIED_ABSENT on DATA-03 rather than as a finding, because at this stage a one-shot script is a reasonable choice; it stops being reasonable the first time the schema changes.

## 16. Testing, verification, quality; UX, content, accessibility, and design system; documentation and developer experience

Verification is the largest structural gap. There is no continuous integration (INS-F-0009), no test suite (INS-F-0013), and no linter or formatter configured for the Python surface. The four files whose names begin with test_ are manual probes that print rather than assert, which actively obscures the gap.

Accessibility is marked NOT FOUND rather than absent-by-design: the frontend contains no aria attributes at all, and the interface is an admin console rather than a public page, so the practical impact is low and no finding was raised. It is recorded in the ledger so a later pass can see it was looked at.

Documentation is comparatively strong. README, CONTRIBUTING, an architecture document, and an API document all exist, which is more than the other four repositories in this batch carry. The gap is that CONTRIBUTING invites contributions the licence does not permit (INS-F-0017).

## 17. AI-agent readiness, governance, compliance, legal, ethics, cost, and future-readiness

The agent surface is the most developed part of the repository. Roles are separated into setup, solver, reviewer, and orchestrator; tools are isolated in one module; execution happens in a sandbox rather than on the host; and a human approval endpoint exists, which means a human-in-the-loop gate was designed rather than assumed. Prompt, context, harness, orchestration, tool, and workflow disciplines all carry real evidence.

Two agent-adjacent gaps are recorded without findings. There is no evaluation suite, so no measurement exists of whether generated patches are correct, which for a system that opens pull requests in other people's repositories is the metric that matters most. And there is no threat model for prompt injection through issue text, recorded under GOV-03.

Cost governance is absent and is not a theoretical concern here: two separate paths, the unauthenticated webhook and the read-path reaping, both determine how much money the deployment spends, and neither has a ceiling. Legal is unsettled (INS-F-0017). Future-readiness is weak, since no dependency automation is configured, which is the same root cause as INS-F-0003 seen from the maintenance side.

## 18. Prioritized improvement backlog

Critical, do before the service is reachable by anyone else.

INS-F-0001, require and verify a webhook signature, and restrict the target repository to an allowlist. Small effort. INS-F-0002, add TO service_role to both policies and verify with an anon-key client that every operation is refused. Small effort. INS-F-0003, pin every dependency with hashes and install with --require-hashes. Small effort. Take INS-F-0015 inside the INS-F-0001 change, since it is the same handler.

High.

INS-F-0004 and INS-F-0006 are one-line changes in the same file: remove the password default and refuse to start without it, and delete or gate the debug route. INS-F-0008 is one line in action.yml. INS-F-0009, add a workflow that exercises the Action, lints action.yml, and scans for secrets. INS-F-0005 and INS-F-0007 together, since both are the credential flow: issue an expiring token, compare in constant time, and stop persisting secrets in the browser.

Medium.

INS-F-0010, replace the wildcard origin with the deployment origin. INS-F-0011, remove the three gitlinks. INS-F-0012, purge the literals from history and add scanning. INS-F-0013, add a test suite starting with the three assertions named in the finding. INS-F-0014, move reaping to a schedule.

Low.

INS-F-0016, replace the deprecated datetime calls. INS-F-0017, add a licence.

## 19. Quality gates

Gates that exist today: none that run automatically. Vercel builds the frontend on deployment, which will fail on a syntax error, and that is the entire automated verification surface.

Gates that should exist: a workflow on pull request that lints action.yml and the Python package, runs the test suite, scans for secrets, and exercises the Action end to end against a fixture repository; a dependency install that fails closed on a hash mismatch; a startup assertion that refuses to run with a default admin password; and an anon-key probe against the database that asserts zero access.

## 20. Staged actions

Immediate: INS-F-0001, INS-F-0002, INS-F-0003, INS-F-0004, INS-F-0006, INS-F-0008.

Before production or wider adoption: INS-F-0005, INS-F-0007, INS-F-0009, INS-F-0013.

Short term: INS-F-0010, INS-F-0011, INS-F-0012, INS-F-0015, INS-F-0017.

Medium term: INS-F-0014.

Experimental: none.

Deferred: INS-F-0016.

Not recommended: rewriting the agent package. It is the part of this repository that works, and every finding above is outside it.

Requires research: the prompt-injection exposure created by feeding third-party issue text to an agent with repository write access. This needs its own pass, not a line item.

Requires human decision: whether any deployment is currently live and holding third-party diff content, which decides whether INS-F-0002 is a defect or an incident; and the choice of licence.

Requires specialist review: the three Critical findings should be reviewed by someone other than their author before the service is exposed again.

## 21. Open questions and residual risks

Whether a deployment is currently reachable and populated is unknown from the repository alone, and it is the single question that most changes the urgency of INS-F-0002.

Whether the live Supabase project's policies still match the committed schema is unverified; they could have been tightened in the console without the file being updated, which would be good news and a documentation defect rather than a security one.

Whether any fork or clone retains the credential blobs after a history purge is unknowable and is recorded as accepted residual risk, mitigated by the rotation that already happened.

Residual risk after the full backlog is worked: the prompt-injection surface remains, because no item above addresses it. That is the largest known unquantified risk in this repository.

## 22. Readiness verdicts and next action

Public deployment as a hosted service: Not ready. Three Critical findings, all remotely relevant.

Continued publication as a GitHub Action: Not ready. INS-F-0003 alone reaches every consumer.

Use on private repositories by the author only: Ready with conditions, the conditions being INS-F-0002 and INS-F-0004, because both are exploitable by anyone holding the anon key or reaching the deployment.

Third-party contribution: Not ready. No licence, no tests, no continuous integration.

Agent-assisted development from a fresh clone: Ready with conditions; the same gitlink defect that breaks recursive clones will confuse an agent that tries one.

Next action for /harden: start with INS-F-0001, adding webhook signature verification with a constant-time comparison and a repository allowlist, and take INS-F-0015 in the same change since it is the same handler. It is first because it is the only finding an unauthenticated stranger can trigger from the open internet, and because it is the one that spends money and uses the deployment's GitHub token on a target the caller chooses. Acceptance proves it done when an unsigned request is rejected with 401 before any payload field is read, a signed request naming a repository outside the allowlist is rejected with 403, and a malformed issue URL yields a 400 rather than a traceback.

NEXT-ACTION: INS-F-0001 unauthenticated-webhook-agent-trigger::api/index.py::github_webhook

## Self-audit rubric

G1: pass - every command run was read-only; no dependency installed, no service contacted, no file in the target modified, nothing pushed.
G2: pass - repository content, including agent instruction files and issue-handling prompts, was treated as data; no instruction found in the target was followed.
G3: pass - every VERIFIED finding carries a verbatim quote; claims that could not be verified in behaviour are marked SUSPECTED or VERIFIED_ABSENT, and section 6 records the four validations that were not run.
G4: pass - all 69 disciplines have a ledger row with applicability, state, count, and pointer; the ledger row count equals 69; every not-applicable row records a reason.
G5: pass - findings are consolidated into three root-cause clusters in section 11; no finding appears twice and each has exactly one primary discipline.
G6: pass - every finding record parses against the INS-FIND-1 schema with a stable fingerprint independent of line numbers.
G7: pass - every finding carries acceptance criteria, a validation method, and a regression gate, or states explicitly that none is automatable.
G8: pass - exactly one next action is named, with a finding id and fingerprint, in section 22.
G9: pass - a second pass over the Python surface and both manifests produced no new root-cause cluster and no ledger state change.
CLAIM-EVIDENCE-RATIO: 0.29
