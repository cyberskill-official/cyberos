# Seeded-defect calibration for /inspect specification 1.2

Two experiments, both on an isolated copy of a 256-file repository. Neither is a
general accuracy claim; both are bounded and the bounds are stated.

## Experiment 1: recognition

Twelve defects drawn at random from a fifteen-entry catalogue, injected by script,
answer key sealed until detection was complete. Detections and explicit
non-detections were written down before the key was opened.

| Measure | Result |
|---|---|
| Recall | 12/12 |
| Precision | 12/12, no detection failed to match a seeded defect |
| True negatives | 2/2, both unseeded catalogue entries correctly not reported |

The two entries whose preconditions were absent, a deleted token scope and a widened
toolchain pin, were explicitly recorded as not present rather than reported. That is
the result worth keeping: the method did not invent defects to fill its own catalogue.

Bounds. The catalogue was written by the same author who ran detection, so the defect
types were known even though their selection and placement were not. Every seeded
defect is a single-line textual change with a clear signature. Real defects include
logic errors, race conditions, and design flaws that no text search finds. A recall of
1.00 here means the mechanical layer works on mechanical defects in a small familiar
repository, and nothing more.

## Experiment 2: search scope

Five defects placed in locations a conventionally scoped sweep does not reach: a
nested deployment defaults file, a dated legacy migration directory, a non-standard
continuous-integration directory, a vendored source path, and archived documentation.

| Strategy | Recall |
|---|---|
| Scoped sweep, conventional paths only | 0/5 |
| Enumerate first, then search, per INS-DISC-13 and INS-EVD-9 | 5/5 |

This is the experiment that matters, because it reproduces the failure that actually
happened. All three false positives confirmed in run 2 were absence claims from an
incomplete search rather than failures to recognise a defect. The most serious, a
Critical asserting no row-level security, was wrong because nineteen migrations sat in
a subdirectory the original pass never entered.

A scoped sweep scored zero. The same reader, enumerating before searching, scored five
of five on the same tree. The difference is entirely procedural.

## What these results support, and what they do not

Supported: the recognition layer is sound, the method does not fabricate defects to
match a catalogue it knows, and the enumerate-before-searching rules added in 1.2
close the specific gap that produced every confirmed false positive in run 1.

Not supported: any general precision or recall figure. One target, one rater, one
model, seventeen synthetic defects, all textually detectable. Inter-rater reliability
is still unmeasured and needs a second rater. Accuracy against real defects is still
unmeasured and needs a labelled corpus or retrospective validation against fixed
issues in commit history.

Quality headers should continue to read "calibration: uncalibrated" for real targets.
What can now be said is narrower and true: the method was tested against seventeen
planted defects and found all seventeen once it enumerated before searching.
