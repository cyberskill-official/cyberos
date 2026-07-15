# architectural-spike-author - failure modes

1. Spike becomes a design doc (unbounded) - INV-1/INV-5 make the timebox data, not
   vibes; SPK-BOX fails the audit when hours are absent or the HALT was skipped.
2. Evidence rot (a cited file later deleted) - the audit checks citations resolve AT
   AUDIT TIME; later rot belongs to the doc-anchor checker class (TASK-SKILL-119), not
   this skill.
3. Recommendation names an unprobed option - SPK-STRUCT-003 explicit rule.
4. Invoked with no real alternatives - blocker: do not fire; the ADR proceeds with
   evidence inline (lean fallback), keeping spikes for genuine forks.
5. Confidence inflation - `high` with thin evidence trips SPK-EVID-004; confidence is
   cross-checked against the evidence count, not self-declared.
