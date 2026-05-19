        ---
        skill_id: compensation-plan-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for compensation-plan-audit

        > Authored via heuristic backfill per FR-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Audit this compensation plan"
- "Check the compensation plan for completeness"
- "Verify the compensation plan meets the rubric"
- "Re-audit the compensation plan"

        ## Negative triggers (MUST NOT route here)

        - "Draft a compensation plan" → compensation-plan-author
- "Create the compensation plan" → compensation-plan-author
- "What is the team on-call rotation" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
