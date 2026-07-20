# TASK-IMP-088 code review

Reviewer: parent ship-tasks agent (batch 3). Diff: `tools/install/install.sh` (+~14), `tools/install/tests/test_install_hygiene.sh` (t06 block).

## Clause -> proof

| Clause | Requirement | Proof |
|---|---|---|
| 1.1 | consumer install writes live `task_template: task@1` | install.sh:194 `cfg_tmpl_line="task_template: task@1"`; live scratch install -> `.cyberos/config.yaml:10 task_template: task@1` (gate log) |
| 1.2 | platform repo keeps the commented default | install.sh:195 `is_platform_repo && cfg_tmpl_line="# task_template: engineering-spec@1"`; t06_platform_keeps_comment ok |
| 1.3 | existing config.yaml untouched on re-install | t06_existing_config_untouched ok (byte compare) |
| 1.4 | scenarios land in the hygiene suite | suite reports 17 passed, 0 failed incl. all three t06 scenarios |

## Judgment

- **Correctness vs ticket**: the resolution chain is untouched, as the decision required - only the scaffold's literal changes, and only on consumer repos. The platform guard reuses the existing `is_platform_repo()` rather than inventing a second detector.
- **Blast radius**: one variable and one `printf` line inside a create-once block. A repo that already has config.yaml never enters it (1.3).
- **Failure mode if wrong**: a consumer whose first authoring run resolves engineering-spec@1 - precisely the state that cost a PLAN-gate override on the sachviet run, now asserted against.
- **Security**: none. One line of local, gitignored config; no execution surface, no secrets.
- **Backwards compatibility**: existing consumers are unaffected until they choose to edit; create-once guarantees no silent profile change under a live repo.
- **AI-specific**: no hallucinated APIs - `is_platform_repo` verified present in the file; diff is 14 lines, reviewable in full.

Verdict: no open findings.

HALT: review acceptance (reviewing -> ready_to_test) is a human gate.
