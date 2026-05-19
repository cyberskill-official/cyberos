        ---
        skill_id: monthly-close-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for monthly-close-audit

        > Authored via heuristic backfill per FR-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Audit this monthly close"
- "Check the monthly close for completeness"
- "Verify the monthly close meets the rubric"
- "Re-audit the monthly close"

        ## Negative triggers (MUST NOT route here)

        - "Draft a monthly close" → monthly-close-author
- "Create the monthly close" → monthly-close-author
- "What is the team on-call rotation" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
