        ---
        skill_id: turnaround-plan-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for turnaround-plan-audit

        > Authored via heuristic backfill per FR-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Audit this turnaround plan"
- "Check the turnaround plan for completeness"
- "Verify the turnaround plan meets the rubric"
- "Re-audit the turnaround plan"

        ## Negative triggers (MUST NOT route here)

        - "Draft a turnaround plan" → turnaround-plan-author
- "Create the turnaround plan" → turnaround-plan-author
- "What is the team on-call rotation" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
