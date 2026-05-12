---
fr_id: FR-001
title: Slack HR-policy bot MVP
profile: solo
project: 2026-05-12-build-a-slack-bot-that-answers-hr-policy
status: draft
eu_ai_act_risk_class: limited
client_visible: false
acceptance_criteria:
  - "Bot installed in CyberSkill Slack workspace"
  - "Answers 24/30 test questions correctly (80%)"
  - "p95 response time < 2s"
  - "Logs every Q&A to a private Slack channel for audit"
  - "Supports markdown answers (lists + code blocks)"
  - "Rate-limited to 100 req/h per user"
task_count: 6
tasks:
  - id: FR-001-T-01
    title: Set up Slack app + OAuth scopes
    description: |
      Create a new Slack app in the CyberSkill workspace via api.slack.com.
      Grant the following bot-token scopes: app_mentions:read, chat:write,
      chat:write.public, commands, im:history, im:read, im:write.
      Generate the signing secret and bot token. Store them in 1Password
      under "CyberSkill HR Bot" and inject into the runtime via env vars
      SLACK_BOT_TOKEN and SLACK_SIGNING_SECRET. Test the install in a
      private dev channel by running /healthcheck on the bot.
    preconditions: []
    deliverables:
      - "Slack app exists in workspace, marked as 'internal-only'"
      - "Bot token + signing secret stored in 1Password"
      - "Manifest committed to repo at slack-app/manifest.yaml"
    acceptance_test:
      shell: "curl -s -X POST https://slack.com/api/auth.test -H 'Authorization: Bearer $SLACK_BOT_TOKEN' | jq -e '.ok == true'"
    sizing: S
    dependencies: []
    parallelisable: true
    assignable_to: [human]
    estimated_hours: 1.5
    status: draft
    runbook_hint: null

  - id: FR-001-T-02
    title: Vector-index HR policy docs
    description: |
      Read every HR policy document under company/hr-policies/ (currently
      14 markdown files). Chunk each at 800-token boundaries with 100-token
      overlap. Embed via OpenAI text-embedding-3-small (or local
      sentence-transformers if running offline). Index to ChromaDB at
      ./data/hr-policies.chromadb. Each chunk's metadata includes:
      source_path, section_header, last_modified. Re-indexing happens on
      every policy doc change via a file-watcher hook.
    preconditions:
      - "ChromaDB installed (pip install chromadb)"
      - "Embedding API key (OpenAI or local model)"
    deliverables:
      - "data/hr-policies.chromadb/ populated"
      - "scripts/reindex.py — idempotent re-index script"
      - "Test: 5 sample queries return relevant chunks"
    acceptance_test:
      shell: "python3 scripts/reindex.py && python3 -c 'import chromadb; c=chromadb.PersistentClient(path=\"data/hr-policies.chromadb\"); print(c.list_collections())' | grep hr-policies"
    sizing: M
    dependencies: []
    parallelisable: true
    assignable_to: [ai-agent]
    agent_profile: "claude-sonnet-4-6, mcp_allowlist: [bash, edit, read, brain.read]"
    estimated_tokens: 6000
    status: draft
    runbook_hint: null

  - id: FR-001-T-03
    title: Implement Q&A handler
    description: |
      Build the Bolt-for-Python event listener that responds to bot mentions
      (@hr-bot) and direct messages. Pipeline: receive event → retrieve top-5
      chunks from ChromaDB by cosine similarity → construct prompt with
      retrieved context + chat history → call Claude Sonnet 4.6 → return
      answer. The answer must include source-document citations. Reject
      questions outside HR scope with "I can only answer HR policy
      questions" canned response. Use the `untrusted_content` wrapping
      pattern when passing user input to the model (per AGENTS.md §4.2).
    preconditions:
      - "FR-001-T-02 done (index exists)"
      - "Anthropic API key stored in env"
    deliverables:
      - "src/handler.py — event handler module"
      - "src/prompt_builder.py — prompt construction"
      - "Tests: 5 in-scope questions return answers + citations; 3 out-of-scope return canned reject"
    acceptance_test:
      shell: "pytest tests/test_handler.py -v"
    sizing: M
    dependencies: [FR-001-T-02]
    parallelisable: false
    assignable_to: [ai-agent, human]
    agent_profile: "claude-sonnet-4-6, mcp_allowlist: [bash, edit, read, brain.read]"
    estimated_tokens: 14000
    estimated_hours: 4.0
    status: draft
    runbook_hint: null

  - id: FR-001-T-04
    title: Add audit logger + rate limiter
    description: |
      Every Q&A handled by the bot must be logged to the private
      #hr-bot-audit Slack channel (created during T-01). Log shape: user
      ID + truncated question + answer-source-docs (titles only, no body)
      + response latency + timestamp. Rate-limit at 100 requests/hour
      per user using a Redis-backed sliding window (or in-memory if Redis
      unavailable; flag for review). When a user exceeds limit, respond
      with "Rate limit hit; try again in <Nmin>" and don't call the LLM.
    preconditions:
      - "FR-001-T-03 done"
      - "#hr-bot-audit Slack channel exists (auto-created by T-01)"
    deliverables:
      - "src/audit.py — Slack channel logger"
      - "src/ratelimit.py — sliding-window limiter"
      - "Tests: limiter blocks 101st request inside an hour; audit channel receives one message per Q&A"
    acceptance_test:
      shell: "pytest tests/test_audit.py tests/test_ratelimit.py -v"
    sizing: M
    dependencies: [FR-001-T-03]
    parallelisable: false
    assignable_to: [ai-agent]
    agent_profile: "claude-sonnet-4-6, mcp_allowlist: [bash, edit, read]"
    estimated_tokens: 8000
    status: draft
    runbook_hint: null

  - id: FR-001-T-05
    title: Build test corpus + run accuracy eval
    description: |
      Curate 30 representative HR policy questions covering: parental leave,
      stock options, sick leave, expense policy, remote work, conflict of
      interest, IP assignment, public-statement policy, harassment reporting,
      annual review. For each, write the expected answer (≤ 100 words) + the
      source policy doc. Run the bot against all 30; score correctness as
      "correct" if answer matches expected on the key facts AND cites the
      right source doc. Acceptance: ≥ 24/30 correct (80%).
    preconditions:
      - "FR-001-T-03 done"
    deliverables:
      - "tests/corpus/30-questions.yaml — questions + expected answers + source docs"
      - "scripts/eval_accuracy.py — runs corpus, scores, prints report"
      - "Eval report: ≥ 24/30 correct"
    acceptance_test:
      shell: "python3 scripts/eval_accuracy.py | grep -E 'PASS|24/30|25/30|26/30|27/30|28/30|29/30|30/30'"
    sizing: M
    dependencies: [FR-001-T-04]
    parallelisable: true
    assignable_to: [human]
    estimated_hours: 3.0
    status: draft
    runbook_hint: null

  - id: FR-001-T-06
    title: Deploy + production smoke test
    description: |
      Deploy the bot to fly.io (or Render; pick during sprint planning).
      Set env vars via the host's secret manager. Run the production
      smoke test: bot responds to a known question within 2s and the
      answer matches the dev environment. Document deployment in
      docs/deployment.md.
    preconditions:
      - "FR-001-T-05 done (passing eval)"
      - "Cloud account exists with billing"
    deliverables:
      - "Bot deployed; URL shareable to CyberSkill team"
      - "docs/deployment.md — runbook"
      - "Smoke test passes: known-question → correct answer in < 2s"
    acceptance_test:
      shell: "bash scripts/prod-smoke-test.sh"
    sizing: S
    dependencies: [FR-001-T-05]
    parallelisable: false
    assignable_to: [human]
    estimated_hours: 2.0
    status: draft
    runbook_hint: null
