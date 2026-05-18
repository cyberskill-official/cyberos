# FR Authoring Discipline — CyberOS

> **Co-located with the auditor that enforces it.** This file lives next to the `feature-request-audit` skill (`modules/skill/feature-request-audit/`) because every rule below is checked by `audit_rubric@2.0`. The discipline doc and the rubric ship together — if you change one, you change the other.
>
> Authored FRs live at `cyberos/docs/feature-requests/{module}/FR-{MOD}-{NNN}-{slug}.md` with sibling `*.audit.md`. This file is the operator-side companion to the skill-side `RUBRIC.md`.

**Source of truth.** This file is normative for every Feature Request in `cyberos/docs/feature-requests/`. It supersedes any prior ad-hoc patterns.

**Created:** 2026-05-16 after a session that wrote 41 FRs across the priority modules (BRAIN, SKILL, PROJ, CHAT) and codified the lessons learned. **Absorbed into the `feature-request-audit` skill on 2026-05-18** — was previously at `cyberos/docs/feature-requests/AUTHORING.md`. Every rule below maps to at least one rework moment that cost ≥ 15 minutes to identify and fix.

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** in this document are to be interpreted as described in BCP 14 (RFC 2119, RFC 8174) when, and only when, they appear in all capitals.

---

## §0 — The Master Rule

> **After creating one FR, loop audit rounds on it until it reaches *perfect* — before starting the next FR.**

This is the single load-bearing discipline. Everything else in this document is subordinate to it.

### What "perfect" means

Perfect = **highly detailed** AND **perfectly matched to core requirements** AND **complete** AND **no truncation**.

- **Highly detailed**: every architectural decision is named, every contract surface is enumerated, every failure mode is listed.
- **Perfectly matched to core requirements**: the spec covers what the FR is *for* — no scope creep, no scope under-coverage. The §1 normative clauses fully express the contract that downstream FRs and engineers depend on.
- **Complete**: all 11 sections present and substantive. No `(elided)`, no `(see other FR)` cross-references that hide the contract.
- **No truncation**: no "summary form," no "compact form due to context budget," no "abridged for brevity," no "inlined into shorter prose." If the author runs into a budget limit, the right action is to **stop, save state, and resume later** — never to ship a truncated FR.

### The Loop

1. **First-pass author** the FR per the 11-section template (§3 below).
2. **Author the audit file** at `<spec-stem>.audit.md` — find at least 6 ISS-xxx findings; score the spec honestly.
3. **If `score_post_revision < 10/10`**: revise the FR addressing every finding.
4. **Re-audit** the revised spec.
5. **Repeat** steps 3–4 until `score_post_revision: 10/10`.
6. **Only then** start the next FR.

### Why this rule first

- **Drift compounds.** A spec with one ambiguity invites a second; downstream FRs that depend on it inherit the ambiguity.
- **Re-entry cost.** Returning to a half-spec'd FR weeks later costs 3× the time of finishing it now — the author has lost the mental model.
- **Audit trail integrity.** Every accepted FR claims `score_post_revision: 10/10`. If some accepted FRs are quietly 8/10 (truncated, summary-form), the score loses its meaning.
- **Reviewer confidence.** The reciprocal-spec promise is "10/10 means it shipped to spec." Sliding the bar breaks that promise.

### How to apply

When tempted to ship a compact FR:

| Temptation | What to do instead |
|---|---|
| "Context budget is tight" | Pause; save state; resume in a fresh session. Don't truncate. |
| "This is a small FR" | If small, then ≤ 300 lines spec is fine AS LONG AS it's complete (all 11 sections present, each meaningful). The size cap isn't the issue — truncation is. |
| "I've established the pattern already; this FR can lean on it" | Use cross-FR primitives via §7 dependencies, but the FR's own §1–§11 must still be self-contained. A reader should not need to open the dependency FR to understand THIS FR's contract. |
| "I'm running 12 FRs in this session; I'll come back and polish" | The rework is 3× more expensive later. Loop to 10/10 NOW. |

