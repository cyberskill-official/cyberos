# TASK-IMP-082 — code review packet

Files under review: `tools/docs-site/render-status-hub.mjs` (stamp derivation), new suite
`scripts/tests/test_render_stamp.sh`, and two one-assertion ripples in the peer suites
(disclosed below). Suite state at review: stamp 6/6, status-hub 10/10, roadmap 7/7, 0 failed.
Other dirt in the same working tree (`tools/install/*`) belongs to batch sibling
TASK-IMP-083 and is covered by that task's own packet.

## §1 clause → proof

| Clause | Requirement | Proof |
|---|---|---|
| 1.1 | default stamp = `fp-` + first 12 lowercase hex of sha256 over ordered inputs: every spec's raw bytes in sorted repo-relative path order, then CHANGELOG.md, then VERSION, when present | `t01_fingerprint_on_all_surfaces` — pins the shape (`^fp-[0-9a-f]{12}$`), then recomputes the value independently (`cat spec1 spec2 CHANGELOG VERSION \| sha256 \| first 12`; concatenation == the renderer's per-file updates) and requires equality. Absent-input arms live in `t02`: VERSION-only corpus, and the fully empty input set pinned to `fp-e3b0c44298fc` = sha256 of zero bytes — the constant proves no hidden input (path, clock) leaks in |
| 1.2 | `CYBEROS_COMMIT` set and non-empty overrides exactly as today | `t05_env_pin_wins` — `abc123` wins on all three surfaces; empty string falls through to `fp-` (the preserved `\|\|` semantics, edge-case-matrix row) |
| 1.3 | two consecutive renders over an unchanged corpus byte-identical (whole file) | `t02_double_render_stable` — `diff -r` over the whole output tree (stronger than the clause's single file), three renders across default locale, `LC_ALL=C`, `LC_ALL=C.UTF-8` (bytewise sort never consults the environment), plus the empty-corpus double render |
| 1.4 | render → commit page → render byte-identical; the page's own bytes are never a render input | `t03_commit_chase_ended` — scratch checkout, commit the rendered page, re-render, `cmp -s`. Structural half: `corpusFingerprint()` hashes `specFiles` + CHANGELOG + VERSION only; the out dir is never read |
| 1.5 | an input change moves the stamp on the next render; the render after that is stable again | `t04_corpus_edit_changes_once` — append one paragraph to a spec: stamp differs; second render after the edit is `diff -r` identical |
| 1.6 | default path never invokes git; a non-git directory gets the same fingerprint semantics | `t06_no_git_needed` — fixture with no `.git` yields a real `fp-` stamp (the old fallback would say `unknown`); a tripwire `git` first on PATH logs every invocation and exits 99, and the log must stay absent; the identical corpus inside a real checkout must render byte-identical pages. Structural half: `gitCommit()` deleted, no `child_process` import; `node:crypto` is the only new import |
| 1.7 | all three surfaces (header meta, footer, cs-data `commit`) carry the same value; no other page content changes | `t01_fingerprint_on_all_surfaces` asserts one value across `built from <span class="code">…</span>`, footer `($s)`, and `"commit":"$s"`; `t05_env_pin_wins` re-asserts all three under a pin. "No other content changes": both peer suites pass with only their stamp assertion repointed — every surrounding assertion (counts, lenses, facets, chunks, no-JS, token-clean) untouched and green |
| 1.8 | suite lands at `scripts/tests/test_render_stamp.sh`, discovered by the existing run_all glob | file at exactly that path, t01–t06 foot-called with the peers' counter/summary/exit contract; discovery is the runner's own glob `scripts/tests/test_*.sh` (run_all.sh:43) — zero wiring. The `run_all.sh` listing itself is AC 7's ops check, recorded in the batch parent's gate log |

## Acceptance criteria

AC 1 `t01_fingerprint_on_all_surfaces` ok · AC 2 `t02_double_render_stable` ok ·
AC 3 `t03_commit_chase_ended` ok · AC 4 `t04_corpus_edit_changes_once` ok ·
AC 5 `t05_env_pin_wins` ok · AC 6 `t06_no_git_needed` ok ·
AC 7 run_all discovery (ops check in the batch parent's gate log).

## Diff size

Production surface is one file: `render-status-hub.mjs` +21/−18 (net +3 — `gitCommit()`
−13, `corpusFingerprint()` +11, the :304 comment rewritten in place, one header-claim line,
one `node:crypto` import, two lines collecting `specFiles` in the existing discovery loop).
Tests: one new 142-line suite plus 3 changed lines across the two peer suites (+1/−1 and
+2/−1 — the latter only because one assertion line was split for length). Task numstat total:
+24/−20 across 3 modified files, 1 new file. No new dependencies; no caller, template, or
hook changed. `dist/` untouched here — rebuild, version-sync and full suite before commit
are the batch parent's step per payload-sync doctrine.

## Peer-suite modifications (disclosure)

Both peer fixtures plant a fake checkout — `echo "abcdef1234567890" > "$d/.git/refs/heads/main"` —
and each had exactly one assertion pinning `abcdef123456`, the first 12 chars of that planted
HEAD. That is the old derivation itself (`COMMIT = … || gitCommit(ROOT)`), the thing §1.1/§1.6
delete, so under the new default those two greps fail by design. Changed minimally:

- `tools/docs-site/tests/test_render_status_hub.sh` — `t01_deck_true`, one assertion:
  - was: `&& grep -q 'VERSION <span class="code">2.0.0</span>' "$h" && grep -q 'abcdef123456' "$h" \`
  - now: `&& grep -q 'VERSION <span class="code">2.0.0</span>' "$h"` and, on its own line,
    `&& grep -Eq 'built from <span class="code">fp-[0-9a-f]{12}</span>' "$h" \`
- `tools/docs-site/tests/test_render_roadmap.sh` — `t02_board_counts_and_release_order`, one assertion:
  - was: `grep -q 'VERSION' "$h" && grep -q 'abcdef123456' "$h" \`
  - now: `grep -q 'VERSION' "$h" && grep -Eq 'built from <span class="code">fp-[0-9a-f]{12}</span>' "$h" \`

The replacements are strictly tighter: the old greps matched the sha anywhere in the page;
the new ones anchor to the `built from` meta and its grammar. The planted `.git` fixtures
were deliberately left in place — they now prove the planted HEAD does NOT leak into the
page (the `fp-` assertion would fail if it did), i.e. the peer suites carry a free copy of
1.6's semantics. Nothing else in either file changed; both suites' `t05_deterministic`
byte-compares were already derivation-agnostic and keep gating 1.3 from outside this task.
This ripple was declared in impl-plan slice 4; no production surface is involved.

## Implementer notes / issues for the reviewer

- Hash choice per edge-case matrix: sha256 via `node:crypto` `createHash`, not homegrown;
  streaming per-file `h.update`, no concat buffer (the large-corpora inspection note).
- Path order is `Buffer.compare` on the repo-relative path — locale-independent by
  construction; `t02` exercises C and C.UTF-8 anyway.
- CRLF / trailing-whitespace edits move the stamp by definition (content is the contract);
  documented in the suite header so nobody "fixes" it.
- ISS-1 (accepted semantics, flagging for the record): `specFiles` is collected before the
  frontmatter parse, so in `CYBEROS_HUB_LENIENT=1` mode a spec whose frontmatter fails still
  counts toward the fingerprint while being excluded from the corpus — the stamp can move
  without visible page change. Conforms to §1.1 ("every task spec file's raw bytes") and
  errs toward over-invalidation, never staleness; strict mode dies on such a file anyway.
- The fingerprint identifies the input corpus, not the output bytes — that is the freshness
  check the old :304 comment asked for ("compare CONTENT"), per the spec's rationale.
- Protected invariants re-checked: no wall clock, no randomness, no environment-dependent
  output beyond the documented pin; the page stays a pure function of the corpus.

## Verdict

| Area | Verdict |
|---|---|
| §1 conformance (1.1–1.8) | pass |
| ACs 1–6 | pass (stamp suite 6/6; AC 7 pending parent's run_all gate log) |
| Byte-stability contract (re-render, chase case) | pass |
| Peer-suite ripple | disclosed; 3 lines, assertions strictly tightened, fixtures intact |
| Invariants (§5) | intact |

HALT: review acceptance (reviewing -> ready_to_test) is a human gate.
