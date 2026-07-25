# TASK-IMP-022 repo context map

## Cone

- `scripts/check_defensive_asserts.sh` (new) — the scanner. Bash wrapper + `python3` heredoc.
- `scripts/tests/test_assert_lint.sh` (new) — t01–t10; registered by `run_all.sh`'s glob, not by a list.
- `modules/skill/task-audit/RUBRIC.md` §9 — one table row + one subsection (`TRACE-008`).
- Six test files with a live `DA-001` site: `modules/cuo/tests/test_baseline.py`,
  `test_placeholder_check.py`, `test_type_discriminator.py`; `modules/memory/tests/core/test_crypto_mode.py`,
  `test_import.py`, `test_store_acl.py`.
- `CHANGELOG.md`.

## Patterns the change must follow

- **`check_doc_anchors.sh` is the shape** (TASK-SKILL-119): bash wrapper, `python3 - "$repo" "$mode" <<'PY'`,
  exit 0 clean / 10 with one machine-readable line per finding / 2 unusable, `--list` census that always
  exits 0. Following it means the new lint is legible to anyone who has read the old one.
- **The glob IS the registration** (`run_all.sh` header, TASK-IMP-107/128): a suite is gated by existing under
  `scripts/tests/`. Adding a name to a second list is the failure mode that header exists to warn about — so
  the suite is not named anywhere, and `t09` asserts the lint's roots match the runner's rather than trusting
  two lists to stay equal.
- **Stated bounds over silent narrowing** (TRACE-007, TASK-IMP-124): every place the floor does not reach —
  `services/**`, heredoc bodies, shell polarity — is named in the artefact a reader would rely on.
- **Fail-loud, no baseline files.** The corpus is fixed to zero rather than snapshotted at six. A baseline
  file is a defensive assertion at the repo level.

## Blast radius

- **Files:** 2 new, 8 modified. The scanner is read-only: `ast.parse` never evaluates, and the shell side
  never executes a line it reads.
- **Runtime:** ~0.3 s over 125 files. It joins the pre-commit path (via `run_all.sh`) and the `suite-gate`
  workflow; neither budget moves measurably.
- **Consumer impact:** none. Nothing is vendored into the payload — this gates the cyberos repo's own corpus.
  A consumer install is byte-identical before and after.
- **Behaviour change in the six edited tests:** each assertion gets STRICTLY stronger. The risk is a
  false red on a future legitimate variation, which is the intended trade: the six were green on
  behaviour the code does not have.

## Module placement

Correct. `improvement` — test-corpus integrity and one rubric rule. No user-visible behaviour, no product
surface, no payload change.