### Exceptions

There are **two** sanctioned exceptions to the size/depth target. Both must be explicit in the FR title AND the audit file:

1. **Stub FRs.** An FR whose explicit purpose is to reserve an OCI tag / skill ID / API namespace for a later phase. The stub MUST fully spec the stub contract (the no-op behaviour, the audit-row emission, the "DeferredToP<n>" outcome). Acceptable ≤ 300 lines. Examples: `FR-SKILL-106` (brain-sync@1 stub for P2), `FR-SKILL-107` (synthesis-author@1 P3 reservation).
2. **Pure-infrastructure / Terraform / config FRs.** Where the contract surface is small (resource provisioning, single Dockerfile, single workflow). Acceptable ≤ 400 lines. Example: `FR-CHAT-001` (Mattermost fork pinning).

Neither exception authorises *truncation* — both still require all 11 sections, just at a smaller-but-complete scale.

---

## §1 — Mandatory FR template (11 sections)

Every FR file MUST contain these 11 sections, in order, with the canonical headings:

### §0 — Frontmatter

```yaml
---
id: FR-<MODULE>-<NUMBER>
title: "<one-line subject, ≤ 120 chars>"
module: <AI | AUTH | BRAIN | CHAT | DOCS | OBS | PROJ | SKILL | ...>
priority: <MUST | SHOULD | COULD | MAY>
status: <draft | accepted | building | shipped | deferred | rejected | superseded>
verify: <T | I | A | D>            # T=test, I=inspection, A=analysis, D=demonstration
phase: <P0 | P1 | P2 | P3>
milestone: <P<n> · slice <m>>
slice: <integer>
owner: <person name>
created: <YYYY-MM-DD>
shipped: null
brain_chain_hash: null
related_frs: [FR-..., FR-...]
depends_on: [FR-..., FR-...]
blocks: [FR-..., FR-...]
source_pages:
  - <URL or path>
source_decisions:
  - <DEC-NNN (one-line description)>
language: <e.g. rust 1.81>
service: <repo path>
new_files:
  - <path>
modified_files:
  - <path>
allowed_tools:
  - <description>
disallowed_tools:
  - <description>
effort_hours: <integer>
sub_tasks:
  - "<time-grained task>"
risk_if_skipped: "<one paragraph>"
---
```

**Frontmatter rules:**
- Comments MUST be on their own line (never `priority: MUST   # comment`). Trailing comments break YAML parsers.
- `effort_hours` MUST be populated. If unknown, use the closest 2h-grain estimate.
- `depends_on` and `blocks` MUST be reciprocal — see §6.2.
- Any `depends_on:` / `blocks:` entry pointing at a non-existent FR MUST carry `# placeholder — not yet specified` inline.

### §1 — Description (BCP-14 normative)

Numbered list of `MUST` / `SHOULD` / `MAY` clauses. Each clause SHOULD be 2–4 sentences. Together they MUST fully express the contract.

### §2 — Why this design (rationale for humans)

One paragraph per non-obvious design decision, named after the §1 clause it justifies. Format: `**Why <design choice> (§1 #N)?** <rationale>`.

### §3 — API contract

Code blocks: types, traits, schemas, migrations, REST endpoints. Whatever surface the FR introduces. Concrete code, not pseudo-code.

### §4 — Acceptance criteria

Numbered list of testable conditions. Each AC MUST be a single sentence beginning with a bold descriptor: `**Tier 1 hits first** — member-override = true ...`.

### §5 — Verification

Code blocks showing how each AC is verified. Rust tests, Go tests, TypeScript tests, bash scripts.

### §6 — Implementation skeleton

If §3 is complete, this section may simply say `(API contract above is the skeleton.)`. Otherwise expand orchestrator code.

### §7 — Dependencies

Bulleted list of upstream + downstream + cross-module FRs the spec depends on.

### §8 — Example payloads

JSON examples of audit rows, request bodies, response bodies, etc.

