---
task_id: TASK-IMP-086
created: 2026-07-16
branch: batch/2-workflow-helpers
folds_into: "audit.md §gate-log (the spec's ACs verify against recorded evidence there; this draft is that record)"
---
# Gate log (draft) - TASK-IMP-086

All commands ran from the repo root on 2026-07-16, branch `batch/2-workflow-helpers`.
Chronology: E0 and E1 ran against the PRE-image (working tree clean at HEAD, no
068-081 row present); E-SPLICE performed the one write; E2-E6 ran against the
POST-image. Every command below is a pure read except E-SPLICE - a reviewer can
re-run E2-E6 verbatim and must get these exact outputs.

## E0 - pre-image scan (AC 4's scan as the pre-check, spec §3)

```
$ awk '/^## improvement/{f=1;next} /^## /{f=0} f && /^- \[/' docs/tasks/BACKLOG.md | sed 's/^- \[\([^]]*\)\].*/\1/' | sort | uniq -c
      3 done
     67 draft
      3 implementing
$ grep -n 'TASK-IMP-0\(6[89]\|7[0-9]\|8[01]\)' docs/tasks/BACKLOG.md; echo "grep exit: $?"
grep exit: 1
$ grep -o 'TASK-IMP-[0-9]*' docs/tasks/BACKLOG.md | sort | uniq -d | wc -l
0
```

73 rows (67 draft + 3 done + 3 implementing), zero 068-081 rows anywhere in the
file, zero duplicate id tokens. The section's row block ended at the 067 row with
082 adjacent - exactly the gap the spec describes.

## E1 - regenerator trial (the byte authority, spec Alternatives - MUST try first)

Dry-run against a temp copy so the repo was never touched by the script
(`ROOT = Path(__file__).resolve().parents[1]`, so a relocated script writes only
under /tmp/dry86):

```
$ rm -rf /tmp/dry86 && mkdir -p /tmp/dry86/scripts /tmp/dry86/docs
$ cp scripts/migrate_improvement_to_task.py /tmp/dry86/scripts/
$ cp -r docs/tasks /tmp/dry86/docs/
$ python3 /tmp/dry86/scripts/migrate_improvement_to_task.py --backlog
regenerated BACKLOG.md: 515 tasks across 29 modules            (exit 0)
$ diff docs/tasks/BACKLOG.md /tmp/dry86/docs/tasks/BACKLOG.md | grep -c '^[<>]'
5
$ diff docs/tasks/BACKLOG.md /tmp/dry86/docs/tasks/BACKLOG.md
10c10
< Totals: 336 draft, 4 ready_to_implement, 15 implementing, 155 done, 1 on_hold, 1 closed
---
> Totals: 336 draft, 4 ready_to_implement, 15 implementing, 158 done, 1 on_hold, 1 closed
240,242d239
< - [done] TASK-IMP-082-status-stamp-byte-stable - Status page provenance stamp becomes a corpus fingerprint (byte-stable)
< - [done] TASK-IMP-083-hookspath-aware-status-hook - install lands the status-sync hook where core.hooksPath points
< - [done] TASK-IMP-084-task-lint-machine-floor - task-lint, a deterministic machine floor under the task-audit rubric
$ grep -c 'TASK-IMP-0\(6[89]\|7[0-9]\|8[01]\)' /tmp/dry86/docs/tasks/BACKLOG.md; echo "grep exit: $?"
0
grep exit: 1
```

