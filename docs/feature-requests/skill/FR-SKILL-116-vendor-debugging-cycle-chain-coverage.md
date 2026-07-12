---
id: FR-SKILL-116
title: "Vendor the debugging-cycle pair + chain-coverage check so no ship-referenced skill can be missing from the payload"
module: SKILL
priority: MUST
status: implementing
class: improvement
verify: T
phase: Wave B - finish the children
owner: Stephen Cheng (CTO)
created: 2026-07-12
shipped: null
memory_chain_hash: null
related_frs: [FR-SKILL-117, FR-SKILL-118, FR-CUO-209, FR-IMP-068]
depends_on: []
blocks: [FR-CUO-209]
source_pages:
  - modules/cuo/chief-technology-officer/workflows/ship-feature-requests.md
  - tools/cyberos-init/build.sh
source_decisions:
  - "2026-07-12 investigation: ship steps 25-26 invoke debugging-cycle-author/-audit, which exist at modules/skill/debugging-cycle-* but are absent from the hardcoded 20-skill vendored set at build.sh:28. The test-failure branch ships with no skill behind it in BOTH profiles."
  - "The fix must be structural, not a one-line patch: the vendored set must be provably derived-from or checked-against the workflow's skill_chain so the class of bug dies."
language: bash
service: tools/cyberos-init/
new_files:
  - tools/cyberos-init/check-chain-coverage.sh
  - tools/cyberos-init/chain-allowlist.txt
  - tools/cyberos-init/tests/test_chain_coverage.sh
modified_files:
  - tools/cyberos-init/build.sh
---

# FR-SKILL-116: Vendor debugging-cycle + chain-coverage check

## §1 - Description

The ship workflow's 31-step `skill_chain` is the contract for what the payload must carry, but the vendored skill list in `build.sh` is a hand-maintained string that silently drifted from it. This FR vendors the missing pair and adds a coverage check that fails the build whenever the two disagree.

Normative clauses:

1. The vendored skill set in `build.sh` MUST include `debugging-cycle-author` and `debugging-cycle-audit`, copied into both `cuo/skills/` and `plugin/skills/` like the existing pairs.
2. A script `tools/cyberos-init/check-chain-coverage.sh <payload-dir>` MUST extract every skill referenced by the vendored workflow docs and verify each has a directory containing a `SKILL.md` in BOTH `<payload>/cuo/skills/` and `<payload>/plugin/skills/`. Extraction is defined per doc kind: `skill: <name>` entries in `<payload>/cuo/ship-feature-requests.md`'s skill_chain, and backtick-quoted tokens matching `[a-z0-9]+(-[a-z0-9]+)*-(author|audit)` in `<payload>/plugin/commands/*.md` (deterministic token grammar, not prose guessing). Any miss MUST exit 10 listing `MISSING <skill> (referenced by <doc>)` per line.
3. Exemptions MUST be declared in `tools/cyberos-init/chain-allowlist.txt` (one name + one `# reason` per line; initial entries: `awh-gate`, `caf-gate`). An allowlist entry exempts its name from BOTH rule kinds - MISSING (script-backed steps with no skill dir) and UNPAIRED (intentionally single skills, e.g. the four NFR skills once FR-CUO-209 vendors them) - the reason string says which. An allowlisted name that nothing references and no payload dir matches MUST warn (stderr) so the allowlist cannot rot silently.
4. The check MUST also enforce pair completeness: for every vendored `<name>-author` there MUST be a vendored `<name>-audit` and vice versa; non-allowlisted violations exit 10 as `UNPAIRED <skill>`.
5. `build.sh` MUST run `check-chain-coverage.sh` against its own output as its final step and propagate a failure - a payload that under-covers its own workflow can no longer be produced.
6. The check MUST be pure read-only over the payload dir (no repo access needed), so FR-IMP-068's CI gate and FR-IMP-069's release job get it for free via the build.

## §4 - Acceptance criteria

