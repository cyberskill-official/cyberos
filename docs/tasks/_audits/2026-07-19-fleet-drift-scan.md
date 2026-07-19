# Fleet drift scan - 2026-07-19

Read-only scan of every installed `.cyberos` machine under `~/Projects` against the
current platform source at HEAD `070bcaa5` (branch `review/imp-hardening`, after the
11-task handoff batch landed). No writes to any install; this note records the finding.

## Method

Each install carries a vendored copy of the platform machine at `<repo>/.cyberos/`
(gitignored in every install, so drift is content divergence, not git state). Three
signatures were hashed per install and compared to the platform source:

- `.cyberos/docs-tools/*.mjs` basename-set signature (install generation proxy)
- `.cyberos/install.sh` sha (vs source `tools/install/install.sh`, sha `839dd576`)
- `.cyberos/manifest.yaml` sha

`.cyberos/install.sh` was confirmed to be a copy of the source `install.sh` (first 20
lines byte-identical), so a hash gap is a genuine version gap, not two unrelated files.

## Inventory

24 installed `.cyberos` machines, one source repo (`CyberSkill/cyberos`). Every install
stamps `VERSION 1.0.0`, matching the platform - so version-string drift is zero and the
real drift is in the machine payload, which the version stamp does not track.

## Finding 1 - the whole fleet is behind the platform source

No install's `install.sh` matches the source sha `839dd576`, including the dogfood
self-install `cyberos/.cyberos` (`52e31e0a`). The self-install trails source by 29
`install.sh` lines, among them the four-line `skill-trust.tsv` seed shipped by IMP-113
this run (present in source, absent in the self-install). Expected: nothing has been
re-installed since the batch landed. Remedy is Goal 4 (re-install everywhere from the
current platform).

## Finding 2 - three install generations, 22 of 24 on the oldest

Bucketed by docs-tools set signature:

| generation | installs | #docs-tools | install.sh | manifest |
|---|---|---|---|---|
| 217b15b3 (oldest) | 22 | 2 (md, render-status-hub) | 1088117c | 03463bbe |
| db6abf5e (sachviet) | 1 | 8 | 9b431495 | 397636ef |
| eb8eafeb (self-install) | 1 | 11 | 52e31e0a | e10bee27 |

The 22 oldest-generation installs carry only two docs-tools scripts; sachviet and the
self-install carry richer, newer machines but are themselves still behind source. The
22 span `CyberSkill/*`, `Hackathon/quote-mind`, and `Personal/*`.

## Note for Goal 5 (sachviet)

sachviet's installed machine (`db6abf5e`, install.sh `9b431495`) is a middle generation,
stale versus source. Proving the loop end-to-end on sachviet via its installed `.cyberos`
should re-install sachviet to the current platform first, or the loop would run an old
machine that predates the handoff batch (no `next-id`, no truth-guard, no cone-audit,
no skill-log).

## Scope note - repo-side tools are not fleet drift

The platform's `tools/install/docs-tools/` holds 12 tools, but three of them
(`skill-log.mjs`, `cone-audit.mjs`, `fm001-migrate.mjs`) are repo-side tools with no
"ships in payload" clause and are deliberately not vendored into `.cyberos`. Their
absence from installs is correct, not drift. The install machine also carries `md.mjs`
and `render-status-hub.mjs`, which are install-side and not in that source directory.
Comparing the two directories directly is an apples-to-oranges error and was corrected
during this scan.
