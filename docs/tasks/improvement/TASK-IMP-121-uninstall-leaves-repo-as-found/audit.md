---
task_id: TASK-IMP-121
audited: 2026-07-18
verdict: FAIL
score: 4/10
issues_open: 6
issues_resolved: 0
template: task@1
audit_rubric_version: audit_rubric@2.0
machine_floor: task-lint clean (exit 0) - run FIRST per TASK-IMP-084. The floor passing is why this
  audit matters: every finding below is a JUDGMENT defect the mechanical check cannot see.
auditor: independent subagent (had not seen the author's reasoning) + author verification of the
  two load-bearing claims against source
---

## §1 - Verdict summary

FAIL at 4/10. The four underlying defects are REAL and reproduced by the harness. The spec
diagnoses two of them from a false premise, carries the TASK-IMP-118 defect class in its own AC 1,
and cites six line numbers that do not resolve. Not a patch: a rewrite against the true mechanism.

## §2 - Findings (ALL OPEN)

### ISS-001 (CRITICAL) - the premise ".cyberos is removed" is false in the default path
§1.1/§1.3 are written against "the removed `.cyberos/`". VERIFIED against source: `uninstall.sh:151`
runs `rm -rf "$CY"` and `:155-158` then runs `mkdir -p "$root/.cyberos/memory"` +
`mv "$KEEP_BRAIN_STASH" "$root/.cyberos/memory/store"`. **`.cyberos/` survives** with BRAIN inside
unless `CYBEROS_UNINSTALL_KEEP_BRAIN=0`. The harness's own fresh-git run listed
`./.cyberos/memory/store/*` among the leftovers - the author had the evidence and wrote the clause
against the opposite state.

### ISS-002 (CRITICAL) - AC 1 asserts a different predicate than §1.1 (the TASK-IMP-118 defect)
§1.1 verb: a symlink "may remain [only if its] `readlink` target [does not resolve] inside the
removed `.cyberos/`". AC 1 asserts: "zero unresolvable targets". These diverge BOTH ways:
- false negative: with BRAIN restored (ISS-001), a symlink into a SURVIVING `.cyberos/` path
  resolves fine -> passes AC 1 -> violates §1.1.
- false positive: an operator's own broken symlink pointing OUTSIDE `.cyberos/` is unresolvable
  -> fails AC 1 -> but §1.2 requires it be KEPT. AC 1 and AC 2 contradict on that input.
The author wrote both the clause and its test, and the test is weaker. This is precisely §15.2.

### ISS-003 (CRITICAL) - §1.3/§1.4 are undecidable; the enabling mechanism is rejected in-spec
Both condition on "if install created it". Nothing on disk records that: `install.sh` tracks
creation in a shell variable that dies with the process; the only ownership marker
(`.cyberos-owned`) is written ONLY into skill copy dirs, never onto `.gitignore` or `.mcp.json`.
The spec's Proposed Solution permits only "readlink target inside `.cyberos/`, or our marker" -
`readlink` is meaningless on a regular file and no marker exists - while Alternatives REJECTS
"Track install's creations in a manifest" as scope creep. The spec forbids the only mechanism
that would make its own clauses decidable.

### ISS-004 (CRITICAL) - §1.4 contradicts §1.7 and the zero-casualties guardrail
An operator who ran `touch .gitignore` before install is byte-indistinguishable from install's
`: > "$gi"`. After block-strip the file is empty; §1.4 says it MUST NOT survive -> uninstall
deletes an operator's file -> §1.7 ("No path present before install may be missing") violated.
§1.7 admits no exception and §3 does not cover the case. Same hazard for `.mcp.json`: install
points operators at `.cyberos/mcp/README.md` for hand-registration, so an operator-authored
`.mcp.json` is a supported path.

### ISS-005 (MAJOR) - section 6 already ACCEPTS the dangling links; the spec frames it as oversight
VERIFIED: `uninstall.sh:162` reads `# 6. skill symlinks into .cyberos (dangling) - leave dirs;
operator cleans`. The dangling is KNOWN and DELIBERATE in source. The spec's Problem section says
four channels "are never looked at" - true of the code path, false of the intent. This is a
recorded decision to OVERTURN with reasons, not a gap to fill. The operator approved overturning
it (2026-07-18) on the readlink-proves-ownership argument, which stands - but the spec must argue
against section 6 explicitly rather than not notice it.

### ISS-006 (MAJOR) - six citations do not resolve
- `:70` cited as "(block strip)" and as the proven newline leak -> `:70` is
  `echo "  stripped cyberos block from pre-commit"`. The sed is `:68`. This is the spec's single
  most load-bearing citation and it points at the echo, not the mechanism.
- `:81` cited as the .gitignore strip -> `:81` is the echo; the strip is `:80`.
- `:114` cited BOTH as the ship-tasks exemption AND as the readlink test it exempts from. One line
  cannot be both. The `[ "$_sc" != "ship-tasks" ]` guard is `:112`.
- `:111` cited as an exemption whose rationale is "avoid clobbering operator files" -> `:111` is a
  comment; it records no such rationale.
- `:105-107` cited as a prune precedent -> it is the "kept unmarked skill dir" echo branch. The
  actual prune is `:118-119` and has NO install-created check.
- `:92-93` resolves but is a comment naming 3 paths, cited to cover 7.

## §3 - Verified accurate (credit)
- `uninstall.sh` never mentions mcp/codex/grok/commandcode/opencode - grep returns zero. The
  five-channel and dead-registration findings are REAL.
- The +1-newline-per-cycle diagnosis is mechanically correct and fixable from `uninstall.sh` alone:
  install appends `\n# >>> cyberos-status-hook v2`, and `:68`'s sed range starts AT the marker, so
  the blank separator above it survives.
- 121's reading of TASK-IMP-106 AC 3 is accurate; `depends_on: [TASK-IMP-106]` correctly serialises.
- TRACE-003 passes: `test_install_hygiene.sh` exists and is declared in modified_files.

## §4 - Required before re-audit
Rewrite §1.1/§1.3 against the surviving-.cyberos reality; re-derive AC 1 to assert §1.1's actual
predicate; resolve ISS-003 by either scoping the creation-manifest IN or narrowing the clauses to
what readlink/marker can decide; reconcile §1.4 with §1.7; argue against section 6 explicitly;
fix all six citations. Residual noted: after 121 re-points t22, TASK-IMP-106 §1.5 has no test left
that verifies it, and a test named `t22_uninstall_behavior_unchanged` whose baseline moved asserts
the opposite of its name.