### §9 — Open questions

`All resolved.` if none. Otherwise `Deferred:` prefix + each item with slice/phase reference.

### §10 — Failure modes inventory

Table with columns `Failure | Detection | Outcome | Recovery`. **At least 10 rows** for a substantive FR. Cover every architectural decision's failure path.

### §11 — Implementation notes

Bulleted notes: "the why behind the how" — tradeoffs that future engineers might second-guess.

### Section terminator

End with `*End of FR-<MODULE>-<NUMBER>.*` on its own line.

---

## §2 — Mandatory audit-file template

Every spec MUST have a matching audit at `<spec-stem>.audit.md`. Structure:

```markdown
---
fr_id: FR-<MODULE>-<NUMBER>
audited: <YYYY-MM-DD>
verdict: PASS (after revision)
score_pre_revision: <X/10>
score_post_expansion: <Y/10>
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

## §1 — Verdict summary

<one paragraph: lines, §1 clause count, AC count, failure-mode count, test count>

## §2 — Findings (all resolved)

### ISS-001 — <one-line concern>
<explanation>. Resolved: <fix reference>; AC #N.

### ISS-002 — <one-line concern>
<explanation>. Resolved: <fix reference>; AC #N.

[... at least 6 ISS entries ...]

## §3 — Resolution

All 6 mechanical concerns addressed. **Score = 10/10.**

---

*End of FR-<MODULE>-<NUMBER> audit.*
```

**Audit rules:**
- `score_post_revision: 10/10` is the only acceptable shipping score.
- Below-6-ISS audits are a red flag — author didn't pressure-test the spec.
- Every ISS finding MUST cite the resolution location (`§1 #N`, `§3`, `AC #N`).
- The audit lives + dies with the spec; never delete an audit when superseding a spec.

---

## §3 — The 40 sub-rules

These are rules the master rule (§0) tends to surface naturally if followed. They are listed here as a checklist so they don't have to be rediscovered each session.

### §3.1 — Frontmatter rules (MUST)

1. **Use `Uuid::nil()`, not numeric `0`,** when referring to the root tenant. The literal `0` is invalid because `tenant_id` is `UUID` everywhere; the nil-UUID `00000000-0000-0000-0000-000000000000` is the canonical convention. Use it in prose AND code.
2. **`depends_on` and `blocks` MUST be reciprocal.** If FR-X has `depends_on: [FR-Y]`, FR-Y MUST have `FR-X` in `blocks` (and vice-versa). Validate via a post-authoring sweep against every other FR.
3. **Mark placeholder FRs explicitly.** Any `depends_on:` or `blocks:` entry pointing to an FR that doesn't yet exist MUST carry an inline comment `# placeholder — not yet specified`.
4. **`status` field MUST be one of** `draft | accepted | building | shipped | deferred | rejected | superseded`. No other values.
5. **`effort_hours` MUST be populated.** If unknown, use the closest 2h-grain estimate; never leave blank.

### §3.2 — Audit-row rules (MUST)

6. **Audit-row kinds MUST match `^[a-z][a-z0-9_]*\.[a-z][a-z0-9_]*$`** — exactly one `.` separating module and event_kind. Examples: `ai.precheck`, `brain.sync_row_filtered`, `skill.invoked_started`, `chat.message`. Anti-pattern: `cli.policy_updated` (no module prefix → drift).
7. **Audit-row kinds MUST be namespaced by the OWNING module.** A skill's audit row is `skill.*`, not `ai.skill_*`. A CHAT-emitted row is `chat.*`. Cross-module rows (e.g. AI Gateway emitting `auth.*`) are forbidden; rows belong to one module each.
8. **FR-AI-003 closed-set list MUST be extended** whenever a new `ai.*` row is introduced. Add a §1 #8 entry citing the originating FR.

### §3.3 — Cross-CLI rules (MUST)

