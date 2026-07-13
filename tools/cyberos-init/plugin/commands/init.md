---
description: Install CyberOS into the current repo (self-hosting - gate autodetect, .cyberos/ machine, FR backlog, agent entry files). Idempotent; also applies updates.
argument-hint: "[repo path, default: current repo]"
---
Install (or update) CyberOS in repo_root = ${1:-the current repo}. Prefer the full payload; fall back to self-hosting from this plugin's own bundle. Never touch an existing `BACKLOG.md`, FRs, `AGENTS.md`, `gates.env`, or BRAIN store.

1. Full payload path (preferred). Locate a CyberOS payload: `$CYBEROS_PAYLOAD`, a sibling checkout's `dist/cyberos/`, or `~/Projects/CyberSkill/cyberos/dist/cyberos/`. If `init.sh` is found there, run `bash <payload>/init.sh <repo_root>` and report its output. init.sh already does the whole job: vendor + gates + backlog + CHANGELOG seed + automatic FR migration with a verify line + the status page at `docs/status/` (index.html + assets/, titled after the target repo; standalone docs/ROADMAP.md, docs/BACKLOG.md, docs/CHANGELOG.md are adopted into their canonical homes where empty, then removed - the page replaces them) + auto-sync (pre-commit hook + run-gates regeneration) + agent surface + one managed `.gitignore` block + payload self-cleanup (an in-repo `.cyberos-init/` copy removes itself on success). Relay the `cyberos-migrate verify:` line and any WARNs to the user. Done.

2. Self-host path (no payload anywhere). Replicate init from the plugin bundle:
   - Detect the repo's build / lint / test / coverage commands (Cargo, package.json, pyproject, go.mod, or Makefile).
   - Create `.cyberos/cuo/` and copy the bundled doctrine into it from `${CLAUDE_PLUGIN_ROOT}/skills/ship-feature-requests/cuo/` (`ship-feature-requests.md`, `EXECUTION-DISCIPLINE.md`, `STATUS-REFERENCE.md`).
   - Write `.cyberos/gates.env` with the detected `BUILD_CMD` / `LINT_CMD` / `TEST_CMD` / `COVERAGE_CMD`, `COVERAGE_MIN=90`, `CAF_ENABLED=false`, `AWH_ENABLED=false`, `HITL_REQUIRED=true`.
   - Scaffold `docs/feature-requests/` with a `BACKLOG.md` index (one backlog for product and improvement FRs; improvement rows tagged `(improvement)`).
   - Write `.cyberos/AGENT-ENTRY.md` - the agent-independent entry point: drive the backlog per `.cyberos/cuo/ship-feature-requests.md`; gates via `gates.env`; HITL required (the human alone sets `ready_to_test` and `done`); agents never push, merge, or deploy.
   - Create pointer stubs `CLAUDE.md`, `GEMINI.md`, `.cursorrules` ONLY where absent, each pointing at `.cyberos/AGENT-ENTRY.md`.
   - Seed `CHANGELOG.md` ONLY where absent (Keep-a-Changelog, one `## [0.1.0] - <date>` section).
   - Ensure `.gitignore` covers `.cyberos/` inside one managed block (`# >>> cyberos ... >>>` / `# <<< cyberos <<<`); regenerate the block in place, never duplicate it, never touch lines outside it.
   - If `docs/feature-requests/` holds flat `FR-*.md` files (root or module level), migrate them to folder-per-FR (`<module>/<STEM>/spec.md`; root-level FRs take their frontmatter `module:`, else the id segment) and update references. Report anything you could not place.

3. Report: which path ran, detected gate commands, what was created vs preserved. If self-hosted, note what the full payload adds (memory protocol + BRAIN, author/audit skills, caf/awh gates) and how to get it: the CyberOS desktop app's Ops tab, or `bash tools/cyberos-init/build.sh` in a CyberOS checkout.

Finish by telling the user their first move: write `FR-001-<slug>.md` (`status: ready_to_implement`), add its backlog row, then run `/ship-feature-requests`.
