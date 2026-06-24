# Outside-resource intake

How an outside resource (a library, a codebase, a reference doc, field data, a design or research
artifact) is evaluated and, only if it earns it, folded into a CyberOS module. The funnel has two
stages and they are kept apart on purpose: a source is unproven until it earns absorption, so it never
touches the gated tree until then.

## Stage 0 - playground (ungated, on-disk only)

Unproven sources live in `playground/` at the repo root. It is gitignored (see the `.gitignore`
"Workbench consume" block and `playground/CONSUMED-FROM-WORKBENCH.md`): CyberSkill-authored notes are
tracked, cloned upstream repos stay on disk for browsing but out of git, so a candidate never bloats
the history or affects CI. `playground/` is outside the Rust workspace (`services/`) and outside the
awh/caf gate's changed-module detection (`modules/` and `services/` only), so nothing here can build,
ship, or move a gate.

What happens in Stage 0 is evaluation, not integration: read the source, and write a one-page verdict
note - what it provides, which module it could strengthen, the overlap and gaps, the license, and a
clear worth-it / not-worth-it call with the reason. No FR mapping, no vendoring, no code yet. Most
candidates should stop here.

To hand me a source: drop it in `playground/` (or give a path or URL) and say which module you have in
mind and what outcome you want (harden, expand, add coverage, replace). If you have no target in mind,
that is fine - the verdict note proposes one.

## Stage 1 - absorption (gated)

Only a candidate with a positive verdict graduates. Then, and only then:

1. Write `docs/absorptions/<name>-absorption.md` - the integration plan: exact overlap with the target
   module, the seam, the license + provenance line, and the FRs each change lands against (authored to
   10/10 first if new). This file is tracked.
2. Decide the seam. Tooling or a verification harness vendors into `tools/<name>/` with its own gate,
   the way CAF did. Library or behaviour folds into the module's crate or package behind its existing
   public surface. A doc or spec becomes FR content or a knowledge note, never loose prose.
3. Ship it through the same chain as any change: implement, review, test, then the module's awh gate
   (rerun its golden set vs the sealed baseline, max-regression 0.0) and the caf gate (rebuild, lint,
   test, audit). A module flips back to done only on awh GREEN and caf CLEAN. New behaviour adds
   golden-set tasks and the baseline is re-sealed once - a deliberate, reviewed step.

## What I cannot do here

This sandbox has no Rust, Docker, or Tauri toolchain, so it reads, evaluates, plans, vendors files, and
runs the Python suites, but the GREEN+CLEAN evidence for Rust or Docker modules is produced on your Mac.
Code is never marked done without that evidence.

## Map

- `playground/` (gitignored, repo root) - Stage 0 drop zone + evaluation. Disposable.
- `playground/CONSUMED-FROM-WORKBENCH.md` - the existing consumption ledger.
- `docs/absorptions/PLAYGROUND-TRIAGE.md` - the ranked worth-it shortlist across what is in playground/.
- `docs/absorptions/<name>-absorption.md` - per-graduated-source integration plan (Stage 1, tracked).
- `tools/<name>/` - vendored tooling or harnesses, each with its own gate.
