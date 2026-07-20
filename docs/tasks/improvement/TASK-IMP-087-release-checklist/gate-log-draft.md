# TASK-IMP-087 gate-log draft (seed for audit.md §gate-log)

Recorded 2026-07-16 against `docs/release/RELEASE-CHECKLIST.md` as written (pre-review state: 18 checklist rows, 3 checked / 15 open / 0 waived). All commands run from the repo root. Grep ids G1-G9 are referenced by edge-case-matrix.md Covered-by cells. Checklist rows are the rows whose first cell matches `^[A-E][0-9]+$`; the channel matrix required by §1.4 is a reference table (3 columns by design), not a set of checklist lines - the structure checks therefore key on the row-id pattern.

## AC 1 - structure (every row: 5 cells, closed-set state, waived needs reason)

G1 cell count - every checklist row splits into exactly 5 cells:

    awk -F'|' '/^\| [A-E][0-9]+ \|/ {print $2": fields="NF" (want 7)"}' docs/release/RELEASE-CHECKLIST.md

Output: 18 rows (A1-A6, B1-B4, C1, D1-D3, E1-E4), every one `fields=7` (awk -F'|' yields empty lead + 5 cells + empty tail = 7 fields). PASS.

G2 closed-set state - state cell values, deduplicated:

    awk -F'|' '/^\| [A-E][0-9]+ \|/ {gsub(/ /,"",$5); print $5}' docs/release/RELEASE-CHECKLIST.md | sort | uniq -c

Output: `3 checked`, `15 open`. Nothing outside {open, checked, waived}. PASS.

G3 waived-requires-reason - waived rows with their evidence length:

    awk -F'|' '/^\| [A-E][0-9]+ \|/ && $5 ~ /waived/ {print $2" evidence-len:"length($6)}' docs/release/RELEASE-CHECKLIST.md

Output: 0 rows - vacuously satisfied today; the rule is normative in the document's Row contract ("A `waived` state REQUIRES its reason in the Evidence cell") and this grep is the re-runnable detector for any future waiver landing empty. PASS (vacuous, disclosed).

## AC 2 - presence (seven IMP-15 lines + three decision lines + channel matrix)

G4 the seven IMP-15 lines by tag:

    grep -o 'IMP-15\.[0-9][ab]*' docs/release/RELEASE-CHECKLIST.md | sort | uniq -c

Output: IMP-15.1 x2 (header range + A1), IMP-15.2a x1 (B1), IMP-15.2b x1 (B2), IMP-15.3 x1 (B3), IMP-15.4 x1 (C1), IMP-15.5 x1 (B4), IMP-15.6 x1 (D1), IMP-15.7 x2 (header range + D3). All seven seed lines present (15.2 split a/b so each command sits in its own cell). PASS.

G5 decision lines in group (e):

    grep -cE '^\| E[0-9]+ \| IMP-06' docs/release/RELEASE-CHECKLIST.md   # -> 1
    grep -cE '^\| E[0-9]+ \| IMP-07' docs/release/RELEASE-CHECKLIST.md   # -> 1
    grep -cE '^\| E[0-9]+ \| IMP-11' docs/release/RELEASE-CHECKLIST.md   # -> 1
    grep -cE '^\| E[0-9]+ \| IMP-08' docs/release/RELEASE-CHECKLIST.md   # -> 1

Output: one row each - E1 (IMP-06), E2 (IMP-07), E4 (IMP-11) are the three §1.3 decision lines, all state `open` with the decision-record pointer (batch-2 PLAN gate manifest Q2/Q3/Q4 + spec `source_decisions`); E3 (IMP-08) is the additional scheduled channel-implementation line. PASS.

G6 channel matrix + re-verify + research date (counts of literal markers):

    grep -cE '\.devin/rules/' ...          # -> 2 (matrix row + E3)
    grep -cE '\.agents/skills/' ...        # -> 2 (matrix row + E3 symlink path prefix)
    grep -cE '\.windsurfrules' ...         # -> 2   |  grep -cE '\.windsurf/rules/' ... # -> 2
    grep -cE '2026-07-16' ...              # -> 8 (research date + decision date mentions)
    grep -c 're-verify the channel matrix below against current tool conventions BEFORE the tag' ... # -> 1