Verdict: FALL BACK to the surgical path (the spec's own off-ramp). The
regenerator's output (a) DELETES the three pre-existing `[done]` rows 082-084 and
(b) rewrites the repo-wide `Totals:` line - both outside what §1 #1.5 permits
("MUST NOT modify any line outside the improvement section, and MUST NOT edit any
pre-existing row other than the header count line"; protected invariant: "No row
deletion"). It also (c) emits ZERO rows for 068-081: `regen_backlog` lists only
ACTIVE statuses (scripts/migrate_improvement_to_task.py:19-20, :201) and all
fourteen tasks are `done`, so the regenerator cannot satisfy §1 #1.1 at all.
Its section header, however, reads `## improvement  (67 draft, 3 implementing,
17 done)` - independent confirmation of the E3 tally below.

## E-SPLICE - the one write (surgical backfill)

Rows were built from each task's frontmatter via yaml.safe_load and flowed into
the file without manual transcription (statuses and titles byte-verbatim). Guards:
HALT on missing spec.md, unparseable frontmatter, id/folder-stem mismatch, or a
multi-line title - no row is ever invented. Splice asserted 067/082 adjacency;
header recomputed from ALL rows in the section in the file's own STATUS_ORDER.

```python
import re, yaml, sys
from pathlib import Path
STATUS_ORDER = ["draft", "ready_to_implement", "implementing", "ready_to_review",
                "reviewing", "ready_to_test", "testing", "done", "on_hold", "closed"]
root = Path('docs/tasks/improvement')
rows = []
for d in sorted(p for p in root.iterdir() if p.is_dir() and re.match(r'TASK-IMP-(06[89]|07\d|08[01])-', p.name)):
    spec = d / 'spec.md'
    if not spec.is_file():
        sys.exit(f"HALT: {d.name} has no spec.md")
    m = re.match(r"\A---\n(.*?)\n---\n", spec.read_text(), re.S)
    fm = yaml.safe_load(m.group(1)) if m else None
    if not isinstance(fm, dict) or not all(k in fm for k in ('id','title','status')):
        sys.exit(f"HALT: {d.name}/spec.md frontmatter unparseable")
    if fm['id'] != '-'.join(d.name.split('-')[:3]):
        sys.exit(f"HALT: {d.name} id mismatch: {fm['id']}")
    if '\n' in str(fm['title']):
        sys.exit(f"HALT: {d.name} multi-line title")
    rows.append(f"- [{fm['status']}] {d.name} - {fm['title']}")
assert len(rows) == 14
bl = Path('docs/tasks/BACKLOG.md')
lines = bl.read_text().split('\n')
i067 = next(i for i,l in enumerate(lines) if l.startswith('- [draft] TASK-IMP-067-'))
i082 = next(i for i,l in enumerate(lines) if l.startswith('- [done] TASK-IMP-082-'))
assert i082 == i067 + 1
new_lines = lines[:i067+1] + rows + lines[i082:]
ihdr = next(i for i,l in enumerate(new_lines) if l.startswith('## improvement'))
inext = next(i for i in range(ihdr+1, len(new_lines)) if new_lines[i].startswith('## '))
tally = {}
for l in new_lines[ihdr+1:inext]:
    m = re.match(r'- \[([a-z_]+)\] ', l)
    if m: tally[m.group(1)] = tally.get(m.group(1), 0) + 1
hdr = f"## improvement  ({', '.join(f'{tally[s]} {s}' for s in STATUS_ORDER if tally.get(s))})"
new_lines[ihdr] = hdr
bl.write_text('\n'.join(new_lines))
```

Run output:

```
old header: ## improvement  (67 draft, 3 implementing, 17 done)
new header: ## improvement  (67 draft, 3 implementing, 17 done)
header byte-change: False
inserted 14 rows between line 239 and old line 240; section tally: {'draft': 67, 'done': 17, 'implementing': 3}
```

All fourteen frontmatter statuses read `done` at write time (no off-ramp present;
the emitter carries whatever the frontmatter says, verbatim). The recomputed
header equals the pre-existing header byte-for-byte - the old header already
counted 068-081 from frontmatter (regen forward-counts unlisted done tasks), so
the backfill brought the ROWS to parity with it and the header contributes zero
diff lines.

## E2 - folder count vs row count (AC 1 parity)

```
$ ls -d docs/tasks/improvement/TASK-IMP-* | wc -l
87
$ awk '/^## improvement/{f=1;next} /^## /{f=0} f && /^- \[/' docs/tasks/BACKLOG.md | wc -l
87
```

87 folders = 87 rows. With E0 (zero 068-081 rows before) and E4 (zero duplicate
stems after), every folder has EXACTLY one row.

## E3 - per-status tally vs the header line (AC 2)

```
$ awk '/^## improvement/{f=1;next} /^## /{f=0} f && /^- \[/' docs/tasks/BACKLOG.md | sed 's/^- \[\([^]]*\)\].*/\1/' | sort | uniq -c | sort -rn
     67 draft
     17 done
      3 implementing
$ grep -n '^## improvement' docs/tasks/BACKLOG.md
171:## improvement  (67 draft, 3 implementing, 17 done)
```

Tally (67 draft, 3 implementing, 17 done; 67+3+17 = 87) equals the header
exactly, in the file's own status order (STATUS_ORDER,
scripts/migrate_improvement_to_task.py:21-22, zero-count statuses omitted -
matching every other section header's convention). The 17 done = 3 pre-existing
rows (082-084) + 14 backfilled; the 3 implementing rows (085-087) are counted,
untouched.

## E4 - duplicate-stem scans (AC 4)

```
$ ls -d docs/tasks/improvement/TASK-IMP-* | xargs -n1 basename | sort | uniq -d | wc -l
0
$ awk '/^## improvement/{f=1;next} /^## /{f=0} f && /^- \[/{print $3}' docs/tasks/BACKLOG.md | sort | uniq -d | wc -l
0
$ grep -o 'TASK-IMP-[0-9]\{3\}' docs/tasks/BACKLOG.md | sort | uniq -d
TASK-IMP-080
$ grep -n 'TASK-IMP-080' docs/tasks/BACKLOG.md
252:- [done] TASK-IMP-080-served-bundle-version-drift - Served-bundle version drift — live site announced v0.1.0 after the 1.0.0 release; refreshed bundle + version-sync gate coverage for apps/console/web
253:- [done] TASK-IMP-081-web-console-bundle-ci-rebuild - CI leg rebuilds + recommits apps/console/web on real source changes - structural follow-up to TASK-IMP-080's served-bundle version-drift fix
```

Zero duplicate stems repo-wide: folder stems 0, row stems (the `$3` key token)
0. The single TOKEN-level hit is TASK-IMP-081's verbatim title citing
"TASK-IMP-080's ... fix" - quoted data inside a title tail, not a stem, and the
spec §3 title-verbatim edge case working as specified (it also explains why E0's
token scan showed 0 before: the citing 081 row did not exist yet). Row-block
order check:

```
$ awk '/^## improvement/{f=1;next} /^## /{f=0} f && /^- \[/{print $3}' docs/tasks/BACKLOG.md | sort -c && echo 'sort -c: in order'
sort -c: in order
```

The whole contiguous block (001..087) is stem-ascending - insertions landed
between the 067 and 082 rows without reordering anything.

## E5 - status + title verbatim recheck (AC 1)

Each of the fourteen rows re-compared byte-for-byte against a fresh yaml parse of
its spec.md frontmatter (`- [<status>] <folder-stem> - <title>`; exactly one
matching line required per task):

```
TASK-IMP-068-payload-version-drift-gate: OK (1 exact row)
TASK-IMP-069-publish-payload-on-release: OK (1 exact row)
TASK-IMP-070-remote-update-awareness: OK (1 exact row)
TASK-IMP-071-durable-release-trigger: OK (1 exact row)
TASK-IMP-072-repo-wide-version-consistency: OK (1 exact row)
TASK-IMP-073-fix-capacitor-mobile-app-icon: OK (1 exact row)
TASK-IMP-074-ship-workflow-hardening: OK (1 exact row)
TASK-IMP-075-mas-updater-exclusion: OK (1 exact row)
TASK-IMP-076-root-cli-and-mcp-connector: OK (1 exact row)
TASK-IMP-077-ios-icon-alpha-flatten: OK (1 exact row)
TASK-IMP-078-store-build-number-monotonic: OK (1 exact row)
TASK-IMP-079-docs-ship-race: OK (1 exact row)
TASK-IMP-080-served-bundle-version-drift: OK (1 exact row)
TASK-IMP-081-web-console-bundle-ci-rebuild: OK (1 exact row)
verbatim mismatches: 0
```

Covers the five titles containing the grammar's own ` - ` separator (068, 070,
071, 072, 081) and the em-dash/backtick/apostrophe titles (073-080): all
byte-verbatim, all on one line, untagged like the section's corpus rows.

## E6 - diff footprint (AC 3)

```
$ git diff --stat docs/tasks/BACKLOG.md
 docs/tasks/BACKLOG.md | 14 ++++++++++++++
 1 file changed, 14 insertions(+)
$ git diff docs/tasks/BACKLOG.md | grep '^@@'
@@ -237,6 +237,20 @@ Totals: 336 draft, 4 ready_to_implement, 15 implementing, 155 done, 1 on_hold, 1
$ git diff -U0 docs/tasks/BACKLOG.md | grep '^@@'
@@ -239,0 +240,14 @@ Totals: 336 draft, 4 ready_to_implement, 15 implementing, 155 done, 1 on_hold, 1
$ echo "added: $(git diff docs/tasks/BACKLOG.md | grep '^+' | grep -cv '^+++')  removed: $(git diff docs/tasks/BACKLOG.md | grep '^-' | grep -cv '^---')"
added: 14  removed: 0
$ grep -n '^## ' docs/tasks/BACKLOG.md | sed -n '/## improvement/{p;n;p}'
171:## improvement  (67 draft, 3 implementing, 17 done)
261:## inv  (10 draft, 1 ready_to_implement)
```

One file, one hunk, 14 insertions, 0 deletions, 0 modifications. The `-U0` hunk
`-239,0 +240,14` is a pure insertion after old line 239 (the 067 row); new lines
240-253 sit strictly inside the improvement section (header line 171, next
section at 261). The `Totals: ...` text after `@@` is git's funcname context
LABEL (markdown `##` headings do not match the default heuristic, so git labels
the hunk with the nearest preceding alphanumeric-initial line) - the Totals line
itself is untouched, as `removed: 0` and the `--stat` prove. Note: the Totals
line's own repo-wide drift (155 vs the corpus's 158 done, E1) predates this task
and is out of scope per the spec ("Other module sections' drift").

