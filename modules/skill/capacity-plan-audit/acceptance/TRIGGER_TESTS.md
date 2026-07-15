        ---
        skill_id: capacity-plan-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for capacity-plan-audit

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Audit this capacity plan"
- "Check the capacity plan for completeness"
- "Verify the capacity plan meets the rubric"
- "Re-audit the capacity plan"

        ## Negative triggers (MUST NOT route here)

        - "Draft a capacity plan" → capacity-plan-author
- "Create the capacity plan" → capacity-plan-author
- "What is the team on-call rotation" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
