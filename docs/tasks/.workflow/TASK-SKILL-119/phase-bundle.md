# TASK-SKILL-119 phase bundle

## repo-context-map + honest gap (steps 1-5)
Checker-first approach (completeness authority pattern from 118): initial scan found 130 dead
references in 6 classes: <skill>/CONTRACT.md (never existed; SKILL.md is the contract), the moved
memory protocol (modules/memory/AGENTS.md -> modules/memory/cyberos/data/AGENTS.md), the moved
STATUS-REFERENCE (docs/tasks/ -> modules/skill/contracts/task/), the dead SDP
atlas (modules/cuo/README.md -> docs/module.md + docs/appendices.md §13), dead convention docs
(docs/RUBRIC_FORMAT/SPEC/AUDIT_LOOP -> live exemplar pair + appendices), and renamed contract dirs
(prd/srs/impl-plan -> full names). ECM: placeholder targets (<artifact>) skipped by grammar
(TASK-SKILL-115 precedent); archives exempted; https skipped; unused exemptions warn (t05).

## implementation (steps 6-14)
scripts/check_doc_anchors.sh (markdown links + backticked path#anchor citations; repo-root-relative
for known top dirs else file-relative; GitHub-slug anchor resolution; --list; exit 0/10/2) +
scripts/doc-anchor-exemptions.txt (reasoned allowlist, chain-allowlist discipline: appendices/
CHANGELOGs = historical record, runners/README = self-declared legacy, index.md GUIDE sources cite
build-generated html). Sweep: 388 files repointed citation-strings-only; ship workflow v1.x stale
note replaced with present-tense fact + chain-coverage citation; bare TBD reworded "future work,
unscheduled (no FR yet)". CI: payload-gate.yml step (path filters already cover modules/skill/** +
modules/cuo/**). Templates (_template scaffolds) repointed to the live atlas with section mapping.

## recorded deviation (newest wins)
AC 1's literal grep ("modules/cuo/README.md returns nothing") is satisfied for every LIVE contract
file; the string remains only inside the three exempted historical archives (appendices.md x2,
CHANGELOG.md) whose citations record the pre-split repo faithfully - rewriting them would falsify
history. The enforceable durable form is the checker (exit 0 = clean), which CI now runs.

## code review vs §1 (steps 16-18)
#1 SDP repoints with stage semantics preserved PASS (implementation-plan-author + ADR-author keep
their stage citations at the live location - grep verified via checker); #2 ship notes current +
no bare TBD PASS (grep clean); #3 checker per contract (grammar, slugs, DEAD format, --list, https
skip) PASS (t02-t04); #4 CI wired PASS (t06); #5 citation-strings-only PASS (534 ins/534 del -
symmetric one-line swaps across 387 files; frontmatter untouched except description-embedded
citation substrings, which are the §1 #1 class itself).

## coverage gate (steps 21-29)
test_check_doc_anchors.sh 6/6; 7/7 cyberos-init suites; ship_manifest 8/8; live tree: anchors OK
341 references, exit 0. tests_failed=0.

## regression note (recorded, refinement queued)
test_pair_parity t04 ("no SKILL.md lines removed vs HEAD") fired mid-flight on this FR's citation
swaps - a legitimate §1 #5 mutation class. Green at rest post-commit (worktree == HEAD). Third
instance of the point-in-time-guard class (after 116's reduced-profile and 209's t08): t04 should
scope to artefact-section heading ranges, per TASK-SKILL-118 §4 AC 4's original wording. Queued as a
next-batch refinement candidate rather than hot-patched (the guard is correct at rest and errs loud,
not silent).