## AC map

| AC | traces_to | evidence |
|---|---|---|
| AC 1 | §1 #1.1, #1.2, #1.3 | E2 (87 folders = 87 rows) + E0 (zero pre-existing 068-081 rows -> exactly one each) + E5 (statuses and titles frontmatter-verbatim, section grammar, untagged) + E4b (stem-ascending block) |
| AC 2 | §1 #1.4 | E3 (tally 67/3/17 = header line 171, recomputed from ALL 87 rows incl. 082-084 done and 085-087 implementing) + E-SPLICE (recompute output) + E1 (regenerator independently computes the same header) |
| AC 3 | §1 #1.5 | E6 (single insertion-only hunk inside section lines 171-260; 0 removed/modified lines - not even the header, which recomputed to identical bytes) + E1 (the path that WOULD have churned rows, rejected on recorded evidence) |
| AC 4 | §1 #1.6 | E0 (pre-check) + E4 (folder stems 0 dup, row stems 0 dup, the one token hit explained as 081's quoted title) |

Human gates (spec §5): review acceptance and final acceptance are recorded human
verdicts; nothing here sets a status.

## CORRECTIVE ADDENDUM (2026-07-16, post-acceptance verification)

The E2/E5/E6 evidence above was truthful for the view it was measured in and FALSE for
every committed object: no commit on batch/2-workflow-helpers carried the 14 rows, and the
batch-1 rows 082-084 were lost from committed state during the same window. Root cause:
concurrent writes to docs/tasks/BACKLOG.md through Cowork's two filesystem views (this
task's agent wrote host-side; the parent's phase flips and commits ran sandbox-side); each
writer's reads were self-consistent while updates crossed views and were lost. The header
never screamed because the pre-existing corpus header already counted the unindexed done
tasks, and backlog-mutate's incremental count adjust inherited that baseline (34 vs true 20).

Flagged by the PR review bot (devin-ai-integration) against the pushed branch. Repair, as
commit 092c9887: single-writer re-insert of all 17 rows via backlog-mutate.mjs, full header
retally from actual rows, then verification against the COMMITTED GIT OBJECT (not any
working view): `git show 092c9887:docs/tasks/BACKLOG.md` = 87 rows in the improvement
section, 0 duplicate stem tokens, header `(67 draft, 20 done)`, all 17 restored [done]
rows present, working tree clean against HEAD.

Rule adopted from this incident (filed as IMP-18 in IMPROVEMENT_HANDOFF.md): shared files
get ONE writer through ONE view, and acceptance evidence for content deliverables MUST be
measured on the committed object, never on a working view.
