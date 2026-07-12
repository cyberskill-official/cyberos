---
id: FR-SKILL-119
title: "Stale-reference sweep - repoint dead SDP anchors across modules/skill and refresh obsolete ship-workflow notes, with a doc-anchor checker"
module: SKILL
priority: SHOULD
status: ready_to_implement
class: improvement
verify: T
phase: Wave B - finish the children
owner: Stephen Cheng (CTO)
created: 2026-07-12
shipped: null
memory_chain_hash: null
related_frs: [FR-SKILL-115, FR-SKILL-118, FR-DOCS-002]
depends_on: []
blocks: []
source_pages:
  - modules/skill/implementation-plan-author/SKILL.md
  - modules/skill/architecture-decision-record-author/SKILL.md
  - modules/cuo/chief-technology-officer/workflows/ship-feature-requests.md
  - modules/cuo/docs/appendices.md
source_decisions:
  - "2026-07-12 investigation: many modules/skill/*/SKILL.md cite modules/cuo/README.md#software-development-process, but modules/cuo/README.md no longer exists (docs split moved the SDP to modules/cuo/docs/module.md + appendices.md §13)."
  - "ship-feature-requests.md:199 still claims the code-review pair 'doesn't exist yet' (it exists and is vendored); :137 references a future Issue Request artefact as bare TBD."
  - "Distinct from FR-SKILL-115 (done), which swept <placeholder> syntax; this sweeps dead cross-references and adds the checker that keeps them dead."
language: bash + markdown
service: modules/skill/ + modules/cuo/ + scripts/
new_files:
  - scripts/check_doc_anchors.sh
  - scripts/tests/test_check_doc_anchors.sh
modified_files:
  - modules/skill/implementation-plan-author/SKILL.md
  - modules/skill/architecture-decision-record-author/SKILL.md
  - modules/cuo/chief-technology-officer/workflows/ship-feature-requests.md
  - "modules/skill/*/SKILL.md (full sweep set = scripts/check_doc_anchors.sh --list output at implementation time)"
---

# FR-SKILL-119: Stale-reference sweep + doc-anchor checker

## §1 - Description

Skills are contracts; contracts that cite documents which no longer exist train agents to distrust citations. The SDP anchor died in the docs split and the ship workflow carries notes that predate its own children. This FR sweeps both and adds the checker that prevents recurrence.

Normative clauses:

1. Every `modules/skill/*/SKILL.md` (and any other modules/skill markdown) citing `modules/cuo/README.md#software-development-process` - including the `SDP §2(a)..(g)` lettered forms - MUST be repointed to the live SDP location (`modules/cuo/docs/appendices.md` §13 stage mapping, or `modules/cuo/docs/module.md` where the prose form fits), preserving each citation's stage letter/number semantics unchanged.
2. `ship-feature-requests.md` MUST drop the obsolete note (currently near line 199) claiming the code-review pair may not exist yet, replacing it with the current fact (pair exists, vendored); the Issue Request `TBD` (near line 137) MUST either point at a tracked FR id or be reworded as explicitly unscheduled future work - a bare `TBD` MUST NOT remain.
3. A script `scripts/check_doc_anchors.sh` MUST scan `modules/skill/**/*.md` and `modules/cuo/**/*.md` for repo-relative markdown links and inline path#anchor citations, and verify each target file exists and (when an anchor is given) the anchor resolves to a heading in that file (GitHub slug rules: lowercase, spaces to hyphens, punctuation stripped). Unresolved references exit 10 as `DEAD <citing-file>:<line> -> <target>`; external URLs (http/https) MUST be skipped; a `--list` flag prints the would-be-swept set without failing.
4. The checker MUST run in CI on changes to `modules/skill/**` or `modules/cuo/**` (extend `payload-gate.yml` from FR-IMP-068 with a step, or the existing voice-and-consistency workflow - implementer's choice, documented in the workflow file).
5. The sweep MUST NOT alter any skill's trigger description, frontmatter, or artefact contract - citation strings only (same byte-stability discipline as FR-SKILL-118 §1 #7).

## §2 - Why this design

Fixing the anchors without a checker just schedules the next rot; a checker without slug-aware anchor resolution would only catch deleted FILES, and the observed failure is a deleted SECTION HOST. Scanning both markdown link syntax and inline `path#anchor` prose covers how skills actually cite (they use both). CI placement rides existing gates rather than adding a new workflow.

## §3 - Contract

```
scripts/check_doc_anchors.sh [--list] [root]
  exit 0   all repo-relative references resolve (prints "anchors OK: N checked in M files")
  exit 10  DEAD <file>:<line> -> <target>   (one per unresolved reference)
  exit 2   root unreadable
  --list   print each reference (resolved or not) as "<file>:<line> <target> <ok|DEAD>"
```

## §4 - Acceptance criteria

1. **Zero dead SDP anchors after the sweep** (§1 #1) - `grep -rn "modules/cuo/README.md" modules/skill modules/cuo` returns nothing, and the checker exits 0 over both trees.
2. **Stage semantics preserved** (§1 #1) - the two named SKILL.md files (implementation-plan-author, architecture-decision-record-author) still cite their respective SDP stages (implementation prep / architecture decision), now at the live location.
3. **Ship workflow notes are current** (§1 #2) - line-199-class note gone, replaced by the present-tense fact; no bare `TBD` remains in the file (grep clean), the Issue Request mention names an FR or says "future work, unscheduled".
4. **Checker resolves anchors, not just files** (§1 #3) - a fixture link to an existing file but nonexistent heading is reported DEAD with file:line; the same link with a valid heading passes.
5. **External URLs skipped, --list works** (§1 #3) - an https link never fails the check; `--list` prints every reference with its status and exits 0.
6. **CI wired** (§1 #4) - the chosen workflow runs the checker on the two path filters; the step is present and the workflow parses.
7. **Contracts byte-stable outside citations** (§1 #5) - for every swept SKILL.md, the diff touches only citation strings (no frontmatter, no description, no artefact-section changes).

## §5 - Verification

```bash
# scripts/tests/test_check_doc_anchors.sh
t01_sweep_leaves_zero_dead()     # AC 1
t02_stage_semantics_kept()       # AC 2  (grep the two files for stage wording + new target)
t03_ship_notes_current()         # AC 3
t04_anchor_vs_file_resolution()  # AC 4  (fixture tree with good-file/bad-anchor case)
t05_external_and_list()          # AC 5
t06_ci_step_present()            # AC 6
t07_citation_only_diffs()        # AC 7  (git diff --unified=0 scoped assertions on a sample)
```

## §6 - Implementation skeleton

Checker: extract candidates via two greps (`\]\([^)h][^)]*\)` for md links; `[a-zA-Z0-9_./-]+\.md(#[a-z0-9-]+)?` for inline paths), normalize against repo root, slugify headings of each target once into an assoc cache, compare. Sweep: run `--list`, sed the dead SDP form to the live target across the listed files, hand-fix the two workflow notes.

## §7 - Dependencies

None upstream; FR-IMP-068's workflow is the preferred CI host but the voice-and-consistency workflow is an acceptable fallback (implementer documents the choice). Related: FR-SKILL-115 (prior sweep, done - different defect class), FR-DOCS-002 (the docs split that orphaned the anchor).

## §8 - Example payloads

```
$ bash scripts/check_doc_anchors.sh
DEAD modules/skill/implementation-plan-author/SKILL.md:5 -> modules/cuo/README.md#software-development-process
DEAD modules/skill/architecture-decision-record-author/SKILL.md:5 -> modules/cuo/README.md#software-development-process
$ echo $?
10
```

## §9 - Open questions

None blocking. Whether the checker later covers docs/feature-requests cross-references too is a cheap follow-up flag; scope here is the skill/cuo contract trees where agents read citations at run time.

## §10 - Failure modes inventory

1. Slug algorithm mismatch (unicode, duplicate headings with -1 suffixes) - implement GitHub's documented rules incl. duplicate suffixing; fixture covers a duplicated heading.
2. False positives on code blocks containing path-like strings - the scanner MUST skip fenced code blocks; fixture includes a fenced `modules/cuo/README.md` mention that must not fail.
3. Sweep sed over-matching (a skill legitimately discussing the OLD path as history) - the sweep list comes from the checker (resolution-based), not from a blind grep; historical mentions inside fenced blocks survive per #2.
4. Anchor exists only in the rendered site, not the markdown (generated pages) - scope is repo markdown; generated-site anchors are FR-DOCS-002's builder concern.
5. Checker runtime creep on big trees - single-pass with a per-file heading cache; budget < 5s over modules/, asserted in t05.

## §11 - Implementation notes

Keep `DEAD ` output grep-stable. The sweep commit should separate mechanical repoints (one commit) from the two hand-edited workflow notes (second commit) for reviewable diffs.

*End of FR-SKILL-119.*
