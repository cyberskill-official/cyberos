# Pilot batch 2 — proposal (drafted 2026-06-13)

**Status: PROPOSAL — awaiting Stephen's approval. No audit has been run.** This document picks the target, argues why, and stages the run kit. Executing it is a gated audit and needs your explicit go (and a per-loop `Approved:` line, as in batch 1).

## Recommendation

**Primary target: `CyberSkill/cyberos`, scoped to the `services/` Rust workspace.** Framework v1.4.0, runner mode, gated, DEPTH standard, LOOP_BUDGET 2, BENCHMARK_MODE none.

Draft run kit (in this folder, NOT yet in the cyberos worktree):
- `batch-2/cyberos-services.audit-profile.draft.yaml` — the audit identity.
- `batch-2/targets.draft.yaml` — the `--batch` entry.

## Why cyberos, and why scoped to `services/`

Batch 1 was three small/medium JS-heavy repos (kymondongiap = FastAPI-style Python + Vite TS; 3d-periodic-table = Vite/Three.js; mock-exam = Next.js). Two gaps remain for the framework's evidence base:

1. **Stack diversity.** The framework has never been run on a **Rust** codebase, or on a large multi-service monorepo. cyberos is 5,834 tracked files — **310 Rust, 240 Python, 49 SQL, shell** — with a real Cargo workspace under `services/` (auth, ai-gateway, mcp-gateway, chat, email, memory, proj, obs-collector, skill-broker + shared crates) and 62 Rust + 75 Python test files. This is the largest and most different-stack target available; it exercises `cargo build/test/clippy` as RUN_COMMANDS and a new secret/idiom surface (Rust error handling, async, RLS SQL) the validator has never seen.
2. **Dogfooding the absorb target.** Your stated strategy is to absorb code-audit-framework into cyberos. Running the protocol *on* cyberos is the honest first step of that absorption — it tells us whether the protocol survives contact with its future host, and cyberos already keeps `docs/tasks/*.audit.md` files, so an audit artifact is idiomatic there.

**Scoping is mandatory.** 5,834 files is far too large for one audit loop. The proposal scopes Phase 1 to `services/` only (≈501 files, the Rust backend). If even that is too broad for loop 1, the tighter option is **`auth` + `ai-gateway`** — the two highest-security-value services (auth owns RLS, signing keys, tenant idempotency; ai-gateway owns the cost ledger + policy/model gateway).

## Candidates considered (and why not)

| Repo | Files | Stack | Verdict |
|---|---|---|---|
| **cyberos** | 5,834 | Rust + Python + SQL | **PRIMARY** — largest, most different, strategic |
| landing-page | 1,105 | TS/TSX + some Python | Backup, but TS-heavy → not stack-different from batch 1 |
| sale-noti | 437 | TS | TS-heavy; smaller |
| gam | 216 | TSX/TS + a little Rust | TS-heavy |
| issue-hunter | 46 | Python | Too small to be a "larger" stress test |
| styx | 374 | mostly PDF/DOCX, 4 Python | Docs/design-heavy, not a code target; also sensitive (Colizeum review) — excluded |

## Protected areas (confirmed by inspection, confirm before Phase 3)

Found in `services/` and pre-listed in the draft profile:
- `migrations/` — every service's SQL migrations (auth: tenants, RLS, signing keys, idempotency; ai-gateway: cost ledger). Behavior-preserving only.
- `src/cli/json_schemas/` — generated JSON schemas (built by `gen_schema.rs`); don't hand-edit generated output.
- `services/shared/cyberos-types/` — the public contract crate (R3 public API).
- `gen_schema.rs` — the schema generator.

No tracked `.env` / `.pem` / `.key` / secret files were found in `services/` (good). **Before launch:** add cyberos's own service-token prefix(es) to `secret_patterns` so R8 catches a real leak — don't invent a regex; fill from the actual format.

## T-tier plan (TESTING-PROTOCOL.md)

- **T0 preflight** — place the profile, confirm `--run services/` preflights to exactly MISSING-FILE (CONFIG sane), as batch 1 did.
- **T1 single run** — gated runner-mode loop on `services/`; park at the gate (`Approved:` empty) for your review, exactly like batch 1.
- **T2 retro** — score /20 via RETROSPECTIVE.md; file a feedback@1 record.
- **T3 fabrication_check** — sample ≥10 measured values, re-run their commands, expect 0 mismatch (Rust `cargo test` output is a new evidence surface to stress R1's verbatim-output rule).
- **T4 cross-model** — **run it this time.** Batch 1's open gate was that T4 was only ever run after the fact (2026-06-13) and on a single follow-up; the DEPTH-semantics and "baseline-pass ≠ no-findings" candidates need a **2nd** cross-model run to promote. A Rust target is the ideal place to get it: run Claude and Gemini/Antigravity on the same scoped `services/` backlog and diff severity + recall (use `pilot/XMODEL-RUNBOOK.md`).
- **One-CRITIC-cycle cap** — at most one protocol change out of the batch.

## What this batch is designed to teach

- Does the validator hold on **Rust** artifacts (clippy/cargo output in fences, Rust paths in protected-area checks, Rust idioms vs the secret denylist)? Any false positive is a G-fixture candidate; any miss is a B-fixture + FAILURE_LOG candidate (TESTING-PROTOCOL).
- A **2nd cross-model data point** toward the two gated candidates (DEPTH semantics; "no-significant-findings needs more than baseline commands pass").
- Whether the **v1.4.0 below-floor re-evaluation rule** earns its keep on a real multi-loop run (its retro watch item).

## Risks / guardrails

- **Size** — scope to `services/` (or auth+ai-gateway). Do not point `--run` at the cyberos root for a single loop.
- **Active repo** — cyberos had a commit today; coordinate timing so the audit branch doesn't fight in-flight work.
- **Gated** — nothing executes without your `Approved:` line; protected areas pre-listed; clippy `-D warnings` makes the build gate honest.
- **Absorb sensitivity** — this audits the framework's future host; treat findings as input to the absorption plan, not just a client report.

## The one decision for you

Reply with one of:
- **"Approve batch-2 on cyberos/services"** → I place the profile, run T0–T1 gated, and park at the gate for your review.
- **"Approve, but scope to auth+ai-gateway"** → tighter first run.
- **"Hold"** → stays a proposal.
