# FR-SKILL-111..115 — 100% completion plan

> **Status:** 2026-05-19 · Owner: Stephen Cheng · **5 FRs at 10/10 specs; ~60% implementation complete; ~40% remains.**
> Companion doc to [`ANTHROPIC_GUIDE_DIGEST.md`](ANTHROPIC_GUIDE_DIGEST.md). Both produced from the same Anthropic-Skills-portability investigation 2026-05-19.

## §1 — One-page status snapshot

| FR | Title | Spec | Templates | RUBRIC | Catalog impact | Runtime code | CI gate |
|---|---|:-:|:-:|:-:|:-:|:-:|:-:|
| **111** | Description trigger enrichment | ✅ | ✅ | ✅ SKB-020..023 | ✅ 3 exemplars; ⚠ 101 to lazy-backfill | ❌ Rust validator deferred (blocked on FR-103) | ❌ broker not built |
| **112** | `acceptance/TRIGGER_TESTS.md` | ✅ | ✅ | ✅ SKB-050..057 | ✅ 3 exemplars; ⚠ 101 to lazy-backfill | ✅ `cuo.trigger_tests` + 18 tests | ⚠ runs via pytest; not gated yet |
| **113** | XML-free frontmatter | ✅ | ✅ | ✅ SKB-040..042 | ✅ 209 files swept; verified parse | ❌ Rust validator deferred (blocked on FR-103) | ❌ broker not built |
| **114** | BASELINE.md at v1.0 | ✅ | ✅ | ✅ SKB-060..066 | n/a (no v1.0 skill in modules/skill yet) | ✅ `cuo.baseline` + 11 tests | ⚠ runs via pytest; not gated yet |
| **115** | Stale-placeholder sweep | ✅ | n/a (separate tooling) | ⚠ SKB-030 spec'd, not added | ❌ 134 files still leaky | ❌ detect.py + suggest.py not built | ❌ |

Legend: ✅ = shipped this session · ⚠ = partial / queued · ❌ = not started.

## §2 — Critical path to 100%

Ordered by dependency + impact. Cumulative effort: **~38-46 hours**. Splits cleanly into 3 focused sessions.

### Session 1 — FR-SKILL-115 implementation (≈ 16 hours)

Why first: it's the largest remaining piece + has no blocker. Closes the 134-file portability gap so Phase-B transpilers (when FR-103 lands) can ship the catalog whole.

1. **Build `tools/sweep-placeholders/detect.py`** (1.0h) per FR-115 §3 — scanner walks `modules/skill/**/SKILL.md`, parses frontmatter, identifies stale `<placeholder>` tokens.
2. **Build `tools/sweep-placeholders/suggest.py`** (1.5h) per FR-115 §3 — per-skill suggestion engine reading body CONTRACT_ECHO, MANIFEST_SCHEMA, MODULE.md persona entry.
3. **Build `modules/cuo/cuo/placeholder_check.py` + tests** (2.0h) — runtime validator + 4-6 pytest functions.
4. **Add SKB-030 to `SKILL_BUNDLE_RUBRIC.md`** (0.5h) — rule statement + severity scheme.
5. **Add §3.13 rule 38f to feature-request-audit skill** (0.5h) — discipline-doc entry.
6. **Generate `tools/sweep-placeholders/report-2026-05-XX.md`** (0.5h) — run detect + suggest across catalog, produce review-ready report.
7. **Operator reviews report + approves substitutions** (1.0h) — Stephen reads + edits 134 entries (~25s per entry; suggest.py speeds this up).
8. **Apply substitutions in persona-grouped batches** (8.0h) — 6-10 batches by persona; commit each with operator-attested rationale.
9. **Build verify.py + run post-sweep** (0.5h) — assert zero residual placeholders + YAML parse + wrap_in_marker invariant + body SHA256 unchanged.
10. **CHANGELOG.md `[SKILL]` bump v0.2.5 → v0.2.6** (0.5h) — narrative entry citing FR-115.

**Output:** registry v0.2.6 with zero stale placeholders. All 134 production skills load cleanly on Anthropic-host transpilers.

### Session 2 — Lazy-backfill cohort + CI gate wiring (≈ 8-10 hours)

Why second: closes the FR-111 + FR-112 "lazy backfill" tail. Today only 3 exemplars carry enriched descriptions + TRIGGER_TESTS.md; the other 101 skills (104 - 3 exemplars) inherit warnings, not errors, until their next fine-tune.

