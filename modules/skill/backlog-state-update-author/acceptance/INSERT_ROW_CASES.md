# INSERT_ROW_CASES - executable case table for backlog-state-update@2 insert-row (FR-CUO-205 §5)

Each case: pre-image fixture -> mutation -> expected verdict (+ rule id). CASE-08 is the
byte-authority proof: delete an existing regenerated row, re-insert it per §2b (sorted by FR STEM, not row string - the status prefix would
reorder otherwise), and the file MUST be byte-identical to the original.

| case | fixture | mutation | expected |
|---|---|---|---|
| CASE-01 | module section exists | insert product row | pass (AC 3, 4) |
| CASE-02 | module section exists | insert class: improvement row | pass - ` (improvement)` suffix exact (AC 3) |
| CASE-03 | row for fr_id already present | insert same fr_id | fail BSU-INS-001 (AC 2) |
| CASE-04 | module has no section | insert row | pass - section created per regenerator conventions, sorted (AC 4) |
| CASE-05 | mutation also touches an unrelated line | insert row | fail BSU-INS-004 (AC 5) |
| CASE-06 | insert.status != FR frontmatter status | insert row | fail BSU-INS-005 (AC 6) |
| CASE-07 | @1 artefact, no mutation_kind | status-cell rewrite | pass with transition note (AC 1) |
| CASE-08 | live BACKLOG.md: remove one existing row, re-insert per §2b | insert row | pass AND `diff` empty vs the original file (AC 3 round-trip) |

Run CASE-08 concretely (the regenerator produced every current row, so re-insertion per §2b
must reproduce the file):

    python3 - <<'PY'
    import re, subprocess
    src = open('docs/feature-requests/BACKLOG.md').read().splitlines(keepends=True)
    # pick the first FR row, delete it, re-insert per §2b (sorted within its section)
    i = next(n for n, l in enumerate(src) if l.startswith('- ['))
    row = src.pop(i)
    # find section bounds again and insert at the sorted position
    # bounds of the section's CONTIGUOUS row block (walk rows only - the blank line
    # between the header and the first row stays outside the block)
    j = i
    while j > 0 and src[j-1].startswith('- ['):
        j -= 1
    k = j
    while k < len(src) and src[k].startswith('- ['):
        k += 1
    stem = lambda l: l.split('] ', 1)[1].split(' - ', 1)[0]
    body = sorted([*src[j:k], row], key=stem)
    out = src[:j] + body + src[k:]
    assert ''.join(out) == open('docs/feature-requests/BACKLOG.md').read(), 'round-trip diverged'
    print('CASE-08: byte-identical round-trip OK')
    PY
