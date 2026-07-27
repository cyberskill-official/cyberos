# inspection_report_rubric@1.0

Ten points. A report scores 10 or it routes back. Cite rule ids in findings;
never paraphrase them.

## Gate 0: machine floor (pass/fail, not scored)

`node tools/inspect-lint.mjs <report.md>` must exit 0. Warnings are advisory and
do not block, but a warning left unexplained in the audit write-up is itself an
IRA-006 finding. Any error is an automatic fail and scoring stops.

The linter's rule families are the mechanical contract and are not restated
here: INSL-STR (report structure, gate lines, claim-evidence ratio), INSL-LED
(coverage ledger completeness, order, applicability, states, counts, pointers),
INSL-FND (finding schema, enums, identifiers, fingerprints, verbatim quotes,
placeholders), INSL-NXT (the single next action), INSL-CRS (findings against
ledger reconciliation).

## Scored rules

One point each unless stated. A rule scores zero if any instance fails.

### IRA-001 Applicability honesty (1)

Every NOT APPLICABLE row states a reason specific to this target, not a generic
one. "No database" against a repository with a schema directory fails. A reason
that would be equally true of any repository fails. Conditional rows
(SEC-05, DELIVERY-07, GOV-06, AIML-01) must say which condition is unmet.

### IRA-002 Evidence state matches evidence (2)

Worth two points because it is the rule the whole artefact rests on.

VERIFIED requires a quote that actually demonstrates the claim, not a quote
merely from the same file. STRONG EVIDENCE requires stated aggregate evidence
and an explicit note of what was not observed. SUSPECTED requires a named reason
for the doubt. VERIFIED_ABSENT requires evidence of looking, not just absence of
a hit. BLOCKED requires the blocked validation named in section 6. Any finding
whose state overstates what section 5 says was actually run fails this rule.

### IRA-003 Section 6 is real (1)

Limitations name specific unexamined surfaces and specific unrun validations,
each with the reason. "Some files were not read in full" fails. A report whose
section 6 is shorter than its executive summary is suspect and must be checked
against the phase list in section 5.

### IRA-004 Clusters are causal (1)

Root-cause clusters in section 11 group findings by shared cause, not by shared
severity, discipline, or file. Each cluster states the cause in one sentence and
says what a single fix at the cause would close. A cluster that is a list of
findings with a heading fails. Findings outside every cluster are named as
independent, not silently omitted.

### IRA-005 Severity ladder is consistent (1)

Severity follows the stated band definitions, and two findings with comparable
impact and likelihood in the same report carry the same severity. Cross-check
the Critical and High summary against the register: a finding described in
section 10 as the most serious must not be outranked by another finding's
severity. Latent defects, where the exploit path depends on a change that has
not happened, are rated for what is true today with the condition stated.

### IRA-006 Uncertainty is surfaced, not smoothed (1)

Open questions in section 21 include anything the report could not determine
that changes a verdict or a severity. Every finding whose severity is
conditional on an unknown carries that unknown in its open_questions. A report
with no open questions across more than ten findings fails unless it states why.

### IRA-007 The next action is the right one (1)

The single next action is defensible as the highest-value starting point on the
stated reasoning, not merely the first Critical or the first row in the
register. The justification names why it is first relative to the alternatives.
Its acceptance criteria are checkable without re-reading the report.

### IRA-008 Strengths are evidenced, not flattery (1)

Every strength record carries the same evidence discipline as a finding,
including a quote for VERIFIED. A strength that restates the project's own
documentation without independent evidence fails. Zero strengths in a report
with more than ten findings is itself a finding about the inspection, not about
the target.

### IRA-009 Read-only discipline held (1)

Section 1 discloses side effects or states none, section 5 lists the commands
actually run, and the two agree. Any command in section 5 that could mutate the
target, install a dependency, execute target code, or test a discovered
credential fails this rule regardless of what section 1 claims.

## Scoring

Sum the scored rules for a maximum of 10. Report as `score / 10` with a
per-rule line. On anything below 10, list the failing rule ids with the quoted
passage that failed, and route back.

## Change control

Changing a rule id's meaning is a major version. Adding a rule is a minor
version and requires re-running the five acceptance fixtures. Any rubric change
that fails a fixture is a contract change and the fixture is the evidence, not
the problem.
