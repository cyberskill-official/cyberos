# kymondongiap — Loop-2 resolution of Gemini's cross-model Highs (2026-06-13)

Follow-up to `pilot/XMODEL-RESULTS.md`. Gemini (Antigravity/Gemini-3-Pro), running as "Loop 2" on Claude's post-fix tree, raised 3 Highs. Status after this session (all changes LOCAL/UNPUSHED in the kymondongiap worktree, not deployed):

| Gemini High | Status | Where |
|---|---|---|
| L2-T2 Security — CORS `*.vercel.app` wildcard + credentials | **FIXED earlier** — explicit allowlist | commit `b0b0eaf` (already on main) |
| L2-T3 Maintainability — failing linters (24 frontend, 29 ruff) | **MOSTLY FIXED** — see below | commit `d106ea4` (local) |
| L2-T1 Correctness — `HallucinationGuardrail._term_to_id` 6-term stub | **DEFERRED** (ready-to-apply fix documented) | not changed |

## L2-T3 linters — commit `d106ea4` (local, unpushed)
- **Frontend: 24 → 0.** All 24 were `prettier/prettier` formatting (whitespace, wrapping, trailing commas). `eslint --fix` cleared them across 7 files. `tsc -b` exit 0. Behavior unchanged.
- **Backend: 29 → 10.** `ruff --fix` cleared 19 (14 `I001` unsorted-imports + 5 `F401` unused-imports) across 10 files. Backend suite still **248 passed / 1 skipped**. The remaining **10 are `E402`** (module import not at top) — deliberate mid-file / late imports in db.py, deps.py, public.py (×5), hr.py, negotiation.py, risk.py. Left for Stephen: they are low-severity *style* and resolving them means either hoisting the imports (a 6-file structural rearrange of the author's code) or a ruff per-file-ignore. Not auto-changed — consistent with "apply safe autofixes; defer structural rearranges."

## L2-T1 guardrail stub — DEFERRED, ready-to-apply fix
`backend/src/rag/pipeline/guardrails.py` `HallucinationGuardrail.__init__` hardcodes a 6-term `_term_to_id` map; its own comment says *"In a real impl, we would use the OntologyRegistry to build this."* The real data exists and the fix needs **no fabrication**:

- `rag/ontology/registry.py` loads `rag/ontology/data/*.yaml` into `registry._entries: Dict[id, OntologyEntry]`; each entry has `name_vi` + `id` (e.g. `name_vi="Bính"`, `id="tc-binh"`) — exactly the shape `_term_to_id` needs.
- `rag/pipeline/orchestrator.py:166` constructs `HallucinationGuardrail(context_terms)` and already holds `self._ontology` (the registry).

**Ready-to-apply recipe (for Stephen's go):**
1. Add an optional param: `HallucinationGuardrail.__init__(self, context_terms, term_to_id: dict[str,str] | None = None)`; default to the existing 6-term map so current callers/tests are unaffected.
2. In orchestrator.py, pass `term_to_id={e.name_vi: e.id for e in self._ontology._entries.values()}` (add a small public accessor on the registry rather than touching `_entries` directly).
3. Add a test asserting the guardrail now covers the full ontology (entry_count terms), not just 6.

**Why deferred (not auto-shipped):** this widens what HALL-001 flags as a hallucination — a **user-facing AI behavior change** (more ontology terms not in `context_terms` would fail the guardrail and reject output). It needs review of how `context_terms` is populated so the expanded map doesn't over-reject legitimate output. Same honesty bar as batch-1's mock-exam BLOCK: don't ship a behavior change unattended.

## Also this session (related, local/unpushed)
- **Robustness `0800560`** — graceful DB degradation: a bad/unreachable `DATABASE_URL` no longer crashes the API at startup (it degrades audit-log persistence + surfaces `/health` `db: degraded`). Directly hardens the failure mode that produced Cloud Run rev 00012's `asyncpg.InvalidPasswordError` crash. 5 new tests; suite 248/1. Relates to the framework's open `live-infra verification` candidate (FAILURE_LOG 2026-06-12).

All kymondongiap changes are LOCAL commits (`0800560`, `d106ea4`) on top of `b0b0eaf` — **not pushed, not deployed.** Prod rev `00013-7rn` (T7 Postgres cutover, this session) is unaffected; shipping these is a future source deploy on Stephen's go.
