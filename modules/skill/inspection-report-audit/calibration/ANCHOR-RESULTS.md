# Anchor results: gam re-inspected at three fixed positions, spec held constant at 1.2

Target unchanged throughout: commit 699d795, 256 tracked files.

| Anchor | Slot | Findings | Strengths | Finding set | Stale-comment count |
|---|---|---|---|---|---|
| A1 | 1 | 6 | 6 | release-install, advisory-flag, unpinned-tools, stale-comments, no-disclosure, no-threat-model | 5 (undercount) |
| A2 | 5 | 6 | 6 | identical | 7 |
| A3 | 10 | 6 | 6 | identical | 7 |

## Result

Three independent derivations at slots 1, 5, and 10 produced the same six findings and the
same six strengths. Every quantitative measure agreed across A2 and A3: one unlocked
release install, two advisory flags, two unpinned tool installs, seven stale annotations,
no security policy, no threat model, and on the strength side one provenance action, one
bill-of-materials action, dependency automation present, nine token-scope declarations,
seven commit-pinned third-party actions, and one frozen-lockfile composite action.

The single disagreement is A1's stale-annotation count of five against seven at both later
positions. A1's count came from a listing truncated by a display limit. That is an A1
undercount, not a later gain, and it produced spec rule INS-DISC-13.

## Reading

The counterbalance was designed to separate two explanations for run 1's monotonic decline
in findings, 15 down to 7 across ten positions: the repositories genuinely improved, or the
inspector degraded across the batch.

If the inspector degraded, an unchanged target should yield fewer findings at slot 10 than
at slot 1. It yields the same six, with the later positions marginally more accurate than
the first. There is no degradation signal in this data.

That is evidence against the fatigue explanation and therefore for the repositories
genuinely differing, which is what run 1's cross-repo narrative assumed without being able
to show it. Three anchors on one target with one rater is a narrow basis, and the finding
sets being identical rather than merely similar is a stronger result than the counts
matching alone.

## Limits

One target, one rater, one model. This measures within-rater stability across position on
a small repository. It does not measure inter-rater reliability, which needs a second
rater, and it does not measure accuracy, which needs seeded defects. A larger or less
familiar target might degrade where this one did not.