9. **All CyberOS CLIs MUST re-export `cyberos-cli-exit::ExitCode`** (the shared crate). No CLI defines its own numeric scheme. The shared values 0–7 are stable cross-CLI contract; module-specific extensions start at the per-module reserved range (200=AUTH, 300=BRAIN, 400=OBS).
10. **Bash CLI wrappers MUST echo a warning when delegating to a `slice_version=*-stub` skill.** Operators must see "this is a placeholder; full impl ships in P<n>" — never silent no-op exits.

### §3.4 — Schema-shape rules (MUST)

11. **Money MUST be stored as `BIGINT minor`** with currency-aware decimals. Never `FLOAT`/`DOUBLE` — even when "it's just for display." Currency-decimals helper (`Currency::decimals()`) is the conversion source.
12. **Append-only tables MUST `REVOKE UPDATE, DELETE` from `cyberos_app` role.** Append-only is enforced by SQL grants, not by handler code (which can be bypassed).
13. **Tenant-scoped tables MUST have RLS with `USING + WITH CHECK`.** USING alone protects reads; WITH CHECK is required for INSERT/UPDATE protection.
14. **Versioned-by-supersession tables MUST use a partial unique index** like `CREATE UNIQUE INDEX uniq_active_X ON X (tenant_id, ...) WHERE effective_to IS NULL`. Enforces "at most one active row per key" without blocking historical rows.

### §3.5 — CRDT vs LWW rules (MUST)

15. **Rich-text fields MUST be Y.Text (CRDT); scalar fields MUST be LWW** with `<field>_updated_at_ns + <field>_updated_by_subject_id`. Never use Y.Map for a scalar — overhead doesn't justify.
16. **CRDT-bound fields MUST NOT have a direct PATCH endpoint.** The Yjs WebSocket relay is the only write path; the SQL column is a materialised view of the latest snapshot.
17. **LWW tie-break MUST be deterministic** — lexicographic on `subject_id` when timestamps are equal. Never rely on insertion order.

### §3.6 — PII-handling rules (MUST)

18. **PII MUST be scrubbed via the `cyberos-brain-pii` ruleset BEFORE chain commit.** Never depend on downstream redaction.
19. **Logs MUST use the `redact()` helper for sensitive fields.** Never `tracing::info!(?email)` with raw PII; always `tracing::info!(email = %redact_email(email))`.
20. **Audit rows MUST carry redacted forms when the field is PII** (e.g. `mst_redacted: "03******78"`); never the full value.
21. **Tenant-scoped PII allowlists exist** (`pii_allowlist: ["regex", ...]` in `manifest.tenants[].pii_allowlist`); use them for legitimate-exception fields like KYC vendor MSTs.

### §3.7 — W3C trace propagation (MUST)

22. **Every outbound HTTP / RPC / queue write MUST carry W3C `traceparent`.** Read from inbound request OR generate one fresh at the trust boundary.
23. **Audit row payloads MUST include `trace_id`** (32-char lower-hex) so OBS dashboards can correlate.
24. **Format OTel `TraceId` via `{}` (Display) — never `{:?}` (Debug).** Debug yields `TraceId(0af7…)`; Display yields the 32-char hex W3C form.

### §3.8 — Audit-before-action (MUST)

25. **Destructive operations MUST emit a BRAIN row BEFORE applying** ("audit-before-action"). Combine with a Postgres transaction so DB write + BRAIN emit are atomic — rollback on either failure.
26. **Pair-write history events** (e.g. `*_started` + `*_completed`) — operators tracing crashes need both bookends. Started without Completed = crash signal.

### §3.9 — Determinism (MUST)

27. **Every catalogue / report-generator output MUST be deterministic.** No `Date.now()`, no random IDs, no hash-map iteration without sorting. Two consecutive runs on the same input MUST produce byte-identical output.
28. **Snapshot files MUST sort by stable key** (e.g. realpath, FR-ID) before iteration.

### §3.10 — Verification rules (MUST)