1. **The pair is vendored** (§1 #1) - after `build.sh`, `debugging-cycle-author/SKILL.md` and `debugging-cycle-audit/SKILL.md` exist under both `cuo/skills/` and `plugin/skills/` in the payload.
2. **Chain extraction is real parsing, not a fixed list** (§1 #2) - adding a fake `skill: nonexistent-author` line to the workflow doc in a scratch payload makes the check exit 10 naming `nonexistent-author` and the doc.
3. **A dropped pair fails the build** (§1 #2, #5) - removing `debugging-cycle-author debugging-cycle-audit` from the vendored set makes `build.sh` itself exit non-zero with the two MISSING lines.
4. **Allowlist works both ways** (§1 #3) - `awh-gate`/`caf-gate` produce no failure; an allowlist entry naming a skill no doc references produces a stderr warning and exit stays 0.
5. **Pair completeness** (§1 #4) - a scratch payload with only `repo-context-map-author` present exits 10 with `UNPAIRED repo-context-map-author`.
6. **Read-only over the payload** (§1 #6) - running the check from an empty cwd against a copied payload dir succeeds; the payload's mtimes/bytes are unchanged after a run.

## §5 - Verification

```bash
# tools/cyberos-init/tests/test_chain_coverage.sh
# Builds one scratch payload, then mutates copies of it per case.

t01_pair_vendored()              # AC 1
t02_parses_chain_not_list()      # AC 2
t03_dropped_pair_fails_build()   # AC 3  (patched build.sh in a temp checkout)
t04_allowlist_both_ways()        # AC 4
t05_unpaired_detected()          # AC 5
t06_readonly_check()             # AC 6  (sha256 of payload tree before/after)
```

## §2 - Why this design

Deriving the vendored set FROM the chain automatically was considered and rejected: the payload legitimately carries skills no chain references yet (FR-CUO-209 vendors the full SDP set), so the honest relation is "chain is a subset of payload", enforced by a checker, with an explicit allowlist for script-backed steps. Checking pairs (#4) in the same pass kills the sibling bug class (author without audit) at zero extra cost.

## §3 - Contract

```
check-chain-coverage.sh <payload-dir>
  exit 0   chain covered, pairs complete (prints "chain OK: N referenced, M vendored, K allowlisted")
  exit 10  MISSING <skill> (referenced by <doc>) | UNPAIRED <skill>   (one per line)
  exit 2   payload dir or workflow doc unreadable
```

## §6 - Implementation skeleton

Extraction: `grep -Eo 'skill: *[a-z0-9-]+' <doc> | awk '{print $2}' | sort -u` over the chain doc + command docs; set-compare against `ls <payload>/cuo/skills/`; suffix swap for pair check. `build.sh` gains the pair in its set string and the final check invocation.

## §7 - Dependencies

None upstream. Blocks FR-CUO-209 (the expanded vendored set lands behind this checker). Related FR-IMP-068 (gate runs the build, so this check rides along) and FR-SKILL-118 (parity of pair CONTENTS; this FR guarantees pair PRESENCE).

## §8 - Example payloads

```
$ bash tools/cyberos-init/check-chain-coverage.sh dist/cyberos
MISSING debugging-cycle-author (referenced by cuo/ship-feature-requests.md)
MISSING debugging-cycle-audit (referenced by cuo/ship-feature-requests.md)
$ echo $?
10
```

## §9 - Open questions

None blocking. When FR-CUO-209 vendors additional workflow docs, clause #2's doc list grows by construction (it scans `plugin/commands/*.md` and the vendored cuo workflow file set, not a hardcoded pair).

## §10 - Failure modes inventory

1. Workflow doc renames its `skill:` key or format - extraction finds zero references; the check MUST treat "0 referenced skills" as exit 2 (structure changed under it), never as a pass.
2. Skill dir exists but is empty - presence test is `SKILL.md` inside the dir, not the dir itself.
3. Allowlist typo (`awhgate`) - the referenced `awh-gate` is then unmatched -> exit 10; typo cannot cause silent skips.
4. Case drift (`Debugging-Cycle-Author`) - names are compared lowercase-exact; a mismatch is a MISSING, surfacing the drift.
5. Payload built by an older build.sh (no check embedded) - CI (FR-IMP-068) runs the current checkout's check against the fresh build, so stale build scripts cannot bypass it on main.

## §11 - Implementation notes

Keep the output grep-stable (`MISSING `/`UNPAIRED ` prefixes); FR-CUO-209's expansion test keys on them. The allowlist file ships in the repo, not the payload - the check runs where the build runs.

*End of FR-SKILL-116.*
