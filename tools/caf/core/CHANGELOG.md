# AUDIT.md Protocol Changelog

Versioning: **MAJOR.MINOR.PATCH**

- **MAJOR** — restructured phases/rules
- **MINOR** — new rule or vector
- **PATCH** — wording/typo/clarity fix

**Rules of the loop:**

1. **One change per version.** Every released version differs from its predecessor by exactly one deliberate change.
2. **Released versions are immutable.** A change = a new file in `improve/versions/` + an entry here. Never edit a released version in place.
3. **Every change cites a trigger** — a retrospective item, a failure-log row, or an eval regression. No speculative edits.
4. **Every change is gated by the eval harness.** `python3 evals/validate.py --all` must be green before a version is released.

---

## v1.5.0 — 2026-06-14 (improvement campaign 6, cycle 13)

- **MINOR: TARGET HEALTH GATE — the audited target must still pass its own RUN_COMMANDS.** Phase 4 now requires that, after any code change, the auditor runs the target's full RUN_COMMANDS (build / lint / typecheck / test) end-to-end with raw output and a per-command timeout; the loop is not complete while they are red. Phase 5 HANDOFF gains a required `Target health: PASS|FAIL` line — a run may not be declared complete without proving the target still builds/lints/tests. AUDIT.md 169→181 lines (≤200).
- **Trigger (FAILURE_LOG 2026-06-13):** during the 2026-06 fine-tuning, a kymondongiap lint fix scoped to `src/` shipped while the target's own CI (`ruff check .`, which covers `tests/` + E402) stayed RED — because the audit never re-ran the target's RUN_COMMANDS after the change. The artifact-validator cannot run builds, so nothing caught it. This is the maintainer's explicitly-requested mechanism: "strictly verify the target still runs without error after the audit."
- **Two enforcing halves:** (1) executable — new `core/evals/verify-target.sh` reads RUN_COMMANDS from the target's `audit-profile.yaml`/AUDIT.md, runs each with a portable timeout, and exits non-zero on any failure/hang (smoke-tested: PASS→0, FAIL→1, TIMEOUT→1). (2) artifact — new validator check `TARGET-HEALTH-UNVERIFIED` (rules.json `TARGET-HEALTH`): a completed HANDOFF (stop condition cited) lacking the `Target health:` line is flagged. **Version-gated to v1.5.0+** artifacts (older runs judged by their own template, like the Mode/Protocol echoes), so existing fixtures are untouched.
- **Eval:** **40/40 green** (was 38/38; +G13 conformant PASS handoff, +B27 missing-record violation), baseline re-recorded at v1.5.0.

## v1.4.0 — 2026-06-13 (improvement campaign 5, cycle 12)