1. **Decide cohort triage policy** (0.5h) — pick which 101 skills get pro-active backfill vs which wait for natural fine-tune. Recommendation: backfill P0+P1 personas now (~50 skills), let P2+ wait (~50 skills).
2. **Backfill 50 P0/P1 persona skills** (5.0h) — for each: enrich description per SKB-020..023 + author `acceptance/TRIGGER_TESTS.md` per SKB-050..057. Use the 3 exemplars as templates. ~6 min per skill × 50 = 5h.
3. **Wire CI gate** (1.5h) — extend `modules/cuo/tests/test_smoke.py` to call `cuo.placeholder_check.run_all()` + `cuo.trigger_tests.run_all()` + `cuo.baseline.run_all()` as auto-discovery tests; fail PR on any production skill violating SKB-020..023, SKB-050..057, SKB-060..066, SKB-030.
4. **Add `cuo-skill-check` CLI subcommand** (1.0h) — `cuo skill-check <path>` runs all 4 validators for one skill; `cuo skill-check --catalog` runs across all skills.
5. **Update README.md Part 13 validation pyramid diagram** (0.5h) — show Layer 1.5 (triggering) + Layer 4 (baseline) + new Layer 1.6 (placeholder-free) explicitly.

**Output:** 53/104 production skills are gold-standard (3 exemplars + 50 P0/P1 backfills); CI fails on any new skill that violates the 4 rule families.

### Session 3 — Rust broker scaffold + transpiler smoke (≈ 14-20 hours)

Why third: this completes FR-111 + FR-113 (Rust validators) and unblocks Phase B (FR-103 host transpilers). Largest scope variance because depends on how much of FR-SKILL-103 we want to ship simultaneously.

**Minimum scope (14 hours):**
1. **Scaffold `services/skill-broker/` cargo crate** (2.0h) — Cargo.toml + src/lib.rs + frontmatter module shell. Depends on existing `services/shared/cyberos-cli-exit` + `services/shared/cyberos-types` crates.
2. **Implement FR-SKILL-103 §3 schema.rs + parser.rs + validators.rs** (4.0h) — minimum to make the broker compile end-to-end + load a SKILL.md frontmatter.
3. **Implement FR-SKILL-111 §3 `description_validator.rs` + tests** (2.0h) — 11 unit tests per FR-111 §5.
4. **Implement FR-SKILL-113 §3 `marker_validator.rs` + tests** (1.5h) — 5 unit tests per FR-113 §5.
5. **Implement `skill.schema.json` JSONSchema mirror** (1.5h) — used by editor LSPs + CI gates.
6. **Implement `cyberos skill validate` CLI** (1.5h) — exit codes 0/1/6; --json output flag.
7. **Integration test against the 3 exemplars** (1.5h) — assert broker loads them cleanly; smoke-test against migrate-to-newer-mermaid-version.

**Extended scope (extra +6 hours for full Phase-B Anthropic transpiler smoke):**
8. **Scaffold `services/skill-broker/src/transpilers/anthropic.rs`** (3.0h) — emits Anthropic-flat-SKILL.md from CCSM.
9. **Integration test: transpile + validate against vanilla Anthropic loader** (3.0h) — confirm `feature-request-author` round-trips correctly through transpiler.

**Output:** Rust broker compiles + runs FR-111 + FR-113 validators. Phase-B transpilation smoke-tested for the 3 exemplars.

## §3 — Dependency graph

```
                                       ┌─── FR-SKILL-113 (impl shipped this session)
                                       │
FR-SKILL-103 ── (broker scaffold) ─────┼─── FR-SKILL-111 Rust validator (deferred)
  (parent;                             │
   pending)                            ├─── FR-SKILL-113 Rust validator (deferred)
                                       │
                                       └─── FR-SKILL-114 broker partner-connector check (deferred)

FR-SKILL-111 (description format)  ──── ✅ Templates + RUBRIC + 3 exemplars shipped
                                   └─── ⚠ 101-skill lazy backfill (Session 2)

FR-SKILL-112 (TRIGGER_TESTS.md)    ──── ✅ Templates + RUBRIC + 3 exemplars + Python validator shipped
                                   └─── ⚠ 101-skill lazy backfill (Session 2)

FR-SKILL-113 (XML-free)            ──── ✅ 209-file sweep done; templates + RUBRIC shipped
                                   └─── ❌ Rust validator deferred to FR-103 ship

FR-SKILL-114 (BASELINE.md)         ──── ✅ Template + RUBRIC + Python validator shipped
                                   └─── ⚠ no v1.0 skill exists yet (backfill is no-op today)

FR-SKILL-115 (placeholder sweep)   ──── ✅ Spec at 10/10 (this session)
                                   └─── ❌ Implementation = Session 1 (16h)
```

## §4 — Recommended sprint cut

| Sprint week | Sessions | Scope | Hours |
|---|---|---|---:|
| **W1** | 1 | FR-115 detect + suggest + 134-file sweep + verify | 16 |
| **W2** | 2 | 50-skill backfill cohort + CI gate wiring | 8-10 |
| **W3** | 3 (minimum) | Rust broker scaffold + FR-111/113 validators | 14 |
| **W3 + W4** | 3 (extended) | + Anthropic transpiler smoke-test | +6 |

Total: **~38-46 hours over 3-4 weeks** at half-day-per-day cadence.

## §5 — Out of scope (recorded for future FRs)

The following appeared during this session's investigation but are NOT part of the 111-115 finish line:

- **FR-SKILL-116** (placeholder): OBS-driven candidate trigger-phrase suggestions per FR-112 §9. Mines real user phrasings post-deploy; auto-proposes additions to TRIGGER_TESTS.md as `refinement_proposal` envelopes. Phase P2+.
- **FR-SKILL-117** (placeholder): marker namespace expansion per FR-113 §9. Adds `untrusted_content_strict` etc. for partner-connector skills.
- **FR-SKILL-118** (placeholder): automated baseline re-measurement at 12-month review-due per FR-114 §9.
- **VN-locale parallel `TRIGGER_TESTS.vi.md`** per FR-112 §1 #10 — Vietnamese-locale users' phrasings.
- **Localised descriptions** (`description_localized.<lang>`) per FR-111 §1 #10.
- **Cleanup of the 627 `.fuse_hidden*` files** on Stephen's macOS side (run `find /Users/stephencheng/Projects/CyberSkill/cyberos -name '.fuse_hidden*' -delete` — FUSE-blocked from the sandbox).
- **Wiki mermaid escapes for the 30+ tokens fixed this session** — already shipped via `tools/fix-mermaid-html/escape-placeholders.py`; future authors should run the script after editing any wiki page that adds new mermaid diagrams.

## §6 — Verification

How to confirm we're at 100%:

```bash
# 1. FR-SKILL-111
python3 -m cuo.placeholder_check --catalog modules/skill/ --field description \
  --rule SKB-020,SKB-021,SKB-022,SKB-023 --status-min accepted
# Expected: exit 0; all 104 production skills carry enriched descriptions.

# 2. FR-SKILL-112
python3 -m cuo.trigger_tests --catalog modules/skill/ --status-min accepted
# Expected: exit 0; all 104 production skills carry acceptance/TRIGGER_TESTS.md
# AND the supervisor classifier routes each fixture correctly.

# 3. FR-SKILL-113
grep -rn 'wrap_in:\s*<' modules/skill/ --include='SKILL.md'
# Expected: zero matches (post-sweep; verified this session).
python3 -m cuo.placeholder_check --rule SKB-040 --catalog modules/skill/
# Expected: zero hits in any frontmatter field.

# 4. FR-SKILL-114
find modules/skill -name BASELINE.md | xargs -I {} python3 -m cuo.baseline {}
# Expected: zero failed baselines (today: no v1.0 skill; this is a no-op gate).

# 5. FR-SKILL-115
python3 tools/sweep-placeholders/detect.py
# Expected: exit 0; total_skills_with_hits: 0.

# 6. Cargo broker validates
cd services/skill-broker && cargo test
# Expected: all tests pass (description_validator + marker_validator + integration).

# 7. End-to-end Anthropic transpile smoke
cyberos build modules/skill/feature-request-author/ --target anthropic
# Expected: emits dist/anthropic/feature-request-author/SKILL.md with valid Anthropic frontmatter.
```

When all 7 checks return clean: **FR-SKILL-111..115 are at 100%.**

## §7 — Today's session inventory

What I'm leaving the operator in a clean state for next session:

**Shipped + verified:**
- 5 FRs at 10/10 (111, 112, 113, 114, 115) with audit siblings
- `modules/skill/SKILL_BUNDLE_RUBRIC.md` — new normative rubric with SKB-* namespace
- `modules/skill/_template/{author,audit}/SKILL.md` — registry v0.2.5 form
- `modules/skill/_template/{author,audit}/acceptance/TRIGGER_TESTS.md` + `_template/author/BASELINE.md` — scaffolds
- 209 production SKILL.md files migrated to `wrap_in_marker:` form
- 3 exemplar backfills (feature-request-author + feature-request-audit + product-requirements-document-author)
- `modules/cuo/cuo/trigger_tests.py` + `cuo/baseline.py` + 29 new tests (78 pass + 1 expected skip; no regressions)
- `tools/migrate-wrap-in/migrate.sh` — bash sweep tool (patched after `\s*$` bug discovered)
- `tools/fix-mermaid-html/escape-placeholders.py` — wiki mermaid placeholder escape
- 19 mermaid blocks across 9 wiki pages fixed (Flow 4 + others)
- `services/shared/cyberos-types/src/lib.rs` — `Default` impls added for `TenantId` + `SubjectId` (unblocks CI clippy)
- `ANTHROPIC_GUIDE_DIGEST.md` — comprehensive 470-line findings doc

**Queued for next session(s):**
- 134-file placeholder sweep (FR-115 Session 1)
- 50-skill backfill cohort (FR-111+112 Session 2)
- CI gate wiring (Session 2)
- Rust broker scaffold + Rust validators (FR-103 prerequisite + FR-111+113 deferred code; Session 3)
- BACKLOG.md `### Headline metrics` regeneration (FR count 244 → 245 with 115)
- `.fuse_hidden*` cleanup on macOS side (user-action; one-liner)

---

*End of FR_111_115_COMPLETION_PLAN.md.*
