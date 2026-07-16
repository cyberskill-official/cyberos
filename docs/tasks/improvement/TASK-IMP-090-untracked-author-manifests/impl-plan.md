# TASK-IMP-090 implementation plan

1. **SKILL.md default** (clause 1.1) - CONTRACT_ECHO line 184: manifest_path default becomes `docs/tasks/.workflow/task-author.<slug>.manifest.json`. Caller override unchanged.
2. **Seed patterns** (clause 1.2) - install.sh lines 44-54: fresh seed writes both patterns via one `printf`; existing seed lacking the manifest pattern gains it once (grep -qxF guard + trailing-newline heal). Comment cites TASK-CUO-206 and TASK-IMP-090 so the next reader knows why two patterns live here.
3. **Index cleanup** (clause 1.3) - `git rm --cached` the three batch manifests; append `*.manifest.json` to this repo's `docs/tasks/.workflow/.gitignore` so they stay on disk untracked.
4. **Approval record** (clause 1.4) - write `docs/tasks/_audits/IMPROVEMENT-BATCHES-2026-07-16.md`: batches 1-3, members TASK-IMP-082..092, PLAN approvals, HITL verdicts, evidence commits, and the 086 corrective incident pointer.
5. **Coverage** (clause 1.5) - t07 in test_install_hygiene.sh: fresh-seed and append-once paths.
6. **Gates** - hygiene suite (t01-t07), then the parent's full suite pair + build + version-sync + scratch install.

Order matters: step 2 after step 1 (088 shares install.sh and lands first per the batch plan); step 3 last among the write steps so the seed it depends on already exists.