29. **Every FR MUST have at least one failure-mode row per architectural decision.** Empty §10 is a sign of insufficient design pressure.
30. **Tests MUST assert failure paths explicitly** — not just happy paths. Each `MUST NOT` in §1 corresponds to a negative test in §5.
31. **CI gates that depend on data fixtures** (e.g. PII-recall, VN-search-recall) MUST commit the fixture corpus with the FR, not "we'll generate it later."

### §3.11 — Documentation discipline (SHOULD)

32. **§2 (Why) MUST give the rationale for non-obvious design choices, not just restate §1.** Future readers need the WHY to make edge-case judgement calls.
33. **§9 (Open questions) SHOULD list deferred work explicitly** rather than implying it via `slice 4+`. Use `Deferred:` prefix + slice/phase reference.
34. **§11 (Implementation notes) is the home for "the why behind the how"** — tradeoffs in the implementation that future engineers might second-guess.

### §3.12 — Audit-file rules (MUST)

35. **Every spec MUST have a matching audit file** at `<spec-stem>.audit.md`. The catalog renderer / coherence sweeper depends on the pair.
36. **Every audit file MUST list at least 6 ISS-xxx findings.** Below 6 = author didn't pressure-test the spec enough.
37. **`score_post_revision: 10/10` is the only acceptable shipping score.** Lower scores require explicit operator approval before status transition.

### §3.13 — Frontmatter-comment hygiene (NICE-TO-FIX)

38. **Avoid trailing `#` comments on frontmatter value lines.** Use standalone comment lines above the field instead. Trailing comments break YAML parsers (observed in early FR-AI-001..005 where `priority: MUST   # MUST | SHOULD | COULD | MAY` polluted parsed value).

### §3.14 — Spec-depth calibration (NICE-TO-FIX)

39. **Target 500–700 lines per substantive FR.** Below 300 (excluding sanctioned stubs/infra per §0 exceptions) suggests under-specification; above 1 000 suggests prose padding that obscures the spec.
40. **Stub FRs (status: draft, P2/P3 reservation) MAY be ≤ 300 lines BUT MUST clearly say** "this is a scaffold; full impl in P<n> via FR-<x>" in the title + §1 #1.

---

## §4 — Coherence-sweep checklist

Run **before every bulk-accept**, ideally as a CI gate:

- [ ] depends_on/blocks reciprocity (every edge in both directions)
- [ ] audit-row namespace consistency (`<module>.<event_kind>` regex)
- [ ] ExitCode shared-crate refs (no inline enums per CLI)
- [ ] FR-AI-003 closed-set up-to-date with all `ai.*` kinds
- [ ] All audit files have `score_post_revision: 10/10`
- [ ] All `effort_hours` populated
- [ ] No FR < 300 lines unless explicitly stub/infra per §0 exceptions
- [ ] No FR > 1 000 lines that isn't justified by genuine surface complexity
- [ ] No trailing `#` comments on frontmatter value lines
- [ ] Every dangling FR reference has `# placeholder` annotation
- [ ] Cross-FR primitives use canonical names (Uuid::nil, sync_class, etc.)

---

## §5 — How to use this document

