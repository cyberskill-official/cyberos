# FM-001 nested-map fork (2026-07-18)

Evidence file resolving the fork left open by TASK-IMP-117's route-back (commit `1aab31bc`). The safe trailing-comment migration (commit `4c02b556`) took the corpus from 2104 trailing-comment findings to 1, but did NOT reach clause 1.6's "corpus FM-001 → 0": **141 specs remain non-conformant**, 140 of them because they carry NESTED-MAP frontmatter under a single key, `build_envelope:`. The operator chose route-back but not the PATH, and the two paths are mutually exclusive:

- **(a) migrate the 140** — fold the nested map to conformant form, IF nested-map frontmatter is a real FM-001 violation the specs should not carry; OR
- **(b) fix task-lint's FM-001** — stop flagging the indentation, IF nested maps are legitimate task@1 frontmatter and the lint is over-strict.

Building the wrong one either "fixes" 140 specs that were never broken, or leaves a real defect in place. This file answers which, with a command behind every claim (the session's standing finding: authors do not check what they originate — so nothing here is recollected, all of it is re-run).

Measured at HEAD `8d779870`. The linter is `tools/install/docs-tools/task-lint.mjs`; the migrator is `tools/install/docs-tools/fm001-migrate.mjs`.

## THE ANSWER (one line)

Nested-map `build_envelope:` frontmatter is a **REAL FM-001 violation** — it is outside the documented task@1 subset, no tool reads it, and task-lint's rejection of nested maps is a **deliberate, documented design boundary, not accidental over-strictness**. Take route **(a): migrate the 140** (by *unwrapping* `build_envelope` to flat top-level keys — the form a sibling `done` spec already uses). Do **not** relax FM-001 (route b). Separately, the 1 apostrophe residual is a bug in the **migrator**, not the lint.

## Re-derived numbers (the commit's claims, re-run — not trusted)

Reproduce:

```
node tools/install/docs-tools/task-lint.mjs --json docs/tasks/ > /tmp/lint.json   # exit 2
node -e 'const f=require("/tmp/lint.json").filter(x=>x.rule_id==="FM-001");
 console.log("non-conformant files:", new Set(f.map(x=>x.file)).size);
 console.log("indented-line findings:", f.filter(x=>x.message.includes("indented line outside")).length);
 console.log("trailing-comment findings:", f.filter(x=>x.message.includes("trailing comment")).length);'
grep -rl "^build_envelope:" docs/tasks/ | wc -l
git show --stat --name-only 4c02b556 | grep -c 'spec.md'
```

| metric | commit says | re-derived | note |
|---|---|---|---|
| non-conformant specs (≥1 FM-001) | 141 | **141** | 140 nested-map + 1 apostrophe (SKILL-104) |
| "indented line outside a block list" findings | 4004 | **4004** | across 140 files |
| specs carrying `build_envelope:` | 140 | **140** | exactly the 140 indented-line files |
| apostrophe residual | 1 @ TASK-SKILL-104:63 | **1 @ TASK-SKILL-104:63** | the only surviving trailing-comment finding |
| specs migrated by `4c02b556` | 497 | **497** | `spec.md` files changed in that commit |

The finding key in `--json` output is **`rule_id`**, not `rule` (an earlier agent grepped `rule`, matched nothing, and concluded 0). Every claim below uses `rule_id`.

CRITICAL: the earlier CO-OCCUR claim was false and is why this only surfaced at scale. The IMP-117 ship verdict asserted trailing + indented + unterminated findings "all CO-OCCUR with trailing and CLEAR TOGETHER." They do not: `4c02b556` removed all trailing comments and 4004 indented findings **remained**. The indented findings are structurally independent — they are nested maps, not a downstream artefact of trailing comments. (`fm001-migrate.mjs:13-18` still asserts the false co-occurrence in its own header.)

## 1. What FM-001 actually is (and whether it models nested maps)

`task-lint.mjs` parses frontmatter with a **strict, hand-written minimal-YAML reader** (`readFrontmatter`, lines 106-195), NOT a general YAML library. Its accept-set is stated in the file header (lines 22-28) and is exactly: `flat key: value`, single/double-quoted strings, inline `[a, b]` lists, block `- item` lists, own-line comments, blank lines. The header then names what it rejects, verbatim: *"Anything beyond that (anchors, aliases, block scalars, **nested maps**, flow maps, trailing comments) is itself an FM-001 finding naming the line: failing loudly beats silently accepting what the rule families never defined."*

So the reader has **no model of nested maps at all**. The mechanism (line 127-130):

```js
if (/^\s/.test(raw)) {
  finding(findings, "error", "FM-001", file, ln, "indented line outside a block list (strict task@1 subset)");
  i++; continue;
}
```

A key with an empty value triggers a look-ahead for block-list items `/^\s+-\s?(.*)$/` only (lines 147-166). A nested map — `build_envelope:` followed by `  language: rust 1.81` — does not match `- item`, so the block-list loop consumes nothing, `build_envelope` is recorded as a null-valued scalar, and each indented child line falls through to line 127 and is flagged. Every indented child = one FM-001 finding. That is the 4004.

(The trace in the task said "around line 153." Line 153 is a *different* FM-001 — a scalar error *inside* a block-list item. The "indented line outside a block list" message is line **128**.)

**Verdict for #1:** FM-001 does not merely fail to anticipate nested maps — it rejects them **by design**, as an explicit, documented boundary. "Fixing the lint" (route b) means overturning a deliberate design decision, not correcting an oversight.

## 2. Is nested-map frontmatter legitimate for task@1?

Every authority for the task@1 shape is **flat**. Nested maps appear nowhere in the contract.

- `tools/install/templates/TASK-TEMPLATE.md` — the template `install.sh:651` hands every new repo, "the FIRST artifact anyone touches." Entirely flat scalars + one inline list (`depends_on: []`). No nested map. Its own header (lines 1-5) documents the FM-001 discipline and says a spec authored from it "is born FM-001-clean." Command: `sed -n '1,39p' tools/install/templates/TASK-TEMPLATE.md`.
- `modules/skill/contracts/task/template.md` — the body skeleton `task-author` uses and `task-audit` validates against. Flat scalars only. Command: `sed -n '1,14p' modules/skill/contracts/task/template.md`.
- `modules/skill/contracts/task/CONTRACT.md` — the task@1 field table (lines 45-60) enumerates every MUST-carry field: `title, author, department, status, priority, created_at, ai_authorship, feature_type, eu_ai_act_risk_class, target_release, client_visible, template`. **All flat scalars. None is a nested map. `build_envelope` is not a task@1 field at all.**

Note the one true nested-map user in-tree: `CONTRACT.md`'s *own* frontmatter carries `escalation_on_breach:` and `determinism:` as nested maps (lines 13-21). But that file is a **contract artefact (`contract_kind: artefact_schema`), not a `task@1` document** — task-lint only lints `template: task@1` specs (`lintFile` stops any other template at FM-004). So the repo *does* use nested-map YAML elsewhere, under a *different* schema, read by a *different* consumer. That is not license for task@1 to.

**Provenance of the 140.** They are not curated task@1 specs. Every one carries markers of the `2026-07-14 schema migration` — e.g. `# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft` (993 FM-112 findings corpus-wide), plus a whole non-task@1 key dialect: `phase, milestone, slice, owner, verify, source_pages, source_decisions, effort_hours, subtasks, risk_if_skipped, memory_chain_hash, related_tasks, blocks`. `build_envelope` is simply the one dialect key that is a nested MAP; the rest are flat/lists and so slip past FM-001. The 140 are an auto-migrated, largely un-re-audited lineage (118 of 140 are `draft`), not a sanctioned convention. Command: `awk '/^---$/{c++} c==1' docs/tasks/ai/TASK-AI-104-vn-provider-integration/spec.md`.

**Verdict for #2:** nested maps are NOT legitimate task@1. Supporting them would be a schema expansion — a MAJOR `contract_version` bump to `task@2` per `CONTRACT.md:88` — not a lint tweak.

## 3. What depends on frontmatter shape (does a nested map break a real consumer, or only FM-001?)

`build_envelope` is read by **nothing**. Command (empty output):

```
grep -rln "build_envelope" tools modules scripts services apps | grep -v docs/tasks
```

Zero references anywhere in code — no producer, no consumer, not even in `dist/`. It is inert data.

The one frontmatter consumer that parses these keys, `batch-select.mjs` (the swarm cone scheduler), reads with **flat line-oriented regex** and only by name: `one(f,k) = f.match(/^${k}:\s*(.*)$/m)` (line 46), `list(f,k)` for inline/block lists (lines 53-58). It reads `id, status, priority, service, depends_on, new_files, modified_files` at the **top level**. It never reads `build_envelope`, so the nested map neither feeds it nor breaks it — it is invisible. And note the converse: batch-select's `^service:` / `^new_files:` regexes match only column-0 keys, so if you nested a key it *does* read, it would silently misread it as empty. batch-select **assumes flat frontmatter** too; FM-001 is not alone in that assumption.

This refutes route (b)'s premise ("every consumer handles nested maps, only FM-001 complains"). Consumers do not *handle* the nested map — they *ignore* it, and the one that parses these keys requires them flat.

**The substantive harm** is here, not just cosmetic non-conformance. The 140 bury their cone data (`service`, `new_files`, `modified_files`) *inside* `build_envelope`, where batch-select cannot see it. Command:

```
# of the 140, how many expose the cone keys at the TOP level batch-select reads?
#   top-level service:  0 / 140      top-level new_files:  2 / 140
```

So for ~all 140, batch-select computes an empty cone → treats them as undeclared → each ships alone (`batch-select.mjs:99-138`). Unwrapping `build_envelope` to top-level keys does not just satisfy FM-001; it **restores the cone to where the scheduler reads it**.

## 4. What are the 140 (consistent designed convention, or ad-hoc?)

Perfectly consistent. All 140 use the single parent key `build_envelope:`, and its direct children are **identical across every one** (count of the 140 carrying each child key):

```
140 language          140 new_files        140 allowed_tools
140 service           140 modified_files   140 disallowed_tools
```

This is an intentionally designed "implementation envelope" (build language, service dir, files to create/modify, tool allow/deny lists), spanning 21 modules — `ai auth crm cuo doc email esop hr inv kb learn mcp okr plugin portal res rew skill ten time` — and 14 `done` specs. So the operator's hypothesis is confirmed on its face: this IS a deliberate, corpus-wide convention, not noise.

But "deliberate authoring convention" is not "sanctioned task@1 construct." The decisive tell is that the **exact same envelope data already exists in flat, FM-001-clean form** elsewhere. `TASK-SKILL-104` (`status: done`) carries `language, service, new_files, modified_files, allowed_tools, disallowed_tools` as **flat top-level keys with block lists** (spec.md:38-66) — no `build_envelope` wrapper — and passes FM-001 structurally. Two variants of one convention exist side by side: nested-under-`build_envelope` (the 140, flagged) and flat-at-top-level (SKILL-104 and kin, clean). The conformant target form is therefore already precedented in the corpus; the migrate is an **unwrap to a shape sibling specs already use**, and that shape is the one batch-select reads.

## 5. The apostrophe edge (TASK-SKILL-104:63) — which tool has the right quote model?

The line (`docs/tasks/skill/TASK-SKILL-104-capability-broker/spec.md:63`, a block-list item under a flat top-level `disallowed_tools:`):

```
  - allow skill subprocess to inherit broker's file descriptors (per §1 #4 — seal stdin/stdout/stderr only)
```

This is an **unquoted (plain) YAML scalar**. Two rules of YAML plain scalars decide it: (1) an apostrophe is a string delimiter only when it *begins* the scalar — a `'` mid-token (as in `broker's`) is a literal character; (2) ` #` (space-then-hash) begins a comment in plain-scalar context. So a real YAML parser reads the value as `allow skill subprocess to inherit broker's file descriptors (per §1` and strips `#4 — seal stdin/stdout/stderr only)` as a comment. There **is** a trailing comment here.

- **task-lint** (`parseScalar`, line 85): `if (s.includes(" #")) return { err: "trailing comment after value" }`. Flags it. **Correct** — the ` #4` is a comment per YAML.
- **migrator** (`trailingCommentIndex`, lines 84-99): tracks quote state and, on hitting the `'` in `broker's`, sets `q="'"` — entering a "single-quoted string" that never closes — so the later ` #4` is seen as *inside a string* and skipped. Returns -1, leaves the line alone. **Wrong** — it treats a mid-plain-scalar apostrophe as opening a quote, which YAML does not do.

**Verdict for #5: task-lint has the correct quote model; the migrator's quote-aware `#` detector is the bug.** The fix belongs in `fm001-migrate.mjs`: only treat `'`/`"` as a delimiter when it begins the scalar value, not when it appears mid-token. In-file corroboration that the ` #N` really is comment-eating live data: lines 64-66 of the same spec show a *prior* split of `(per §1 #5)` already mangled into `#5)` on its own line above a dangling `(per §1` — the exact damage a real YAML comment-strip inflicts. (Note this is orthogonal to the nested-map fork: even after the 140 are migrated, this one migrator bug remains and must be fixed for the migrator to be sound.)

## Recommendation

**Route (a): migrate the 140 specs. Do NOT relax FM-001 (reject route b).** Specifically, migrate by **unwrapping** `build_envelope` — promote its six children to flat top-level keys — rather than deleting it, because (i) that is the form a `done` sibling (`TASK-SKILL-104`) already uses, so it is precedented and audit-precedented; and (ii) `service`/`new_files`/`modified_files` at top level are exactly what `batch-select.mjs` reads, so the unwrap restores cone visibility to the scheduler instead of leaving it buried.

Why not route (b) — teaching FM-001 to accept nested maps:

1. Nested maps are outside the documented task@1 contract (CONTRACT.md field table, template.md, TASK-TEMPLATE.md — all flat). Accepting them is a `task@2` schema bump, not a lint fix.
2. task-lint rejects nested maps **by design** (header lines 22-28), on the stated principle "failing loudly beats silently accepting what the rule families never defined."
3. Relaxing the indentation check would NOT *validate* `build_envelope` — task-lint's per-field family (FM-101..116) has no model of nested maps — it would only **stop flagging** it, i.e. silently accept arbitrary unread structure. That weakens the machine floor: the precise failure the floor exists to prevent (cf. `2026-07-18-phase-corpus-measurement.md`: "the floor cannot see a false measurement").
4. No consumer reads `build_envelope`, and the one that parses these keys (`batch-select`) assumes flat — so (b) fixes a complaint nothing else shares while leaving the cone data unreadable.

Scope note for whoever extends the migrator: `build_envelope` is only *one* key of a larger non-task@1 dialect the 2026-07-14 migration injected (`phase, milestone, owner, slice, verify, source_pages, source_decisions, effort_hours, subtasks, risk_if_skipped, …`). Those pass FM-001 only because they are flat/lists; they are equally not task@1. Reaching a *fully* task@1-clean corpus is a larger reconciliation than this fork. But the fork question is answerable on its own, and the answer is unambiguous: **the nested map is a real violation — migrate it; the lint is right.**

Also fix `fm001-migrate.mjs`'s quote model (§5) so the migrator and task-lint agree on what a comment is, independent of the 140.

## Confidence and what I could not determine

**High confidence** on the core verdict (nested-map `build_envelope` is a real FM-001 violation, not lint over-strictness) and on the apostrophe verdict (task-lint right, migrator wrong). Both rest on re-run commands, not recollection; all five headline numbers (141 / 4004 / 140 / 1 / 497) re-derive exactly against `--json`/`rule_id`.

**High confidence** that nothing reads `build_envelope`: `grep -rln` across `tools modules scripts services apps` is empty. Residual risk: a consumer that iterates *all* frontmatter keys generically rather than by name would surface it without a literal `build_envelope` reference. I found no such consumer (batch-select reads by name; build_envelope is absent from `dist/`), so this risk is low but not disproven by grep alone.

**Could not determine:** whether the 14 `done` build_envelope specs were ever accepted by a `task-audit` that actually ran FM-001 over their nested maps, or reached `done` before/around the 2026-07-14 migration without re-audit. The pervasive `# UNREVIEWED` markers and 993 corpus-wide FM-112 findings indicate much of this lineage is auto-migrated and never re-floored — which, if anything, argues *against* the nested map being a blessed convention. I did not attempt to reconstruct each `done` spec's audit history.

**Migration caveat (for the operator, not a blocker):** 2 of 140 already carry a top-level `new_files:` in addition to the one inside `build_envelope`. A naive unwrap would create a duplicate key → FM-003. The migrator that performs the unwrap must detect and reconcile pre-existing top-level collisions rather than blindly hoisting.

---

Filed as investigation only. No status flipped, no spec frontmatter touched, task-lint and the migrator unchanged, IMP-117's spec untouched. The next actor owns the migrate.