---

# FR-001 — Slack HR-policy bot MVP

## Problem statement

CyberSkill has 10 employees and a growing HR policy library (14 documents today, projected 30+ within 12 months). New employees ask the same questions (parental leave, stock options, remote work) repeatedly via DM to Stephen. This consumes ~30 min/week of founder time and delivers inconsistent answers depending on whether Stephen is freshly familiar with a policy.

A Slack bot that answers HR-policy questions with citations + audit trail solves all three problems: consistency (same answer every time), latency (instant vs waiting for a human), and observability (audit channel shows what's being asked, surfacing policy gaps).

## Users

- **Primary**: new employees (week 1-4) who don't know the policy library yet
- **Secondary**: existing employees who need a quick lookup
- **Tertiary**: Stephen (founder) — the bot becomes a self-service replacement for ~30 min/week of his time

## Success metrics

- **Accuracy**: ≥ 24/30 test questions answered correctly (80%)
- **Latency**: p95 response time < 2s
- **Adoption**: ≥ 5/10 employees ask the bot ≥ 1 question/week within 4 weeks of launch
- **Founder-time reclaim**: ≤ 5 min/week of HR-question DMs to Stephen (down from ~30)

## Scope

In scope:
- Q&A over the 14-document policy library
- Slack-only delivery (no web UI, no email)
- English-only (Vietnamese in v2)
- Citations to source docs in every answer

Out of scope:
- Policy edits via bot (read-only)
- Multi-workspace deployment (CyberSkill workspace only)
- Anonymous mode (every Q is logged with the user's ID)

## Risks

- **R1 — Hallucination on edge-case policies.** Mitigation: every answer must cite a source doc; users see citations and can verify.
- **R2 — Slack API rate-limit on heavy use day.** Mitigation: per-user rate limit (FR-001-T-04) + audit log to surface spikes.
- **R3 — Policy doc drift.** Mitigation: re-index on every doc change (FR-001-T-02 file-watcher).

## EU AI Act classification

Risk class: **limited**. The bot processes HR-related inputs (could touch protected-class topics) but does not make HR decisions. Output is informational only. Citation requirement + audit log meet the §16 transparency requirement for limited-risk systems.

## Total estimated effort

- Human: 10.5 hours (T-01 + T-05 + T-06)
- AI agent: 28,000 tokens (T-02 + T-03 + T-04)
- Estimated calendar: 4 weeks (single-developer pace; one focused day per task)

## Tasks

_(see `tasks:` frontmatter list above. Each task is comprehensive + has a runnable acceptance test + is assignable.)_