- **Before writing a new FR:** read §0 (Master Rule) and §1 (template). The rest is a checklist for self-audit.
- **When auditing an FR:** the §3 sub-rules are the categories of findings to look for.
- **When reviewing a PR that adds an FR:** confirm §0 was followed — was there an audit-loop until 10/10?
- **When discovering a new anti-pattern:** add it to §3 with a one-line origin reference (which FR's mistake taught it).

---

## §6 — Versioning + amendment

This document follows the same precedence rule as `AGENTS.md` §0: explicit user instructions in chat take priority. Changes to this document MUST be made via PR with `legal-reviewed` label or explicit operator approval, since downstream automation (catalog renderer, coherence sweep) depends on the conventions.

---

## §7 — Session continuation policy (autonomous march)

**Added 2026-05-17 by explicit operator approval.**

When the operator says "continue", "march", or any equivalent open-ended go-ahead, the FR-authoring agent **MUST** keep draining the topological-order frontier autonomously and **MUST NOT** stop between FRs to ask "should I keep going?" The agent stops only when one of these conditions fires:

1. **Decision required.** A genuine design choice surfaces that the operator alone can resolve — e.g., the next FR's scope is ambiguous in the BACKLOG, a normative DEC entry would commit the company to a course not previously chosen, or a coherence error implies a backlog-level priority swap. In that case stop, summarise the decision, and present 2–4 options via `AskUserQuestion`.
2. **Session-limit warning.** The harness signals approaching context exhaustion (system reminder about token budget, or the agent observes the working set creeping toward the context window). In that case stop after the current FR's audit-loop + coherence patch reach a clean state, then emit the §14 block + a "resume point" pointer naming the next-ready FR.
3. **Coherence sweep fails post-patch.** If `coherence_check.py` reports errors that mechanical reciprocity edits can't resolve (e.g., a true cycle in the dependency graph), stop and surface the dependency conflict.
4. **Audit cannot reach 10/10 in three loops.** If three iterations of audit→revise→re-audit on a single FR fail to land 10/10 (rare — usually means the FR's scope is genuinely under-specified at the backlog level), stop and ask the operator to clarify scope before continuing.

Routine surprises (a single missing dependency on an upstream FR, a one-off reciprocity gap, a small clarification needed in implementation details) are **NOT** stop conditions — the agent fills the gap inline and continues.

**Per-FR loop the agent runs without prompting:** pick next-ready from frontier → write spec → write audit → loop to 10/10 → run coherence check → patch upstream reciprocity → emit single-line FR-shipped marker → loop back to pick next-ready.

**End-of-march report (when stop condition fires):** a single response covering every FR drained in the session, with §14 block listing every non-BRAIN file change in one consolidated `📁 Files changed:` block.

---

## §8 — Audit-finding pattern library

**Consolidated 2026-05-17 from STRICT_REDO_PROGRESS.md (now deleted).** When auditing an FR, run this checklist before declaring 10/10. Each pattern below has been a real ISS finding on a shipped FR — they are the categories of mechanical concern that the AUTHORING discipline catches.

### §8.1 — Cross-FR / single-source-of-truth concerns

- **§8.1a Single-source-of-truth violations.** When two modules can answer the same question (`Provider::is_zdr()` AND `zdr::is_zdr`), pick one as canonical and remove the other surface. Origin: FR-AI-006 ISS-001.
- **§8.1b §1 SHOULD vs §4 MUST mismatch.** Never have §1 say MAY/SHOULD when §4 asserts MUST. Either scope SHOULDs to a specific slice, move them to the FR that owns the behaviour, or upgrade §1 to MUST. Origin: FR-AI-008 ISS-001.
- **§8.1c Invariants declared in §1 but not enforced in §6.** Every §1 MUST-clause needs §6 enforcement or §4 verification. If §1 #12 says "is_embedding ⇒ output=0", the loader must check it. Origin: FR-AI-007 ISS-002.
- **§8.1d Constant defined but never referenced.** Every documented constant MUST appear in at least one §6 code path; otherwise the SLA it represents isn't enforced. Origin: FR-AI-010 ISS-002.
- **§8.1e Metric-label cardinality drift between §1 and §6.** Every documented label value must have at least one emit site in §6, OR be removed from §1's enumeration. Origin: FR-AI-008 ISS-004.

### §8.2 — Test coverage gaps

- **§8.2a Promised tests not in §5.** Every AC referencing a test type (proptest, property test, integration test) must have an example body in §5 — not just a named tokio test. Origin: FR-AI-006 ISS-002, FR-AI-007 ISS-001.
- **§8.2b Metric assertions promised in ACs but no test body.** Every metric-MUST in §4 needs a `metric_value(name, labels)` helper invocation in §5. State-only checks don't verify the metric emission. Origin: FR-AI-009 ISS-001.
- **§8.2c Aggregate metric hides per-component regression.** When an SLO is "≥X% recall" or "≤Y latency" across N components, the test MUST assert per-component AND aggregate, not just aggregate. Origin: FR-AI-012 ISS-004.
- **§8.2d Absence claims need lints.** When §1 claims ABSENCE ("no network calls", "no persistence", "no DB"), the FR must include an AST/grep-based CI lint that enforces the absence at PR time. Origin: FR-AI-012 ISS-002.

### §8.3 — Concurrency + state-transition correctness

- **§8.3a State transitions not CAS-guarded → emit_transition fires twice under race.** Any "MUST emit once" transition needs a CAS that gates the emit on CAS-winner status. Origin: FR-AI-009 ISS-002.
- **§8.3b Registration function not idempotent → silent duplicate registration.** Any "register-X-at-startup" function needs a guard global + WARN-on-double-call + `reset_for_tests()` cfg-gated reset. Origin: FR-AI-012 ISS-003, FR-AI-009 ISS-004.
- **§8.3c `init` swallowing double-call errors via `.ok()` breaks test isolation.** Surface programmer errors with `.expect()` AND provide a `reset_for_tests()` cfg-gated function for legitimate test re-init. Origin: FR-AI-009 ISS-004.
- **§8.3d Per-call String allocation on the hot path contradicts <100ns claim.** When a §1 latency MUST is "<100ns single atomic load", the lookup key MUST use `Borrow`-based zero-alloc lookup, not owned-key construction. Origin: FR-AI-009 ISS-003.

### §8.4 — Stream / async / cleanup hygiene

- **§8.4a `let _ = tx.send().await` swallows disconnect on terminal events.** In mpsc-based stream pipelines, EVERY send needs an `.is_err()` branch that propagates disconnect — silent swallow on terminal events misclassifies outcome. Origin: FR-AI-010 ISS-003.
- **§8.4b `Drop` impl using `Handle::try_current()` fails silently during shutdown.** When Drop tries to async-spawn, branch on runtime availability — log loudly + emit OBS counter when unavailable so cleanup-job dependence is visible to operators. Origin: FR-AI-010 ISS-004.

### §8.5 — PII / security / trust-boundary concerns

- **§8.5a Trusting upstream sort order without defensive re-sort.** When correctness depends on a property in another module/process, re-assert the property defensively. Origin: FR-AI-011 ISS-002.
- **§8.5b Denylist sanitizer for error-message PII leak.** For PII-safety filters, prefer allowlist (known error codes) over denylist (heuristic patterns). Denylists always have edge cases. Origin: FR-AI-011 ISS-003.
- **§8.5c Closed-enum `from_str` returns None silently → PII passthrough.** Every `from_str` mapping a string to a closed enum needs a runtime warn/counter on the unmapped path AND a CI test that asserts coverage. Origin: FR-AI-011 ISS-004.

### §8.6 — Data-shape / parsing fragility

- **§8.6a Metric label fragility from Debug-format.** Using `format!("{:?}", enum)` for OBS labels couples your wire format to Debug output (which Rust may change). Explicit `as_metric_label()` method, never Debug-format an enum to a metric label. Origin: FR-AI-007 ISS-003.
- **§8.6b Path-handling edge cases.** `path.parent()` for bare filenames returns `Some("")` not None. Use explicit match arms, not optimistic `unwrap_or`. Origin: FR-AI-007 ISS-004.
- **§8.6c Header data via string-scraping instead of structured field.** Header semantics belong in a structured field on the error variant; never reverse-parse data out of error messages. Origin: FR-AI-008 ISS-003.

### How to use §8

When writing a `*.audit.md`, walk this checklist. Many findings will not apply to a given FR — that's fine. The point is that the categories themselves are the audit's pressure-test rubric. New patterns surfaced in future audits SHOULD be appended here with origin reference.

---

*End of AUTHORING_DISCIPLINE.md — version 1.3 — 2026-05-18 (absorbed into `feature-request-audit` skill from former `docs/feature-requests/AUTHORING.md`).*
