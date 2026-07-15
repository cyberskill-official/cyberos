# architectural-spike-author - pipeline

P0 intake: validate inputs (audited task, repo-context-map, one question, integer
timebox). Refuse a multi-question spike - one spike, one decision. Record
timebox_hours in the draft artefact BEFORE any probing (INV-1). Allocate spike_id.

P1 probe: per option - state the hypothesis; gather evidence (repo reads, commands
with captured output, external references); record cost_estimate + risks. Stop
adding options when the marginal option has no distinct hypothesis.

P2 decide: name exactly one recommendation with confidence (low/medium/high per the
evidence rule); move every rejected option into the discard log with a reason.

P3 reconcile: record actual_hours. HALT point: actual > 1.5x timebox -> present the
three-way operator choice (extend / force / discard) and record the verdict.

P4 emit: write the artefact, emit the architectural_spike_authored audit row, hand
off to architectural-spike-audit, then architecture-decision-record-author.

Halt points: P3 over-budget (mandatory operator verdict); P0 refusal when the
question is not a real fork (single obvious option) - route to the ADR lean fallback.