- **MINOR: Phase 4 re-evaluates below-floor items whose premise changed.** A new Phase 4 bullet requires re-rating, before the stop test, any issue logged below `SEVERITY_FLOOR` whose premise a task completed this loop changed (e.g. a "CORS is Low until auth exists" note, after auth is added); if it now meets the floor it is carried into the next loop's backlog as an OPEN finding, and a stale below-floor severity is never a stop reason. Net protocol size change: +5 lines (one bullet; AUDIT.md 164→169 lines, still ≤200).
- **Trigger:** FAILURE_LOG 2026-06-13 — "below-floor staleness", surfaced by the T4 cross-model diff on kymondongiap: Claude's own below-floor note said CORS "becomes High the moment auth is added", Claude then added auth (T2) but never re-classified CORS; Gemini, auditing fresh, scored it High. **Rule of Three explicitly waived by the maintainer (Stephen, 2026-06-13)** — promoted at 1 observation on his instruction; the protocol normally logs a single observation rather than codifying it. Recorded as a waiver in both FAILURE_LOG and rules.json so the exception is auditable.
- **Coverage (honest):** behavior rule, gold-only — whether an agent actually re-rated a below-floor item is a semantic/recall property, not soundly machine-detectable without false positives, so the validator does NOT flag the omission (same posture as R7's padding note). New rules.json rule `PHASE4-REEVAL`; new fixture `G12-below-floor-reeval` pins the conformant two-loop re-promotion so the pattern is a tested, locked example a future validator change can't wrongly flag. A sound opt-in enforcement (a structured `escalates to <Sev> when <ID> done` clause + check) is proposed for a future cycle.
- **Eval:** **38/38 green** (was 37/37; +G12), baseline re-recorded at v1.4.0. AUDIT.md/baseline otherwise unchanged.

## v1.3.0 — 2026-06-10 (improvement campaign 4, cycle 1)

- **MINOR: The backlog now echoes its protocol version.** Phase 2's "Scope & method" template line opens with `Protocol: <this file's title version>`, making every artifact self-describing. The validator gates template requirements on the stated version (a v1.0.0 artifact is judged by the v1.0.0 template — no Mode echo required; a current artifact omitting the echo is itself nonconformant), which makes mixed-version artifact fleets validate correctly from v1.3.0 forward. Net protocol size change: 0 lines (edit to an existing template line; AUDIT.md remains 164 lines).
- **Trigger:** Architect review 2026-06-10, finding F-5 (version skew) — artifacts don't state which protocol produced them, so the validator assumed the current template and false-positived on older runs. Second observation of the self-describing-artifact family (first: campaign-2's Mode echo, v1.1.0) — promotion bar met. FAILURE_LOG row promoted this cycle.
- **Harness, same cycle:** version-aware `check_template_conformance` (`CURRENT_PROTOCOL` kept in lockstep by check-docs-sync); trap `B24-missing-protocol-echo`; precision sibling `G10-legacy-version-artifact` (stated v1.0.0, no Mode echo → clean). All 30 template-conformant fixtures strengthened to the v1.3.0 Scope line. Evals: **34/34 green**, baseline re-recorded at v1.3.0.
- **Also in campaign 4 (infra, landed before this cycle):** the calibration sprint from the architect review — fence-aware parsing (F-1: quoted tool output neither trips checks nor satisfies the template; G07/B19), the Final-column escape closed (F-2: column shape selects semantics, never disables checks; B20), artifact problems became verdicts (F-3: `MALFORMED-FILE`, never tracebacks; B21), CONFIG-preflight precision (F-4: generics/redirection/`#`-values stay valid; G08), waivers v1 (`docs/AUDIT-WAIVERS.yaml` — audit-trailed, expiring suppressions; G09/B22), `--aggregate` portfolio roll-up, published report schema (`schemas/report.v1.json`, CI-validated), redaction-distance behavior pinned (B23), COMPLIANCE.md control mapping, air-gap property documented.

## v1.2.0 — 2026-06-10 (improvement campaign 3, cycle 1)

- **MINOR: Phase 0 gains a CONFIG preflight.** Three lines: if any CONFIG value still contains `<placeholder>` text, or MODE / DEPTH / BENCHMARK_MODE / SEVERITY_FLOOR is outside its allowed set, the agent STOPs and asks the human instead of improvising CONFIG. Closes the most likely first-run failure mode on a new codebase: a half-filled CONFIG silently "completed" by the agent, which then audits against invented constraints.
- **Trigger:** Structural review 2026-06-10, gap G-D — CONFIG improvisation is the documented norm for agents handed partial configs; nothing in the protocol or the validator checked CONFIG sanity (FAILURE_LOG row "CONFIG improvisation", promoted this cycle). Protocol side of `improve/BLINDSPOTS.md` BS-13.
- **Harness, same cycle (landed as the preceding infra commit):** the validator preflights the target AUDIT.md's CONFIG — `CONFIG-PLACEHOLDER` / `CONFIG-BAD-ENUM` (trap fixture `B17-config-placeholder`) — and auto-loads `PROTECTED_AREAS` into the R3 tripwire (`B18-config-autoprotect`), retiring the `--protected` double entry (gap G-F). Instruction budget: 161 → 164 lines, within the 200-line cap. Evals: **24/24 green**, baseline re-recorded at v1.2.0.
- **Also in this campaign (infra, not protocol changes):** template-conformance meta-tripwire `TEMPLATE-NONCONFORMANT` (BS-12 — non-template output no longer escapes every check; fixtures B15/G05/G06), compound exact-set fixture (B16), precision pack G03/G04 (G:B ratio 2:14 → 6:18), `--report json|sarif` findings export, `evals/scripts/retro-summary.py`, Apache-2.0 LICENSE + NOTICE, CI eval gate (suite + version-sync + baseline sha256 + snapshot immutability), CONTRIBUTING.md, SECURITY.md.

## v1.1.0 — 2026-06-10 (improvement campaign 2, cycle 1)

- **MINOR: The backlog now echoes its MODE.** Phase 2's "Scope & method" template line gains a leading `Mode: <MODE>` field. This makes the gated-mode guarantee post-hoc enforceable: the validator now flags any loop section that declares `Mode: gated` and contains executed tasks (DONE / IN-PROGRESS / BLOCKED) without an `Approved:` line — previously the check fired only when the line already existed, so *omitting it entirely* evaded the gate.
- **Trigger:** Blind-spot review 2026-06-10 (High) — live exploit demonstrated against v1.0.0: a synthetic gated run with executed tasks and zero `Approved:` lines validated "CLEAN — no violations" (`improve/BLINDSPOTS.md` BS-08). Second observation of the family first logged in pre-release cycle 4 (FAILURE_LOG) — promotion bar met.
- **Harness:** `check_approvals` fires on `Mode: gated` OR an `Approved:` line; new trap fixture `B14-gated-missing-approval` (the exploit, verbatim, as a fixture); `G01-clean-run` strengthened to exercise the positive gated path. Net protocol size change: 0 lines. Evals: **16/16 green.**

## v1.0.0 — 2026-06-10

- **Initial public release** under [cyberskill-official/code-audit-framework](https://github.com/cyberskill-official/code-audit-framework). The repository is the product: the AUDIT.md protocol plus the machinery that improves it (the `improve/` loop, the `evals/` regression gate, and the product page).
- **What 1.0.0 contains.** A ~160-line, AI-agnostic audit protocol: one-sentence role, per-project CONFIG block, 8 core rules (evidence-or-nothing, honest targets, protected core, file-is-memory, one-task micro-loop, 3-strike circuit breaker, severity-weighted findings, secret redaction), and a 6-phase state machine with a reachable stop rule.
- **Provenance.** 1.0.0 consolidates an internal pre-release lineage: a 150 KB / 1,898-line monolith prompt (4 documented production runs), a research-backed rewrite, and a 5-cycle self-improvement campaign (2026-06-10) that closed three High-severity letter-vs-intent gaps — row-traceable evidence (each measured metric's output block opens with `$ <verify command>`), durable gated-mode approvals (the `Approved:` artifact), and a single closed metric-status vocabulary across R1 and Phase 5 — each change gated by the fault-injection eval suite (**15/15 green at release**). Internal version identifiers from that lineage (v1.x-era, v2.0.0–v2.1.1) appear in `improve/FAILURE_LOG.md` and `improve/retros/`; the full history is preserved in git log prior to this release.
