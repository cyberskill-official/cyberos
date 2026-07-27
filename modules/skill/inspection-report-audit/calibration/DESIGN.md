# Run 2 design: counterbalanced re-inspection under spec 1.2

Purpose. Run 1 produced findings counts falling monotonically with position
(15, 17, 12, 10, 11, 9, 10, 9, 8, 7). That is equally consistent with the
repositories improving and with the inspector degrading, and a second
forward-ordered pass cannot separate them. This run counterbalances.

Anchor. gam, inspected at sequence positions 1, 5, and 10. gam sits mid
distribution in run 1 (57 applicable, 10 findings, 6 strengths) and is small
enough to re-inspect repeatedly. If anchor findings fall with position while the
repository is unchanged, the inspector degrades. If they hold, the run 1 decline
is a property of the repositories.

Order. The nine non-anchor repositories run in reverse of run 1.

| Slot | Target | Role |
|---|---|---|
| 1 | gam | anchor A1 |
| 2 | strategem | reverse 1 |
| 3 | shopass | reverse 2 |
| 4 | practice | reverse 3 |
| 5 | gam | anchor A2 |
| 6 | landing-page | reverse 4 |
| 7 | sach-viet | reverse 5 |
| 8 | kristen-calendar | reverse 6 |
| 9 | dom-defender | reverse 7 |
| 10 | gam | anchor A3 |
| 11 | issue-hunter | reverse 8 |
| 12 | my-cv | reverse 9 |

Controls. Fresh context per target (INS-FLOW-5). Batch position recorded in each
report (INS-FLOW-6). Same inspector, same tooling, spec 1.2 throughout.

Reading. Two comparisons come out of this. Anchor variance across positions 1,
5, and 10 answers the degradation question. Each repository's run 1 versus run 2
counts answer what spec 1.2 changed, and for the anchor both are available at
once, which is why the anchor runs first.

Limits. One inspector and one model. This measures within-rater stability across
position, not inter-rater reliability, which needs a second rater. It also does
not establish accuracy, which needs seeded defects. Both are named in the
research report as work this design does not do.
