# TASK-IMP-083 — observability injection

## Where observability lives for this change (and why that is the whole surface)

This task ships shell that runs in two moments an operator is already watching a terminal: install/uninstall time, and commit time. There is no daemon, no metrics pipeline, no log file — stdout/stderr IS the telemetry, and the tests grep it, which keeps the surface honest (an assertion on output is an SLO on output).

1. The install summary's auto-sync line — `auto-sync -> ${HOOK_SET}; …` — is the single operator-facing statement of what the hook step did. The failure this task kills was an OBSERVABILITY failure first: the line said "pre-commit hook v2 installed" while the hook sat in a directory git never reads — success indistinguishable from failure. The injected signal is `${hook_at}`: when `core.hooksPath` is set the line names the path actually written ("… installed at .githooks/pre-commit"), so a human can verify WHERE with `ls` and git's own config. When unset the line is byte-identical to before — the absence of a path suffix is itself information (default location), and the regression contract (§1.4) forbids more. Asserted by `t05_summary_names_path` and (exact wording, negative case) `t05_no_hookspath_regression`.

2. The hook's own echo lines are the runtime proof of firing, unchanged by design (hook bodies are out of scope): `cyberos: regenerating docs/status …` / `cyberos: docs/status staged` (standalone), `cyberos: docs/status regenerated + staged` (append block), and the blocking `cyberos: ERROR …` lines to stderr. Because they only print when git actually executes the hook, they are location-proof: on a hooksPath repo they now appear at commit time where before this task there was silence. `t05_hookspath_standalone` asserts the behavior those lines narrate (docs/status regenerated AND staged into the same commit) rather than scraping commit-time stdout — the staged tree is the stronger, quieter witness.

3. Uninstall's action lines — `removed managed pre-commit hook` / `stripped cyberos block from pre-commit` — now truthfully describe the resolved location, and after the ownership fix they can no longer report "removed managed" while deleting a FOREIGN hook. (Enabling fix: the mis-grouped root resolution meant uninstall printed "nothing to do (no .cyberos/)" on every git repo — a false-negative signal — now unreachable.)

## What was deliberately NOT injected

- No new echo of the resolved path from inside step 6b: the summary line already carries it once; a second print is drift surface.
- No verbose/debug flag: the state machine's five HOOK_SET strings enumerate every terminal state; a sixth channel would say nothing new.
- No hook-body changes (byte-identical invariant; the append block runs inside FOREIGN hooks where extra output is someone else's noise).