Matrix row count: `awk '/^\| Agent \/ tool \|/,/^$/' ... | grep -c '^|'` -> 11 (header + separator + 9 tool rows: spine, Claude, Gemini, Cursor, Grok, Copilot, .agents/rules, and the two researched candidates .agents/skills + Devin/Windsurf). PASS.

## AC 3 - machine lines name their command verbatim

G7 command-cell extraction (backticked commands on agent rows):

    grep -oE '`(cd dist/cyberos && npm pack --dry-run|cd dist/cyberos && npm pack|npx --yes <path-to-tgz> help && npx --yes <path-to-tgz> install|bash tools/install/build.sh && bash tools/install/check-version-sync.sh dist/cyberos && bash scripts/tests/run_all.sh|git clone sachviet && npm ci && npm run coverage|gh workflow run release.yml)`' docs/release/RELEASE-CHECKLIST.md

Output - all six extracted verbatim:

    `bash tools/install/build.sh && bash tools/install/check-version-sync.sh dist/cyberos && bash scripts/tests/run_all.sh`   (A6 trio)
    `cd dist/cyberos && npm pack --dry-run`                                    (B1)
    `cd dist/cyberos && npm pack`                                              (B2)
    `npx --yes <path-to-tgz> help && npx --yes <path-to-tgz> install`          (B2)
    `gh workflow run release.yml`                                              (B4, operator-owned but command named)
    `git clone sachviet && npm ci && npm run coverage`                         (D3)

Human-only lines (A1, B3, C1, D1, D2, E1-E4) each state their satisfying evidence in prose (session record, research pass, section content, decision pointer). PASS.

## AC 4 - no secrets; cross-links resolve

G8 credential-pattern scan (expect zero hits; grep exit 1 = none):

    grep -nEi '(ghp_[A-Za-z0-9]|github_pat_|xox[baprs]-|AKIA[0-9A-Z]{16}|-----BEGIN|(api[_-]?key|password|secret|token)[[:space:]]*[:=][[:space:]]*[A-Za-z0-9_-]{8,}|Bearer [A-Za-z0-9])' docs/release/RELEASE-CHECKLIST.md

Output: no matches (exit 1). The document also avoids bare credential vocabulary entirely ("credentials" appears once, in the prohibition). PASS.

G9 cross-link existence (paths from the repo root; commits via read-only git):

    OK  ../IMPROVEMENT_HANDOFF.md            (sibling of the checkout, per the doc's own note -
                                              deliberately not tracked in-repo; resolves on the
                                              operator layout the doc names)
    OK  docs/tasks/.workflow/task-author.improvement-batch-2.manifest.json
    OK  docs/tasks/improvement/TASK-IMP-085-workflow-helpers/spec.md
    OK  .github/workflows/release.yml        (on: push tags 'v*' + workflow_dispatch - B4's claim)
    OK  dist/cyberos/cyberos.plugin          (B3's asset)
    OK  dist/cyberos/GUIDE.md                (D2; source tools/install/docs/index.md via build.sh:195)
    OK  CHANGELOG.md                         (head: `## [1.0.0] - 2026-07-14` - D1's claim)
    OK  scripts/tests/test_render_stamp.sh + tools/install/tests/test_install_hygiene.sh + tools/install/tests/test_task_lint.sh (A2-A4 suites)
    commits feff8cef a882e705 81ac11a3 27292774 ca9ae490 e9cfb97a: all resolve (`git cat-file -t` = commit);
    range 27292774^..ca9ae490 = 6 commits (the batch-1 governed run). PASS.

Repo facts re-verified independently for the evidence cells: `dist/cyberos/package.json` name `@cyberskill/cyberos` version 1.0.0 bin `cyberos -> cli/bin/cli.mjs`; `dist/cyberos/.claude-plugin/marketplace.json` stamps 1.0.0.

## Summary

AC 1 PASS (G1/G2/G3) - AC 2 PASS (G4/G5/G6) - AC 3 PASS (G7) - AC 4 PASS (G8/G9). 0 undefined states; 0 empty waivers; 6 verbatim machine commands; 9-row channel matrix with research date and re-verify line; all cross-links resolve on the stated layout.
